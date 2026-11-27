#pragma once
#include <cstdint>
#include <cstddef>

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
static_assert(sizeof(LazyTesterFpsMeterReading) == 16 && offsetof(LazyTesterFpsMeterReading, valid) == 12);
extern "C" {
void lazytester_fps_meter_init(LazyTesterFpsMeterState*, std::int64_t frequency);
void lazytester_fps_meter_reset(LazyTesterFpsMeterState*);
void lazytester_fps_meter_record(LazyTesterFpsMeterState*, std::int64_t now);
void lazytester_fps_meter_read(const LazyTesterFpsMeterState*, std::int64_t now, LazyTesterFpsMeterReading*);
}
