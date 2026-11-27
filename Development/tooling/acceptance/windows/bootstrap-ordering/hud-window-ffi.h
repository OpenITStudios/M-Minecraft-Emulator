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
