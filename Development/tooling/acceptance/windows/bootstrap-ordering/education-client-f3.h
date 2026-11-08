#pragma once
#include <windows.h>
#include <dxgi1_2.h>
#include <cstdint>

// Education CLIENT only. No world Add-On/Script API, desktop app, layered
// window, process manager or game binary writes. Entire source is compiled
// only by the separately opted-in, non-product CMake experiment.
// IMPORTANT: The actual Education Player/input ABI has NOT been established.
namespace education_client_f3 {

enum class Dimension : std::uint32_t { Overworld = 0, Nether = 1, End = 2 };
struct PlayerFrame {
    std::uint32_t bytes = sizeof(PlayerFrame);
    // Original engine adapter captures this at the SAME time as the
    // Player snapshot, from GetTickCount64 in this client process.
    // A delay between capture and publish must not renew its lifetime.
    std::uint64_t worldGeneration = 0; // Strictly increasing per owned game process.
    std::uint64_t capturedTickMs = 0;
    double x = 0, y = 0, z = 0;
    double yaw = 0, pitch = 0;
    Dimension dimension = Dimension::Overworld;
};
enum class KeyResult { Ignored, ShowRequested, HideRequested, Copied, CopyFailed };
// KeyResult is input intent, never an in-game rendering success receipt.

// This claim MUST originate from a separately reviewed, version-pinned
// in-game engine adapter on the original OGLES window thread. It is NOT
// called by DXGI code and is NOT authorization by a HWND alone.
bool claim_game_window(HWND game, DWORD gameThread) noexcept;

// A verified engine adapter publishes actual local Player data (not screen
// OCR, a PID/username, world telemetry or desktop bookmarks). The adapter
// and its engine-safe read thread are NOT implemented at this source stage.
// capturedTickMs and worldGeneration must originate in that same engine
// callback. Stale, delayed or regressing generations are not admitted.
bool publish_player(HWND game, const PlayerFrame& frame) noexcept;

// Called ONLY by a verified game keyboard dispatch owner on its own thread.
// WM_KEYDOWN/WM_KEYUP F3 are accepted, NOT WM_SYSKEYDOWN/Alt+F3;
// F3+C only acts during the user's explicit held F3 gesture.
// Clipboard Win32 calls happen AFTER releasing the F3 session lock;
// revalidate the same Player/world capture and foreground before copying.
// No SetWindowsHookEx, subclass, polling, injected or global hotkey is
// installed here. "physical" and "gameplay" need actual engine provenance.
KeyResult game_key(HWND game, DWORD gameThread, UINT message, WPARAM key,
                   LPARAM parameters, bool gameplay, bool physical) noexcept;

// Composes directly into a compatible D3D11 DXGI game backbuffer, NOT an
// independent Windows layered window. Does nothing until both verified
// game-specific owners above have published a fresh Player frame.
// FPS is read lazily from the ONE existing DXGI policy only when a
// visible, current Player frame requires an actual D3D11 draw.
void before_present(IDXGISwapChain1* original, HWND game, DWORD gameThread) noexcept;
// Release F3-held backbuffers BEFORE native ResizeBuffers/ResizeBuffers1,
// ResizeTarget and SetFullscreenState; reset cached render admission only
// on these genuine DXGI mode transitions or final chain retirement.
void before_resize(HWND game) noexcept;
void retire(HWND game) noexcept;
} // namespace education_client_f3
