#include "dxgi-pacing.h"
#include "frame-pacing.h"
#include <dxgi1_5.h>
#include <cstring>
#include <memory>

namespace dxgi_pacing {
namespace {
using Present = HRESULT (STDMETHODCALLTYPE*)(IDXGISwapChain*, UINT, UINT);
using Present1 = HRESULT (STDMETHODCALLTYPE*)(IDXGISwapChain1*, UINT, UINT, const DXGI_PRESENT_PARAMETERS*);
Present originalPresent = nullptr;
Present1 originalPresent1 = nullptr;
thread_local bool insidePresent = false;
struct Binding {
    presentation_pacing::Pacer pacer;
    IUnknown* identity = nullptr;
    IDXGISwapChain* receiver = nullptr;
    void* table[41]{};
    HWND window = nullptr;
    DWORD windowThread = 0;
    std::atomic<bool> enabled{true};
    std::atomic_flag consuming = ATOMIC_FLAG_INIT;
    std::atomic<unsigned long long> present{0}, present1{0}, bypassed{0}, faults{0};
    std::atomic<LONGLONG> lastRelease{0};
    std::atomic<unsigned> lastCeiling{0};
    ~Binding() {
        if (receiver) receiver->Release();
        if (identity) identity->Release();
    }
};
// Pinned after successful installation, matching the compatibility DLL's process lifetime.
// A failed publication releases its temporary binding. No live unhook/reload route exists.
std::atomic<Binding*> binding{nullptr};

bool owns(Binding& selected, IDXGISwapChain* chain) noexcept {
    if (chain != selected.receiver) {
        IUnknown* identity = nullptr;
        if (FAILED(chain->QueryInterface(IID_PPV_ARGS(&identity)))) return false;
        const bool same = identity == selected.identity;
        identity->Release();
        if (!same) return false;
    }
    DWORD process = 0;
    if (GetWindowThreadProcessId(selected.window, &process) != selected.windowThread
        || process != GetCurrentProcessId()) return false;
    DXGI_SWAP_CHAIN_DESC description{};
    return SUCCEEDED(chain->GetDesc(&description)) && description.OutputWindow == selected.window;
}
struct Call {
    Binding* selected = nullptr;
    bool outer = !insidePresent;
    bool acquired = false;
    Call(IDXGISwapChain* chain, UINT sync, UINT flags, bool newer) noexcept {
        if (!outer) return;
        insidePresent = true;
        selected = binding.load(std::memory_order_acquire);
        if (!selected) return;
        // TEST is not a frame, and DO_NOT_WAIT must keep its nonblocking contract.
        if (!selected->enabled.load(std::memory_order_acquire)
            || sync > 4 || (flags & (DXGI_PRESENT_TEST | DXGI_PRESENT_DO_NOT_WAIT))
            || !owns(*selected, chain)) {
            selected->bypassed.fetch_add(1, std::memory_order_relaxed);
            return;
        }
        acquired = !selected->consuming.test_and_set(std::memory_order_acquire);
        if (!acquired) {
            // The timing core has one rendering consumer. Never race its clock state
            // or serialize a second presentation thread behind an unknown game lock.
            selected->faults.fetch_add(1, std::memory_order_relaxed);
            selected->enabled.store(false, std::memory_order_release);
            selected->pacer.stop();
            return;
        }
        const auto permit = selected->pacer.wait_for_frame();
        if (permit.outcome != presentation_pacing::Outcome::Frame) {
            if (permit.outcome == presentation_pacing::Outcome::Error)
                selected->faults.fetch_add(1, std::memory_order_relaxed);
            selected->enabled.store(false, std::memory_order_release);
            return;
        }
        selected->lastRelease.store(permit.released, std::memory_order_relaxed);
        selected->lastCeiling.store(permit.ceiling, std::memory_order_relaxed);
        (newer ? selected->present1 : selected->present).fetch_add(1, std::memory_order_relaxed);
    }
    ~Call() {
        if (acquired) selected->consuming.clear(std::memory_order_release);
        if (outer) insidePresent = false;
    }
};
HRESULT STDMETHODCALLTYPE pacedPresent(IDXGISwapChain* chain, UINT sync, UINT flags) {
    const DWORD before = GetLastError();
    HRESULT result;
    DWORD after;
    {
        Call call(chain, sync, flags, false);
        SetLastError(before);
        result = originalPresent(chain, sync, flags);
        after = GetLastError();
    }
    SetLastError(after);
    return result;
}
HRESULT STDMETHODCALLTYPE pacedPresent1(IDXGISwapChain1* chain, UINT sync, UINT flags,
                                       const DXGI_PRESENT_PARAMETERS* parameters) {
    const DWORD before = GetLastError();
    HRESULT result;
    DWORD after;
    {
        Call call(chain, sync, flags, true);
        SetLastError(before);
        result = originalPresent1(chain, sync, flags, parameters);
        after = GetLastError();
    }
    SetLastError(after);
    return result;
}
}
bool install(IDXGISwapChain1* chain, HWND window) noexcept {
    // Installation is an explicitly quiescent operation; never perform it from DllMain.
    if (binding.load(std::memory_order_acquire) || !chain || !window) return false;
    try {
        auto selected = std::make_unique<Binding>();
        DWORD process = 0;
        selected->windowThread = GetWindowThreadProcessId(window, &process);
        if (!selected->windowThread || process != GetCurrentProcessId()) return false;
        DXGI_SWAP_CHAIN_DESC description{};
        if (FAILED(chain->GetDesc(&description)) || description.OutputWindow != window
            || FAILED(chain->QueryInterface(IID_PPV_ARGS(&selected->identity)))) return false;
        selected->window = window;
        selected->receiver = static_cast<IDXGISwapChain*>(chain);
        selected->receiver->AddRef();
        // Pin the complete SDK SwapChain4 interface before copying its 41-method ABI.
        IDXGISwapChain4* newest = nullptr;
        if (FAILED(chain->QueryInterface(IID_PPV_ARGS(&newest)))) return false;
        const bool sameInterface = static_cast<IDXGISwapChain1*>(newest) == chain;
        newest->Release();
        if (!sameInterface) return false;
        MEMORY_BASIC_INFORMATION memory{};
        if (!VirtualQuery(chain, &memory, sizeof(memory)) || memory.State != MEM_COMMIT
            || (memory.Protect & PAGE_GUARD)
            || !(memory.Protect & (PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY))
            || reinterpret_cast<ULONG_PTR>(chain) % alignof(void*)) return false;
        void** table = *reinterpret_cast<void***>(chain);
        originalPresent = reinterpret_cast<Present>(table[8]);
        originalPresent1 = reinterpret_cast<Present1>(table[22]);
        if (!originalPresent || !originalPresent1 || table[8] == table[22]) return false;
        memcpy(selected->table, table, sizeof(selected->table));
        selected->table[8] = reinterpret_cast<void*>(pacedPresent);
        selected->table[22] = reinterpret_cast<void*>(pacedPresent1);
        HMODULE owner = nullptr;
        if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
            reinterpret_cast<LPCWSTR>(pacedPresent), &owner)) return false;
        binding.store(selected.get(), std::memory_order_release);
        if (InterlockedCompareExchangePointer(reinterpret_cast<PVOID volatile*>(chain), selected->table, table) != table) {
            binding.store(nullptr, std::memory_order_release);
            return false;
        }
        selected.release();
        return true;
    } catch (...) { return false; }
}
bool foreground(bool active) noexcept {
    Binding* selected = binding.load(std::memory_order_acquire);
    return selected && selected->enabled.load(std::memory_order_acquire) && selected->pacer.foreground(active);
}
bool stop() noexcept {
    Binding* selected = binding.load(std::memory_order_acquire);
    if (!selected) return false;
    selected->enabled.store(false, std::memory_order_release);
    return selected->pacer.stop();
}
Snapshot snapshot() noexcept {
    Binding* selected = binding.load(std::memory_order_acquire);
    if (!selected) return {};
    return {selected->present.load(), selected->present1.load(), selected->bypassed.load(),
        selected->faults.load(), selected->lastRelease.load(), selected->lastCeiling.load()};
}
} // namespace dxgi_pacing
