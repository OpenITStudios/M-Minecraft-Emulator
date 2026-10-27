#pragma once
#include <windows.h>
#include <dxgi.h>
#include <memory>
#include <vector>

namespace fps_hud {
struct Reading { unsigned long long accepted = 0; unsigned milliFps = 0; bool valid = false; };
// Accepted native Present/Present1 rate, not scanout/display or a configured ceiling.
class Meter {
    SRWLOCK gate_ = SRWLOCK_INIT;
    LONGLONG frequency_, first_ = 0, last_ = 0;
    unsigned long long intervals_ = 0;
    Reading reading_{};
public:
    explicit Meter(LONGLONG frequency) : frequency_(frequency > 0 && frequency <= 1000000000000LL ? frequency : 0) {}
    void reset() noexcept;
    void record(LONGLONG now, HRESULT result, UINT flags) noexcept;
    Reading read(LONGLONG now = 0) noexcept; // Default samples QPC under the read lock.
};
RECT placement(RECT physicalClient, UINT dpi) noexcept;
bool physicalClient(HWND owner, RECT& result) noexcept;
std::vector<DWORD> raster(Reading value, UINT dpi, SIZE& size);
struct Snapshot {
    Reading reading;
    HWND window = nullptr;
    RECT bounds{};
    UINT dpi = 0;
    unsigned draws = 0, failures = 0;
    bool visible = false;
    // Synthetic/runtime diagnostic only: not a gameplay or FPS verdict.
    bool workerActive = false;
};
// Create/destroy outside loader lock; join rendering callers before destruction.
// One process policy retains this object across native chain recreation.
// Its existing installation/lifetime owner serializes bind/unbind; present/read may race.
class Hud {
    struct State;
    std::unique_ptr<State> state_;
public:
    Hud();
    ~Hud();
    bool bind(HWND owner, DWORD thread) noexcept;
    void unbind() noexcept;
    void present(HRESULT result, UINT flags) noexcept;
    Snapshot snapshot() noexcept;
};
}
