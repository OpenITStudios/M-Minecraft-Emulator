#pragma once
#include <cstdint>

namespace graphics_warm_pools {
// Startup capacity only. Native allocation, growth and GPU retirement remain game-owned.
constexpr unsigned initial_default_heaps(const wchar_t* name, unsigned heapType,
    unsigned flags, std::uint64_t bytes, std::uint64_t alignment,
    unsigned initial, unsigned reserve) noexcept {
    constexpr wchar_t expected[] = L"DefaultBufferHeap";
    if (!name) return initial;
    for (unsigned i = 0; i < sizeof(expected) / sizeof(*expected); ++i)
        if (name[i] != expected[i]) return initial;
    return heapType == 1 && flags == 0 && bytes == 16ULL * 1024 * 1024
        && alignment == 65536 && initial == 16 && reserve == 1 ? 4 : initial;
}
// Called after native factory creation, outside loader callbacks. Disabled unless built in explicitly.
void consider() noexcept;
}
