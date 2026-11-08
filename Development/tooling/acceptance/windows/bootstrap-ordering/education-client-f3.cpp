#include "education-client-f3.h"
#include "dxgi-pacing.h" // Existing in-process FPS owner; no new HUD meter.
#include <d3d11_1.h>
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
    // Isolate our Direct2D draw from the game's D3D11 immediate state.
    // A device without the 11.1 state-swap interfaces is unsupported.
    ComPtr<ID3D11DeviceContext1> immediate;
    ComPtr<ID3DDeviceContextState> f3ContextState;
    ComPtr<IDWriteFactory> textFactory;
    ComPtr<IDWriteTextFormat> format;
    ComPtr<ID2D1SolidColorBrush> foreground, background;
    std::array<ComPtr<ID2D1Bitmap1>, MaxBuffers> buffers{};
    UINT bufferCount = 0;
    DXGI_FORMAT pixelFormat = DXGI_FORMAT_UNKNOWN;
    // An unsupported D3D11 device/format should not trigger a Direct2D
    // factory/device creation attempt on every game Present. Reset only
    // with a genuine ResizeBuffers or final chain teardown.
    bool refused = false;
    bool ownsBackbuffers() const noexcept {
        // Only a completely admitted painter holds resources that need
        // deferred cleanup after the user hides the debug panel.
        // A previously REFUSED chain must keep its refusal until a real
        // swapchain mode transition, never retry just because F3 is hidden.
        return context.Get() != nullptr;
    }

    void release() noexcept {
        if (context) context->SetTarget(nullptr);
        for (auto& image : buffers) image.Reset();
        foreground.Reset(); background.Reset();
        format.Reset(); textFactory.Reset();
        context.Reset(); device.Reset(); factory.Reset();
        f3ContextState.Reset(); immediate.Reset();
        bufferCount = 0; pixelFormat = DXGI_FORMAT_UNKNOWN;
        refused = false;
    }
    bool refuse() noexcept {
        // Any unsupported or failed native backbuffer admission gets ONE
        // attempt per chain. Release all D2D-held backbuffer COM references;
        // only an original swapchain mode change or teardown may retry.
        release();
        refused = true;
        return false;
    }

    bool prepare(IDXGISwapChain4* swap) noexcept {
        if (!swap || refused) return false;
        if (context) return true;
        ComPtr<ID3D11Device> graphics;
        ComPtr<IDXGIDevice> dxgiDevice;
        DXGI_SWAP_CHAIN_DESC1 desc{};
        // Direct2D interop requires the game's EXISTING D3D11 device to
        // have BGRA support. Never replace/recreate Minecraft's device.
        const bool compatible = SUCCEEDED(swap->GetDesc1(&desc))
            && desc.Width >= 480 && desc.Height >= 260
            && desc.BufferCount > 0 && desc.BufferCount <= MaxBuffers
            && (desc.Format == DXGI_FORMAT_B8G8R8A8_UNORM
                || desc.Format == DXGI_FORMAT_R8G8B8A8_UNORM)
            && SUCCEEDED(swap->GetDevice(IID_PPV_ARGS(&graphics)))
            && (graphics->GetCreationFlags() & D3D11_CREATE_DEVICE_BGRA_SUPPORT) != 0
            && SUCCEEDED(graphics.As(&dxgiDevice));
        if (!compatible) { refused = true; return false; }
        // Microsoft's D3D11.1 context-state swapping is documented for
        // Direct2D plug-ins that share an application's immediate context.
        // Do not draw if we cannot restore the exact original pipeline state.
        ComPtr<ID3D11Device1> graphics11;
        if (FAILED(graphics.As(&graphics11))) {
            refused = true;
            return false;
        }
        graphics11->GetImmediateContext1(immediate.GetAddressOf());
        const D3D_FEATURE_LEVEL featureLevel = graphics->GetFeatureLevel();
        const UINT stateFlags =
            (graphics->GetCreationFlags() & D3D11_CREATE_DEVICE_SINGLETHREADED)
                ? D3D11_1_CREATE_DEVICE_CONTEXT_STATE_SINGLETHREADED : 0;
        if (!immediate || FAILED(graphics11->CreateDeviceContextState(
                stateFlags, &featureLevel, 1, D3D11_SDK_VERSION,
                __uuidof(ID3D11Device1), nullptr,
                f3ContextState.GetAddressOf()))) {
            release();
            refused = true;
            return false;
        }
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
            release();
            refused = true; // Do not repeatedly re-initialize on every frame.
            return false;
        }
        bufferCount = desc.BufferCount;
        pixelFormat = desc.Format;
        return true;
    }

    bool draw(IDXGISwapChain4* swap, const wchar_t* text) noexcept {
        if (!prepare(swap)) return false;
        const UINT index = swap->GetCurrentBackBufferIndex();
        if (index >= bufferCount) return refuse();
        if (!buffers[index]) {
            ComPtr<IDXGISurface> surface;
            if (FAILED(swap->GetBuffer(index, IID_PPV_ARGS(&surface))))
                return refuse();
            D2D1_BITMAP_PROPERTIES1 properties{};
            properties.pixelFormat.format = pixelFormat;
            properties.pixelFormat.alphaMode = D2D1_ALPHA_MODE_IGNORE;
            properties.bitmapOptions = D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW;
            properties.dpiX = properties.dpiY = 96.0f;
            if (FAILED(context->CreateBitmapFromDxgiSurface(surface.Get(),
                &properties, buffers[index].GetAddressOf())))
                return refuse();
        }
        // Preserve the game's shader, targets, blend, viewport, resource
        // and rasterizer bindings. Never clear or replace the game context.
        ComPtr<ID3DDeviceContextState> gameState;
        immediate->SwapDeviceContextState(f3ContextState.Get(),
            gameState.GetAddressOf());
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
        // Restore on both success and failed EndDraw, BEFORE any release.
        immediate->SwapDeviceContextState(gameState.Get(), nullptr);
        if (FAILED(result)) return refuse();
        // A failed EndDraw releases all interop targets and stays refused
        // until ResizeBuffers/teardown, never an automatic per-Present retry.
        return true;
    }
};

