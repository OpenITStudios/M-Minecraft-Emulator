#pragma once
#include <cstdint>

// Fixed-width C ABI; zero releases, negative waits, positive refuses.
extern "C" {
std::uint32_t lazytester_frame_pacing_valid_ceilings(std::uint32_t active, std::uint32_t background);
std::uint32_t lazytester_frame_pacing_valid_frequency(std::int64_t frequency);
std::int64_t lazytester_frame_pacing_deadline(std::int64_t frequency, std::int64_t previous, std::int64_t now, std::uint32_t ceiling);
}
