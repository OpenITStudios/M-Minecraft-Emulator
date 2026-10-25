#include "graphics-warm-pools.h"
#if defined(BOOTSTRAP_GRAPHICS_WARM_POOLS_EXPERIMENT)
#include <windows.h>
#include <tlhelp32.h>
#include <d3d12.h>
#include <bcrypt.h>
#include <detours.h>
#include <intrin.h>
#include <atomic>
#include <cstring>
#include <vector>

namespace graphics_warm_pools {
// Private action receipt, located through this candidate's linker map; no export or wire change.
std::atomic<unsigned long> successfulInitialReductions{0};
namespace {
using Initialize = bool (*)(void*, const wchar_t*, ID3D12Device*,
    const D3D12_HEAP_PROPERTIES*, D3D12_HEAP_FLAGS, std::uint64_t,
    std::uint64_t, unsigned, unsigned);
Initialize original = nullptr;
std::atomic<bool> installing{false}, installed{false};
BYTE* image = nullptr;

bool codeRange(BYTE* address, SIZE_T bytes) noexcept {
    for (BYTE* at = address; at < address + bytes;) {
        MEMORY_BASIC_INFORMATION memory{};
        if (!VirtualQuery(at, &memory, sizeof(memory)) || memory.State != MEM_COMMIT
            || memory.Type != MEM_IMAGE || memory.AllocationBase != image
            || (memory.Protect & PAGE_GUARD)
            || !(memory.Protect & (PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY))) return false;
        BYTE* next = static_cast<BYTE*>(memory.BaseAddress) + memory.RegionSize;
        if (next <= at) return false;
        at = next;
    }
    return true;
}
bool digest(BCRYPT_ALG_HANDLE algorithm, DWORD rva, ULONG bytes, const char* expected) noexcept {
    if (!codeRange(image + rva, bytes)) return false;
    UCHAR hash[32]{};
    if (BCryptHash(algorithm, nullptr, 0, image + rva, bytes, hash, sizeof(hash)) != 0) return false;
    constexpr char hex[] = "0123456789abcdef";
    for (unsigned i = 0; i < sizeof(hash); ++i)
        if (hex[hash[i] >> 4] != expected[i * 2] || hex[hash[i] & 15] != expected[i * 2 + 1]) return false;
    return true;
}
bool knownCode() noexcept {
    image = reinterpret_cast<BYTE*>(GetModuleHandleW(nullptr));
    if (!image) return false;
    const auto* dos = reinterpret_cast<const IMAGE_DOS_HEADER*>(image);
    if (dos->e_magic != IMAGE_DOS_SIGNATURE || dos->e_lfanew < 64 || dos->e_lfanew > 3072) return false;
    const auto* nt = reinterpret_cast<const IMAGE_NT_HEADERS64*>(image + dos->e_lfanew);
    if (nt->Signature != IMAGE_NT_SIGNATURE || nt->FileHeader.Machine != IMAGE_FILE_MACHINE_AMD64
        || nt->OptionalHeader.Magic != IMAGE_NT_OPTIONAL_HDR64_MAGIC
        || nt->FileHeader.TimeDateStamp != 0x6a63e832 || nt->OptionalHeader.SizeOfImage != 0x1255c000) return false;
    BCRYPT_ALG_HANDLE algorithm = nullptr;
    if (BCryptOpenAlgorithmProvider(&algorithm, BCRYPT_SHA256_ALGORITHM, nullptr, 0) != 0) return false;
    // Observed loaded-code hashes match in both original clients. No proprietary code bytes are embedded.
    const bool valid = digest(algorithm, 0xbc22120, 691, "86794e185e14e77601ebb39505b9d275941d30d48fb2efeb182adec918f715c9")
        && digest(algorithm, 0xbc22490, 402, "a2b9f04086e7c1e4a1d5153d694a161a031293cfdb535cc8a3f6887de2a3a2eb")
        && digest(algorithm, 0xbc22660, 402, "fe604388ba591130bc89c816353d9553c397333fdd2f5704a9f8cf960e09a182")
        && digest(algorithm, 0xbc22840, 2363, "75f850894d2902d7a5d61902c8e22c13f01eb68b43032f5e3dd4680b4c76285a");
    BCryptCloseAlgorithmProvider(algorithm, 0);
    return valid;
}
bool initialize(void* owner, const wchar_t* name, ID3D12Device* device,
    const D3D12_HEAP_PROPERTIES* properties, D3D12_HEAP_FLAGS flags,
    std::uint64_t bytes, std::uint64_t alignment, unsigned initial, unsigned reserve) {
    const DWORD error = GetLastError();
    const unsigned requested = initial;
    if (_ReturnAddress() == image + 0xb026c8c && name == reinterpret_cast<const wchar_t*>(image + 0xd0d5bb6)
        && properties == reinterpret_cast<const D3D12_HEAP_PROPERTIES*>(image + 0xf544910)) {
        const HeapProperties observed{static_cast<unsigned>(properties->Type),
            static_cast<unsigned>(properties->CPUPageProperty), static_cast<unsigned>(properties->MemoryPoolPreference),
            properties->CreationNodeMask, properties->VisibleNodeMask};
        initial = initial_default_heaps(name, observed,
            static_cast<unsigned>(flags), bytes, alignment, initial, reserve);
    }
    SetLastError(error);
    const bool result = original(owner, name, device, properties, flags, bytes, alignment, initial, reserve);
    if (result && initial != requested) successfulInitialReductions.fetch_add(1, std::memory_order_relaxed);
    return result;
}
bool attach() {
    HMODULE pinned = nullptr;
    if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
        reinterpret_cast<LPCWSTR>(initialize), &pinned)) return false;
    const HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
    if (snapshot == INVALID_HANDLE_VALUE) return false;
    std::vector<HANDLE> threads;
    bool begun = false, okay = false;
    try {
        THREADENTRY32 entry{}; entry.dwSize = sizeof(entry);
        if (!Thread32First(snapshot, &entry)) throw 0;
        do {
            if (entry.th32OwnerProcessID == GetCurrentProcessId() && entry.th32ThreadID != GetCurrentThreadId()) {
                HANDLE thread = OpenThread(THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT | THREAD_SET_CONTEXT
                    | THREAD_QUERY_INFORMATION, FALSE, entry.th32ThreadID);
                if (!thread) {
                    if (GetLastError() == ERROR_INVALID_PARAMETER) continue;
                    throw 0;
                }
                try { threads.push_back(thread); } catch (...) { CloseHandle(thread); throw; }
            }
            entry.dwSize = sizeof(entry);
        } while (Thread32Next(snapshot, &entry));
        if (GetLastError() != ERROR_NO_MORE_FILES || DetourTransactionBegin() != NO_ERROR) throw 0;
        begun = true;
        if (DetourUpdateThread(GetCurrentThread()) != NO_ERROR) throw 0;
        for (HANDLE thread : threads) if (DetourUpdateThread(thread) != NO_ERROR) throw 0;
        original = reinterpret_cast<Initialize>(image + 0xbc22120);
        if (DetourAttach(reinterpret_cast<PVOID*>(&original), reinterpret_cast<PVOID>(initialize)) != NO_ERROR) throw 0;
        const LONG result = DetourTransactionCommit();
        begun = false;
        okay = result == NO_ERROR;
    } catch (...) {}
    if (begun) DetourTransactionAbort();
    for (HANDLE thread : threads) CloseHandle(thread);
    CloseHandle(snapshot);
    return okay;
}
}
void consider() noexcept {
    if (installed.load(std::memory_order_acquire) || installing.exchange(true, std::memory_order_acquire)) return;
    const DWORD error = GetLastError();
    // Early calls before the game's code is ready preserve native behavior; a later factory can retry.
    if (knownCode() && attach()) installed.store(true, std::memory_order_release);
    SetLastError(error);
    installing.store(false, std::memory_order_release);
}
}
#else
namespace graphics_warm_pools { void consider() noexcept {} }
#endif
