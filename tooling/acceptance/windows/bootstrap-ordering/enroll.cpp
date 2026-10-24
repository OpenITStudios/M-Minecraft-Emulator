#include <windows.h>
#include <detours.h>
#include <string>
#include <stdexcept>
#include <cstring>
#include "receipt.h"
#include "education-config.h"
#include "startup-observation.h"

// Private development adapter. Caller owns the original suspended process and pinned image.
// No target-path/PID reopening, resume, helper, discovery or product command.
namespace {
HMODULE module = nullptr;
void require(bool value) { if (!value) throw std::runtime_error("bootstrap enrollment refused"); }
struct File {
    HANDLE value = INVALID_HANDLE_VALUE;
    explicit File(const std::wstring& path) : value(CreateFileW(path.c_str(), GENERIC_READ,
        FILE_SHARE_READ, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr)) {
        require(value != INVALID_HANDLE_VALUE);
    }
    ~File() { if (value != INVALID_HANDLE_VALUE) CloseHandle(value); }
};
struct Key {
    HKEY value = nullptr;
    ~Key() { if (value) RegCloseKey(value); }
};
struct View {
    void* value = nullptr;
    ~View() { if (value) UnmapViewOfFile(value); }
};
std::wstring sibling(const wchar_t* leaf) {
    wchar_t path[32768]{};
    const DWORD size = GetModuleFileNameW(module, path, 32768);
    require(size && size < 32768);
    const std::wstring name(path, size);
    const auto separator = name.find_last_of(L"\\/");
    require(separator != std::wstring::npos);
    return name.substr(0, separator + 1) + leaf;
}
FILE_ID_INFO fileId(HANDLE file) {
    FILE_ATTRIBUTE_TAG_INFO tag{};
    FILE_ID_INFO id{};
    require(GetFileInformationByHandleEx(file, FileAttributeTagInfo, &tag, sizeof(tag))
        && !(tag.FileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT))
        && GetFileInformationByHandleEx(file, FileIdInfo, &id, sizeof(id)));
    return id;
}
bool sameId(const FILE_ID_INFO& a, const FILE_ID_INFO& b) {
    return a.VolumeSerialNumber == b.VolumeSerialNumber
        && memcmp(a.FileId.Identifier, b.FileId.Identifier, 16) == 0;
}
std::wstring keyName(HKEY key) {
    using Query = NTSTATUS (NTAPI*)(HANDLE, ULONG, PVOID, ULONG, PULONG);
    const auto query = reinterpret_cast<Query>(GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "NtQueryKey"));
    struct Name { ULONG bytes; wchar_t text[1024]; } name{};
    ULONG returned = 0;
    require(query && query(key, 3, &name, sizeof(name), &returned) == 0
        && name.bytes > 0 && name.bytes <= sizeof(name.text) && name.bytes % sizeof(wchar_t) == 0);
    return std::wstring(name.text, name.bytes / sizeof(wchar_t));
}
void newKey(HKEY root, const std::wstring& path, Key& result) {
    DWORD disposition = 0;
    require(RegCreateKeyExW(root, path.c_str(), 0, nullptr, REG_OPTION_NON_VOLATILE,
        KEY_ALL_ACCESS, nullptr, &result.value, &disposition) == ERROR_SUCCESS
        && disposition == REG_CREATED_NEW_KEY);
}
void marker(HKEY key, DWORD value) {
    require(RegSetValueExW(key, L"Marker", 0, REG_DWORD,
        reinterpret_cast<const BYTE*>(&value), sizeof(value)) == ERROR_SUCCESS);
}
education_bootstrap::Config validate(HANDLE process, HANDLE thread, DWORD pid, ULONGLONG birth,
    HANDLE configuration, HANDLE expectedImage) {
    FILETIME created{}, exited{}, kernel{}, user{};
    DWORD session = 0, currentSession = 0;
    USHORT machine = 0, native = 0;
    require(pid != GetCurrentProcessId() && GetProcessId(process) == pid && GetProcessIdOfThread(thread) == pid
        && WaitForSingleObject(process, 0) == WAIT_TIMEOUT
        && GetProcessTimes(process, &created, &exited, &kernel, &user)
        && ((static_cast<ULONGLONG>(created.dwHighDateTime) << 32) | created.dwLowDateTime) == birth
        && ProcessIdToSessionId(pid, &session) && ProcessIdToSessionId(GetCurrentProcessId(), &currentSession)
        && session && session == currentSession
        && IsWow64Process2(process, &machine, &native)
        && machine == IMAGE_FILE_MACHINE_UNKNOWN && native == IMAGE_FILE_MACHINE_AMD64);
    wchar_t actual[32768]{};
    DWORD size = 32768;
    require(QueryFullProcessImageNameW(process, 0, actual, &size));
    File target(std::wstring(actual, size));
    require(sameId(fileId(expectedImage), fileId(target.value)));
    View writeAttempt;
    writeAttempt.value = MapViewOfFile(configuration, FILE_MAP_WRITE, 0, 0, sizeof(education_bootstrap::Config));
    require(!writeAttempt.value);
    View configView;
    configView.value = MapViewOfFile(configuration, FILE_MAP_READ, 0, 0, sizeof(education_bootstrap::Config));
    require(configView.value != nullptr);
    const auto config = *static_cast<const education_bootstrap::Config*>(configView.value);
    require(config.magic == education_bootstrap::Magic && config.version == education_bootstrap::Version
        && config.bytes == sizeof(config) && config.scope == education_bootstrap::StorageAndRegistryOnly
        && !config.reserved && config.slot >= 1 && config.slot <= 2 && config.pid == pid
        && config.session == session && config.created.dwHighDateTime == created.dwHighDateTime
        && config.created.dwLowDateTime == created.dwLowDateTime && config.userKeyPath[511] == 0);
    Key backing;
    require(DuplicateHandle(process, config.registryRoot, GetCurrentProcess(),
        reinterpret_cast<HANDLE*>(&backing.value), 0, FALSE, DUPLICATE_SAME_ACCESS));
    Key userKey;
    require(RegOpenCurrentUser(KEY_READ, &userKey.value) == ERROR_SUCCESS
        && keyName(userKey.value) == config.userKeyPath);
    wchar_t leaf[64]{};
    require(probe::registryStoreLeaf(config.privateId, config.slot, leaf)
        && _wcsicmp(keyName(backing.value).c_str(),
            (std::wstring(config.userKeyPath) + L"\\Software\\MivubiEducationPilot\\" + leaf).c_str()) == 0);
    return config;
}
void insert(HANDLE process) {
    File compatibility(sibling(L"bootstrap-compat.dll"));
    fileId(compatibility.value);
    std::string import;
    for (const wchar_t c : sibling(L"bootstrap-compat.dll")) {
        require(c > 0 && c <= 0x7f);
        import.push_back(static_cast<char>(c));
    }
    LPCSTR imports[]{import.c_str()};
    require(DetourUpdateProcessWithDll(process, imports, 1) != FALSE);
}
DWORD prepare(HANDLE process, HANDLE thread, DWORD pid, ULONGLONG birth,
    HANDLE configuration, HANDLE proof, bool expectEnrolled) {
    try {
        File expected(sibling(L"bootstrap-ordering.exe"));
        const auto config = validate(process, thread, pid, birth, configuration, expected.value);
        Key backing;
        require(DuplicateHandle(process, config.registryRoot, GetCurrentProcess(),
            reinterpret_cast<HANDLE*>(&backing.value), 0, FALSE, DUPLICATE_SAME_ACCESS));
        View proofView;
        proofView.value = MapViewOfFile(proof, FILE_MAP_READ | FILE_MAP_WRITE, 0, 0, sizeof(probe::Receipt));
        require(proofView.value != nullptr);
        const auto name = L"Local\\EducationEntry_" + std::to_wstring(pid) + L"_" + std::to_wstring(birth);
        require(name.size() < 128);
        const auto branch = L"Software\\" + name.substr(6);
        Key host, hostOnly, own;
        newKey(HKEY_CURRENT_USER, branch, host);
        newKey(host.value, L"HostOnly", hostOnly);
        newKey(backing.value, branch, own);
        marker(host.value, 0x484f5354);
        marker(own.value, config.slot * 100);
        HANDLE cached = nullptr;
        require(DuplicateHandle(GetCurrentProcess(), host.value, process, &cached,
            0, FALSE, DUPLICATE_SAME_ACCESS));
        auto& receipt = *static_cast<probe::Receipt*>(proofView.value);
        ZeroMemory(&receipt, sizeof(receipt));
        receipt.magic = probe::Magic;
        receipt.version = probe::Version;
        receipt.pid = pid;
        receipt.created = config.created;
        receipt.mode = probe::ConfiguredStorage;
        receipt.slot = config.slot;
        receipt.registryBacking = 1;
        receipt.registryBaseline = expectEnrolled ? 0 : 1;
        receipt.inheritedRegistry = config.registryRoot;
        receipt.cachedHostKey = reinterpret_cast<HKEY>(cached);
        receipt.fileLogicalRoot = config.logicalRoot;
        receipt.fileOwnRoot = config.privateRoot;
        memcpy(receipt.registryUserPath, config.userKeyPath, sizeof(receipt.registryUserPath));
        require(wcscpy_s(receipt.logical, name.c_str()) == 0);
        // Caller alone owns/resumes/cancels the original suspended child.
        return ERROR_SUCCESS;
    } catch (...) { return ERROR_INVALID_DATA; }
}
}
extern "C" __declspec(dllexport) DWORD WINAPI PrepareOwnedFixture(
    HANDLE process, HANDLE thread, DWORD pid, ULONGLONG birth, HANDLE configuration, HANDLE proof) {
    return prepare(process, thread, pid, birth, configuration, proof, false);
}
extern "C" __declspec(dllexport) DWORD WINAPI PrepareOwnedFixtureForEnrollment(
    HANDLE process, HANDLE thread, DWORD pid, ULONGLONG birth, HANDLE configuration, HANDLE proof) {
    return prepare(process, thread, pid, birth, configuration, proof, true);
}
// ABI v2: original process/thread, PID/birth, read-only parent config, pinned parent image.
// No writable receipt or fixture-side registry setup is consumed by enrollment.
extern "C" __declspec(dllexport) DWORD WINAPI EnrollPreparedEducationV2(
    HANDLE process, HANDLE thread, DWORD pid, ULONGLONG birth, HANDLE configuration, HANDLE expectedImage, HANDLE observation) {
    try {
        const auto config = validate(process, thread, pid, birth, configuration, expectedImage);
        if (observation) require(startup_observation::initialize(observation, config));
        insert(process);
        return ERROR_SUCCESS;
    } catch (...) { return ERROR_INVALID_DATA; }
}
BOOL WINAPI DllMain(HINSTANCE instance, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) module = instance;
    return TRUE;
}