struct Session {
    SRWLOCK lock = SRWLOCK_INIT;
    HWND owner = nullptr;
    DWORD ownerThread = 0;
    bool claimed = false, shown = false, f3Held = false, chord = false, fresh = false;
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
        && frame.capturedTickMs != 0
        && within(frame.x, 30000000.) && within(frame.z, 30000000.)
        && within(frame.y, 30000000.) && within(frame.pitch, 90.)
        && within(frame.yaw, 1000000.)
        && (frame.dimension == Dimension::Overworld
            || frame.dimension == Dimension::Nether
            || frame.dimension == Dimension::End);
}
bool sampleFresh(const PlayerFrame& frame, ULONGLONG now) noexcept {
    return validFrame(frame) && frame.capturedTickMs <= now
        && now - frame.capturedTickMs <= FrameFreshnessMs;
}
bool live(Session& state) noexcept {
    const ULONGLONG now = GetTickCount64();
    return state.fresh && state.worldGeneration == state.frame.worldGeneration
        && state.observed == state.frame.capturedTickMs
        && sampleFresh(state.frame, now);
}
void hide(Session& state) noexcept {
    state.shown = state.f3Held = state.chord = false;
}
std::int32_t blockCoordinate(double value) noexcept {
    // Called exclusively after validFrame: finite world coordinates are
    // bounded to 30 million, well inside a signed 32-bit integer.
    // Java/Minecraft block semantics are FLOOR (not truncation), including
    // negative fractional XYZ. An integer also cannot format as "-0".
    return static_cast<std::int32_t>(std::floor(value));
}
bool copyCoordinates(HWND owner, const PlayerFrame& frame) noexcept {
    wchar_t coordinates[128]{};
    const int size = swprintf_s(coordinates, L"%d %d %d",
        blockCoordinate(frame.x), blockCoordinate(frame.y),
        blockCoordinate(frame.z));
    if (size <= 0) return false;
    // Prepare all memory BEFORE taking the process-global Windows clipboard.
    // EmptyClipboard can send WM_DESTROYCLIPBOARD to the previous owner.
    // NEVER hold session.lock across Open/Empty/Set/CloseClipboard.
    const SIZE_T bytes = (static_cast<SIZE_T>(size) + 1) * sizeof(wchar_t);
    HGLOBAL memory = GlobalAlloc(GMEM_MOVEABLE, bytes);
    if (!memory) return false;
    void* destination = GlobalLock(memory);
    if (!destination) {
        GlobalFree(memory);
        return false;
    }
    memcpy(destination, coordinates, bytes);
    GlobalUnlock(memory);
    if (!OpenClipboard(owner)) {
        GlobalFree(memory);
        return false;
    }
    const bool success = EmptyClipboard()
        && SetClipboardData(CF_UNICODETEXT, memory) != nullptr;
    // Successful SetClipboardData transfers the memory ownership to
    // Windows. On failure the producer still owns it and must free it.
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
        const ULONGLONG now = GetTickCount64();
        const bool staleGeneration = session.worldGeneration != 0
            && frame.worldGeneration < session.worldGeneration;
        const bool staleCapture = session.observed != 0
            && frame.capturedTickMs <= session.observed;
        // The actual engine owner timestamps the Player snapshot at capture.
        // A delayed callback CANNOT refresh its expiry at queue delivery.
        // World generation and captured time are strictly monotonic.
        if (staleGeneration || staleCapture) {
            // Never revoke a newer good frame because an older world,
            // queued packet or invalid old callback arrived late.
        } else if (sampleFresh(frame, now)) {
            if (session.worldGeneration && session.worldGeneration != frame.worldGeneration)
                hide(session); // A new world cannot inherit old F3 state.
            session.frame = frame;
            session.worldGeneration = frame.worldGeneration;
            session.observed = frame.capturedTickMs;
            session.fresh = true;
            accepted = true;
        } else if (!validFrame(frame)) {
            // Only a non-stale invalid current-player reading may revoke
            // earlier proof. Expired valid data is simply not promoted;
            // the original 250-ms expiry still hides it on next Present.
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
    PlayerFrame requestedCopy{};
    bool copyRequested = false;
    AcquireSRWLockExclusive(&session.lock);
    const bool admitted = session.claimed && session.owner == game
        && session.ownerThread == gameThread
        && GetCurrentThreadId() == gameThread
        && exactWindow(game, gameThread)
        && GetForegroundWindow() == game
        && gameplay && physical && live(session);
    if (!admitted) {
        hide(session);
    } else {
        // F3 must be the game's ordinary keyboard key, not Alt+F3 or
        // another system-key accelerator. Bit 30 rejects autorepeat.
        // Input provenance (gameplay/physical) remains the responsibility
        // of a future exact Education engine key adapter.
        const bool keyDown = message == WM_KEYDOWN;
        const bool keyUp = message == WM_KEYUP;
        const bool altContext = (static_cast<ULONGLONG>(parameters) & (1ULL << 29)) != 0;
        const bool repeated = (static_cast<ULONGLONG>(parameters) & (1ULL << 30)) != 0;
        if (key == VK_F3 && keyDown && !altContext && !repeated && !session.f3Held) {
            session.f3Held = true;
            session.chord = false;
        } else if (key == 'C' && keyDown && !altContext && !repeated
            && session.f3Held && !session.chord && session.shown) {
            session.chord = true;
            requestedCopy = session.frame;
            copyRequested = true;
            // Do NOT call Windows clipboard functions under session.lock:
            // they can reenter the UI message loop and block DXGI Present.
        } else if (key == VK_F3 && (keyUp || message == WM_SYSKEYUP)) {
            // Always clear a held F3 when released, including if Alt was
            // added during the hold; never toggle for a system-key release.
            if (keyUp && !altContext && session.f3Held && !session.chord) {
                session.shown = !session.shown;
                result = session.shown ? KeyResult::ShowRequested : KeyResult::HideRequested;
            }
            session.f3Held = false;
            session.chord = false;
        } else if (key == VK_F3 && message == WM_SYSKEYDOWN) {
            session.f3Held = false;
            session.chord = false;
        }
    }
    ReleaseSRWLockExclusive(&session.lock);
    if (copyRequested) {
        // A captured frame is a value from the exact physical key event,
        // NOT live game data pulled from a second player by name or PID.
        // Recheck that the original F3 action and its owned Player/world
        // have not been revoked since the lock was released.
        AcquireSRWLockShared(&session.lock);
        const bool current = session.claimed && session.owner == game
            && session.ownerThread == gameThread && session.shown
            && session.worldGeneration == requestedCopy.worldGeneration
            && session.observed == requestedCopy.capturedTickMs
            && live(session);
        ReleaseSRWLockShared(&session.lock);
        const bool foreground = GetCurrentThreadId() == gameThread
            && exactWindow(game, gameThread) && GetForegroundWindow() == game;
        result = current && foreground
            && sampleFresh(requestedCopy, GetTickCount64())
            && copyCoordinates(game, requestedCopy)
            ? KeyResult::Copied : KeyResult::CopyFailed;
        // A later world transition cannot undo an already-successful user
        // clipboard request; the copied value is from the keypress snapshot.
    }
    SetLastError(error);
    return result;
}

void before_present(IDXGISwapChain1* original, HWND game, DWORD gameThread) noexcept {
    const DWORD error = GetLastError();
    AcquireSRWLockExclusive(&session.lock);
    // The original DXGI wrapper invokes us on every eligible Present.
    // A hidden F3 panel must never query the game's swapchain, initialize
    // D2D, create targets or sample FPS each frame. Drop any retained
    // D2D/backbuffer COM references once, on the next normal Present after
    // F3 was hidden. Rapid hide/show before that Present may reuse them.
    if (!session.claimed || !session.shown) {
        if (session.painter.ownsBackbuffers()) session.painter.release();
        ReleaseSRWLockExclusive(&session.lock);
        SetLastError(error);
        return;
    }
    const bool eligible = session.owner == game
        && session.ownerThread == gameThread && exactWindow(game, gameThread)
        && GetForegroundWindow() == game && live(session) && original;
    if (!eligible) {
        hide(session); // No stale data, background focus or absent Player.
    } else {
        ComPtr<IDXGISwapChain4> chain;
        if (FAILED(original->QueryInterface(IID_PPV_ARGS(&chain)))
            || !session.painter.prepare(chain.Get())) {
            hide(session);
        } else {
            if (session.shown) {
                const auto& f = session.frame;
                wchar_t text[768]{}, fps[40]{};
                // No FPS telemetry work on the ordinary hidden-F3
                // Present path. The existing native HUD meter remains
                // the sole source, sampled only when text is drawn.
                const auto reading = dxgi_pacing::hud_snapshot().reading;
                if (reading.valid) {
                    swprintf_s(fps, L"%.1f",
                        static_cast<double>(reading.milliFps) / 1000.);
                } else {
                    wcscpy_s(fps, L"--");
                }
                const std::int32_t bx = blockCoordinate(f.x);
                const std::int32_t by = blockCoordinate(f.y);
                const std::int32_t bz = blockCoordinate(f.z);
                const std::int32_t cx = blockCoordinate(static_cast<double>(bx) / 16.);
                const std::int32_t cz = blockCoordinate(static_cast<double>(bz) / 16.);
                const int written = swprintf_s(text,
                    L"Minecraft Education | F3 Debug\n"
                    L"XYZ: %.2f / %.2f / %.2f\n"
                    L"Block: %d / %d / %d\n"
                    L"Chunk: %d / %d\n"
                    L"Facing: %ls\n"
                    L"Yaw / Pitch: %.2f / %.2f\n"
                    L"Dimension: %ls\n"
                    L"FPS: %ls\n"
                    L"F3: Hide    F3+C: Copy XYZ",
                    f.x, f.y, f.z, bx, by, bz, cx, cz,
                    facing(f.yaw), f.yaw, f.pitch, dimension(f.dimension), fps);
                if (written <= 0 || !session.painter.draw(chain.Get(), text)) {
                    hide(session);
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
        session.claimed = session.fresh = false;
        session.owner = nullptr; session.ownerThread = 0;
        session.observed = 0; session.worldGeneration = 0;
        session.frame = {};
    }
    ReleaseSRWLockExclusive(&session.lock);
    SetLastError(error);
}
} // namespace education_client_f3
