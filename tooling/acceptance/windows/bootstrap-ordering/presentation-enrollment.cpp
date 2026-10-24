#include "presentation-enrollment.h"
#include <dxgi1_6.h>
#include <detours.h>
#include <atomic>
#include <memory>
#include <cstring>
#include <cwchar>

namespace presentation_enrollment {
namespace {
using Factory = HRESULT (WINAPI*)(REFIID, void**);
using Factory2 = HRESULT (WINAPI*)(UINT, REFIID, void**);
using CreateOld = HRESULT (STDMETHODCALLTYPE*)(IDXGIFactory*, IUnknown*, DXGI_SWAP_CHAIN_DESC*, IDXGISwapChain**);
using CreateHwnd = HRESULT (STDMETHODCALLTYPE*)(IDXGIFactory2*, IUnknown*, HWND, const DXGI_SWAP_CHAIN_DESC1*,
    const DXGI_SWAP_CHAIN_FULLSCREEN_DESC*, IDXGIOutput*, IDXGISwapChain1**);
Factory originalFactory = nullptr, originalFactory1 = nullptr;
Factory2 originalFactory2 = nullptr;
bool armed = false;
thread_local bool insideFactory = false;
struct FactoryBinding {
    void* table[32]{}; // Complete Windows SDK IDXGIFactory7 ABI.
    IDXGIFactory7* receiver = nullptr;
    CreateOld createOld = nullptr;
    CreateHwnd createHwnd = nullptr;
    FactoryBinding* next = nullptr;
    ~FactoryBinding() { if (receiver) receiver->Release(); }
};
std::atomic<FactoryBinding*> factories{nullptr};
SRWLOCK publication = SRWLOCK_INIT;
unsigned retained = 0;
std::atomic<LONG> factoryCount{0}, candidateCount{0}, ignored{0}, failures{0};
std::atomic<DWORD> width{0}, height{0};
FactoryBinding* find(void* receiver) noexcept {
    for (auto* item = factories.load(std::memory_order_acquire); item; item = item->next)
        if (item->receiver == receiver) return item;
    return nullptr;
}
void observe(HWND window, UINT w, UINT h) noexcept {
    DWORD process = 0;
    wchar_t name[128]{};
    if (!window || !GetWindowThreadProcessId(window, &process) || process != GetCurrentProcessId()
        || GetAncestor(window, GA_ROOT) != window || !GetClassNameW(window, name, 128)
        || wcscmp(name, L"OGLES") != 0) {
        ignored.fetch_add(1, std::memory_order_relaxed);
        return;
    }
    // Observation only: retain no swap-chain reference, change no caller object,
    // and never prevent its final Release/recreation on the same HWND.
    width.store(w, std::memory_order_relaxed); height.store(h, std::memory_order_relaxed);
    candidateCount.fetch_add(1, std::memory_order_relaxed);
}
HRESULT STDMETHODCALLTYPE createdOld(IDXGIFactory* factory, IUnknown* device,
                                    DXGI_SWAP_CHAIN_DESC* description, IDXGISwapChain** result) {
    auto* selected = find(factory);
    if (!selected) { failures.fetch_add(1); return E_UNEXPECTED; }
    const HRESULT status = selected->createOld(factory, device, description, result);
    const DWORD error = GetLastError();
    if (SUCCEEDED(status) && result && *result && description)
        observe(description->OutputWindow, description->BufferDesc.Width, description->BufferDesc.Height);
    SetLastError(error);
    return status;
}
HRESULT STDMETHODCALLTYPE createdHwnd(IDXGIFactory2* factory, IUnknown* device, HWND window,
    const DXGI_SWAP_CHAIN_DESC1* description, const DXGI_SWAP_CHAIN_FULLSCREEN_DESC* fullscreen,
    IDXGIOutput* restricted, IDXGISwapChain1** result) {
    auto* selected = find(factory);
    if (!selected) { failures.fetch_add(1); return E_UNEXPECTED; }
    const HRESULT status = selected->createHwnd(factory, device, window, description, fullscreen, restricted, result);
    const DWORD error = GetLastError();
    if (SUCCEEDED(status) && result && *result && description) observe(window, description->Width, description->Height);
    SetLastError(error);
    return status;
}
void enroll(IUnknown* factory) noexcept {
    try {
        if (find(factory)) return;
        auto selected = std::make_unique<FactoryBinding>();
        if (FAILED(factory->QueryInterface(IID_PPV_ARGS(&selected->receiver)))
            || static_cast<IUnknown*>(selected->receiver) != factory) { failures.fetch_add(1); return; }
        MEMORY_BASIC_INFORMATION memory{};
        if (!VirtualQuery(factory, &memory, sizeof(memory)) || memory.State != MEM_COMMIT
            || (memory.Protect & PAGE_GUARD)
            || !(memory.Protect & (PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY))
            || reinterpret_cast<ULONG_PTR>(factory) % alignof(void*)) { failures.fetch_add(1); return; }
        void** table = *reinterpret_cast<void***>(factory);
        memcpy(selected->table, table, sizeof(selected->table));
        selected->createOld = reinterpret_cast<CreateOld>(table[10]);
        selected->createHwnd = reinterpret_cast<CreateHwnd>(table[15]);
        if (!selected->createOld || !selected->createHwnd) { failures.fetch_add(1); return; }
        selected->table[10] = reinterpret_cast<void*>(createdOld);
        selected->table[15] = reinterpret_cast<void*>(createdHwnd);
        HMODULE owner = nullptr;
        if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
            reinterpret_cast<LPCWSTR>(createdHwnd), &owner)) { failures.fetch_add(1); return; }
        // No COM, loader or allocation work while publishing the immutable list.
        AcquireSRWLockExclusive(&publication);
        if (find(factory) || retained >= 32) {
            const bool full = retained >= 32;
            ReleaseSRWLockExclusive(&publication);
            if (full) failures.fetch_add(1);
            return;
        }
        selected->next = factories.load(std::memory_order_relaxed);
        factories.store(selected.get(), std::memory_order_release);
        const bool installed = InterlockedCompareExchangePointer(reinterpret_cast<PVOID volatile*>(factory),
            selected->table, table) == table;
        ++retained;
        selected.release(); // Bounded, process-lifetime factory identity/table.
        ReleaseSRWLockExclusive(&publication);
        (installed ? factoryCount : failures).fetch_add(1);
    } catch (...) { failures.fetch_add(1); }
}
struct FactoryCall {
    bool outer = !insideFactory;
    FactoryCall() { insideFactory = true; }
    ~FactoryCall() { if (outer) insideFactory = false; }
};
HRESULT WINAPI createFactory(REFIID id, void** result) {
    FactoryCall call;
    const HRESULT status = originalFactory(id, result);
    const DWORD error = GetLastError();
    if (call.outer && SUCCEEDED(status) && result && *result) enroll(static_cast<IUnknown*>(*result));
    SetLastError(error); return status;
}
HRESULT WINAPI createFactory1(REFIID id, void** result) {
    FactoryCall call;
    const HRESULT status = originalFactory1(id, result);
    const DWORD error = GetLastError();
    if (call.outer && SUCCEEDED(status) && result && *result) enroll(static_cast<IUnknown*>(*result));
    SetLastError(error); return status;
}
HRESULT WINAPI createFactory2(UINT flags, REFIID id, void** result) {
    FactoryCall call;
    const HRESULT status = originalFactory2(flags, id, result);
    const DWORD error = GetLastError();
    if (call.outer && SUCCEEDED(status) && result && *result) enroll(static_cast<IUnknown*>(*result));
    SetLastError(error); return status;
}
}
bool prepare() noexcept {
    wchar_t setting[16]{};
    SetLastError(ERROR_SUCCESS);
    const DWORD size = GetEnvironmentVariableW(Variable, setting, 16);
    if (!size && GetLastError() == ERROR_ENVVAR_NOT_FOUND) return true;
    // Until the manager focus capability exists, only observation can be armed.
    if (size != 7 || wcscmp(setting, L"observe") != 0) return false;
    armed = true;
    return true;
}
bool attach() noexcept {
    if (!armed) return true;
    originalFactory = &CreateDXGIFactory;
    originalFactory1 = &CreateDXGIFactory1;
    originalFactory2 = &CreateDXGIFactory2;
    return DetourAttach(reinterpret_cast<PVOID*>(&originalFactory), reinterpret_cast<PVOID>(createFactory)) == NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalFactory1), reinterpret_cast<PVOID>(createFactory1)) == NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalFactory2), reinterpret_cast<PVOID>(createFactory2)) == NO_ERROR;
}
Snapshot snapshot() noexcept {
    return {sizeof(Snapshot), armed ? 1UL : 0UL, factoryCount.load(), candidateCount.load(), ignored.load(),
        failures.load(), width.load(), height.load()};
}
}
