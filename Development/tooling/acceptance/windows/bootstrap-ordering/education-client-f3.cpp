#include "education-client-f3.h"
#include <d3d11.h>
#include <dxgi1_5.h>
#include <d2d1_1.h>
#include <d2d1_1helper.h>
#include <dwrite.h>
#include <wrl/client.h>
#include <cmath>
#include <cwchar>
#include <cstring>
#include <array>

// No Education engine offsets, world registration, message hooks, owned
// Window/overlay, injected input, timers or new session controller.
// The original game's native Player data and F3 dispatch adapter remain
// unbound until their exact Education ABI is independently verified.
namespace education_client_f3 {
namespace {
using Microsoft::WRL::ComPtr;
constexpr ULONGLONG FrameFreshnessMs = 250;
constexpr UINT MaxBuffers = 16;

struct Painter {
    ComPtr<ID2D1Factory1> factory;
    ComPtr<ID2D1Device> device;
    ComPtr<ID2D1DeviceContext> context;
    ComPtr<IDWriteFactory> textFactory;
    ComPtr<IDWriteTextFormat> format;
    ComPtr<ID2D1SolidColorBrush> foreground, background;
    std::array<ComPtr<ID2D1Bitmap1>, MaxBuffers> buffers{};
    UINT bufferCount = 0;
    DXGI_FORMAT pixelFormat = DXGI_FORMAT_UNKNOWN;

    void release() noexcept {
        if (context) context->SetTarget(nullptr);
        for (auto& image : buffers) image.Reset();
        foreground.Reset(); background.Reset();
        format.Reset(); textFactory.Reset();
        context.Reset(); device.Reset(); factory.Reset();
        bufferCount = 0; pixelFormat = DXGI_FORMAT_UNKNOWN;
    }

    bool prepare(IDXGISwapChain4* swap) noexcept {
        if (!swap) return false;
        if (context) return true;
        ComPtr<ID3D11Device> graphics;
        ComPtr<IDXGIDevice> dxgiDevice;
        DXGI_SWAP_CHAIN_DESC1 desc{};
        if (FAILED(swap->GetDesc1(&desc)) || desc.Width < 480 || desc.Height < 260
            || desc.BufferCount == 0 || desc.BufferCount > MaxBuffers
            || (desc.Format != DXGI_FORMAT_B8G8R8A8_UNORM
                && desc.Format != DXGI_FORMAT_R8G8B8A8_UNORM)
            || FAILED(swap->GetDevice(IID_PPV_ARGS(&graphics)))
            || FAILED(graphics.As(&dxgiDevice))) return false;
        // D3D12, HDR or unsupported buffers deliberately remain unavailable.
        if (FAILED(D2D1CreateFactory(D2D1_FACTORY_TYPE_MULTI_THREADED,
            __uuidof(ID2D1Factory1), nullptr,
            reinterpret_cast<void**>(factory.GetAddressOf())))
            || FAILED(factory->CreateDevice(dxgiDevice.Get(), device.GetAddressOf()))
            || FAILED(device->CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE,
                context.GetAddressOf()))
            || FAILED(DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED,
                __uuidof(IDWriteFactory),
                reinterpret_cast<IUnknown**>(textFactory.GetAddressOf())))
            || FAILED(textFactory->CreateTextFormat(L"Consolas", nullptr,
                DWRITE_FONT_WEIGHT_NORMAL, DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL, 15.0f, L"en-us",
                format.GetAddressOf()))
            || FAILED(context->CreateSolidColorBrush(
                D2D1::ColorF(0.92f, 0.96f, 1.0f, 1.0f),
                foreground.GetAddressOf()))
            || FAILED(context->CreateSolidColorBrush(
                D2D1::ColorF(0.02f, 0.035f, 0.07f, 0.82f),
                background.GetAddressOf()))) {
            release(); return false;
        }
        bufferCount = desc.BufferCount;
        pixelFormat = desc.Format;
        return true;
    }

    bool draw(IDXGISwapChain4* swap, const wchar_t* text) noexcept {
        if (!prepare(swap)) return false;
        const UINT index = swap->GetCurrentBackBufferIndex();
        if (index >= bufferCount) return false;
        if (!buffers[index]) {
            ComPtr<IDXGISurface> surface;
            if (FAILED(swap->GetBuffer(index, IID_PPV_ARGS(&surface)))) return false;
            D2D1_BITMAP_PROPERTIES1 properties{};
            properties.pixelFormat.format = pixelFormat;
            properties.pixelFormat.alphaMode = D2D1_ALPHA_MODE_IGNORE;
            properties.bitmapOptions = D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW;
            properties.dpiX = properties.dpiY = 96.0f;
            if (FAILED(context->CreateBitmapFromDxgiSurface(surface.Get(),
                &properties, buffers[index].GetAddressOf()))) return false;
        }
        context->SetTarget(buffers[index].Get());
        context->BeginDraw();
        const D2D1_RECT_F backdrop = D2D1::RectF(10.f, 10.f, 420.f, 210.f);
        context->FillRectangle(backdrop, background.Get());
        context->DrawText(text, static_cast<UINT32>(wcslen(text)),
            format.Get(), D2D1::RectF(20.f, 18.f, 414.f, 203.f),
            foreground.Get(), D2D1_DRAW_TEXT_OPTIONS_CLIP,
            DWRITE_MEASURING_MODE_NATURAL);
        const HRESULT result = context->EndDraw();
        context->SetTarget(nullptr);
        if (FAILED(result)) { release(); return false; }
        return true;
    }
};

