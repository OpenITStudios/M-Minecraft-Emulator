#pragma once
#include <windows.h>
#include <cstdint>
struct LazyTesterHudWindowHost;
using LazyTesterHudNotify = void (*)(void*, HWND);
extern "C" {
LazyTesterHudWindowHost* lazytester_hud_window_host_create(void*, LazyTesterHudNotify);
void lazytester_hud_window_host_destroy(LazyTesterHudWindowHost*);
void lazytester_hud_window_clear(LazyTesterHudWindowHost*);
HWND lazytester_hud_window_get(LazyTesterHudWindowHost*, HWND, LONG, LONG);
std::uint32_t lazytester_hud_window_wait(HANDLE, HANDLE);
}

struct LazyTesterHudRenderer;
struct LazyTesterHudComposition { HWND window; RECT bounds; std::uint32_t dpi, visible, drawn; };
static_assert(sizeof(LazyTesterHudComposition)==40);
extern "C" {
LazyTesterHudRenderer* lazytester_hud_renderer_create();
void lazytester_hud_renderer_destroy(LazyTesterHudRenderer*);
void lazytester_hud_renderer_reset(LazyTesterHudRenderer*);
std::uint32_t lazytester_hud_renderer_update(LazyTesterHudRenderer*, LazyTesterHudWindowHost*, HWND,
    std::uint32_t, std::uint32_t, std::uint32_t, std::uint32_t, LazyTesterHudComposition*);
}
