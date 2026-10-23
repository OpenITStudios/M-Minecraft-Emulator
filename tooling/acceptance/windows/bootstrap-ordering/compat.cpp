#include "receipt.h"
#include <detours.h>
#include <cstdio>
#include <cwchar>

namespace {
probe::NativeCreate originalCreate = nullptr;
probe::NativeOpen originalOpen = nullptr;
probe::NativeDirectory openDirectory = nullptr;
HANDLE baseDirectory = nullptr;
probe::CompareHandles compareHandles = nullptr;
wchar_t logical[128]{};
unsigned logicalLength = 0;
wchar_t slotSuffix = 0;

wchar_t folded(wchar_t c, bool insensitive) {
    return insensitive && c >= L'A' && c <= L'Z' ? static_cast<wchar_t>(c + L'a' - L'A') : c;
}
bool selected(const UNICODE_STRING& name, unsigned leaf, bool insensitive) {
    const unsigned count = name.Length / sizeof(wchar_t);
    if (count - leaf != logicalLength) return false;
    for (unsigned i = 0; i < logicalLength; ++i)
        if (folded(name.Buffer[leaf + i], insensitive) != folded(logical[i], insensitive)) return false;
    return true;
}
bool translate(const OBJECT_ATTRIBUTES& attributes, UNICODE_STRING& name, wchar_t (&buffer)[256]) {
    if (attributes.Length != sizeof(OBJECT_ATTRIBUTES) || !attributes.ObjectName) return false;
    const UNICODE_STRING source = *attributes.ObjectName;
    if (!source.Buffer || source.Length % sizeof(wchar_t)
        || source.MaximumLength < source.Length) return false;
    const unsigned count = source.Length / sizeof(wchar_t);
    if (count == 0 || count > 253) return false;
    unsigned leaf = 0;
    for (unsigned i = 0; i < count; ++i) if (source.Buffer[i] == L'\\') leaf = i + 1;
    if (!selected(source, leaf, (attributes.Attributes & OBJ_CASE_INSENSITIVE) != 0)) return false;

    HANDLE parent = attributes.RootDirectory;
    HANDLE opened = nullptr;
    if (leaf != 0) {
        // Resolve the parent, including Local aliases, without querying arbitrary object names.
        UNICODE_STRING parentName{static_cast<USHORT>((leaf - 1) * sizeof(wchar_t)),
                                 static_cast<USHORT>((leaf - 1) * sizeof(wchar_t)), source.Buffer};
        OBJECT_ATTRIBUTES parentAttributes{sizeof(OBJECT_ATTRIBUTES), attributes.RootDirectory,
            &parentName, attributes.Attributes & OBJ_CASE_INSENSITIVE, nullptr, nullptr};
        if (openDirectory(&opened, 0x0002 /* DIRECTORY_TRAVERSE */, &parentAttributes) < 0) return false;
        parent = opened;
    }
    const bool same = parent && compareHandles(parent, baseDirectory) != FALSE;
    if (opened) CloseHandle(opened);
    if (!same) return false;

    for (unsigned i = 0; i < count; ++i) buffer[i] = source.Buffer[i];
    buffer[count] = L'_';
    buffer[count + 1] = slotSuffix;
    // Native strings are counted; never require or copy a caller's terminator.
    name = {static_cast<USHORT>((count + 2) * sizeof(wchar_t)),
            static_cast<USHORT>(sizeof(buffer)), buffer};
    return true;
}
NTSTATUS NTAPI translatedCreate(PHANDLE handle, ACCESS_MASK access,
                                POBJECT_ATTRIBUTES attributes, BOOLEAN owner) {
    const DWORD saved = GetLastError();
    NTSTATUS status;
    __try {
        OBJECT_ATTRIBUTES copy{};
        UNICODE_STRING name{};
        wchar_t buffer[256]{};
        const bool mapped = attributes && translate(*attributes, name, buffer);
        if (mapped) { copy = *attributes; copy.ObjectName = &name; }
        SetLastError(saved);
        status = originalCreate(handle, access, mapped ? &copy : attributes, owner);
    } __except (EXCEPTION_EXECUTE_HANDLER) {
        status = static_cast<NTSTATUS>(GetExceptionCode());
    }
    SetLastError(saved);
    return status;
}
NTSTATUS NTAPI translatedOpen(PHANDLE handle, ACCESS_MASK access, POBJECT_ATTRIBUTES attributes) {
    const DWORD saved = GetLastError();
    NTSTATUS status;
    __try {
        OBJECT_ATTRIBUTES copy{};
        UNICODE_STRING name{};
        wchar_t buffer[256]{};
        const bool mapped = attributes && translate(*attributes, name, buffer);
        if (mapped) { copy = *attributes; copy.ObjectName = &name; }
        SetLastError(saved);
        status = originalOpen(handle, access, mapped ? &copy : attributes);
    } __except (EXCEPTION_EXECUTE_HANDLER) {
        status = static_cast<NTSTATUS>(GetExceptionCode());
    }
    SetLastError(saved);
    return status;
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
    }
    slotSuffix = config.slot == 1 ? L'A' : L'B';
    for (unsigned i = 0; i < length; ++i) if (config.own[i] != config.logical[i]) return false;
    if (config.own[length] != L'_' || config.own[length + 1] != slotSuffix
        || config.own[length + 2] != L'\0') return false;
    constexpr unsigned localPrefix = 6;
    logicalLength = length - localPrefix;
    for (unsigned i = 0; i < logicalLength; ++i) logical[i] = config.logical[localPrefix + i];

