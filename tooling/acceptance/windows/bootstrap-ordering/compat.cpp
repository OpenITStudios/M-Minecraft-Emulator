#include "receipt.h"
#include <detours.h>
#include <cstdio>
#include <cwchar>
#include <cstring>

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
probe::NativeFileCreate originalFileCreate = nullptr;
probe::NativeFileOpen originalFileOpen = nullptr;
HANDLE fileLogicalRoot = nullptr, fileOwnRoot = nullptr;
FILE_ID_INFO logicalDirectoryId{};
wchar_t logicalDevice[512]{}, logicalDos[512]{};

bool sameFileId(const FILE_ID_INFO& a, const FILE_ID_INFO& b) {
    return a.VolumeSerialNumber == b.VolumeSerialNumber
        && memcmp(a.FileId.Identifier, b.FileId.Identifier, sizeof(a.FileId.Identifier)) == 0;
}
bool directoryInfo(HANDLE handle, FILE_ID_INFO& id) {
    FILE_ATTRIBUTE_TAG_INFO tag{};
    if (GetFileType(handle) != FILE_TYPE_DISK
        || !GetFileInformationByHandleEx(handle, FileAttributeTagInfo, &tag, sizeof(tag))
        || !(tag.FileAttributes & FILE_ATTRIBUTE_DIRECTORY)
        || (tag.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT)) return false;
    return GetFileInformationByHandleEx(handle, FileIdInfo, &id, sizeof(id)) != FALSE;
}
bool asciiPath(const wchar_t* path, unsigned length) {
    for (unsigned i = 0; i < length; ++i) if (path[i] < 32 || path[i] > 126) return false;
    return true;
}
bool initializeFiles(const probe::Receipt& config, HMODULE ntdll) {
    if (config.fileRound > 1) return false;
    FILE_ID_INFO ownId{};
    if (!directoryInfo(config.fileLogicalRoot, logicalDirectoryId)
        || !directoryInfo(config.fileOwnRoot, ownId)
        || sameFileId(logicalDirectoryId, ownId)) return false;
    FILE_CASE_SENSITIVE_INFO caseInfo{};
    if (!GetFileInformationByHandleEx(config.fileLogicalRoot, FileCaseSensitiveInfo, &caseInfo, sizeof(caseInfo))
        || caseInfo.Flags != 0
        || !GetFileInformationByHandleEx(config.fileOwnRoot, FileCaseSensitiveInfo, &caseInfo, sizeof(caseInfo))
        || caseInfo.Flags != 0) return false;
    wchar_t ownPath[512]{};
    const DWORD deviceLength = GetFinalPathNameByHandleW(config.fileLogicalRoot, logicalDevice, 512, VOLUME_NAME_NT);
    const DWORD dosLength = GetFinalPathNameByHandleW(config.fileLogicalRoot, logicalDos, 512, VOLUME_NAME_DOS);
    const DWORD ownLength = GetFinalPathNameByHandleW(config.fileOwnRoot, ownPath, 512, VOLUME_NAME_NT);
    if (deviceLength < 9 || deviceLength >= 512 || dosLength < 7 || dosLength >= 512
        || ownLength == 0 || ownLength >= 512
        || !asciiPath(logicalDevice, deviceLength) || !asciiPath(logicalDos, dosLength)
        || !asciiPath(ownPath, ownLength) || wcsncmp(logicalDos, L"\\\\?\\", 4) != 0) return false;
    constexpr wchar_t suffix[] = L"\\logical";
    const unsigned parentLength = deviceLength - 8;
    if (wcscmp(logicalDevice + parentLength, suffix) != 0 || ownLength != parentLength + 2) return false;
    for (unsigned i = 0; i < parentLength; ++i) if (ownPath[i] != logicalDevice[i]) return false;
    if (ownPath[parentLength] != L'\\' || ownPath[parentLength + 1] != slotSuffix) return false;
    logicalDos[1] = L'?'; // Convert the canonical Win32 extended path to its NT DOS form.
    logicalDos[2] = L'?';
    fileLogicalRoot = config.fileLogicalRoot;
    fileOwnRoot = config.fileOwnRoot;
    originalFileCreate = reinterpret_cast<probe::NativeFileCreate>(GetProcAddress(ntdll, "NtCreateFile"));
    originalFileOpen = reinterpret_cast<probe::NativeFileOpen>(GetProcAddress(ntdll, "NtOpenFile"));
    return originalFileCreate && originalFileOpen;
}
enum class FileRoute { Other, Private, Unsupported };
bool pathPrefix(const UNICODE_STRING& name, const wchar_t* prefix, unsigned& leaf) {
    const unsigned count = name.Length / sizeof(wchar_t);
    const unsigned length = static_cast<unsigned>(wcslen(prefix));
    if (count < length) return false;
    for (unsigned i = 0; i < length; ++i)
        if (folded(name.Buffer[i], true) != folded(prefix[i], true)) return false;
    if (count == length) { leaf = count; return true; }
    if (name.Buffer[length] != L'\\') return false;
    leaf = length + 1;
    return true;
}
FileRoute fileRoute(const OBJECT_ATTRIBUTES& attributes) {
    if (attributes.Length != sizeof(OBJECT_ATTRIBUTES) || !attributes.ObjectName) return FileRoute::Other;
    const UNICODE_STRING name = *attributes.ObjectName;
    if (!name.Buffer || name.Length % sizeof(wchar_t) || name.MaximumLength < name.Length)
        return FileRoute::Other;
    const unsigned count = name.Length / sizeof(wchar_t);
    unsigned leaf = 0;
    if (count && name.Buffer[0] == L'\\') {
        if (!pathPrefix(name, logicalDevice, leaf) && !pathPrefix(name, logicalDos, leaf)) return FileRoute::Other;
        if (attributes.RootDirectory) return FileRoute::Unsupported;
    } else {
        FILE_ID_INFO rootId{};
        if (!attributes.RootDirectory || !directoryInfo(attributes.RootDirectory, rootId)
            || !sameFileId(rootId, logicalDirectoryId)) return FileRoute::Other;
    }
    constexpr wchar_t chosen[] = L"state.bin";
    if (count - leaf != 9) return FileRoute::Unsupported;
    for (unsigned i = 0; i < 9; ++i)
        if (folded(name.Buffer[leaf + i], true) != chosen[i]) return FileRoute::Unsupported;
    return FileRoute::Private;
}
bool unsupportedFile(ACCESS_MASK access, ULONG options) {
    return (access & (DELETE | WRITE_DAC | WRITE_OWNER | GENERIC_ALL | FILE_EXECUTE)) != 0
        || (options & (FILE_DIRECTORY_FILE | FILE_DELETE_ON_CLOSE | FILE_OPEN_BY_FILE_ID | FILE_OPEN_REPARSE_POINT)) != 0;
}
NTSTATUS NTAPI translatedFileCreate(PHANDLE handle, ACCESS_MASK access, POBJECT_ATTRIBUTES attributes,
    PIO_STATUS_BLOCK io, PLARGE_INTEGER allocation, ULONG fileAttributes, ULONG share,
    ULONG disposition, ULONG options, PVOID ea, ULONG eaLength) {
    const DWORD saved = GetLastError();
    NTSTATUS status;
    __try {
        const FileRoute route = attributes ? fileRoute(*attributes) : FileRoute::Other;
        if (route == FileRoute::Unsupported || (route == FileRoute::Private && unsupportedFile(access, options))) {
            status = static_cast<NTSTATUS>(0xc00000bbUL);
        } else {
            OBJECT_ATTRIBUTES copy{};
            wchar_t leaf[] = L"state.bin";
            UNICODE_STRING name{18, 18, leaf};
            if (route == FileRoute::Private) {
                copy = *attributes; copy.RootDirectory = fileOwnRoot; copy.ObjectName = &name;
            }
            SetLastError(saved);
            status = originalFileCreate(handle, access, route == FileRoute::Private ? &copy : attributes,
                io, allocation, fileAttributes, share, disposition, options, ea, eaLength);
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) { status = static_cast<NTSTATUS>(GetExceptionCode()); }
    SetLastError(saved);
    return status;
}
NTSTATUS NTAPI translatedFileOpen(PHANDLE handle, ACCESS_MASK access, POBJECT_ATTRIBUTES attributes,
    PIO_STATUS_BLOCK io, ULONG share, ULONG options) {
    const DWORD saved = GetLastError();
    NTSTATUS status;
    __try {
        const FileRoute route = attributes ? fileRoute(*attributes) : FileRoute::Other;
        if (route == FileRoute::Unsupported || (route == FileRoute::Private && unsupportedFile(access, options))) {
            status = static_cast<NTSTATUS>(0xc00000bbUL);
        } else {
            OBJECT_ATTRIBUTES copy{};
            wchar_t leaf[] = L"state.bin";
            UNICODE_STRING name{18, 18, leaf};
            if (route == FileRoute::Private) {
                copy = *attributes; copy.RootDirectory = fileOwnRoot; copy.ObjectName = &name;
            }
            SetLastError(saved);
            status = originalFileOpen(handle, access, route == FileRoute::Private ? &copy : attributes, io, share, options);
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) { status = static_cast<NTSTATUS>(GetExceptionCode()); }
    SetLastError(saved);
    return status;
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
    if (!originalCreate || !originalOpen || !openDirectory || !compareHandles || !initializeFiles(config, ntdll)) return false;
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
                        reinterpret_cast<PVOID>(translatedOpen)) != NO_ERROR
        || DetourAttach(reinterpret_cast<PVOID*>(&originalFileCreate),
                        reinterpret_cast<PVOID>(translatedFileCreate)) != NO_ERROR
        || DetourAttach(reinterpret_cast<PVOID*>(&originalFileOpen),
                        reinterpret_cast<PVOID>(translatedFileOpen)) != NO_ERROR) {
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
