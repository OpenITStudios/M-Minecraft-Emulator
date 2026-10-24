#include "receipt.h"
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
void require(bool condition) {
    if (!condition) throw std::runtime_error("self-test operation failed");
}
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
            if (!appData && _wcsnicmp(item, prefix, (sizeof(prefix) / sizeof(wchar_t)) - 1) != 0)
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
    const std::set<std::wstring> expectedEntries{L"state.bin", L"nested", L"Roaming", L"Local", L"Registry", leaf};
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
    for (const auto& bad : {L"..\\..\\..\\guard\\state.bin", L"state.bin:stream",
                           L"one\\two\\three\\four\\five\\six\\seven\\eight\\nine.bin"}) {
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
        Handle directory(CreateFileW(known.c_str(), FILE_LIST_DIRECTORY | SYNCHRONIZE,
            FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, nullptr));
        require(directory.value != INVALID_HANDLE_VALUE
            && nativeEntries(directory.value, false) == std::set<std::wstring>{L"appdata.bin"});
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

        Handle denied;
        SetLastError(sentinel);
        require(open(&denied.value, FILE_GENERIC_READ | DELETE, &attributes, &io,
            FILE_SHARE_READ | FILE_SHARE_WRITE, options) == static_cast<NTSTATUS>(0xc00000bbUL)
            && GetLastError() == sentinel && !denied.value);
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
    }
    void afterPeer(probe::Receipt& receipt) {
        expectBytes(held.value, slotMarker(receipt, receipt.fileRound));
        Handle root(CreateFileW(directoryPath(receipt.fileLogicalRoot).c_str(),
            FILE_LIST_DIRECTORY | SYNCHRONIZE, FILE_SHARE_READ | FILE_SHARE_WRITE,
            nullptr, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, nullptr));
        require(root.value != INVALID_HANDLE_VALUE
            && nativeEntries(root.value, false) == std::set<std::wstring>{
                L"state.bin", L"nested", L"Roaming", L"Local", L"Registry", receipt.slot == 1 ? L"slot-A.bin" : L"slot-B.bin"});
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
        const auto initialize = reinterpret_cast<BOOL (*)()>(
            GetProcAddress(GetModuleHandleW(L"bootstrap-compat.dll"), "BootstrapRegistry"));
        require(initialize && initialize() && initialize());
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
        require(WaitForMultipleObjects(2, states, FALSE, 10000) == WAIT_OBJECT_0);
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
        require(SetHandleInformation(branch.value, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) != FALSE);
        Key root;
        require(RegOpenCurrentUser(KEY_READ, &root.value) == ERROR_SUCCESS);
        userPath = nativeKeyName(root.value);
    }
    void unchanged() {
        require(registryValue(branch.value, L"Marker") == 0x484f5354);
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
            && InterlockedCompareExchange(&first.mapping.value->registryChecks, 0, 0) == 8191
            && InterlockedCompareExchange(&second.mapping.value->registryChecks, 0, 0) == 8191
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
                "\"earlyDenied\":[7,7],\"earlyAllowedControl\":7,\"registryChecks\":[8191,8191],\"invalidHiveRejected\":true,\"fileChecks\":[1048575,1048575],\"rounds\":2,\"persistent\":true,\"hostUnchanged\":true,\"slotMismatchRejected\":true,\"appDataBindingRejected\":true,"
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
            require(receipt->registryChecks == 8191 && receipt->earlyPrivate == 7
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
        "\"earlyPrivate\":[7,7],\"registryChecks\":[8191,8191],\"rounds\":2,"
        "\"persistent\":true,\"hostUnchanged\":true,\"invalidBackingRejected\":3,"
        "\"applicationHiveLock\":false,\"securityBoundary\":false,\"providerReady\":false,"
        "\"minecraft\":\"NOT_EXECUTED\"}\n");
    return true;
}

int childMain() {
    probe::View view;
    if (!view.open()) return 81;
    probe::record(view.value, probe::Main);
    probe::registryCanary(probe::Main);
    if (DependencyCanary() != 7) return 82;
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
    if (argc == 2 && wcscmp(argv[1], L"--bootstrap-test-child") == 0) {
        try { return childMain(); } catch (...) { return 85; }
    }
    const bool mutexTest = argc == 2 && wcscmp(argv[1], L"--run-mutex-self-test") == 0;
    const bool ordinaryTest = argc == 2 && wcscmp(argv[1], L"--run-inherited-registry-self-test") == 0;
    if (!mutexTest && !ordinaryTest && (argc != 2 || wcscmp(argv[1], L"--run-self-test") != 0)) return 64;
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
