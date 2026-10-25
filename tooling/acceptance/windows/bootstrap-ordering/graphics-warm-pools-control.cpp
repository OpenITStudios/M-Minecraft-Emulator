#include "graphics-warm-pools.h"
#include <windows.h>
#include <cstdio>

int main() {
    DWORD before = 0, after = 0;
    if (!GetProcessHandleCount(GetCurrentProcess(), &before)) return 2;
    graphics_warm_pools::consider(); // This control's image must be rejected before private game RVAs are read.
    if (!GetProcessHandleCount(GetCurrentProcess(), &after) || before != after) return 2;
    using graphics_warm_pools::initial_default_heaps;
    const auto decide = [](const wchar_t* name, unsigned type, unsigned flags,
        std::uint64_t bytes, std::uint64_t alignment, unsigned count, unsigned reserve) {
        return initial_default_heaps(name, {type, 1, 2, 1, 1}, flags, bytes, alignment, count, reserve);
    };
    const std::uint64_t block = 16ULL * 1024 * 1024;
    if (decide(L"DefaultBufferHeap", 4, 0, block, 65536, 16, 1) != 4
        || decide(nullptr, 4, 0, block, 65536, 16, 1) != 16
        || decide(L"UploadBufferHeap", 4, 0, block, 65536, 16, 1) != 16
        || decide(L"DefaultBufferHeap", 1, 0, block, 65536, 16, 1) != 16
        || decide(L"DefaultBufferHeap", 4, 1, block, 65536, 16, 1) != 16
        || decide(L"DefaultBufferHeap", 4, 0, block / 2, 65536, 16, 1) != 16
        || decide(L"DefaultBufferHeap", 4, 0, block, 256, 16, 1) != 16
        || decide(L"DefaultBufferHeap", 4, 0, block, 65536, 8, 1) != 8
        || decide(L"DefaultBufferHeap", 4, 0, block, 65536, 16, 4) != 16) return 1;
    for (unsigned field = 1; field < 5; ++field) {
        unsigned values[]{4, 1, 2, 1, 1}; values[field] = 0;
        if (initial_default_heaps(L"DefaultBufferHeap", {values[0], values[1], values[2], values[3], values[4]},
            0, block, 65536, 16, 1) != 16) return 1;
    }
    std::puts("Only the exact observed DefaultBufferHeap tuple changes; native growth is outside this policy.");
    return 0;
}
