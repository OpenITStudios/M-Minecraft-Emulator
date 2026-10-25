#include "receipt.h"
#include "education-config.h"
#include "startup-observation.h"
#include "presentation-enrollment.h"
#include "dxgi-pacing.h"
#include <d3d12.h>
#include <dxgi1_6.h>
#include <wrl/client.h>
#include <detours.h>
#include <cstdio>
#include <cwchar>
#include <stdexcept>
#include <string>
#include <vector>
#include <set>
#include <cstring>
#include <winioctl.h>
#include <shlobj.h>
#include <knownfolders.h>
#include "owned-helper-handoff.h"

extern "C" __declspec(dllimport) int DependencyCanary();

void NTAPI BootstrapTls(PVOID, DWORD reason, PVOID) {
    if (reason == DLL_PROCESS_ATTACH) {
        probe::canary(probe::Tls);
        probe::registryCanary(probe::Tls);
    }
}
#pragma section(".CRT$XLB", long, read)
extern "C" {
__declspec(allocate(".CRT$XLB")) PIMAGE_TLS_CALLBACK bootstrap_tls_callback = BootstrapTls;
}
#pragma comment(linker, "/INCLUDE:_tls_used")
#pragma comment(linker, "/INCLUDE:bootstrap_tls_callback")

namespace {
bool cleanupFailed = false;
unsigned lastFailureLine = 0;
bool configuredDiagnosticChild = false;
const wchar_t* presentationControl = nullptr;
DWORD policyControlRound = 0;
void requireAt(bool condition, unsigned line) {
    if (!condition) {
        lastFailureLine = line;
        std::fprintf(stderr, "{\"controlFailureLine\":%u,\"win32\":%lu}\n", line, GetLastError());
        throw std::runtime_error("self-test operation failed");
    }
}
#define require(...) requireAt((__VA_ARGS__), __LINE__)
struct Handle {
    HANDLE value = nullptr;
    explicit Handle(HANDLE handle = nullptr) : value(handle) {}
    Handle(const Handle&) = delete;
    Handle& operator=(const Handle&) = delete;
    ~Handle() { if (value && value != INVALID_HANDLE_VALUE) CloseHandle(value); }
};
struct Mapping {
    probe::Receipt* value;
    explicit Mapping(HANDLE handle) : value(static_cast<probe::Receipt*>(
        MapViewOfFile(handle, FILE_MAP_READ | FILE_MAP_WRITE, 0, 0, sizeof(probe::Receipt)))) {
        require(value != nullptr);
    }
    ~Mapping() { UnmapViewOfFile(value); }
};
struct Attributes {
    std::vector<BYTE> bytes;
    LPPROC_THREAD_ATTRIBUTE_LIST value = nullptr;
    explicit Attributes(HANDLE* inherited, SIZE_T count = 1) {
        SIZE_T size = 0;
        InitializeProcThreadAttributeList(nullptr, 1, 0, &size);
        require(size != 0);
        bytes.resize(size);
        value = reinterpret_cast<LPPROC_THREAD_ATTRIBUTE_LIST>(bytes.data());
        require(InitializeProcThreadAttributeList(value, 1, 0, &size) != FALSE);
        if (!UpdateProcThreadAttribute(value, 0, PROC_THREAD_ATTRIBUTE_HANDLE_LIST,
                                       inherited, sizeof(HANDLE) * count, nullptr, nullptr)) {
            DeleteProcThreadAttributeList(value);
            value = nullptr;
            require(false);
        }
    }
    ~Attributes() { if (value) DeleteProcThreadAttributeList(value); }
};
struct Child {
    PROCESS_INFORMATION info{};
    Child(const Child&) = delete;
    Child& operator=(const Child&) = delete;
    Child() = default;
    bool stop() {
        if (!info.hProcess) return true;
        DWORD state = WaitForSingleObject(info.hProcess, 0);
        if (state == WAIT_OBJECT_0) return true;
        if (state != WAIT_TIMEOUT) return false;
        if (!TerminateProcess(info.hProcess, 90))
            return WaitForSingleObject(info.hProcess, 0) == WAIT_OBJECT_0; // Exit may win the race.
        return WaitForSingleObject(info.hProcess, 5000) == WAIT_OBJECT_0;
    }
    ~Child() {
        if (!stop()) cleanupFailed = true; // Original handle only; never reopen by PID.
        if (info.hThread) CloseHandle(info.hThread);
        if (info.hProcess) CloseHandle(info.hProcess);
    }
};
std::wstring executable() {
    std::wstring path(32768, L'\0');
    const DWORD length = GetModuleFileNameW(nullptr, path.data(), static_cast<DWORD>(path.size()));
    require(length > 0 && length < path.size());
    path.resize(length);
    return path;
}
std::wstring directoryPath(HANDLE root);
std::vector<wchar_t> environment(HANDLE section, HANDLE appDataRoot = nullptr) {
    wchar_t* original = GetEnvironmentStringsW();
    require(original != nullptr);
    std::vector<wchar_t> result;
    try {
        for (const wchar_t* item = original; *item; item += wcslen(item) + 1) {
            constexpr wchar_t prefix[] = L"MIVUBI_BOOTSTRAP_TEST_SECTION=";
            const bool appData = appDataRoot && (_wcsnicmp(item, L"APPDATA=", 8) == 0
                || _wcsnicmp(item, L"LOCALAPPDATA=", 13) == 0);
            const bool configured = _wcsnicmp(item, education_bootstrap::SectionVariable,
                wcslen(education_bootstrap::SectionVariable)) == 0
                && item[wcslen(education_bootstrap::SectionVariable)] == L'=';
            const bool observed = _wcsnicmp(item, startup_observation::Variable,
                wcslen(startup_observation::Variable)) == 0
                && item[wcslen(startup_observation::Variable)] == L'=';
            const bool presentation = _wcsnicmp(item, presentation_enrollment::Variable,
                wcslen(presentation_enrollment::Variable)) == 0
                && item[wcslen(presentation_enrollment::Variable)] == L'=';
            const bool presentationSection = _wcsnicmp(item, presentation_enrollment::SectionVariable,
                wcslen(presentation_enrollment::SectionVariable)) == 0
                && item[wcslen(presentation_enrollment::SectionVariable)] == L'=';
            if (!configured && !observed && !presentation && !presentationSection && !appData
                && _wcsnicmp(item, prefix, (sizeof(prefix) / sizeof(wchar_t)) - 1) != 0)
                result.insert(result.end(), item, item + wcslen(item) + 1);
        }
        const std::wstring setting = std::wstring(probe::SectionVariable) + L"="
            + std::to_wstring(reinterpret_cast<ULONG_PTR>(section));
        result.insert(result.end(), setting.begin(), setting.end());
        result.push_back(L'\0');
        if (appDataRoot) {
            const auto path = directoryPath(appDataRoot);
            require(path.substr(0, 4) == L"\\\\?\\");
            for (const auto& appSetting : {L"APPDATA=" + path.substr(4) + L"\\Roaming",
                                       L"LOCALAPPDATA=" + path.substr(4) + L"\\Local"}) {
                result.insert(result.end(), appSetting.begin(), appSetting.end());
                result.push_back(L'\0');
            }
        }
        result.push_back(L'\0');
    } catch (...) {
        FreeEnvironmentStringsW(original);
        throw;
    }
    FreeEnvironmentStringsW(original);
    return result;
}
bool run(probe::Mode mode, const wchar_t* name, const std::wstring& self,
         const std::string& compatibility) {
    SECURITY_ATTRIBUTES security{sizeof(SECURITY_ATTRIBUTES), nullptr, TRUE};
    Handle section(CreateFileMappingW(INVALID_HANDLE_VALUE, &security, PAGE_READWRITE,
                                     0, sizeof(probe::Receipt), nullptr));
    require(section.value != nullptr);
    Mapping mapping(section.value);
    ZeroMemory(mapping.value, sizeof(probe::Receipt));
    mapping.value->magic = probe::Magic;
    mapping.value->version = probe::Version;
    mapping.value->mode = mode;
    Attributes attributes(&section.value);
    auto env = environment(section.value);
    STARTUPINFOEXW startup{};
    startup.StartupInfo.cb = sizeof(startup);
    startup.lpAttributeList = attributes.value;
    std::wstring command = L"\"" + self + L"\" --bootstrap-test-child";
    Child child;
    require(CreateProcessW(self.c_str(), command.data(), nullptr, nullptr, TRUE,
        CREATE_SUSPENDED | CREATE_UNICODE_ENVIRONMENT | EXTENDED_STARTUPINFO_PRESENT
            | CREATE_NO_WINDOW,
        env.data(), nullptr, &startup.StartupInfo, &child.info) != FALSE);
    mapping.value->pid = child.info.dwProcessId;
    FILETIME exited{}, kernel{}, user{};
    require(GetProcessTimes(child.info.hProcess, &mapping.value->created, &exited, &kernel, &user)
        != FALSE);
    if (mode != probe::Baseline) {
        const char* dlls[] = {compatibility.c_str()};
        // Update the handle we just created; no Detours cross-bitness/helper fallback.
        require(DetourUpdateProcessWithDll(child.info.hProcess, dlls, 1) != FALSE);
    }
    MemoryBarrier();
    require(ResumeThread(child.info.hThread) == 1);
    const DWORD wait = WaitForSingleObject(child.info.hProcess,
                                           mode == probe::Timeout ? 1500 : 10000);
    const bool timedOut = wait == WAIT_TIMEOUT;
    require(wait == WAIT_OBJECT_0 || timedOut);
    require(child.stop()); // Receipt is inspected only after the exact child has exited.
    DWORD exit = STILL_ACTIVE;
    require(GetExitCodeProcess(child.info.hProcess, &exit) != FALSE && exit != STILL_ACTIVE);
    const LONG count = InterlockedCompareExchange(&mapping.value->count, 0, 0);
    require(count >= 0 && count <= probe::Capacity);
    LONG stages[8]{};
    LONG before = 0;
    for (LONG i = 0; i < count; ++i) {
        const auto event = mapping.value->events[i];
        require(event.stage >= probe::Attach && event.stage <= probe::Main);
        require(event.armed == 0 || event.armed == 1);
        ++stages[event.stage];
        if (event.stage >= probe::Dependency && event.armed == 0) ++before;
    }
    const bool markers = stages[probe::Dependency] == 1 && stages[probe::Tls] == 1
        && stages[probe::Main] == 1;
    bool accepted = false;
    switch (mode) {
    case probe::Baseline:
        accepted = !timedOut && exit == 0 && count == 3 && markers && before == 3
            && stages[probe::Attach] == 0 && mapping.value->armed == 0;
        break;
    case probe::Normal:
        accepted = !timedOut && exit == 0 && count == 5 && markers && before == 0
            && stages[probe::Attach] == 1 && stages[probe::Armed] == 1
            && mapping.value->armed == 1;
        break;
    case probe::FailAttach:
        accepted = !timedOut && exit != 0 && stages[probe::Attach] == 1
            && stages[probe::Rejected] == 1 && stages[probe::Armed] == 0
            && stages[probe::Main] == 0 && stages[probe::Suppressed] == 0
            && stages[probe::Dependency] <= 1 && stages[probe::Tls] <= 1
            && count == 2 + stages[probe::Dependency] + stages[probe::Tls]
            && mapping.value->armed == 0;
        break;
    case probe::MutexCase:
        require(false); // Mutex mode uses the two-phase pair runner, never this path.
        break;
    case probe::Timeout:
        accepted = timedOut && exit == 90 && count == 5 && stages[probe::Attach] == 1
            && stages[probe::Suppressed] == 1
            && stages[probe::Armed] == 0 && markers && before == 3
            && mapping.value->armed == 0;
        break;
    }
    std::printf("{\"case\":\"%ls\",\"selfCheck\":%s,\"timedOut\":%s,"
                "\"canariesBeforeArm\":%ld,\"events\":[",
                name, accepted ? "true" : "false", timedOut ? "true" : "false", before);
    for (LONG i = 0; i < count; ++i) {
        std::printf("%s{\"stage\":%ld,\"armed\":%ld}", i ? "," : "",
                    mapping.value->events[i].stage, mapping.value->events[i].armed);
    }
    std::printf("],\"providerReady\":false,\"minecraft\":\"NOT_EXECUTED\"}\n");
    return accepted;
}
UNICODE_STRING nativeName(std::wstring& text) {
    require(text.size() <= 32767);
    const auto bytes = static_cast<USHORT>(text.size() * sizeof(wchar_t));
    return {bytes, bytes, text.data()};
}
LONG nativeMutexChecks(probe::Receipt& receipt, HANDLE expected) {
    const HMODULE ntdll = GetModuleHandleW(L"ntdll.dll");
    require(ntdll != nullptr);
    const auto create = reinterpret_cast<probe::NativeCreate>(GetProcAddress(ntdll, "NtCreateMutant"));
    const auto open = reinterpret_cast<probe::NativeOpen>(GetProcAddress(ntdll, "NtOpenMutant"));
    const auto directory = reinterpret_cast<probe::NativeDirectory>(GetProcAddress(ntdll, "NtOpenDirectoryObject"));
    const auto compare = reinterpret_cast<probe::CompareHandles>(
        GetProcAddress(GetModuleHandleW(L"kernelbase.dll"), "CompareObjectHandles"));
    require(create && open && directory && compare);
    DWORD session = 0;
    require(ProcessIdToSessionId(GetCurrentProcessId(), &session) != FALSE);
    std::wstring directoryPath = L"\\Sessions\\" + std::to_wstring(session) + L"\\BaseNamedObjects";
    auto directoryName = nativeName(directoryPath);
    OBJECT_ATTRIBUTES rootAttributes{sizeof(OBJECT_ATTRIBUTES), nullptr, &directoryName,
        OBJ_CASE_INSENSITIVE, nullptr, nullptr};
    Handle root;
    require(directory(&root.value, 0x0002, &rootAttributes) == 0 && root.value);
    const std::wstring base = std::wstring(receipt.logical).substr(6);
    std::wstring absolutePath = directoryPath + L"\\" + base;
    auto absoluteName = nativeName(absolutePath);
    OBJECT_ATTRIBUTES attributes{sizeof(OBJECT_ATTRIBUTES), nullptr, &absoluteName,
        OBJ_CASE_INSENSITIVE, nullptr, nullptr};
    constexpr DWORD sentinel = 0x13579bd;
    LONG checks = 0;

    Handle absolute;
    SetLastError(sentinel);
    NTSTATUS status = open(&absolute.value, SYNCHRONIZE, &attributes);
    DWORD error = GetLastError();
    require(status == 0 && error == sentinel && absolute.value
        && compare(absolute.value, expected));
    checks |= 1;

    std::wstring relativePath = base;
    auto relativeName = nativeName(relativePath);
    attributes.RootDirectory = root.value;
    attributes.ObjectName = &relativeName;
    Handle relative;
    SetLastError(sentinel);
    status = open(&relative.value, SYNCHRONIZE, &attributes);
    error = GetLastError();
    require(status == 0 && error == sentinel && relative.value
        && compare(relative.value, expected));
    std::wstring aliasPath = L"Local\\" + base;
    auto aliasName = nativeName(aliasPath);
    attributes.ObjectName = &aliasName;
    Handle alias;
    SetLastError(sentinel);
    status = open(&alias.value, SYNCHRONIZE, &attributes);
    error = GetLastError();
    require(status == 0 && error == sentinel && alias.value
        && compare(alias.value, expected));
    checks |= 2;

    std::wstring countedPath = base + L"_NOT_PART_OF_COUNTED_NAME";
    auto countedName = nativeName(countedPath);
    countedName.Length = countedName.MaximumLength = static_cast<USHORT>(base.size() * sizeof(wchar_t));
    attributes.ObjectName = &countedName;
    Handle counted;
    SetLastError(sentinel);
    status = open(&counted.value, SYNCHRONIZE, &attributes);
    error = GetLastError();
    require(status == 0 && error == sentinel && counted.value
        && compare(counted.value, expected)
        && countedPath == base + L"_NOT_PART_OF_COUNTED_NAME"
        && countedName.Length == base.size() * sizeof(wchar_t)
        && attributes.RootDirectory == root.value && attributes.ObjectName == &countedName);
    checks |= 4;

    std::wstring lowercase = base;
    for (auto& c : lowercase) if (c >= L'A' && c <= L'Z') c = static_cast<wchar_t>(c + L'a' - L'A');
    auto lowercaseName = nativeName(lowercase);
    attributes.ObjectName = &lowercaseName;
    Handle insensitive;
    SetLastError(sentinel);
    status = open(&insensitive.value, SYNCHRONIZE, &attributes);
    error = GetLastError();
    require(status == 0 && error == sentinel && insensitive.value
        && compare(insensitive.value, expected));
    attributes.Attributes = 0;
    Handle sensitive;
    SetLastError(sentinel);
    status = open(&sensitive.value, SYNCHRONIZE, &attributes);
    error = GetLastError();
    require(status == static_cast<NTSTATUS>(0xc0000034UL) && !sensitive.value && error == sentinel);
    checks |= 8;

    attributes.Attributes = OBJ_OPENIF | OBJ_INHERIT | OBJ_CASE_INSENSITIVE;
    attributes.ObjectName = &relativeName;
    Handle existing;
    SetLastError(sentinel);
    status = create(&existing.value, SYNCHRONIZE | MUTEX_MODIFY_STATE, &attributes, FALSE);
    error = GetLastError();
    DWORD flags = 0;
    require(status == static_cast<NTSTATUS>(0x40000000UL) && error == sentinel && existing.value
        && compare(existing.value, expected)
        && GetHandleInformation(existing.value, &flags) && (flags & HANDLE_FLAG_INHERIT));
    checks |= 16;

    Handle unnamed;
    SetLastError(sentinel);
    status = create(&unnamed.value, SYNCHRONIZE | MUTEX_MODIFY_STATE, nullptr, FALSE);
    error = GetLastError();
    require(status == 0 && error == sentinel && unnamed.value
        && !compare(unnamed.value, expected)
        && WaitForSingleObject(unnamed.value, 0) == WAIT_OBJECT_0 && ReleaseMutex(unnamed.value));
    checks |= 32;

    attributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_INHERIT;
    Handle limited;
    SetLastError(sentinel);
    status = open(&limited.value, MUTEX_MODIFY_STATE, &attributes);
    error = GetLastError();
    require(status == 0 && error == sentinel && limited.value
        && GetHandleInformation(limited.value, &flags) && (flags & HANDLE_FLAG_INHERIT));
    const DWORD limitedWait = WaitForSingleObject(limited.value, 0);
    const DWORD limitedError = GetLastError();
    require(limitedWait == WAIT_FAILED && limitedError == ERROR_ACCESS_DENIED);
    std::wstring missingPath = base + L"_missing";
    auto missingName = nativeName(missingPath);
    attributes.ObjectName = &missingName;
    Handle missing;
    SetLastError(sentinel);
    status = open(&missing.value, SYNCHRONIZE, &attributes);
    error = GetLastError();
    require(status == static_cast<NTSTATUS>(0xc0000034UL) && !missing.value && error == sentinel);
    checks |= 64;

    std::wstring otherPath = L"\\KnownDlls";
    auto otherName = nativeName(otherPath);
    rootAttributes.ObjectName = &otherName;
    Handle otherRoot;
    require(directory(&otherRoot.value, 0x0002, &rootAttributes) == 0 && otherRoot.value);
    attributes.RootDirectory = otherRoot.value;
    attributes.ObjectName = &relativeName;
    Handle wrongRoot;
    SetLastError(sentinel);
    status = open(&wrongRoot.value, SYNCHRONIZE, &attributes);
    error = GetLastError();
    require(status == static_cast<NTSTATUS>(0xc0000034UL) && !wrongRoot.value && error == sentinel);
    checks |= 128;

    UNICODE_STRING badName{2, 2, reinterpret_cast<PWSTR>(static_cast<ULONG_PTR>(1))};
    attributes.RootDirectory = root.value;
    attributes.ObjectName = &badName;
    Handle malformed;
    SetLastError(sentinel);
    status = open(&malformed.value, SYNCHRONIZE, &attributes);
    error = GetLastError();
    require(status == static_cast<NTSTATUS>(0xc0000005UL) && !malformed.value && error == sentinel);
    checks |= 256;
    return checks;
}


std::wstring directoryPath(HANDLE root) {
    wchar_t buffer[512]{};
    const DWORD length = GetFinalPathNameByHandleW(root, buffer, 512, VOLUME_NAME_DOS);
    require(length > 0 && length < 512);
    return std::wstring(buffer, length);
}
void expectBytes(HANDLE file, const std::string& expected) {
    require(file && file != INVALID_HANDLE_VALUE);
    LARGE_INTEGER zero{};
    require(SetFilePointerEx(file, zero, nullptr, FILE_BEGIN) != FALSE);
    char buffer[32]{};
    DWORD read = 0;
    require(ReadFile(file, buffer, sizeof(buffer), &read, nullptr) != FALSE
        && read == expected.size() && std::string(buffer, read) == expected);
}
void expectFile(const std::wstring& path, const std::string& expected) {
    Handle file(CreateFileW(path.c_str(), GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE,
        nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr));
    expectBytes(file.value, expected);
}
void writeBytes(HANDLE file, const std::string& text) {
    LARGE_INTEGER zero{};
    require(SetFilePointerEx(file, zero, nullptr, FILE_BEGIN) != FALSE);
    DWORD written = 0;
    require(WriteFile(file, text.data(), static_cast<DWORD>(text.size()), &written, nullptr)
        && written == text.size() && SetEndOfFile(file) && FlushFileBuffers(file));
}
std::string slotMarker(const probe::Receipt& receipt, DWORD round) {
    return std::string(receipt.slot == 1 ? "A" : "B") + std::to_string(round + 1);
}

using QueryDirectory = NTSTATUS (NTAPI*)(HANDLE, HANDLE, PIO_APC_ROUTINE, PVOID,
    PIO_STATUS_BLOCK, PVOID, ULONG, FILE_INFORMATION_CLASS, BOOLEAN, PUNICODE_STRING, BOOLEAN);
std::set<std::wstring> nativeEntries(HANDLE root, bool single, const wchar_t* filter = nullptr) {
    const auto query = reinterpret_cast<QueryDirectory>(
        GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "NtQueryDirectoryFile"));
    require(query != nullptr);
    std::wstring pattern = filter ? filter : L"";
    auto name = nativeName(pattern);
    std::set<std::wstring> result;
    for (unsigned scan = 0; scan < 64; ++scan) {
        alignas(8) BYTE buffer[2048]{};
        IO_STATUS_BLOCK io{};
        const NTSTATUS status = query(root, nullptr, nullptr, nullptr, &io, buffer,
            sizeof(buffer), static_cast<FILE_INFORMATION_CLASS>(12), single ? TRUE : FALSE,
            scan == 0 && filter ? &name : nullptr, scan == 0 ? TRUE : FALSE);
        if (status == static_cast<NTSTATUS>(0x80000006UL)) return result; // NO_MORE_FILES
        require(status == 0 && io.Information >= 12 && io.Information <= sizeof(buffer));
        SIZE_T offset = 0;
        unsigned entries = 0;
        for (;;) {
            require(offset + 12 <= io.Information);
            ULONG next = 0, bytes = 0;
            memcpy(&next, buffer + offset, sizeof(next));
            memcpy(&bytes, buffer + offset + 8, sizeof(bytes));
            require(bytes % sizeof(wchar_t) == 0 && bytes <= io.Information - offset - 12);
            const std::wstring entry(reinterpret_cast<wchar_t*>(buffer + offset + 12), bytes / sizeof(wchar_t));
            if (entry != L"." && entry != L"..") require(result.insert(entry).second);
            ++entries;
            if (!next) break;
            require(next >= 12 + bytes && next % 4 == 0 && next < io.Information - offset);
            offset += next;
        }
        require(!single || entries == 1);
    }
    require(false);
    return result;
}
void directoryChecks(probe::Receipt& receipt, LONG& checks) {
    const auto open = reinterpret_cast<probe::NativeFileOpen>(
        GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "NtOpenFile"));
    const auto create = reinterpret_cast<probe::NativeFileCreate>(
        GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "NtCreateFile"));
    require(open && create);
    const auto logicalPath = directoryPath(receipt.fileLogicalRoot);
    Handle root(CreateFileW(logicalPath.c_str(), FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
        FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, nullptr));
    require(root.value != INVALID_HANDLE_VALUE);
    FILE_ID_INFO actual{}, expected{};
    require(GetFileInformationByHandleEx(root.value, FileIdInfo, &actual, sizeof(actual))
        && GetFileInformationByHandleEx(receipt.fileOwnRoot, FileIdInfo, &expected, sizeof(expected))
        && actual.VolumeSerialNumber == expected.VolumeSerialNumber
        && memcmp(actual.FileId.Identifier, expected.FileId.Identifier, sizeof(actual.FileId.Identifier)) == 0);
    checks |= 128;

    std::wstring leaf = receipt.slot == 1 ? L"slot-A.bin" : L"slot-B.bin";
    auto name = nativeName(leaf);
    OBJECT_ATTRIBUTES attributes{sizeof(OBJECT_ATTRIBUTES), root.value, &name,
        OBJ_CASE_INSENSITIVE, nullptr, nullptr};
    IO_STATUS_BLOCK io{};
    constexpr ULONG options = 0x20 | 0x40;
    if (receipt.fileRound == 1) {
        Handle previous;
        require(open(&previous.value, FILE_GENERIC_READ, &attributes, &io,
            FILE_SHARE_READ | FILE_SHARE_WRITE, options) == 0);
        expectBytes(previous.value, slotMarker(receipt, 0));
    }
    Handle own;
    constexpr DWORD sentinel = 0x13579bd;
    SetLastError(sentinel);
    const NTSTATUS status = create(&own.value, FILE_GENERIC_READ | FILE_GENERIC_WRITE,
        &attributes, &io, nullptr, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ | FILE_SHARE_WRITE,
        receipt.fileRound == 0 ? 2 : 1, options, nullptr, 0);
    const DWORD error = GetLastError();
    require(status == 0 && error == sentinel && io.Information == (receipt.fileRound == 0 ? 2u : 1u)
        && attributes.RootDirectory == root.value && attributes.ObjectName == &name);
    writeBytes(own.value, slotMarker(receipt, receipt.fileRound));
    std::set<std::wstring> expectedEntries{L"state.bin", L"nested", L"Roaming", L"Local", L"Registry", leaf};
    if (receipt.fileRound) expectedEntries.insert(L"a");
    require(nativeEntries(root.value, false) == expectedEntries);
    checks |= 256;
    require(nativeEntries(root.value, true) == expectedEntries); // Restart on the same handle.
    checks |= 512;

    Handle filtered(CreateFileW(logicalPath.c_str(), FILE_LIST_DIRECTORY | SYNCHRONIZE,
        FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, nullptr));
    require(filtered.value != INVALID_HANDLE_VALUE
        && nativeEntries(filtered.value, true, L"slot-*.bin") == std::set<std::wstring>{leaf});
    WIN32_FIND_DATAW found{};
    HANDLE search = FindFirstFileW((logicalPath + L"\\*").c_str(), &found);
    require(search != INVALID_HANDLE_VALUE);
    std::set<std::wstring> win32;
    bool valid = true;
    do {
        const std::wstring entry = found.cFileName;
        if (entry != L"." && entry != L"..") valid = win32.insert(entry).second && valid;
    } while (FindNextFileW(search, &found));
    const DWORD findError = GetLastError();
    const BOOL closed = FindClose(search);
    require(valid && findError == ERROR_NO_MORE_FILES && closed && win32 == expectedEntries);
    checks |= 1024;

    std::wstring peer = receipt.slot == 1 ? L"slot-B.bin" : L"slot-A.bin";
    auto peerName = nativeName(peer);
    attributes.ObjectName = &peerName;
    Handle absent;
    require(open(&absent.value, FILE_GENERIC_READ, &attributes, &io,
        FILE_SHARE_READ | FILE_SHARE_WRITE, options) == static_cast<NTSTATUS>(0xc0000034UL)
        && !absent.value);
    expectFile(logicalPath + L"\\" + leaf, slotMarker(receipt, receipt.fileRound));
    for (auto bad : {L"..\\outside.bin", L"state.bin:stream", L"sub\\..\\file.bin", L"trailing."}) {
        std::wstring invalid = bad;
        auto badName = nativeName(invalid);
        attributes.ObjectName = &badName;
        Handle denied;
        require(open(&denied.value, FILE_GENERIC_READ, &attributes, &io,
            FILE_SHARE_READ | FILE_SHARE_WRITE, options) == static_cast<NTSTATUS>(0xc00000bbUL)
            && !denied.value);
    }
    Handle hidden(CreateFileW((logicalPath + L"\\host-only.bin").c_str(), GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr));
    require(hidden.value == INVALID_HANDLE_VALUE && GetLastError() == ERROR_FILE_NOT_FOUND);
    checks |= 2048;
}


