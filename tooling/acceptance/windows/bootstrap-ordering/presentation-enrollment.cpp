#include "presentation-enrollment.h"
#include <dxgi1_6.h>
#include <detours.h>
#include <atomic>
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
    CreationCall call;
    const HRESULT status = originalOld(factory, device, description, result);
    const DWORD error = GetLastError();
    if (call.outer && SUCCEEDED(status) && result && *result && description)
        observe(description->OutputWindow, description->BufferDesc.Width, description->BufferDesc.Height);
    SetLastError(error);
    return status;
}
HRESULT STDMETHODCALLTYPE createdHwnd(IDXGIFactory2* factory, IUnknown* device, HWND window,
    const DXGI_SWAP_CHAIN_DESC1* description, const DXGI_SWAP_CHAIN_FULLSCREEN_DESC* fullscreen,
    IDXGIOutput* restricted, IDXGISwapChain1** result) {
    CreationCall call;
    const HRESULT status = originalHwnd(factory, device, window, description, fullscreen, restricted, result);
    const DWORD error = GetLastError();
    if (call.outer && SUCCEEDED(status) && result && *result && description) observe(window, description->Width, description->Height);
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
    if (call.outer && SUCCEEDED(status) && result && *result) factoryCount.fetch_add(1);
    SetLastError(error); return status;
}
HRESULT WINAPI createFactory1(REFIID id, void** result) {
    FactoryCall call;
    const HRESULT status = originalFactory1(id, result);
    const DWORD error = GetLastError();
    if (call.outer && SUCCEEDED(status) && result && *result) factoryCount.fetch_add(1);
    SetLastError(error); return status;
}
HRESULT WINAPI createFactory2(UINT flags, REFIID id, void** result) {
    FactoryCall call;
    const HRESULT status = originalFactory2(flags, id, result);
    const DWORD error = GetLastError();
    if (call.outer && SUCCEEDED(status) && result && *result) factoryCount.fetch_add(1);
    SetLastError(error); return status;
}
}
bool prepare(const education_bootstrap::Config& process) noexcept {
    wchar_t setting[32]{};
    SetLastError(ERROR_SUCCESS);
    const DWORD size = GetEnvironmentVariableW(Variable, setting, 32);
    if (!size && GetLastError() == ERROR_ENVVAR_NOT_FOUND) {
        SetLastError(ERROR_SUCCESS);
        return !GetEnvironmentVariableW(SectionVariable, setting, 32) && GetLastError() == ERROR_ENVVAR_NOT_FOUND;
    }
    // No pacing marker is accepted before a focus capability is implemented.
    if (size != 7 || wcscmp(setting, L"observe")) return false;
    const DWORD length = GetEnvironmentVariableW(SectionVariable, setting, 32);
    if (!length || length >= 32) return false;
    ULONG_PTR raw = 0;
    for (DWORD i = 0; i < length; ++i) {
        if (setting[i] < L'0' || setting[i] > L'9') return false;
        const ULONG_PTR digit = setting[i] - L'0';
        if (raw > (~static_cast<ULONG_PTR>(0) - digit) / 10) return false;
        raw = raw * 10 + digit;
    }
    if (!raw) return false;
    const HANDLE section = reinterpret_cast<HANDLE>(raw);
    if (!SetHandleInformation(section, HANDLE_FLAG_INHERIT, 0)) return false;
    void* writable = MapViewOfFile(section, FILE_MAP_WRITE, 0, 0, sizeof(CreationConfig));
    if (writable) { UnmapViewOfFile(writable); CloseHandle(section); return false; }
    const void* view = MapViewOfFile(section, FILE_MAP_READ, 0, 0, sizeof(CreationConfig));
    if (!view) { CloseHandle(section); return false; }
    CreationConfig config{};
    memcpy(&config, view, sizeof(config));
    UnmapViewOfFile(view); CloseHandle(section);
    CreationConfig actual{}; HMODULE module = nullptr;
    if (config.magic != CreationMagic || config.version != 1 || config.bytes != sizeof(config)
        || config.pid != process.pid || config.session != process.session
        || memcmp(&config.created, &process.created, sizeof(FILETIME))
        || !imageMetadata(actual, module) || config.imageBytes != actual.imageBytes
        || config.imageTimestamp != actual.imageTimestamp || memcmp(&config.imageId, &actual.imageId, sizeof(FILE_ID_INFO))) return false;
    const auto entry = [&](DWORD rva) -> void* {
        if (rva < 4096 || rva >= actual.imageBytes) return nullptr;
        auto* address = reinterpret_cast<BYTE*>(module) + rva;
        MEMORY_BASIC_INFORMATION memory{};
        if (!VirtualQuery(address, &memory, sizeof(memory)) || memory.AllocationBase != module
            || memory.State != MEM_COMMIT || memory.Type != MEM_IMAGE || (memory.Protect & PAGE_GUARD)
            || !(memory.Protect & (PAGE_EXECUTE | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY))) return nullptr;
        return address;
    };
    originalOld = reinterpret_cast<CreateOld>(entry(config.oldRva));
    originalHwnd = reinterpret_cast<CreateHwnd>(entry(config.hwndRva));
    if (!originalOld || !originalHwnd || config.oldRva == config.hwndRva) return false;
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
