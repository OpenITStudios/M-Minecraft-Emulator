// Synthetic native isolation proof. Never launches or loads Minecraft/Sandboxie.
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <userenv.h>
#include <sddl.h>
#include <aclapi.h>
#include <shlobj.h>
#include <objbase.h>
#include <filesystem>
#include <string>
#include <vector>
#include <cstdio>
#include <cstdint>
#include <cwctype>
#include <stdexcept>

namespace fs = std::filesystem;
bool cleanup_failed = false;
struct Failure { const char* stage; DWORD code; };
void require(bool ok, const char* stage) { if (!ok) throw Failure{stage, GetLastError()}; }
void hrcheck(HRESULT hr, const char* stage) { if (FAILED(hr)) throw Failure{stage, static_cast<DWORD>(hr)}; }
struct Handle {
    HANDLE value = nullptr;
    explicit Handle(HANDLE h = nullptr) : value(h) {}
    Handle(const Handle&) = delete;
    Handle& operator=(const Handle&) = delete;
    ~Handle() { if (value && value != INVALID_HANDLE_VALUE) CloseHandle(value); }
};
struct LocalBuffer {
    void* value = nullptr;
    ~LocalBuffer() { if (value) LocalFree(value); }
};
std::wstring environment(const wchar_t* key) {
    DWORD size = GetEnvironmentVariableW(key, nullptr, 0);
    if (!size) return L"";
    std::vector<wchar_t> value(size);
    DWORD copied = GetEnvironmentVariableW(key, value.data(), size);
    require(copied && copied < size, "environment");
    return std::wstring(value.data(), copied);
}
bool same_path(const std::wstring& a, const std::wstring& b) {
    return !a.empty() && !b.empty() && _wcsicmp(a.c_str(), b.c_str()) == 0;
}
bool under_profile(const std::wstring& path, const fs::path& profile) {
    const std::wstring prefix = profile.wstring() + L"\\";
    return same_path(path, profile.wstring()) ||
        (path.size() >= prefix.size() && _wcsnicmp(path.c_str(), prefix.c_str(), prefix.size()) == 0);
}
void write(const fs::path& path, const void* data, DWORD count) {
    Handle file(CreateFileW(path.c_str(), GENERIC_WRITE, 0, nullptr, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, nullptr));
    require(file.value != INVALID_HANDLE_VALUE, "create-fixture-file");
    DWORD written = 0;
    require(WriteFile(file.value, data, count, &written, nullptr) && written == count, "write-fixture-file");
}
bool readable(const fs::path& path) {
    Handle file(CreateFileW(path.c_str(), GENERIC_READ, FILE_SHARE_READ, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr));
    if (file.value != INVALID_HANDLE_VALUE) return true;
    require(GetLastError() == ERROR_ACCESS_DENIED, "expected-access-denied");
    return false;
}
void grant_read(const fs::path& path, PSID sid) {
    PACL old_acl = nullptr, new_acl = nullptr;
    LocalBuffer descriptor;
    DWORD status = GetNamedSecurityInfoW(path.c_str(), SE_FILE_OBJECT, DACL_SECURITY_INFORMATION,
        nullptr, nullptr, &old_acl, nullptr, &descriptor.value);
    if (status) throw Failure{"read-fixture-acl", status};
    EXPLICIT_ACCESSW entry{};
    entry.grfAccessPermissions = FILE_GENERIC_READ | FILE_GENERIC_EXECUTE;
    entry.grfAccessMode = GRANT_ACCESS;
    entry.grfInheritance = NO_INHERITANCE;
    entry.Trustee.TrusteeForm = TRUSTEE_IS_SID;
    entry.Trustee.ptstrName = static_cast<LPWSTR>(sid);
    status = SetEntriesInAclW(1, &entry, old_acl, &new_acl);
    LocalBuffer acl{new_acl};
    if (status) throw Failure{"merge-fixture-acl", status};
    status = SetNamedSecurityInfoW(const_cast<LPWSTR>(path.c_str()), SE_FILE_OBJECT,
        DACL_SECURITY_INFORMATION, nullptr, nullptr, new_acl, nullptr);
    if (status) throw Failure{"write-fixture-acl", status};
}
struct Profile {
    std::wstring name;
    fs::path folder;
    PSID sid = nullptr;
    bool created = false;
    explicit Profile(const std::wstring& value) : name(value) {
        // Existing profiles are never adopted, modified or deleted by this probe.
        hrcheck(CreateAppContainerProfile(name.c_str(), L"Mivubi synthetic isolation fixture",
            L"Temporary synthetic data only; no accounts or Minecraft", nullptr, 0, &sid), "create-new-profile");
        created = true;
        try {
            LPWSTR text = nullptr, location = nullptr;
            require(ConvertSidToStringSidW(sid, &text) != 0, "profile-sid-text");
            LocalBuffer sid_text{text};
            hrcheck(GetAppContainerFolderPath(text, &location), "profile-folder");
            folder = location;
            CoTaskMemFree(location);
        } catch (...) { cleanup(); throw; }
    }
    Profile(const Profile&) = delete;
    bool cleanup() {
        if (sid) { FreeSid(sid); sid = nullptr; }
        if (!created) return true;
        const HRESULT result = DeleteAppContainerProfile(name.c_str());
        if (SUCCEEDED(result)) created = false;
        else cleanup_failed = true;
        return SUCCEEDED(result);
    }
    ~Profile() { cleanup(); }
};
struct Child {
    PROCESS_INFORMATION process{};
    Child() = default;
    Child(const Child&) = delete;
    ~Child() {
        if (process.hProcess) {
            if (WaitForSingleObject(process.hProcess, 0) == WAIT_TIMEOUT) {
                // Only the exact self-created fixture handle, never a discovered PID.
                TerminateProcess(process.hProcess, 99);
                WaitForSingleObject(process.hProcess, 3000);
            }
            CloseHandle(process.hProcess);
            CloseHandle(process.hThread);
        }
    }
};
std::wstring quote(const fs::path& value) { return L"\"" + value.wstring() + L"\""; }
void launch(Child& child, const fs::path& binary, const fs::path& root,
    const Profile& own, const Profile& peer, bool reopen = false) {
    SIZE_T bytes = 0;
    InitializeProcThreadAttributeList(nullptr, 1, 0, &bytes);
    require(bytes > 0, "attribute-size");
    std::vector<uintptr_t> storage((bytes + sizeof(uintptr_t) - 1) / sizeof(uintptr_t));
    auto attributes = reinterpret_cast<LPPROC_THREAD_ATTRIBUTE_LIST>(storage.data());
    require(InitializeProcThreadAttributeList(attributes, 1, 0, &bytes) != 0, "attribute-init");
    struct Cleanup { LPPROC_THREAD_ATTRIBUTE_LIST value; ~Cleanup(){ DeleteProcThreadAttributeList(value); } } cleanup{attributes};
    SECURITY_CAPABILITIES security{};
    security.AppContainerSid = own.sid;
    require(UpdateProcThreadAttribute(attributes, 0, PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES,
        &security, sizeof(security), nullptr, nullptr) != 0, "attribute-security");
    STARTUPINFOEXW startup{};
    startup.StartupInfo.cb = sizeof(startup);
    startup.lpAttributeList = attributes;
    std::wstring command = quote(binary) + (reopen ? L" --child-reopen " : L" --child ")
        + quote(root) + L" " + quote(own.folder) + L" " + quote(peer.folder);
    require(CreateProcessW(binary.c_str(), command.data(), nullptr, nullptr, FALSE,
        EXTENDED_STARTUPINFO_PRESENT | CREATE_NO_WINDOW, nullptr, root.c_str(), &startup.StartupInfo,
        &child.process) != 0, "create-contained-fixture");
}
bool token_matches(HANDLE process, PSID expected) {
    HANDLE raw = nullptr;
    require(OpenProcessToken(process, TOKEN_QUERY, &raw) != 0, "open-fixture-token");
    Handle token(raw);
    DWORD flag = 0, size = 0;
    require(GetTokenInformation(raw, TokenIsAppContainer, &flag, sizeof(flag), &size) != 0, "query-container-flag");
    if (!flag) return false;
    GetTokenInformation(raw, TokenAppContainerSid, nullptr, 0, &size);
    require(size >= sizeof(TOKEN_APPCONTAINER_INFORMATION) && size <= 65536, "token-size");
    std::vector<uintptr_t> storage((size + sizeof(uintptr_t) - 1) / sizeof(uintptr_t));
    require(GetTokenInformation(raw, TokenAppContainerSid, storage.data(), size, &size) != 0, "query-container-sid");
    const auto info = reinterpret_cast<TOKEN_APPCONTAINER_INFORMATION*>(storage.data());
    return EqualSid(info->TokenAppContainer, expected) != 0;
}
struct Observation { DWORD version = 1, flags = 0; };
enum : DWORD { OWN_FILE = 1, PEER_DENIED = 2, HOST_DENIED = 4, MUTEX_PRIVATE = 8,
    LOCAL_ENV = 16, LOCAL_KNOWN_FOLDER = 32, ROAMING_ENV_PRIVATE = 64,
    ROAMING_KNOWN_AVAILABLE = 128, ROAMING_KNOWN_PRIVATE = 256,
    CUSTOM_ROAMING_OWN = 512, CUSTOM_ROAMING_PEER_DENIED = 1024,
    CUSTOM_ROAMING_CREATED = 2048, CUSTOM_ROAMING_REOPENED = 4096 };