void treeChecks(probe::Receipt& receipt, LONG& checks) {
    const auto logical = directoryPath(receipt.fileLogicalRoot);
    const std::wstring deep = logical + L"\\nested\\level\\deep";
    for (const auto& suffix : {L"\\nested\\level", L"\\nested\\level\\deep"}) {
        const BOOL created = CreateDirectoryW((logical + suffix).c_str(), nullptr);
        const DWORD error = GetLastError();
        require(created || (receipt.fileRound == 1 && error == ERROR_ALREADY_EXISTS));
    }
    checks |= 4096;
    const auto open = reinterpret_cast<probe::NativeFileOpen>(
        GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "NtOpenFile"));
    const auto create = reinterpret_cast<probe::NativeFileCreate>(
        GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "NtCreateFile"));
    require(open && create);
    Handle directory(CreateFileW(deep.c_str(), FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
        FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, nullptr));
    require(directory.value != INVALID_HANDLE_VALUE);
    std::wstring state = L"state.bin";
    auto stateName = nativeName(state);
    OBJECT_ATTRIBUTES attributes{sizeof(OBJECT_ATTRIBUTES), directory.value, &stateName,
        OBJ_CASE_INSENSITIVE, nullptr, nullptr};
    IO_STATUS_BLOCK io{};
    constexpr ULONG options = 0x20 | 0x40;
    if (receipt.fileRound == 0) {
        Handle missing;
        require(open(&missing.value, FILE_GENERIC_READ, &attributes, &io,
            FILE_SHARE_READ | FILE_SHARE_WRITE, options) == static_cast<NTSTATUS>(0xc0000034UL)
            && !missing.value); // Host has a nested state.bin; private never falls back.
    } else {
        expectFile(deep + L"\\state.bin", slotMarker(receipt, 0));
    }
    Handle file;
    constexpr DWORD sentinel = 0x13579bd;
    SetLastError(sentinel);
    const NTSTATUS status = create(&file.value, FILE_GENERIC_READ | FILE_GENERIC_WRITE,
        &attributes, &io, nullptr, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ | FILE_SHARE_WRITE,
        receipt.fileRound == 0 ? 2 : 1, options, nullptr, 0);
    const DWORD error = GetLastError();
    require(status == 0 && error == sentinel && io.Information == (receipt.fileRound == 0 ? 2u : 1u)
        && attributes.RootDirectory == directory.value && attributes.ObjectName == &stateName
        && attributes.Attributes == OBJ_CASE_INSENSITIVE);
    writeBytes(file.value, slotMarker(receipt, receipt.fileRound));
    expectFile(deep + L"\\state.bin", slotMarker(receipt, receipt.fileRound));
    const auto ownLeaf = receipt.slot == 1 ? L"slot-A.bin" : L"slot-B.bin";
    if (receipt.fileRound == 1) expectFile(deep + L"\\" + ownLeaf, slotMarker(receipt, 0));
    Handle own(CreateFileW((deep + L"\\" + ownLeaf).c_str(), GENERIC_READ | GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, receipt.fileRound == 0 ? CREATE_NEW : OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL, nullptr));
    require(own.value != INVALID_HANDLE_VALUE);
    writeBytes(own.value, slotMarker(receipt, receipt.fileRound));
    checks |= 8192;

    require(nativeEntries(directory.value, false) == std::set<std::wstring>{L"state.bin", ownLeaf}
        && nativeEntries(directory.value, true, nullptr) == std::set<std::wstring>{L"state.bin", ownLeaf});
    std::wstring peer = receipt.slot == 1 ? L"slot-B.bin" : L"slot-A.bin";
    auto peerName = nativeName(peer);
    attributes.ObjectName = &peerName;
    Handle absent;
    require(open(&absent.value, FILE_GENERIC_READ, &attributes, &io,
        FILE_SHARE_READ | FILE_SHARE_WRITE, options) == static_cast<NTSTATUS>(0xc0000034UL)
        && !absent.value);
    checks |= 16384;

    for (const auto& bad : {L"\\nested\\escape", L"\\nested\\escape\\state.bin"}) {
        Handle escaped(CreateFileW((logical + bad).c_str(), GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, nullptr));
        require(escaped.value == INVALID_HANDLE_VALUE);
    }
    Handle overwrite(CreateFileW((logical + L"\\nested\\escape\\state.bin").c_str(), GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr));
    require(overwrite.value == INVALID_HANDLE_VALUE);
    for (const auto& bad : {L"..\\..\\..\\guard\\state.bin", L"state.bin:stream", L"state.bin."}) {
        std::wstring path = bad;
        auto badName = nativeName(path);
        attributes.ObjectName = &badName;
        Handle denied;
        require(open(&denied.value, FILE_GENERIC_READ, &attributes, &io,
            FILE_SHARE_READ | FILE_SHARE_WRITE, options) == static_cast<NTSTATUS>(0xc00000bbUL)
            && !denied.value);
    }
    checks |= 32768;
}


struct ComApartment {
    ComApartment() { require(SUCCEEDED(CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED))); }
    ~ComApartment() { CoUninitialize(); }
};
std::wstring childEnvironment(const wchar_t* name) {
    wchar_t value[512]{};
    const DWORD length = GetEnvironmentVariableW(name, value, 512);
    require(length > 0 && length < 512);
    return std::wstring(value, length);
}
std::wstring knownPath(REFKNOWNFOLDERID id, DWORD flags = 0) {
    PWSTR result = nullptr;
    constexpr DWORD sentinel = 0x13579bd;
    SetLastError(sentinel);
    const HRESULT status = SHGetKnownFolderPath(id, flags, nullptr, &result);
    const DWORD error = GetLastError();
    const bool valid = status == S_OK && error == sentinel && result;
    std::wstring path = valid ? result : L"";
    CoTaskMemFree(result);
    require(valid);
    return path;
}
void deepStorageChecks(probe::Receipt& receipt) {
    const auto root = directoryPath(receipt.fileLogicalRoot);
    std::wstring relative;
    for (unsigned parent = 1; parent <= 255; ++parent) {
        if (!relative.empty()) relative += L"\\";
        relative += L"a";
        const auto directory = root + L"\\" + relative;
        if (receipt.fileRound == 0) require(CreateDirectoryW(directory.c_str(), nullptr));
        else {
            const DWORD attributes = GetFileAttributesW(directory.c_str());
            require(attributes != INVALID_FILE_ATTRIBUTES && (attributes & FILE_ATTRIBUTE_DIRECTORY));
        }
        if (parent != 16 && parent != 255) continue;
        const auto path = directory + L"\\x";
        if (receipt.fileRound) expectFile(path, slotMarker(receipt, 0));
        {
            Handle file(CreateFileW(path.c_str(), GENERIC_READ | GENERIC_WRITE | DELETE,
                FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr,
                receipt.fileRound ? OPEN_EXISTING : CREATE_NEW, FILE_ATTRIBUTE_NORMAL, nullptr));
            require(file.value != INVALID_HANDLE_VALUE);
            writeBytes(file.value, slotMarker(receipt, receipt.fileRound));
        }
        WIN32_FILE_ATTRIBUTE_DATA data{};
        require(GetFileAttributesExW(path.c_str(), GetFileExInfoStandard, &data)
            && data.nFileSizeLow == slotMarker(receipt, receipt.fileRound).size());
        auto text = relative + L"\\x";
        auto name = nativeName(text);
        OBJECT_ATTRIBUTES attributes{sizeof(OBJECT_ATTRIBUTES), receipt.fileLogicalRoot, &name,
            OBJ_CASE_INSENSITIVE, nullptr, nullptr};
        IO_STATUS_BLOCK io{};
        Handle file;
        const auto open = reinterpret_cast<probe::NativeFileOpen>(
            GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "NtOpenFile"));
        require(open && open(&file.value, FILE_GENERIC_READ, &attributes, &io,
            FILE_SHARE_READ | FILE_SHARE_WRITE, FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT) == 0);
        expectBytes(file.value, slotMarker(receipt, receipt.fileRound));
        require(CloseHandle(file.value)); file.value = nullptr;
        {
            Handle parentDirectory(CreateFileW(directory.c_str(), FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES,
                FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, nullptr));
            require(parentDirectory.value != INVALID_HANDLE_VALUE);
            std::wstring leaf = L"x";
            auto leafName = nativeName(leaf);
            OBJECT_ATTRIBUTES relativeAttributes{sizeof(OBJECT_ATTRIBUTES), parentDirectory.value, &leafName,
                OBJ_CASE_INSENSITIVE, nullptr, nullptr};
            Handle reopened;
            require(open(&reopened.value, FILE_GENERIC_READ, &relativeAttributes, &io,
                FILE_SHARE_READ | FILE_SHARE_WRITE, FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT) == 0);
            expectBytes(reopened.value, slotMarker(receipt, receipt.fileRound));
        }
        {
            // Exercise the rename walker and long canonical source at both depths.
            Handle source(CreateFileW(path.c_str(), GENERIC_READ | GENERIC_WRITE | DELETE,
                FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr));
            require(source.value != INVALID_HANDLE_VALUE);
            struct Rename { ULONG flags; HANDLE root; ULONG bytes; wchar_t name[512]; } info{};
            info.root = receipt.fileLogicalRoot;
            info.bytes = static_cast<ULONG>(text.size() * sizeof(wchar_t));
            memcpy(info.name, text.data(), info.bytes);
            info.name[text.size() - 1] = L'y';
            const auto rename = reinterpret_cast<NTSTATUS (NTAPI*)(HANDLE, PIO_STATUS_BLOCK,
                PVOID, ULONG, FILE_INFORMATION_CLASS)>(GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "NtSetInformationFile"));
            require(rename);
            const NTSTATUS renamed = rename(source.value, &io, &info,
                static_cast<ULONG>(offsetof(Rename, name) + info.bytes), static_cast<FILE_INFORMATION_CLASS>(10));
            require(renamed == 0);
            info.name[text.size() - 1] = L'x';
            require(rename(source.value, &io, &info,
                static_cast<ULONG>(offsetof(Rename, name) + info.bytes), static_cast<FILE_INFORMATION_CLASS>(10)) == 0);
        }
        if (parent == 255) {
            require(text.size() == 511);
            text += L"xx"; // Still refuse the existing total-length boundary.
            name = nativeName(text);
            Handle denied;
            require(open(&denied.value, FILE_GENERIC_READ, &attributes, &io,
                FILE_SHARE_READ | FILE_SHARE_WRITE, FILE_NON_DIRECTORY_FILE) == static_cast<NTSTATUS>(0xc00000bbUL)
                && !denied.value);
        }
    }
}

void appDataChecks(probe::Receipt& receipt, LONG& checks) {
    ComApartment apartment;
    const auto own = directoryPath(receipt.fileOwnRoot).substr(4);
    const auto logical = directoryPath(receipt.fileLogicalRoot);
    for (const auto& folder : {L"Roaming", L"Local"}) {
        const bool roaming = wcscmp(folder, L"Roaming") == 0;
        const auto& id = roaming ? FOLDERID_RoamingAppData : FOLDERID_LocalAppData;
        const auto env = childEnvironment(roaming ? L"APPDATA" : L"LOCALAPPDATA");
        const auto known = knownPath(id);
        require(env == own + L"\\" + folder && known == env
            && knownPath(id, KF_FLAG_DONT_VERIFY) == env);
        checks |= 65536;
        const auto filePath = env + L"\\appdata.bin";
        if (receipt.fileRound == 0) {
            Handle missing(CreateFileW(filePath.c_str(), GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE,
                nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr));
            require(missing.value == INVALID_HANDLE_VALUE && GetLastError() == ERROR_FILE_NOT_FOUND);
        } else {
            expectFile(filePath, slotMarker(receipt, 0));
        }
        checks |= 262144;
        Handle file(CreateFileW(filePath.c_str(), GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, receipt.fileRound == 0 ? CREATE_NEW : OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL, nullptr));
        require(file.value != INVALID_HANDLE_VALUE);
        writeBytes(file.value, slotMarker(receipt, receipt.fileRound));
        expectFile(known + L"\\appdata.bin", slotMarker(receipt, receipt.fileRound));
        expectFile(logical + L"\\" + folder + L"\\appdata.bin", slotMarker(receipt, receipt.fileRound));
        // Official Desktop data directory contains spaces; exercise the real pathname shape.
        const auto education = known + L"\\Minecraft Education Edition";
        if (receipt.fileRound == 0) require(CreateDirectoryW(education.c_str(), nullptr));
        const auto marker = education + L"\\storage-marker.bin";
        Handle educationFile(CreateFileW(marker.c_str(), GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, receipt.fileRound == 0 ? CREATE_NEW : OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL, nullptr));
        require(educationFile.value != INVALID_HANDLE_VALUE);
        if (receipt.fileRound != 0) expectBytes(educationFile.value, slotMarker(receipt, 0));
        writeBytes(educationFile.value, slotMarker(receipt, receipt.fileRound));
        expectFile(logical + L"\\" + folder + L"\\Minecraft Education Edition\\storage-marker.bin",
            slotMarker(receipt, receipt.fileRound));
        Handle directory(CreateFileW(known.c_str(), FILE_LIST_DIRECTORY | SYNCHRONIZE,
            FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, nullptr));
        require(directory.value != INVALID_HANDLE_VALUE
            && nativeEntries(directory.value, false) == std::set<std::wstring>{L"appdata.bin", L"Minecraft Education Edition"});
        checks |= 131072;
        PWSTR rejected = nullptr;
        constexpr DWORD sentinel = 0x13579bd;
        SetLastError(sentinel);
        const HRESULT foreign = SHGetKnownFolderPath(id, 0, INVALID_HANDLE_VALUE, &rejected);
        const DWORD error = GetLastError();
        require(foreign == E_ACCESSDENIED && error == sentinel && rejected == nullptr);
        for (const DWORD flag : {static_cast<DWORD>(KF_FLAG_DEFAULT_PATH), static_cast<DWORD>(KF_FLAG_CREATE)}) {
            require(SHGetKnownFolderPath(id, flag, nullptr, &rejected) == E_NOTIMPL && rejected == nullptr);
        }
        require(SHGetKnownFolderPath(id, 0, nullptr, nullptr) == E_POINTER);
        checks |= 524288;
    }
}

struct FileChild {
    Handle held;
    LONG checks = 0;
    explicit FileChild(probe::Receipt& receipt) {
        const auto path = directoryPath(receipt.fileLogicalRoot) + L"\\state.bin";
        if (receipt.fileRound == 0) {
            Handle missing(CreateFileW(path.c_str(), GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE,
                nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr));
            const DWORD error = GetLastError();
            require(missing.value == INVALID_HANDLE_VALUE && error == ERROR_FILE_NOT_FOUND);
        } else {
            expectFile(path, slotMarker(receipt, 0));
        }
        checks |= 1;
        held.value = CreateFileW(path.c_str(), GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr,
            receipt.fileRound == 0 ? CREATE_NEW : OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr);
        require(held.value != INVALID_HANDLE_VALUE);
        writeBytes(held.value, slotMarker(receipt, receipt.fileRound));
        checks |= 2;

        const HMODULE ntdll = GetModuleHandleW(L"ntdll.dll");
        const auto open = reinterpret_cast<probe::NativeFileOpen>(GetProcAddress(ntdll, "NtOpenFile"));
        const auto create = reinterpret_cast<probe::NativeFileCreate>(GetProcAddress(ntdll, "NtCreateFile"));
        require(open && create);
        std::wstring absolutePath = L"\\??\\" + path.substr(4);
        auto absoluteName = nativeName(absolutePath);
        OBJECT_ATTRIBUTES attributes{sizeof(OBJECT_ATTRIBUTES), nullptr, &absoluteName,
            OBJ_CASE_INSENSITIVE, nullptr, nullptr};
        IO_STATUS_BLOCK io{};
        constexpr DWORD sentinel = 0x13579bd;
        constexpr ULONG options = 0x20 | 0x40; // synchronous, non-directory
        Handle absolute;
        SetLastError(sentinel);
        const NTSTATUS status = open(&absolute.value, FILE_GENERIC_READ, &attributes, &io,
            FILE_SHARE_READ | FILE_SHARE_WRITE, options);
        const DWORD error = GetLastError();
        require(status == 0 && error == sentinel && io.Information == 1);
        expectBytes(absolute.value, slotMarker(receipt, receipt.fileRound));
        checks |= 4;

        std::wstring counted = L"state.bin_IGNORED";
        auto relativeName = nativeName(counted);
        relativeName.Length = relativeName.MaximumLength = 18;
        attributes.RootDirectory = receipt.fileLogicalRoot;
        attributes.ObjectName = &relativeName;
        attributes.Attributes |= OBJ_INHERIT;
        Handle relative;
        io = {};
        SetLastError(sentinel);
        const NTSTATUS created = create(&relative.value, FILE_GENERIC_READ, &attributes, &io,
            nullptr, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ | FILE_SHARE_WRITE,
            3 /* FILE_OPEN_IF */, options, nullptr, 0);
        const DWORD createError = GetLastError();
        DWORD flags = 0, written = 0;
        require(created == 0 && createError == sentinel && io.Information == 1
            && GetHandleInformation(relative.value, &flags) && (flags & HANDLE_FLAG_INHERIT)
            && relativeName.Length == 18 && counted == L"state.bin_IGNORED"
            && attributes.RootDirectory == receipt.fileLogicalRoot
            && attributes.ObjectName == &relativeName);
        expectBytes(relative.value, slotMarker(receipt, receipt.fileRound));
        require(!WriteFile(relative.value, "X", 1, &written, nullptr)
            && GetLastError() == ERROR_ACCESS_DENIED);
        checks |= 8;

        Handle allowedDelete;
        SetLastError(sentinel);
        require(open(&allowedDelete.value, FILE_GENERIC_READ | DELETE, &attributes, &io,
            FILE_SHARE_READ | FILE_SHARE_WRITE, options) == static_cast<NTSTATUS>(0xc0000043UL)
            && GetLastError() == sentinel && !allowedDelete.value); // Existing fixture readers deny delete sharing.
        Handle denied;
        std::wstring other = L"state.bin:stream";
        auto otherName = nativeName(other);
        attributes.ObjectName = &otherName;
        require(open(&denied.value, FILE_GENERIC_READ, &attributes, &io,
            FILE_SHARE_READ | FILE_SHARE_WRITE, options) == static_cast<NTSTATUS>(0xc00000bbUL)
            && !denied.value);
        checks |= 16;
        const auto logicalDirectory = directoryPath(receipt.fileLogicalRoot);
        const auto separator = logicalDirectory.find_last_of(L'\\');
        require(separator != std::wstring::npos);
        expectFile(logicalDirectory.substr(0, separator) + L"\\outside.bin", "OUTSIDE");
        checks |= 32;
        directoryChecks(receipt, checks);
        treeChecks(receipt, checks);
        appDataChecks(receipt, checks);
        deepStorageChecks(receipt);
    }
    void afterPeer(probe::Receipt& receipt) {
        expectBytes(held.value, slotMarker(receipt, receipt.fileRound));
        Handle root(CreateFileW(directoryPath(receipt.fileLogicalRoot).c_str(),
            FILE_LIST_DIRECTORY | SYNCHRONIZE, FILE_SHARE_READ | FILE_SHARE_WRITE,
            nullptr, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, nullptr));
        require(root.value != INVALID_HANDLE_VALUE
            && nativeEntries(root.value, false) == std::set<std::wstring>{
                L"a", L"state.bin", L"nested", L"Roaming", L"Local", L"Registry", receipt.slot == 1 ? L"slot-A.bin" : L"slot-B.bin"});
        const auto deep = directoryPath(receipt.fileLogicalRoot) + L"\\nested\\level\\deep";
        expectFile(deep + L"\\state.bin", slotMarker(receipt, receipt.fileRound));
        Handle nested(CreateFileW(deep.c_str(), FILE_LIST_DIRECTORY | SYNCHRONIZE,
            FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, nullptr));
        require(nested.value != INVALID_HANDLE_VALUE
            && nativeEntries(nested.value, false) == std::set<std::wstring>{
                L"state.bin", receipt.slot == 1 ? L"slot-A.bin" : L"slot-B.bin"});
        for (const auto& folder : {L"Roaming", L"Local"})
            expectFile(directoryPath(receipt.fileLogicalRoot) + L"\\" + folder + L"\\appdata.bin",
                slotMarker(receipt, receipt.fileRound));
        checks |= 64;
        InterlockedExchange(&receipt.fileChecks, checks);
    }
};


struct Key {
    HKEY value = nullptr;
    Key() = default;
    Key(const Key&) = delete;
    Key& operator=(const Key&) = delete;
    ~Key() { if (value) RegCloseKey(value); }
};
std::wstring registryPath(const std::wstring& name) {
    return L"Software\\" + name.substr(6);
}
DWORD registryValue(HKEY key, const wchar_t* name) {
    DWORD type = 0, value = 0, bytes = sizeof(value);
    require(RegQueryValueExW(key, name, nullptr, &type, reinterpret_cast<BYTE*>(&value), &bytes)
        == ERROR_SUCCESS && type == REG_DWORD && bytes == sizeof(value));
    return value;
}
void setRegistryValue(HKEY key, const wchar_t* name, DWORD value) {
    require(RegSetValueExW(key, name, 0, REG_DWORD, reinterpret_cast<const BYTE*>(&value),
        sizeof(value)) == ERROR_SUCCESS);
}
DWORD registryMarker(const probe::Receipt& receipt) {
    return receipt.slot * 100 + receipt.fileRound + 1;
}

using NativeRegistryOpen = NTSTATUS (NTAPI*)(PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES);
using NativeRegistryOpenEx = NTSTATUS (NTAPI*)(PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES, ULONG);
using NativeRegistryCreate = NTSTATUS (NTAPI*)(PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES,
    ULONG, PUNICODE_STRING, ULONG, PULONG);
std::wstring nativeKeyName(HKEY key) {
    using Query = NTSTATUS (NTAPI*)(HANDLE, ULONG, PVOID, ULONG, PULONG);
    const auto query = reinterpret_cast<Query>(GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "NtQueryKey"));
    require(query != nullptr);
    alignas(8) BYTE buffer[2048]{};
    ULONG returned = 0, bytes = 0;
    require(query(key, 3, buffer, sizeof(buffer), &returned) == 0 && returned >= 4 && returned <= sizeof(buffer));
    memcpy(&bytes, buffer, sizeof(bytes));
    require(bytes % sizeof(wchar_t) == 0 && bytes <= returned - 4);
    return std::wstring(reinterpret_cast<wchar_t*>(buffer + 4), bytes / sizeof(wchar_t));
}
struct SelfImpersonation {
    bool active = true;
    SelfImpersonation() { require(ImpersonateSelf(SecurityIdentification) != FALSE); }
    void finish() { require(RevertToSelf() != FALSE); active = false; }
    ~SelfImpersonation() { if (active && !RevertToSelf()) cleanupFailed = true; }
};


NTSTATUS outsideRegistryLongStatus() {
    const auto open = reinterpret_cast<NativeRegistryOpen>(
        GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "NtOpenKey"));
    require(open != nullptr);
    std::wstring path = L"\\Registry\\Machine\\" + std::wstring(1100, L'x');
    auto name = nativeName(path);
    OBJECT_ATTRIBUTES attributes{sizeof(OBJECT_ATTRIBUTES), nullptr, &name,
        OBJ_CASE_INSENSITIVE, nullptr, nullptr};
    Key rejected;
    const NTSTATUS status = open(reinterpret_cast<PHANDLE>(&rejected.value), KEY_READ, &attributes);
    require(status < 0 && !rejected.value);
    return status;
}

