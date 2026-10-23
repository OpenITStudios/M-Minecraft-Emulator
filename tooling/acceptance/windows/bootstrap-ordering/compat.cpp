#include "receipt.h"
#include <detours.h>

namespace {
decltype(&CreateMutexW) originalCreate = CreateMutexW;
decltype(&OpenMutexW) originalOpen = OpenMutexW;
wchar_t logical[128]{}, backing[128]{};

bool equal(const wchar_t* first, const wchar_t* second) {
    for (unsigned i = 0; i < 128; ++i) {
        if (first[i] != second[i]) return false;
        if (!first[i]) return true;
    }
    return false;
}
LPCWSTR route(LPCWSTR name) {
    return name && equal(name, logical) ? backing : name;
}
HANDLE WINAPI translatedCreate(LPSECURITY_ATTRIBUTES attributes, BOOL owner, LPCWSTR name) {
    return originalCreate(attributes, owner, route(name));
}
HANDLE WINAPI translatedOpen(DWORD access, BOOL inherit, LPCWSTR name) {
    return originalOpen(access, inherit, route(name));
}
bool install(const probe::Receipt& config) {
    constexpr wchar_t prefix[] = L"Local\\MivubiMutexProbe_";
    unsigned length = 0;
    while (length < 124 && config.logical[length]) ++length;
    if (length < sizeof(prefix) / sizeof(wchar_t) || length >= 124
        || config.slot < 1 || config.slot > 2) return false;
    for (unsigned i = 0; prefix[i]; ++i)
        if (config.logical[i] != prefix[i]) return false;
    for (unsigned i = 0; i < length; ++i) {
        if (i >= (sizeof(prefix) / sizeof(wchar_t)) - 1
            && (config.logical[i] < L'0' || config.logical[i] > L'9')
            && config.logical[i] != L'_') return false;
        logical[i] = config.logical[i];
        backing[i] = config.logical[i];
    }
    backing[length] = L'_';
    backing[length + 1] = config.slot == 1 ? L'A' : L'B';
    backing[length + 2] = L'\0';
    if (!equal(backing, config.own)) return false;
    // Current-thread-only transaction during this test child's initial DLL attach.
    if (!DetourRestoreAfterWith() || DetourTransactionBegin() != NO_ERROR) return false;
    if (DetourUpdateThread(GetCurrentThread()) != NO_ERROR
        || DetourAttach(reinterpret_cast<PVOID*>(&originalCreate),
                        reinterpret_cast<PVOID>(translatedCreate)) != NO_ERROR
        || DetourAttach(reinterpret_cast<PVOID*>(&originalOpen),
                        reinterpret_cast<PVOID>(translatedOpen)) != NO_ERROR) {
        DetourTransactionAbort();
        return false;
    }
    return DetourTransactionCommit() == NO_ERROR;
}
} // namespace

extern "C" void BootstrapOrdinalOne() {}

extern "C" BOOL WINAPI DllMain(HINSTANCE, DWORD reason, LPVOID) {
    if (reason != DLL_PROCESS_ATTACH) return TRUE; // Process-lifetime fixture; no live unload.
    probe::View view;
    if (!view.open()) return FALSE;
    probe::record(view.value, probe::Attach);
    if (view.value->mode == probe::FailAttach) {
        probe::record(view.value, probe::Rejected);
        return FALSE;
    }
    if (view.value->mode == probe::Timeout) {
        probe::record(view.value, probe::Suppressed);
        return TRUE;
    }
    if (view.value->mode == probe::MutexCase) {
        if (!install(*view.value)) {
            probe::record(view.value, probe::Rejected);
            return FALSE;
        }
    } else if (view.value->mode != probe::Normal) {
        return FALSE;
    }
    InterlockedExchange(&view.value->armed, 1);
    probe::record(view.value, probe::Armed);
    return TRUE;
}
