#include "dxgi-pacing.h"
#include "frame-pacing.h"
#include <dxgi1_5.h>
#include <wrl/client.h>
#include <memory>

namespace dxgi_pacing {
namespace {
using Microsoft::WRL::ComPtr;
struct Policy {
    presentation_pacing::Pacer pacer;
    explicit Policy(const presentation_pacing::Input& input = {}) : pacer(input) {}
    SRWLOCK rendering = SRWLOCK_INIT;
    std::atomic<unsigned long long> present{0}, present1{0}, bypassed{0}, faults{0};
    std::atomic<unsigned long long> created{0}, destroyed{0}, unsupported{0};
    std::atomic<unsigned> live{0};
    std::atomic<LONGLONG> lastRelease{0};
    std::atomic<unsigned> lastCeiling{0};
};
// Process policy owns clocks/counters only. No COM object is retained here.
std::atomic<Policy*> policy{nullptr};
SRWLOCK installation = SRWLOCK_INIT;
thread_local bool insidePresent = false;
void codeAnchor() {}
class Chain final : public IDXGISwapChain4, public IDXGISwapChainMedia {
    std::atomic<ULONG> references_{1};
    Policy& policy_;
    ComPtr<IDXGISwapChain4> inner_;
    ComPtr<IDXGISwapChainMedia> media_;
    HWND window_;
    DWORD windowThread_;
    ~Chain() {
        // Drop every native interface before allowing a replacement binding.
        media_.Reset(); inner_.Reset();
        policy_.destroyed.fetch_add(1);
        policy_.live.store(0, std::memory_order_release);
    }
    template<class Native> HRESULT present(UINT sync, UINT flags, bool newer, Native native) {
        const DWORD before = GetLastError();
        const bool outer = !insidePresent;
        insidePresent = true;
        if (outer) {
            DWORD process = 0;
            if (sync > 4 || (flags & (DXGI_PRESENT_TEST | DXGI_PRESENT_DO_NOT_WAIT))
                || GetWindowThreadProcessId(window_, &process) != windowThread_
                || process != GetCurrentProcessId()) policy_.bypassed.fetch_add(1);
            else {
                // Clock serialization ends BEFORE entering the native renderer.
                AcquireSRWLockExclusive(&policy_.rendering);
                const auto permit = policy_.pacer.wait_for_frame();
                if (permit.outcome == presentation_pacing::Outcome::Frame) {
                    policy_.lastRelease.store(permit.released);
                    policy_.lastCeiling.store(permit.ceiling);
                    (newer ? policy_.present1 : policy_.present).fetch_add(1);
                } else {
                    policy_.bypassed.fetch_add(1);
                    if (permit.outcome == presentation_pacing::Outcome::Error) {
                        policy_.faults.fetch_add(1); policy_.pacer.stop();
                    }
                }
                ReleaseSRWLockExclusive(&policy_.rendering);
            }
        }
        SetLastError(before);
        const HRESULT result = native();
        const DWORD after = GetLastError();
        if (outer) insidePresent = false;
        SetLastError(after); return result;
    }
public:
    Chain(Policy& p, ComPtr<IDXGISwapChain4> inner, ComPtr<IDXGISwapChainMedia> media, HWND window, DWORD thread)
        : policy_(p), inner_(std::move(inner)), media_(std::move(media)), window_(window), windowThread_(thread) {
        policy_.created.fetch_add(1);
    }
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID id, void** result) override {
        if (!result) return E_POINTER;
        *result = nullptr;
        if (id == __uuidof(IUnknown) || id == __uuidof(IDXGIObject) || id == __uuidof(IDXGIDeviceSubObject)
            || id == __uuidof(IDXGISwapChain) || id == __uuidof(IDXGISwapChain1)
            || id == __uuidof(IDXGISwapChain2) || id == __uuidof(IDXGISwapChain3)
            || id == __uuidof(IDXGISwapChain4)) *result = static_cast<IDXGISwapChain4*>(this);
        else if (id == __uuidof(IDXGISwapChainMedia) && media_) *result = static_cast<IDXGISwapChainMedia*>(this);
        else {
            ComPtr<IUnknown> native;
            const HRESULT supported = inner_->QueryInterface(id, reinterpret_cast<void**>(native.GetAddressOf()));
            if (SUCCEEDED(supported)) {
                // Do not leak an inner IUnknown with a different COM identity.
                // This unsupported extension is an explicit compatibility gap.
                policy_.unsupported.fetch_add(1);
                policy_.faults.fetch_add(1);
                return E_NOINTERFACE;
            }
            return supported;
        }
        AddRef(); return S_OK;
    }
    ULONG STDMETHODCALLTYPE AddRef() override { return references_.fetch_add(1) + 1; }
    ULONG STDMETHODCALLTYPE Release() override {
        const ULONG left = references_.fetch_sub(1) - 1;
        if (!left) delete this;
        return left; // Our own count, not a native object's diagnostic count.
    }
    HRESULT STDMETHODCALLTYPE Present(UINT sync, UINT flags) override {
        return present(sync, flags, false, [&] { return inner_->Present(sync, flags); });
    }
    HRESULT STDMETHODCALLTYPE Present1(UINT sync, UINT flags, const DXGI_PRESENT_PARAMETERS* parameters) override {
        return present(sync, flags, true, [&] { return inner_->Present1(sync, flags, parameters); });
    }
#define FORWARD(name, signature, args) HRESULT STDMETHODCALLTYPE name signature override { return inner_->name args; }
    FORWARD(SetPrivateData, (REFGUID key, UINT bytes, const void* data), (key, bytes, data))
    FORWARD(SetPrivateDataInterface, (REFGUID key, const IUnknown* data), (key, data))
    FORWARD(GetPrivateData, (REFGUID key, UINT* bytes, void* data), (key, bytes, data))
    FORWARD(GetParent, (REFIID id, void** value), (id, value))
    FORWARD(GetDevice, (REFIID id, void** value), (id, value))
    FORWARD(GetBuffer, (UINT index, REFIID id, void** value), (index, id, value))
    FORWARD(SetFullscreenState, (BOOL fullscreen, IDXGIOutput* target), (fullscreen, target))
    FORWARD(GetFullscreenState, (BOOL* fullscreen, IDXGIOutput** target), (fullscreen, target))
    FORWARD(GetDesc, (DXGI_SWAP_CHAIN_DESC* value), (value))
    FORWARD(ResizeBuffers, (UINT count, UINT w, UINT h, DXGI_FORMAT format, UINT flags), (count, w, h, format, flags))
    FORWARD(ResizeTarget, (const DXGI_MODE_DESC* value), (value))
    FORWARD(GetContainingOutput, (IDXGIOutput** value), (value))
    FORWARD(GetFrameStatistics, (DXGI_FRAME_STATISTICS* value), (value))
    FORWARD(GetLastPresentCount, (UINT* value), (value))
    FORWARD(GetDesc1, (DXGI_SWAP_CHAIN_DESC1* value), (value))
    FORWARD(GetFullscreenDesc, (DXGI_SWAP_CHAIN_FULLSCREEN_DESC* value), (value))
    FORWARD(GetHwnd, (HWND* value), (value))
    FORWARD(GetCoreWindow, (REFIID id, void** value), (id, value))
    FORWARD(GetRestrictToOutput, (IDXGIOutput** value), (value))
    FORWARD(SetBackgroundColor, (const DXGI_RGBA* value), (value))
    FORWARD(GetBackgroundColor, (DXGI_RGBA* value), (value))
    FORWARD(SetRotation, (DXGI_MODE_ROTATION value), (value))
    FORWARD(GetRotation, (DXGI_MODE_ROTATION* value), (value))
    FORWARD(SetSourceSize, (UINT w, UINT h), (w, h))
    FORWARD(GetSourceSize, (UINT* w, UINT* h), (w, h))
    FORWARD(SetMaximumFrameLatency, (UINT value), (value))
    FORWARD(GetMaximumFrameLatency, (UINT* value), (value))
    FORWARD(SetMatrixTransform, (const DXGI_MATRIX_3X2_F* value), (value))
    FORWARD(GetMatrixTransform, (DXGI_MATRIX_3X2_F* value), (value))
    FORWARD(CheckColorSpaceSupport, (DXGI_COLOR_SPACE_TYPE value, UINT* support), (value, support))
    FORWARD(SetColorSpace1, (DXGI_COLOR_SPACE_TYPE value), (value))
    FORWARD(ResizeBuffers1, (UINT count, UINT w, UINT h, DXGI_FORMAT format, UINT flags,
        const UINT* masks, IUnknown* const* queues), (count, w, h, format, flags, masks, queues))
    FORWARD(SetHDRMetaData, (DXGI_HDR_METADATA_TYPE type, UINT bytes, void* data), (type, bytes, data))
#undef FORWARD
    BOOL STDMETHODCALLTYPE IsTemporaryMonoSupported() override { return inner_->IsTemporaryMonoSupported(); }
    HANDLE STDMETHODCALLTYPE GetFrameLatencyWaitableObject() override { return inner_->GetFrameLatencyWaitableObject(); }
    UINT STDMETHODCALLTYPE GetCurrentBackBufferIndex() override { return inner_->GetCurrentBackBufferIndex(); }
    HRESULT STDMETHODCALLTYPE GetFrameStatisticsMedia(DXGI_FRAME_STATISTICS_MEDIA* value) override {
        return media_->GetFrameStatisticsMedia(value);
    }
    HRESULT STDMETHODCALLTYPE SetPresentDuration(UINT value) override { return media_->SetPresentDuration(value); }
    HRESULT STDMETHODCALLTYPE CheckPresentDurationSupport(UINT desired, UINT* smaller, UINT* larger) override {
        return media_->CheckPresentDurationSupport(desired, smaller, larger);
    }
};
}
bool initialize(const presentation_pacing::Input& input) noexcept {
    AcquireSRWLockExclusive(&installation);
    bool initialized = false;
    try {
        if (!policy.load(std::memory_order_acquire)) {
            auto* configured = new Policy(input);
            policy.store(configured, std::memory_order_release);
            initialized = true;
        }
    } catch (...) {}
    ReleaseSRWLockExclusive(&installation);
    return initialized;
}
bool wrap(IDXGISwapChain1** owned, HWND window) noexcept {
    if (!owned || !*owned || !window) return false;
    AcquireSRWLockExclusive(&installation);
    bool wrapped = false, reserved = false;
    Policy* p = nullptr;
    try {
        p = policy.load(std::memory_order_acquire);
        if (p && p->live.load(std::memory_order_acquire)) throw std::runtime_error("Active chain remains");
        DWORD process = 0;
        const DWORD thread = GetWindowThreadProcessId(window, &process);
        DXGI_SWAP_CHAIN_DESC description{};
        if (!thread || process != GetCurrentProcessId() || FAILED((*owned)->GetDesc(&description))
            || description.OutputWindow != window) throw std::runtime_error("Window mismatch");
        ComPtr<IDXGISwapChain4> inner;
        if (FAILED((*owned)->QueryInterface(IID_PPV_ARGS(&inner)))) throw std::runtime_error("Interface unavailable");
        ComPtr<IDXGISwapChainMedia> media;
        const HRESULT optional = inner.As(&media);
        if (FAILED(optional) && optional != E_NOINTERFACE) throw std::runtime_error("Media interface query failed");
        HMODULE module = nullptr;
        if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
            reinterpret_cast<LPCWSTR>(codeAnchor), &module)) throw std::runtime_error("Code pin failed");
        if (!p) { p = new Policy(); policy.store(p, std::memory_order_release); }
        p->live.store(1); reserved = true;
        auto* result = new Chain(*p, std::move(inner), std::move(media), window, thread);
        (*owned)->Release(); // Transfer the caller's one reference into the wrapper.
        *owned = static_cast<IDXGISwapChain1*>(result);
        wrapped = true;
    } catch (...) { if (reserved) p->live.store(0); }
    ReleaseSRWLockExclusive(&installation);
    return wrapped;
}
bool foreground(bool active) noexcept {
    auto* p = policy.load(std::memory_order_acquire);
    return p && p->pacer.foreground(active);
}
bool stop() noexcept {
    auto* p = policy.load(std::memory_order_acquire);
    return p && p->pacer.stop();
}
Snapshot snapshot() noexcept {
    auto* p = policy.load(std::memory_order_acquire);
    if (!p) return {};
    return {p->present.load(), p->present1.load(), p->bypassed.load(), p->faults.load(),
        p->lastRelease.load(), p->lastCeiling.load(), p->live.load(), p->created.load(), p->destroyed.load(),
        p->unsupported.load()};
}
} // namespace dxgi_pacing