LONG registryEnumerationChecks(HKEY cached, HKEY own, DWORD slot) {
    using Enumerate = NTSTATUS (NTAPI*)(HANDLE, ULONG, ULONG, PVOID, ULONG, PULONG);
    const HMODULE ntdll = GetModuleHandleW(L"ntdll.dll");
    const auto keys = reinterpret_cast<Enumerate>(GetProcAddress(ntdll, "NtEnumerateKey"));
    const auto values = reinterpret_cast<Enumerate>(GetProcAddress(ntdll, "NtEnumerateValueKey"));
    require(keys && values);
    constexpr DWORD sentinel = 0x13579bd;
    // Compare the cached HOST handle with a real private handle, including untouched buffer bytes.
    for (auto enumerate : {keys, values}) {
        for (ULONG information : {0UL, 1UL, 2UL, 99UL}) {
            for (ULONG index : {0UL, 1UL, 2UL, 99UL}) {
                for (ULONG length : {0UL, 1UL, 16UL, 64UL, 256UL}) {
                    alignas(8) BYTE expected[256], actual[256];
                    memset(expected, 0xa5, sizeof(expected));
                    memset(actual, 0xa5, sizeof(actual));
                    ULONG expectedBytes = 0x12345678, actualBytes = 0x12345678;
                    SetLastError(sentinel);
                    const NTSTATUS expectedStatus = enumerate(own, index, information,
                        length ? expected : nullptr, length, &expectedBytes);
                    require(GetLastError() == sentinel);
                    SetLastError(sentinel);
                    const NTSTATUS actualStatus = enumerate(cached, index, information,
                        length ? actual : nullptr, length, &actualBytes);
                    require(GetLastError() == sentinel && actualStatus == expectedStatus
                        && actualBytes == expectedBytes && memcmp(actual, expected, sizeof(actual)) == 0);
                }
            }
        }
    }
    LONG checks = 8192;
    std::set<std::wstring> names;
    for (DWORD i = 0; i < 3; ++i) {
        wchar_t name[64]{};
        DWORD length = 64;
        const LSTATUS status = RegEnumValueW(cached, i, name, &length, nullptr, nullptr, nullptr, nullptr);
        if (i == 2) require(status == ERROR_NO_MORE_ITEMS);
        else require(status == ERROR_SUCCESS && names.insert(std::wstring(name, length)).second);
    }
    require(names == std::set<std::wstring>{L"Marker", slot == 1 ? L"OnlyA" : L"OnlyB"});
    names.clear();
    for (DWORD i = 0; i < 3; ++i) {
        wchar_t name[64]{};
        DWORD length = 64;
        const LSTATUS status = RegEnumKeyExW(cached, i, name, &length, nullptr, nullptr, nullptr, nullptr);
        if (i == 2) require(status == ERROR_NO_MORE_ITEMS);
        else require(status == ERROR_SUCCESS && names.insert(std::wstring(name, length)).second);
    }
    require(names == std::set<std::wstring>{L"Settings", L"Native"});
    checks |= 16384;
    for (ACCESS_MASK right : {static_cast<ACCESS_MASK>(KEY_QUERY_VALUE),
                             static_cast<ACCESS_MASK>(KEY_ENUMERATE_SUB_KEYS)}) {
        Key limitedCached, limitedOwn;
        require(DuplicateHandle(GetCurrentProcess(), cached, GetCurrentProcess(),
            reinterpret_cast<PHANDLE>(&limitedCached.value), right, FALSE, 0) != FALSE
            && DuplicateHandle(GetCurrentProcess(), own, GetCurrentProcess(),
            reinterpret_cast<PHANDLE>(&limitedOwn.value), right, FALSE, 0) != FALSE);
        for (auto enumerate : {keys, values}) {
            alignas(8) BYTE expected[256], actual[256];
            memset(expected, 0xa5, sizeof(expected));
            memset(actual, 0xa5, sizeof(actual));
            ULONG expectedBytes = 0x12345678, actualBytes = 0x12345678;
            const NTSTATUS expectedStatus = enumerate(limitedOwn.value, 0, 0,
                expected, sizeof(expected), &expectedBytes);
            SetLastError(sentinel);
            const NTSTATUS status = enumerate(limitedCached.value, 0, 0,
                actual, sizeof(actual), &actualBytes);
            const bool allowed = enumerate == keys ? right == KEY_ENUMERATE_SUB_KEYS : right == KEY_QUERY_VALUE;
            require(GetLastError() == sentinel && status == expectedStatus
                && status == (allowed ? 0 : static_cast<NTSTATUS>(0xc0000022UL))
                && actualBytes == expectedBytes && memcmp(actual, expected, sizeof(actual)) == 0);
        }
    }
    {
        SelfImpersonation impersonation;
        for (auto enumerate : {keys, values}) {
            alignas(8) BYTE output[64]{};
            ULONG returned = 0;
            SetLastError(sentinel);
            require(enumerate(cached, 0, 0, output, sizeof(output), &returned)
                == static_cast<NTSTATUS>(0xc00000bbUL) && GetLastError() == sentinel
                && returned == 0 && output[0] == 0);
        }
        impersonation.finish();
    }
    checks |= 32768;
    return checks;
}

struct RegistryChild {
    Key branch, hostRoot, nativeHeld;
    std::wstring hostName;
    NTSTATUS outsideLong = 0;
    LONG checks = 0;
    explicit RegistryChild(probe::Receipt& receipt) {
        hostRoot.value = receipt.cachedHostKey; // Only the fresh synthetic host branch is inherited.
        hostName = receipt.registryUserPath;
        outsideLong = outsideRegistryLongStatus();
        const bool ordinary = receipt.registryBacking == 1;
        require(receipt.earlyAllowed == 0);
        if (ordinary) {
            require(receipt.earlyPrivate == 7 && receipt.earlyDenied == 0);
            require(registryValue(hostRoot.value, L"Marker")
                == receipt.slot * 100 + (receipt.fileRound ? 1 : 0) + 3);
        } else {
            require(receipt.earlyDenied == 7 && receipt.earlyPrivate == 0);
            Key deniedRoot;
            require(RegOpenCurrentUser(KEY_READ, &deniedRoot.value) == ERROR_ACCESS_DENIED && !deniedRoot.value);
            DWORD type = 0, value = 0, bytes = sizeof(value);
            require(RegQueryValueExW(hostRoot.value, L"Marker", nullptr, &type,
                reinterpret_cast<BYTE*>(&value), &bytes) == ERROR_ACCESS_DENIED);
            const DWORD sameHost = 0x484f5354;
            require(RegSetValueExW(hostRoot.value, L"Marker", 0, REG_DWORD,
                reinterpret_cast<const BYTE*>(&sameHost), sizeof(sameHost)) == ERROR_ACCESS_DENIED);
        }
        if (receipt.mode != probe::ConfiguredStorage) {
            const auto initialize = reinterpret_cast<BOOL (*)()>(
                GetProcAddress(GetModuleHandleW(L"bootstrap-compat.dll"), "BootstrapRegistry"));
            require(initialize && initialize() && initialize());
        } // Configured storage must work without calling into the compatibility DLL from main.
        checks |= 1;
        const auto path = registryPath(receipt.logical);
        Key previous;
        const LSTATUS opened = RegOpenKeyExW(HKEY_CURRENT_USER, path.c_str(), 0, KEY_READ, &previous.value);
        if (ordinary) require(opened == ERROR_SUCCESS
            && registryValue(previous.value, L"Marker")
                == receipt.slot * 100 + (receipt.fileRound ? 1 : 0) + 3);
        else if (receipt.fileRound == 0) require(opened == ERROR_FILE_NOT_FOUND && !previous.value);
        else {
            require(opened == ERROR_SUCCESS
                && registryValue(previous.value, L"Marker") == receipt.slot * 100 + 1);
        }
        checks |= 2;
        DWORD disposition = 0;
        require(RegCreateKeyExW(HKEY_CURRENT_USER, path.c_str(), 0, nullptr,
            REG_OPTION_NON_VOLATILE, KEY_ALL_ACCESS, nullptr, &branch.value, &disposition) == ERROR_SUCCESS
            && disposition == static_cast<DWORD>(!ordinary && receipt.fileRound == 0 ? REG_CREATED_NEW_KEY : REG_OPENED_EXISTING_KEY));
        setRegistryValue(branch.value, L"Marker", registryMarker(receipt));
        setRegistryValue(branch.value, receipt.slot == 1 ? L"OnlyA" : L"OnlyB", receipt.slot);
        std::set<std::wstring> names;
        for (DWORD i = 0; i < 3; ++i) {
            wchar_t name[64]{};
            DWORD characters = 64;
            const LSTATUS status = RegEnumValueW(branch.value, i, name, &characters, nullptr,
                nullptr, nullptr, nullptr);
            if (i == 2) require(status == ERROR_NO_MORE_ITEMS);
            else require(status == ERROR_SUCCESS && names.insert(std::wstring(name, characters)).second);
        }
        require(names == std::set<std::wstring>{L"Marker", receipt.slot == 1 ? L"OnlyA" : L"OnlyB"});
        checks |= 4;
        Key readonly;
        require(RegOpenKeyExW(HKEY_CURRENT_USER, path.c_str(), 0, KEY_READ, &readonly.value) == ERROR_SUCCESS);
        const DWORD wrong = 0;
        require(RegSetValueExW(readonly.value, L"Marker", 0, REG_DWORD,
            reinterpret_cast<const BYTE*>(&wrong), sizeof(wrong)) == ERROR_ACCESS_DENIED
            && registryValue(readonly.value, L"Marker") == registryMarker(receipt));
        Key nested;
        require(RegCreateKeyExW(branch.value, L"Settings", 0, nullptr, REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS, nullptr, &nested.value, nullptr) == ERROR_SUCCESS);
        setRegistryValue(nested.value, L"Slot", receipt.slot);
        require(registryValue(nested.value, L"Slot") == receipt.slot);
        checks |= 8;
        nativeChecks(receipt);
        require(RegFlushKey(branch.value) == ERROR_SUCCESS);
        checks |= 16;
    }
    void nativeChecks(probe::Receipt& receipt) {
        const HMODULE ntdll = GetModuleHandleW(L"ntdll.dll");
        const auto open = reinterpret_cast<NativeRegistryOpen>(GetProcAddress(ntdll, "NtOpenKey"));
        const auto openEx = reinterpret_cast<NativeRegistryOpenEx>(GetProcAddress(ntdll, "NtOpenKeyEx"));
        const auto create = reinterpret_cast<NativeRegistryCreate>(GetProcAddress(ntdll, "NtCreateKey"));
        require(open && openEx && create);
        constexpr DWORD sentinel = 0x13579bd;
        Key current, viaUser;
        SetLastError(sentinel);
        const LSTATUS user = RegOpenCurrentUser(KEY_READ, &current.value);
        const DWORD userError = GetLastError();
        require(user == ERROR_SUCCESS && userError == sentinel
            && RegOpenKeyExW(current.value, registryPath(receipt.logical).c_str(), 0,
                KEY_READ, &viaUser.value) == ERROR_SUCCESS
            && registryValue(viaUser.value, L"Marker") == registryMarker(receipt));
        checks |= 128;
        const auto path = registryPath(receipt.logical);
        std::wstring absolute = hostName + L"\\" + path;
        auto name = nativeName(absolute);
        OBJECT_ATTRIBUTES attributes{sizeof(OBJECT_ATTRIBUTES), nullptr, &name,
            OBJ_CASE_INSENSITIVE, nullptr, nullptr};
        Key opened;
        SetLastError(sentinel);
        const NTSTATUS status = open(reinterpret_cast<PHANDLE>(&opened.value), KEY_READ, &attributes);
        const DWORD error = GetLastError();
        require(status == 0 && error == sentinel
            && registryValue(opened.value, L"Marker") == registryMarker(receipt));
        SetLastError(sentinel);
        const NTSTATUS exStatus = openEx(reinterpret_cast<PHANDLE>(&nativeHeld.value), KEY_READ, &attributes, 0);
        const DWORD exError = GetLastError();
        require(exStatus == 0 && exError == sentinel
            && registryValue(nativeHeld.value, L"Marker") == registryMarker(receipt));
        checks |= 256;

        std::wstring counted = L"_IGNORED";
        auto relativeName = nativeName(counted);
        relativeName.Length = relativeName.MaximumLength = 0;
        attributes.RootDirectory = hostRoot.value;
        attributes.ObjectName = &relativeName;
        Key relative;
        SetLastError(sentinel);
        const NTSTATUS relativeStatus = open(reinterpret_cast<PHANDLE>(&relative.value), KEY_READ, &attributes);
        const DWORD relativeError = GetLastError();
        require(relativeStatus == 0 && relativeError == sentinel
            && registryValue(relative.value, L"Marker") == registryMarker(receipt)
            && counted == L"_IGNORED" && attributes.RootDirectory == hostRoot.value
            && attributes.ObjectName == &relativeName
            && relativeName.Length == 0);
        require(registryValue(hostRoot.value, L"Marker") == registryMarker(receipt));
        setRegistryValue(hostRoot.value, L"Marker", registryMarker(receipt));
        Key cachedReadonly;
        require(DuplicateHandle(GetCurrentProcess(), hostRoot.value, GetCurrentProcess(),
            reinterpret_cast<PHANDLE>(&cachedReadonly.value), KEY_QUERY_VALUE, FALSE, 0) != FALSE);
        const DWORD deniedWrite = 0;
        require(RegSetValueExW(cachedReadonly.value, L"Marker", 0, REG_DWORD,
            reinterpret_cast<const BYTE*>(&deniedWrite), sizeof(deniedWrite)) == ERROR_ACCESS_DENIED
            && registryValue(cachedReadonly.value, L"Marker") == registryMarker(receipt));
        checks |= 512;

        std::wstring child = absolute + L"\\Native";
        auto childName = nativeName(child);
        attributes.RootDirectory = nullptr;
        attributes.ObjectName = &childName;
        ULONG disposition = 0;
        Key created;
        SetLastError(sentinel);
        const NTSTATUS createStatus = create(reinterpret_cast<PHANDLE>(&created.value), KEY_ALL_ACCESS,
            &attributes, 0, nullptr, 0, &disposition);
        const DWORD createError = GetLastError();
        require(createStatus == 0 && createError == sentinel
            && disposition == static_cast<ULONG>(receipt.fileRound == 0 ? REG_CREATED_NEW_KEY : REG_OPENED_EXISTING_KEY));
        setRegistryValue(created.value, L"Slot", receipt.slot);
        Key viaHKCU;
        require(RegOpenKeyExW(branch.value, L"Native", 0, KEY_READ, &viaHKCU.value) == ERROR_SUCCESS
            && registryValue(viaHKCU.value, L"Slot") == receipt.slot);
        checks |= 1024;

        Key rejected;
        SetLastError(sentinel);
        require(openEx(reinterpret_cast<PHANDLE>(&rejected.value), KEY_READ, &attributes,
            REG_OPTION_OPEN_LINK) == static_cast<NTSTATUS>(0xc00000bbUL)
            && GetLastError() == sentinel && !rejected.value);
        require(create(reinterpret_cast<PHANDLE>(&rejected.value), KEY_ALL_ACCESS, &attributes,
            0, nullptr, REG_OPTION_CREATE_LINK, &disposition) == static_cast<NTSTATUS>(0xc00000bbUL)
            && !rejected.value);
        std::wstring absent = absolute + L"\\Missing";
        auto missingName = nativeName(absent);
        attributes.ObjectName = &missingName;
        SetLastError(sentinel);
        require(open(reinterpret_cast<PHANDLE>(&rejected.value), KEY_READ, &attributes)
            == static_cast<NTSTATUS>(0xc0000034UL) && GetLastError() == sentinel && !rejected.value);
        UNICODE_STRING badName{2, 2, reinterpret_cast<PWSTR>(static_cast<ULONG_PTR>(1))};
        attributes.ObjectName = &badName;
        SetLastError(sentinel);
        require(open(reinterpret_cast<PHANDLE>(&rejected.value), KEY_READ, &attributes)
            == static_cast<NTSTATUS>(0xc0000005UL) && GetLastError() == sentinel && !rejected.value);
        attributes.ObjectName = &name;
        attributes.RootDirectory = hostRoot.value;
        require(open(reinterpret_cast<PHANDLE>(&rejected.value), KEY_READ, &attributes)
            == static_cast<NTSTATUS>(0xc00000bbUL) && !rejected.value);
        attributes.RootDirectory = nullptr;
        {
            SelfImpersonation impersonation;
            require(RegOpenCurrentUser(KEY_READ, &rejected.value) == ERROR_NOT_SUPPORTED && !rejected.value);
            require(open(reinterpret_cast<PHANDLE>(&rejected.value), KEY_READ, &attributes)
                == static_cast<NTSTATUS>(0xc00000bbUL) && !rejected.value);
            impersonation.finish();
        }
        std::wstring machine = L"\\Registry\\Machine";
        auto machineName = nativeName(machine);
        attributes.ObjectName = &machineName;
        Key other;
        SetLastError(sentinel);
        const NTSTATUS otherStatus = open(reinterpret_cast<PHANDLE>(&other.value), KEY_READ, &attributes);
        const DWORD otherError = GetLastError();
        require(otherStatus == 0 && otherError == sentinel && other.value); // Root metadata, no values queried.
        require(outsideRegistryLongStatus() == outsideLong);
        checks |= 2048;
    }
    void afterPeer(probe::Receipt& receipt) {
        checks |= registryEnumerationChecks(hostRoot.value, branch.value, receipt.slot);
        require(registryValue(nativeHeld.value, L"Marker") == registryMarker(receipt));
        checks |= 4096;
        require(registryValue(branch.value, L"Marker") == registryMarker(receipt));
        DWORD bytes = 0;
        require(RegQueryValueExW(branch.value, receipt.slot == 1 ? L"OnlyB" : L"OnlyA",
            nullptr, nullptr, nullptr, &bytes) == ERROR_FILE_NOT_FOUND);
        checks |= 32;
        if (receipt.registryBacking == 1) {
            // The ordinary store has no app-hive lock. Its container path must not bypass routing.
            auto container = nativeKeyName(receipt.inheritedRegistry);
            container.resize(container.find_last_of(L'\\'));
            auto name = nativeName(container);
            OBJECT_ATTRIBUTES attributes{sizeof(OBJECT_ATTRIBUTES), nullptr, &name,
                OBJ_CASE_INSENSITIVE, nullptr, nullptr};
            const auto open = reinterpret_cast<NativeRegistryOpen>(
                GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "NtOpenKey"));
            Key denied;
            require(open && open(reinterpret_cast<PHANDLE>(&denied.value), KEY_READ, &attributes)
                == static_cast<NTSTATUS>(0xc00000bbUL) && !denied.value);
        } else {
            auto peer = directoryPath(receipt.fileOwnRoot).substr(4);
            require(!peer.empty() && (peer.back() == L'A' || peer.back() == L'B'));
            peer.back() = receipt.slot == 1 ? L'B' : L'A';
            Key denied;
            const LSTATUS status = RegLoadAppKeyW((peer + L"\\Registry\\HKCU.dat").c_str(),
                &denied.value, KEY_READ, 0, 0);
            require(status != ERROR_SUCCESS && !denied.value);
        }
        checks |= 64;
        InterlockedExchange(&receipt.registryChecks, checks);
    }
};

int mutexChild(probe::Receipt& receipt) {
    RegistryChild registry(receipt);
    FileChild files(receipt);
    Handle first(CreateMutexW(nullptr, TRUE, receipt.logical));
    const DWORD firstError = GetLastError();
    require(first.value && firstError != ERROR_ALREADY_EXISTS);
    LONG checks = 1;
    SECURITY_ATTRIBUTES inheritable{sizeof(SECURITY_ATTRIBUTES), nullptr, TRUE};
    Handle duplicate(CreateMutexW(&inheritable, FALSE, receipt.logical));
    const DWORD duplicateError = GetLastError();
    DWORD flags = 0;
    require(duplicate.value && duplicateError == ERROR_ALREADY_EXISTS
        && GetHandleInformation(duplicate.value, &flags) && (flags & HANDLE_FLAG_INHERIT));
    checks |= 2;
    Handle opened(OpenMutexW(SYNCHRONIZE | MUTEX_MODIFY_STATE, FALSE, receipt.logical));
    require(opened.value && WaitForSingleObject(opened.value, 0) == WAIT_OBJECT_0
        && ReleaseMutex(opened.value));
    checks |= 4;
    Handle unnamed(CreateMutexW(nullptr, FALSE, nullptr));
    require(unnamed.value && WaitForSingleObject(unnamed.value, 0) == WAIT_OBJECT_0
        && ReleaseMutex(unnamed.value));
    checks |= 8;
    const std::wstring absent = std::wstring(receipt.logical) + L"_missing";
    Handle missing(OpenMutexW(SYNCHRONIZE, FALSE, absent.c_str()));
    const DWORD missingError = GetLastError();
    require(!missing.value && missingError == ERROR_FILE_NOT_FOUND);
    checks |= 16;
    Handle limited(OpenMutexW(MUTEX_MODIFY_STATE, TRUE, receipt.logical));
    require(limited.value && GetHandleInformation(limited.value, &flags)
        && (flags & HANDLE_FLAG_INHERIT));
    const DWORD limitedWait = WaitForSingleObject(limited.value, 0);
    const DWORD limitedError = GetLastError();
    require(limitedWait == WAIT_FAILED && limitedError == ERROR_ACCESS_DENIED);
    checks |= 32;
    InterlockedExchange(&receipt.nativeChecks, nativeMutexChecks(receipt, first.value));
    InterlockedExchange(&receipt.mutexChecks, checks);
    require(SetEvent(receipt.ready) != FALSE);
    require(WaitForSingleObject(receipt.go, 10000) == WAIT_OBJECT_0);
    Handle peer(OpenMutexW(SYNCHRONIZE, FALSE, receipt.peer));
    require(peer.value && WaitForSingleObject(peer.value, 0) == WAIT_TIMEOUT);
    files.afterPeer(receipt);
    registry.afterPeer(receipt);
    // Same-user direct backing-name access remains possible: explicitly not an ACL boundary.
    checks |= 64;
    InterlockedExchange(&receipt.mutexChecks, checks);
    require(SetEvent(receipt.done) != FALSE);
    require(WaitForSingleObject(receipt.finish, 10000) == WAIT_OBJECT_0);
    require(ReleaseMutex(first.value) != FALSE);
    return 0;
}

HANDLE sharedEvent() {
    SECURITY_ATTRIBUTES security{sizeof(SECURITY_ATTRIBUTES), nullptr, TRUE};
    HANDLE event = CreateEventW(&security, TRUE, FALSE, nullptr);
    require(event != nullptr);
    return event;
}
HANDLE sharedSection() {
    SECURITY_ATTRIBUTES security{sizeof(SECURITY_ATTRIBUTES), nullptr, TRUE};
    HANDLE section = CreateFileMappingW(INVALID_HANDLE_VALUE, &security, PAGE_READWRITE,
                                        0, sizeof(probe::Receipt), nullptr);
    require(section != nullptr);
    return section;
}
void copyName(wchar_t (&destination)[128], const std::wstring& name) {
    require(name.size() < 128);
    for (size_t i = 0; i <= name.size(); ++i) destination[i] = name.c_str()[i];
}
struct MutexChild {
    Handle section{sharedSection()};
    Mapping mapping{section.value};
    Handle ready{sharedEvent()}, done{sharedEvent()};
    Child child; // Destroy child before receipt/events, including constructor failure.
    MutexChild(DWORD slot, const std::wstring& name, HANDLE go, HANDLE finish,
               const std::wstring& self, const std::string& compatibility,
               DWORD round, HANDLE logicalRoot, HANDLE ownRoot, HKEY cachedHostKey,
               const std::wstring& userPath, bool wrongAppData = false, bool registryBaseline = false,
               HKEY inheritedRegistry = nullptr, bool ordinaryRegistry = false) {
        ZeroMemory(mapping.value, sizeof(probe::Receipt));
        auto& receipt = *mapping.value;
        receipt.magic = probe::Magic;
        receipt.version = probe::Version;
        receipt.mode = probe::MutexCase;
        receipt.slot = slot;
        receipt.fileRound = round;
        receipt.registryBaseline = registryBaseline ? 1 : 0;
        receipt.cachedHostKey = cachedHostKey;
        receipt.registryBacking = ordinaryRegistry ? 1 : 0;
        receipt.inheritedRegistry = inheritedRegistry;
        require(userPath.size() < 512);
        memcpy(receipt.registryUserPath, userPath.c_str(), (userPath.size() + 1) * sizeof(wchar_t));
        receipt.fileLogicalRoot = logicalRoot;
        receipt.fileOwnRoot = ownRoot;
        receipt.ready = ready.value;
        receipt.go = go;
        receipt.done = done.value;
        receipt.finish = finish;
        copyName(receipt.logical, name);
        copyName(receipt.own, name + (slot == 1 ? L"_A" : L"_B"));
        copyName(receipt.peer, name + (slot == 1 ? L"_B" : L"_A"));
        HANDLE inherited[]{section.value, ready.value, go, done.value, finish, logicalRoot, ownRoot, cachedHostKey, inheritedRegistry};
        Attributes attributes(inherited, inheritedRegistry ? 9 : 8);
        auto env = environment(section.value, wrongAppData ? logicalRoot : ownRoot);
        STARTUPINFOEXW startup{};
        startup.StartupInfo.cb = sizeof(startup);
        startup.lpAttributeList = attributes.value;
        std::wstring command = L"\"" + self + L"\" --bootstrap-test-child";
        require(CreateProcessW(self.c_str(), command.data(), nullptr, nullptr, TRUE,
            CREATE_SUSPENDED | CREATE_UNICODE_ENVIRONMENT | EXTENDED_STARTUPINFO_PRESENT
                | CREATE_NO_WINDOW,
            env.data(), nullptr, &startup.StartupInfo, &child.info) != FALSE);
        receipt.pid = child.info.dwProcessId;
        FILETIME exited{}, kernel{}, user{};
        require(GetProcessTimes(child.info.hProcess, &receipt.created, &exited, &kernel, &user)
            != FALSE);
        const char* dlls[]{compatibility.c_str()};
        if (!registryBaseline) require(DetourUpdateProcessWithDll(child.info.hProcess, dlls, 1) != FALSE);
        MemoryBarrier();
        require(ResumeThread(child.info.hThread) == 1);
    }
    void phase(HANDLE event) {
        HANDLE states[]{event, child.info.hProcess};
        const DWORD outcome = WaitForMultipleObjects(2, states, FALSE, 10000);
        if (outcome == WAIT_OBJECT_0 + 1) {
            DWORD code = 0;
            require(GetExitCodeProcess(child.info.hProcess, &code));
            std::fprintf(stderr, "{\"controlChildExit\":%lu}\n", code);
        }
        require(outcome == WAIT_OBJECT_0);
        require(WaitForSingleObject(child.info.hProcess, 0) == WAIT_TIMEOUT);
    }
    void exitedNormally() {
        require(WaitForSingleObject(child.info.hProcess, 10000) == WAIT_OBJECT_0);
        DWORD code = STILL_ACTIVE;
        require(GetExitCodeProcess(child.info.hProcess, &code) != FALSE && code == 0);
        const LONG count = InterlockedCompareExchange(&mapping.value->count, 0, 0);
        require(count == 5 && mapping.value->armed == 1);
        constexpr LONG stages[]{probe::Attach, probe::Armed, probe::Dependency,
                                probe::Tls, probe::Main};
        for (LONG i = 0; i < count; ++i) {
            require(mapping.value->events[i].stage == stages[i]
                && mapping.value->events[i].armed == (i == 0 ? 0 : 1));
        }
    }
};


