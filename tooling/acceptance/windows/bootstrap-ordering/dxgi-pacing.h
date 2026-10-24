#pragma once
#include <dxgi1_2.h>

// Process-lifetime native experiment; install on an owned, unpublished chain,
// outside loader lock. It neither discovers nor injects into a game. No live DLL unload.
// Retained references block flip recreation in the control: not ready for game deployment.
namespace dxgi_pacing {
struct Snapshot {
    unsigned long long pacedPresent, pacedPresent1, bypassed, faults;
    LONGLONG lastRelease;
    unsigned lastCeiling;
};
bool install(IDXGISwapChain1* chain, HWND window) noexcept;
bool foreground(bool active) noexcept;
bool stop() noexcept;
Snapshot snapshot() noexcept;
}