    const HMODULE ntdll = GetModuleHandleW(L"ntdll.dll");
    if (!ntdll) return false;
    originalCreate = reinterpret_cast<probe::NativeCreate>(GetProcAddress(ntdll, "NtCreateMutant"));
    originalOpen = reinterpret_cast<probe::NativeOpen>(GetProcAddress(ntdll, "NtOpenMutant"));
    openDirectory = reinterpret_cast<probe::NativeDirectory>(GetProcAddress(ntdll, "NtOpenDirectoryObject"));
    const HMODULE kernelbase = GetModuleHandleW(L"kernelbase.dll");
    compareHandles = kernelbase ? reinterpret_cast<probe::CompareHandles>(
        GetProcAddress(kernelbase, "CompareObjectHandles")) : nullptr;
    if (!originalCreate || !originalOpen || !openDirectory || !compareHandles) return false;
    DWORD session = 0;
    wchar_t path[80]{};
    if (!ProcessIdToSessionId(GetCurrentProcessId(), &session)
        || swprintf_s(path, L"\\Sessions\\%lu\\BaseNamedObjects", session) < 0) return false;
    const auto bytes = static_cast<USHORT>(wcslen(path) * sizeof(wchar_t));
    UNICODE_STRING rootName{bytes, bytes, path};
    OBJECT_ATTRIBUTES root{sizeof(OBJECT_ATTRIBUTES), nullptr, &rootName, OBJ_CASE_INSENSITIVE, nullptr, nullptr};
    if (openDirectory(&baseDirectory, 0x0002 /* DIRECTORY_TRAVERSE */, &root) < 0) return false;

    // One native layer serves the direct calls and the tested Win32 wrappers.
    if (!DetourRestoreAfterWith() || DetourTransactionBegin() != NO_ERROR) {
        CloseHandle(baseDirectory); baseDirectory = nullptr; return false;
    }
    if (DetourUpdateThread(GetCurrentThread()) != NO_ERROR
        || DetourAttach(reinterpret_cast<PVOID*>(&originalCreate),
                        reinterpret_cast<PVOID>(translatedCreate)) != NO_ERROR
        || DetourAttach(reinterpret_cast<PVOID*>(&originalOpen),
                        reinterpret_cast<PVOID>(translatedOpen)) != NO_ERROR) {
        DetourTransactionAbort();
        CloseHandle(baseDirectory); baseDirectory = nullptr; return false;
    }
    if (DetourTransactionCommit() != NO_ERROR) {
        CloseHandle(baseDirectory); baseDirectory = nullptr; return false;
    }
    return true; // Directory handle and hooks live until this owned child exits.
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