void syntheticJunction(const std::wstring& link, const std::wstring& target) {
    require(CreateDirectoryW(link.c_str(), nullptr) != FALSE);
    Handle directory(CreateFileW(link.c_str(), GENERIC_WRITE, 0, nullptr, OPEN_EXISTING,
        FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT, nullptr));
    require(directory.value != INVALID_HANDLE_VALUE);
    struct MountPoint {
        ULONG tag;
        USHORT bytes, reserved;
        USHORT substituteOffset, substituteBytes, printOffset, printBytes;
        wchar_t paths[1024];
    } data{};
    static_assert(offsetof(MountPoint, paths) == 16);
    const std::wstring substitute = L"\\??\\" + target;
    require(substitute.size() + target.size() + 2 < 1024);
    data.tag = IO_REPARSE_TAG_MOUNT_POINT;
    data.substituteBytes = static_cast<USHORT>(substitute.size() * sizeof(wchar_t));
    data.printOffset = static_cast<USHORT>(data.substituteBytes + sizeof(wchar_t));
    data.printBytes = static_cast<USHORT>(target.size() * sizeof(wchar_t));
    memcpy(data.paths, substitute.c_str(), data.substituteBytes + sizeof(wchar_t));
    memcpy(reinterpret_cast<BYTE*>(data.paths) + data.printOffset,
        target.c_str(), data.printBytes + sizeof(wchar_t));
    data.bytes = static_cast<USHORT>(8 + data.printOffset + data.printBytes + sizeof(wchar_t));
    DWORD returned = 0;
    require(DeviceIoControl(directory.value, FSCTL_SET_REPARSE_POINT, &data,
        data.bytes + 8, nullptr, 0, &returned, nullptr) != FALSE);
}

struct FileRoots {
    std::wstring root;
    Handle logical, a, b;
    FileRoots() {
        wchar_t temporary[512]{};
        const DWORD length = GetTempPathW(512, temporary);
        require(length > 0 && length < 512);
        root = std::wstring(temporary) + L"MivubiFileProbe_" + std::to_wstring(GetCurrentProcessId())
            + L"_" + std::to_wstring(GetTickCount64());
        require(CreateDirectoryW(root.c_str(), nullptr) != FALSE); // Refuse existing test roots.
        for (const auto& leaf : {L"logical", L"A", L"B"})
            require(CreateDirectoryW((root + L"\\" + leaf).c_str(), nullptr) != FALSE);
        for (const auto& leaf : {L"guard", L"A\\Registry", L"B\\Registry", L"A\\Roaming", L"A\\Local", L"B\\Roaming", L"B\\Local",
                L"logical\\Roaming", L"logical\\Local", L"A\\nested", L"B\\nested",
                L"logical\\nested", L"logical\\nested\\level", L"logical\\nested\\level\\deep"})
            require(CreateDirectoryW((root + L"\\" + leaf).c_str(), nullptr) != FALSE);
        syntheticJunction(root + L"\\A\\nested\\escape", root + L"\\guard");
        syntheticJunction(root + L"\\B\\nested\\escape", root + L"\\guard");
        SECURITY_ATTRIBUTES security{sizeof(SECURITY_ATTRIBUTES), nullptr, TRUE};
        const auto open = [&](const wchar_t* leaf) {
            HANDLE result = CreateFileW((root + L"\\" + leaf).c_str(), FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY,
                FILE_SHARE_READ | FILE_SHARE_WRITE, &security, OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT, nullptr);
            require(result != INVALID_HANDLE_VALUE);
            return result;
        };
        logical.value = open(L"logical"); a.value = open(L"A"); b.value = open(L"B");
        for (const auto& leaf : {L"logical\\state.bin", L"logical\\host-only.bin", L"logical\\nested\\level\\deep\\state.bin", L"guard\\state.bin", L"logical\\Roaming\\appdata.bin", L"logical\\Local\\appdata.bin", L"outside.bin"}) {
            Handle file(CreateFileW((root + L"\\" + leaf).c_str(), GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, nullptr));
            require(file.value != INVALID_HANDLE_VALUE);
            writeBytes(file.value, wcscmp(leaf, L"outside.bin") == 0 ? "OUTSIDE" : "HOST");
        }
    }
    void verify(DWORD round) {
        require(GetFileAttributesW((root + L"\\logical\\a").c_str()) == INVALID_FILE_ATTRIBUTES
            && GetLastError() == ERROR_FILE_NOT_FOUND);
        expectFile(root + L"\\logical\\state.bin", "HOST");
        expectFile(root + L"\\outside.bin", "OUTSIDE");
        expectFile(root + L"\\logical\\host-only.bin", "HOST");
        expectFile(root + L"\\logical\\nested\\level\\deep\\state.bin", "HOST");
        expectFile(root + L"\\guard\\state.bin", "HOST");
        for (const auto& folder : {L"Roaming", L"Local"}) {
            expectFile(root + L"\\logical\\" + folder + L"\\appdata.bin", "HOST");
            expectFile(root + L"\\A\\" + folder + L"\\appdata.bin", "A" + std::to_string(round + 1));
            expectFile(root + L"\\B\\" + folder + L"\\appdata.bin", "B" + std::to_string(round + 1));
        }
        expectFile(root + L"\\A\\nested\\level\\deep\\state.bin", "A" + std::to_string(round + 1));
        expectFile(root + L"\\B\\nested\\level\\deep\\state.bin", "B" + std::to_string(round + 1));
        expectFile(root + L"\\A\\nested\\level\\deep\\slot-A.bin", "A" + std::to_string(round + 1));
        expectFile(root + L"\\B\\nested\\level\\deep\\slot-B.bin", "B" + std::to_string(round + 1));
        expectFile(root + L"\\A\\slot-A.bin", "A" + std::to_string(round + 1));
        expectFile(root + L"\\B\\slot-B.bin", "B" + std::to_string(round + 1));
        expectFile(root + L"\\A\\state.bin", "A" + std::to_string(round + 1));
        expectFile(root + L"\\B\\state.bin", "B" + std::to_string(round + 1));
    }
    // Preserve only newly created synthetic files; no recursive cleanup or user data reads.
};


struct RegistryHost {
    Key branch;
    std::wstring path, userPath;
    explicit RegistryHost(const std::wstring& name) : path(registryPath(name)) {
        DWORD disposition = 0;
        require(RegCreateKeyExW(HKEY_CURRENT_USER, path.c_str(), 0, nullptr,
            REG_OPTION_NON_VOLATILE, KEY_ALL_ACCESS, nullptr, &branch.value, &disposition) == ERROR_SUCCESS
            && disposition == REG_CREATED_NEW_KEY);
        setRegistryValue(branch.value, L"Marker", 0x484f5354);
        Key hostOnly;
        require(RegCreateKeyExW(branch.value, L"HostOnly", 0, nullptr, REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS, nullptr, &hostOnly.value, &disposition) == ERROR_SUCCESS
            && disposition == REG_CREATED_NEW_KEY);
        require(SetHandleInformation(branch.value, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) != FALSE);
        Key root;
        require(RegOpenCurrentUser(KEY_READ, &root.value) == ERROR_SUCCESS);
        userPath = nativeKeyName(root.value);
    }
    void unchanged() {
        require(registryValue(branch.value, L"Marker") == 0x484f5354);
        wchar_t name[64]{};
        DWORD length = 64;
        require(RegEnumKeyExW(branch.value, 0, name, &length, nullptr, nullptr, nullptr, nullptr)
            == ERROR_SUCCESS && std::wstring(name, length) == L"HostOnly");
        length = 64;
        require(RegEnumKeyExW(branch.value, 1, name, &length, nullptr, nullptr, nullptr, nullptr)
            == ERROR_NO_MORE_ITEMS);
        DWORD bytes = 0;
        require(RegQueryValueExW(branch.value, L"OnlyA", nullptr, nullptr, nullptr, &bytes) == ERROR_FILE_NOT_FOUND
            && RegQueryValueExW(branch.value, L"OnlyB", nullptr, nullptr, nullptr, &bytes) == ERROR_FILE_NOT_FOUND);
    }
    void exclusive(const std::wstring& root) {
        unchanged();
        for (const auto& slot : {L"A", L"B"}) {
            Key denied;
            require(RegLoadAppKeyW((root + L"\\" + slot + L"\\Registry\\HKCU.dat").c_str(),
                &denied.value, KEY_READ, 0, 0) != ERROR_SUCCESS && !denied.value);
        }
    }
    void persisted(const std::wstring& root, DWORD round) {
        unchanged();
        DWORD slot = 0;
        for (const auto& leaf : {L"A", L"B"}) {
            ++slot;
            Key hive, own;
            require(RegLoadAppKeyW((root + L"\\" + leaf + L"\\Registry\\HKCU.dat").c_str(),
                &hive.value, KEY_READ, REG_PROCESS_APPKEY, 0) == ERROR_SUCCESS
                && RegOpenKeyExW(hive.value, path.c_str(), 0, KEY_READ, &own.value) == ERROR_SUCCESS
                && registryValue(own.value, L"Marker") == slot * 100 + round + 1
                && registryValue(own.value, slot == 1 ? L"OnlyA" : L"OnlyB") == slot);
            Key native;
            require(RegOpenKeyExW(own.value, L"Native", 0, KEY_READ, &native.value) == ERROR_SUCCESS
                && registryValue(native.value, L"Slot") == slot);
        }
    }
    // Preserve this fresh synthetic HOST key; never recurse into existing registry data.
};

bool mutexPair(const std::wstring& self, const std::string& compatibility) {
    FileRoots files;
    const std::wstring name = L"Local\\MivubiMutexProbe_" + std::to_wstring(GetCurrentProcessId())
        + L"_" + std::to_wstring(GetTickCount64());
    RegistryHost registry(name);
    Handle host(CreateMutexW(nullptr, TRUE, name.c_str()));
    const DWORD hostError = GetLastError();
    require(host.value && hostError != ERROR_ALREADY_EXISTS);
    {
        Handle go(sharedEvent()), finish(sharedEvent());
        MutexChild baseline(1, name, go.value, finish.value, self, compatibility,
            0, files.logical.value, files.a.value, registry.branch.value, registry.userPath, false, true);
        require(WaitForSingleObject(baseline.child.info.hProcess, 10000) == WAIT_OBJECT_0);
        DWORD code = 1;
        require(GetExitCodeProcess(baseline.child.info.hProcess, &code) && code == 0);
        const auto& receipt = *baseline.mapping.value;
        require(receipt.count == 3 && receipt.armed == 0 && receipt.earlyAllowed == 7
            && receipt.earlyDenied == 0 && receipt.registryChecks == 0);
        constexpr LONG stages[]{probe::Dependency, probe::Tls, probe::Main};
        for (LONG i = 0; i < 3; ++i)
            require(receipt.events[i].stage == stages[i] && receipt.events[i].armed == 0);
        registry.unchanged();
    }
    for (const bool wrongAppData : {false, true}) {
        Handle go(sharedEvent()), finish(sharedEvent());
        MutexChild mismatch(1, name, go.value, finish.value, self, compatibility,
            0, files.logical.value, wrongAppData ? files.a.value : files.b.value,
            registry.branch.value, registry.userPath, wrongAppData);
        require(WaitForSingleObject(mismatch.child.info.hProcess, 10000) == WAIT_OBJECT_0);
        DWORD code = 0;
        require(GetExitCodeProcess(mismatch.child.info.hProcess, &code) && code != 0
            && mismatch.mapping.value->armed == 0);
        bool rejected = false;
        const LONG count = mismatch.mapping.value->count;
        require(count >= 2 && count <= probe::Capacity);
        for (LONG i = 0; i < count; ++i) {
            const LONG stage = mismatch.mapping.value->events[i].stage;
            require(stage != probe::Armed && stage != probe::Main);
            rejected = rejected || stage == probe::Rejected;
        }
        require(rejected);
    }
    {
        const auto path = files.root + L"\\A\\Registry\\HKCU.dat";
        {
            Handle invalid(CreateFileW(path.c_str(), GENERIC_WRITE, 0, nullptr, CREATE_NEW,
                FILE_ATTRIBUTE_NORMAL, nullptr));
            require(invalid.value != INVALID_HANDLE_VALUE);
            writeBytes(invalid.value, "INVALID_SYNTHETIC_HIVE");
        }
        Handle go(sharedEvent()), finish(sharedEvent());
        MutexChild rejected(1, name, go.value, finish.value, self, compatibility,
            0, files.logical.value, files.a.value, registry.branch.value, registry.userPath);
        require(WaitForSingleObject(rejected.child.info.hProcess, 10000) == WAIT_OBJECT_0);
        DWORD code = 0;
        require(GetExitCodeProcess(rejected.child.info.hProcess, &code) && code == 85
            && rejected.mapping.value->registryChecks == 0);
        registry.unchanged();
        require(MoveFileW(path.c_str(), (files.root + L"\\A\\Registry\\invalid.dat").c_str()) != FALSE);
    }
    for (DWORD round = 0; round < 2; ++round) {
        Handle go(sharedEvent()), finish(sharedEvent());
        MutexChild first(1, name, go.value, finish.value, self, compatibility,
            round, files.logical.value, files.a.value, registry.branch.value, registry.userPath);
        MutexChild second(2, name, go.value, finish.value, self, compatibility,
            round, files.logical.value, files.b.value, registry.branch.value, registry.userPath);
        first.phase(first.ready.value);
        second.phase(second.ready.value);
        {
            Handle a(OpenMutexW(SYNCHRONIZE, FALSE, (name + L"_A").c_str()));
            Handle b(OpenMutexW(SYNCHRONIZE, FALSE, (name + L"_B").c_str()));
            require(a.value && b.value && WaitForSingleObject(a.value, 0) == WAIT_TIMEOUT
                && WaitForSingleObject(b.value, 0) == WAIT_TIMEOUT);
            Handle original(OpenMutexW(SYNCHRONIZE | MUTEX_MODIFY_STATE, FALSE, name.c_str()));
            require(original.value && WaitForSingleObject(original.value, 0) == WAIT_OBJECT_0
                && ReleaseMutex(original.value));
        }
        files.verify(round);
        registry.exclusive(files.root);
        require(SetEvent(go.value) != FALSE);
        first.phase(first.done.value);
        second.phase(second.done.value);
        require(InterlockedCompareExchange(&first.mapping.value->mutexChecks, 0, 0) == 127
            && InterlockedCompareExchange(&second.mapping.value->mutexChecks, 0, 0) == 127
            && InterlockedCompareExchange(&first.mapping.value->nativeChecks, 0, 0) == 511
            && InterlockedCompareExchange(&second.mapping.value->nativeChecks, 0, 0) == 511
            && InterlockedCompareExchange(&first.mapping.value->fileChecks, 0, 0) == 1048575
            && InterlockedCompareExchange(&second.mapping.value->fileChecks, 0, 0) == 1048575
            && InterlockedCompareExchange(&first.mapping.value->registryChecks, 0, 0) == 65535
            && InterlockedCompareExchange(&second.mapping.value->registryChecks, 0, 0) == 65535
            && first.mapping.value->earlyDenied == 7 && second.mapping.value->earlyDenied == 7
            && first.mapping.value->earlyAllowed == 0 && second.mapping.value->earlyAllowed == 0);
        require(SetEvent(finish.value) != FALSE);
        first.exitedNormally();
        second.exitedNormally();
        for (const auto& suffix : {L"_A", L"_B"}) {
            Handle gone(OpenMutexW(SYNCHRONIZE, FALSE, (name + suffix).c_str()));
            const DWORD error = GetLastError();
            require(!gone.value && error == ERROR_FILE_NOT_FOUND);
        }
        files.verify(round);
        registry.persisted(files.root, round);
    }
    require(ReleaseMutex(host.value) != FALSE);
    std::printf("{\"case\":\"mutex-two-child\",\"selfCheck\":true,\"checks\":[127,127],"
                "\"earlyDenied\":[7,7],\"earlyAllowedControl\":7,\"registryChecks\":[65535,65535],\"invalidHiveRejected\":true,\"fileChecks\":[1048575,1048575],\"rounds\":2,\"persistent\":true,\"hostUnchanged\":true,\"slotMismatchRejected\":true,\"appDataBindingRejected\":true,"
                "\"nativeChecks\":[511,511],\"distinctHostAndPeerObjects\":true,\"rawPeerAccessible\":true,"
                "\"securityBoundary\":false,\"providerReady\":false,"
                "\"minecraft\":\"NOT_EXECUTED\"}\n");
    return true;
}


bool ordinaryRegistryPair(const std::wstring& self, const std::string& compatibility) {
    FileRoots files;
    const std::wstring name = L"Local\\MivubiMutexProbe_" + std::to_wstring(GetCurrentProcessId())
        + L"_" + std::to_wstring(GetTickCount64());
    RegistryHost registry(name);
    Key stores[2];
    std::wstring paths[2];
    SECURITY_ATTRIBUTES security{sizeof(SECURITY_ATTRIBUTES), nullptr, TRUE};
    for (DWORD slot = 1; slot <= 2; ++slot) {
        FILE_ID_INFO id{};
        require(GetFileInformationByHandleEx(slot == 1 ? files.a.value : files.b.value,
            FileIdInfo, &id, sizeof(id)) != FALSE);
        wchar_t leaf[64]{};
        require(probe::registryStoreLeaf(id, slot, leaf));
        paths[slot - 1] = registry.path + L"_Stores\\" + leaf;
        DWORD disposition = 0;
        require(RegCreateKeyExW(HKEY_CURRENT_USER, paths[slot - 1].c_str(), 0, nullptr,
            REG_OPTION_NON_VOLATILE, KEY_ALL_ACCESS, &security, &stores[slot - 1].value,
            &disposition) == ERROR_SUCCESS && disposition == REG_CREATED_NEW_KEY);
        Key branch;
        require(RegCreateKeyExW(stores[slot - 1].value, registry.path.c_str(), 0, nullptr,
            REG_OPTION_NON_VOLATILE, KEY_ALL_ACCESS, nullptr, &branch.value, &disposition)
            == ERROR_SUCCESS && disposition == REG_CREATED_NEW_KEY);
        setRegistryValue(branch.value, L"Marker", slot * 100);
    }
    for (DWORD invalid = 0; invalid < 3; ++invalid) {
        Handle go(sharedEvent()), finish(sharedEvent());
        // Wrong slot, wrong object type, and absent backing never fall back to the host/app-hive.
        Handle wrongType(sharedEvent());
        HKEY backing = invalid == 0 ? stores[1].value
            : invalid == 1 ? reinterpret_cast<HKEY>(wrongType.value) : nullptr;
        MutexChild rejected(1, name, go.value, finish.value, self, compatibility,
            0, files.logical.value, files.a.value, registry.branch.value, registry.userPath,
            false, false, backing, true);
        require(WaitForSingleObject(rejected.child.info.hProcess, 10000) == WAIT_OBJECT_0);
        DWORD code = 0;
        require(GetExitCodeProcess(rejected.child.info.hProcess, &code) && code != 0
            && rejected.mapping.value->armed == 0 && rejected.mapping.value->earlyPrivate == 0);
        bool marked = false;
        for (LONG i = 0; i < rejected.mapping.value->count && i < probe::Capacity; ++i)
            marked = marked || rejected.mapping.value->events[i].stage == probe::Rejected;
        require(marked);
        registry.unchanged();
    }
    for (DWORD round = 0; round < 2; ++round) {
        Handle go(sharedEvent()), finish(sharedEvent());
        MutexChild first(1, name, go.value, finish.value, self, compatibility,
            round, files.logical.value, files.a.value, registry.branch.value, registry.userPath,
            false, false, stores[0].value, true);
        MutexChild second(2, name, go.value, finish.value, self, compatibility,
            round, files.logical.value, files.b.value, registry.branch.value, registry.userPath,
            false, false, stores[1].value, true);
        first.phase(first.ready.value);
        second.phase(second.ready.value);
        registry.unchanged();
        require(SetEvent(go.value) != FALSE);
        first.phase(first.done.value);
        second.phase(second.done.value);
        for (auto* receipt : {first.mapping.value, second.mapping.value})
            require(receipt->registryChecks == 65535 && receipt->earlyPrivate == 7
                && receipt->earlyDenied == 0 && receipt->earlyAllowed == 0);
        require(SetEvent(finish.value) != FALSE);
        first.exitedNormally();
        second.exitedNormally();
        registry.unchanged();
        for (DWORD slot = 1; slot <= 2; ++slot) {
            auto& store = stores[slot - 1];
            require(RegCloseKey(store.value) == ERROR_SUCCESS);
            store.value = nullptr;
            // Reopen ordinary nonvolatile backing; do not recreate or reseed it on relaunch.
            require(RegOpenKeyExW(HKEY_CURRENT_USER, paths[slot - 1].c_str(), 0,
                KEY_ALL_ACCESS, &store.value) == ERROR_SUCCESS
                && SetHandleInformation(store.value, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT));
            Key branch;
            require(RegOpenKeyExW(store.value, registry.path.c_str(), 0, KEY_READ, &branch.value)
                == ERROR_SUCCESS && registryValue(branch.value, L"Marker") == slot * 100 + round + 1
                && registryValue(branch.value, slot == 1 ? L"OnlyA" : L"OnlyB") == slot);
            DWORD bytes = 0;
            require(RegQueryValueExW(branch.value, slot == 1 ? L"OnlyB" : L"OnlyA",
                nullptr, nullptr, nullptr, &bytes) == ERROR_FILE_NOT_FOUND);
        }
    }
    std::printf("{\"case\":\"ordinary-inherited-registry\",\"selfCheck\":true,"
        "\"earlyPrivate\":[7,7],\"registryChecks\":[65535,65535],\"rounds\":2,"
        "\"persistent\":true,\"hostUnchanged\":true,\"invalidBackingRejected\":3,"
        "\"applicationHiveLock\":false,\"securityBoundary\":false,\"providerReady\":false,"
        "\"minecraft\":\"NOT_EXECUTED\"}\n");
    return true;
}


struct ConfiguredRoots {
    std::wstring root;
    Handle logical, a, b;
    ConfiguredRoots() {
        wchar_t temporary[512]{};
        const DWORD length = GetTempPathW(512, temporary);
        require(length > 0 && length < 512);
        root = std::wstring(temporary) + L"MivubiEducationConfig_" + std::to_wstring(GetCurrentProcessId())
            + L"_" + std::to_wstring(GetTickCount64());
        require(CreateDirectoryW(root.c_str(), nullptr) != FALSE);
        SECURITY_ATTRIBUTES security{sizeof(SECURITY_ATTRIBUTES), nullptr, TRUE};
        for (const auto& leaf : {L"host-view", L"client-one", L"client-two"}) {
            const auto path = root + L"\\" + leaf;
            require(CreateDirectoryW(path.c_str(), nullptr) != FALSE);
            for (const auto& child : {L"Roaming", L"Local"})
                require(CreateDirectoryW((path + L"\\" + child).c_str(), nullptr) != FALSE);
            HANDLE opened = CreateFileW(path.c_str(), FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY,
                FILE_SHARE_READ | FILE_SHARE_WRITE, &security, OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT, nullptr);
            require(opened != INVALID_HANDLE_VALUE);
            if (wcscmp(leaf, L"host-view") == 0) logical.value = opened;
            else if (wcscmp(leaf, L"client-one") == 0) a.value = opened;
            else b.value = opened;
        }
        for (const auto& folder : {L"Roaming", L"Local"}) {
            Handle file(CreateFileW((root + L"\\host-view\\" + folder + L"\\configured.bin").c_str(),
                GENERIC_WRITE, 0, nullptr, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, nullptr));
            require(file.value != INVALID_HANDLE_VALUE);
            writeBytes(file.value, "HOST");
        }
    }
};

struct ConfigView {
    education_bootstrap::Config* value;
    explicit ConfigView(HANDLE section) : value(static_cast<education_bootstrap::Config*>(
        MapViewOfFile(section, FILE_MAP_WRITE, 0, 0, sizeof(education_bootstrap::Config)))) {
        require(value != nullptr);
        ZeroMemory(value, sizeof(*value));
    }
    ~ConfigView() { UnmapViewOfFile(value); }
};

