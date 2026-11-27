#pragma once
#include <cstdint>
#include <cstddef>
struct LazyTesterNativePacer;
struct LazyTesterFramePermit { std::uint32_t outcome, ceiling; std::int64_t released; };
static_assert(sizeof(LazyTesterFramePermit) == 16 && offsetof(LazyTesterFramePermit, released) == 8);

// Fixed-width C ABI; zero releases, negative waits, positive refuses.
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
