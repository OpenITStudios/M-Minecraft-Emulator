#pragma once
#include <windows.h>
#include <cstdint>
struct LazyTesterNativeHud;
struct LazyTesterHudSnapshot {
    std::uint64_t accepted; std::uint32_t milli, valid; HWND window; RECT bounds;
    std::uint32_t dpi, draws, failures, visible, active;
};
static_assert(sizeof(LazyTesterHudSnapshot)==64);
extern "C" {
LazyTesterNativeHud* lazytester_hud_create(HANDLE,HANDLE,HANDLE);
void lazytester_hud_destroy(LazyTesterNativeHud*);
std::uint32_t lazytester_hud_bind(const LazyTesterNativeHud*,HWND,DWORD);
void lazytester_hud_unbind(const LazyTesterNativeHud*);
void lazytester_hud_present(const LazyTesterNativeHud*,HRESULT,UINT);
void lazytester_hud_snapshot(const LazyTesterNativeHud*,LazyTesterHudSnapshot*);
std::uint32_t lazytester_hud_selection(std::uint32_t,HANDLE,HANDLE,HANDLE);
}