struct PresentationConfigView {
    presentation_enrollment::PolicyConfig* value;
    explicit PresentationConfigView(HANDLE section) : value(static_cast<presentation_enrollment::PolicyConfig*>(
        MapViewOfFile(section, FILE_MAP_WRITE, 0, 0, sizeof(*value)))) {
        require(value != nullptr); ZeroMemory(value, sizeof(*value));
    }
    ~PresentationConfigView() { UnmapViewOfFile(value); }
};
struct ConfiguredChild {
    Handle section{sharedSection()};
    Mapping receipt{section.value};
    Handle configSection{CreateFileMappingW(INVALID_HANDLE_VALUE, nullptr, PAGE_READWRITE,
        0, sizeof(education_bootstrap::Config), nullptr)};
    ConfigView config{configSection.value};
    Handle presentationSection{CreateFileMappingW(INVALID_HANDLE_VALUE, nullptr, PAGE_READWRITE,
        0, sizeof(presentation_enrollment::PolicyConfig), nullptr)};
    PresentationConfigView presentation{presentationSection.value};
    Handle presentationRead;
    Handle policyLevel{CreateEventW(nullptr, TRUE, FALSE, nullptr)};
    Handle policyChanged{CreateEventW(nullptr, FALSE, FALSE, nullptr)};
    Handle policyStop{CreateEventW(nullptr, TRUE, FALSE, nullptr)};
    Handle policyReaders[4];
    Child controller;
    Handle readonly;
    Handle ready{sharedEvent()}, done{sharedEvent()};
    Child child;
    ConfiguredChild(DWORD slot, DWORD round, ConfiguredRoots& roots, HKEY backing,
        RegistryHost& registry, const std::wstring& name, HANDLE go, HANDLE finish,
        const std::wstring& self, const std::string& compatibility, DWORD invalid = 0) {
        const HANDLE current = GetCurrentProcess();
        require(DuplicateHandle(current, configSection.value, current, &readonly.value,
            invalid == 5 ? FILE_MAP_READ | FILE_MAP_WRITE : FILE_MAP_READ, TRUE, 0) != FALSE);
        auto& status = *receipt.value;
        ZeroMemory(&status, sizeof(status));
        status.magic = probe::Magic; status.version = probe::Version;
        status.mode = probe::ConfiguredStorage; status.slot = slot; status.fileRound = round >= 6 ? 0 : round;
        status.registryBacking = 1; status.inheritedRegistry = backing;
        status.cachedHostKey = registry.branch.value;
        status.fileLogicalRoot = roots.logical.value;
        status.fileOwnRoot = slot == 1 ? roots.a.value : roots.b.value;
        status.ready = ready.value; status.done = done.value; status.go = go; status.finish = finish;
        copyName(status.logical, name);
        require(registry.userPath.size() < 512);
        memcpy(status.registryUserPath, registry.userPath.c_str(),
            (registry.userPath.size() + 1) * sizeof(wchar_t));
        auto& target = *config.value;
        target.magic = education_bootstrap::Magic; target.version = education_bootstrap::Version;
        target.bytes = sizeof(target); target.slot = slot;
        target.scope = education_bootstrap::StorageAndRegistryOnly;
        target.logicalRoot = status.fileLogicalRoot; target.privateRoot = status.fileOwnRoot;
        target.registryRoot = backing;
        require(ProcessIdToSessionId(GetCurrentProcessId(), &target.session) != FALSE);
        require(GetFileInformationByHandleEx(target.logicalRoot, FileIdInfo,
            &target.logicalId, sizeof(target.logicalId)) != FALSE
            && GetFileInformationByHandleEx(target.privateRoot, FileIdInfo,
                &target.privateId, sizeof(target.privateId)) != FALSE);
        memcpy(target.userKeyPath, status.registryUserPath, sizeof(target.userKeyPath));
        if (presentationControl) {
            require(DuplicateHandle(current, presentationSection.value, current, &presentationRead.value,
                invalid == 11 ? FILE_MAP_READ | FILE_MAP_WRITE : FILE_MAP_READ, TRUE, 0));
            auto& creation = presentation.value->creation;
            HMODULE image = nullptr;
            require(presentation_enrollment::imageMetadata(creation, image));
            Microsoft::WRL::ComPtr<IDXGIFactory2> factory;
            require(SUCCEEDED(CreateDXGIFactory2(0, IID_PPV_ARGS(&factory))));
            void** table = *reinterpret_cast<void***>(factory.Get());
            const auto rva = [&](void* entry) -> DWORD {
                MEMORY_BASIC_INFORMATION memory{};
                require(VirtualQuery(entry, &memory, sizeof(memory)) && memory.AllocationBase == image
                    && reinterpret_cast<ULONG_PTR>(entry) > reinterpret_cast<ULONG_PTR>(image));
                const auto offset = reinterpret_cast<ULONG_PTR>(entry) - reinterpret_cast<ULONG_PTR>(image);
                require(offset < creation.imageBytes);
                return static_cast<DWORD>(offset);
            };
            creation.oldRva = rva(table[10]); creation.hwndRva = rva(table[15]);
            creation.magic = presentation_enrollment::CreationMagic; creation.version = 1;
            creation.bytes = sizeof(creation); creation.session = target.session;
        }
        const bool paced = presentationControl && wcscmp(presentationControl, L"pace") == 0;
        std::vector<HANDLE> inherited{section.value, readonly.value, ready.value, done.value, go, finish,
            target.logicalRoot, target.privateRoot, backing, status.cachedHostKey};
        if (presentationControl) inherited.push_back(presentationRead.value);
        if (paced) {
            auto& policy = *presentation.value;
            policy.creation.version = 2; policy.creation.bytes = sizeof(policy);
            HANDLE controllerSource = current;
            if (round == 7) {
                STARTUPINFOW startup{sizeof(startup)};
                auto command = L"\"" + self + L"\" --bootstrap-observation-child";
                require(CreateProcessW(self.c_str(), command.data(), nullptr, nullptr, FALSE,
                    CREATE_SUSPENDED | CREATE_NO_WINDOW, nullptr, nullptr, &startup, &controller.info));
                controllerSource = controller.info.hProcess;
            }
            policy.controllerPid = GetProcessId(controllerSource);
            FILETIME exit{}, kernel{}, user{};
            require(GetProcessTimes(controllerSource, &policy.controllerCreated, &exit, &kernel, &user));
            Handle wrongLevel(CreateEventW(nullptr, FALSE, FALSE, nullptr));
            Handle wrongChanged(CreateEventW(nullptr, TRUE, FALSE, nullptr));
            Handle wrongStop(CreateEventW(nullptr, FALSE, FALSE, nullptr));
            const HANDLE source[]{invalid == 26 ? wrongLevel.value : policyLevel.value,
                invalid == 27 ? wrongChanged.value : policyChanged.value,
                invalid == 28 ? wrongStop.value : policyStop.value, controllerSource};
            HANDLE* output[]{&policy.level, &policy.changed, &policy.stop, &policy.controller};
            for (unsigned i = 0; i < 4; ++i) {
                DWORD access = SYNCHRONIZE | (i == 3 ? PROCESS_QUERY_LIMITED_INFORMATION : 1UL);
                if (i == 0 && invalid == 29) access |= EVENT_MODIFY_STATE;
                if (i == 0 && invalid == 37) access = SYNCHRONIZE;
                if (i == 3 && invalid == 38) access |= PROCESS_VM_READ;
                require(DuplicateHandle(current, source[i], current, &policyReaders[i].value, access, TRUE, 0));
                *output[i] = policyReaders[i].value;
                inherited.push_back(*output[i]);
            }
            if (invalid == 25) policy.level = nullptr;
            if (invalid == 30) policy.changed = policy.level;
            if (invalid == 31) ++policy.controllerCreated.dwLowDateTime;
            if (invalid == 32) ++policy.controllerPid;
            if (invalid == 33) policy.controller = GetCurrentProcess();
            if (invalid == 34) policy.reserved = 1;
            if (invalid == 35) policy.level = policy.controller;
            if (invalid == 36) require(SetEvent(policyStop.value));
            if (invalid == 39) policy.creation.version = 1;
        }
        Attributes attributes(inherited.data(), inherited.size());
        auto env = environment(section.value, target.privateRoot);
        env.pop_back();
        if (presentationControl) {
            const auto setting = std::wstring(presentation_enrollment::Variable) + L"=" + presentationControl;
            if (invalid != 24) { env.insert(env.end(), setting.begin(), setting.end()); env.push_back(0); }
            if (invalid != 14) {
                const auto capability = std::wstring(presentation_enrollment::SectionVariable) + L"="
                    + (invalid == 15 ? L"invalid" : invalid == 21 ? L"0"
                        : invalid == 22 ? L"18446744073709551616" : invalid == 23 ? L"18446744073709551616000000000000000"
                        : std::to_wstring(reinterpret_cast<ULONG_PTR>(presentationRead.value)));
                env.insert(env.end(), capability.begin(), capability.end()); env.push_back(0);
            }
        }
        const auto setting = std::wstring(education_bootstrap::SectionVariable) + L"="
            + std::to_wstring(reinterpret_cast<ULONG_PTR>(readonly.value));
        env.insert(env.end(), setting.begin(), setting.end());
        env.push_back(0); env.push_back(0);
        STARTUPINFOEXW startup{};
        startup.StartupInfo.cb = sizeof(startup); startup.lpAttributeList = attributes.value;
        std::wstring command = L"\"" + self + L"\" " + (round == 6 ? L"--bootstrap-policy-stop-child"
            : round == 7 ? L"--bootstrap-policy-death-child" : L"--bootstrap-test-child");
        require(CreateProcessW(self.c_str(), command.data(), nullptr, nullptr, TRUE,
            CREATE_SUSPENDED | CREATE_UNICODE_ENVIRONMENT | EXTENDED_STARTUPINFO_PRESENT | CREATE_NO_WINDOW,
            env.data(), nullptr, &startup.StartupInfo, &child.info) != FALSE);
        target.pid = status.pid = child.info.dwProcessId;
        FILETIME exit{}, kernel{}, user{};
        require(GetProcessTimes(child.info.hProcess, &target.created, &exit, &kernel, &user) != FALSE);
        status.created = target.created;
        if (presentationControl) {
            auto& creation = presentation.value->creation;
            creation.pid = target.pid; creation.created = target.created;
            if (invalid == 6) ++creation.pid;
            if (invalid == 7) ++creation.created.dwLowDateTime;
            if (invalid == 8) creation.imageId.FileId.Identifier[0] ^= 1;
            if (invalid == 9) creation.oldRva = 0;
            if (invalid == 10) creation.oldRva = creation.imageBytes;
            if (invalid == 12) creation.oldRva = creation.hwndRva;
            if (invalid == 13) ++creation.session;
            if (invalid == 16) ++creation.magic;
            if (invalid == 17) ++creation.version;
            if (invalid == 18) ++creation.bytes;
            if (invalid == 19) ++creation.imageTimestamp;
            if (invalid == 20) ++creation.imageBytes;
        }
        if (invalid == 1) ++target.pid;
        if (invalid == 2) target.privateId.FileId.Identifier[0] ^= 1;
        if (invalid == 3) target.slot = slot == 1 ? 2 : 1;
        if (invalid == 4) ++target.created.dwLowDateTime;
        const char* dlls[]{compatibility.c_str()};
        require(DetourUpdateProcessWithDll(child.info.hProcess, dlls, 1) != FALSE);
        MemoryBarrier();
        require(ResumeThread(child.info.hThread) == 1);
    }
    void phase(HANDLE event) {
        HANDLE states[]{event, child.info.hProcess};
        const DWORD outcome = WaitForMultipleObjects(2, states, FALSE, 10000);
        if (outcome == WAIT_OBJECT_0 + 1) {
            DWORD code = 0;
            require(GetExitCodeProcess(child.info.hProcess, &code));
            std::fprintf(stderr, "{\"controlChildExit\":%lu}\n", code);
        }
        require(outcome == WAIT_OBJECT_0);
        require(WaitForSingleObject(child.info.hProcess, 0) == WAIT_TIMEOUT);
    }
    void exited(bool success) {
        require(WaitForSingleObject(child.info.hProcess, 10000) == WAIT_OBJECT_0);
        DWORD code = 0;
        require(GetExitCodeProcess(child.info.hProcess, &code) != FALSE
            && (success ? code == 0 : code != 0));
        const auto& status = *receipt.value;
        require(status.armed == (success ? 1 : 0));
        if (success) require(status.count == 5 && status.earlyPrivate == 7
            && status.earlyAllowed == 0 && status.registryChecks == 65535 && status.nativeChecks == 1);
        else {
            require(status.count == 2 && status.events[0].stage == probe::Attach
                && status.events[1].stage == probe::Rejected && status.earlyPrivate == 0);
        }
    }
    void recoveryExited(DWORD expected) {
        require(WaitForSingleObject(child.info.hProcess, 10000) == WAIT_OBJECT_0);
        DWORD code = 0;
        require(GetExitCodeProcess(child.info.hProcess, &code) && code == expected);
        const auto& status = *receipt.value;
        require(status.armed == 1 && status.count == 5 && status.earlyPrivate == 7
            && status.nativeChecks == 1);
    }
};

// Same native mutation workflow runs unmediated and in both private slots.
// All names/bytes are generated canaries; no game database is opened.
void nativeStorageMutationChecks(const std::wstring& base, bool mediated = false) {
    using SetInformation = NTSTATUS (NTAPI*)(HANDLE, PIO_STATUS_BLOCK, PVOID, ULONG, FILE_INFORMATION_CLASS);
    const auto set = reinterpret_cast<SetInformation>(GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "NtSetInformationFile"));
    require(set != nullptr);
    constexpr DWORD sharing = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
    constexpr DWORD sentinel = 0x13579bdf;
    const auto read = [](const std::wstring& path, const std::string& bytes) {
        Handle file(CreateFileW(path.c_str(), GENERIC_READ, sharing, nullptr, OPEN_EXISTING, 0, nullptr));
        expectBytes(file.value, bytes);
    };
    const auto check = [](NTSTATUS actual, NTSTATUS expected) {
        if (actual != expected) std::printf("{\"mutationStatus\":%lu,\"expected\":%lu}\n",
            static_cast<ULONG>(actual), static_cast<ULONG>(expected));
        require(actual == expected && GetLastError() == sentinel);
    };
    for (ULONG kind : {10UL, 65UL}) for (unsigned form = 0; form < 3; ++form) {
        const auto sourcePath = base + L"\\mutation-source.bin";
        const auto targetPath = base + L"\\mutation-target.bin";
        {
            Handle target(CreateFileW(targetPath.c_str(), GENERIC_WRITE, sharing,
                nullptr, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, nullptr));
            require(target.value != INVALID_HANDLE_VALUE);
            writeBytes(target.value, "OLD-COMMITTED");
        }
        {
            Handle source(CreateFileW(sourcePath.c_str(), GENERIC_READ | GENERIC_WRITE | DELETE,
                sharing, nullptr, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, nullptr));
            require(source.value != INVALID_HANDLE_VALUE);
            writeBytes(source.value, "NEW-CHECKPOINT");
            FILE_ID_INFO sourceId{};
            require(GetFileInformationByHandleEx(source.value, FileIdInfo, &sourceId, sizeof(sourceId)));
            Handle directory(CreateFileW(base.c_str(), FILE_TRAVERSE | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
                sharing, nullptr, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, nullptr));
            if (directory.value == INVALID_HANDLE_VALUE) std::printf("{\"traverseDirectoryOpenError\":%lu}\n", GetLastError());
            require(directory.value != INVALID_HANDLE_VALUE);
            if (mediated) {
                // The same ambiguous access on an ordinary file must remain refused.
                Handle execute(CreateFileW(sourcePath.c_str(), FILE_EXECUTE | FILE_READ_ATTRIBUTES,
                    sharing, nullptr, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, nullptr));
                require(execute.value == INVALID_HANDLE_VALUE);
                expectBytes(source.value, "NEW-CHECKPOINT");
                const auto absent = base + L"\\execute-create.bin";
                Handle create(CreateFileW(absent.c_str(), FILE_EXECUTE, sharing, nullptr, CREATE_NEW,
                    FILE_FLAG_BACKUP_SEMANTICS, nullptr));
                require(create.value == INVALID_HANDLE_VALUE
                    && GetFileAttributesW(absent.c_str()) == INVALID_FILE_ATTRIBUTES
                    && GetLastError() == ERROR_FILE_NOT_FOUND);
            }
            std::wstring nativeBase = L"\\??\\" + base;
            auto baseName = nativeName(nativeBase);
            OBJECT_ATTRIBUTES openAttributes{sizeof(OBJECT_ATTRIBUTES), nullptr, &baseName,
                OBJ_CASE_INSENSITIVE, nullptr, nullptr};
            const auto open = reinterpret_cast<probe::NativeFileOpen>(
                GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "NtOpenFile"));
            Handle reopened;
            IO_STATUS_BLOCK opened{};
            SetLastError(sentinel);
            require(open != nullptr);
            check(open(&reopened.value, FILE_TRAVERSE | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
                &openAttributes, &opened, sharing, FILE_SYNCHRONOUS_IO_NONALERT), 0);
            std::wstring name = form == 2 ? targetPath : L"mutation-target.bin";
            if (form == 2) { // Ordinary NT DOS absolute name, not a Win32 path.
                require(name.size() > 2 && name[1] == L':');
                name = L"\\??\\" + name;
            }
            struct Rename { ULONG flags; HANDLE root; ULONG bytes; wchar_t name[512]; } info{};
            require(name.size() < 512);
            info.root = form == 1 ? directory.value : nullptr;
            info.bytes = static_cast<ULONG>(name.size() * sizeof(wchar_t));
            memcpy(info.name, name.data(), info.bytes);
            const auto rename = [&](ULONG flags, NTSTATUS expected, HANDLE object = nullptr) {
                info.flags = flags;
                const auto before = info;
                IO_STATUS_BLOCK io{};
                SetLastError(sentinel);
                const auto status = set(object ? object : source.value, &io, &info,
                    static_cast<ULONG>(offsetof(Rename, name) + info.bytes), static_cast<FILE_INFORMATION_CLASS>(kind));
                if (status != expected) std::printf("{\"renameClass\":%lu,\"form\":%u,\"flags\":%lu}\n", kind, form, flags);
                check(status, expected);
                require(memcmp(&before, &info, sizeof(info)) == 0);
            };
            {
                Handle limited(CreateFileW(sourcePath.c_str(), GENERIC_READ, sharing, nullptr, OPEN_EXISTING, 0, nullptr));
                require(limited.value != INVALID_HANDLE_VALUE);
                rename(1, static_cast<NTSTATUS>(0xc0000022), limited.value);
                expectBytes(limited.value, "NEW-CHECKPOINT"); read(targetPath, "OLD-COMMITTED");
            }
            rename(0, static_cast<NTSTATUS>(0xc0000035)); // Collision keeps both generations.
            expectBytes(source.value, "NEW-CHECKPOINT"); read(targetPath, "OLD-COMMITTED");
            require(SetFileAttributesW(targetPath.c_str(), FILE_ATTRIBUTE_READONLY));
            rename(1, static_cast<NTSTATUS>(0xc0000022));
            expectBytes(source.value, "NEW-CHECKPOINT"); read(targetPath, "OLD-COMMITTED");
            require(SetFileAttributesW(targetPath.c_str(), FILE_ATTRIBUTE_NORMAL));
            {
                Handle busy(CreateFileW(targetPath.c_str(), GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE,
                    nullptr, OPEN_EXISTING, 0, nullptr));
                require(busy.value != INVALID_HANDLE_VALUE);
                rename(1, static_cast<NTSTATUS>(0xc0000022));
                expectBytes(busy.value, "OLD-COMMITTED"); expectBytes(source.value, "NEW-CHECKPOINT");
            }
            {
                Handle snapshot(CreateFileW(targetPath.c_str(), GENERIC_READ, sharing, nullptr, OPEN_EXISTING, 0, nullptr));
                require(snapshot.value != INVALID_HANDLE_VALUE);
                // Classic replacement refuses an open target. Ex POSIX can preserve its old reader.
                rename(kind == 65 ? 3 : 1, kind == 65 ? 0 : static_cast<NTSTATUS>(0xc0000022));
                expectBytes(snapshot.value, "OLD-COMMITTED");
            }
            if (kind == 10) rename(1, 0);
            read(targetPath, "NEW-CHECKPOINT");
            Handle replacement(CreateFileW(targetPath.c_str(), GENERIC_READ, sharing, nullptr, OPEN_EXISTING, 0, nullptr));
            require(replacement.value != INVALID_HANDLE_VALUE);
            FILE_ID_INFO renamed{};
            require(GetFileInformationByHandleEx(replacement.value, FileIdInfo, &renamed, sizeof(renamed))
                && memcmp(&sourceId, &renamed, sizeof(sourceId)) == 0);
            require(GetFileAttributesW(sourcePath.c_str()) == INVALID_FILE_ATTRIBUTES
                && GetLastError() == ERROR_FILE_NOT_FOUND);
            // Delete/cancel must follow Windows, for both classic and Ex forms.
            for (ULONG flags : {1UL, 0UL}) {
                IO_STATUS_BLOCK io{};
                SetLastError(sentinel);
                check(set(source.value, &io, &flags, kind == 10 ? 1 : sizeof(flags),
                    static_cast<FILE_INFORMATION_CLASS>(kind == 10 ? 13 : 64)), 0);
            }
        }
        read(targetPath, "NEW-CHECKPOINT"); // Cancellation survived original handle closure.
        for (bool allowed : {false, true}) {
            Handle target(CreateFileW(targetPath.c_str(), GENERIC_READ | (allowed ? DELETE : 0),
                sharing, nullptr, OPEN_EXISTING, 0, nullptr));
            require(target.value != INVALID_HANDLE_VALUE);
            ULONG flags = 1;
            IO_STATUS_BLOCK io{};
            SetLastError(sentinel);
            check(set(target.value, &io, &flags, kind == 10 ? 1 : sizeof(flags),
                static_cast<FILE_INFORMATION_CLASS>(kind == 10 ? 13 : 64)),
                allowed ? 0 : static_cast<NTSTATUS>(0xc0000022));
            expectBytes(target.value, "NEW-CHECKPOINT");
        }
        require(GetFileAttributesW(targetPath.c_str()) == INVALID_FILE_ATTRIBUTES
            && GetLastError() == ERROR_FILE_NOT_FOUND);
    }
}