struct Session {
    SRWLOCK lock = SRWLOCK_INIT;
    HWND owner = nullptr;
    DWORD ownerThread = 0;
    bool claimed = false, shown = false, f3Held = false, chord = false, fresh = false;
    bool rendererReady = false; // Proven compatible only by current D3D11 device admission.
    ULONGLONG observed = 0;
    std::uint64_t worldGeneration = 0;
    PlayerFrame frame{};
    Painter painter{};
};
Session session;

bool exactWindow(HWND window, DWORD thread) noexcept {
    if (!window || !thread) return false;
    DWORD process = 0;
    wchar_t type[32]{};
    return GetWindowThreadProcessId(window, &process) == thread
        && process == GetCurrentProcessId()
        && GetAncestor(window, GA_ROOT) == window
        && GetClassNameW(window, type, 32) != 0
        && wcscmp(type, L"OGLES") == 0;
}
bool validFrame(const PlayerFrame& frame) noexcept {
    const auto within = [](double x, double bound) noexcept {
        return std::isfinite(x) && std::abs(x) <= bound;
    };
    return frame.bytes == sizeof(PlayerFrame)
        && frame.worldGeneration != 0
        && within(frame.x, 30000000.) && within(frame.z, 30000000.)
        && within(frame.y, 30000000.) && within(frame.pitch, 90.)
        && within(frame.yaw, 1000000.)
        && (frame.dimension == Dimension::Overworld
            || frame.dimension == Dimension::Nether
            || frame.dimension == Dimension::End);
}
bool live(Session& state) noexcept {
    return state.fresh && state.worldGeneration != 0
        && GetTickCount64() >= state.observed
        && GetTickCount64() - state.observed <= FrameFreshnessMs
        && validFrame(state.frame);
}
void hide(Session& state) noexcept {
    state.shown = state.f3Held = state.chord = false;
}
bool copyCoordinates(HWND owner, const PlayerFrame& frame) noexcept {
    wchar_t coordinates[128]{};
    const int size = swprintf_s(coordinates, L"%.0f %.0f %.0f",
        std::floor(frame.x), std::floor(frame.y), std::floor(frame.z));
    if (size <= 0 || !OpenClipboard(owner)) return false;
    const SIZE_T bytes = (static_cast<SIZE_T>(size) + 1) * sizeof(wchar_t);
    HGLOBAL memory = GlobalAlloc(GMEM_MOVEABLE, bytes);
    if (!memory) { CloseClipboard(); return false; }
    void* destination = GlobalLock(memory);
    if (!destination) {
        GlobalFree(memory); CloseClipboard(); return false;
    }
    memcpy(destination, coordinates, bytes);
    GlobalUnlock(memory);
    const bool success = EmptyClipboard()
        && SetClipboardData(CF_UNICODETEXT, memory) != nullptr;
    if (!success) GlobalFree(memory);
    CloseClipboard();
    return success;
}
const wchar_t* dimension(Dimension value) noexcept {
    switch (value) {
    case Dimension::Overworld: return L"Overworld";
    case Dimension::Nether: return L"Nether";
    case Dimension::End: return L"The End";
    }
    return L"Unknown";
}
const wchar_t* facing(double yaw) noexcept {
    const double degrees = std::fmod(std::fmod(yaw, 360.) + 360., 360.);
    if (degrees < 45. || degrees >= 315.) return L"South (+Z)";
    if (degrees < 135.) return L"West (-X)";
    if (degrees < 225.) return L"North (-Z)";
    return L"East (+X)";
}
} // namespace

bool claim_game_window(HWND game, DWORD gameThread) noexcept {
    const DWORD error = GetLastError();
    bool claimed = false;
    if (exactWindow(game, gameThread) && GetCurrentThreadId() == gameThread) {
        AcquireSRWLockExclusive(&session.lock);
        if (!session.claimed) {
            session.owner = game; session.ownerThread = gameThread;
            session.claimed = true; claimed = true;
        } else {
            claimed = session.owner == game && session.ownerThread == gameThread;
        }
        ReleaseSRWLockExclusive(&session.lock);
    }
    SetLastError(error);
    return claimed;
}

bool publish_player(HWND game, const PlayerFrame& frame) noexcept {
    const DWORD error = GetLastError();
    bool accepted = false;
    AcquireSRWLockExclusive(&session.lock);
    if (session.claimed && session.owner == game &&
        exactWindow(game, session.ownerThread)) {
        if (validFrame(frame)) {
            if (session.worldGeneration && session.worldGeneration != frame.worldGeneration)
                hide(session); // A new world cannot inherit the old F3 overlay.
            session.frame = frame;
            session.worldGeneration = frame.worldGeneration;
            session.observed = GetTickCount64();
            session.fresh = true;
            accepted = true;
        } else {
            session.fresh = false;
            hide(session);
        }
    }
    ReleaseSRWLockExclusive(&session.lock);
    SetLastError(error);
    return accepted;
}

