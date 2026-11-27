#pragma once
#include <windows.h>
#include <cstdint>
#include <cstddef>

// Single native FPS boundary. Internal Rust modules do not use C++ headers.

// Exact Rust repr(C) ABI. Inline state stays owned by the existing C++ Meter.
struct LazyTesterFpsMeterState {
    std::int64_t frequency, first, last;
    std::uint64_t intervals, accepted;
    std::uint32_t milliFps, valid;
};
struct LazyTesterFpsMeterReading {
    std::uint64_t accepted;
    std::uint32_t milliFps, valid;
};
static_assert(sizeof(LazyTesterFpsMeterState) == 48 && alignof(LazyTesterFpsMeterState) == 8);
static_assert(offsetof(LazyTesterFpsMeterState, accepted) == 32 && offsetof(LazyTesterFpsMeterState, valid) == 44);
static_assert(sizeof(LazyTesterFpsMeterReading) == 16 && offsetof(LazyTesterFpsMeterReading, valid) == 12);
extern "C" {
void lazytester_fps_meter_init(LazyTesterFpsMeterState*, std::int64_t frequency);
void lazytester_fps_meter_reset(LazyTesterFpsMeterState*);
void lazytester_fps_meter_record(LazyTesterFpsMeterState*, std::int64_t now);
void lazytester_fps_meter_read(const LazyTesterFpsMeterState*, std::int64_t now, LazyTesterFpsMeterReading*);
}

struct LazyTesterNativePacer;
struct LazyTesterFramePermit { std::uint32_t outcome, ceiling; std::int64_t released; };
static_assert(sizeof(LazyTesterFramePermit) == 16 && offsetof(LazyTesterFramePermit, released) == 8);

// Fixed-width C ABI: FramePermit outcome 0 = frame, 1 = stopped, 2 = error.
extern "C" {
LazyTesterNativePacer* lazytester_pacer_create(void* level, void* changed, void* stop, void* controller, std::uint32_t active, std::uint32_t background);
void lazytester_pacer_destroy(LazyTesterNativePacer*);
std::uint32_t lazytester_pacer_foreground(const LazyTesterNativePacer*, std::uint32_t active);
std::uint32_t lazytester_pacer_stop(const LazyTesterNativePacer*);
void lazytester_pacer_wait(const LazyTesterNativePacer*, LazyTesterFramePermit*);
std::uint32_t lazytester_frame_pacing_valid_ceilings(std::uint32_t active, std::uint32_t background);
std::uint32_t lazytester_frame_pacing_valid_frequency(std::int64_t frequency);
std::int64_t lazytester_frame_pacing_deadline(std::int64_t frequency, std::int64_t previous, std::int64_t now, std::uint32_t ceiling);
}

struct LazyTesterHudCanvas;
struct LazyTesterHudCanvasView { HDC dc; const DWORD* pixels; SIZE size; };
static_assert(sizeof(LazyTesterHudCanvasView)==24 && alignof(LazyTesterHudCanvasView)==8);
static_assert(offsetof(LazyTesterHudCanvasView,pixels)==8 && offsetof(LazyTesterHudCanvasView,size)==16);
extern "C" {
void lazytester_hud_placement(const RECT*, std::uint32_t, RECT*);
std::uint32_t lazytester_hud_physical_client(HWND, RECT*);
LazyTesterHudCanvas* lazytester_hud_canvas_create();
void lazytester_hud_canvas_destroy(LazyTesterHudCanvas*);
std::uint32_t lazytester_hud_canvas_paint(LazyTesterHudCanvas*, std::uint32_t, std::uint32_t, std::uint32_t, LazyTesterHudCanvasView*);
}

struct LazyTesterNativeHud;
struct LazyTesterHudSnapshot {
    std::uint64_t accepted; std::uint32_t milli, valid; HWND window; RECT bounds;
    std::uint32_t dpi, draws, failures, visible, active;
};
static_assert(sizeof(LazyTesterHudSnapshot)==64 && alignof(LazyTesterHudSnapshot)==8);
static_assert(offsetof(LazyTesterHudSnapshot,window)==16 && offsetof(LazyTesterHudSnapshot,bounds)==24);
static_assert(offsetof(LazyTesterHudSnapshot,dpi)==40 && offsetof(LazyTesterHudSnapshot,active)==56);
extern "C" {
LazyTesterNativeHud* lazytester_hud_create(HANDLE,HANDLE,HANDLE);
void lazytester_hud_destroy(LazyTesterNativeHud*);
std::uint32_t lazytester_hud_bind(const LazyTesterNativeHud*,HWND,DWORD);
void lazytester_hud_unbind(const LazyTesterNativeHud*);
void lazytester_hud_present(const LazyTesterNativeHud*,HRESULT,UINT);
void lazytester_hud_snapshot(const LazyTesterNativeHud*,LazyTesterHudSnapshot*);
std::uint32_t lazytester_hud_selection(std::uint32_t,HANDLE,HANDLE,HANDLE);
}

extern "C" std::uint32_t lazytester_presentation_limits(std::uint32_t version, std::uint32_t reserved, std::uint32_t* active, std::uint32_t* background);

extern "C" std::uint32_t lazytester_presentation_validate(std::uint32_t version, std::uint32_t reserved,
    HANDLE level, HANDLE changed, HANDLE stop, HANDLE controller, HANDLE selected,
    std::uint32_t controllerPid, std::uint32_t birthLow, std::uint32_t birthHigh,
    std::uint32_t childPid, std::uint32_t childSession, std::uint32_t* active, std::uint32_t* background);

struct LazyTesterPresentationImage { std::uint32_t timestamp,bytes; FILE_ID_INFO id; };
static_assert(sizeof(LazyTesterPresentationImage)==32 && offsetof(LazyTesterPresentationImage,id)==8);
extern "C" {
std::uint32_t lazytester_presentation_image(LazyTesterPresentationImage*,HMODULE*);
std::uint32_t lazytester_presentation_bytes(std::uint32_t pacing,std::uint32_t version);
void* lazytester_presentation_entry(HMODULE,std::uint32_t imageBytes,std::uint32_t rva);
}

struct LazyTesterPresentationAdmission {
    std::uint32_t mode,active,background,reserved;
    void* oldEntry; void* hwndEntry; HANDLE level,changed,stop,controller,selected;
};
static_assert(sizeof(LazyTesterPresentationAdmission)==72 && offsetof(LazyTesterPresentationAdmission,oldEntry)==16);
static_assert(alignof(LazyTesterPresentationAdmission)==8 && offsetof(LazyTesterPresentationAdmission,selected)==64);
extern "C" std::uint32_t lazytester_presentation_admit(std::uint32_t pid,std::uint32_t session,
    std::uint32_t birthLow,std::uint32_t birthHigh,LazyTesterPresentationAdmission*);

extern "C" std::uint32_t lazytester_presentation_candidate(HWND window);