void worldStorageChecks(probe::Receipt& receipt) {
    configuredDiagnosticChild = true;
    auto base = directoryPath(receipt.fileLogicalRoot).substr(4);
    for (const auto* part : {L"Roaming", L"Minecraft Education Edition", L"games", L"com.mojang",
                             L"minecraftWorlds", L"world_A+=", L"db"}) {
        base += std::wstring(L"\\") + part;
        if (!receipt.fileRound) require(CreateDirectoryW(base.c_str(), nullptr)
            || (wcscmp(part, L"Roaming") == 0 && GetLastError() == ERROR_ALREADY_EXISTS));
        else {
            const DWORD attributes = GetFileAttributesW(base.c_str());
            require(attributes != INVALID_FILE_ATTRIBUTES && (attributes & FILE_ATTRIBUTE_DIRECTORY));
        }
    }
    nativeStorageMutationChecks(base, true);
    const auto temporary = base + L"\\.manifest.tmp";
    const auto current = base + L"\\CURRENT";
    const auto marker = std::to_string(receipt.slot) + "-" + std::to_string(receipt.fileRound + 1);
    if (receipt.fileRound) expectFile(current, std::to_string(receipt.slot) + "-1");
    {
        Handle output(CreateFileW(temporary.c_str(), GENERIC_READ | GENERIC_WRITE | DELETE,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, nullptr, CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL, nullptr));
        require(output.value != INVALID_HANDLE_VALUE);
        writeBytes(output.value, marker);
        require(FlushFileBuffers(output.value));
    }
    require(MoveFileExW(temporary.c_str(), current.c_str(), MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH));
    expectFile(current, marker);
    // A failed database checkpoint must preserve both the committed and retryable data.
    const auto retry = base + L"\\.retry.tmp";
    {
        Handle checkpoint(CreateFileW(retry.c_str(), GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ, nullptr, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, nullptr));
        require(checkpoint.value != INVALID_HANDLE_VALUE);
        writeBytes(checkpoint.value, marker + "-retry");
    }
    {
        Handle reader(CreateFileW(current.c_str(), GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr));
        require(reader.value != INVALID_HANDLE_VALUE);
        const BOOL replaced = MoveFileExW(retry.c_str(), current.c_str(), MOVEFILE_REPLACE_EXISTING);
        const DWORD replaceError = GetLastError();
        // Matches the unhooked Windows replacement result for a target without SHARE_DELETE.
        require(!replaced && replaceError == ERROR_ACCESS_DENIED);
        expectBytes(reader.value, marker);
        expectFile(current, marker);
        expectFile(retry, marker + "-retry");
        Handle collision(CreateFileW(current.c_str(), GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, nullptr));
        require(collision.value == INVALID_HANDLE_VALUE && GetLastError() == ERROR_FILE_EXISTS);
        expectBytes(reader.value, marker);
    }
    require(MoveFileExW(retry.c_str(), current.c_str(), MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH));
    expectFile(current, marker + "-retry");
    require(GetFileAttributesW(retry.c_str()) == INVALID_FILE_ATTRIBUTES
        && GetLastError() == ERROR_FILE_NOT_FOUND);
    // Restore the same per-round marker for the independent controller's backing check.
    {
        Handle checkpoint(CreateFileW(temporary.c_str(), GENERIC_WRITE, FILE_SHARE_READ,
            nullptr, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, nullptr));
        require(checkpoint.value != INVALID_HANDLE_VALUE);
        writeBytes(checkpoint.value, marker);
    }
    require(MoveFileExW(temporary.c_str(), current.c_str(), MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH));
    WIN32_FIND_DATAW found{};
    const HANDLE search = FindFirstFileW((base + L"\\*").c_str(), &found);
    require(search != INVALID_HANDLE_VALUE);
    bool currentFound = false;
    do { if (wcscmp(found.cFileName, L"CURRENT") == 0) currentFound = true; }
    while (FindNextFileW(search, &found));
    const DWORD searchError = GetLastError();
    require(FindClose(search) && searchError == ERROR_NO_MORE_FILES && currentFound);
    for (DWORD rights : {static_cast<DWORD>(WRITE_OWNER), static_cast<DWORD>(WRITE_DAC)}) {
        Handle security(CreateFileW(base.c_str(), rights, FILE_SHARE_READ | FILE_SHARE_WRITE,
            nullptr, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT, nullptr));
        require(security.value != INVALID_HANDLE_VALUE);
    }
    WIN32_FILE_ATTRIBUTE_DATA metadata{};
    require(GetFileAttributesExW(current.c_str(), GetFileExInfoStandard, &metadata)
        && metadata.nFileSizeHigh == 0 && metadata.nFileSizeLow == marker.size());
    const auto ownCurrent = directoryPath(receipt.fileOwnRoot).substr(4)
        + base.substr(directoryPath(receipt.fileLogicalRoot).size() - 4) + L"\\CURRENT";
    WIN32_FILE_ATTRIBUTE_DATA backingMetadata{};
    require(GetFileAttributesExW(ownCurrent.c_str(), GetFileExInfoStandard, &backingMetadata)
        && memcmp(&metadata, &backingMetadata, sizeof(metadata)) == 0);
    const DWORD rootAttributes = GetFileAttributesW(directoryPath(receipt.fileLogicalRoot).c_str());
    require(rootAttributes != INVALID_FILE_ATTRIBUTES && (rootAttributes & FILE_ATTRIBUTE_DIRECTORY));
    const auto unicodeName = base + L"\\\u4e16\u754c (A)+=.bin";
    {
        Handle file(CreateFileW(unicodeName.c_str(), GENERIC_ALL,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, nullptr, CREATE_NEW, 0, nullptr));
        require(file.value != INVALID_HANDLE_VALUE);
        writeBytes(file.value, marker);
    }
    expectFile(unicodeName, marker);
    require(DeleteFileW(unicodeName.c_str()));
    const auto rename = reinterpret_cast<NTSTATUS (NTAPI*)(HANDLE, PIO_STATUS_BLOCK, PVOID, ULONG, FILE_INFORMATION_CLASS)>(
        GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "NtSetInformationFile"));
    require(rename != nullptr);
    {
        Handle file(CreateFileW(current.c_str(), GENERIC_READ | DELETE,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, nullptr, OPEN_EXISTING, 0, nullptr));
        require(file.value != INVALID_HANDLE_VALUE);
        struct Rename { ULONG flags; HANDLE root; ULONG bytes; wchar_t name[32]; } info{};
        const wchar_t next[] = L"CURRENT.next";
        memcpy(info.name, next, sizeof(next)); info.bytes = sizeof(next) - 2;
        IO_STATUS_BLOCK io{};
        constexpr DWORD sentinel = 0x13579bd;
        SetLastError(sentinel);
        require(rename(file.value, &io, &info, static_cast<ULONG>(offsetof(Rename, name) + info.bytes),
            static_cast<FILE_INFORMATION_CLASS>(10)) == 0 && GetLastError() == sentinel);
    }
    require(MoveFileExW((base + L"\\CURRENT.next").c_str(), current.c_str(), MOVEFILE_REPLACE_EXISTING));
    expectFile(current, marker);
    const auto disposable = base + L"\\disposable.tmp";
    {
        Handle file(CreateFileW(disposable.c_str(), GENERIC_READ | GENERIC_WRITE | DELETE,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, nullptr, CREATE_NEW,
            FILE_FLAG_DELETE_ON_CLOSE, nullptr));
        require(file.value != INVALID_HANDLE_VALUE);
    }
    require(GetFileAttributesW(disposable.c_str()) == INVALID_FILE_ATTRIBUTES && GetLastError() == ERROR_FILE_NOT_FOUND);
    const auto deletable = base + L"\\delete-me.tmp";
    { Handle file(CreateFileW(deletable.c_str(), GENERIC_WRITE, 0, nullptr, CREATE_NEW, 0, nullptr));
      require(file.value != INVALID_HANDLE_VALUE); }
    require(DeleteFileW(deletable.c_str()));
    // Ordinary data names and filesystem queries must behave like the backing directory.
    ULARGE_INTEGER available{}, total{}, free{};
    require(GetDiskFreeSpaceExW(base.c_str(), &available, &total, &free) && total.QuadPart > 0);
    const auto source = base + L"\\escape.tmp";
    { Handle file(CreateFileW(source.c_str(), GENERIC_WRITE, 0, nullptr, CREATE_NEW, 0, nullptr));
      require(file.value != INVALID_HANDLE_VALUE); }
    const auto outside = directoryPath(receipt.fileLogicalRoot).substr(4) + L"-outside.tmp";
    require(!MoveFileExW(source.c_str(), outside.c_str(), MOVEFILE_REPLACE_EXISTING)
        && GetFileAttributesW(outside.c_str()) == INVALID_FILE_ATTRIBUTES);
    require(!CreateHardLinkW(outside.c_str(), current.c_str(), nullptr)
        && GetFileAttributesW(outside.c_str()) == INVALID_FILE_ATTRIBUTES);
    require(DeleteFileW(source.c_str()));
    for (unsigned iteration = 0; iteration < 16; ++iteration) {
        { Handle checkpoint(CreateFileW(temporary.c_str(), GENERIC_READ | GENERIC_WRITE | DELETE,
              FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, nullptr, CREATE_NEW, 0, nullptr));
          require(checkpoint.value != INVALID_HANDLE_VALUE);
          writeBytes(checkpoint.value, marker); require(FlushFileBuffers(checkpoint.value)); }
        require(MoveFileExW(temporary.c_str(), current.c_str(), MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH));
        expectFile(current, marker);
    }
    require(!RemoveDirectoryW(directoryPath(receipt.fileLogicalRoot).c_str()));
}

int interruptedStorageChild(probe::Receipt& receipt) {
    configuredDiagnosticChild = true;
    const auto base = directoryPath(receipt.fileLogicalRoot).substr(4)
        + L"\\Roaming\\Minecraft Education Edition\\games\\com.mojang\\minecraftWorlds\\world_A+=\\db";
    const auto current = base + L"\\recovery-current";
    const auto retry = base + L"\\.recovery-retry.tmp";
    const auto committed = std::to_string(receipt.slot) + "-committed";
    const auto pending = std::to_string(receipt.slot) + "-pending";
    if (receipt.fileRound == 2) {
        Handle original(CreateFileW(current.c_str(), GENERIC_WRITE, FILE_SHARE_READ,
            nullptr, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, nullptr));
        require(original.value != INVALID_HANDLE_VALUE);
        writeBytes(original.value, committed);
    }
    expectFile(current, committed);
    {
        Handle checkpoint(CreateFileW(retry.c_str(), GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr,
            receipt.fileRound == 2 ? CREATE_NEW : OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr));
        require(checkpoint.value != INVALID_HANDLE_VALUE);
        if (receipt.fileRound == 2) writeBytes(checkpoint.value, pending);
        else expectBytes(checkpoint.value, pending);
        // Parent interrupts only its own first child here, before close or replacement.
        require(SetEvent(receipt.ready));
        require(WaitForSingleObject(receipt.go, 10000) == WAIT_OBJECT_0);
    }
    expectFile(current, committed);
    expectFile(retry, pending);
    require(MoveFileExW(retry.c_str(), current.c_str(), MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH));
    expectFile(current, pending);
    require(GetFileAttributesW(retry.c_str()) == INVALID_FILE_ATTRIBUTES
        && GetLastError() == ERROR_FILE_NOT_FOUND);
    require(SetEvent(receipt.done));
    require(WaitForSingleObject(receipt.finish, 10000) == WAIT_OBJECT_0);
    return 0;
}

std::string mappedMarker(DWORD slot) {
    std::string bytes(16, '\0');
    bytes.replace(0, 8, std::to_string(slot) + "-mapped");
    return bytes;
}

int mappedStorageChild(probe::Receipt& receipt) {
    configuredDiagnosticChild = true;
    const auto base = directoryPath(receipt.fileLogicalRoot).substr(4)
        + L"\\Roaming\\Minecraft Education Edition\\games\\com.mojang\\minecraftWorlds\\world_A+=\\db";
    const auto current = base + L"\\mapping-current";
    const auto retry = base + L"\\.mapping-retry.tmp";
    const auto expected = mappedMarker(receipt.slot);
    if (receipt.fileRound == 4) {
        Handle previous(CreateFileW(current.c_str(), GENERIC_WRITE, FILE_SHARE_READ,
            nullptr, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, nullptr));
        require(previous.value != INVALID_HANDLE_VALUE);
        writeBytes(previous.value, "previous");
    } else {
        expectFile(current, expected);
        require(GetFileAttributesW(retry.c_str()) == INVALID_FILE_ATTRIBUTES
            && GetLastError() == ERROR_FILE_NOT_FOUND);
    }
    {
        Handle checkpoint(CreateFileW((receipt.fileRound == 4 ? retry : current).c_str(),
            GENERIC_READ | GENERIC_WRITE | DELETE,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, nullptr,
            receipt.fileRound == 4 ? CREATE_NEW : OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr));
        require(checkpoint.value != INVALID_HANDLE_VALUE);
        if (receipt.fileRound == 4) {
            LARGE_INTEGER size{}; size.QuadPart = expected.size();
            require(SetFilePointerEx(checkpoint.value, size, nullptr, FILE_BEGIN) && SetEndOfFile(checkpoint.value));
        }
        Handle mapping(CreateFileMappingW(checkpoint.value, nullptr, PAGE_READWRITE, 0, 0, nullptr));
        require(mapping.value != nullptr);
        struct View {
            void* value;
            ~View() { if (value) UnmapViewOfFile(value); }
        } view{MapViewOfFile(mapping.value, FILE_MAP_READ | FILE_MAP_WRITE, 0, 0, expected.size())};
        require(view.value != nullptr);
        if (receipt.fileRound == 4) {
            memcpy(view.value, expected.data(), expected.size());
            require(FlushViewOfFile(view.value, expected.size()) && FlushFileBuffers(checkpoint.value));
            require(MoveFileExW(retry.c_str(), current.c_str(), MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH));
        } else require(memcmp(view.value, expected.data(), expected.size()) == 0);
        OVERLAPPED region{}; region.Offset = 8;
        require(LockFileEx(checkpoint.value, LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
            0, 1, 0, &region));
        // Interrupt only the owned writer after rename, while its mapping and byte lock remain live.
        require(SetEvent(receipt.ready));
        require(WaitForSingleObject(receipt.go, 10000) == WAIT_OBJECT_0);
        require(UnlockFileEx(checkpoint.value, 0, 1, 0, &region));
    }
    expectFile(current, expected);
    require(SetEvent(receipt.done));
    require(WaitForSingleObject(receipt.finish, 10000) == WAIT_OBJECT_0);
    return 0;
}

int helperViewMain() {
    probe::View report;
    require(report.open() && report.value->mode == probe::Baseline);
    auto& status = *report.value;
    const bool bound = status.registryBacking == 1;
    require((GetModuleHandleW(L"bootstrap-compat.dll") != nullptr) == bound
        && education_bootstrap::present() == bound);
    const auto privatePath = directoryPath(status.fileOwnRoot).substr(4);
    LONG fileChecks = 0;
    for (const auto& folder : {L"Roaming", L"Local"}) {
        const bool roaming = wcscmp(folder, L"Roaming") == 0;
        const auto expected = privatePath + L"\\" + folder;
        require(childEnvironment(roaming ? L"APPDATA" : L"LOCALAPPDATA") == expected);
        fileChecks |= roaming ? 1 : 2;
        PWSTR known = nullptr;
        require(SUCCEEDED(SHGetKnownFolderPath(roaming ? FOLDERID_RoamingAppData : FOLDERID_LocalAppData,
            KF_FLAG_DONT_VERIFY, nullptr, &known)) && known);
        const bool matches = _wcsicmp(known, expected.c_str()) == 0;
        CoTaskMemFree(known);
        require(matches == bound);
        fileChecks |= roaming ? 4 : 8;
    }
    // Read only the freshly generated host canary, never an account/authentication key.
    Key direct, current, relative;
    const auto path = registryPath(status.logical);
    const DWORD expectedMarker = bound ? registryMarker(status) : 0x484f5354;
    require(RegOpenKeyExW(HKEY_CURRENT_USER, path.c_str(), 0, KEY_READ, &direct.value) == ERROR_SUCCESS
        && registryValue(direct.value, L"Marker") == expectedMarker);
    require(RegOpenCurrentUser(KEY_READ, &current.value) == ERROR_SUCCESS
        && RegOpenKeyExW(current.value, path.c_str(), 0, KEY_READ, &relative.value) == ERROR_SUCCESS
        && registryValue(relative.value, L"Marker") == expectedMarker);
    InterlockedExchange(&status.fileChecks, fileChecks);
    InterlockedExchange(&status.registryChecks, 3);
    return 0;
}

void helperBindingCheck(probe::Receipt& receipt, bool bound) {
    Handle section(sharedSection());
    Mapping report(section.value);
    ZeroMemory(report.value, sizeof(probe::Receipt));
    report.value->magic = probe::Magic; report.value->version = probe::Version;
    report.value->mode = probe::Baseline; report.value->slot = receipt.slot;
    report.value->registryBacking = bound ? 1 : 0; report.value->fileRound = receipt.fileRound;
    copyName(report.value->logical, receipt.logical);
    Handle privateRoot;
    require(DuplicateHandle(GetCurrentProcess(), receipt.fileOwnRoot, GetCurrentProcess(),
        &privateRoot.value, 0, TRUE, DUPLICATE_SAME_ACCESS));
    report.value->fileOwnRoot = privateRoot.value;
    Handle configSection(CreateFileMappingW(INVALID_HANDLE_VALUE, nullptr, PAGE_READWRITE,
        0, sizeof(education_bootstrap::Config), nullptr));
    require(configSection.value != nullptr);
    ConfigView config(configSection.value);
    Handle readonly, logicalRoot;
    Key registryRoot;
    std::vector<HANDLE> inherited{section.value, privateRoot.value};
    if (bound) {
        // Test-only explicit bootstrap of this fixed own executable; never a child-discovery hook.
        require(education_bootstrap::read(*config.value));
        require(config.value->scope == education_bootstrap::StorageAndRegistryOnly
            && config.value->singletonCallerRva == 0);
        require(DuplicateHandle(GetCurrentProcess(), config.value->logicalRoot, GetCurrentProcess(),
            &logicalRoot.value, 0, TRUE, DUPLICATE_SAME_ACCESS));
        require(DuplicateHandle(GetCurrentProcess(), config.value->registryRoot, GetCurrentProcess(),
            reinterpret_cast<PHANDLE>(&registryRoot.value), 0, TRUE, DUPLICATE_SAME_ACCESS));
        require(DuplicateHandle(GetCurrentProcess(), configSection.value, GetCurrentProcess(),
            &readonly.value, FILE_MAP_READ, TRUE, 0));
        config.value->logicalRoot = logicalRoot.value; config.value->privateRoot = privateRoot.value;
        config.value->registryRoot = registryRoot.value;
        inherited.push_back(logicalRoot.value); inherited.push_back(registryRoot.value);
        inherited.push_back(readonly.value);
    }
    Attributes attributes(inherited.data(), inherited.size());
    // Preserve the private environment; strip the parent's process-bound bootstrap capabilities.
    auto env = environment(section.value);
    if (bound) {
        env.pop_back();
        const auto setting = std::wstring(education_bootstrap::SectionVariable) + L"="
            + std::to_wstring(reinterpret_cast<ULONG_PTR>(readonly.value));
        env.insert(env.end(), setting.begin(), setting.end());
        env.push_back(0); env.push_back(0);
    }
    const auto self = executable();
    std::wstring command = L"\"" + self + L"\" --bootstrap-helper-view";
    STARTUPINFOEXW startup{};
    startup.StartupInfo.cb = sizeof(startup); startup.lpAttributeList = attributes.value;
    Child helper;
    require(CreateProcessW(self.c_str(), command.data(), nullptr, nullptr, TRUE,
        CREATE_SUSPENDED | CREATE_UNICODE_ENVIRONMENT | EXTENDED_STARTUPINFO_PRESENT | CREATE_NO_WINDOW,
        env.data(), nullptr, &startup.StartupInfo, &helper.info));
    report.value->pid = helper.info.dwProcessId;
    FILETIME exited{}, kernel{}, user{};
    require(GetProcessTimes(helper.info.hProcess, &report.value->created, &exited, &kernel, &user));
    if (bound) {
        config.value->pid = helper.info.dwProcessId; config.value->created = report.value->created;
        const auto separator = self.find_last_of(L"\\/");
        require(separator != std::wstring::npos);
        struct Library {
            HMODULE value;
            ~Library() { if (value && !FreeLibrary(value)) cleanupFailed = true; }
        } library{LoadLibraryExW((self.substr(0, separator + 1) + L"bootstrap-enroll.dll").c_str(), nullptr,
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32)};
        require(library.value != nullptr);
        using Enroll = DWORD (WINAPI*)(HANDLE, HANDLE, DWORD, ULONGLONG, HANDLE, HANDLE, HANDLE);
        const auto enroll = reinterpret_cast<Enroll>(GetProcAddress(library.value, "EnrollPreparedEducationV2"));
        require(enroll != nullptr);
        Handle image(CreateFileW(self.c_str(), GENERIC_READ, FILE_SHARE_READ, nullptr, OPEN_EXISTING,
            FILE_FLAG_OPEN_REPARSE_POINT, nullptr));
        require(image.value != INVALID_HANDLE_VALUE);
        const ULONGLONG birth = (static_cast<ULONGLONG>(report.value->created.dwHighDateTime) << 32)
            | report.value->created.dwLowDateTime;
        Key callerUser;
        require(RegOpenCurrentUser(KEY_READ, &callerUser.value) == ERROR_SUCCESS
            && nativeKeyName(callerUser.value) != config.value->userKeyPath);
        // This API belongs to the unmediated manager; a mediated caller must remain refused.
        require(enroll(helper.info.hProcess, helper.info.hThread, helper.info.dwProcessId, birth,
            readonly.value, image.value, nullptr) == ERROR_INVALID_DATA);
        // Separate low-level positive control on this own fixture only; never a production fallback.
        std::string compatibility;
        for (const auto c : self.substr(0, separator + 1) + L"bootstrap-compat.dll") {
            require(c > 0 && c <= 0x7f); compatibility.push_back(static_cast<char>(c));
        }
        const char* dlls[]{compatibility.c_str()};
        require(DetourUpdateProcessWithDll(helper.info.hProcess, dlls, 1));
    }
    MemoryBarrier();
    require(ResumeThread(helper.info.hThread) == 1);
    require(WaitForSingleObject(helper.info.hProcess, 10000) == WAIT_OBJECT_0);
    DWORD code = STILL_ACTIVE;
    require(GetExitCodeProcess(helper.info.hProcess, &code) && code == 0);
    require(!cleanupFailed);
    require(report.value->count == 2 && report.value->armed == 0
        && report.value->fileChecks == 15 && report.value->registryChecks == 3);
    // The enrolled parent still has its own effective Known Folder and HKCU after its helper exits.
    const auto own = directoryPath(receipt.fileOwnRoot).substr(4);
    require(knownPath(FOLDERID_RoamingAppData) == own + L"\\Roaming"
        && knownPath(FOLDERID_LocalAppData) == own + L"\\Local");
    Key parent;
    require(RegOpenKeyExW(HKEY_CURRENT_USER, registryPath(receipt.logical).c_str(), 0,
        KEY_READ, &parent.value) == ERROR_SUCCESS
        && registryValue(parent.value, L"Marker") != 0x484f5354);
}

void presentationChecks(probe::Receipt& receipt) {
    using namespace Microsoft::WRL;
    using Read = BOOL (*)(presentation_enrollment::Snapshot*);
    const auto read = reinterpret_cast<Read>(GetProcAddress(GetModuleHandleW(L"bootstrap-compat.dll"),
        "BootstrapPresentationSnapshot"));
    presentation_enrollment::Snapshot observed{}; observed.bytes = sizeof(observed);
    require(read && read(&observed));
    wchar_t setting[16]{};
    if (!GetEnvironmentVariableW(presentation_enrollment::Variable, setting, 16)) {
        require(!observed.armed && !observed.factories && !observed.candidates && !observed.failures);
        return;
    }
    require(observed.armed == 1 && observed.factories == 0 && observed.candidates == 0);
    struct Window {
        HWND value;
        explicit Window(const wchar_t* name, int size = 16) {
            WNDCLASSW type{}; type.lpfnWndProc = DefWindowProcW;
            type.hInstance = GetModuleHandleW(nullptr); type.lpszClassName = name;
            require(RegisterClassW(&type));
            value = CreateWindowExW(WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW, name, L"", WS_POPUP,
                0, 0, size, size, nullptr, nullptr, type.hInstance, nullptr);
            require(value && !IsWindowVisible(value));
        }
        ~Window() { DestroyWindow(value); }
    } selected(L"OGLES", policyControlRound == 8 ? 900 : 16), unrelated(L"MivubiPresentationAux");
    ComPtr<IDXGIFactory4> factory;
    require(SUCCEEDED(CreateDXGIFactory2(0, IID_PPV_ARGS(&factory))));
    require(read(&observed) && observed.factories == 1 && observed.failures == 0);
    ComPtr<IDXGIFactory> legacy;
    ComPtr<IDXGIFactory1> version1;
    require(SUCCEEDED(CreateDXGIFactory(IID_PPV_ARGS(&legacy)))
        && SUCCEEDED(CreateDXGIFactory1(IID_PPV_ARGS(&version1)))
        && read(&observed) && observed.factories == 3 && observed.failures == 0);
    ComPtr<IDXGIAdapter> warp;
    ComPtr<ID3D12Device> device;
    ComPtr<ID3D12CommandQueue> queue;
    require(SUCCEEDED(factory->EnumWarpAdapter(IID_PPV_ARGS(&warp)))
        && SUCCEEDED(D3D12CreateDevice(warp.Get(), D3D_FEATURE_LEVEL_11_0, IID_PPV_ARGS(&device))));
    D3D12_COMMAND_QUEUE_DESC command{}; command.Type = D3D12_COMMAND_LIST_TYPE_DIRECT;
    require(SUCCEEDED(device->CreateCommandQueue(&command, IID_PPV_ARGS(&queue))));
    DXGI_SWAP_CHAIN_DESC1 description{};
    description.Width = description.Height = 16;
    description.Format = DXGI_FORMAT_R8G8B8A8_UNORM; description.SampleDesc.Count = 1;
    description.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
    description.BufferCount = 2; description.SwapEffect = DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL;
    const auto create = [&](HWND window) {
        ComPtr<IDXGISwapChain1> chain;
        const auto before = description;
        require(SUCCEEDED(factory->CreateSwapChainForHwnd(queue.Get(), window, &description,
            nullptr, nullptr, &chain)) && memcmp(&before, &description, sizeof(before)) == 0);
        return chain;
    };
    auto foreign = create(unrelated.value);
    require(read(&observed) && observed.candidates == 0 && observed.ignored == 1);
    auto chain = create(selected.value);
    require(read(&observed) && observed.candidates == 1 && observed.width == 16 && observed.height == 16);
    if (policyControlRound >= 6 && policyControlRound <= 8) {
        using PacingRead = BOOL (*)(dxgi_pacing::Snapshot*, DWORD);
        const auto pacingRead = reinterpret_cast<PacingRead>(GetProcAddress(GetModuleHandleW(L"bootstrap-compat.dll"),
            "BootstrapPacingSnapshot"));
        require(pacingRead && !pacingRead(nullptr, sizeof(dxgi_pacing::Snapshot)));
        dxgi_pacing::Snapshot invalidSnapshot{};
        require(!pacingRead(&invalidSnapshot, sizeof(invalidSnapshot) - 1));
        LARGE_INTEGER frequency{}; require(QueryPerformanceFrequency(&frequency));
        dxgi_pacing::Snapshot timing{};
        const auto frames = [&](unsigned ceiling) {
            LONGLONG previous = 0;
            for (unsigned frame = 0; frame < 4; ++frame) {
                require(SUCCEEDED(chain->Present(0, 0)) && pacingRead(&timing, sizeof(timing))
                    && timing.lastCeiling == ceiling && !timing.faults && timing.liveBindings == 1);
                if (previous) require(timing.lastRelease - previous >= (frequency.QuadPart + ceiling - 1) / ceiling);
                previous = timing.lastRelease;
            }
        };
        frames(10);
        if (policyControlRound == 8) {
            chain.Reset(); chain = create(selected.value); frames(10);
            using HudRead = BOOL (*)(fps_hud::Snapshot*, DWORD);
            const auto hudRead = reinterpret_cast<HudRead>(GetProcAddress(GetModuleHandleW(L"bootstrap-compat.dll"),
                "BootstrapHudSnapshot"));
            fps_hud::Snapshot hud{};
            require(hudRead && !hudRead(nullptr, sizeof(hud)) && !hudRead(&hud, sizeof(hud) - 1));
            const ULONGLONG started = GetTickCount64();
            for (;;) {
                require(hudRead(&hud, sizeof(hud)) && !hud.failures);
                if (hud.window && hud.draws && hud.bounds.right > hud.bounds.left) break;
                require(GetTickCount64() - started < 3000); Sleep(5);
            }
            require(!hud.visible && !IsWindowVisible(hud.window) && GetWindow(hud.window, GW_OWNER) == selected.value);
            require(receipt.ready && receipt.finish && SetEvent(receipt.ready)
                && WaitForSingleObject(receipt.finish, 10000) == WAIT_OBJECT_0);
            const auto presents = timing.pacedPresent;
            require(SUCCEEDED(chain->Present(0, 0)) && pacingRead(&timing, sizeof(timing))
                && timing.pacedPresent == presents && timing.bypassed == 1 && !timing.faults);
            chain.Reset(); chain = create(selected.value);
            require(SUCCEEDED(chain->Present(0, 0)) && pacingRead(&timing, sizeof(timing))
                && timing.pacedPresent == presents && timing.bypassed == 2 && !timing.faults);
        } else for (unsigned phase = 0; phase < 4; ++phase) {
            require(SetEvent(receipt.ready) && WaitForSingleObject(receipt.go, 10000) == WAIT_OBJECT_0);
            if (phase == 2) { chain.Reset(); chain = create(selected.value); }
            if (phase < 3) frames(phase == 1 ? 10 : 144);
            else {
                const auto presents = timing.pacedPresent;
                require(SUCCEEDED(chain->Present(0, 0)) && pacingRead(&timing, sizeof(timing))
                    && timing.pacedPresent == presents && timing.bypassed == 1 && !timing.faults);
                chain.Reset(); chain = create(selected.value);
                require(SUCCEEDED(chain->Present(0, 0)) && pacingRead(&timing, sizeof(timing))
                    && timing.pacedPresent == presents && timing.bypassed == 2 && !timing.faults);
            }
        }
        chain.Reset();
        require(pacingRead(&timing, sizeof(timing)) && !timing.liveBindings
            && timing.createdBindings == 3 && timing.destroyedBindings == 3 && !timing.unsupportedInterfaces);
        return;
    }
    chain.Reset(); // The observer must not pin an obsolete flip chain on this HWND.
    chain = create(selected.value);
    require(read(&observed) && observed.candidates == 2 && observed.failures == 0);
    chain.Reset();
    DXGI_SWAP_CHAIN_DESC oldDescription{};
    oldDescription.BufferDesc.Width = oldDescription.BufferDesc.Height = 16;
    oldDescription.BufferDesc.Format = description.Format; oldDescription.SampleDesc.Count = 1;
    oldDescription.BufferUsage = description.BufferUsage; oldDescription.BufferCount = 2;
    oldDescription.OutputWindow = selected.value; oldDescription.Windowed = TRUE;
    oldDescription.SwapEffect = DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL;
    const auto unchanged = oldDescription;
    ComPtr<IDXGISwapChain> oldChain;
    require(SUCCEEDED(legacy->CreateSwapChain(queue.Get(), &oldDescription, &oldChain))
        && memcmp(&oldDescription, &unchanged, sizeof(unchanged)) == 0);
    require(read(&observed) && observed.candidates == 3 && observed.failures == 0);
    ComPtr<IDXGISwapChain1> failed;
    require(FAILED(factory->CreateSwapChainForHwnd(nullptr, selected.value, &description,
        nullptr, nullptr, &failed)) && !failed);
    require(read(&observed) && observed.candidates == 3 && observed.failures == 0);
    // A factory recovered from a native chain must use the same creation hook.
    ComPtr<IDXGIFactory2> recovered;
    ComPtr<IUnknown> legacyIdentity, recoveredIdentity;
    require(SUCCEEDED(oldChain->GetParent(IID_PPV_ARGS(&recovered)))
        && SUCCEEDED(legacy.As(&legacyIdentity)) && SUCCEEDED(recovered.As(&recoveredIdentity))
        && legacyIdentity.Get() == recoveredIdentity.Get());
    oldChain.Reset();
    require(SUCCEEDED(recovered->CreateSwapChainForHwnd(queue.Get(), selected.value,
        &description, nullptr, nullptr, &chain)));
    chain.Reset(); recovered.Reset(); recoveredIdentity.Reset(); legacyIdentity.Reset();
    require(read(&observed) && observed.candidates == 4 && observed.failures == 0);
    const LONG factoriesBefore = observed.factories;
    // Repeated factory lifetimes must not exhaust a retained-reference registry.
    for (unsigned lifetime = 0; lifetime < 36; ++lifetime) {
        ComPtr<IDXGIFactory4> transient;
        require(SUCCEEDED(CreateDXGIFactory2(0, IID_PPV_ARGS(&transient))));
        factory = transient;
        chain = create(selected.value);
        chain.Reset();
        factory.Reset();
    }
    require(read(&observed));
    require(observed.factories == factoriesBefore + 36);
    require(observed.candidates == 40);
    require(observed.failures == 0);
}

