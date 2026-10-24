#include "receipt.h"
#include "education-config.h"
#include "startup-observation.h"
#include "presentation-enrollment.h"
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
using SetFileInformation = NTSTATUS (NTAPI*)(HANDLE, PIO_STATUS_BLOCK, PVOID, ULONG, FILE_INFORMATION_CLASS);
SetFileInformation originalSetFileInformation = nullptr;
using QueryAttributes = NTSTATUS (NTAPI*)(POBJECT_ATTRIBUTES, PVOID);
QueryAttributes originalQueryAttributes = nullptr, originalQueryFullAttributes = nullptr;
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
struct InstallConfig {
    DWORD slot;
    HANDLE fileLogicalRoot, fileOwnRoot;
    HKEY inheritedRegistry;
    DWORD registryBacking;
    const wchar_t* registryUserPath;
    const education_bootstrap::Config* education = nullptr;
};
bool initializeFiles(const InstallConfig& config, HMODULE ntdll) {
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
    if (config.education) {
        if (!sameFileId(logicalDirectoryId, config.education->logicalId)
            || !sameFileId(ownDirectoryId, config.education->privateId)) return false;
        const auto contains = [](const wchar_t* root, unsigned rootLength,
                                 const wchar_t* child, unsigned childLength) {
            if (childLength < rootLength) return false;
            for (unsigned i = 0; i < rootLength; ++i)
                if (folded(root[i], true) != folded(child[i], true)) return false;
            return childLength == rootLength || child[rootLength] == L'\\';
        };
        // Nested views would route backing paths ambiguously. Refuse, never fall back.
        if (contains(logicalDevice, deviceLength, ownDevice, ownLength)
            || contains(ownDevice, ownLength, logicalDevice, deviceLength)) return false;
    } else {
        constexpr wchar_t suffix[] = L"\\logical";
        const unsigned parentLength = deviceLength - 8;
        if (wcscmp(logicalDevice + parentLength, suffix) != 0 || ownLength != parentLength + 2) return false;
        for (unsigned i = 0; i < parentLength; ++i) if (ownDevice[i] != logicalDevice[i]) return false;
        if (ownDevice[parentLength] != L'\\' || ownDevice[parentLength + 1] != slotSuffix) return false;
    }
    logicalDos[1] = L'?'; // Convert the canonical Win32 extended path to its NT DOS form.
    logicalDos[2] = L'?';
    ownDos[1] = ownDos[2] = L'?';
    fileLogicalRoot = config.fileLogicalRoot;
    fileOwnRoot = config.fileOwnRoot;
    originalFileCreate = reinterpret_cast<probe::NativeFileCreate>(GetProcAddress(ntdll, "NtCreateFile"));
    originalFileOpen = reinterpret_cast<probe::NativeFileOpen>(GetProcAddress(ntdll, "NtOpenFile"));
    originalSetFileInformation = reinterpret_cast<SetFileInformation>(GetProcAddress(ntdll, "NtSetInformationFile"));
    originalQueryAttributes = reinterpret_cast<QueryAttributes>(GetProcAddress(ntdll, "NtQueryAttributesFile"));
    originalQueryFullAttributes = reinterpret_cast<QueryAttributes>(GetProcAddress(ntdll, "NtQueryFullAttributesFile"));
    return originalFileCreate && originalFileOpen && originalSetFileInformation
        && originalQueryAttributes && originalQueryFullAttributes;
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
// A 511-character relative path has at most 255 one-character parents.
constexpr unsigned ParentHandleCapacity = 512 / 2;
constexpr unsigned CanonicalPrivatePathCharacters = 512 + 512;
bool validRelative(const wchar_t* path, unsigned count, DWORD* reason = nullptr) {
    using namespace startup_observation;
    const auto reject = [reason](PathReason value) { if (reason) *reason = value; return false; };
    if (count >= 512) return reject(TotalLength);
    if (!count) return true;
    unsigned component = 0;
    for (unsigned i = 0; i < count; ++i) {
        const wchar_t ch = path[i];
        if (ch == L'\\') {
            if (!component) return reject(EmptyComponent);
            if (path[i - 1] == L'.' || path[i - 1] == L' ') return reject(TrailingDotOrSpace);
            component = 0;
            continue;
        }
        if (ch < 32 || ch == L':' || ch == L'/' || ch == L'"' || ch == L'<' || ch == L'>'
            || ch == L'|' || ch == L'?' || ch == L'*') return reject(UnsupportedCharacter);
        if (!component && ch == L' ') return reject(LeadingSpace);
        if (ch >= 0xd800 && ch <= 0xdbff) {
            if (i + 1 == count || path[i + 1] < 0xdc00 || path[i + 1] > 0xdfff) return reject(InvalidUtf16);
            ++i; ++component;
        } else if (ch >= 0xdc00 && ch <= 0xdfff) return reject(InvalidUtf16);
        if (++component > 255) return reject(ComponentLength);
    }
    if (!component) return reject(EmptyComponent);
    if (path[count - 1] == L'.' || path[count - 1] == L' ') return reject(TrailingDotOrSpace);
    return true;
}
FileRoute fileRoute(const OBJECT_ATTRIBUTES& attributes, wchar_t (&path)[512], unsigned& length,
                    startup_observation::PathRejection* rejection = nullptr) {
    using namespace startup_observation;
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
        if (attributes.RootDirectory) {
            if (rejection) *rejection = {AbsoluteRootConflict, RootConflict};
            return FileRoute::Unsupported;
        }
    } else {
        FILE_ID_INFO rootId{};
        if (!attributes.RootDirectory || !directoryInfo(attributes.RootDirectory, rootId))
            return FileRoute::Other;
        if (!sameFileId(rootId, logicalDirectoryId) && !sameFileId(rootId, ownDirectoryId)) {
            wchar_t parentPath[CanonicalPrivatePathCharacters]{};
            const DWORD size = GetFinalPathNameByHandleW(attributes.RootDirectory,
                parentPath, CanonicalPrivatePathCharacters, VOLUME_NAME_NT);
            if (!size || size >= CanonicalPrivatePathCharacters) return FileRoute::Other;
            const auto bytes = static_cast<USHORT>(size * sizeof(wchar_t));
            UNICODE_STRING parent{bytes, bytes, parentPath};
            unsigned offset = 0;
            if (!pathPrefix(parent, logicalDevice, offset) && !pathPrefix(parent, ownDevice, offset))
                return FileRoute::Other;
            length = size - offset;
            DWORD reason = 0;
            if (!validRelative(parentPath + offset, length, &reason)) {
                if (rejection) *rejection = {RelativeParent, reason};
                return FileRoute::Unsupported;
            }
            memcpy(path, parentPath + offset, length * sizeof(wchar_t));
            if (count) path[length++] = L'\\';
        }
    }
    if (count - leaf >= 512 - length) {
        if (rejection) *rejection = {ComposedPath, TotalLength};
        return FileRoute::Unsupported;
    }
    memcpy(path + length, name.Buffer + leaf, (count - leaf) * sizeof(wchar_t));
    length += count - leaf;
    path[length] = L'\0';
    // FindFirstFile's native directory open retains its final separator. Keep it for the OS.
    const unsigned validated = length > 1 && path[length - 1] == L'\\' ? length - 1 : length;
    DWORD reason = 0;
    if (validRelative(path, validated, &reason)) return FileRoute::Private;
    if (rejection) *rejection = {ComposedPath, reason};
    return FileRoute::Unsupported;
}
constexpr ULONG DontReparse = 0x1000; // OBJ_DONT_REPARSE from OBJECT_ATTRIBUTES.
constexpr NTSTATUS Unsupported = static_cast<NTSTATUS>(0xc00000bbUL);
NTSTATUS privateParent(wchar_t* path, unsigned length, HANDLE& parent,
                       UNICODE_STRING& leaf, HANDLE (&held)[ParentHandleCapacity], unsigned& count) {
    if (length >= 512) return Unsupported;
    parent = fileOwnRoot;
    unsigned start = 0;
    for (unsigned i = 0; i < length; ++i) {
        if (path[i] != L'\\') continue;
        if (count >= ParentHandleCapacity) return Unsupported;
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
bool unsupportedFile(ACCESS_MASK access, ULONG options, bool root, bool existing) {
    return (root && (access & (DELETE | WRITE_DAC | WRITE_OWNER | GENERIC_ALL)))
        || ((access & FILE_EXECUTE) && !(root || (options & FILE_DIRECTORY_FILE)
            || (existing && !(options & FILE_NON_DIRECTORY_FILE))))
        || (options & FILE_OPEN_BY_FILE_ID) != 0
        || (root && (options & FILE_DELETE_ON_CLOSE))
        || (root && (options & FILE_NON_DIRECTORY_FILE));
}
ULONG privateOpenOptions(ACCESS_MASK access, ULONG options) {
    // FILE_TRAVERSE and FILE_EXECUTE share a bit. An unspecified-type existing
    // open may traverse a directory, but must never gain that access to a file.
    // Create/overwrite without explicit DIRECTORY_FILE remains refused above.
    return (access & FILE_EXECUTE) ? options | FILE_DIRECTORY_FILE : options;
}
NTSTATUS NTAPI translatedFileCreate(PHANDLE handle, ACCESS_MASK access, POBJECT_ATTRIBUTES attributes,
    PIO_STATUS_BLOCK io, PLARGE_INTEGER allocation, ULONG fileAttributes, ULONG share,
    ULONG disposition, ULONG options, PVOID ea, ULONG eaLength) {
    const DWORD saved = GetLastError();
    NTSTATUS status = Unsupported;
    HANDLE held[ParentHandleCapacity]{};
    unsigned heldCount = 0;
    unsigned diagnostic = 0, diagnosticLength = 0;
    startup_observation::PathRejection rejection{};
    __try {
        wchar_t path[512]{};
        unsigned length = 0;
        const FileRoute route = attributes ? fileRoute(*attributes, path, length, &rejection) : FileRoute::Other;
        diagnosticLength = length;
        if (route != FileRoute::Other) diagnostic = route == FileRoute::Unsupported ? 1 : 2;
        if (route == FileRoute::Other) {
            SetLastError(saved);
            status = originalFileCreate(handle, access, attributes, io, allocation,
                fileAttributes, share, disposition, options, ea, eaLength);
        } else if (route == FileRoute::Private
            && !unsupportedFile(access, options, length == 0, disposition == 1)
            && (length != 0 || disposition == 1)) {
            OBJECT_ATTRIBUTES copy = *attributes;
            UNICODE_STRING leaf{};
            status = privateParent(path, length, copy.RootDirectory, leaf, held, heldCount);
            diagnostic = 3;
            if (status >= 0) {
                copy.ObjectName = &leaf;
                copy.Attributes |= DontReparse;
                SetLastError(saved);
                status = originalFileCreate(handle, access, &copy, io, allocation,
                    fileAttributes, share, disposition, privateOpenOptions(access, options), ea, eaLength);
                diagnostic = 4;
            }
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) { status = static_cast<NTSTATUS>(GetExceptionCode()); }
    while (heldCount) CloseHandle(held[--heldCount]);
    if (status < 0 && diagnostic && diagnostic != 4) {
        __try { if (handle) *handle = nullptr; if (io) { io->Status = status; io->Information = 0; } }
        __except (EXCEPTION_EXECUTE_HANDLER) { status = static_cast<NTSTATUS>(GetExceptionCode()); }
    }
    if (diagnostic) startup_observation::storageResult(status >= 0 ? 0 : diagnostic,
        1, access, options, disposition, diagnosticLength, status, rejection);
    SetLastError(saved);
    return status;
}
NTSTATUS NTAPI translatedFileOpen(PHANDLE handle, ACCESS_MASK access, POBJECT_ATTRIBUTES attributes,
    PIO_STATUS_BLOCK io, ULONG share, ULONG options) {
    const DWORD saved = GetLastError();
    NTSTATUS status = Unsupported;
    HANDLE held[ParentHandleCapacity]{};
    unsigned heldCount = 0;
    unsigned diagnostic = 0, diagnosticLength = 0;
    startup_observation::PathRejection rejection{};
    __try {
        wchar_t path[512]{};
        unsigned length = 0;
        const FileRoute route = attributes ? fileRoute(*attributes, path, length, &rejection) : FileRoute::Other;
        diagnosticLength = length;
        if (route != FileRoute::Other) diagnostic = route == FileRoute::Unsupported ? 1 : 2;
        if (route == FileRoute::Other) {
            SetLastError(saved);
            status = originalFileOpen(handle, access, attributes, io, share, options);
        } else if (route == FileRoute::Private && !unsupportedFile(access, options, length == 0, true)) {
            OBJECT_ATTRIBUTES copy = *attributes;
            UNICODE_STRING leaf{};
            status = privateParent(path, length, copy.RootDirectory, leaf, held, heldCount);
            diagnostic = 3;
            if (status >= 0) {
                copy.ObjectName = &leaf;
                copy.Attributes |= DontReparse;
                SetLastError(saved);
                status = originalFileOpen(handle, access, &copy, io, share, privateOpenOptions(access, options));
                diagnostic = 4;
            }
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) { status = static_cast<NTSTATUS>(GetExceptionCode()); }
    while (heldCount) CloseHandle(held[--heldCount]);
    if (status < 0 && diagnostic && diagnostic != 4) {
        __try { if (handle) *handle = nullptr; if (io) { io->Status = status; io->Information = 0; } }
        __except (EXCEPTION_EXECUTE_HANDLER) { status = static_cast<NTSTATUS>(GetExceptionCode()); }
    }
    if (diagnostic) startup_observation::storageResult(status >= 0 ? 0 : diagnostic,
        2, access, options, 0, diagnosticLength, status, rejection);
    SetLastError(saved);
    return status;
}

NTSTATUS queryPrivateAttributes(QueryAttributes original, POBJECT_ATTRIBUTES attributes, PVOID output) {
    const DWORD saved = GetLastError();
    NTSTATUS status = Unsupported;
    HANDLE held[ParentHandleCapacity]{};
    unsigned heldCount = 0;
    __try {
        wchar_t path[512]{};
        unsigned length = 0;
        const FileRoute route = attributes ? fileRoute(*attributes, path, length) : FileRoute::Other;
        if (route == FileRoute::Other) {
            SetLastError(saved);
            status = original(attributes, output);
        } else if (route == FileRoute::Private) {
            OBJECT_ATTRIBUTES copy = *attributes;
            UNICODE_STRING leaf{};
            status = privateParent(path, length, copy.RootDirectory, leaf, held, heldCount);
            if (status >= 0) {
                copy.ObjectName = &leaf; copy.Attributes |= DontReparse;
                SetLastError(saved);
                status = original(&copy, output);
            }
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) { status = static_cast<NTSTATUS>(GetExceptionCode()); }
    while (heldCount) CloseHandle(held[--heldCount]);
    SetLastError(saved);
    return status;
}
NTSTATUS NTAPI translatedQueryAttributes(POBJECT_ATTRIBUTES attributes, PVOID output) {
    return queryPrivateAttributes(originalQueryAttributes, attributes, output);
}
NTSTATUS NTAPI translatedQueryFullAttributes(POBJECT_ATTRIBUTES attributes, PVOID output) {
    return queryPrivateAttributes(originalQueryFullAttributes, attributes, output);
}

// The returned data handle is private; name-changing operations must keep it private too.
struct RenameInformation {
    ULONG flags;
    HANDLE root;
    ULONG nameBytes;
    wchar_t name[512];
};
static_assert(offsetof(RenameInformation, name) == offsetof(FILE_RENAME_INFO, FileName));
constexpr ULONG MinimumRenameBytes = sizeof(FILE_RENAME_INFO);
NTSTATUS NTAPI translatedSetInformation(HANDLE file, PIO_STATUS_BLOCK io, PVOID information,
                                        ULONG bytes, FILE_INFORMATION_CLASS informationClass) {
    const DWORD saved = GetLastError();
    NTSTATUS status = Unsupported;
    HANDLE held[ParentHandleCapacity]{};
    unsigned heldCount = 0;
    bool handled = false;
    __try {
        wchar_t source[CanonicalPrivatePathCharacters]{};
        const DWORD size = GetFileType(file) == FILE_TYPE_DISK
            ? GetFinalPathNameByHandleW(file, source, CanonicalPrivatePathCharacters, VOLUME_NAME_NT) : 0;
        UNICODE_STRING sourceName{static_cast<USHORT>(size < CanonicalPrivatePathCharacters ? size * 2 : 0),
            static_cast<USHORT>(size < CanonicalPrivatePathCharacters ? size * 2 : 0), source};
        unsigned sourceLeaf = 0;
        const bool own = size && size < CanonicalPrivatePathCharacters && pathPrefix(sourceName, ownDevice, sourceLeaf);
        const ULONG kind = static_cast<ULONG>(informationClass);
        if (own && (kind == 10 || kind == 65)) {
            handled = true;
            if (sourceLeaf < size && information && bytes >= MinimumRenameBytes) {
                const auto* input = static_cast<const RenameInformation*>(information);
                const ULONG nameBytes = input->nameBytes;
                if (nameBytes && nameBytes % 2 == 0 && nameBytes < sizeof(input->name)
                    && nameBytes <= bytes - offsetof(RenameInformation, name)) {
                    wchar_t destination[512]{};
                    UNICODE_STRING name{static_cast<USHORT>(nameBytes), static_cast<USHORT>(nameBytes),
                        const_cast<PWSTR>(input->name)};
                    OBJECT_ATTRIBUTES attributes{sizeof(OBJECT_ATTRIBUTES), input->root, &name,
                        OBJ_CASE_INSENSITIVE, nullptr, nullptr};
                    unsigned length = 0;
                    FileRoute route = FileRoute::Unsupported;
                    if (input->root || input->name[0] == L'\\') {
                        route = fileRoute(attributes, destination, length);
                    } else {
                        // Rootless simple names are relative to the source's current parent.
                        unsigned parent = size;
                        while (parent > sourceLeaf && source[parent - 1] != L'\\') --parent;
                        const unsigned prefix = parent - sourceLeaf;
                        if (prefix + nameBytes / 2 < 512) {
                            memcpy(destination, source + sourceLeaf, prefix * 2);
                            memcpy(destination + prefix, input->name, nameBytes);
                            length = prefix + nameBytes / 2;
                            if (validRelative(destination, length)) route = FileRoute::Private;
                        }
                    }
                    if (route == FileRoute::Private && length) {
                        HANDLE parent = nullptr;
                        UNICODE_STRING leaf{};
                        status = privateParent(destination, length, parent, leaf, held, heldCount);
                        if (status >= 0) {
                            RenameInformation copy{};
                            copy.flags = input->flags; copy.root = parent; copy.nameBytes = leaf.Length;
                            memcpy(copy.name, leaf.Buffer, leaf.Length);
                            // Keep the padded native structure for a one-character leaf.
                            const auto renamedBytes = static_cast<ULONG>(offsetof(RenameInformation, name) + leaf.Length);
                            SetLastError(saved);
                            status = originalSetFileInformation(file, io, &copy,
                                renamedBytes < MinimumRenameBytes ? MinimumRenameBytes : renamedBytes, informationClass);
                        }
                    }
                }
            }
        } else if (own && (kind == 11 || kind == 57 || kind == 72 || kind == 31 || kind == 40 || kind == 71
            || (sourceLeaf == size && (kind == 13 || kind == 64)))) {
            handled = true; // No link/short-name/case-mode bypass or deletion of the profile root.
        } else {
            SetLastError(saved);
            status = originalSetFileInformation(file, io, information, bytes, informationClass);
        }
        if (handled && status < 0 && io) { io->Status = status; io->Information = 0; }
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
    // A direct import makes Shell32 a loader dependency; do not assume the target imported it.
    originalKnownPath = &SHGetKnownFolderPath;
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
bool initializeRegistry(const InstallConfig& config);
bool attachRegistry();
bool selectFixtureMutex(const probe::Receipt& config) {
    if (config.fileRound > 1) return false;
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

    return true;
}

using CreateProcessCall = BOOL (WINAPI*)(LPCWSTR, LPWSTR, LPSECURITY_ATTRIBUTES,
    LPSECURITY_ATTRIBUTES, BOOL, DWORD, LPVOID, LPCWSTR, LPSTARTUPINFOW, LPPROCESS_INFORMATION);
CreateProcessCall originalProcess = nullptr;
using MutexW = HANDLE (WINAPI*)(LPSECURITY_ATTRIBUTES, BOOL, LPCWSTR);
using MutexA = HANDLE (WINAPI*)(LPSECURITY_ATTRIBUTES, BOOL, LPCSTR);
using MutexExW = HANDLE (WINAPI*)(LPSECURITY_ATTRIBUTES, LPCWSTR, DWORD, DWORD);
using MutexExA = HANDLE (WINAPI*)(LPSECURITY_ATTRIBUTES, LPCSTR, DWORD, DWORD);
MutexW originalMutexW = nullptr;
MutexA originalMutexA = nullptr;
MutexExW originalMutexExW = nullptr;
MutexExA originalMutexExA = nullptr;

struct CallerRange { ULONG_PTR base; DWORD bytes, kind; };
CallerRange callerRanges[4]{};
void initializeCallerRanges() {
    const wchar_t* modules[]{nullptr, L"kernelbase.dll", L"kernel32.dll", L"SecureEngineSDK64.dll"};
    for (unsigned i = 0; i < 4; ++i) {
        const HMODULE module = GetModuleHandleW(modules[i]); // Already loaded only; never LoadLibrary.
        if (!module) continue;
        __try {
            const auto dos = reinterpret_cast<const IMAGE_DOS_HEADER*>(module);
            if (dos->e_magic != IMAGE_DOS_SIGNATURE || dos->e_lfanew <= 0 || dos->e_lfanew > 0x100000)
                continue;
            const auto nt = reinterpret_cast<const IMAGE_NT_HEADERS64*>(
                reinterpret_cast<const BYTE*>(module) + dos->e_lfanew);
            if (nt->Signature != IMAGE_NT_SIGNATURE || nt->OptionalHeader.Magic != IMAGE_NT_OPTIONAL_HDR64_MAGIC
                || !nt->OptionalHeader.SizeOfImage || nt->OptionalHeader.SizeOfImage > 0x7fffffff) continue;
            callerRanges[i] = {reinterpret_cast<ULONG_PTR>(module), nt->OptionalHeader.SizeOfImage, i + 1};
        } __except (EXCEPTION_EXECUTE_HANDLER) {}
    }
}
void attributeCaller(startup_observation::Record& record) {
    for (const auto& range : callerRanges) {
        if (range.bytes && record.caller >= range.base && record.caller - range.base < range.bytes) {
            record.callerModule = range.kind;
            record.callerRva = static_cast<DWORD>(record.caller - range.base);
            record.callerImageBytes = range.bytes;
            return;
        }
    }
}
template<class Character>
bool copyGuidName(startup_observation::Record& record, const Character* name, bool owner) {
    if (!name || !owner) return false; // The observed candidate is a 36-character GUID with initial ownership.
    __try {
        for (unsigned i = 0; i < 36; ++i) {
            const auto ch = static_cast<wchar_t>(name[i]);
            const bool separator = i == 8 || i == 13 || i == 18 || i == 23;
            if (separator ? ch != L'-'
                : !((ch >= L'0' && ch <= L'9') || (ch >= L'a' && ch <= L'f')
                    || (ch >= L'A' && ch <= L'F'))) return false;
            record.name[i] = ch;
        }
        if (name[36] != 0) return false;
    } __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
    record.nameBytes = 72;
    record.flags |= startup_observation::InitialOwner;
    return true;
}
void attributeNative(startup_observation::Record& record) {
    if (record.root && record.nameBytes && record.name[0] != L'\\'
        && !(record.flags & startup_observation::UnreadableArguments)
        && baseDirectory && compareHandles(reinterpret_cast<HANDLE>(record.root), baseDirectory))
        record.namespaceKind = startup_observation::SessionBaseNamedObjects;
    startup_observation::Record selected{};
    if (record.nameBytes == 72 && copyGuidName(selected, record.name,
        (record.flags & startup_observation::InitialOwner) != 0)) attributeCaller(record);
}

DWORD singletonCallerRva = 0;
std::atomic<unsigned> singletonBinding{0};
wchar_t singletonName[37]{}, singletonSuffix[80]{};
bool initializeSelectedMutex(const InstallConfig& config, bool observe) {
    if (!config.education || config.education->scope != education_bootstrap::StorageRegistrySelectedMutex)
        return true;
    singletonCallerRva = config.education->singletonCallerRva;
    if (!observe || !singletonCallerRva || !callerRanges[0].bytes
        || singletonCallerRva >= callerRanges[0].bytes) return false;
    // The verified private directory identity supplies a stable, per-profile suffix.
    int used = swprintf_s(singletonSuffix, L"_Mivubi_%016llx_",
        static_cast<unsigned long long>(ownDirectoryId.VolumeSerialNumber));
    if (used < 0) return false;
    for (BYTE byte : ownDirectoryId.FileId.Identifier) {
        if (swprintf_s(singletonSuffix + used, 80 - static_cast<size_t>(used), L"%02x",
            static_cast<unsigned>(byte)) != 2) return false;
        used += 2;
    }
    return true;
}
void bindSelectedMutex(const startup_observation::Record& record) {
    if (!singletonCallerRva || record.callerModule != startup_observation::MainImage
        || record.callerRva != singletonCallerRva) return;
    unsigned empty = 0;
    if (singletonBinding.compare_exchange_strong(empty, 1, std::memory_order_acq_rel)) {
        memcpy(singletonName, record.name, 72);
        singletonName[36] = 0;
        singletonBinding.store(2, std::memory_order_release);
    }
}
bool mapSelectedMutex(const startup_observation::Record& record, const OBJECT_ATTRIBUTES* source,
    OBJECT_ATTRIBUTES& copy, UNICODE_STRING& text, wchar_t (&buffer)[256]) {
    if (singletonBinding.load(std::memory_order_acquire) != 2 || record.nameBytes != 72
        || record.namespaceKind != startup_observation::SessionBaseNamedObjects) return false;
    const bool insensitive = (record.attributes & OBJ_CASE_INSENSITIVE) != 0;
    for (unsigned i = 0; i < 36; ++i)
        if (folded(record.name[i], insensitive) != folded(singletonName[i], insensitive)) return false;
    __try {
        if (!source || source->Length != sizeof(OBJECT_ATTRIBUTES)) return false;
        if (swprintf_s(buffer, L"%s%s", singletonName, singletonSuffix) < 0) return false;
        const auto bytes = static_cast<USHORT>(wcslen(buffer) * sizeof(wchar_t));
        text = {bytes, bytes, buffer};
        copy = *source;
        copy.ObjectName = &text; // Preserve access, original root, attributes and security pointers.
        return true;
    } __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}

void finishWin32Mutex(startup_observation::Record& record, HANDLE result, DWORD error) {
    startup_observation::logging = true;
    record.status = result ? TRUE : FALSE;
    record.returned = reinterpret_cast<ULONG_PTR>(result);
    attributeCaller(record); // Cached image bounds only; no allocation, module/file/profile lookup in hooks.
    startup_observation::finish(record, error);
}
HANDLE WINAPI observedMutexW(LPSECURITY_ATTRIBUTES security, BOOL owner, LPCWSTR name) {
    const DWORD before = GetLastError();
    startup_observation::Record record{};
    const bool observe = copyGuidName(record, name, owner != FALSE)
        && startup_observation::begin(record, startup_observation::Win32MutexW, _ReturnAddress());
    record.argument = MUTEX_ALL_ACCESS;
    SetLastError(before);
    const HANDLE result = originalMutexW(security, owner, name);
    const DWORD after = GetLastError();
    if (observe) finishWin32Mutex(record, result, after);
    SetLastError(after);
    return result;
}
HANDLE WINAPI observedMutexA(LPSECURITY_ATTRIBUTES security, BOOL owner, LPCSTR name) {
    const DWORD before = GetLastError();
    startup_observation::Record record{};
    const bool candidate = copyGuidName(record, name, owner != FALSE);
    const bool observe = candidate
        && startup_observation::begin(record, startup_observation::Win32MutexA, _ReturnAddress());
    if (candidate && singletonCallerRva) {
        record.caller = reinterpret_cast<ULONG_PTR>(_ReturnAddress());
        attributeCaller(record);
        bindSelectedMutex(record); // Functional binding is not limited by the observation clock/capacity.
    }
    record.argument = MUTEX_ALL_ACCESS;
    SetLastError(before);
    const HANDLE result = originalMutexA(security, owner, name);
    const DWORD after = GetLastError();
    if (observe) finishWin32Mutex(record, result, after);
    SetLastError(after);
    return result;
}
HANDLE WINAPI observedMutexExW(LPSECURITY_ATTRIBUTES security, LPCWSTR name, DWORD flags, DWORD access) {
    const DWORD before = GetLastError();
    startup_observation::Record record{};
    const bool observe = copyGuidName(record, name, (flags & CREATE_MUTEX_INITIAL_OWNER) != 0)
        && startup_observation::begin(record, startup_observation::Win32MutexExW, _ReturnAddress());
    record.argument = access; record.attributes = flags;
    SetLastError(before);
    const HANDLE result = originalMutexExW(security, name, flags, access);
    const DWORD after = GetLastError();
    if (observe) finishWin32Mutex(record, result, after);
    SetLastError(after);
    return result;
}
HANDLE WINAPI observedMutexExA(LPSECURITY_ATTRIBUTES security, LPCSTR name, DWORD flags, DWORD access) {
    const DWORD before = GetLastError();
    startup_observation::Record record{};
    const bool observe = copyGuidName(record, name, (flags & CREATE_MUTEX_INITIAL_OWNER) != 0)
        && startup_observation::begin(record, startup_observation::Win32MutexExA, _ReturnAddress());
    record.argument = access; record.attributes = flags;
    SetLastError(before);
    const HANDLE result = originalMutexExA(security, name, flags, access);
    const DWORD after = GetLastError();
    if (observe) finishWin32Mutex(record, result, after);
    SetLastError(after);
    return result;
}

NTSTATUS NTAPI observedCreate(PHANDLE handle, ACCESS_MASK access, POBJECT_ATTRIBUTES attributes, BOOLEAN owner) {
    const DWORD before = GetLastError();
    startup_observation::Record record{};
    const bool observe = startup_observation::begin(record, startup_observation::CreateMutant, _ReturnAddress());
    if (observe || singletonBinding.load(std::memory_order_acquire) == 2) {
        startup_observation::logging = true;
        startup_observation::objectArguments(record, access, attributes);
        if (owner) record.flags |= startup_observation::InitialOwner;
        attributeNative(record);
        startup_observation::logging = false;
    }
    SetLastError(before);
    OBJECT_ATTRIBUTES mapped{};
    UNICODE_STRING text{};
    wchar_t replacement[256]{};
    const bool translated = mapSelectedMutex(record, attributes, mapped, text, replacement);
    if (translated) record.flags |= startup_observation::SingletonTranslated;
    SetLastError(before);
    const NTSTATUS status = originalCreate(handle, access, translated ? &mapped : attributes, owner);
    const DWORD after = GetLastError();
    if (observe) {
        startup_observation::logging = true;
        startup_observation::objectResult(record, status, handle);
        startup_observation::finish(record, after);
    }
    SetLastError(after);
    return status;
}
NTSTATUS NTAPI observedOpen(PHANDLE handle, ACCESS_MASK access, POBJECT_ATTRIBUTES attributes) {
    const DWORD before = GetLastError();
    startup_observation::Record record{};
    const bool observe = startup_observation::begin(record, startup_observation::OpenMutant, _ReturnAddress());
    if (observe || singletonBinding.load(std::memory_order_acquire) == 2) {
        startup_observation::logging = true;
        startup_observation::objectArguments(record, access, attributes);
        attributeNative(record);
        startup_observation::logging = false;
    }
    SetLastError(before);
    OBJECT_ATTRIBUTES mapped{};
    UNICODE_STRING text{};
    wchar_t replacement[256]{};
    const bool translated = mapSelectedMutex(record, attributes, mapped, text, replacement);
    if (translated) record.flags |= startup_observation::SingletonTranslated;
    SetLastError(before);
    const NTSTATUS status = originalOpen(handle, access, translated ? &mapped : attributes);
    const DWORD after = GetLastError();
    if (observe) {
        startup_observation::logging = true;
        startup_observation::objectResult(record, status, handle);
        startup_observation::finish(record, after);
    }
    SetLastError(after);
    return status;
}
BOOL WINAPI observedProcess(LPCWSTR application, LPWSTR command, LPSECURITY_ATTRIBUTES processSecurity,
    LPSECURITY_ATTRIBUTES threadSecurity, BOOL inherit, DWORD flags, LPVOID environment,
    LPCWSTR directory, LPSTARTUPINFOW startup, LPPROCESS_INFORMATION info) {
    const DWORD before = GetLastError();
    startup_observation::Record record{};
    const bool observe = startup_observation::begin(record, startup_observation::CreateProcess, _ReturnAddress());
    if (observe) { record.argument = flags; record.attributes = inherit ? 1 : 0; }
    SetLastError(before);
    const BOOL result = originalProcess(application, command, processSecurity, threadSecurity,
        inherit, flags, environment, directory, startup, info);
    const DWORD after = GetLastError();
    if (observe) {
        startup_observation::logging = true;
        startup_observation::processResult(record, result, info);
        startup_observation::finish(record, after);
    }
    SetLastError(after);
    return result;
}

bool install(const InstallConfig& config, bool mutex) {
    slotSuffix = config.slot == 1 ? L'A' : L'B';
    const HMODULE ntdll = GetModuleHandleW(L"ntdll.dll");
    if (!ntdll) return false;
    const bool observe = startup_observation::active != nullptr;
    const HMODULE kernel = GetModuleHandleW(L"kernel32.dll");
    originalProcess = kernel ? reinterpret_cast<CreateProcessCall>(GetProcAddress(kernel, "CreateProcessW")) : nullptr;
    if (observe) {
        originalMutexW = kernel ? reinterpret_cast<MutexW>(GetProcAddress(kernel, "CreateMutexW")) : nullptr;
        originalMutexA = kernel ? reinterpret_cast<MutexA>(GetProcAddress(kernel, "CreateMutexA")) : nullptr;
        originalMutexExW = kernel ? reinterpret_cast<MutexExW>(GetProcAddress(kernel, "CreateMutexExW")) : nullptr;
        originalMutexExA = kernel ? reinterpret_cast<MutexExA>(GetProcAddress(kernel, "CreateMutexExA")) : nullptr;
        if (!originalProcess || !originalMutexW || !originalMutexA || !originalMutexExW || !originalMutexExA)
            return false;
        initializeCallerRanges();
    }
    originalCreate = reinterpret_cast<probe::NativeCreate>(GetProcAddress(ntdll, "NtCreateMutant"));
    originalOpen = reinterpret_cast<probe::NativeOpen>(GetProcAddress(ntdll, "NtOpenMutant"));
    openDirectory = reinterpret_cast<probe::NativeDirectory>(GetProcAddress(ntdll, "NtOpenDirectoryObject"));
    const HMODULE kernelbase = GetModuleHandleW(L"kernelbase.dll");
    compareHandles = kernelbase ? reinterpret_cast<probe::CompareHandles>(
        GetProcAddress(kernelbase, "CompareObjectHandles")) : nullptr;
    if (!originalCreate || !originalOpen || !openDirectory || !compareHandles || !initializeFiles(config, ntdll)
        || !initializeAppData() || !initializeRegistry(config) || !initializeSelectedMutex(config, observe)) return false;
    DWORD session = 0;
    wchar_t path[80]{};
    if (!ProcessIdToSessionId(GetCurrentProcessId(), &session)
        || swprintf_s(path, L"\\Sessions\\%lu\\BaseNamedObjects", session) < 0) return false;
    const auto bytes = static_cast<USHORT>(wcslen(path) * sizeof(wchar_t));
    UNICODE_STRING rootName{bytes, bytes, path};
    OBJECT_ATTRIBUTES root{sizeof(OBJECT_ATTRIBUTES), nullptr, &rootName, OBJ_CASE_INSENSITIVE, nullptr, nullptr};
    if ((mutex || observe) && openDirectory(&baseDirectory, 0x0002 /* DIRECTORY_TRAVERSE */, &root) < 0) return false;

    // One native layer serves the direct calls and the tested Win32 wrappers.
    if (!DetourRestoreAfterWith() || DetourTransactionBegin() != NO_ERROR) {
        CloseHandle(baseDirectory); baseDirectory = nullptr; return false;
    }
    if (DetourUpdateThread(GetCurrentThread()) != NO_ERROR
        || ((mutex || observe) && (DetourAttach(reinterpret_cast<PVOID*>(&originalCreate),
                        mutex ? reinterpret_cast<PVOID>(translatedCreate) : reinterpret_cast<PVOID>(observedCreate)) != NO_ERROR
            || DetourAttach(reinterpret_cast<PVOID*>(&originalOpen),
                        mutex ? reinterpret_cast<PVOID>(translatedOpen) : reinterpret_cast<PVOID>(observedOpen)) != NO_ERROR))
        || (observe && DetourAttach(reinterpret_cast<PVOID*>(&originalProcess),
                        reinterpret_cast<PVOID>(observedProcess)) != NO_ERROR)
        || (observe && (DetourAttach(reinterpret_cast<PVOID*>(&originalMutexW), reinterpret_cast<PVOID>(observedMutexW)) != NO_ERROR
            || DetourAttach(reinterpret_cast<PVOID*>(&originalMutexA), reinterpret_cast<PVOID>(observedMutexA)) != NO_ERROR
            || DetourAttach(reinterpret_cast<PVOID*>(&originalMutexExW), reinterpret_cast<PVOID>(observedMutexExW)) != NO_ERROR
            || DetourAttach(reinterpret_cast<PVOID*>(&originalMutexExA), reinterpret_cast<PVOID>(observedMutexExA)) != NO_ERROR))
        || DetourAttach(reinterpret_cast<PVOID*>(&originalFileCreate),
                        reinterpret_cast<PVOID>(translatedFileCreate)) != NO_ERROR
        || DetourAttach(reinterpret_cast<PVOID*>(&originalFileOpen),
                        reinterpret_cast<PVOID>(translatedFileOpen)) != NO_ERROR
        || DetourAttach(reinterpret_cast<PVOID*>(&originalSetFileInformation),
                        reinterpret_cast<PVOID>(translatedSetInformation)) != NO_ERROR
        || DetourAttach(reinterpret_cast<PVOID*>(&originalQueryAttributes),
                        reinterpret_cast<PVOID>(translatedQueryAttributes)) != NO_ERROR
        || DetourAttach(reinterpret_cast<PVOID*>(&originalQueryFullAttributes),
                        reinterpret_cast<PVOID>(translatedQueryFullAttributes)) != NO_ERROR
        || DetourAttach(reinterpret_cast<PVOID*>(&originalKnownPath),
                        reinterpret_cast<PVOID>(translatedKnownPath)) != NO_ERROR || !attachRegistry()
        || !presentation_enrollment::attach()) {
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
extern "C" BOOL BootstrapPresentationSnapshot(presentation_enrollment::Snapshot* output) {
    if (!output || output->bytes != sizeof(*output)) return FALSE;
    *output = presentation_enrollment::snapshot();
    return TRUE;
}

extern "C" BOOL WINAPI DllMain(HINSTANCE, DWORD reason, LPVOID) {
    if (reason != DLL_PROCESS_ATTACH) return TRUE; // Process-lifetime fixture; no live unload.
    if (education_bootstrap::present()) {
        education_bootstrap::Config config{};
        // The optional receipt is solely for our controlled executable's assertions.
        probe::View test;
        const bool controlled = test.open() && test.value->mode == probe::ConfiguredStorage;
        if (controlled) probe::record(test.value, probe::Attach);
        const bool valid = education_bootstrap::read(config);
        InstallConfig installConfig{config.slot, config.logicalRoot, config.privateRoot,
            config.registryRoot, 1, config.userKeyPath, &config};
        if (!valid || !presentation_enrollment::prepare(config)
            || !startup_observation::attach(config) || !install(installConfig, false)) {
            if (controlled) probe::record(test.value, probe::Rejected);
            return FALSE;
        }
        if (controlled) {
            InterlockedExchange(&test.value->armed, 1);
            probe::record(test.value, probe::Armed);
        }
        return TRUE; // Selected storage hooks only; no provider/auth/singleton assertion.
    }
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
        const auto& source = *view.value;
        const InstallConfig config{source.slot, source.fileLogicalRoot, source.fileOwnRoot,
            source.inheritedRegistry, source.registryBacking, source.registryUserPath};
        if (!selectFixtureMutex(source) || !install(config, true)) {
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
using Enumerate = NTSTATUS (NTAPI*)(HANDLE, ULONG, ULONG, PVOID, ULONG, PULONG);
Enumerate originalEnumerateKey = nullptr, originalEnumerateValue = nullptr;

wchar_t userKeyPath[512]{}, storePrefix[512]{}, storeOwn[512]{};
bool predefinedMapped = false;
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
    if (storePrefix[0] && pathPrefix(absolute, storePrefix, leaf)) {
        // Only our backing subtree bypasses translation; peer/container paths are refused.
        return pathPrefix(absolute, storeOwn, leaf) ? KeyRoute::Other : KeyRoute::Unsupported;
    }
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
NTSTATUS enumeratePrivate(HANDLE key, ULONG index, ULONG information,
    PVOID output, ULONG length, PULONG returned, Enumerate original, ACCESS_MASK right) {
    const DWORD saved = GetLastError();
    NTSTATUS status = Unsupported;
    HANDLE mapped = nullptr;
    __try {
        KeyRoute route = KeyRoute::Other;
        status = openPrivateValue(key, right, mapped, route);
        if (status >= 0) {
            SetLastError(saved);
            status = original(route == KeyRoute::Private ? mapped : key,
                index, information, output, length, returned);
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) { status = static_cast<NTSTATUS>(GetExceptionCode()); }
    if (mapped) CloseHandle(mapped);
    SetLastError(saved);
    return status;
}
NTSTATUS NTAPI translatedEnumerateKey(HANDLE key, ULONG index, ULONG information,
    PVOID output, ULONG length, PULONG returned) {
    return enumeratePrivate(key, index, information, output, length, returned,
        originalEnumerateKey, KEY_ENUMERATE_SUB_KEYS);
}
NTSTATUS NTAPI translatedEnumerateValue(HANDLE key, ULONG index, ULONG information,
    PVOID output, ULONG length, PULONG returned) {
    return enumeratePrivate(key, index, information, output, length, returned,
        originalEnumerateValue, KEY_QUERY_VALUE);
}
bool attachRegistryValues() {
    return DetourAttach(reinterpret_cast<PVOID*>(&originalQueryValue), reinterpret_cast<PVOID>(translatedQueryValue)) == NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalSetValue), reinterpret_cast<PVOID>(translatedSetValue)) == NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalEnumerateKey), reinterpret_cast<PVOID>(translatedEnumerateKey)) == NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalEnumerateValue), reinterpret_cast<PVOID>(translatedEnumerateValue)) == NO_ERROR;
}

bool initializeRegistry(const InstallConfig& config) {
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
    originalEnumerateKey = reinterpret_cast<Enumerate>(GetProcAddress(ntdll, "NtEnumerateKey"));
    originalEnumerateValue = reinterpret_cast<Enumerate>(GetProcAddress(ntdll, "NtEnumerateValueKey"));
    if (!originalOpenUser || !originalOpenKey || !originalOpenKeyEx || !originalCreateKey
        || !queryKey || !originalQueryValue || !originalSetValue || !queryObject
        || !originalEnumerateKey || !originalEnumerateValue) return false;
    unsigned length = 0;
    while (length < 512 && config.registryUserPath[length]) ++length;
    constexpr wchar_t prefix[] = L"\\REGISTRY\\USER\\";
    if (length <= 15 || length >= 512) return false;
    for (unsigned i = 0; prefix[i]; ++i)
        if (folded(config.registryUserPath[i], true) != folded(prefix[i], true)) return false;
    memcpy(userKeyPath, config.registryUserPath, (length + 1) * sizeof(wchar_t));
    if ((config.registryBacking == 1) != (config.inheritedRegistry != nullptr)) return false;
    if (config.registryBacking == 1) {
        wchar_t leaf[64]{}, expected[512]{}, actual[512]{};
        if (!probe::registryStoreLeaf(ownDirectoryId, config.slot, leaf)) return false;
        if (config.education) {
            // The inherited ordinary backing is available before any EXE-main cooperation.
            // Token identity metadata only: no registry/profile open under loader lock.
            HANDLE token = nullptr;
            if (!OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &token)) return false;
            alignas(8) BYTE bytes[256]{};
            DWORD returned = 0;
            const BOOL queried = GetTokenInformation(token, TokenUser, bytes, sizeof(bytes), &returned);
            CloseHandle(token);
            if (!queried || returned < sizeof(TOKEN_USER)) return false;
            const auto sid = reinterpret_cast<TOKEN_USER*>(bytes)->User.Sid;
            if (!IsValidSid(sid)) return false;
            const auto authority = GetSidIdentifierAuthority(sid);
            unsigned long long authorityValue = 0;
            for (BYTE byte : authority->Value) authorityValue = (authorityValue << 8) | byte;
            wchar_t actualUser[512]{};
            // MS-DTYP SID string syntax: decimal below 2^32, otherwise 12 hex digits.
            const int formatted = authorityValue < 0x100000000ULL
                ? swprintf_s(actualUser, L"\\REGISTRY\\USER\\S-1-%lu", static_cast<DWORD>(authorityValue))
                : swprintf_s(actualUser, L"\\REGISTRY\\USER\\S-1-0x%012llx", authorityValue);
            if (formatted < 0) return false;
            const BYTE count = *GetSidSubAuthorityCount(sid);
            for (BYTE i = 0; i < count; ++i) {
                const size_t used = wcslen(actualUser);
                if (swprintf_s(actualUser + used, 512 - used, L"-%lu", *GetSidSubAuthority(sid, i)) < 0)
                    return false;
            }
            if (_wcsicmp(actualUser, userKeyPath) != 0
                || swprintf_s(storePrefix, L"%s\\Software\\MivubiEducationPilot", userKeyPath) < 0)
                return false;
        } else if (swprintf_s(storePrefix, L"%s\\Software\\%s_Stores", userKeyPath, logical) < 0)
            return false;
        if (swprintf_s(expected, L"%s\\%s", storePrefix, leaf) < 0
            || !keyName(config.inheritedRegistry, actual) || _wcsicmp(actual, expected) != 0)
            return false;
        PUBLIC_OBJECT_BASIC_INFORMATION info{};
        ULONG returned = 0;
        if (queryObject(config.inheritedRegistry, 0, &info, sizeof(info), &returned) < 0
            || (info.GrantedAccess & KEY_ALL_ACCESS) != KEY_ALL_ACCESS) return false;
        memcpy(storeOwn, expected, (wcslen(expected) + 1) * sizeof(wchar_t));
        privateHive.store(config.inheritedRegistry);
    }
    return true; // Handle/name metadata only; no load/create/profile/COM setup under loader lock.
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
    if (predefinedMapped) return TRUE; // Owned main thread only; not concurrent enrollment.
    FILE_ID_INFO id{};
    if (!directoryInfo(fileOwnRoot, id) || !sameFileId(id, ownDirectoryId)) return FALSE;
    HKEY hive = privateHive.load(), root = nullptr;
    const bool loadedHere = !hive;
    if (loadedHere) {
        wchar_t path[512]{};
        if (swprintf_s(path, L"%s\\Registry\\HKCU.dat", ownDos + 4) < 0) return FALSE;
        const DWORD attributes = GetFileAttributesW(path);
        if (attributes != INVALID_FILE_ATTRIBUTES
            && (attributes & (FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DIRECTORY))) return FALSE;
        if (RegLoadAppKeyW(path, &hive, KEY_ALL_ACCESS, REG_PROCESS_APPKEY, 0) != ERROR_SUCCESS) return FALSE;
    }
    LSTATUS status = RegOpenKeyExW(hive, L"", 0, KEY_ALL_ACCESS, &root);
    if (status == ERROR_SUCCESS) status = RegOverridePredefKey(HKEY_CURRENT_USER, root);
    if (root) RegCloseKey(root);
    if (status != ERROR_SUCCESS) { if (loadedHere) RegCloseKey(hive); return FALSE; }
    privateHive.store(hive);
    predefinedMapped = true;
    return TRUE;
}
