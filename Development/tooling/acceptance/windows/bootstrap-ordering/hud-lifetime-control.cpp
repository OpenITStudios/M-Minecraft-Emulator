#include "fps-hud.h"
#include <windows.h>
int main() {
    WNDCLASSW c{}; c.lpfnWndProc=DefWindowProcW; c.hInstance=GetModuleHandleW(nullptr); c.lpszClassName=L"LazyTesterHudLifetimeFixture";
    if (!RegisterClassW(&c)) return 1;
    HWND w=CreateWindowExW(WS_EX_NOACTIVATE|WS_EX_TOOLWINDOW,c.lpszClassName,L"",WS_POPUP,0,0,640,480,nullptr,nullptr,c.hInstance,nullptr);
    if (!w) return 2;
    auto *hud=new fps_hud::Hud();
    if (!hud->bind(w,GetCurrentThreadId())) return 3;
    Sleep(300);
    hud->unbind(); DestroyWindow(w); Sleep(300);
    ExitThread(0);
}
