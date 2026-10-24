#pragma once
#include <dxgi1_2.h>

namespace dxgi_pacing {
struct Snapshot {
    unsigned long long pacedPresent, pacedPresent1, bypassed, faults;
    LONGLONG lastRelease;
    unsigned lastCeiling, liveBindings;
    unsigned long long createdBindings, destroyedBindings, unsupportedInterfaces;
};
// Outside loader lock, before publication. Success replaces the caller's one
// owned reference; failure preserves it. Native objects/vtables are unchanged.
bool wrap(IDXGISwapChain1** owned, HWND window) noexcept;
bool foreground(bool active) noexcept;
bool stop() noexcept;
Snapshot snapshot() noexcept;
}
