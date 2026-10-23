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
    explicit Attributes(HANDLE* inherited) {
        SIZE_T size = 0;
        InitializeProcThreadAttributeList(nullptr, 1, 0, &size);
        require(size != 0);
        bytes.resize(size);
        value = reinterpret_cast<LPPROC_THREAD_ATTRIBUTE_LIST>(bytes.data());
        require(InitializeProcThreadAttributeList(value, 1, 0, &size) != FALSE);
        if (!UpdateProcThreadAttribute(value, 0, PROC_THREAD_ATTRIBUTE_HANDLE_LIST,
                                       inherited, sizeof(HANDLE), nullptr, nullptr)) {
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
int childMain() {
    probe::View view;
    if (!view.open()) return 81;
    probe::record(view.value, probe::Main);
    if (DependencyCanary() != 7) return 82;
    if (view.value->mode == probe::Timeout) {
        Sleep(INFINITE); // Deliberate owned-child timeout control, outside loader lock.
        return 83;
    }
    return view.value->mode == probe::Normal && view.value->armed != 1 ? 84 : 0;
}
} // namespace

int wmain(int argc, wchar_t** argv) {
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
    if (argc == 2 && wcscmp(argv[1], L"--bootstrap-test-child") == 0) return childMain();
    if (argc != 2 || wcscmp(argv[1], L"--run-self-test") != 0) return 64;
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
