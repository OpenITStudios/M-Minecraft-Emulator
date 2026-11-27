#pragma once
#include <windows.h>
#include <cstdint>
struct LazyTesterHudCanvas;
struct LazyTesterHudCanvasView { HDC dc; const DWORD* pixels; SIZE size; };
static_assert(sizeof(LazyTesterHudCanvasView)==24);
extern "C" {
void lazytester_hud_placement(const RECT*, std::uint32_t, RECT*);
std::uint32_t lazytester_hud_physical_client(HWND, RECT*);
LazyTesterHudCanvas* lazytester_hud_canvas_create();
void lazytester_hud_canvas_destroy(LazyTesterHudCanvas*);
std::uint32_t lazytester_hud_canvas_paint(LazyTesterHudCanvas*, std::uint32_t, std::uint32_t, std::uint32_t, LazyTesterHudCanvasView*);
}
