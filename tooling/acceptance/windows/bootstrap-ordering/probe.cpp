#include "receipt.h"
#include <detours.h>
#include <cstdio>
#include <cwchar>
#include <stdexcept>
#include <string>
#include <vector>

extern "C" __declspec(dllimport) int DependencyCanary();

void NTAPI BootstrapTls(PVOID, DWORD reason, PVOID) {
    if (reason == DLL_PROCESS_ATTACH) probe::canary(probe::Tls);
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
        if (state != WAIT_TIMEOUT || !TerminateProcess(info.hProcess, 90)) return false;
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
std::vector<wchar_t> environment(HANDLE section) {
    wchar_t* original = GetEnvironmentStringsW();
    require(original != nullptr);
    std::vector<wchar_t> result;
    try {
        for (const wchar_t* item = original; *item; item += wcslen(item) + 1) {
            constexpr wchar_t prefix[] = L"MIVUBI_BOOTSTRAP_TEST_SECTION=";
            if (_wcsnicmp(item, prefix, (sizeof(prefix) / sizeof(wchar_t)) - 1) != 0)
                result.insert(result.end(), item, item + wcslen(item) + 1);
        }
        const std::wstring setting = std::wstring(probe::SectionVariable) + L"="
            + std::to_wstring(reinterpret_cast<ULONG_PTR>(section));
        result.insert(result.end(), setting.begin(), setting.end());
        result.push_back(L'\0');
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
        std::wstring other = L"other.bin";
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
    }
    void afterPeer(probe::Receipt& receipt) {
        expectBytes(held.value, slotMarker(receipt, receipt.fileRound));
        checks |= 64;
        InterlockedExchange(&receipt.fileChecks, checks);
    }
};

int mutexChild(probe::Receipt& receipt) {
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
               DWORD round, HANDLE logicalRoot, HANDLE ownRoot) {
        ZeroMemory(mapping.value, sizeof(probe::Receipt));
        auto& receipt = *mapping.value;
        receipt.magic = probe::Magic;
        receipt.version = probe::Version;
        receipt.mode = probe::MutexCase;
        receipt.slot = slot;
        receipt.fileRound = round;
        receipt.fileLogicalRoot = logicalRoot;
        receipt.fileOwnRoot = ownRoot;
        receipt.ready = ready.value;
        receipt.go = go;
        receipt.done = done.value;
        receipt.finish = finish;
        copyName(receipt.logical, name);
        copyName(receipt.own, name + (slot == 1 ? L"_A" : L"_B"));
        copyName(receipt.peer, name + (slot == 1 ? L"_B" : L"_A"));
        HANDLE inherited[]{section.value, ready.value, go, done.value, finish, logicalRoot, ownRoot};
        Attributes attributes(inherited, 7);
        auto env = environment(section.value);
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
        require(DetourUpdateProcessWithDll(child.info.hProcess, dlls, 1) != FALSE);
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
        SECURITY_ATTRIBUTES security{sizeof(SECURITY_ATTRIBUTES), nullptr, TRUE};
        const auto open = [&](const wchar_t* leaf) {
            HANDLE result = CreateFileW((root + L"\\" + leaf).c_str(), FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY,
                FILE_SHARE_READ | FILE_SHARE_WRITE, &security, OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT, nullptr);
            require(result != INVALID_HANDLE_VALUE);
            return result;
        };
        logical.value = open(L"logical"); a.value = open(L"A"); b.value = open(L"B");
        for (const auto& leaf : {L"logical\\state.bin", L"outside.bin"}) {
            Handle file(CreateFileW((root + L"\\" + leaf).c_str(), GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, nullptr));
            require(file.value != INVALID_HANDLE_VALUE);
            writeBytes(file.value, wcscmp(leaf, L"outside.bin") == 0 ? "OUTSIDE" : "HOST");
        }
    }
    void verify(DWORD round) {
        expectFile(root + L"\\logical\\state.bin", "HOST");
        expectFile(root + L"\\outside.bin", "OUTSIDE");
        expectFile(root + L"\\A\\state.bin", "A" + std::to_string(round + 1));
        expectFile(root + L"\\B\\state.bin", "B" + std::to_string(round + 1));
    }
    // Preserve only newly created synthetic files; no recursive cleanup or user data reads.
};

bool mutexPair(const std::wstring& self, const std::string& compatibility) {
    FileRoots files;
    const std::wstring name = L"Local\\MivubiMutexProbe_" + std::to_wstring(GetCurrentProcessId())
        + L"_" + std::to_wstring(GetTickCount64());
    Handle host(CreateMutexW(nullptr, TRUE, name.c_str()));
    const DWORD hostError = GetLastError();
    require(host.value && hostError != ERROR_ALREADY_EXISTS);
    {
        Handle go(sharedEvent()), finish(sharedEvent());
        MutexChild mismatch(1, name, go.value, finish.value, self, compatibility,
            0, files.logical.value, files.b.value);
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
    for (DWORD round = 0; round < 2; ++round) {
    Handle go(sharedEvent()), finish(sharedEvent());
    MutexChild first(1, name, go.value, finish.value, self, compatibility,
        round, files.logical.value, files.a.value);
    MutexChild second(2, name, go.value, finish.value, self, compatibility,
        round, files.logical.value, files.b.value);
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
    require(SetEvent(go.value) != FALSE);
    first.phase(first.done.value);
    second.phase(second.done.value);
    require(InterlockedCompareExchange(&first.mapping.value->mutexChecks, 0, 0) == 127
        && InterlockedCompareExchange(&second.mapping.value->mutexChecks, 0, 0) == 127
        && InterlockedCompareExchange(&first.mapping.value->nativeChecks, 0, 0) == 511
        && InterlockedCompareExchange(&second.mapping.value->nativeChecks, 0, 0) == 511
        && InterlockedCompareExchange(&first.mapping.value->fileChecks, 0, 0) == 127
        && InterlockedCompareExchange(&second.mapping.value->fileChecks, 0, 0) == 127);
    require(SetEvent(finish.value) != FALSE);
    first.exitedNormally();
    second.exitedNormally();
    for (const auto& suffix : {L"_A", L"_B"}) {
        Handle gone(OpenMutexW(SYNCHRONIZE, FALSE, (name + suffix).c_str()));
        const DWORD error = GetLastError();
        require(!gone.value && error == ERROR_FILE_NOT_FOUND);
    }
    files.verify(round);
    }
    require(ReleaseMutex(host.value) != FALSE);
    std::printf("{\"case\":\"mutex-two-child\",\"selfCheck\":true,\"checks\":[127,127],"
                "\"fileChecks\":[127,127],\"rounds\":2,\"persistent\":true,\"hostUnchanged\":true,\"slotMismatchRejected\":true,"
                "\"nativeChecks\":[511,511],\"distinctHostAndPeerObjects\":true,\"rawPeerAccessible\":true,"
                "\"securityBoundary\":false,\"providerReady\":false,"
                "\"minecraft\":\"NOT_EXECUTED\"}\n");
    return true;
}

int childMain() {
    probe::View view;
    if (!view.open()) return 81;
    probe::record(view.value, probe::Main);
    if (DependencyCanary() != 7) return 82;
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
    if (!mutexTest && (argc != 2 || wcscmp(argv[1], L"--run-self-test") != 0)) return 64;
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
