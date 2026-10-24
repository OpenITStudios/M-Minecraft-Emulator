#include "frame-pacing.h"
#include "dxgi-pacing.h"
#include <d3d11.h>
#include <d3d12.h>
#include <dxgi1_5.h>
#include <wrl/client.h>
#include <array>
#include <cstdio>
#include <cwchar>
#include <memory>
#include <thread>
#include <vector>
#include <exception>

namespace {
using Microsoft::WRL::ComPtr;
using presentation_pacing::Outcome;
using presentation_pacing::Permit;
constexpr wchar_t WindowClass[] = L"MivubiOwnHiddenPacingControl";
void checkAt(bool condition, unsigned line) {
    if (!condition) {
        std::fprintf(stderr, "{\"pacingControlFailureLine\":%u}\n", line);
        throw std::runtime_error("Pacing control failed");
    }
}
#define check(...) checkAt((__VA_ARGS__), __LINE__)
LONGLONG clockNow() {
    LARGE_INTEGER value{};
    check(QueryPerformanceCounter(&value) != FALSE);
    return value.QuadPart;
}
struct Event {
    HANDLE value = CreateEventW(nullptr, TRUE, FALSE, nullptr);
    Event() { check(value != nullptr); }
    ~Event() { CloseHandle(value); }
};
struct HiddenWindow {
    HWND value = CreateWindowExW(WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW, WindowClass,
        L"", WS_POPUP, 0, 0, 16, 16, nullptr, nullptr, GetModuleHandleW(nullptr), nullptr);
    HiddenWindow() { check(value != nullptr && !IsWindowVisible(value)); }
    ~HiddenWindow() { DestroyWindow(value); }
};
struct Client {
    presentation_pacing::Pacer pacer;
    Event ready;
    std::thread worker;
    std::vector<Permit> frames;
    std::exception_ptr failure;
    unsigned occluded = 0;
    ~Client() { pacer.stop(); if (worker.joinable()) worker.join(); }
    void start(HANDLE gate) {
        worker = std::thread([this, gate] {
            try {
                HiddenWindow window;
                DXGI_SWAP_CHAIN_DESC description{};
                description.BufferDesc.Width = description.BufferDesc.Height = 16;
                description.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
                description.SampleDesc.Count = 1;
                description.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
                description.BufferCount = 1;
                description.OutputWindow = window.value;
                description.Windowed = TRUE;
                description.SwapEffect = DXGI_SWAP_EFFECT_DISCARD;
                ComPtr<ID3D11Device> device;
                ComPtr<ID3D11DeviceContext> context;
                ComPtr<IDXGISwapChain> chain;
                check(SUCCEEDED(D3D11CreateDeviceAndSwapChain(nullptr, D3D_DRIVER_TYPE_WARP,
                    nullptr, 0, nullptr, 0, D3D11_SDK_VERSION, &description,
                    &chain, &device, nullptr, &context)));
                ComPtr<ID3D11Texture2D> texture;
                ComPtr<ID3D11RenderTargetView> target;
                check(SUCCEEDED(chain->GetBuffer(0, IID_PPV_ARGS(&texture))));
                check(SUCCEEDED(device->CreateRenderTargetView(texture.Get(), nullptr, &target)));
                check(SetEvent(ready.value) != FALSE);
                check(WaitForSingleObject(gate, 10000) == WAIT_OBJECT_0);
                for (;;) {
                    MSG message{};
                    while (PeekMessageW(&message, nullptr, 0, 0, PM_REMOVE)) {
                        TranslateMessage(&message); DispatchMessageW(&message);
                    }
                    const Permit permit = pacer.wait_for_frame();
                    if (permit.outcome == Outcome::Stopped) break;
                    check(permit.outcome == Outcome::Frame);
                    frames.push_back(permit);
                    const float colour[]{0.1f, 0.2f, 0.3f, 1.0f};
                    context->ClearRenderTargetView(target.Get(), colour);
                    const HRESULT presented = chain->Present(0, 0);
                    check(SUCCEEDED(presented));
                    if (presented == DXGI_STATUS_OCCLUDED) ++occluded;
                }
            } catch (...) { failure = std::current_exception(); SetEvent(ready.value); }
        });
    }
};
unsigned long long cpuTime() {
    FILETIME created{}, exited{}, kernel{}, user{};
    check(GetProcessTimes(GetCurrentProcess(), &created, &exited, &kernel, &user));
    const auto ticks = [](FILETIME value) {
        return (static_cast<unsigned long long>(value.dwHighDateTime) << 32) | value.dwLowDateTime;
    };
    return ticks(kernel) + ticks(user);
}
void cancellationAndLateFrame() {
    presentation_pacing::Pacer cancelled;
    check(cancelled.wait_for_frame().outcome == Outcome::Frame);
    Event entered;
    Permit interrupted{Outcome::Error, 0, 0};
    std::thread waiting([&] { SetEvent(entered.value); interrupted = cancelled.wait_for_frame(); });
    const DWORD ready = WaitForSingleObject(entered.value, 1000);
    Sleep(5);
    const bool stopped = cancelled.stop();
    waiting.join();
    check(ready == WAIT_OBJECT_0 && stopped && interrupted.outcome == Outcome::Stopped
        && !cancelled.foreground(true));

    presentation_pacing::Pacer late;
    check(late.foreground(true));
    check(late.wait_for_frame().outcome == Outcome::Frame);
    Sleep(30); // Simulate a slow frame without accruing extra frame permits.
    const Permit second = late.wait_for_frame(), third = late.wait_for_frame();
    LARGE_INTEGER frequency{};
    check(QueryPerformanceFrequency(&frequency));
    check(second.outcome == Outcome::Frame && third.outcome == Outcome::Frame
        && third.released - second.released >= (frequency.QuadPart + 143) / 144);
}
void adapterChecks(bool modern) {
    HiddenWindow selectedWindow, foreignWindow;
    ComPtr<ID3D11Device> device;
    ComPtr<ID3D11DeviceContext> context;
    check(SUCCEEDED(D3D11CreateDevice(nullptr, D3D_DRIVER_TYPE_WARP, nullptr, 0,
        nullptr, 0, D3D11_SDK_VERSION, &device, nullptr, &context)));
    ComPtr<IDXGIFactory4> factory;
    check(SUCCEEDED(CreateDXGIFactory2(0, IID_PPV_ARGS(&factory))));
    ComPtr<ID3D12Device> device12;
    ComPtr<ID3D12CommandQueue> queue;
    if (modern) {
        ComPtr<IDXGIAdapter> warp;
        check(SUCCEEDED(factory->EnumWarpAdapter(IID_PPV_ARGS(&warp)))
            && SUCCEEDED(D3D12CreateDevice(warp.Get(), D3D_FEATURE_LEVEL_11_0, IID_PPV_ARGS(&device12))));
        D3D12_COMMAND_QUEUE_DESC description{};
        description.Type = D3D12_COMMAND_LIST_TYPE_DIRECT;
        check(SUCCEEDED(device12->CreateCommandQueue(&description, IID_PPV_ARGS(&queue))));
    }
    const auto create = [&](HWND window, bool shadow = false) {
        DXGI_SWAP_CHAIN_DESC1 description{};
        description.Width = description.Height = 16;
        description.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
        description.SampleDesc.Count = 1;
        description.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
        description.BufferCount = modern && !shadow ? 2 : 1;
        description.SwapEffect = modern && !shadow ? DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL : DXGI_SWAP_EFFECT_DISCARD;
        ComPtr<IDXGISwapChain1> chain;
        IUnknown* presenter = modern && !shadow ? static_cast<IUnknown*>(queue.Get()) : device.Get();
        check(SUCCEEDED(factory->CreateSwapChainForHwnd(presenter, window, &description,
            nullptr, nullptr, &chain)));
        return chain;
    };
    auto target = create(selectedWindow.value);
    auto other = create(foreignWindow.value);
    // Only one flip chain per HWND; the unrelated same-window UI uses a bitblt chain.
    auto sameWindow = create(selectedWindow.value, true);
    if (modern) { target.Reset(); target = create(selectedWindow.value); }
    constexpr DWORD sentinel = 0x13579bdf;
    SetLastError(sentinel);
    const HRESULT invalid = target->Present(5, 0);
    const DWORD invalidError = GetLastError();
    check(FAILED(invalid));
    DXGI_PRESENT_PARAMETERS parameters{};
    const auto unchanged = parameters;
    SetLastError(sentinel);
    const HRESULT invalidNewer = target->Present1(5, 0, &parameters);
    const DWORD invalidNewerError = GetLastError();
    check(FAILED(invalidNewer));
    check(!dxgi_pacing::wrap(nullptr, selectedWindow.value));
    auto* unchangedTarget = target.Get();
    check(!dxgi_pacing::wrap(target.GetAddressOf(), foreignWindow.value) && target.Get() == unchangedTarget);
    check(dxgi_pacing::wrap(target.GetAddressOf(), selectedWindow.value));
    check(!dxgi_pacing::wrap(other.GetAddressOf(), foreignWindow.value));
    // Forward real resource/identity operations, then release buffers before resize.
    {
        DXGI_SWAP_CHAIN_DESC1 description{};
        HWND actualWindow = nullptr;
        check(SUCCEEDED(target->GetDesc1(&description)) && description.Width == 16
            && SUCCEEDED(target->GetHwnd(&actualWindow)) && actualWindow == selectedWindow.value);
        ComPtr<IUnknown> actualDevice, expectedDevice;
        if (modern) check(SUCCEEDED(device12.As(&expectedDevice)));
        else check(SUCCEEDED(device.As(&expectedDevice)));
        check(SUCCEEDED(target->GetDevice(IID_PPV_ARGS(&actualDevice)))
            && actualDevice.Get() == expectedDevice.Get());
        ComPtr<IUnknown> buffer, bufferAgain;
        check(SUCCEEDED(target->GetBuffer(0, IID_PPV_ARGS(&buffer)))
            && SUCCEEDED(target->GetBuffer(0, IID_PPV_ARGS(&bufferAgain)))
            && buffer.Get() == bufferAgain.Get());
        const GUID key{0xfec1a1fb, 0x531b, 0x423f, {0x85,0xb2,0x71,0x20,0x45,0x26,0xdd,0xea}};
        const UINT marker = 0x576170;
        UINT observed = 0, bytes = sizeof(observed);
        check(SUCCEEDED(target->SetPrivateData(key, sizeof(marker), &marker))
            && SUCCEEDED(target->GetPrivateData(key, &bytes, &observed))
            && bytes == sizeof(marker) && observed == marker);
    }
    LARGE_INTEGER frequency{};
    check(QueryPerformanceFrequency(&frequency));
    check(dxgi_pacing::foreground(true));
    check(SUCCEEDED(target->Present(0, 0)));
    auto before = dxgi_pacing::snapshot();
    check(before.pacedPresent == 1 && before.lastCeiling == 144);
    check(SUCCEEDED(target->Present1(0, 0, &parameters)));
    auto after = dxgi_pacing::snapshot();
    check(after.pacedPresent == 1 && after.pacedPresent1 == 1
        && after.lastRelease - before.lastRelease >= (frequency.QuadPart + 143) / 144
        && memcmp(&parameters, &unchanged, sizeof(parameters)) == 0);
    check(dxgi_pacing::foreground(false));
    before = after;
    check(SUCCEEDED(target->Present1(0, 0, &parameters)));
    after = dxgi_pacing::snapshot();
    check(after.lastCeiling == 10 && after.pacedPresent1 == 2
        && after.lastRelease - before.lastRelease >= (frequency.QuadPart + 9) / 10);
    if (modern) {
        ComPtr<ID3D12Fence> fence;
        Event idle;
        check(SUCCEEDED(device12->CreateFence(0, D3D12_FENCE_FLAG_NONE, IID_PPV_ARGS(&fence)))
            && SUCCEEDED(queue->Signal(fence.Get(), 1))
            && SUCCEEDED(fence->SetEventOnCompletion(1, idle.value))
            && WaitForSingleObject(idle.value, 10000) == WAIT_OBJECT_0);
    }
    check(SUCCEEDED(target->ResizeBuffers(0, 24, 24, DXGI_FORMAT_UNKNOWN, 0)));
    before = after;
    check(SUCCEEDED(target->Present(0, 0)));
    after = dxgi_pacing::snapshot();
    check(after.pacedPresent == 2 && after.lastCeiling == 10
        && after.lastRelease - before.lastRelease >= (frequency.QuadPart + 9) / 10);
    before = after;
    check(SUCCEEDED(other->Present(0, 0)));
    check(SUCCEEDED(sameWindow->Present1(0, 0, &parameters)));
    check(SUCCEEDED(target->Present(0, DXGI_PRESENT_TEST)));
    check(SUCCEEDED(target->Present1(0, DXGI_PRESENT_TEST, &parameters)));
    // Native DO_NOT_WAIT may legitimately report WAS_STILL_DRAWING; preserve it.
    const HRESULT nonblocking = target->Present(0, DXGI_PRESENT_DO_NOT_WAIT);
    check(SUCCEEDED(nonblocking) || nonblocking == DXGI_ERROR_WAS_STILL_DRAWING);
    const HRESULT nonblockingNewer = target->Present1(0, DXGI_PRESENT_DO_NOT_WAIT, &parameters);
    check(SUCCEEDED(nonblockingNewer) || nonblockingNewer == DXGI_ERROR_WAS_STILL_DRAWING);
    SetLastError(sentinel);
    const HRESULT forwarded = target->Present(5, 0);
    const DWORD forwardedError = GetLastError();
    check(forwarded == invalid && forwardedError == invalidError);
    SetLastError(sentinel);
    const HRESULT forwardedNewer = target->Present1(5, 0, &parameters);
    const DWORD forwardedNewerError = GetLastError();
    check(forwardedNewer == invalidNewer && forwardedNewerError == invalidNewerError
        && memcmp(&parameters, &unchanged, sizeof(parameters)) == 0);
    after = dxgi_pacing::snapshot();
    check(after.pacedPresent == before.pacedPresent && after.pacedPresent1 == before.pacedPresent1
        && after.lastRelease == before.lastRelease && after.bypassed >= before.bypassed + 6
        && after.faults == 0);
    // All standard aliases share the wrapper's canonical IUnknown.
    {
        ComPtr<IDXGISwapChain4> newest;
        ComPtr<IUnknown> identity, identityAgain;
        check(SUCCEEDED(target.As(&newest)) && SUCCEEDED(target.As(&identity))
            && SUCCEEDED(newest.As(&identityAgain)) && identity.Get() == identityAgain.Get());
        ComPtr<IDXGISwapChainMedia> media;
        if (SUCCEEDED(target.As(&media))) {
            ComPtr<IUnknown> mediaIdentity;
            check(SUCCEEDED(media.As(&mediaIdentity)) && mediaIdentity.Get() == identity.Get());
        }
        const GUID unsupported{0x04c4e64d, 0x89d3, 0x40de, {0x8b,0x16,0x95,0xc3,0xf7,0xf0,0xad,0x80}};
        void* absent = reinterpret_cast<void*>(1);
        check(target->QueryInterface(unsupported, &absent) == E_NOINTERFACE && absent == nullptr);
        check(newest->QueryInterface(__uuidof(IUnknown), nullptr) == E_POINTER);
    }
    const auto createdBefore = dxgi_pacing::snapshot().createdBindings;
    for (unsigned generation = 0; generation < 4; ++generation) {
        // An application alias legitimately keeps the object alive.
        ComPtr<IUnknown> alias;
        check(SUCCEEDED(target.As(&alias)));
        target.Reset();
        check(dxgi_pacing::snapshot().liveBindings == 1);
        alias.Reset();
        after = dxgi_pacing::snapshot();
        check(after.liveBindings == 0 && after.createdBindings == after.destroyedBindings);
        target = create(selectedWindow.value);
        check(dxgi_pacing::wrap(target.GetAddressOf(), selectedWindow.value));
        before = after;
        check(SUCCEEDED(target->Present1(0, 0, &parameters)));
        after = dxgi_pacing::snapshot();
        check(after.lastCeiling == 10 && after.lastRelease - before.lastRelease
            >= (frequency.QuadPart + 9) / 10);
    }
    check(after.createdBindings == createdBefore + 4 && after.faults == 0);
    // Separate callers share the clock without holding our lock in native DXGI.
    // Keep the HWND owner pumping: DXGI may send messages to this thread.
    before = after;
    check(dxgi_pacing::foreground(true));
    Event begin;
    std::atomic<unsigned> completed{0}, failed{0};
    std::array<std::thread, 2> callers;
    for (unsigned index = 0; index < callers.size(); ++index) {
        callers[index] = std::thread([&, chain = target, index] {
            if (WaitForSingleObject(begin.value, 10000) != WAIT_OBJECT_0) failed.fetch_add(1);
            else for (unsigned frame = 0; frame < 4; ++frame) {
                DXGI_PRESENT_PARAMETERS ownParameters{};
                const HRESULT hr = index ? chain->Present1(0, 0, &ownParameters) : chain->Present(0, 0);
                if (FAILED(hr)) failed.fetch_add(1);
            }
            completed.fetch_add(1);
        });
    }
    check(SetEvent(begin.value));
    const auto deadline = GetTickCount64() + 15000;
    while (completed.load() != callers.size()) {
        if (GetTickCount64() >= deadline) {
            std::fprintf(stderr, "Own presentation concurrency control timed out\n");
            ExitProcess(1); // Only this disposable test executable, never a game.
        }
        MsgWaitForMultipleObjectsEx(0, nullptr, 10, QS_ALLINPUT, MWMO_INPUTAVAILABLE);
        MSG message{};
        while (PeekMessageW(&message, nullptr, 0, 0, PM_REMOVE)) {
            TranslateMessage(&message); DispatchMessageW(&message);
        }
    }
    for (auto& caller : callers) caller.join();
    after = dxgi_pacing::snapshot();
    check(failed.load() == 0 && after.faults == 0
        && after.pacedPresent == before.pacedPresent + 4
        && after.pacedPresent1 == before.pacedPresent1 + 4 && after.lastCeiling == 144);
    check(dxgi_pacing::stop() && !dxgi_pacing::foreground(true));
    before = dxgi_pacing::snapshot();
    check(SUCCEEDED(target->Present(0, 0)));
    after = dxgi_pacing::snapshot();
    check(after.lastRelease == before.lastRelease && after.faults == 0);
    target.Reset();
    after = dxgi_pacing::snapshot();
    check(after.liveBindings == 0 && after.createdBindings == after.destroyedBindings);
    std::printf("{\"scope\":\"OWN_DXGI_TYPED_WRAPPER\",\"selfCheck\":true,\"backend\":\"%s\","
        "\"present\":%llu,\"present1\":%llu,\"bypassed\":%llu,\"recreatedChains\":4,"
        "\"liveBindings\":%u,\"createdBindings\":%llu,\"destroyedBindings\":%llu,"
        "\"unsupportedNativeInterfaces\":%llu,\"nativeFailureAndLastErrorPreserved\":true,"
        "\"identityAndAliasesChecked\":true,\"resourceForwardingChecked\":true,\"concurrentCalls\":8,\"resizeRetainsBinding\":true,\"gameHook\":false,"
        "\"minecraft\":\"NOT_EXECUTED\"}\n", modern ? "D3D12_WARP" : "D3D11_WARP",
        after.pacedPresent, after.pacedPresent1, after.bypassed, after.liveBindings,
        after.createdBindings, after.destroyedBindings, after.unsupportedInterfaces);

}
int run() {
    cancellationAndLateFrame();
    WNDCLASSW type{};
    type.lpfnWndProc = DefWindowProcW;
    type.hInstance = GetModuleHandleW(nullptr);
    type.lpszClassName = WindowClass;
    check(RegisterClassW(&type) != 0);
    Event gate;
    std::array<std::unique_ptr<Client>, 5> clients;
    for (auto& client : clients) { client = std::make_unique<Client>(); client->start(gate.value); }
    for (const auto& client : clients) check(WaitForSingleObject(client->ready.value, 10000) == WAIT_OBJECT_0);
    LARGE_INTEGER frequency{};
    check(QueryPerformanceFrequency(&frequency));
    std::array<LONGLONG, 5> switched{};
    const auto cpuStart = cpuTime();
    const auto started = clockNow();
    check(clients[0]->pacer.foreground(true));
    switched[0] = clockNow();
    check(SetEvent(gate.value));
    for (unsigned active = 0; active < clients.size(); ++active) {
        if (active) {
            check(clients[active - 1]->pacer.foreground(false));
            switched[active] = clockNow();
            check(clients[active]->pacer.foreground(true));
        }
        Sleep(600); // Controller only; renderer sleeps through waitable timer/event.
    }
    const auto stopStarted = clockNow();
    for (auto& client : clients) check(client->pacer.stop());
    for (auto& client : clients) client->worker.join();
    const auto stopped = clockNow();
    const auto cpuEnd = cpuTime();
    const double elapsed = static_cast<double>(stopped - started) / static_cast<double>(frequency.QuadPart);
    double maxPromotionMs = 0;
    unsigned totalFrames = 0, occluded = 0;
    for (unsigned slot = 0; slot < clients.size(); ++slot) {
        auto& client = *clients[slot];
        if (client.failure) std::rethrow_exception(client.failure);
        check(client.frames.size() >= 30 && !client.pacer.foreground(true));
        bool promoted = false;
        unsigned foregroundFrames = 0, backgroundFrames = 0;
        for (size_t i = 0; i < client.frames.size(); ++i) {
            const auto& frame = client.frames[i];
            check(frame.ceiling == 144 || frame.ceiling == 10);
            if (frame.ceiling == 144) ++foregroundFrames; else ++backgroundFrames;
            if (!promoted && frame.ceiling == 144 && frame.released >= switched[slot]) {
                const double latency = 1000.0 * static_cast<double>(frame.released - switched[slot])
                    / static_cast<double>(frequency.QuadPart);
                if (latency > maxPromotionMs) maxPromotionMs = latency;
                promoted = true;
            }
            if (i) check(frame.released - client.frames[i - 1].released
                >= (frequency.QuadPart + frame.ceiling - 1) / frame.ceiling);
        }
        check(promoted && foregroundFrames >= 10 && backgroundFrames >= 10);
        totalFrames += static_cast<unsigned>(client.frames.size());
        occluded += client.occluded;
    }
    const double stopMs = 1000.0 * static_cast<double>(stopped - stopStarted)
        / static_cast<double>(frequency.QuadPart);
    check(maxPromotionMs < 100 && stopMs < 1000);
    adapterChecks(false);
    std::printf("{\"scope\":\"FIVE_OWN_HIDDEN_DXGI_RENDERERS\",\"selfCheck\":true,"
        "\"foregroundCeiling\":144,\"backgroundCeiling\":10,\"releaseSamples\":%u,"
        "\"occludedPresents\":%u,\"elapsedSeconds\":%.3f,\"processCpuSeconds\":%.3f,"
        "\"maximumPromotionMs\":%.3f,\"stopMs\":%.3f,"
        "\"displayedFpsVerified\":false,\"gameHostHealth\":\"UNKNOWN\","
        "\"minecraft\":\"NOT_EXECUTED\"}\n", totalFrames, occluded, elapsed,
        static_cast<double>(cpuEnd - cpuStart) / 10000000.0, maxPromotionMs, stopMs);
    return 0;
}
} // namespace
int wmain(int argc, wchar_t** argv) {
    if (argc != 2) return 64;
    const bool modern = wcscmp(argv[1], L"--run-d3d12-adapter-self-test") == 0;
    if (!modern && wcscmp(argv[1], L"--run-self-test") != 0) return 64;
    try {
        if (!modern) return run();
        WNDCLASSW type{};
        type.lpfnWndProc = DefWindowProcW;
        type.hInstance = GetModuleHandleW(nullptr);
        type.lpszClassName = WindowClass;
        check(RegisterClassW(&type) != 0);
        adapterChecks(true);
        return 0;
    }
    catch (...) {
        std::printf("{\"scope\":\"FIVE_OWN_HIDDEN_DXGI_RENDERERS\",\"selfCheck\":false,"
            "\"minecraft\":\"NOT_EXECUTED\"}\n");
        return 1;
    }
}
