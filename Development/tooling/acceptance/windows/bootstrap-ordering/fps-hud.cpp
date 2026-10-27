#include "fps-hud.h"
#include <algorithm>
#include <atomic>
#include <cmath>
#include <cwchar>
#include <stdexcept>
#include <thread>

namespace fps_hud {
void Meter::reset() noexcept {
    AcquireSRWLockExclusive(&gate_); first_ = last_ = 0; intervals_ = 0; reading_ = {};
    ReleaseSRWLockExclusive(&gate_);
}
void Meter::record(LONGLONG now, HRESULT result, UINT flags) noexcept {
    if (result != S_OK || (flags & DXGI_PRESENT_TEST) || frequency_ <= 0 || now <= 0) return;
    AcquireSRWLockExclusive(&gate_);
    ++reading_.accepted;
    // Native completions on separate threads can acquire this lock out of order.
    now = (std::max)(now, last_);
    if (!first_ || now - last_ > frequency_ * 2) {
        first_ = now; intervals_ = 0; reading_.valid = false;
    } else {
        ++intervals_;
        const LONGLONG elapsed = now - first_;
        if (elapsed >= (frequency_ + 1) / 2) {
            const double rate = 1000.0 * static_cast<double>(intervals_) * static_cast<double>(frequency_)
                / static_cast<double>(elapsed);
            reading_.milliFps = static_cast<unsigned>((std::min)(rate, 9999999.0));
            reading_.valid = true; first_ = now; intervals_ = 0;
        }
    }
    last_ = now; ReleaseSRWLockExclusive(&gate_);
}
Reading Meter::read(LONGLONG now) noexcept {
    AcquireSRWLockShared(&gate_);
    if (!now) { LARGE_INTEGER value{}; if (QueryPerformanceCounter(&value)) now = value.QuadPart; }
    auto value = reading_;
    value.valid = value.valid && now >= last_ && now - last_ <= frequency_ * 2;
    ReleaseSRWLockShared(&gate_); return value;
}
RECT placement(RECT client, UINT dpi) noexcept {
    if (dpi < 48 || dpi > 480) return {};
    const LONG width = MulDiv(128, static_cast<int>(dpi), 96), height = MulDiv(32, static_cast<int>(dpi), 96);
    const LONG margin = MulDiv(12, static_cast<int>(dpi), 96);
    if (static_cast<LONGLONG>(client.right) - client.left < width + 2 * margin
        || static_cast<LONGLONG>(client.bottom) - client.top < height + 2 * margin) return {};
    return {client.right - margin - width, client.top + margin, client.right - margin, client.top + margin + height};
}
bool physicalClient(HWND owner, RECT& result) noexcept {
    const auto context = GetWindowDpiAwarenessContext(owner);
    const auto previous = context ? SetThreadDpiAwarenessContext(context) : nullptr;
    if (!previous) return false;
    RECT client{}; POINT a{}, b{};
    bool okay = GetClientRect(owner, &client) != FALSE;
    a = {client.left, client.top}; b = {client.right, client.bottom};
    okay = okay && ClientToScreen(owner, &a) && ClientToScreen(owner, &b)
        && LogicalToPhysicalPointForPerMonitorDPI(owner, &a) && LogicalToPhysicalPointForPerMonitorDPI(owner, &b);
    const bool restored = SetThreadDpiAwarenessContext(previous) != nullptr;
    result = {a.x, a.y, b.x, b.y}; return okay && restored;
}
namespace {
struct Canvas {
    HDC dc = CreateCompatibleDC(nullptr);
    HBITMAP bitmap = nullptr;
    HFONT font = nullptr;
    HGDIOBJ oldBitmap = nullptr, oldFont = nullptr;
    DWORD* pixels = nullptr;
    ~Canvas() {
        if (oldFont) SelectObject(dc, oldFont);
        if (oldBitmap) SelectObject(dc, oldBitmap);
        if (font) DeleteObject(font);
        if (bitmap) DeleteObject(bitmap);
        if (dc) DeleteDC(dc);
    }
    bool allocate(SIZE size) {
        BITMAPINFO info{}; info.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
        info.bmiHeader.biWidth = size.cx; info.bmiHeader.biHeight = -size.cy;
        info.bmiHeader.biPlanes = 1; info.bmiHeader.biBitCount = 32; info.bmiHeader.biCompression = BI_RGB;
        bitmap = CreateDIBSection(dc, &info, DIB_RGB_COLORS, reinterpret_cast<void**>(&pixels), nullptr, 0);
        if (!dc || !bitmap) return false;
        oldBitmap = SelectObject(dc, bitmap); return oldBitmap && oldBitmap != HGDI_ERROR;
    }
};
LONGLONG now() noexcept { LARGE_INTEGER value{}; return QueryPerformanceCounter(&value) ? value.QuadPart : 0; }
LRESULT CALLBACK windowProcedure(HWND window, UINT message, WPARAM w, LPARAM l) {
    if (message == WM_NCHITTEST) return HTTRANSPARENT;
    if (message == WM_MOUSEACTIVATE) return MA_NOACTIVATE;
    if (message == WM_ERASEBKGND) return 1;
    return DefWindowProcW(window, message, w, l);
}
void moduleAnchor() {}
constexpr wchar_t ClassName[] = L"MivubiMeasuredFpsHud";
void paint(Canvas& canvas, Reading value, UINT dpi, SIZE& size) {
    if (dpi < 48 || dpi > 480) throw std::runtime_error("HUD DPI unavailable");
    size = {MulDiv(128, static_cast<int>(dpi), 96), MulDiv(32, static_cast<int>(dpi), 96)};
    if (!canvas.allocate(size)) throw std::runtime_error("HUD bitmap unavailable");
    std::fill_n(canvas.pixels, static_cast<size_t>(size.cx) * size.cy, 0xff182026UL);
    canvas.font = CreateFontW(-MulDiv(17, static_cast<int>(dpi), 96), 0, 0, 0, FW_BOLD, FALSE, FALSE,
        FALSE, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, ANTIALIASED_QUALITY,
        DEFAULT_PITCH, L"Segoe UI");
    if (!canvas.font) throw std::runtime_error("HUD font unavailable");
    canvas.oldFont = SelectObject(canvas.dc, canvas.font);
    if (!canvas.oldFont || canvas.oldFont == HGDI_ERROR) throw std::runtime_error("HUD font selection failed");
    wchar_t text[32]{};
    if (value.valid) swprintf_s(text, L"%.1f FPS", value.milliFps / 1000.0);
    else wcscpy_s(text, L"-- FPS");
    SetBkMode(canvas.dc, TRANSPARENT); SetTextColor(canvas.dc, RGB(255, 255, 255));
    RECT bounds{0, 0, size.cx, size.cy};
    if (!DrawTextW(canvas.dc, text, -1, &bounds, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX)
        || !GdiFlush()) throw std::runtime_error("HUD text unavailable");
    for (LONG i = 0; i < size.cx * size.cy; ++i) canvas.pixels[i] |= 0xff000000UL;
}
}
std::vector<DWORD> raster(Reading value, UINT dpi, SIZE& size) {
    Canvas canvas; paint(canvas, value, dpi, size);
    return {canvas.pixels, canvas.pixels + static_cast<size_t>(size.cx) * size.cy};
}
struct Hud::State {
    Meter meter;
    SRWLOCK gate = SRWLOCK_INIT;
    HWND owner = nullptr;
    DWORD ownerThread = 0;
    HANDLE ownerLife = nullptr;
    std::atomic<bool> workerFinished{false};
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
    State() : meter(frequency()) {
        if (!stop || !changed) {
            if (stop) CloseHandle(stop); if (changed) CloseHandle(changed);
            throw std::runtime_error("HUD events unavailable");
        }
    }
    ~State() {
        SetEvent(stop); if (worker.joinable()) worker.join();
        if (ownerLife) CloseHandle(ownerLife);
        CloseHandle(changed); CloseHandle(stop);
    }
    void fail() noexcept { AcquireSRWLockExclusive(&gate); ++display.failures; ReleaseSRWLockExclusive(&gate); }
    static thread_local State* callbackOwner;
    static void CALLBACK location(HWINEVENTHOOK, DWORD, HWND window, LONG object, LONG, DWORD, DWORD) {
        auto* state = callbackOwner;
        if (!state || object != OBJID_WINDOW) return;
        AcquireSRWLockShared(&state->gate); const bool own = window == state->owner; ReleaseSRWLockShared(&state->gate);
        if (own) SetEvent(state->changed);
    }
    void run() noexcept {
        HWND hud = nullptr, previousOwner = nullptr;
        callbackOwner = this;
        const auto dpiContext = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        HWINEVENTHOOK hook = nullptr;
        try {
            if (!dpiContext) throw std::runtime_error("HUD DPI context unavailable");
            WNDCLASSW type{}; type.lpfnWndProc = windowProcedure;
            type.hInstance = GetModuleHandleW(nullptr); type.lpszClassName = ClassName;
            if (!RegisterClassW(&type)) {
                WNDCLASSW existing{};
                if (GetLastError() != ERROR_CLASS_ALREADY_EXISTS || !GetClassInfoW(type.hInstance, ClassName, &existing)
                    || existing.lpfnWndProc != windowProcedure) throw std::runtime_error("HUD class unavailable");
            }
            hook = SetWinEventHook(EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_LOCATIONCHANGE, nullptr,
                location, GetCurrentProcessId(), 0, WINEVENT_OUTOFCONTEXT);
            if (!hook) throw std::runtime_error("HUD location hook unavailable");
            unsigned previousGeneration = 0, previousRate = ~0U;
            bool previousValid = false;
            RECT previousBounds{}; UINT previousDpi = 0;
            for (;;) {
                AcquireSRWLockExclusive(&gate);
                if (ownerLife && WaitForSingleObject(ownerLife, 0) == WAIT_OBJECT_0) {
                    workerFinished.store(true, std::memory_order_release);
                    ReleaseSRWLockExclusive(&gate); break;
                }
                const HWND target = owner; const DWORD thread = ownerThread; const unsigned version = generation;
                ReleaseSRWLockExclusive(&gate);
                DWORD pid = 0; RECT client{};
                const bool bound = target && GetWindowThreadProcessId(target, &pid) == thread && pid == GetCurrentProcessId();
                // An unbound HUD must not retain a worker after its former
                // game HWND was destroyed while the owner's thread survives.
                // A newly bound owner/generation wins over a stale observation.
                const bool destroyedOwner = target ? !bound : previousOwner && !IsWindow(previousOwner);
                if (destroyedOwner) {
                    AcquireSRWLockExclusive(&gate);
                    const bool unchanged = owner == target && generation == version;
                    if (unchanged) workerFinished.store(true, std::memory_order_release);
                    ReleaseSRWLockExclusive(&gate);
                    if (unchanged) break;
                    continue;
                }
                if (!bound || target != previousOwner || version != previousGeneration || (hud && !IsWindow(hud))) {
                    if (hud && IsWindow(hud)) DestroyWindow(hud);
                    hud = nullptr; previousOwner = target; previousGeneration = version; previousDpi = 0;
                }
                if (bound && physicalClient(target, client)) {
                    if (!hud) hud = CreateWindowExW(WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
                        ClassName, L"", WS_POPUP, client.left, client.top, 0, 0, target, nullptr, type.hInstance, nullptr);
                    if (!hud) throw std::runtime_error("HUD window unavailable");
                    const UINT dpi = GetDpiForWindow(hud);
                    const RECT bounds = placement(client, dpi);
                    const auto value = meter.read();
                    const bool fits = bounds.right > bounds.left;
                    const bool visible = fits && IsWindowVisible(target) && !IsIconic(target);
                    if (fits) {
                        const unsigned shownRate = (value.milliFps + 50) / 100;
                        const bool redraw = dpi != previousDpi || value.valid != previousValid || shownRate != previousRate;
                        if (redraw) {
                            // Draw directly into the upload DIB; no copied pixel vector or second bitmap.
                            SIZE size{}; Canvas upload; paint(upload, value, dpi, size);
                            POINT position{bounds.left, bounds.top}, origin{};
                            BLENDFUNCTION blend{AC_SRC_OVER, 0, 240, AC_SRC_ALPHA};
                            if (!UpdateLayeredWindow(hud, nullptr, &position, &size, upload.dc, &origin, 0, &blend, ULW_ALPHA))
                                throw std::runtime_error("HUD composition unavailable");
                            AcquireSRWLockExclusive(&gate); ++display.draws; ReleaseSRWLockExclusive(&gate);
                        } else if (memcmp(&bounds, &previousBounds, sizeof(bounds))) {
                            if (!SetWindowPos(hud, nullptr, bounds.left, bounds.top, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE))
                                throw std::runtime_error("HUD positioning failed");
                        }
                        previousDpi = dpi; previousBounds = bounds; previousValid = value.valid; previousRate = shownRate;
                    }
                    if ((IsWindowVisible(hud) != FALSE) != visible) ShowWindow(hud, visible ? SW_SHOWNOACTIVATE : SW_HIDE);
                    AcquireSRWLockExclusive(&gate);
                    display.window = hud; display.bounds = bounds; display.dpi = dpi; display.visible = visible;
                    ReleaseSRWLockExclusive(&gate);
                } else {
                    if (hud && IsWindowVisible(hud)) ShowWindow(hud, SW_HIDE);
                    AcquireSRWLockExclusive(&gate); display.window = nullptr; display.visible = false; ReleaseSRWLockExclusive(&gate);
                }
                HANDLE waits[]{stop, changed};
                const DWORD result = MsgWaitForMultipleObjectsEx(2, waits, 250, QS_ALLINPUT, MWMO_INPUTAVAILABLE);
                if (result == WAIT_OBJECT_0) break;
                if (result == WAIT_FAILED) throw std::runtime_error("HUD wait failed");
                MSG message{};
                while (PeekMessageW(&message, nullptr, 0, 0, PM_REMOVE)) { TranslateMessage(&message); DispatchMessageW(&message); }
            }
        } catch (...) { fail(); }
        if (hud && IsWindow(hud)) DestroyWindow(hud);
        if (hook) UnhookWinEvent(hook);
        callbackOwner = nullptr;
        if (dpiContext) SetThreadDpiAwarenessContext(dpiContext);
        AcquireSRWLockExclusive(&gate);
        display.window = nullptr; display.visible = false;
        // Every worker exit path (owner exited, owner HWND destroyed, STOP or
        // a native/GDI error) must permit bind() to join and restart it.
        workerFinished.store(true, std::memory_order_release);
        ReleaseSRWLockExclusive(&gate);
    }
};
thread_local Hud::State* Hud::State::callbackOwner = nullptr;
Hud::Hud() : state_(std::make_unique<State>()) {}
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
        const bool finished = s.workerFinished.load(std::memory_order_acquire);
        if (s.ownerLife) CloseHandle(s.ownerLife);
        s.ownerLife = life;
        ReleaseSRWLockExclusive(&s.gate);
        if (finished && s.worker.joinable()) s.worker.join();
        if (!s.worker.joinable()) {
            s.workerFinished.store(false, std::memory_order_release);
            HMODULE pinned = nullptr;
            if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
                reinterpret_cast<LPCWSTR>(moduleAnchor), &pinned)) return false;
            s.worker = std::thread([&s] { s.run(); });
        }
        AcquireSRWLockExclusive(&s.gate); s.owner = owner; s.ownerThread = thread; ++s.generation; s.meter.reset(); ReleaseSRWLockExclusive(&s.gate);
        return SetEvent(s.changed) != FALSE;
    } catch (...) { s.fail(); return false; }
}
void Hud::unbind() noexcept {
    auto& s = *state_;
    AcquireSRWLockExclusive(&s.gate); s.owner = nullptr; s.ownerThread = 0; ++s.generation; s.meter.reset(); ReleaseSRWLockExclusive(&s.gate);
    SetEvent(s.changed);
}
void Hud::present(HRESULT result, UINT flags) noexcept {
    if (result == S_OK && !(flags & DXGI_PRESENT_TEST)) state_->meter.record(now(), result, flags);
}
Snapshot Hud::snapshot() noexcept {
    auto& s = *state_;
    AcquireSRWLockShared(&s.gate); auto value = s.display; ReleaseSRWLockShared(&s.gate);
    value.reading = s.meter.read(); return value;
}
}