int configuredStorageChild(probe::Receipt& receipt) {
    configuredDiagnosticChild = true;
    presentationChecks(receipt);
    if (policyControlRound == 6 || policyControlRound == 7) {
        InterlockedExchange(&receipt.nativeChecks, 1);
        return 0;
    }
    // Candidate storage configuration must not guess or rewrite a singleton.
    Handle shared(CreateMutexW(nullptr, FALSE, receipt.logical));
    require(shared.value && GetLastError() == ERROR_ALREADY_EXISTS);
    InterlockedExchange(&receipt.nativeChecks, 1);
    if (receipt.fileRound == 2 || receipt.fileRound == 3) return interruptedStorageChild(receipt);
    if (receipt.fileRound == 4 || receipt.fileRound == 5) return mappedStorageChild(receipt);
    RegistryChild registry(receipt);
    helperBindingCheck(receipt, false);
    helperBindingCheck(receipt, true);
    worldStorageChecks(receipt);
    for (const auto& folder : {L"Roaming", L"Local"}) {
        const auto logical = directoryPath(receipt.fileLogicalRoot).substr(4) + L"\\" + folder + L"\\configured.bin";
        const auto own = directoryPath(receipt.fileOwnRoot).substr(4) + L"\\" + folder + L"\\configured.bin";
        if (!receipt.fileRound) {
            Handle absent(CreateFileW(logical.c_str(), GENERIC_READ, FILE_SHARE_READ,
                nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr));
            require(absent.value == INVALID_HANDLE_VALUE && GetLastError() == ERROR_FILE_NOT_FOUND);
        } else expectFile(logical, std::to_string(receipt.slot) + "-1");
        Handle output(CreateFileW(logical.c_str(), GENERIC_WRITE, FILE_SHARE_READ,
            nullptr, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr));
        require(output.value != INVALID_HANDLE_VALUE);
        writeBytes(output.value, std::to_string(receipt.slot) + "-" + std::to_string(receipt.fileRound + 1));
        const auto id = wcscmp(folder, L"Roaming") == 0 ? FOLDERID_RoamingAppData : FOLDERID_LocalAppData;
        PWSTR known = nullptr;
        require(SUCCEEDED(SHGetKnownFolderPath(id, KF_FLAG_DONT_VERIFY, nullptr, &known)) && known);
        const bool matches = own == std::wstring(known) + L"\\configured.bin";
        CoTaskMemFree(known);
        require(matches);
    }
    require(SetEvent(receipt.ready) != FALSE);
    require(WaitForSingleObject(receipt.go, 10000) == WAIT_OBJECT_0);
    registry.afterPeer(receipt);
    require(SetEvent(receipt.done) != FALSE);
    require(WaitForSingleObject(receipt.finish, 10000) == WAIT_OBJECT_0);
    return 0;
}

bool configuredStoragePair(const std::wstring& self, const std::string& compatibility) {
    ConfiguredRoots roots;
    const auto reference = roots.root + L"\\native-reference";
    require(CreateDirectoryW(reference.c_str(), nullptr));
    nativeStorageMutationChecks(reference); // Same assertions against real unmediated Windows.
    require(RemoveDirectoryW(reference.c_str()));
    const auto name = L"Local\\EducationCanary_" + std::to_wstring(GetCurrentProcessId())
        + L"_" + std::to_wstring(GetTickCount64());
    RegistryHost registry(name);
    Handle unchangedMutex(CreateMutexW(nullptr, TRUE, name.c_str()));
    require(unchangedMutex.value && GetLastError() != ERROR_ALREADY_EXISTS);
    Key stores[2];
    std::wstring paths[2];
    SECURITY_ATTRIBUTES security{sizeof(SECURITY_ATTRIBUTES), nullptr, TRUE};
    for (DWORD slot = 1; slot <= 2; ++slot) {
        FILE_ID_INFO id{};
        require(GetFileInformationByHandleEx(slot == 1 ? roots.a.value : roots.b.value,
            FileIdInfo, &id, sizeof(id)) != FALSE);
        wchar_t leaf[64]{};
        require(probe::registryStoreLeaf(id, slot, leaf));
        paths[slot - 1] = L"Software\\MivubiEducationPilot\\" + std::wstring(leaf);
        DWORD disposition = 0;
        require(RegCreateKeyExW(HKEY_CURRENT_USER, paths[slot - 1].c_str(), 0, nullptr,
            REG_OPTION_NON_VOLATILE, KEY_ALL_ACCESS, &security, &stores[slot - 1].value,
            &disposition) == ERROR_SUCCESS && disposition == REG_CREATED_NEW_KEY);
        Key branch;
        require(RegCreateKeyExW(stores[slot - 1].value, registry.path.c_str(), 0, nullptr,
            REG_OPTION_NON_VOLATILE, KEY_ALL_ACCESS, nullptr, &branch.value, nullptr) == ERROR_SUCCESS);
        setRegistryValue(branch.value, L"Marker", slot * 100);
    }
    if (presentationControl) {
        const auto selectedMode = presentationControl;
        presentationControl = L"invalid";
        Handle go(sharedEvent()), finish(sharedEvent());
        ConfiguredChild rejected(1, 0, roots, stores[0].value, registry, name, go.value, finish.value,
            self, compatibility);
        rejected.exited(false);
        presentationControl = selectedMode;
    }
    const bool paced = presentationControl && wcscmp(presentationControl, L"pace") == 0;
    for (DWORD invalid = 1; invalid <= (paced ? 39UL : presentationControl ? 24UL : 5UL); ++invalid) {
        Handle go(sharedEvent()), finish(sharedEvent());
        ConfiguredChild rejected(1, 0, roots, stores[0].value, registry, name, go.value, finish.value,
            self, compatibility, invalid);
        rejected.exited(false);
        registry.unchanged();
    }
    if (paced) for (DWORD round : {6UL, 7UL}) {
        Key canary;
        require(RegOpenKeyExW(stores[0].value, registry.path.c_str(), 0, KEY_SET_VALUE, &canary.value) == ERROR_SUCCESS);
        setRegistryValue(canary.value, L"Marker", 100);
        Handle go(CreateEventW(nullptr, FALSE, FALSE, nullptr)), finish(sharedEvent());
        require(go.value && SetHandleInformation(go.value, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT));
        ConfiguredChild policyChild(1, round, roots, stores[0].value, registry, name, go.value, finish.value,
            self, compatibility);
        for (unsigned phase = 0; phase < 4; ++phase) {
            policyChild.phase(policyChild.ready.value);
            require(ResetEvent(policyChild.ready.value));
            if (phase == 3) {
                if (round == 6) require(SetEvent(policyChild.policyStop.value));
                else {
                    require(ResumeThread(policyChild.controller.info.hThread) == 1
                        && WaitForSingleObject(policyChild.controller.info.hProcess, 10000) == WAIT_OBJECT_0);
                    DWORD code = 1;
                    require(GetExitCodeProcess(policyChild.controller.info.hProcess, &code) && code == 0);
                }
            } else {
                require(phase == 1 ? ResetEvent(policyChild.policyLevel.value) : SetEvent(policyChild.policyLevel.value));
                require(SetEvent(policyChild.policyChanged.value));
            }
            require(SetEvent(go.value));
        }
        policyChild.recoveryExited(0);
        std::printf("{\"case\":\"bootstrap-creation-policy\",\"ceilings\":[10,144,10,144],\"recreated\":true,"
            "\"stickyStop\":true,\"controllerExit\":%s,\"minecraft\":\"NOT_EXECUTED\"}\n", round == 7 ? "true" : "false");
        setRegistryValue(canary.value, L"Marker", 100);
        registry.unchanged();
    }
    for (DWORD round = 0; round < 2; ++round) {
        Handle go(sharedEvent()), finish(sharedEvent());
        ConfiguredChild first(1, round, roots, stores[0].value, registry, name, go.value, finish.value,
            self, compatibility);
        ConfiguredChild second(2, round, roots, stores[1].value, registry, name, go.value, finish.value,
            self, compatibility);
        first.phase(first.ready.value); second.phase(second.ready.value);
        registry.unchanged();
        require(SetEvent(go.value) != FALSE);
        first.phase(first.done.value); second.phase(second.done.value);
        require(SetEvent(finish.value) != FALSE);
        first.exited(true); second.exited(true);
        for (const auto& folder : {L"Roaming", L"Local"}) {
            expectFile(roots.root + L"\\host-view\\" + folder + L"\\configured.bin", "HOST");
            expectFile(roots.root + L"\\client-one\\" + folder + L"\\configured.bin", "1-" + std::to_string(round + 1));
            expectFile(roots.root + L"\\client-two\\" + folder + L"\\configured.bin", "2-" + std::to_string(round + 1));
        }
        for (DWORD slot = 1; slot <= 2; ++slot) {
            const auto leaf = slot == 1 ? L"client-one" : L"client-two";
            expectFile(roots.root + L"\\" + leaf
                + L"\\Roaming\\Minecraft Education Edition\\games\\com.mojang\\minecraftWorlds\\world_A+=\\db\\CURRENT",
                std::to_string(slot) + "-" + std::to_string(round + 1));
            require(RegCloseKey(stores[slot - 1].value) == ERROR_SUCCESS);
            stores[slot - 1].value = nullptr;
            require(RegOpenKeyExW(HKEY_CURRENT_USER, paths[slot - 1].c_str(), 0, KEY_ALL_ACCESS,
                &stores[slot - 1].value) == ERROR_SUCCESS
                && SetHandleInformation(stores[slot - 1].value, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT));
            Key branch;
            require(RegOpenKeyExW(stores[slot - 1].value, registry.path.c_str(), 0, KEY_READ,
                &branch.value) == ERROR_SUCCESS
                && registryValue(branch.value, L"Marker") == slot * 100 + round + 1);
        }
        registry.unchanged();
        require(GetFileAttributesW((roots.root + L"\\host-view\\Roaming\\Minecraft Education Edition").c_str())
            == INVALID_FILE_ATTRIBUTES);
    }
    {
        Handle go(sharedEvent()), finish(sharedEvent());
        // Re-arm only generated registry canaries for the existing nonzero-round startup check.
        for (DWORD slot = 1; slot <= 2; ++slot) {
            Key branch;
            require(RegOpenKeyExW(stores[slot - 1].value, registry.path.c_str(), 0, KEY_ALL_ACCESS,
                &branch.value) == ERROR_SUCCESS);
            setRegistryValue(branch.value, L"Marker", slot * 100 + 1);
        }
        ConfiguredChild interrupted(1, 2, roots, stores[0].value, registry, name, go.value, finish.value,
            self, compatibility);
        ConfiguredChild peer(2, 2, roots, stores[1].value, registry, name, go.value, finish.value,
            self, compatibility);
        interrupted.phase(interrupted.ready.value); peer.phase(peer.ready.value);
        // Original self-child handle only; never discover or terminate an external process.
        require(TerminateProcess(interrupted.child.info.hProcess, 91));
        interrupted.recoveryExited(91);
        const auto suffix = L"\\Roaming\\Minecraft Education Edition\\games\\com.mojang\\minecraftWorlds\\world_A+=\\db";
        const auto first = roots.root + L"\\client-one" + suffix;
        expectFile(first + L"\\recovery-current", "1-committed");
        expectFile(first + L"\\.recovery-retry.tmp", "1-pending");
        {
            Handle released(CreateFileW((first + L"\\.recovery-retry.tmp").c_str(), GENERIC_READ | GENERIC_WRITE,
                0, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr));
            require(released.value != INVALID_HANDLE_VALUE);
            expectBytes(released.value, "1-pending");
        }
        peer.phase(peer.ready.value); // It survived and still waits at its own checkpoint.
        require(SetEvent(go.value));
        peer.phase(peer.done.value);
        {
            Key branch;
            require(RegOpenKeyExW(stores[0].value, registry.path.c_str(), 0, KEY_ALL_ACCESS,
                &branch.value) == ERROR_SUCCESS);
            setRegistryValue(branch.value, L"Marker", 101);
        }
        ConfiguredChild reopened(1, 3, roots, stores[0].value, registry, name, go.value, finish.value,
            self, compatibility);
        reopened.phase(reopened.ready.value); reopened.phase(reopened.done.value);
        for (DWORD slot = 1; slot <= 2; ++slot) {
            const auto own = roots.root + (slot == 1 ? L"\\client-one" : L"\\client-two") + suffix;
            expectFile(own + L"\\recovery-current", std::to_string(slot) + "-pending");
            require(GetFileAttributesW((own + L"\\.recovery-retry.tmp").c_str()) == INVALID_FILE_ATTRIBUTES
                && GetLastError() == ERROR_FILE_NOT_FOUND);
            expectFile(own + L"\\CURRENT", std::to_string(slot) + "-2");
        }
        require(SetEvent(finish.value));
        reopened.recoveryExited(0); peer.recoveryExited(0);
        registry.unchanged();
        require(GetFileAttributesW((roots.root + L"\\host-view\\Roaming\\Minecraft Education Edition").c_str())
            == INVALID_FILE_ATTRIBUTES);
    }
    {
        Handle go(sharedEvent()), finish(sharedEvent());
        for (DWORD slot = 1; slot <= 2; ++slot) {
            Key branch;
            require(RegOpenKeyExW(stores[slot - 1].value, registry.path.c_str(), 0, KEY_ALL_ACCESS,
                &branch.value) == ERROR_SUCCESS);
            setRegistryValue(branch.value, L"Marker", slot * 100 + 1);
        }
        ConfiguredChild interrupted(1, 4, roots, stores[0].value, registry, name, go.value, finish.value,
            self, compatibility);
        ConfiguredChild peer(2, 4, roots, stores[1].value, registry, name, go.value, finish.value,
            self, compatibility);
        interrupted.phase(interrupted.ready.value); peer.phase(peer.ready.value);
        const auto suffix = L"\\Roaming\\Minecraft Education Edition\\games\\com.mojang\\minecraftWorlds\\world_A+=\\db";
        for (DWORD slot = 1; slot <= 2; ++slot) {
            const auto path = roots.root + (slot == 1 ? L"\\client-one" : L"\\client-two") + suffix + L"\\mapping-current";
            Handle contender(CreateFileW(path.c_str(), GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, nullptr, OPEN_EXISTING, 0, nullptr));
            require(contender.value != INVALID_HANDLE_VALUE);
            OVERLAPPED overlap{}; overlap.Offset = 8;
            require(!LockFileEx(contender.value, LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                0, 1, 0, &overlap) && GetLastError() == ERROR_LOCK_VIOLATION);
            OVERLAPPED separate{}; separate.Offset = 9;
            require(LockFileEx(contender.value, LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                0, 1, 0, &separate) && UnlockFileEx(contender.value, 0, 1, 0, &separate));
        }
        require(TerminateProcess(interrupted.child.info.hProcess, 92));
        interrupted.recoveryExited(92);
        const auto first = roots.root + L"\\client-one" + suffix;
        {
            Handle released(CreateFileW((first + L"\\mapping-current").c_str(), GENERIC_READ | GENERIC_WRITE,
                0, nullptr, OPEN_EXISTING, 0, nullptr));
            require(released.value != INVALID_HANDLE_VALUE);
            OVERLAPPED region{}; region.Offset = 8;
            require(LockFileEx(released.value, LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                0, 1, 0, &region) && UnlockFileEx(released.value, 0, 1, 0, &region));
            expectBytes(released.value, mappedMarker(1));
        }
        require(GetFileAttributesW((first + L"\\.mapping-retry.tmp").c_str()) == INVALID_FILE_ATTRIBUTES
            && GetLastError() == ERROR_FILE_NOT_FOUND);
        peer.phase(peer.ready.value);
        require(SetEvent(go.value)); peer.phase(peer.done.value);
        {
            Key branch;
            require(RegOpenKeyExW(stores[0].value, registry.path.c_str(), 0, KEY_ALL_ACCESS,
                &branch.value) == ERROR_SUCCESS);
            setRegistryValue(branch.value, L"Marker", 101);
        }
        ConfiguredChild reopened(1, 5, roots, stores[0].value, registry, name, go.value, finish.value,
            self, compatibility);
        reopened.phase(reopened.ready.value); reopened.phase(reopened.done.value);
        require(SetEvent(finish.value));
        reopened.recoveryExited(0); peer.recoveryExited(0);
        for (DWORD slot = 1; slot <= 2; ++slot) {
            const auto own = roots.root + (slot == 1 ? L"\\client-one" : L"\\client-two") + suffix;
            expectFile(own + L"\\mapping-current", mappedMarker(slot));
            expectFile(own + L"\\CURRENT", std::to_string(slot) + "-2");
            require(GetFileAttributesW((own + L"\\.mapping-retry.tmp").c_str()) == INVALID_FILE_ATTRIBUTES
                && GetLastError() == ERROR_FILE_NOT_FOUND);
        }
        registry.unchanged();
        require(GetFileAttributesW((roots.root + L"\\host-view\\Roaming\\Minecraft Education Edition").c_str())
            == INVALID_FILE_ATTRIBUTES);
    }
    std::printf("{\"case\":\"configured-storage-pair\",\"selfCheck\":true,\"earlyPrivate\":[7,7],"
        "\"registryChecks\":[65535,65535],\"rounds\":2,\"persistent\":true,\"hostUnchanged\":true,"
        "\"invalidConfigRejected\":5,\"mainRegistryCallback\":false,\"fixtureNameRequired\":false,"
        "\"storageWorkflow\":true,\"atomicReplaceCycles\":64,\"failedReplacePreservesData\":true,"
        "\"createCollisionPreservesData\":true,\"renameAndLinkEscapeDenied\":true,"
        "\"interruptedWriterRecovered\":true,\"peerSurvived\":true,\"exclusiveRetryReopen\":true,"
        "\"postReplaceMappedWriterRecovered\":true,\"byteRangeLocksVerified\":true,"
        "\"unboundHelperEnvironmentPrivate\":true,\"unboundHelperKnownFoldersPrivate\":false,"
        "\"unboundHelperHkcuPrivate\":false,\"unboundHelpersChecked\":4,"
        "\"explicitHelperKnownFoldersPrivate\":true,\"explicitHelperHkcuPrivate\":true,"
        "\"explicitHelpersChecked\":4,\"automaticHelperEnrollment\":false,"
        "\"mediatedCallerEnrollmentRefusals\":4,"
        "\"presentationEnrollmentChecked\":%s,"
        "\"presentationInvalidCapabilitiesRejected\":%u,\"presentationCreationCoverage\":\"%s\","
        "\"singleton\":\"UNKNOWN\",\"singletonRemap\":false,\"helpers\":\"UNKNOWN\",\"securityBoundary\":false,"
        "\"providerReady\":false,\"minecraft\":\"NOT_EXECUTED\"}\n", presentationControl ? "true" : "false", paced ? 34U : presentationControl ? 19U : 0U,
        presentationControl ? "STARTUP_RVA_40_CREATIONS" : "OFF");
    return true;
}




