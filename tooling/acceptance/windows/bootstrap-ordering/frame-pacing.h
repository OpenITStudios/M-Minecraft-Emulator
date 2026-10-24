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

class Pacer {
    enum class Mode { Background, Foreground, Stopped };
    std::atomic<Mode> mode_{Mode::Background};
    HANDLE changed_ = nullptr, timer_ = nullptr;
    LONGLONG frequency_ = 0, previous_ = 0;
public:
    Pacer() {
        LARGE_INTEGER frequency{};
        if (!QueryPerformanceFrequency(&frequency) || frequency.QuadPart <= 0
            || frequency.QuadPart > (std::numeric_limits<LONGLONG>::max)() / 20000000)
            throw std::runtime_error("Unsupported pacing clock");
        frequency_ = frequency.QuadPart;
        changed_ = CreateEventW(nullptr, FALSE, FALSE, nullptr);
        timer_ = CreateWaitableTimerExW(nullptr, nullptr, CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
            TIMER_MODIFY_STATE | SYNCHRONIZE);
        if (!changed_ || !timer_) {
            if (changed_) CloseHandle(changed_);
            if (timer_) CloseHandle(timer_);
            throw std::runtime_error("Pacing synchronization unavailable");
        }
    }
    Pacer(const Pacer&) = delete;
    Pacer& operator=(const Pacer&) = delete;
    ~Pacer() { CloseHandle(timer_); CloseHandle(changed_); }

    bool foreground(bool active) {
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
        HANDLE waits[]{changed_, timer_};
        for (;;) {
            const Mode mode = mode_.load(std::memory_order_acquire);
            if (mode == Mode::Stopped) return {Outcome::Stopped, 0, 0};
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
            const DWORD result = WaitForMultipleObjects(2, waits, FALSE, INFINITE);
            if (result != WAIT_OBJECT_0 && result != WAIT_OBJECT_0 + 1)
                return {Outcome::Error, 0, 0};
            // A focus event interrupts the old deadline; timer wakes recheck QPC.
        }
    }
};
} // namespace presentation_pacing
