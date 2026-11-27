#include "fps-hud.h"
#include <stdexcept>

namespace fps_hud {
// Native filter, clock and synchronization stay at the Windows boundary.
void Meter::reset() noexcept {
    AcquireSRWLockExclusive(&gate_);
    lazytester_fps_meter_reset(&state_);
    ReleaseSRWLockExclusive(&gate_);
}
void Meter::record(LONGLONG now, HRESULT result, UINT flags) noexcept {
    if (result != S_OK || (flags & DXGI_PRESENT_TEST) || now <= 0) return;
    AcquireSRWLockExclusive(&gate_);
    lazytester_fps_meter_record(&state_, now);
    ReleaseSRWLockExclusive(&gate_);
}
Reading Meter::read(LONGLONG now) noexcept {
    AcquireSRWLockShared(&gate_);
    if (!now) { LARGE_INTEGER value{}; if (QueryPerformanceCounter(&value)) now = value.QuadPart; }
    LazyTesterFpsMeterReading value{};
    lazytester_fps_meter_read(&state_, now, &value);
    ReleaseSRWLockShared(&gate_);
    return {value.accepted, value.milliFps, value.valid != 0};
}
RECT placement(RECT client, UINT dpi) noexcept {
    RECT result{}; lazytester_hud_placement(&client, dpi, &result); return result;
}
bool physicalClient(HWND owner, RECT& result) noexcept {
    return lazytester_hud_physical_client(owner, &result) != 0;
}
namespace {
struct Canvas {
    LazyTesterHudCanvas* state = lazytester_hud_canvas_create();
    HDC dc = nullptr;
    const DWORD* pixels = nullptr;
    Canvas() = default;
    Canvas(const Canvas&) = delete;
    Canvas& operator=(const Canvas&) = delete;
    ~Canvas() { lazytester_hud_canvas_destroy(state); }
};
void paint(Canvas& canvas, Reading value, UINT dpi, SIZE& size) {
    LazyTesterHudCanvasView view{};
    if (!lazytester_hud_canvas_paint(canvas.state, value.milliFps, value.valid ? 1U : 0U, dpi, &view))
        throw std::runtime_error("HUD raster unavailable");
    canvas.dc = view.dc; canvas.pixels = view.pixels; size = view.size;
}
}
std::vector<DWORD> raster(Reading value, UINT dpi, SIZE& size) {
    Canvas canvas; paint(canvas, value, dpi, size);
    return {canvas.pixels, canvas.pixels + static_cast<size_t>(size.cx) * size.cy};
}
bool selectionAllowsHud(bool managed, HANDLE selected, HANDLE stop, HANDLE controller) noexcept {
    return lazytester_hud_selection(managed ? 1U : 0U, selected, stop, controller) != 0;
}
Hud::Hud(HANDLE selected, HANDLE stop, HANDLE controller) : state_(lazytester_hud_create(selected,stop,controller)) {
    if (!state_) throw std::runtime_error("HUD capability unavailable");
}
Hud::~Hud() { lazytester_hud_destroy(state_); }
bool Hud::bind(HWND owner, DWORD thread) noexcept { return lazytester_hud_bind(state_,owner,thread)!=0; }
void Hud::unbind() noexcept { lazytester_hud_unbind(state_); }
void Hud::present(HRESULT result, UINT flags) noexcept { lazytester_hud_present(state_,result,flags); }
Snapshot Hud::snapshot() noexcept {
    LazyTesterHudSnapshot value{}; lazytester_hud_snapshot(state_,&value);
    return {{value.accepted,value.milli,value.valid!=0},value.window,value.bounds,value.dpi,
        value.draws,value.failures,value.visible!=0,value.active!=0};
}
}
