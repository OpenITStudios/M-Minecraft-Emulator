#pragma once
#include <dxgi1_2.h>
#include "frame-pacing.h"
#include "fps-hud.h"

namespace dxgi_pacing {
struct Snapshot {
    unsigned long long pacedPresent, pacedPresent1, bypassed, faults;
    LONGLONG lastRelease;
    unsigned lastCeiling, liveBindings;
    unsigned long long createdBindings, destroyedBindings, unsupportedInterfaces;
};
// Outside loader lock, before publication. Success replaces the caller's one
// owned reference; failure preserves it. Native objects/vtables are unchanged.
// Optional external policy, once, outside loader lock and before wrap.
bool initialize(const presentation_pacing::Input& input) noexcept;
bool wrap(IDXGISwapChain1** owned, HWND window) noexcept;
bool foreground(bool active) noexcept;
bool stop() noexcept;
Snapshot snapshot() noexcept;
fps_hud::Snapshot hud_snapshot() noexcept;
}
