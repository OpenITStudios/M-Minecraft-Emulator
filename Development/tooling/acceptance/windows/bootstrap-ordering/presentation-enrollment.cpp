#include "presentation-enrollment.h"
#include <dxgi1_6.h>
#include <detours.h>
#include <atomic>
#include <wrl/client.h>
#include "dxgi-pacing.h"
#include "graphics-warm-pools.h"

namespace presentation_enrollment {
namespace {
using Factory = HRESULT (WINAPI*)(REFIID, void**);
using Factory2 = HRESULT (WINAPI*)(UINT, REFIID, void**);
using CreateOld = HRESULT (STDMETHODCALLTYPE*)(IDXGIFactory*, IUnknown*, DXGI_SWAP_CHAIN_DESC*, IDXGISwapChain**);
using CreateHwnd = HRESULT (STDMETHODCALLTYPE*)(IDXGIFactory2*, IUnknown*, HWND, const DXGI_SWAP_CHAIN_DESC1*,
    const DXGI_SWAP_CHAIN_FULLSCREEN_DESC*, IDXGIOutput*, IDXGISwapChain1**);
Factory originalFactory = nullptr, originalFactory1 = nullptr;
Factory2 originalFactory2 = nullptr;
bool armed = false, pacing = false;
presentation_pacing::Input input{};
INIT_ONCE initialized = INIT_ONCE_STATIC_INIT;
bool policyReady = false;
BOOL CALLBACK initializePolicy(PINIT_ONCE, PVOID, PVOID*) {
    policyReady = dxgi_pacing::initialize(input);
    for (HANDLE handle : {input.level, input.changed, input.stop, input.controller, input.hudSelected}) if (handle) CloseHandle(handle);
    input = {};
    return TRUE; // A failed initialization is sticky; preserve native presentation.
}
thread_local bool insideFactory = false;
CreateOld originalOld = nullptr;
CreateHwnd originalHwnd = nullptr;
thread_local bool insideCreation = false;
std::atomic<LONG> factoryCount{0}, candidateCount{0}, ignored{0}, failures{0};
std::atomic<DWORD> width{0}, height{0};
struct CreationCall {
    bool outer = !insideCreation;
    CreationCall() { insideCreation = true; }
    ~CreationCall() { if (outer) insideCreation = false; }
};
bool observe(HWND window, UINT w, UINT h) noexcept {
    if (!lazytester_presentation_candidate(window)) {
        ignored.fetch_add(1, std::memory_order_relaxed);
        return false;
    }
    // Observation only: retain no swap-chain reference, change no caller object,
    // and never prevent its final Release/recreation on the same HWND.
    width.store(w, std::memory_order_relaxed); height.store(h, std::memory_order_relaxed);
    candidateCount.fetch_add(1, std::memory_order_relaxed);
    return true;
}
bool initializePacing() noexcept {
    return InitOnceExecuteOnce(&initialized, initializePolicy, nullptr, nullptr) && policyReady;
}
void wrap(IDXGISwapChain1** result, HWND window) noexcept {
    if (!initializePacing() || !dxgi_pacing::wrap(result, window)) failures.fetch_add(1);
}
HRESULT STDMETHODCALLTYPE createdOld(IDXGIFactory* factory, IUnknown* device,
                                    DXGI_SWAP_CHAIN_DESC* description, IDXGISwapChain** result) {
    CreationCall call;
    const HRESULT status = originalOld(factory, device, description, result);
    const DWORD error = GetLastError();
    if (call.outer && SUCCEEDED(status) && result && *result && description
        && observe(description->OutputWindow, description->BufferDesc.Width, description->BufferDesc.Height) && pacing) {
        Microsoft::WRL::ComPtr<IDXGISwapChain1> modern;
        if (SUCCEEDED((*result)->QueryInterface(IID_PPV_ARGS(&modern))) && initializePacing()
            && dxgi_pacing::wrap(modern.GetAddressOf(), description->OutputWindow)) {
            (*result)->Release(); *result = modern.Detach();
        } else failures.fetch_add(1);
    }
    SetLastError(error);
    return status;
}
HRESULT STDMETHODCALLTYPE createdHwnd(IDXGIFactory2* factory, IUnknown* device, HWND window,
    const DXGI_SWAP_CHAIN_DESC1* description, const DXGI_SWAP_CHAIN_FULLSCREEN_DESC* fullscreen,
    IDXGIOutput* restricted, IDXGISwapChain1** result) {
    CreationCall call;
    const HRESULT status = originalHwnd(factory, device, window, description, fullscreen, restricted, result);
    const DWORD error = GetLastError();
    if (call.outer && SUCCEEDED(status) && result && *result && description
        && observe(window, description->Width, description->Height) && pacing) wrap(result, window);
    SetLastError(error);
    return status;
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
    if (call.outer && SUCCEEDED(status) && result && *result) {
        factoryCount.fetch_add(1);
        if (pacing) graphics_warm_pools::consider();
    }
    SetLastError(error); return status;
}
HRESULT WINAPI createFactory1(REFIID id, void** result) {
    FactoryCall call;
    const HRESULT status = originalFactory1(id, result);
    const DWORD error = GetLastError();
    if (call.outer && SUCCEEDED(status) && result && *result) {
        factoryCount.fetch_add(1);
        if (pacing) graphics_warm_pools::consider();
    }
    SetLastError(error); return status;
}
HRESULT WINAPI createFactory2(UINT flags, REFIID id, void** result) {
    FactoryCall call;
    const HRESULT status = originalFactory2(flags, id, result);
    const DWORD error = GetLastError();
    if (call.outer && SUCCEEDED(status) && result && *result) {
        factoryCount.fetch_add(1);
        if (pacing) graphics_warm_pools::consider();
    }
    SetLastError(error); return status;
}
}
bool prepare(const education_bootstrap::Config& process) noexcept {
    LazyTesterPresentationAdmission admitted{};
    if (!lazytester_presentation_admit(process.pid,process.session,
        process.created.dwLowDateTime,process.created.dwHighDateTime,&admitted)) return false;
    if (!admitted.mode) return true;
    pacing = admitted.mode == 2;
    if (pacing) input = {admitted.level,admitted.changed,admitted.stop,admitted.controller,
        admitted.active,admitted.background,admitted.selected};
    originalOld = reinterpret_cast<CreateOld>(admitted.oldEntry);
    originalHwnd = reinterpret_cast<CreateHwnd>(admitted.hwndEntry);
    armed = true;
    return true;
}
bool attach() noexcept {
    if (!armed) return true;
    originalFactory = &CreateDXGIFactory;
    originalFactory1 = &CreateDXGIFactory1;
    originalFactory2 = &CreateDXGIFactory2;
    return DetourAttach(reinterpret_cast<PVOID*>(&originalOld), reinterpret_cast<PVOID>(createdOld)) == NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalHwnd), reinterpret_cast<PVOID>(createdHwnd)) == NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalFactory), reinterpret_cast<PVOID>(createFactory)) == NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalFactory1), reinterpret_cast<PVOID>(createFactory1)) == NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalFactory2), reinterpret_cast<PVOID>(createFactory2)) == NO_ERROR;
}
Snapshot snapshot() noexcept {
    return {sizeof(Snapshot), armed ? 1UL : 0UL, factoryCount.load(), candidateCount.load(), ignored.load(),
        failures.load(), width.load(), height.load()};
}
}
