#include "fps-hud.h"
#include "hud-canvas-ffi.h"
#include "hud-window-ffi.h"
#include <algorithm>
#include <atomic>
#include <cmath>
#include <cwchar>
#include <stdexcept>
#include <thread>

namespace fps_hud {
// Native filter, clock and synchronization stay at the Windows boundary.
void Meter::reset() noexcept {
    AcquireSRWLockExclusive(&gate_);
    lazytester_fps_meter_reset(&state_);
    ReleaseSRWLockExclusive(&gate_);
}
void Meter::record(LONGLONG now, HRESULT result, UINT flags) noexcept {
    if (result != S_OK || (flags & DXGI_PRESENT_TEST) || now <= 0) return;
    AcquireSRWLockExclusive(&gate_);
    lazytester_fps_meter_record(&state_, now);
    ReleaseSRWLockExclusive(&gate_);
}
Reading Meter::read(LONGLONG now) noexcept {
    AcquireSRWLockShared(&gate_);
    if (!now) { LARGE_INTEGER value{}; if (QueryPerformanceCounter(&value)) now = value.QuadPart; }
    LazyTesterFpsMeterReading value{};
    lazytester_fps_meter_read(&state_, now, &value);
    ReleaseSRWLockShared(&gate_);
    return {value.accepted, value.milliFps, value.valid != 0};
}
RECT placement(RECT client, UINT dpi) noexcept {
    RECT result{}; lazytester_hud_placement(&client, dpi, &result); return result;
}
bool physicalClient(HWND owner, RECT& result) noexcept {
    return lazytester_hud_physical_client(owner, &result) != 0;
}
namespace {
struct Canvas {
    LazyTesterHudCanvas* state = lazytester_hud_canvas_create();
    HDC dc = nullptr;
    const DWORD* pixels = nullptr;
    Canvas() = default;
    Canvas(const Canvas&) = delete;
    Canvas& operator=(const Canvas&) = delete;
    ~Canvas() { lazytester_hud_canvas_destroy(state); }
};
LONGLONG now() noexcept { LARGE_INTEGER value{}; return QueryPerformanceCounter(&value) ? value.QuadPart : 0; }
void moduleAnchor() {}
void paint(Canvas& canvas, Reading value, UINT dpi, SIZE& size) {
    LazyTesterHudCanvasView view{};
    if (!lazytester_hud_canvas_paint(canvas.state, value.milliFps, value.valid ? 1U : 0U, dpi, &view))
        throw std::runtime_error("HUD raster unavailable");
    canvas.dc = view.dc; canvas.pixels = view.pixels; size = view.size;
}
}
std::vector<DWORD> raster(Reading value, UINT dpi, SIZE& size) {
    Canvas canvas; paint(canvas, value, dpi, size);
    return {canvas.pixels, canvas.pixels + static_cast<size_t>(size.cx) * size.cy};
}
bool selectionAllowsHud(bool managed, HANDLE hudSelected, HANDLE policyStop, HANDLE controller) noexcept {
    return !managed || (hudSelected && policyStop && controller
        && WaitForSingleObject(hudSelected, 0) == WAIT_OBJECT_0
        && WaitForSingleObject(policyStop, 0) == WAIT_TIMEOUT
        && WaitForSingleObject(controller, 0) == WAIT_TIMEOUT);
}
struct Hud::State {
    Meter meter;
    SRWLOCK gate = SRWLOCK_INIT;
    HWND owner = nullptr;
    DWORD ownerThread = 0;
    HANDLE ownerLife = nullptr;
    HANDLE hudSelected = nullptr, policyStop = nullptr, controller = nullptr;
    bool managed = false;
    bool selectionAdmitted() const noexcept {
        return selectionAllowsHud(managed, hudSelected, policyStop, controller);
    }
    std::atomic<bool> workerFinished{false};
    std::atomic<bool> workerActive{false};
    // The process-global policy outlives all swap chains. Once the last
    // chain unbinds, a still-valid but unused game HWND must not keep the
    // separate native HUD message-loop thread alive indefinitely.
    ULONGLONG unboundAt = 0;
    unsigned generation = 0;
    Snapshot display{};
    HANDLE stop = CreateEventW(nullptr, TRUE, FALSE, nullptr);
    HANDLE changed = CreateEventW(nullptr, FALSE, FALSE, nullptr);
    std::thread worker;
    static LONGLONG frequency() {
        LARGE_INTEGER value{};
        if (!QueryPerformanceFrequency(&value) || value.QuadPart <= 0 || value.QuadPart > 1000000000000LL)
            throw std::runtime_error("HUD clock unavailable");
        return value.QuadPart;
    }
    State(HANDLE selected, HANDLE stopped, HANDLE producer) : meter(frequency()), managed(selected || stopped || producer) {
        const auto copy = [](HANDLE source, HANDLE& destination) {
            return source && DuplicateHandle(GetCurrentProcess(), source, GetCurrentProcess(), &destination, SYNCHRONIZE, FALSE, 0);
        };
        const bool valid = !selected || (copy(selected, hudSelected) && copy(stopped, policyStop) && copy(producer, controller));
        if (!stop || !changed || !valid) {
            for (HANDLE h : {hudSelected, policyStop, controller}) if (h) CloseHandle(h);
            if (stop) CloseHandle(stop); if (changed) CloseHandle(changed);
            throw std::runtime_error("HUD events unavailable");
        }
    }
    ~State() {
        SetEvent(stop); if (worker.joinable()) worker.join();
        if (ownerLife) CloseHandle(ownerLife);
        for (HANDLE h : {hudSelected, policyStop, controller}) if (h) CloseHandle(h);
        CloseHandle(changed); CloseHandle(stop);
    }
    void fail() noexcept { AcquireSRWLockExclusive(&gate); ++display.failures; ReleaseSRWLockExclusive(&gate); }
    static void location(void* context, HWND window) noexcept {
        auto* state = static_cast<State*>(context);
        AcquireSRWLockShared(&state->gate); const bool own = window == state->owner; ReleaseSRWLockShared(&state->gate);
        if (own) SetEvent(state->changed);
    }
    void run() noexcept {
        HWND hud = nullptr, previousOwner = nullptr;
        workerActive.store(true, std::memory_order_release);
        auto* host = lazytester_hud_window_host_create(this, location);
        try {
            if (!host) throw std::runtime_error("HUD window host unavailable");
            std::unique_ptr<LazyTesterHudRenderer, decltype(&lazytester_hud_renderer_destroy)> renderer(
                lazytester_hud_renderer_create(), lazytester_hud_renderer_destroy);
            if (!renderer) throw std::runtime_error("HUD renderer unavailable");
            unsigned previousGeneration = 0;
            for (;;) {
                AcquireSRWLockExclusive(&gate);
                if (ownerLife && WaitForSingleObject(ownerLife, 0) == WAIT_OBJECT_0) {
                    workerFinished.store(true, std::memory_order_release);
                    ReleaseSRWLockExclusive(&gate); break;
                }
                const HWND target = owner; const DWORD thread = ownerThread;
                const unsigned version = generation; const ULONGLONG idleSince = unboundAt;
                ReleaseSRWLockExclusive(&gate);
                DWORD pid = 0;
                const bool bound = target && GetWindowThreadProcessId(target, &pid) == thread && pid == GetCurrentProcessId();
                // The unbind path runs before the original Minecraft GUI
                // thread/HWND necessarily exits. A process-global Policy must
                // not retain this native worker indefinitely after that.
                // Allow a short grace period for normal swap-chain recreation.
                constexpr ULONGLONG UnboundGraceMilliseconds = 1000;
                const bool destroyedOwner = target && !bound;
                const bool idleExpired = !target && idleSince
                    && GetTickCount64() - idleSince >= UnboundGraceMilliseconds;
                if (destroyedOwner || idleExpired) {
                    AcquireSRWLockExclusive(&gate);
                    // Publish a new owner and advance generation under this
                    // same gate in bind(); never retire a newer rebind.
                    const bool unchanged = owner == target && generation == version
                        && (!idleExpired || unboundAt == idleSince);
                    if (unchanged) workerFinished.store(true, std::memory_order_release);
                    ReleaseSRWLockExclusive(&gate);
                    if (unchanged) break;
                    continue;
                }
                if (!bound || target != previousOwner || version != previousGeneration || (hud && !IsWindow(hud))) {
                    lazytester_hud_window_clear(host);
                    hud = nullptr; previousOwner = target; previousGeneration = version;
                    lazytester_hud_renderer_reset(renderer.get());
                }
                const auto value = meter.read();
                LazyTesterHudComposition composed{};
                if (!lazytester_hud_renderer_update(renderer.get(), host, target, bound ? 1U : 0U,
                    selectionAdmitted() ? 1U : 0U, value.milliFps, value.valid ? 1U : 0U, &composed))
                    throw std::runtime_error("HUD composition unavailable");
                if (composed.window) hud = composed.window;
                AcquireSRWLockExclusive(&gate);
                display.window = composed.window; display.visible = composed.visible != 0;
                if (composed.window) { display.bounds = composed.bounds; display.dpi = composed.dpi; }
                display.draws += composed.drawn;
                ReleaseSRWLockExclusive(&gate);
                const auto result = lazytester_hud_window_wait(stop, changed);
                if (result == 1) break;
                if (result != 0) throw std::runtime_error("HUD wait failed");
            }
        } catch (...) { fail(); }
        lazytester_hud_window_host_destroy(host);
        AcquireSRWLockExclusive(&gate);
        display.window = nullptr; display.visible = false;
        // Every worker exit path (owner exited, owner HWND destroyed,
        // grace-expired unbind, STOP or a native/GDI error) is observable.
        workerActive.store(false, std::memory_order_release);
        workerFinished.store(true, std::memory_order_release);
        ReleaseSRWLockExclusive(&gate);
    }
};
Hud::Hud(HANDLE hudSelected, HANDLE policyStop, HANDLE controller) : state_(std::make_unique<State>(hudSelected, policyStop, controller)) {}
Hud::~Hud() = default;
bool Hud::bind(HWND owner, DWORD thread) noexcept {
    DWORD pid = 0;
    if (!owner || !thread || GetWindowThreadProcessId(owner, &pid) != thread || pid != GetCurrentProcessId()
        || GetAncestor(owner, GA_ROOT) != owner) return false;
    auto& s = *state_;
    try {
        HANDLE life = OpenThread(SYNCHRONIZE | THREAD_QUERY_LIMITED_INFORMATION, FALSE, thread);
        if (!life) return false;
        DWORD confirmedPid = 0;
        if (GetProcessIdOfThread(life) != GetCurrentProcessId()
            || GetWindowThreadProcessId(owner, &confirmedPid) != thread
            || confirmedPid != GetCurrentProcessId()
            || WaitForSingleObject(life, 0) != WAIT_TIMEOUT) {
            CloseHandle(life); return false;
        }
        AcquireSRWLockExclusive(&s.gate);
        if (s.ownerLife) CloseHandle(s.ownerLife);
        s.ownerLife = life;
        // Publish the new generation BEFORE checking the retirement marker.
        // Otherwise an idle worker could commit to exit between the check
        // and the new binding, silently dropping the replacement HUD.
        s.owner = owner; s.ownerThread = thread; s.unboundAt = 0;
        ++s.generation; s.meter.reset();
        const bool finished = s.workerFinished.load(std::memory_order_acquire);
        ReleaseSRWLockExclusive(&s.gate);
        if (finished && s.worker.joinable()) s.worker.join();
        if (!s.worker.joinable()) {
            HMODULE pinned = nullptr;
            if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
                reinterpret_cast<LPCWSTR>(moduleAnchor), &pinned)) {
                unbind();
                return false;
            }
            s.workerFinished.store(false, std::memory_order_release);
            s.worker = std::thread([&s] { s.run(); });
        }
        return SetEvent(s.changed) != FALSE;
    } catch (...) { unbind(); s.fail(); return false; }
}
void Hud::unbind() noexcept {
    auto& s = *state_;
    AcquireSRWLockExclusive(&s.gate);
    s.owner = nullptr; s.ownerThread = 0;
    s.unboundAt = GetTickCount64(); ++s.generation; s.meter.reset();
    ReleaseSRWLockExclusive(&s.gate);
    SetEvent(s.changed);
}
void Hud::present(HRESULT result, UINT flags) noexcept {
    if (result == S_OK && !(flags & DXGI_PRESENT_TEST)) state_->meter.record(now(), result, flags);
}
Snapshot Hud::snapshot() noexcept {
    auto& s = *state_;
    // Keep the owner binding and meter reading in one snapshot generation.
    // bind/unbind take this gate EXCLUSIVE before resetting the meter;
    // readers retain the identical lock order: state gate, then meter.
    AcquireSRWLockShared(&s.gate);
    auto value = s.display;
    value.reading = s.meter.read();
    value.workerActive = s.workerActive.load(std::memory_order_acquire);
    // A still-alive HUD worker may be in the one-second unbound grace.
    // A successful synthetic/native Present must not resurrect a live
    // rate after the game owner was dropped. Counts remain diagnostic.
    if (!s.owner || !s.ownerThread || !value.workerActive) value.reading.valid = false;
    ReleaseSRWLockShared(&s.gate);
    return value;
}
}

