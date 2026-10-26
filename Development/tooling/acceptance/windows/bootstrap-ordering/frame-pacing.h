#pragma once
#include <windows.h>
#include <atomic>
#include <limits>
#include <stdexcept>

// One rendering consumer, policy changes from another thread; join before destruction.
// No game hook, simulation scheduling, process throttling or global timer change.
namespace presentation_pacing {
enum class Outcome { Frame, Stopped, Error };
struct Permit {
    Outcome outcome;
    unsigned ceiling;
    LONGLONG released;
};

// Trusted producer contract: manual-reset level/stop, auto-reset changed, and
// the controller PROCESS handle. The consumer duplicates SYNCHRONIZE rights;
// the capability decoder must establish object types/identity before handing off.
struct Input { HANDLE level = nullptr, changed = nullptr, stop = nullptr, controller = nullptr; };
class Pacer {
    enum class Mode { Background, Foreground, Stopped };
    std::atomic<Mode> mode_{Mode::Background};
    HANDLE changed_ = nullptr, timer_ = nullptr;
    LONGLONG frequency_ = 0, previous_ = 0;
    HANDLE external_[4]{};
    void close() noexcept {
        if (timer_) CloseHandle(timer_);
        if (changed_) CloseHandle(changed_);
        for (HANDLE handle : external_) if (handle) CloseHandle(handle);
    }
public:
    explicit Pacer(const Input& input = {}) {
        try {
            const HANDLE source[]{input.level, input.changed, input.stop, input.controller};
            unsigned count = 0;
            for (HANDLE handle : source) {
                if (reinterpret_cast<INT_PTR>(handle) < 0) throw std::runtime_error("Pseudo policy handle refused");
                if (handle) ++count;
            }
            if (count && count != 4) throw std::runtime_error("Incomplete policy input");
            if (count) {
                if (!GetProcessId(input.controller)) throw std::runtime_error("Missing controller process");
                using Compare = BOOL (WINAPI*)(HANDLE, HANDLE);
                const auto compare = reinterpret_cast<Compare>(GetProcAddress(GetModuleHandleW(L"kernelbase.dll"), "CompareObjectHandles"));
                if (!compare) throw std::runtime_error("Policy identity comparison unavailable");
                for (unsigned i = 0; i < 4; ++i) {
                    for (unsigned j = 0; j < i; ++j)
                        if (source[i] == source[j] || compare(source[i], source[j])) throw std::runtime_error("Aliased policy input");
                    if (!DuplicateHandle(GetCurrentProcess(), source[i], GetCurrentProcess(), &external_[i],
                        SYNCHRONIZE, FALSE, 0)) throw std::runtime_error("Policy read capability unavailable");
                }
            }
            LARGE_INTEGER frequency{};
            if (!QueryPerformanceFrequency(&frequency) || frequency.QuadPart <= 0
                || frequency.QuadPart > (std::numeric_limits<LONGLONG>::max)() / 20000000)
                throw std::runtime_error("Unsupported pacing clock");
            frequency_ = frequency.QuadPart;
            changed_ = CreateEventW(nullptr, FALSE, FALSE, nullptr);
            timer_ = CreateWaitableTimerExW(nullptr, nullptr, CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
                TIMER_MODIFY_STATE | SYNCHRONIZE);
            if (!changed_ || !timer_) throw std::runtime_error("Pacing synchronization unavailable");
        } catch (...) { close(); throw; }
    }
    Pacer(const Pacer&) = delete;
    Pacer& operator=(const Pacer&) = delete;
    ~Pacer() { close(); }

    bool foreground(bool active) {
        if (external_[3]) return false; // One policy producer for this consumer.
        Mode expected = mode_.load(std::memory_order_acquire);
        const Mode desired = active ? Mode::Foreground : Mode::Background;
        for (;;) {
            if (expected == Mode::Stopped) return false;
            if (expected == desired) return true;
            if (mode_.compare_exchange_weak(expected, desired, std::memory_order_acq_rel))
                return SetEvent(changed_) != FALSE;
        }
    }
    bool stop() {
        mode_.store(Mode::Stopped, std::memory_order_release);
        return SetEvent(changed_) != FALSE;
    }
    Permit wait_for_frame() {
        HANDLE waits[]{external_[2], external_[3], external_[1], changed_, timer_};
        const bool external = external_[3] != nullptr;
        for (;;) {
            Mode mode = mode_.load(std::memory_order_acquire);
            if (mode == Mode::Stopped) return {Outcome::Stopped, 0, 0};
            if (external) {
                for (unsigned i = 0; i < 2; ++i) {
                    const DWORD state = WaitForSingleObject(waits[i], 0);
                    if (state == WAIT_OBJECT_0) { stop(); return {Outcome::Stopped, 0, 0}; }
                    if (state != WAIT_TIMEOUT) return {Outcome::Error, 0, 0};
                }
                const DWORD level = WaitForSingleObject(external_[0], 0);
                if (level != WAIT_OBJECT_0 && level != WAIT_TIMEOUT) return {Outcome::Error, 0, 0};
                mode = level == WAIT_OBJECT_0 ? Mode::Foreground : Mode::Background;
            }
            const unsigned ceiling = mode == Mode::Foreground ? 144 : 10;
            LARGE_INTEGER now{};
            if (!QueryPerformanceCounter(&now) || now.QuadPart < previous_)
                return {Outcome::Error, 0, 0};
            const LONGLONG interval = (frequency_ + ceiling - 1) / ceiling;
            const LONGLONG elapsed = now.QuadPart - previous_;
            if (!previous_ || elapsed >= interval) {
                // Never repay missed deadlines with a catch-up burst.
                previous_ = now.QuadPart;
                return {Outcome::Frame, ceiling, previous_};
            }
            const LONGLONG remaining = interval - elapsed;
            LARGE_INTEGER due{};
            due.QuadPart = -(remaining * 10000000 + frequency_ - 1) / frequency_;
            if (!SetWaitableTimerEx(timer_, &due, 0, nullptr, nullptr, nullptr, 0))
                return {Outcome::Error, 0, 0};
            const DWORD count = external ? 5 : 2;
            const DWORD result = WaitForMultipleObjects(count, external ? waits : waits + 3, FALSE, INFINITE);
            if (result >= WAIT_OBJECT_0 + count)
                return {Outcome::Error, 0, 0};
            // A focus event interrupts the old deadline; timer wakes recheck QPC.
        }
    }
};
} // namespace presentation_pacing