KeyResult game_key(HWND game, DWORD gameThread, UINT message, WPARAM key,
                   LPARAM parameters, bool gameplay, bool physical) noexcept {
    const DWORD error = GetLastError();
    KeyResult result = KeyResult::Ignored;
    AcquireSRWLockExclusive(&session.lock);
    const bool admitted = session.claimed && session.owner == game
        && session.ownerThread == gameThread
        && GetCurrentThreadId() == gameThread
        && exactWindow(game, gameThread)
        && GetForegroundWindow() == game
        && gameplay && physical && session.rendererReady && live(session);
    if (!admitted) {
        hide(session);
    } else {
        const bool down = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;
        const bool up = message == WM_KEYUP || message == WM_SYSKEYUP;
        if (key == VK_F3 && down && (parameters & (1LL << 30)) == 0 && !session.f3Held) {
            session.f3Held = true;
            session.chord = false;
        } else if (key == 'C' && down && (parameters & (1LL << 30)) == 0
            && session.f3Held && !session.chord && session.shown) {
            session.chord = true;
            result = copyCoordinates(game, session.frame) ?
                KeyResult::Copied : KeyResult::CopyFailed;
        } else if (key == VK_F3 && up) {
            if (session.f3Held && !session.chord) {
                session.shown = !session.shown;
                result = session.shown ? KeyResult::ShowRequested : KeyResult::HideRequested;
            }
            session.f3Held = false;
            session.chord = false;
        }
    }
    ReleaseSRWLockExclusive(&session.lock);
    SetLastError(error);
    return result;
}

void before_present(IDXGISwapChain1* original, HWND game, DWORD gameThread,
                    unsigned milliFps, bool measuredFps) noexcept {
    const DWORD error = GetLastError();
    AcquireSRWLockExclusive(&session.lock);
    const bool eligible = session.claimed && session.owner == game
        && session.ownerThread == gameThread && exactWindow(game, gameThread)
        && GetForegroundWindow() == game && live(session) && original;
    if (!eligible) {
        hide(session); // No stale data, background focus or absent Player.
        session.rendererReady = false;
    } else {
        ComPtr<IDXGISwapChain4> chain;
        if (FAILED(original->QueryInterface(IID_PPV_ARGS(&chain)))
            || !session.painter.prepare(chain.Get())) {
            hide(session);
            session.rendererReady = false; // No blank/unsupported panel.
        } else {
            session.rendererReady = true;
            if (session.shown) {
                const auto& f = session.frame;
                wchar_t text[768]{}, fps[40]{};
                if (measuredFps) {
                    swprintf_s(fps, L"%.1f", static_cast<double>(milliFps) / 1000.);
                } else {
                    wcscpy_s(fps, L"--");
                }
                const double bx = std::floor(f.x), by = std::floor(f.y), bz = std::floor(f.z);
                const double cx = std::floor(bx / 16.), cz = std::floor(bz / 16.);
                const int written = swprintf_s(text,
                    L"Minecraft Education | F3 Debug\n"
                    L"XYZ: %.2f / %.2f / %.2f\n"
                    L"Block: %.0f / %.0f / %.0f\n"
                    L"Chunk: %.0f / %.0f\n"
                    L"Facing: %ls\n"
                    L"Yaw / Pitch: %.2f / %.2f\n"
                    L"Dimension: %ls\n"
                    L"FPS: %ls\n"
                    L"F3: Hide    F3+C: Copy XYZ",
                    f.x, f.y, f.z, bx, by, bz, cx, cz,
                    facing(f.yaw), f.yaw, f.pitch, dimension(f.dimension), fps);
                if (written <= 0 || !session.painter.draw(chain.Get(), text)) {
                    hide(session);
                    session.rendererReady = false;
                    // No successful Draw acknowledgement is fabricated if a
                    // device/swapchain reset or an incompatible buffer fails.
                }
            }
        }
    }
    ReleaseSRWLockExclusive(&session.lock);
    SetLastError(error);
}
void before_resize(HWND game) noexcept {
    const DWORD error = GetLastError();
    AcquireSRWLockExclusive(&session.lock);
    if (session.claimed && session.owner == game) {
        session.painter.release(); // Drop all backbuffer refs before ResizeBuffers.
        session.rendererReady = false;
        hide(session);
    }
    ReleaseSRWLockExclusive(&session.lock);
    SetLastError(error);
}
void retire(HWND game) noexcept {
    const DWORD error = GetLastError();
    AcquireSRWLockExclusive(&session.lock);
    if (session.claimed && session.owner == game) {
        session.painter.release();
        hide(session);
        session.claimed = session.fresh = session.rendererReady = false;
        session.owner = nullptr; session.ownerThread = 0;
        session.observed = 0; session.worldGeneration = 0;
        session.frame = {};
    }
    ReleaseSRWLockExclusive(&session.lock);
    SetLastError(error);
}
} // namespace education_client_f3
