#include "receipt.h"
#include <detours.h>
#include <cstdio>
#include <cwchar>
#include <cstring>
#include <shlobj.h>
#include <knownfolders.h>
#include <atomic>

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
FILE_ID_INFO logicalDirectoryId{}, ownDirectoryId{};
wchar_t logicalDevice[512]{}, logicalDos[512]{}, ownDevice[512]{}, ownDos[512]{};

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
    if (!directoryInfo(config.fileLogicalRoot, logicalDirectoryId)
        || !directoryInfo(config.fileOwnRoot, ownDirectoryId)
        || sameFileId(logicalDirectoryId, ownDirectoryId)) return false;
    FILE_CASE_SENSITIVE_INFO caseInfo{};
    if (!GetFileInformationByHandleEx(config.fileLogicalRoot, FileCaseSensitiveInfo, &caseInfo, sizeof(caseInfo))
        || caseInfo.Flags != 0
        || !GetFileInformationByHandleEx(config.fileOwnRoot, FileCaseSensitiveInfo, &caseInfo, sizeof(caseInfo))
        || caseInfo.Flags != 0) return false;
    const DWORD deviceLength = GetFinalPathNameByHandleW(config.fileLogicalRoot, logicalDevice, 512, VOLUME_NAME_NT);
    const DWORD dosLength = GetFinalPathNameByHandleW(config.fileLogicalRoot, logicalDos, 512, VOLUME_NAME_DOS);
    const DWORD ownLength = GetFinalPathNameByHandleW(config.fileOwnRoot, ownDevice, 512, VOLUME_NAME_NT);
    const DWORD ownDosLength = GetFinalPathNameByHandleW(config.fileOwnRoot, ownDos, 512, VOLUME_NAME_DOS);
    if (deviceLength < 9 || deviceLength >= 512 || dosLength < 7 || dosLength >= 512
        || ownLength == 0 || ownLength >= 512 || ownDosLength < 7 || ownDosLength >= 480
        || !asciiPath(ownDos, ownDosLength) || wcsncmp(ownDos, L"\\\\?\\", 4) != 0
        || !asciiPath(logicalDevice, deviceLength) || !asciiPath(logicalDos, dosLength)
        || !asciiPath(ownDevice, ownLength) || wcsncmp(logicalDos, L"\\\\?\\", 4) != 0) return false;
    constexpr wchar_t suffix[] = L"\\logical";
    const unsigned parentLength = deviceLength - 8;
    if (wcscmp(logicalDevice + parentLength, suffix) != 0 || ownLength != parentLength + 2) return false;
    for (unsigned i = 0; i < parentLength; ++i) if (ownDevice[i] != logicalDevice[i]) return false;
    if (ownDevice[parentLength] != L'\\' || ownDevice[parentLength + 1] != slotSuffix) return false;
    logicalDos[1] = L'?'; // Convert the canonical Win32 extended path to its NT DOS form.
    logicalDos[2] = L'?';
    ownDos[1] = ownDos[2] = L'?';
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
bool validRelative(const wchar_t* path, unsigned count) {
    if (count >= 512) return false;
    if (!count) return true;
    unsigned component = 0, depth = 1;
    for (unsigned i = 0; i < count; ++i) {
        const wchar_t ch = path[i];
        if (ch == L'\\') {
            if (!component || path[i - 1] == L'.' || ++depth > 8) return false;
            component = 0;
            continue;
        }
        const bool alphanumeric = (ch >= L'a' && ch <= L'z') || (ch >= L'A' && ch <= L'Z')
            || (ch >= L'0' && ch <= L'9');
        if (!alphanumeric && (!component || (ch != L'.' && ch != L'_' && ch != L'-')))
            return false;
        if (++component > 64) return false;
    }
    return component != 0 && path[count - 1] != L'.';
}
FileRoute fileRoute(const OBJECT_ATTRIBUTES& attributes, wchar_t (&path)[512], unsigned& length) {
    if (attributes.Length != sizeof(OBJECT_ATTRIBUTES) || !attributes.ObjectName) return FileRoute::Other;
    const UNICODE_STRING name = *attributes.ObjectName;
    if (!name.Buffer || name.Length % sizeof(wchar_t) || name.MaximumLength < name.Length)
        return FileRoute::Other;
    const unsigned count = name.Length / sizeof(wchar_t);
    unsigned leaf = 0;
    length = 0;
    if (count && name.Buffer[0] == L'\\') {
        if (!pathPrefix(name, logicalDevice, leaf) && !pathPrefix(name, logicalDos, leaf)
            && !pathPrefix(name, ownDevice, leaf) && !pathPrefix(name, ownDos, leaf)) return FileRoute::Other;
        if (attributes.RootDirectory) return FileRoute::Unsupported;
    } else {
        FILE_ID_INFO rootId{};
        if (!attributes.RootDirectory || !directoryInfo(attributes.RootDirectory, rootId))
            return FileRoute::Other;
        if (!sameFileId(rootId, logicalDirectoryId) && !sameFileId(rootId, ownDirectoryId)) {
            wchar_t parentPath[512]{};
            const DWORD size = GetFinalPathNameByHandleW(attributes.RootDirectory,
                parentPath, 512, VOLUME_NAME_NT);
            if (!size || size >= 512) return FileRoute::Other;
            const auto bytes = static_cast<USHORT>(size * sizeof(wchar_t));
            UNICODE_STRING parent{bytes, bytes, parentPath};
            unsigned offset = 0;
            if (!pathPrefix(parent, logicalDevice, offset) && !pathPrefix(parent, ownDevice, offset))
                return FileRoute::Other;
            length = size - offset;
            if (!validRelative(parentPath + offset, length)) return FileRoute::Unsupported;
            memcpy(path, parentPath + offset, length * sizeof(wchar_t));
            if (count) path[length++] = L'\\';
        }
    }
    if (count - leaf >= 512 - length) return FileRoute::Unsupported;
    memcpy(path + length, name.Buffer + leaf, (count - leaf) * sizeof(wchar_t));
    length += count - leaf;
    path[length] = L'\0';
    return validRelative(path, length) ? FileRoute::Private : FileRoute::Unsupported;
}
constexpr ULONG DontReparse = 0x1000; // OBJ_DONT_REPARSE from OBJECT_ATTRIBUTES.
constexpr NTSTATUS Unsupported = static_cast<NTSTATUS>(0xc00000bbUL);
NTSTATUS privateParent(wchar_t* path, unsigned length, HANDLE& parent,
                       UNICODE_STRING& leaf, HANDLE (&held)[8], unsigned& count) {
    parent = fileOwnRoot;
    unsigned start = 0;
    for (unsigned i = 0; i < length; ++i) {
        if (path[i] != L'\\') continue;
        const auto bytes = static_cast<USHORT>((i - start) * sizeof(wchar_t));
        UNICODE_STRING name{bytes, bytes, path + start};
        OBJECT_ATTRIBUTES attributes{sizeof(OBJECT_ATTRIBUTES), parent, &name,
            OBJ_CASE_INSENSITIVE | DontReparse, nullptr, nullptr};
        IO_STATUS_BLOCK io{};
        HANDLE next = nullptr;
        const NTSTATUS status = originalFileOpen(&next,
            FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES | SYNCHRONIZE, &attributes,
            &io, FILE_SHARE_READ | FILE_SHARE_WRITE,
            FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_REPARSE_POINT);
        if (status < 0) return status;
        held[count++] = next; // Keep every ancestor pinned through the final open.
        FILE_ID_INFO id{};
        FILE_CASE_SENSITIVE_INFO caseInfo{};
        if (!directoryInfo(next, id)
            || !GetFileInformationByHandleEx(next, FileCaseSensitiveInfo, &caseInfo, sizeof(caseInfo))
            || caseInfo.Flags != 0) return Unsupported;
        parent = next;
        start = i + 1;
    }
    const auto bytes = static_cast<USHORT>((length - start) * sizeof(wchar_t));
    leaf = {bytes, bytes, path + start};
    return 0;
}
bool unsupportedFile(ACCESS_MASK access, ULONG options, bool root, bool newDirectory = false) {
    return (access & (DELETE | WRITE_DAC | WRITE_OWNER | GENERIC_ALL)) != 0
        || ((access & FILE_EXECUTE) && !(root || (options & FILE_DIRECTORY_FILE)))
        || (options & (FILE_DELETE_ON_CLOSE | FILE_OPEN_BY_FILE_ID)) != 0
        || (!newDirectory && (options & FILE_OPEN_REPARSE_POINT))
        || (root && (options & FILE_NON_DIRECTORY_FILE));
}
NTSTATUS NTAPI translatedFileCreate(PHANDLE handle, ACCESS_MASK access, POBJECT_ATTRIBUTES attributes,
    PIO_STATUS_BLOCK io, PLARGE_INTEGER allocation, ULONG fileAttributes, ULONG share,
    ULONG disposition, ULONG options, PVOID ea, ULONG eaLength) {
    const DWORD saved = GetLastError();
    NTSTATUS status = Unsupported;
    HANDLE held[8]{};
    unsigned heldCount = 0;
    __try {
        wchar_t path[512]{};
        unsigned length = 0;
        const FileRoute route = attributes ? fileRoute(*attributes, path, length) : FileRoute::Other;
        if (route == FileRoute::Other) {
            SetLastError(saved);
            status = originalFileCreate(handle, access, attributes, io, allocation,
                fileAttributes, share, disposition, options, ea, eaLength);
        } else if (route == FileRoute::Private
            && !unsupportedFile(access, options, length == 0,
                disposition == 2 && (options & FILE_DIRECTORY_FILE) != 0)
            && (length != 0 || disposition == 1)) {
            OBJECT_ATTRIBUTES copy = *attributes;
            UNICODE_STRING leaf{};
            status = privateParent(path, length, copy.RootDirectory, leaf, held, heldCount);
            if (status >= 0) {
                copy.ObjectName = &leaf;
                copy.Attributes |= DontReparse;
                SetLastError(saved);
                status = originalFileCreate(handle, access, &copy, io, allocation,
                    fileAttributes, share, disposition, options, ea, eaLength);
            }
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) { status = static_cast<NTSTATUS>(GetExceptionCode()); }
    while (heldCount) CloseHandle(held[--heldCount]);
    SetLastError(saved);
    return status;
}
NTSTATUS NTAPI translatedFileOpen(PHANDLE handle, ACCESS_MASK access, POBJECT_ATTRIBUTES attributes,
    PIO_STATUS_BLOCK io, ULONG share, ULONG options) {
    const DWORD saved = GetLastError();
    NTSTATUS status = Unsupported;
    HANDLE held[8]{};
    unsigned heldCount = 0;
    __try {
        wchar_t path[512]{};
        unsigned length = 0;
        const FileRoute route = attributes ? fileRoute(*attributes, path, length) : FileRoute::Other;
        if (route == FileRoute::Other) {
            SetLastError(saved);
            status = originalFileOpen(handle, access, attributes, io, share, options);
        } else if (route == FileRoute::Private && !unsupportedFile(access, options, length == 0)) {
            OBJECT_ATTRIBUTES copy = *attributes;
            UNICODE_STRING leaf{};
            status = privateParent(path, length, copy.RootDirectory, leaf, held, heldCount);
            if (status >= 0) {
                copy.ObjectName = &leaf;
                copy.Attributes |= DontReparse;
                SetLastError(saved);
                status = originalFileOpen(handle, access, &copy, io, share, options);
            }
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) { status = static_cast<NTSTATUS>(GetExceptionCode()); }
    while (heldCount) CloseHandle(held[--heldCount]);
    SetLastError(saved);
    return status;
}

using KnownPath = HRESULT (WINAPI*)(REFKNOWNFOLDERID, DWORD, HANDLE, PWSTR*);
KnownPath originalKnownPath = nullptr;
wchar_t roamingPath[512]{}, localPath[512]{};
bool initializeAppData() {
    if (swprintf_s(roamingPath, L"%s\\Roaming", ownDos + 4) < 0
        || swprintf_s(localPath, L"%s\\Local", ownDos + 4) < 0) return false;
    wchar_t environment[512]{};
    const DWORD roaming = GetEnvironmentVariableW(L"APPDATA", environment, 512);
    if (!roaming || roaming >= 512 || wcscmp(environment, roamingPath) != 0) return false;
    const DWORD local = GetEnvironmentVariableW(L"LOCALAPPDATA", environment, 512);
    if (!local || local >= 512 || wcscmp(environment, localPath) != 0) return false;
    // Resolve an already imported module; no shell/COM lookup or LoadLibrary in DllMain.
    const HMODULE shell = GetModuleHandleW(L"shell32.dll");
    originalKnownPath = shell ? reinterpret_cast<KnownPath>(
        GetProcAddress(shell, "SHGetKnownFolderPath")) : nullptr;
    return originalKnownPath != nullptr;
}
HRESULT WINAPI translatedKnownPath(REFKNOWNFOLDERID id, DWORD flags, HANDLE token, PWSTR* output) {
    const wchar_t* selected = IsEqualGUID(id, FOLDERID_RoamingAppData) ? roamingPath
        : IsEqualGUID(id, FOLDERID_LocalAppData) ? localPath : nullptr;
    if (!selected) return originalKnownPath(id, flags, token, output);
    const DWORD saved = GetLastError();
    if (!output) return E_POINTER;
    *output = nullptr;
    HRESULT result = S_OK;
    if (token) result = E_ACCESSDENIED;
    // shortcut: default/DONT_VERIFY only; review other flag semantics before expanding coverage.
    else if (flags & ~static_cast<DWORD>(KF_FLAG_DONT_VERIFY)) result = E_NOTIMPL;
    else {
        if (!(flags & KF_FLAG_DONT_VERIFY)) {
            const DWORD attributes = GetFileAttributesW(selected);
            if (attributes == INVALID_FILE_ATTRIBUTES) result = HRESULT_FROM_WIN32(GetLastError());
            else if (!(attributes & FILE_ATTRIBUTE_DIRECTORY)
                || (attributes & FILE_ATTRIBUTE_REPARSE_POINT)) result = E_ACCESSDENIED;
        }
        if (SUCCEEDED(result)) {
            const SIZE_T bytes = (wcslen(selected) + 1) * sizeof(wchar_t);
            auto memory = static_cast<PWSTR>(CoTaskMemAlloc(bytes));
            if (!memory) result = E_OUTOFMEMORY;
            else { memcpy(memory, selected, bytes); *output = memory; }
        }
    }
    SetLastError(saved);
    return result;
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
bool initializeRegistry(const probe::Receipt& config);
bool attachRegistry();
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
    if (!originalCreate || !originalOpen || !openDirectory || !compareHandles || !initializeFiles(config, ntdll)
        || !initializeAppData() || !initializeRegistry(config)) return false;
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
                        reinterpret_cast<PVOID>(translatedFileOpen)) != NO_ERROR
        || DetourAttach(reinterpret_cast<PVOID*>(&originalKnownPath),
                        reinterpret_cast<PVOID>(translatedKnownPath)) != NO_ERROR || !attachRegistry()) {
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

namespace {
std::atomic<HKEY> privateHive{nullptr};
static_assert(std::atomic<HKEY>::is_always_lock_free);
constexpr NTSTATUS RegistryDenied = static_cast<NTSTATUS>(0xc0000022UL);
using OpenUser = LSTATUS (WINAPI*)(REGSAM, PHKEY);
using OpenKey = NTSTATUS (NTAPI*)(PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES);
using OpenKeyEx = NTSTATUS (NTAPI*)(PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES, ULONG);
using CreateKey = NTSTATUS (NTAPI*)(PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES, ULONG,
    PUNICODE_STRING, ULONG, PULONG);
using QueryKey = NTSTATUS (NTAPI*)(HANDLE, ULONG, PVOID, ULONG, PULONG);
OpenUser originalOpenUser = nullptr;
OpenKey originalOpenKey = nullptr;
OpenKeyEx originalOpenKeyEx = nullptr;
CreateKey originalCreateKey = nullptr;
QueryKey queryKey = nullptr;
using QueryValue = NTSTATUS (NTAPI*)(HANDLE, PUNICODE_STRING, ULONG, PVOID, ULONG, PULONG);
using SetValue = NTSTATUS (NTAPI*)(HANDLE, PUNICODE_STRING, ULONG, ULONG, PVOID, ULONG);
using QueryObject = NTSTATUS (NTAPI*)(HANDLE, ULONG, PVOID, ULONG, PULONG);
QueryValue originalQueryValue = nullptr;
SetValue originalSetValue = nullptr;
QueryObject queryObject = nullptr;

wchar_t userKeyPath[512]{};
bool keyName(HANDLE handle, wchar_t (&path)[512]) {
    alignas(8) BYTE buffer[sizeof(ULONG) + sizeof(path)]{};
    ULONG returned = 0;
    if (queryKey(handle, 3 /* KeyNameInformation */, buffer, sizeof(buffer), &returned) < 0
        || returned < sizeof(ULONG) || returned > sizeof(buffer)) return false;
    ULONG bytes = 0;
    memcpy(&bytes, buffer, sizeof(bytes));
    if (bytes % sizeof(wchar_t) || bytes >= sizeof(path) || bytes > returned - sizeof(ULONG)) return false;
    memcpy(path, buffer + sizeof(ULONG), bytes);
    path[bytes / sizeof(wchar_t)] = L'\0';
    return true;
}
LSTATUS noImpersonation() {
    HANDLE token = nullptr;
    if (OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, TRUE, &token)) {
        CloseHandle(token);
        return ERROR_NOT_SUPPORTED;
    }
    const DWORD error = GetLastError();
    return error == ERROR_NO_TOKEN ? ERROR_SUCCESS : static_cast<LSTATUS>(error);
}
LSTATUS WINAPI translatedOpenUser(REGSAM access, PHKEY result) {
    const DWORD saved = GetLastError();
    if (!result) return ERROR_INVALID_PARAMETER;
    *result = nullptr;
    const HKEY hive = privateHive.load();
    LSTATUS status = hive ? noImpersonation() : ERROR_ACCESS_DENIED;
    if (status == ERROR_SUCCESS) status = RegOpenKeyExW(hive, L"", 0, access, result);
    SetLastError(saved);
    return status;
}
enum class KeyRoute { Other, Private, Unsupported };
KeyRoute registryRoute(const OBJECT_ATTRIBUTES& attributes, wchar_t (&path)[1024],
                       UNICODE_STRING& name) {
    if (attributes.Length != sizeof(OBJECT_ATTRIBUTES) || !attributes.ObjectName) return KeyRoute::Other;
    const UNICODE_STRING source = *attributes.ObjectName;
    if (!source.Buffer || source.Length % sizeof(wchar_t) || source.MaximumLength < source.Length)
        return KeyRoute::Other;
    const unsigned count = source.Length / sizeof(wchar_t);
    unsigned prefix = 0;
    bool knownUser = false;
    if (count && source.Buffer[0] == L'\\') {
        unsigned offset = 0;
        if (!pathPrefix(source, userKeyPath, offset)) return KeyRoute::Other;
        if (attributes.RootDirectory) return KeyRoute::Unsupported;
        knownUser = true;
    } else {
        wchar_t parent[512]{};
        if (!attributes.RootDirectory || !keyName(attributes.RootDirectory, parent)) return KeyRoute::Other;
        prefix = static_cast<unsigned>(wcslen(parent));
        const auto bytes = static_cast<USHORT>(prefix * sizeof(wchar_t));
        UNICODE_STRING rootName{bytes, bytes, parent};
        unsigned offset = 0;
        knownUser = pathPrefix(rootName, userKeyPath, offset);
        memcpy(path, parent, prefix * sizeof(wchar_t));
        if (count) path[prefix++] = L'\\';
    }
    if (count >= 1024 - prefix) return knownUser ? KeyRoute::Unsupported : KeyRoute::Other;
    memcpy(path + prefix, source.Buffer, source.Length);
    const unsigned total = prefix + count;
    const auto bytes = static_cast<USHORT>(total * sizeof(wchar_t));
    UNICODE_STRING absolute{bytes, bytes, path};
    unsigned leaf = 0;
    if (!pathPrefix(absolute, userKeyPath, leaf)) return KeyRoute::Other;
    if (!validRelative(path + leaf, total - leaf)) return KeyRoute::Unsupported;
    const auto suffix = static_cast<USHORT>((total - leaf) * sizeof(wchar_t));
    name = {suffix, suffix, path + leaf};
    return KeyRoute::Private;
}
NTSTATUS NTAPI translatedOpenKey(PHANDLE result, ACCESS_MASK access, POBJECT_ATTRIBUTES attributes) {
    const DWORD saved = GetLastError();
    NTSTATUS status = Unsupported;
    __try {
        wchar_t path[1024]{};
        UNICODE_STRING name{};
        const auto route = attributes ? registryRoute(*attributes, path, name) : KeyRoute::Other;
        const HKEY hive = privateHive.load();
        OBJECT_ATTRIBUTES copy{};
        if (route == KeyRoute::Private && hive && noImpersonation() == ERROR_SUCCESS) {
            copy = *attributes; copy.RootDirectory = hive; copy.ObjectName = &name;
            SetLastError(saved);
            status = originalOpenKey(result, access, &copy);
        } else if (route == KeyRoute::Private && !hive) {
            status = RegistryDenied;
        } else if (route == KeyRoute::Other) {
            SetLastError(saved);
            status = originalOpenKey(result, access, attributes);
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) { status = static_cast<NTSTATUS>(GetExceptionCode()); }
    SetLastError(saved);
    return status;
}
NTSTATUS NTAPI translatedOpenKeyEx(PHANDLE result, ACCESS_MASK access,
                                  POBJECT_ATTRIBUTES attributes, ULONG options) {
    const DWORD saved = GetLastError();
    NTSTATUS status = Unsupported;
    __try {
        wchar_t path[1024]{};
        UNICODE_STRING name{};
        const auto route = attributes ? registryRoute(*attributes, path, name) : KeyRoute::Other;
        const HKEY hive = privateHive.load();
        OBJECT_ATTRIBUTES copy{};
        if (route == KeyRoute::Private && hive && options == 0 && noImpersonation() == ERROR_SUCCESS) {
            copy = *attributes; copy.RootDirectory = hive; copy.ObjectName = &name;
            SetLastError(saved);
            status = originalOpenKeyEx(result, access, &copy, options);
        } else if (route == KeyRoute::Private && !hive) {
            status = RegistryDenied;
        } else if (route == KeyRoute::Other) {
            SetLastError(saved);
            status = originalOpenKeyEx(result, access, attributes, options);
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) { status = static_cast<NTSTATUS>(GetExceptionCode()); }
    SetLastError(saved);
    return status;
}
NTSTATUS NTAPI translatedCreateKey(PHANDLE result, ACCESS_MASK access,
    POBJECT_ATTRIBUTES attributes, ULONG title, PUNICODE_STRING className, ULONG options, PULONG disposition) {
    const DWORD saved = GetLastError();
    NTSTATUS status = Unsupported;
    __try {
        wchar_t path[1024]{};
        UNICODE_STRING name{};
        const auto route = attributes ? registryRoute(*attributes, path, name) : KeyRoute::Other;
        const HKEY hive = privateHive.load();
        OBJECT_ATTRIBUTES copy{};
        if (route == KeyRoute::Private && hive && options == 0 && noImpersonation() == ERROR_SUCCESS) {
            copy = *attributes; copy.RootDirectory = hive; copy.ObjectName = &name;
            SetLastError(saved);
            status = originalCreateKey(result, access, &copy, title, className, options, disposition);
        } else if (route == KeyRoute::Private && !hive) {
            status = RegistryDenied;
        } else if (route == KeyRoute::Other) {
            SetLastError(saved);
            status = originalCreateKey(result, access, attributes, title, className, options, disposition);
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) { status = static_cast<NTSTATUS>(GetExceptionCode()); }
    SetLastError(saved);
    return status;
}

NTSTATUS openPrivateValue(HANDLE source, ACCESS_MASK right, HANDLE& mapped, KeyRoute& route) {
    wchar_t sourceName[512]{};
    if (!keyName(source, sourceName)) { route = KeyRoute::Other; return 0; }
    const auto bytes = static_cast<USHORT>(wcslen(sourceName) * sizeof(wchar_t));
    UNICODE_STRING name{bytes, bytes, sourceName};
    OBJECT_ATTRIBUTES attributes{sizeof(OBJECT_ATTRIBUTES), nullptr, &name,
        OBJ_CASE_INSENSITIVE, nullptr, nullptr};
    wchar_t path[1024]{};
    UNICODE_STRING selected{};
    route = registryRoute(attributes, path, selected);
    if (route == KeyRoute::Other) return 0;
    if (route != KeyRoute::Private) return Unsupported;
    const HKEY hive = privateHive.load();
    if (!hive) return RegistryDenied;
    if (noImpersonation() != ERROR_SUCCESS) return Unsupported;
    PUBLIC_OBJECT_BASIC_INFORMATION info{};
    ULONG returned = 0;
    if (queryObject(source, 0 /* ObjectBasicInformation */, &info, sizeof(info), &returned) < 0
        || (info.GrantedAccess & right) != right) return RegistryDenied;
    attributes.RootDirectory = hive;
    attributes.ObjectName = &selected;
    return originalOpenKey(&mapped, right, &attributes);
}
NTSTATUS NTAPI translatedQueryValue(HANDLE key, PUNICODE_STRING name, ULONG information,
    PVOID output, ULONG length, PULONG returned) {
    const DWORD saved = GetLastError();
    NTSTATUS status = Unsupported;
    HANDLE mapped = nullptr;
    __try {
        KeyRoute route = KeyRoute::Other;
        status = openPrivateValue(key, KEY_QUERY_VALUE, mapped, route);
        if (status >= 0) {
            SetLastError(saved);
            status = originalQueryValue(route == KeyRoute::Private ? mapped : key,
                name, information, output, length, returned);
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) { status = static_cast<NTSTATUS>(GetExceptionCode()); }
    if (mapped) CloseHandle(mapped);
    SetLastError(saved);
    return status;
}
NTSTATUS NTAPI translatedSetValue(HANDLE key, PUNICODE_STRING name, ULONG title,
    ULONG type, PVOID data, ULONG bytes) {
    const DWORD saved = GetLastError();
    NTSTATUS status = Unsupported;
    HANDLE mapped = nullptr;
    __try {
        KeyRoute route = KeyRoute::Other;
        status = openPrivateValue(key, KEY_SET_VALUE, mapped, route);
        if (status >= 0) {
            SetLastError(saved);
            status = originalSetValue(route == KeyRoute::Private ? mapped : key, name, title, type, data, bytes);
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) { status = static_cast<NTSTATUS>(GetExceptionCode()); }
    if (mapped) CloseHandle(mapped);
    SetLastError(saved);
    return status;
}
bool attachRegistryValues() {
    return DetourAttach(reinterpret_cast<PVOID*>(&originalQueryValue), reinterpret_cast<PVOID>(translatedQueryValue)) == NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalSetValue), reinterpret_cast<PVOID>(translatedSetValue)) == NO_ERROR;
}

bool initializeRegistry(const probe::Receipt& config) {
    const HMODULE ntdll = GetModuleHandleW(L"ntdll.dll");
    const HMODULE advapi = GetModuleHandleW(L"advapi32.dll");
    if (!ntdll || !advapi) return false;
    originalOpenUser = reinterpret_cast<OpenUser>(GetProcAddress(advapi, "RegOpenCurrentUser"));
    originalOpenKey = reinterpret_cast<OpenKey>(GetProcAddress(ntdll, "NtOpenKey"));
    originalOpenKeyEx = reinterpret_cast<OpenKeyEx>(GetProcAddress(ntdll, "NtOpenKeyEx"));
    originalCreateKey = reinterpret_cast<CreateKey>(GetProcAddress(ntdll, "NtCreateKey"));
    queryKey = reinterpret_cast<QueryKey>(GetProcAddress(ntdll, "NtQueryKey"));
    originalQueryValue = reinterpret_cast<QueryValue>(GetProcAddress(ntdll, "NtQueryValueKey"));
    originalSetValue = reinterpret_cast<SetValue>(GetProcAddress(ntdll, "NtSetValueKey"));
    queryObject = reinterpret_cast<QueryObject>(GetProcAddress(ntdll, "NtQueryObject"));
    if (!originalOpenUser || !originalOpenKey || !originalOpenKeyEx || !originalCreateKey
        || !queryKey || !originalQueryValue || !originalSetValue || !queryObject) return false;
    unsigned length = 0;
    while (length < 512 && config.registryUserPath[length]) ++length;
    constexpr wchar_t prefix[] = L"\\REGISTRY\\USER\\";
    if (length <= 15 || length >= 512) return false;
    for (unsigned i = 0; prefix[i]; ++i)
        if (folded(config.registryUserPath[i], true) != folded(prefix[i], true)) return false;
    memcpy(userKeyPath, config.registryUserPath, (length + 1) * sizeof(wchar_t));
    return true; // Receipt metadata only; no registry/profile setup while under loader lock.
}
bool attachRegistry() {
    return DetourAttach(reinterpret_cast<PVOID*>(&originalOpenUser), reinterpret_cast<PVOID>(translatedOpenUser)) == NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalOpenKey), reinterpret_cast<PVOID>(translatedOpenKey)) == NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalOpenKeyEx), reinterpret_cast<PVOID>(translatedOpenKeyEx)) == NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalCreateKey), reinterpret_cast<PVOID>(translatedCreateKey)) == NO_ERROR
        && attachRegistryValues();
}
}
extern "C" BOOL BootstrapRegistry() {
    probe::View view;
    if (!view.open() || view.value->mode != probe::MutexCase || view.value->armed != 1
        || view.value->fileOwnRoot != fileOwnRoot) return FALSE;
    if (privateHive.load()) return TRUE; // Owned main thread only; not a concurrent enrollment API.
    FILE_ID_INFO id{};
    if (!directoryInfo(fileOwnRoot, id) || !sameFileId(id, ownDirectoryId)) return FALSE;
    wchar_t path[512]{};
    if (swprintf_s(path, L"%s\\Registry\\HKCU.dat", ownDos + 4) < 0) return FALSE;
    const DWORD attributes = GetFileAttributesW(path);
    if (attributes != INVALID_FILE_ATTRIBUTES
        && (attributes & (FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DIRECTORY))) return FALSE;
    HKEY hive = nullptr, root = nullptr;
    if (RegLoadAppKeyW(path, &hive, KEY_ALL_ACCESS, REG_PROCESS_APPKEY, 0) != ERROR_SUCCESS) return FALSE;
    LSTATUS status = RegOpenKeyExW(hive, L"", 0, KEY_ALL_ACCESS, &root);
    if (status == ERROR_SUCCESS) status = RegOverridePredefKey(HKEY_CURRENT_USER, root);
    if (root) RegCloseKey(root);
    if (status != ERROR_SUCCESS) { RegCloseKey(hive); return FALSE; }
    privateHive.store(hive); // Publish only after the predefined mapping is complete.
    return TRUE;
}
