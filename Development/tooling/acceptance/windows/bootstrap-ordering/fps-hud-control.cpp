#include "fps-hud.h"
#include <algorithm>
#include <atomic>
#include <cstdio>
#include <fstream>
#include <stdexcept>
#include <string>
#include <thread>

namespace {
void checkAt(bool condition, unsigned line) {
    if (!condition) { std::fprintf(stderr, "HUD failure line %u / %lu\n", line, GetLastError()); throw std::runtime_error("HUD check failed"); }
}
#define check(...) checkAt((__VA_ARGS__), __LINE__)
LONGLONG now() { LARGE_INTEGER value{}; check(QueryPerformanceCounter(&value)); return value.QuadPart; }
unsigned long long cpu() {
    FILETIME a{}, b{}, k{}, u{}; check(GetProcessTimes(GetCurrentProcess(), &a, &b, &k, &u));
    return ((static_cast<unsigned long long>(k.dwHighDateTime) << 32) | k.dwLowDateTime)
        + ((static_cast<unsigned long long>(u.dwHighDateTime) << 32) | u.dwLowDateTime);
}
void pump() { MSG m{}; while (PeekMessageW(&m, nullptr, 0, 0, PM_REMOVE)) { TranslateMessage(&m); DispatchMessageW(&m); } }
template<class Test> void until(Test test) {
    const auto start = GetTickCount64();
    while (!test()) { check(GetTickCount64() - start < 3000); pump(); Sleep(5); }
}
void meter() {
    fps_hud::Meter m(144000);
    m.record(1, S_OK, 0); check(!m.read(1).valid);
    for (unsigned i = 1; i <= 144; ++i) m.record(1 + i * 1000, S_OK, 0);
    auto v = m.read(144001); check(v.valid && v.accepted == 145 && v.milliFps == 144000);
    for (const auto hr : {E_FAIL, DXGI_ERROR_WAS_STILL_DRAWING, DXGI_STATUS_OCCLUDED}) m.record(145001, hr, 0);
    m.record(145001, S_OK, DXGI_PRESENT_TEST); check(m.read(145001).accepted == 145);
    m.record(145001, S_OK, DXGI_PRESENT_DO_NOT_WAIT); check(m.read(145001).accepted == 146);
    check(!m.read(433002).valid);
    m.record(433003, S_OK, 0); check(!m.read(433003).valid);
    m.reset(); check(m.read(1).accepted == 0 && !m.read(1).valid);
    m.record(1001, S_OK, 0); m.record(2001, S_OK, 0); m.record(1501, S_OK, 0);
    m.record(73001, S_OK, 0); check(m.read(73001).valid && m.read(73001).accepted == 4);
    m.reset();
    for (unsigned i = 0; i <= 10; ++i) m.record(1 + i * 14400, S_OK, 0);
    check(m.read(144001).milliFps == 10000);
    m.reset();
    for (unsigned i = 0; i <= 10; ++i) m.record(1 + i * 28800, S_OK, 0);
    check(m.read(288001).milliFps == 5000); // Measures actual 5 FPS, not a 10 FPS ceiling.
    m.record(1, S_OK, 0); check(!m.read(1).valid);
    fps_hud::Meter invalid(0); invalid.record(1, S_OK, 0); check(!invalid.read(1).valid);
    for (UINT dpi : {96U, 144U, 192U, 288U}) {
        const RECT client{-1000, 20, 1000, 1220};
        const auto box = fps_hud::placement(client, dpi);
        check(box.right == client.right - MulDiv(12, static_cast<int>(dpi), 96)
            && box.top == client.top + MulDiv(12, static_cast<int>(dpi), 96)
            && box.right - box.left == MulDiv(128, static_cast<int>(dpi), 96));
    }
    check(fps_hud::placement({0, 0, 16, 16}, 96).right == 0);
}
void rasterLifetimes() {
    const auto baseline = GetGuiResources(GetCurrentProcess(), GR_GDIOBJECTS);
    for (UINT dpi : {48U, 96U, 144U, 192U, 288U, 480U}) {
        for (unsigned i = 0; i < 20; ++i) {
            SIZE size{};
            const auto pixels = fps_hud::raster({0, i % 2 ? 144000U : 10000U, i % 3 != 0}, dpi, size);
            check(size.cx == MulDiv(128, static_cast<int>(dpi), 96)
                && size.cy == MulDiv(32, static_cast<int>(dpi), 96));
            check(pixels.size() == static_cast<size_t>(size.cx) * size.cy
                && std::all_of(pixels.begin(), pixels.end(), [](DWORD p) { return (p >> 24) == 255; }));
        }
    }
    for (UINT dpi : {0U, 47U, 481U}) {
        bool refused = false; SIZE size{};
        try { const auto pixels = fps_hud::raster({}, dpi, size); }
        catch (const std::runtime_error&) { refused = true; }
        check(refused);
    }
    check(GetGuiResources(GetCurrentProcess(), GR_GDIOBJECTS) == baseline);
}
void preview() {
    constexpr LONG width = 384, height = 96;
    std::vector<DWORD> page(width * height, 0xff0c1014UL);
    for (unsigned row = 0; row < 3; ++row) {
        fps_hud::Reading value{0, row == 0 ? 144000U : 10000U, row < 2};
        SIZE size{}; const auto pixels = fps_hud::raster(value, 96, size);
        check(size.cx == 128 && size.cy == 32);
        check(std::count_if(pixels.begin(), pixels.end(), [](DWORD pixel) { return (pixel & 0xffffff) != 0x182026; }) > 50);
        check(std::all_of(pixels.begin(), pixels.end(), [](DWORD pixel) { return (pixel >> 24) == 255; }));
        for (LONG y = 0; y < size.cy; ++y) std::copy_n(pixels.begin() + y * size.cx, size.cx, page.begin() + (row * 32 + y) * width);
        const auto large = fps_hud::raster(value, 192, size);
        check(size.cx == 256 && size.cy == 64 && !large.empty());
    }
    BITMAPFILEHEADER file{}; BITMAPINFOHEADER info{};
    info.biSize = sizeof(info); info.biWidth = width; info.biHeight = -height; info.biPlanes = 1;
    info.biBitCount = 32; info.biCompression = BI_RGB;
    file.bfType = 0x4d42; file.bfOffBits = sizeof(file) + sizeof(info);
    file.bfSize = file.bfOffBits + width * height * sizeof(DWORD);
    std::ofstream out("../fps-hud-preview.bmp", std::ios::binary);
    out.write(reinterpret_cast<const char*>(&file), sizeof(file)); out.write(reinterpret_cast<const char*>(&info), sizeof(info));
    out.write(reinterpret_cast<const char*>(page.data()), static_cast<std::streamsize>(page.size() * sizeof(DWORD)));
    check(out.good());
}
int run() {
    meter(); preview(); rasterLifetimes();
    const auto oldDpi = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2); check(oldDpi);
    WNDCLASSW type{}; type.lpfnWndProc = DefWindowProcW; type.hInstance = GetModuleHandleW(nullptr);
    type.lpszClassName = L"MivubiOwnHiddenHudControl"; check(RegisterClassW(&type));
    HWND owner = CreateWindowExW(WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW, type.lpszClassName, L"", WS_POPUP,
        100, 100, 900, 600, nullptr, nullptr, type.hInstance, nullptr);
    check(owner && !IsWindowVisible(owner));
    const auto baseGdi = GetGuiResources(GetCurrentProcess(), GR_GDIOBJECTS);
    const auto started = now(); const auto cpuStart = cpu();
    unsigned draws = 0; double idleCpuSeconds = 0;
    for (unsigned cycle = 0; cycle < 8; ++cycle) {
        fps_hud::Hud hud;
        check(!hud.bind(owner, GetCurrentThreadId() + 1));
        check(hud.bind(owner, GetCurrentThreadId()));
        until([&] { auto s = hud.snapshot(); check(!s.failures); return s.window != nullptr; });
        auto state = hud.snapshot();
        const DWORD originalWorker = GetWindowThreadProcessId(state.window, nullptr);
        const LONG_PTR styles = GetWindowLongPtrW(state.window, GWL_EXSTYLE);
        check((styles & (WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW))
            == (WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW)
            && !(styles & WS_EX_TOPMOST));
        check(!IsWindowVisible(state.window) && !state.visible && GetWindow(state.window, GW_OWNER) == owner);
        check(SendMessageW(state.window, WM_NCHITTEST, 0, 0) == HTTRANSPARENT
            && SendMessageW(state.window, WM_MOUSEACTIVATE, 0, 0) == MA_NOACTIVATE);
        RECT client{}; check(fps_hud::physicalClient(owner, client));
        auto expected = fps_hud::placement(client, state.dpi);
        until([&] { const auto s = hud.snapshot(); return memcmp(&s.bounds, &expected, sizeof(expected)) == 0; });
        check(SetWindowPos(owner, nullptr, 140 + static_cast<int>(cycle), 120, 800 - static_cast<int>(cycle), 500, SWP_NOACTIVATE | SWP_NOZORDER));
        check(fps_hud::physicalClient(owner, client)); expected = fps_hud::placement(client, state.dpi);
        until([&] { const auto s = hud.snapshot(); return memcmp(&s.bounds, &expected, sizeof(expected)) == 0; });
        RECT actual{}; check(GetWindowRect(state.window, &actual) && memcmp(&actual, &expected, sizeof(actual)) == 0);
        hud.present(E_FAIL, 0); hud.present(DXGI_STATUS_OCCLUDED, 0); hud.present(S_OK, DXGI_PRESENT_TEST);
        check(hud.snapshot().reading.accepted == 0);
        hud.present(S_OK, 0); check(hud.snapshot().reading.accepted == 1);
        if (cycle == 0) {
            for (unsigned frame = 0; frame < 6; ++frame) { pump(); Sleep(100); hud.present(S_OK, 0); }
            until([&] { return hud.snapshot().reading.valid && hud.snapshot().draws >= 2; });
            const auto idleStart = cpu(); const auto idleTime = GetTickCount64();
            while (GetTickCount64() - idleTime < 2100) { pump(); Sleep(5); }
            until([&] { return !hud.snapshot().reading.valid && hud.snapshot().draws >= 3; });
            idleCpuSeconds = static_cast<double>(cpu() - idleStart) / 10000000.0;
            check(!hud.snapshot().failures && !hud.snapshot().visible);
        }
        draws += hud.snapshot().draws;
        hud.unbind(); until([&] { return hud.snapshot().window == nullptr; });
        check(!hud.snapshot().reading.valid && hud.snapshot().reading.accepted == 0);
        check(hud.bind(owner, GetCurrentThreadId()));
        until([&] { const auto s = hud.snapshot(); check(!s.failures); return s.window != nullptr; });
        check(hud.snapshot().reading.accepted == 0
            && GetWindowThreadProcessId(hud.snapshot().window, nullptr) != 0);
        // The worker may be reused during a rapid rebind; its identity
        // is not a correctness contract after an idle retirement.
        check(originalWorker != 0);
        hud.unbind(); until([&] { return hud.snapshot().window == nullptr; });
    }
    // Dropping/recreating the window without unbinding is a realistic
    // Minecraft shutdown sequence: the GUI thread itself may remain alive.
    // HUD worker must retire, then a later verified game window can rebind
    // the same policy without a dead joinable std::thread.
    {
        fps_hud::Hud retained;
        HWND old=CreateWindowExW(WS_EX_NOACTIVATE|WS_EX_TOOLWINDOW,type.lpszClassName,L"",
            WS_POPUP,100,100,850,550,nullptr,nullptr,type.hInstance,nullptr);
        check(old && retained.bind(old,GetCurrentThreadId()));
        until([&] {auto state=retained.snapshot();check(!state.failures);return state.window!=nullptr;});
        check(DestroyWindow(old));
        until([&] {return retained.snapshot().window==nullptr;});
        HWND replacement=CreateWindowExW(WS_EX_NOACTIVATE|WS_EX_TOOLWINDOW,type.lpszClassName,L"",
            WS_POPUP,100,100,850,550,nullptr,nullptr,type.hInstance,nullptr);
        check(replacement);
        // The destroyed-owner worker is finished before its display is cleared.
        check(retained.bind(replacement,GetCurrentThreadId()));
        until([&] {auto state=retained.snapshot();check(!state.failures);return state.window!=nullptr;});
        retained.unbind();
        check(DestroyWindow(replacement));
    }
    // A final released swap-chain can leave the original Minecraft GUI
    // HWND/thread alive for some time. The old process-global HUD worker
    // then looped forever because it only watched HWND/thread destruction.
    // This synthetic fixture keeps the original HWND alive throughout.
    {
        fps_hud::Hud closing;
        HWND stillAlive=CreateWindowExW(WS_EX_NOACTIVATE|WS_EX_TOOLWINDOW,type.lpszClassName,L"",
            WS_POPUP,100,100,850,550,nullptr,nullptr,type.hInstance,nullptr);
        check(stillAlive);
        check(closing.bind(stillAlive,GetCurrentThreadId()));
        until([&] {auto state=closing.snapshot();return state.workerActive && state.window!=nullptr;});
        closing.unbind();
        until([&] {return !closing.snapshot().workerActive;});
        check(IsWindow(stillAlive) && GetWindowThreadProcessId(stillAlive,nullptr)==GetCurrentThreadId());
        // Rebind must explicitly join the retired std::thread and start a
        // fresh worker instead of leaving a joinable, finished thread behind.
        check(closing.bind(stillAlive,GetCurrentThreadId()));
        until([&] {auto state=closing.snapshot();return state.workerActive && state.window!=nullptr;});
        closing.unbind();
        until([&] {return !closing.snapshot().workerActive;});
        check(DestroyWindow(stillAlive));
    }
    // Flood only this process's synthetic HUD message queue. A native
    // worker that drains the queue until empty can starve forever while
    // GetTickCount64() and the STOP/unbind checks never run again.
    // Bounded pumping must retire even while WM_NULL traffic continues.
    {
        fps_hud::Hud flooded;
        HWND window=CreateWindowExW(WS_EX_NOACTIVATE|WS_EX_TOOLWINDOW,type.lpszClassName,L"",
            WS_POPUP,100,100,850,550,nullptr,nullptr,type.hInstance,nullptr);
        check(window && flooded.bind(window,GetCurrentThreadId()));
        until([&] {auto state=flooded.snapshot();return state.workerActive && state.window!=nullptr;});
        const DWORD thread=GetWindowThreadProcessId(flooded.snapshot().window,nullptr);
        check(thread && thread!=GetCurrentThreadId());
        std::atomic<bool> producing{true};
        std::thread producer([&] {
            while (producing.load(std::memory_order_acquire)) {
                for (unsigned i=0;i<128;++i) {
                    if (!PostThreadMessageW(thread,WM_NULL,0,0)) break;
                }
                Sleep(1);
            }
        });
        flooded.unbind();
        bool retired=false;
        const auto started=GetTickCount64();
        while (GetTickCount64()-started < 2900) {
            if (!flooded.snapshot().workerActive) {retired=true;break;}
            pump(); Sleep(5);
        }
        producing.store(false,std::memory_order_release);
        producer.join(); // Joined before any throwing assertion can unwind.
        check(retired);
        check(IsWindow(window)); // No game HWND/thread destruction shortcut.
        check(DestroyWindow(window));
    }
    check(GetGuiResources(GetCurrentProcess(), GR_GDIOBJECTS) == baseGdi);
    check(DestroyWindow(owner)); check(SetThreadDpiAwarenessContext(oldDpi));
    LARGE_INTEGER frequency{}; check(QueryPerformanceFrequency(&frequency));
    const double seconds = static_cast<double>(now() - started) / static_cast<double>(frequency.QuadPart);
    const double cpuSeconds = static_cast<double>(cpu() - cpuStart) / 10000000.0;
    fps_hud::Meter benchmark(frequency.QuadPart);
    const auto start = now();
    for (unsigned i = 1; i <= 1000000; ++i) benchmark.record(i, S_OK, 0);
    const double ns = 1000000000.0 * static_cast<double>(now() - start) / static_cast<double>(frequency.QuadPart) / 1000000.0;
    fps_hud::Hud hotPath;
    const auto hotStart = now();
    for (unsigned i = 0; i < 1000000; ++i) hotPath.present(S_OK, 0);
    const double hotNs = 1000000000.0 * static_cast<double>(now() - hotStart) / static_cast<double>(frequency.QuadPart) / 1000000.0;
    std::printf("{\"scope\":\"OWN_HIDDEN_FPS_HUD\",\"selfCheck\":true,\"cycles\":8,\"draws\":%u,"
        "\"seconds\":%.3f,\"cpuSeconds\":%.4f,\"idleTwoSecondsCpu\":%.4f,\"meterNanoseconds\":%.2f,"
        "\"recordWithClockNanoseconds\":%.2f,\"gdiHandlesBalanced\":true,"
        "\"visibleInputTest\":\"NOT_EXECUTED\",\"minecraft\":\"NOT_EXECUTED\"}\n", draws, seconds, cpuSeconds, idleCpuSeconds, ns, hotNs);
    return 0;
}
}
int wmain(int argc, wchar_t** argv) {
    if (argc != 2 || wcscmp(argv[1], L"--run-self-test")) return 64;
    try { return run(); } catch (...) { return 1; }
}