void checkStartupObservation(const education_bootstrap::Config& config, const probe::Receipt& receipt) {
    SetLastError(ERROR_SUCCESS);
    if (!GetEnvironmentVariableW(startup_observation::Variable, nullptr, 0)
        && GetLastError() == ERROR_ENVVAR_NOT_FOUND) return;
    startup_observation::View view;
    require(view.open(config));
    auto& journal = *view.value;
    require(journal.clockReady == 1);
    // Exercise actual mediated routing with generated names, never game paths.
    const auto fileCreate = reinterpret_cast<probe::NativeFileCreate>(
        GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "NtCreateFile"));
    require(fileCreate && journal.storage.counts[1] == 0);
    for (const wchar_t* invalid : {L"invalid:stream", L"invalid."}) {
        const auto bytes = static_cast<USHORT>(wcslen(invalid) * sizeof(wchar_t));
        UNICODE_STRING path{bytes, bytes, const_cast<PWSTR>(invalid)};
        OBJECT_ATTRIBUTES relative{sizeof(OBJECT_ATTRIBUTES), config.logicalRoot, &path,
            OBJ_CASE_INSENSITIVE, nullptr, nullptr};
        IO_STATUS_BLOCK io{};
        HANDLE output = nullptr;
        SetLastError(0x13579bdf);
        require(fileCreate(&output, GENERIC_WRITE, &relative, &io, nullptr, 0,
            FILE_SHARE_READ | FILE_SHARE_WRITE, 5, FILE_NON_DIRECTORY_FILE, nullptr, 0)
                == static_cast<NTSTATUS>(0xc00000bb)
            && output == nullptr && io.Status == static_cast<NTSTATUS>(0xc00000bb)
            && GetLastError() == 0x13579bdf);
    }
    require(journal.storage.counts[1] == 2 && journal.storage.first[0].committed == 1
        && journal.storage.first[0].operation == 1 && journal.storage.first[0].pathLength == 14
        && journal.storage.reserved[0] == 1
        && journal.storage.reserved[1] == startup_observation::ComposedPath
        && journal.storage.reserved[2] == startup_observation::UnsupportedCharacter); // First sample wins.
    const auto name = L"\\Sessions\\" + std::to_wstring(config.session)
        + L"\\BaseNamedObjects\\" + std::wstring(receipt.logical + 6);
    require(name.size() < 192);
    UNICODE_STRING text{static_cast<USHORT>(name.size() * sizeof(wchar_t)),
        static_cast<USHORT>(name.size() * sizeof(wchar_t)), const_cast<PWSTR>(name.c_str())};
    OBJECT_ATTRIBUTES attributes{sizeof(OBJECT_ATTRIBUTES), nullptr, &text,
        OBJ_CASE_INSENSITIVE | 0x80 /* OBJ_OPENIF */, nullptr, nullptr};
    const HMODULE ntdll = GetModuleHandleW(L"ntdll.dll");
    const auto create = reinterpret_cast<probe::NativeCreate>(GetProcAddress(ntdll, "NtCreateMutant"));
    const auto open = reinterpret_cast<probe::NativeOpen>(GetProcAddress(ntdll, "NtOpenMutant"));
    require(create && open);
    HANDLE a = nullptr, b = nullptr, c = nullptr;
    constexpr DWORD sentinel = 0x13579bdf, access = 0x1f0001;
    SetLastError(sentinel);
    require(create(&a, access, &attributes, FALSE) == 0 && a && GetLastError() == sentinel);
    Handle first(a);
    SetLastError(sentinel);
    require(create(&b, access, &attributes, FALSE) == static_cast<NTSTATUS>(0x40000000)
        && b && GetLastError() == sentinel);
    Handle second(b);
    attributes.Attributes = OBJ_CASE_INSENSITIVE;
    SetLastError(sentinel);
    require(open(&c, access, &attributes) == 0 && c && GetLastError() == sentinel);
    Handle third(c);
    const auto absent = name + L"_absent";
    text = {static_cast<USHORT>(absent.size() * sizeof(wchar_t)),
        static_cast<USHORT>(absent.size() * sizeof(wchar_t)), const_cast<PWSTR>(absent.c_str())};
    HANDLE missing = nullptr;
    SetLastError(sentinel);
    require(open(&missing, access, &attributes) == static_cast<NTSTATUS>(0xc0000034)
        && !missing && GetLastError() == sentinel);

    const auto self = executable();
    auto command = L"\"" + self + L"\" --bootstrap-observation-child";
    STARTUPINFOW startup{sizeof(STARTUPINFOW)};
    Child child;
    auto& info = child.info;
    require(CreateProcessW(self.c_str(), command.data(), nullptr, nullptr, FALSE, CREATE_NO_WINDOW,
        nullptr, nullptr, &startup, &info));
    const DWORD successfulError = GetLastError();
    require(WaitForSingleObject(info.hProcess, 10000) == WAIT_OBJECT_0);
    DWORD code = 1;
    require(GetExitCodeProcess(info.hProcess, &code) && code == 0);
    FILETIME birth{}, exit{}, kernel{}, user{};
    require(GetProcessTimes(info.hProcess, &birth, &exit, &kernel, &user));
    const auto nonexistent = self.substr(0, self.find_last_of(L"\\/") + 1) + L"missing-observation-control.exe";
    require(GetFileAttributesW(nonexistent.c_str()) == INVALID_FILE_ATTRIBUTES);
    PROCESS_INFORMATION failed{};
    require(!CreateProcessW(nonexistent.c_str(), nullptr, nullptr, nullptr, FALSE, 0,
        nullptr, nullptr, &startup, &failed) && GetLastError() == ERROR_FILE_NOT_FOUND);
    bool created = false, existed = false, opened = false, absentSeen = false, spawned = false, failedSeen = false;
    require(journal.losses == 0 && journal.claimed <= static_cast<LONG>(startup_observation::Capacity));
    for (LONG i = 0; i < journal.claimed; ++i) {
        const auto& event = journal.records[i];
        require(event.committed == 1 && event.thread && event.caller && event.end >= event.start);
        if (event.kind == startup_observation::CreateProcess) {
            if (event.status == TRUE && event.childPid == info.dwProcessId) {
                require(event.childTid == info.dwThreadId && event.flags == 0
                    && event.childCreated.dwHighDateTime == birth.dwHighDateTime
                    && event.childCreated.dwLowDateTime == birth.dwLowDateTime
                    && event.argument == CREATE_NO_WINDOW && event.attributes == 0
                    && event.lastError == successfulError);
                spawned = true;
            }
            if (event.status == FALSE && event.lastError == ERROR_FILE_NOT_FOUND) failedSeen = true;
        } else if (event.nameBytes == name.size() * sizeof(wchar_t)
            && memcmp(event.name, name.data(), event.nameBytes) == 0) {
            require(event.flags == 0 && event.argument == access && event.root == 0
                && event.lastError == sentinel);
            if (event.kind == startup_observation::CreateMutant && event.status == 0) created = true;
            if (event.kind == startup_observation::CreateMutant && event.status == 0x40000000) existed = true;
            if (event.kind == startup_observation::OpenMutant && event.status == 0) opened = true;
        } else if (event.nameBytes == absent.size() * sizeof(wchar_t)
            && memcmp(event.name, absent.data(), event.nameBytes) == 0
            && event.kind == startup_observation::OpenMutant && event.status == static_cast<LONG>(0xc0000034))
            absentSeen = true;
    }
    require(created && existed && opened && absentSeen && spawned && failedSeen);
    // Extend the existing observation control with the four real Win32 wrappers.
    // A fresh per-fixture GUID prevents collision with real application objects.
    const bool scoped = config.scope == education_bootstrap::StorageRegistrySelectedMutex;
    wchar_t guid[37]{};
    if (scoped) {
        unsigned short part = static_cast<unsigned short>(config.logicalId.FileId.Identifier[0]
            | (static_cast<unsigned>(config.logicalId.FileId.Identifier[1]) << 8));
        unsigned long long tail = 0;
        for (unsigned i = 0; i < 6; ++i)
            tail |= static_cast<unsigned long long>(config.logicalId.FileId.Identifier[i + 2]) << (8 * i);
        require(swprintf_s(guid, L"feedface-%04x-4000-8000-%012llx", static_cast<unsigned>(part), tail) == 36);
    } else require(swprintf_s(guid, L"fee1dead-%04x-4000-8000-%012llx",
        static_cast<unsigned>(config.slot), static_cast<unsigned long long>(GetCurrentProcessId())) == 36);
    char ansi[37]{};
    for (unsigned i = 0; i < 36; ++i) ansi[i] = static_cast<char>(guid[i]);
    SetLastError(sentinel);
    Handle winW(CreateMutexW(nullptr, TRUE, guid));
    require(winW.value && GetLastError() == static_cast<DWORD>(scoped ? ERROR_ALREADY_EXISTS : ERROR_SUCCESS));
    SetLastError(sentinel);
    Handle winExW(CreateMutexExW(nullptr, guid, CREATE_MUTEX_INITIAL_OWNER, MUTEX_ALL_ACCESS));
    require(winExW.value && GetLastError() == ERROR_ALREADY_EXISTS);
    SetLastError(sentinel);
    Handle winA(CreateMutexA(nullptr, TRUE, ansi));
    require(winA.value && GetLastError() == static_cast<DWORD>(scoped ? ERROR_SUCCESS : ERROR_ALREADY_EXISTS));
    SetLastError(sentinel);
    Handle winExA(CreateMutexExA(nullptr, ansi, CREATE_MUTEX_INITIAL_OWNER, MUTEX_ALL_ACCESS));
    require(winExA.value && GetLastError() == ERROR_ALREADY_EXISTS);
    const auto sameObjects = reinterpret_cast<probe::CompareHandles>(
        GetProcAddress(GetModuleHandleW(L"kernelbase.dll"), "CompareObjectHandles"));
    require(sameObjects && sameObjects(winW.value, winExW.value)
        && sameObjects(winA.value, winExA.value)
        && (sameObjects(winW.value, winA.value) != FALSE) == !scoped);
    Handle reopened(OpenMutexW(MUTEX_ALL_ACCESS, FALSE, guid));
    require(reopened.value && sameObjects(reopened.value, winA.value));
    require(ReleaseMutex(scoped ? winA.value : winW.value));
    bool outer[4]{}, scopedNative = false, mappedCreate = false, mappedOpen = false;
    for (LONG i = 0; i < journal.claimed; ++i) {
        const auto& event = journal.records[i];
        if (event.nameBytes != 72 || memcmp(event.name, guid, 72) != 0) continue;
        if (event.kind == startup_observation::CreateMutant
            && event.namespaceKind == startup_observation::SessionBaseNamedObjects)
            scopedNative = true;
        if ((event.flags & startup_observation::SingletonTranslated) != 0) {
            require(scoped && event.namespaceKind == startup_observation::SessionBaseNamedObjects);
            if (event.kind == startup_observation::CreateMutant && event.status == 0) mappedCreate = true;
            if (event.kind == startup_observation::OpenMutant && event.status == 0) mappedOpen = true;
        }
        if (event.kind >= startup_observation::Win32MutexW && event.kind <= startup_observation::Win32MutexExA
            && event.callerModule == startup_observation::MainImage) {
            const DWORD index = event.kind - startup_observation::Win32MutexW;
            require(event.status == TRUE && event.returned && event.callerRva > 0
                && event.callerRva < event.callerImageBytes
                && event.flags == startup_observation::InitialOwner
                && event.lastError == static_cast<DWORD>(
                    (scoped ? index == 2 : index == 0) ? ERROR_SUCCESS : ERROR_ALREADY_EXISTS));
            outer[index] = true;
        }
    }
    require(scopedNative && outer[0] && outer[1] && outer[2] && outer[3]
        && (!scoped || (mappedCreate && mappedOpen)));
    const auto longName = name + std::wstring(200, L'x');
    text = {static_cast<USHORT>(longName.size() * sizeof(wchar_t)),
        static_cast<USHORT>(longName.size() * sizeof(wchar_t)), const_cast<PWSTR>(longName.c_str())};
    const LONG next = journal.claimed;
    require(next < static_cast<LONG>(startup_observation::Capacity));
    require(open(&missing, access, &attributes) < 0 && !missing);
    require(journal.records[next].committed == 1
        && (journal.records[next].flags & startup_observation::TruncatedName));
    for (DWORD i = 0; i < startup_observation::Capacity + 1; ++i) open(&missing, access, &attributes);
    require(journal.claimed == startup_observation::Capacity
        && journal.losses == startup_observation::CapacityReached);
    require(startup_observation::within(journal, journal.started + journal.frequency * 30)
        && !startup_observation::within(journal, journal.started + journal.frequency * 31));
    // Own cooperative control only: move its clock into the past to exercise expired hooks.
    journal.started -= journal.frequency * 31;
    const LONG before = journal.claimed;
    require(open(&missing, access, &attributes) < 0 && journal.claimed == before);
    const LONG storageBefore = journal.storage.counts[2];
    const auto storageProbe = directoryPath(config.privateRoot);
    SetLastError(ERROR_SUCCESS);
    const HANDLE rejectedFile = CreateFileW(storageProbe.c_str(), WRITE_DAC, FILE_SHARE_READ | FILE_SHARE_WRITE,
        nullptr, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, nullptr);
    require(rejectedFile == INVALID_HANDLE_VALUE && GetLastError() == ERROR_NOT_SUPPORTED
        && journal.storage.counts[2] == storageBefore + 1
        && journal.storage.first[1].committed == 1 && journal.claimed == before);
    std::printf("{\"case\":\"startup-observation-control\",\"callsChecked\":10,\"bounded\":true,"
        "\"lastErrorPreserved\":true,\"childRole\":\"UNKNOWN\",\"providerReady\":false}\n");
}

bool sendOwnedDescendant(HANDLE section) {
    struct View {
        owned_helper::Handoff* value;
        ~View() { if (value) UnmapViewOfFile(value); }
    } view{static_cast<owned_helper::Handoff*>(MapViewOfFile(section, FILE_MAP_READ | FILE_MAP_WRITE,
        0, 0, sizeof(owned_helper::Handoff)))};
    require(view.value != nullptr);
    auto& wire = *view.value;
    require(wire.magic == owned_helper::Magic && wire.version == 1 && wire.bytes == sizeof(wire)
        && wire.reserved == 0 && wire.reservedEnd == 0);
    if (InterlockedCompareExchange(&wire.decision, 0, 0) != owned_helper::Pending) return false;
    HANDLE inherited[]{wire.configuration, wire.proof};
    Attributes attributes(inherited, 2);
    auto env = environment(wire.proof);
    env.pop_back();
    const auto setting = std::wstring(education_bootstrap::SectionVariable) + L"="
        + std::to_wstring(reinterpret_cast<ULONG_PTR>(wire.configuration));
    env.insert(env.end(), setting.begin(), setting.end());
    env.push_back(0); env.push_back(0);
    const auto self = executable();
    std::wstring command = L"\"" + self + L"\"";
    STARTUPINFOEXW startup{};
    startup.StartupInfo.cb = sizeof(startup); startup.lpAttributeList = attributes.value;
    Child descendant;
    if (!CreateProcessW(self.c_str(), command.data(), nullptr, nullptr, TRUE,
        CREATE_SUSPENDED | CREATE_UNICODE_ENVIRONMENT | EXTENDED_STARTUPINFO_PRESENT | CREATE_NO_WINDOW,
        env.data(), nullptr, &startup.StartupInfo, &descendant.info)) {
        wire.error = GetLastError();
        require(SetEvent(wire.ready));
        return false;
    }
    wire.process = descendant.info.hProcess; wire.thread = descendant.info.hThread;
    wire.pid = descendant.info.dwProcessId;
    FILETIME exited{}, kernel{}, user{};
    if (!GetProcessTimes(descendant.info.hProcess, &wire.created, &exited, &kernel, &user))
        wire.error = GetLastError();
    MemoryBarrier();
    require(SetEvent(wire.ready));
    if (WaitForSingleObject(wire.reply, 10000) != WAIT_OBJECT_0
        || InterlockedCompareExchange(&wire.decision, 0, 0) != owned_helper::Accepted || wire.error)
        return false; // Existing Child guard cancels only this original suspended fixture.
    // Receiver now owns duplicates of these exact original objects; never resume or kill them here.
    const auto original = descendant.info;
    descendant.info = {};
    const BOOL threadClosed = CloseHandle(original.hThread), processClosed = CloseHandle(original.hProcess);
    require(threadClosed && processClosed);
    return true;
}

int readRustEnrollmentControl() {
    wchar_t presentationSetting[16]{};
    SetLastError(ERROR_SUCCESS);
    const DWORD presentationSize = GetEnvironmentVariableW(presentation_enrollment::Variable, presentationSetting, 16);
    const bool paced = presentationSize != 0;
    require(paced ? presentationSize == 4 && wcscmp(presentationSetting, L"pace") == 0
        : GetLastError() == ERROR_ENVVAR_NOT_FOUND);
    SetLastError(ERROR_SUCCESS);
    const DWORD sectionSize = GetEnvironmentVariableW(presentation_enrollment::SectionVariable, presentationSetting, 16);
    require(paced ? sectionSize > 0 && sectionSize < 16 : sectionSize == 0 && GetLastError() == ERROR_ENVVAR_NOT_FOUND);
    probe::View view;
    require(view.open() && view.value->mode == probe::ConfiguredStorage);
    probe::record(view.value, probe::Main);
    probe::registryCanary(probe::Main);
    require(DependencyCanary() == 7);
    auto& receipt = *view.value;
    const bool baseline = receipt.registryBaseline == 1;
    const probe::Stage hooked[]{probe::Attach, probe::Armed, probe::Dependency, probe::Tls, probe::Main};
    const probe::Stage unhooked[]{probe::Dependency, probe::Tls, probe::Main};
    require(receipt.count == (baseline ? 3 : 5) && receipt.armed == (baseline ? 0 : 1)
        && receipt.earlyAllowed == (baseline ? 7 : 0)
        && receipt.earlyPrivate == (baseline ? 0 : 7) && receipt.earlyDenied == 0);
    for (LONG i = 0; i < receipt.count; ++i) {
        require(receipt.events[i].stage == (baseline ? unhooked[i] : hooked[i])
            && receipt.events[i].armed == (baseline || i == 0 ? 0 : 1));
    }
    if (paced) {
        require(!baseline && receipt.earlyPrivate == 7);
        policyControlRound = 8;
        presentationChecks(receipt);
        std::printf("{\"case\":\"rust-focus-bootstrap\",\"backgroundCeiling\":10,\"recreated\":true,"
            "\"leaseDropStops\":true,\"minecraft\":\"NOT_EXECUTED\"}\n");
        return 0;
    }
    if (baseline) {
        std::printf("{\"case\":\"rust-enrollment-baseline\",\"hostCanaries\":7,\"providerReady\":false}\n");
        return 0;
    }
    education_bootstrap::Config config{};
    require(education_bootstrap::read(config));
    const auto ownPath = directoryPath(config.privateRoot).substr(4);
    // Path identity must be private even for an unmediated descendant (e.g. CEF).
    for (const auto* variable : {L"TMP", L"TEMP"}) {
        require(childEnvironment(variable) == ownPath + L"\\Local\\Temp");
    }
    wchar_t temporary[512]{};
    require(GetTempPathW(512, temporary) > 0
        && std::wstring(temporary) == ownPath + L"\\Local\\Temp\\");
    const auto logicalPath = directoryPath(config.logicalRoot).substr(4);
    for (const auto& folder : {L"Roaming", L"Local"}) {
        wchar_t environment[512]{};
        require(GetEnvironmentVariableW(wcscmp(folder, L"Roaming") == 0 ? L"APPDATA" : L"LOCALAPPDATA",
            environment, 512) > 0 && ownPath + L"\\" + folder == environment);
        PWSTR known = nullptr;
        require(SUCCEEDED(SHGetKnownFolderPath(wcscmp(folder, L"Roaming") == 0
            ? FOLDERID_RoamingAppData : FOLDERID_LocalAppData, KF_FLAG_DONT_VERIFY, nullptr, &known)) && known);
        const bool matches = ownPath + L"\\" + folder == known;
        CoTaskMemFree(known);
        require(matches);
    }
    DWORD round = 0, type = 0, bytes = sizeof(round);
    const auto status = RegQueryValueExW(config.registryRoot, L"RustEnrolledRound", nullptr, &type,
        reinterpret_cast<BYTE*>(&round), &bytes);
    require(status == ERROR_FILE_NOT_FOUND || (status == ERROR_SUCCESS && type == REG_DWORD
        && bytes == sizeof(round) && round < 2));
    ++round;
    setRegistryValue(config.registryRoot, L"RustEnrolledRound", round);
    require(RegFlushKey(config.registryRoot) == ERROR_SUCCESS);
    {
        // A logical host path must arrive in this slot's backing directory.
        Handle output(CreateFileW((logicalPath + L"\\Roaming\\rust-enrolled.txt").c_str(), GENERIC_WRITE,
            FILE_SHARE_READ, nullptr, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr));
        require(output.value != INVALID_HANDLE_VALUE);
        writeBytes(output.value, std::to_string(config.slot) + ":" + std::to_string(round));
    }
    // The cached host handle must now read this slot's private marker; the parent checks the host.
    require(registryValue(receipt.cachedHostKey, L"Marker") == config.slot * 100 + 3);
    checkStartupObservation(config, receipt);
    if (receipt.go) require(sendOwnedDescendant(receipt.go));
    // Optional existing receipt events let the Rust owner verify live Drop preservation.
    if (receipt.ready || receipt.finish) {
        require(receipt.ready && receipt.finish && SetEvent(receipt.ready));
        require(WaitForSingleObject(receipt.finish, 10000) == WAIT_OBJECT_0);
    }
    std::printf("{\"case\":\"rust-enrollment-control\",\"slot\":%lu,\"round\":%lu,"
        "\"earlyPrivate\":7,\"providerReady\":false,\"minecraft\":\"NOT_EXECUTED\"}\n", config.slot, round);
    return 0;
}

int readRustBootstrap() {
    education_bootstrap::Config config{};
    require(education_bootstrap::read(config));
    const HANDLE handles[]{config.logicalRoot, config.privateRoot, reinterpret_cast<HANDLE>(config.registryRoot)};
    for (HANDLE handle : handles) {
        DWORD flags = 0;
        require(GetHandleInformation(handle, &flags) && !(flags & HANDLE_FLAG_INHERIT));
    }
    FILE_ID_INFO logical{}, own{};
    require(GetFileInformationByHandleEx(config.logicalRoot, FileIdInfo, &logical, sizeof(logical))
        && GetFileInformationByHandleEx(config.privateRoot, FileIdInfo, &own, sizeof(own)));
    require(logical.VolumeSerialNumber == config.logicalId.VolumeSerialNumber
        && memcmp(logical.FileId.Identifier, config.logicalId.FileId.Identifier, 16) == 0
        && own.VolumeSerialNumber == config.privateId.VolumeSerialNumber
        && memcmp(own.FileId.Identifier, config.privateId.FileId.Identifier, 16) == 0);
    wchar_t leaf[64]{};
    require(probe::registryStoreLeaf(own, config.slot, leaf));
    Key user;
    require(RegOpenCurrentUser(KEY_READ, &user.value) == ERROR_SUCCESS);
    const auto actualUser = nativeKeyName(user.value);
    require(actualUser == config.userKeyPath);
    const auto expected = actualUser + L"\\Software\\MivubiEducationPilot\\" + leaf;
    require(_wcsicmp(nativeKeyName(config.registryRoot).c_str(), expected.c_str()) == 0);
    const auto ownPath = directoryPath(config.privateRoot).substr(4);
    for (const auto& folder : {L"Roaming", L"Local"}) {
        wchar_t environment[512]{};
        require(GetEnvironmentVariableW(wcscmp(folder, L"Roaming") == 0 ? L"APPDATA" : L"LOCALAPPDATA",
            environment, 512) > 0 && ownPath + L"\\" + folder == environment);
    }
    DWORD round = 0, type = 0, bytes = sizeof(round);
    const auto status = RegQueryValueExW(config.registryRoot, L"RustBridgeRound", nullptr, &type,
        reinterpret_cast<BYTE*>(&round), &bytes);
    require(status == ERROR_FILE_NOT_FOUND || (status == ERROR_SUCCESS && type == REG_DWORD
        && bytes == sizeof(round) && round < 2));
    ++round;
    setRegistryValue(config.registryRoot, L"RustBridgeRound", round);
    require(RegFlushKey(config.registryRoot) == ERROR_SUCCESS);
    {
        Handle output(CreateFileW((ownPath + L"\\Roaming\\rust-bridge.txt").c_str(), GENERIC_WRITE,
            FILE_SHARE_READ, nullptr, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr));
        require(output.value != INVALID_HANDLE_VALUE);
        writeBytes(output.value, std::to_string(config.slot) + ":" + std::to_string(round));
    }
    std::printf("{\"case\":\"rust-config-reader\",\"slot\":%lu,\"round\":%lu,"
        "\"selfCheck\":true,\"providerReady\":false,\"minecraft\":\"NOT_EXECUTED\"}\n",
        config.slot, round);
    return 0;
}

int childMain() {
    probe::View view;
    if (!view.open()) return 81;
    probe::record(view.value, probe::Main);
    probe::registryCanary(probe::Main);
    if (DependencyCanary() != 7) return 82;
    if (view.value->mode == probe::ConfiguredStorage) return configuredStorageChild(*view.value);
    if (view.value->mode == probe::MutexCase && view.value->registryBaseline) return 0;
    if (view.value->mode == probe::MutexCase && view.value->registryBacking == 1) {
        RegistryChild registry(*view.value);
        require(SetEvent(view.value->ready) != FALSE);
        require(WaitForSingleObject(view.value->go, 10000) == WAIT_OBJECT_0);
        registry.afterPeer(*view.value);
        require(SetEvent(view.value->done) != FALSE);
        require(WaitForSingleObject(view.value->finish, 10000) == WAIT_OBJECT_0);
        return 0;
    }
    if (view.value->mode == probe::MutexCase) return mutexChild(*view.value);
    if (view.value->mode == probe::Timeout) {
        Sleep(INFINITE); // Deliberate owned-child timeout control, outside loader lock.
        return 83;
    }
    return view.value->mode == probe::Normal && view.value->armed != 1 ? 84 : 0;
}
} // namespace

int wmain(int argc, wchar_t** argv) {
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
    if (argc == 2 && wcscmp(argv[1], L"--bootstrap-observation-child") == 0) return 0;
    if (argc == 2 && wcscmp(argv[1], L"--bootstrap-helper-view") == 0) {
        try { return helperViewMain(); } catch (...) { return 87; }
    }
    if (argc == 2 && wcscmp(argv[1], L"--bootstrap-policy-stop-child") == 0) policyControlRound = 6;
    if (argc == 2 && wcscmp(argv[1], L"--bootstrap-policy-death-child") == 0) policyControlRound = 7;
    if (policyControlRound || (argc == 2 && wcscmp(argv[1], L"--bootstrap-test-child") == 0)) {
        try { return childMain(); } catch (...) { return configuredDiagnosticChild && lastFailureLine
            ? 60000 + static_cast<int>(lastFailureLine) : 85; }
    }
    // Fixed no-argument receiver for an explicitly built Rust/C++ interoperability check.
    if (argc == 1 && education_bootstrap::present()) {
        try {
            probe::View view;
            return view.open() ? readRustEnrollmentControl() : readRustBootstrap();
        } catch (...) { return 86; }
    }
    const bool pacingTest = argc == 2 && wcscmp(argv[1], L"--run-presentation-pacing-self-test") == 0;
    const bool presentationTest = argc == 2 && wcscmp(argv[1], L"--run-presentation-enrollment-self-test") == 0;
    presentationControl = pacingTest ? L"pace" : presentationTest ? L"observe" : nullptr;
    const bool configuredTest = pacingTest || presentationTest || (argc == 2 && wcscmp(argv[1], L"--run-configured-storage-self-test") == 0);
    const bool mutexTest = argc == 2 && wcscmp(argv[1], L"--run-mutex-self-test") == 0;
    const bool ordinaryTest = argc == 2 && wcscmp(argv[1], L"--run-inherited-registry-self-test") == 0;
    if (!configuredTest && !mutexTest && !ordinaryTest && (argc != 2 || wcscmp(argv[1], L"--run-self-test") != 0)) return 64;
    try {
        USHORT process = 0, native = 0;
        require(IsWow64Process2(GetCurrentProcess(), &process, &native) != FALSE
            && process == IMAGE_FILE_MACHINE_UNKNOWN && native == IMAGE_FILE_MACHINE_AMD64);
        const std::wstring self = executable();
        const auto separator = self.find_last_of(L"\\/");
        require(separator != std::wstring::npos);
        const std::wstring dll = self.substr(0, separator + 1) + L"bootstrap-compat.dll";
        // Test-only bound: avoid ambiguous ANSI import conversion; no path fallback.
        std::string compatibility;
        for (wchar_t character : dll) {
            require(character > 0 && character <= 0x7f);
            compatibility.push_back(static_cast<char>(character));
        }
        if (configuredTest) return configuredStoragePair(self, compatibility) && !cleanupFailed ? 0 : 1;
        if (ordinaryTest) return ordinaryRegistryPair(self, compatibility) && !cleanupFailed ? 0 : 1;
        if (mutexTest) return mutexPair(self, compatibility) && !cleanupFailed ? 0 : 1;
        bool passed = run(probe::Baseline, L"baseline", self, compatibility);
        passed = run(probe::Normal, L"first-import", self, compatibility) && passed;
        passed = run(probe::FailAttach, L"attach-failure", self, compatibility) && passed;
        passed = run(probe::Timeout, L"missing-arm-timeout", self, compatibility) && passed;
        passed = passed && !cleanupFailed;
        std::printf("{\"scope\":\"SELF_CHILD_BOOTSTRAP_ONLY\",\"selfCheck\":%s,"
                    "\"providerReady\":false,\"minecraft\":\"NOT_EXECUTED\"}\n",
                    passed ? "true" : "false");
        return passed ? 0 : 1;
    } catch (...) {
        std::printf("{\"scope\":\"SELF_CHILD_BOOTSTRAP_ONLY\",\"selfCheck\":false,"
                    "\"error\":\"INCOMPLETE_OR_FAILED_OPERATION\",\"cleanupConfirmed\":%s,"
                    "\"providerReady\":false,\"minecraft\":\"NOT_EXECUTED\"}\n",
                    cleanupFailed ? "false" : "true");
        return 2;
    }
}
