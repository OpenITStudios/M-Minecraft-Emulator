#include "frame-pacing.h"
#include <d3d11.h>
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
void check(bool condition) { if (!condition) throw std::runtime_error("Pacing control failed"); }
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
    if (argc != 2 || wcscmp(argv[1], L"--run-self-test") != 0) return 64;
    try { return run(); }
    catch (...) {
        std::printf("{\"scope\":\"FIVE_OWN_HIDDEN_DXGI_RENDERERS\",\"selfCheck\":false,"
            "\"minecraft\":\"NOT_EXECUTED\"}\n");
        return 1;
    }
}
