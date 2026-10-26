#pragma once
#include <cstdint>

namespace graphics_warm_pools {
struct HeapProperties {
    unsigned type, cpuPageProperty, memoryPool, creationNodeMask, visibleNodeMask;
};
// Legacy count canary also changes the native total-heap retention floor.
constexpr unsigned initial_default_heaps(const wchar_t* name, const HeapProperties& properties,
    unsigned flags, std::uint64_t bytes, std::uint64_t alignment,
    unsigned initial, unsigned reserve) noexcept {
    constexpr wchar_t expected[] = L"DefaultBufferHeap";
    if (!name) return initial;
    for (unsigned i = 0; i < sizeof(expected) / sizeof(*expected); ++i)
        if (name[i] != expected[i]) return initial;
    return properties.type == 4 && properties.cpuPageProperty == 1 && properties.memoryPool == 2
        && properties.creationNodeMask == 1 && properties.visibleNodeMask == 1
        && flags == 0 && bytes == 16ULL * 1024 * 1024
        && alignment == 65536 && initial == 16 && reserve == 1 ? 4 : initial;
}
// Independent Upload retention canary; never changes the initial/total-heap floor.
constexpr unsigned upload_empty_retention(const wchar_t* name, unsigned flags,
    std::uint64_t bytes, std::uint64_t alignment, unsigned initial, unsigned reserve) noexcept {
    constexpr wchar_t expected[] = L"UploadBufferHeap";
    if (!name) return reserve;
    for (unsigned i = 0; i < sizeof(expected) / sizeof(*expected); ++i)
        if (name[i] != expected[i]) return reserve;
    return flags == 0 && bytes == 8ULL * 1024 * 1024 && alignment == 65536
        && initial == 2 && reserve == 4 ? 1 : reserve;
}
// Called after native factory creation, outside loader callbacks. Disabled unless built in explicitly.
void consider() noexcept;
}