int contained(const fs::path& root, const fs::path& own, const fs::path& peer, bool reopen) {
    HANDLE raw = nullptr;
    require(OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &raw) != 0, "child-token");
    Handle token(raw);
    DWORD flag = 0, size = 0;
    require(GetTokenInformation(raw, TokenIsAppContainer, &flag, sizeof(flag), &size) != 0 && flag,
        "child-must-be-contained");
    Handle mutex(CreateMutexW(nullptr, FALSE, L"Local\\MivubiIndependentSyntheticMutex"));
    DWORD mutex_status = GetLastError();
    require(mutex.value != nullptr, "synthetic-mutex");
    Observation observation;
    if (mutex_status != ERROR_ALREADY_EXISTS) observation.flags |= MUTEX_PRIVATE;
    if (readable(own / L"canary")) observation.flags |= OWN_FILE;
    if (!readable(peer / L"canary")) observation.flags |= PEER_DENIED;
    if (!readable(root / L"host-canary")) observation.flags |= HOST_DENIED;
    if (same_path(environment(L"LOCALAPPDATA"), own.wstring())) observation.flags |= LOCAL_ENV;
    PWSTR local = nullptr;
    hrcheck(SHGetKnownFolderPath(FOLDERID_LocalAppData, 0, nullptr, &local), "known-local-appdata");
    if (same_path(local, own.wstring())) observation.flags |= LOCAL_KNOWN_FOLDER;
    CoTaskMemFree(local);
    // Compare paths only. Never open the host roaming profile or account files.
    if (under_profile(environment(L"APPDATA"), own)) observation.flags |= ROAMING_ENV_PRIVATE;
    PWSTR roaming = nullptr;
    if (SUCCEEDED(SHGetKnownFolderPath(FOLDERID_RoamingAppData, 0, nullptr, &roaming))) {
        observation.flags |= ROAMING_KNOWN_AVAILABLE;
        if (under_profile(roaming, own)) observation.flags |= ROAMING_KNOWN_PRIVATE;
    }
    if (roaming) CoTaskMemFree(roaming);

    // Only fresh synthetic profile paths are opened. Do NOT write at APPDATA
    // or SHGetKnownFolderPath(FOLDERID_RoamingAppData): on a real Windows host,
    // those APIs may still resolve to the host's account-bearing profile.
    const auto own_roaming = own / L"synthetic-roaming";
    const auto peer_roaming = peer / L"synthetic-roaming";
    if (readable(own_roaming / L"canary")) observation.flags |= CUSTOM_ROAMING_OWN;
    if (!readable(peer_roaming / L"canary")) observation.flags |= CUSTOM_ROAMING_PEER_DENIED;
    const auto persistent = own_roaming / L"persistent-canary";
    if (reopen) {
        if (readable(persistent)) observation.flags |= CUSTOM_ROAMING_REOPENED;
    } else {
        const char value = 'r';
        write(persistent, &value, 1);
        observation.flags |= CUSTOM_ROAMING_CREATED;
    }

    write(own / (reopen ? L"observation-reopen" : L"observation"),
        &observation, sizeof(observation));
    // Bounded synthetic lifetime. No permanent worker or game control.
    const ULONGLONG deadline = GetTickCount64() + 20000;
    while (GetTickCount64() < deadline) {
        if (GetFileAttributesW((own / L"stop").c_str()) != INVALID_FILE_ATTRIBUTES) return 0;
        Sleep(20);
    }
    return 98;
}
Observation observe(const Profile& profile, Child& child, bool reopen = false) {
    const auto path = profile.folder / (reopen ? L"observation-reopen" : L"observation");
    const ULONGLONG deadline = GetTickCount64() + 5000;
    while (GetTickCount64() < deadline) {
        Handle file(CreateFileW(path.c_str(), GENERIC_READ, 0, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr));
        if (file.value != INVALID_HANDLE_VALUE) {
            Observation value; DWORD read = 0;
            require(ReadFile(file.value, &value, sizeof(value), &read, nullptr) && read == sizeof(value) && value.version == 1, "read-observation");
            return value;
        }
        DWORD status = GetLastError();
        require(status == ERROR_FILE_NOT_FOUND || status == ERROR_SHARING_VIOLATION, "observation-path");
        DWORD exit_code = 0;
        require(GetExitCodeProcess(child.process.hProcess, &exit_code) != 0, "fixture-exit-code");
        if (WaitForSingleObject(child.process.hProcess, 0) != WAIT_TIMEOUT) throw Failure{"fixture-exited-before-observation", exit_code};
        Sleep(20);
    }
    throw Failure{"observation-timeout", WAIT_TIMEOUT};
}
void stop(const Profile& profile, Child& child) {
    const char marker = 'x'; write(profile.folder / L"stop", &marker, 1);
    require(WaitForSingleObject(child.process.hProcess, 3000) == WAIT_OBJECT_0, "fixture-normal-stop");
    DWORD code = 0; require(GetExitCodeProcess(child.process.hProcess, &code) != 0 && code == 0, "fixture-normal-exit");
}
int parent(const fs::path& output, bool negative_control) {
    const auto root = fs::absolute(output / L"fixture");
    require(fs::create_directory(root), "new-fixture-directory-required");
    GUID guid{}; hrcheck(CoCreateGuid(&guid), "unique-profile-name");
    wchar_t guid_text[40]{}; require(StringFromGUID2(guid, guid_text, 40) != 0, "unique-name-format");
    std::wstring suffix;
    for (wchar_t c : std::wstring(guid_text)) if (iswalnum(c)) suffix += c;
    const auto first_name = L"Mivubi.Probe." + suffix + L".a";
    const auto second_name = L"Mivubi.Probe." + suffix + L".b";
    // Inspection-only receipt, written before mutation; never automatic cleanup authority.
    const std::wstring monikers = L"Planned synthetic monikers; not cleanup authority\n" + first_name + L"\n" + second_name + L"\n";
    write(root / L"created-profile-monikers.utf16", monikers.data(), static_cast<DWORD>(monikers.size() * sizeof(wchar_t)));
    Profile first(first_name);
    Profile second(second_name);
    require(EqualSid(first.sid, second.sid) == FALSE, "distinct-profile-sids");
    wchar_t self[32768]{};
    DWORD length = GetModuleFileNameW(nullptr, self, 32768);
    require(length && length < 32768, "fixture-image-path");
    const auto binary = root / L"ContainedFixture.exe";
    require(CopyFileW(self, binary.c_str(), TRUE) != 0, "copy-self-fixture-only");
    for (PSID sid : {first.sid, second.sid}) { grant_read(root, sid); grant_read(binary, sid); }
    const char marker = 'x';
    write(root / L"host-canary", &marker, 1);
    write(first.folder / L"canary", &marker, 1);
    write(second.folder / L"canary", &marker, 1);
    // Never reuse an account-bearing profile or change existing ACLs.
    const auto first_roaming = first.folder / L"synthetic-roaming";
    const auto second_roaming = second.folder / L"synthetic-roaming";
    require(fs::create_directory(first_roaming) && fs::create_directory(second_roaming),
        "new-private-roaming-folders");
    write(first_roaming / L"canary", &marker, 1);
    write(second_roaming / L"canary", &marker, 1);
    Child a, b;
    launch(a, binary, root, first, second);
    // Deliberately reuse token A only in the explicit red-capable synthetic control.
    launch(b, binary, root, negative_control ? first : second, first);
    require(token_matches(a.process.hProcess, first.sid) && token_matches(b.process.hProcess, second.sid), "actual-distinct-container-tokens");
    const auto left = observe(first, a), right = observe(second, b);
    const DWORD required = OWN_FILE | PEER_DENIED | HOST_DENIED | MUTEX_PRIVATE
        | CUSTOM_ROAMING_OWN | CUSTOM_ROAMING_PEER_DENIED | CUSTOM_ROAMING_CREATED;
    require((left.flags & required) == required && (right.flags & required) == required,
        "synthetic-isolation-contract");
    require(WaitForSingleObject(a.process.hProcess, 0) == WAIT_TIMEOUT && WaitForSingleObject(b.process.hProcess, 0) == WAIT_TIMEOUT, "concurrent-fixtures");
    stop(first, a);
    require(WaitForSingleObject(b.process.hProcess, 0) == WAIT_TIMEOUT, "other-fixture-survives");
    stop(second, b);
    // Two fresh child processes, using the SAME synthetic AppContainer profile
    // identities, must independently read their own previous writes.
    Child reopened_a, reopened_b;
    launch(reopened_a, binary, root, first, second, true);
    launch(reopened_b, binary, root, second, first, true);
    require(token_matches(reopened_a.process.hProcess, first.sid)
        && token_matches(reopened_b.process.hProcess, second.sid),
        "reopened-distinct-container-tokens");
    const auto reopened_left = observe(first, reopened_a, true);
    const auto reopened_right = observe(second, reopened_b, true);
    const DWORD persisted = CUSTOM_ROAMING_OWN | CUSTOM_ROAMING_PEER_DENIED
        | CUSTOM_ROAMING_REOPENED;
    require((reopened_left.flags & persisted) == persisted
        && (reopened_right.flags & persisted) == persisted,
        "private-synthetic-roaming-persistence");
    require(WaitForSingleObject(reopened_a.process.hProcess, 0) == WAIT_TIMEOUT
        && WaitForSingleObject(reopened_b.process.hProcess, 0) == WAIT_TIMEOUT,
        "concurrent-reopened-fixtures");
    stop(first, reopened_a);
    require(WaitForSingleObject(reopened_b.process.hProcess, 0) == WAIT_TIMEOUT,
        "other-reopened-fixture-survives");
    stop(second, reopened_b);
    CloseHandle(reopened_a.process.hProcess); CloseHandle(reopened_a.process.hThread); reopened_a.process = {};
    CloseHandle(reopened_b.process.hProcess); CloseHandle(reopened_b.process.hThread); reopened_b.process = {};
    // Close process/thread handles before deleting only the two profiles created above.
    CloseHandle(a.process.hProcess); CloseHandle(a.process.hThread); a.process = {};
    CloseHandle(b.process.hProcess); CloseHandle(b.process.hThread); b.process = {};
    const bool cleaned_a = first.cleanup(), cleaned_b = second.cleanup();
    require(cleaned_a && cleaned_b, "fixture-profile-cleanup");
    std::printf("{\"schema\":1,\"result\":\"PASS\",\"syntheticOnly\":true,\"distinctAppContainerTokens\":true,\"concurrentProcesses\":true,\"privateMutexNamespaces\":true,\"ownStorageReadable\":true,\"peerStorageDenied\":true,\"hostCanaryDenied\":true,\"independentNormalStop\":true,\"profilesRemoved\":true,\"localAppDataEnvironmentRedirected\":%s,\"localAppDataKnownFolderRedirected\":%s,\"roamingEnvironmentUnderContainer\":%s,\"roamingKnownFolderQuerySucceeded\":%s,\"roamingKnownFolderUnderContainer\":%s,\"syntheticPrivateRoamingWritten\":true,\"syntheticPrivateRoamingPeerDenied\":true,\"syntheticPrivateRoamingPersistsAfterRestart\":true,\"minecraftRoamingCompatibility\":\"NOT_EXECUTED\",\"minecraftAcceptance\":\"NOT_EXECUTED\"}\n",
        (left.flags & LOCAL_ENV) && (right.flags & LOCAL_ENV) ? "true" : "false",
        (left.flags & LOCAL_KNOWN_FOLDER) && (right.flags & LOCAL_KNOWN_FOLDER) ? "true" : "false",
        (left.flags & ROAMING_ENV_PRIVATE) && (right.flags & ROAMING_ENV_PRIVATE) ? "true" : "false",
        (left.flags & ROAMING_KNOWN_AVAILABLE) && (right.flags & ROAMING_KNOWN_AVAILABLE) ? "true" : "false",
        (left.flags & ROAMING_KNOWN_PRIVATE) && (right.flags & ROAMING_KNOWN_PRIVATE) ? "true" : "false");
    return 0;
}
int wmain(int argc, wchar_t** argv) {
    try {
        if (argc == 5 && std::wstring(argv[1]) == L"--child") return contained(argv[2], argv[3], argv[4], false);
        if (argc == 5 && std::wstring(argv[1]) == L"--child-reopen") return contained(argv[2], argv[3], argv[4], true);
        if (argc == 3 && std::wstring(argv[1]) == L"--run") return parent(argv[2], false);
        if (argc == 3 && std::wstring(argv[1]) == L"--negative-control") return parent(argv[2], true);
        throw Failure{"explicit-synthetic-run-required", ERROR_INVALID_PARAMETER};
    } catch (const Failure& failure) {
        std::printf("{\"schema\":1,\"result\":\"FAIL\",\"stage\":\"%s\",\"code\":%lu,\"syntheticOnly\":true,\"profilesRemoved\":%s}\n", failure.stage, failure.code, cleanup_failed ? "false" : "true");
    } catch (const std::exception&) {
        std::printf("{\"schema\":1,\"result\":\"FAIL\",\"stage\":\"filesystem-or-allocation\",\"syntheticOnly\":true}\n");
    }
    return 1;
}
