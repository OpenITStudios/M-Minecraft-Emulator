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
    ULONGLONG created_file_time = 0;
    // Borrowed only from the fresh Profile passed to launch(). Profile
    // lifetime encloses all children in parent(); no second owner database.
    const Profile* owner = nullptr;
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
// Query only the exact self-created process handle, not discovered PIDs.
ULONGLONG creation_filetime(HANDLE process) {
    FILETIME created{}, exited{}, kernel{}, user{};
    require(GetProcessTimes(process, &created, &exited, &kernel, &user) != 0,
        "fixture-creation-time");
    return (static_cast<ULONGLONG>(created.dwHighDateTime) << 32)
        | static_cast<ULONGLONG>(created.dwLowDateTime);
}
void launch(Child& child, const fs::path& binary, const fs::path& root,
    const Profile& own, const Profile& peer, bool reopen = false) {
    require(child.process.hProcess == nullptr && child.owner == nullptr
        && own.created && own.sid != nullptr && !own.folder.empty(),
        "new-owned-fixture-child-required");
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
    child.created_file_time = creation_filetime(child.process.hProcess);
    require(child.created_file_time != 0, "invalid-owned-fixture-creation");
    // Publish the owner only after process creation and FILETIME succeed.
    child.owner = &own;
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
// Never read a record on behalf of a different/reused or already dead PID.
// These checks apply only to the exact child spawned by this fixture parent.
const Profile& owned_profile(const Child& child) {
    require(child.owner != nullptr && child.owner->created
        && child.owner->sid != nullptr && !child.owner->folder.empty(),
        "fixture-owner-not-current");
    return *child.owner;
}
void verify_created_child(const Child& child) {
    const auto& expected = owned_profile(child);
    require(child.process.hProcess != nullptr && child.created_file_time != 0,
        "fixture-missing-owned-handle");
    require(WaitForSingleObject(child.process.hProcess, 0) == WAIT_TIMEOUT,
        "fixture-not-live");
    require(creation_filetime(child.process.hProcess) == child.created_file_time,
        "fixture-creation-changed");
    require(token_matches(child.process.hProcess, expected.sid), "fixture-token-mismatch");
}
struct Observation { DWORD version = 1, flags = 0; };
static_assert(sizeof(DWORD) == 4 && sizeof(Observation) == 8,
    "Synthetic fixture frame must match the Rust 8-byte decoder");
enum : DWORD { OWN_FILE = 1, PEER_DENIED = 2, HOST_DENIED = 4, MUTEX_PRIVATE = 8,
    LOCAL_ENV = 16, LOCAL_KNOWN_FOLDER = 32, ROAMING_ENV_PRIVATE = 64,
    ROAMING_KNOWN_AVAILABLE = 128, ROAMING_KNOWN_PRIVATE = 256,
    CUSTOM_ROAMING_OWN = 512, CUSTOM_ROAMING_PEER_DENIED = 1024,
    CUSTOM_ROAMING_CREATED = 2048, CUSTOM_ROAMING_REOPENED = 4096,
    ROAMING_ENV_AVAILABLE = 8192 };
const char* status(bool queried, bool inside) {
    if (!queried) return "UNAVAILABLE";
    return inside ? "UNDER_EXPECTED_PROFILE" : "OUTSIDE_EXPECTED_PROFILE";
}
const char* local_status(const Observation& o) {
    return status(true, (o.flags & LOCAL_KNOWN_FOLDER) != 0);
}
const char* env_status(const Observation& o) {
    return status((o.flags & ROAMING_ENV_AVAILABLE) != 0,
        (o.flags & ROAMING_ENV_PRIVATE) != 0);
}
const char* roaming_status(const Observation& o) {
    return status((o.flags & ROAMING_KNOWN_AVAILABLE) != 0,
        (o.flags & ROAMING_KNOWN_PRIVATE) != 0);
}
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
    const auto roaming_env = environment(L"APPDATA");
    if (!roaming_env.empty()) observation.flags |= ROAMING_ENV_AVAILABLE;
    if (under_profile(roaming_env, own)) observation.flags |= ROAMING_ENV_PRIVATE;
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
// Opt-in, fixture-only C++ -> Rust OS capability handoff. Only the
// original owner chooses child and opened record; Rust receives no path,
// profile SID, username, credentials, process launch authority or token.
void verify_rust_handoff(const fs::path& helper, const Child& child,
    HANDLE file, bool reopen) {
    verify_created_child(child);
    require(fs::is_regular_file(helper) && helper.wstring().find(L'"') == std::wstring::npos
        && helper.wstring().find(L'\r') == std::wstring::npos
        && helper.wstring().find(L'\n') == std::wstring::npos,
        "rust-handoff-helper-path");
    HANDLE child_copy = nullptr;
    require(DuplicateHandle(GetCurrentProcess(), child.process.hProcess,
        GetCurrentProcess(), &child_copy,
        PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE, TRUE, 0) != 0,
        "rust-handoff-duplicate-child");
    Handle copied_process(child_copy);
    HANDLE file_copy = nullptr;
    require(DuplicateHandle(GetCurrentProcess(), file, GetCurrentProcess(),
        &file_copy, 0, TRUE, DUPLICATE_SAME_ACCESS) != 0,
        "rust-handoff-duplicate-file");
    Handle copied_file(file_copy);
    HANDLE allowlist[] = {copied_process.value, copied_file.value};
    SIZE_T attr_bytes = 0;
    InitializeProcThreadAttributeList(nullptr, 1, 0, &attr_bytes);
    require(attr_bytes != 0, "rust-handoff-attribute-size");
    std::vector<uintptr_t> aligned(
        (attr_bytes + sizeof(uintptr_t) - 1) / sizeof(uintptr_t));
    auto attrs = reinterpret_cast<LPPROC_THREAD_ATTRIBUTE_LIST>(aligned.data());
    require(InitializeProcThreadAttributeList(attrs, 1, 0, &attr_bytes) != 0,
        "rust-handoff-attributes");
    struct Cleanup {
        LPPROC_THREAD_ATTRIBUTE_LIST ptr;
        ~Cleanup() { DeleteProcThreadAttributeList(ptr); }
    } cleanup{attrs};
    require(UpdateProcThreadAttribute(attrs, 0, PROC_THREAD_ATTRIBUTE_HANDLE_LIST,
        allowlist, sizeof(allowlist), nullptr, nullptr) != 0,
        "rust-handoff-handle-list");

    STARTUPINFOEXW startup{};
    startup.StartupInfo.cb = sizeof(startup);
    startup.lpAttributeList = attrs;
    const auto command = quote(helper) + L" --verify-inherited-fixture "
        + std::to_wstring(reinterpret_cast<uintptr_t>(copied_process.value)) + L" "
        + std::to_wstring(reinterpret_cast<uintptr_t>(copied_file.value)) + L" "
        + std::to_wstring(child.process.dwProcessId) + L" "
        + std::to_wstring(child.created_file_time)
        + (reopen ? L" reopened" : L" first");
    std::wstring mutable_command = command;
    PROCESS_INFORMATION helper_info{};
    require(CreateProcessW(helper.c_str(), mutable_command.data(),
        nullptr, nullptr, TRUE, EXTENDED_STARTUPINFO_PRESENT | CREATE_NO_WINDOW,
        nullptr, nullptr, &startup.StartupInfo, &helper_info) != 0,
        "rust-handoff-start");
    Handle helper_process(helper_info.hProcess), helper_thread(helper_info.hThread);
    const auto wait = WaitForSingleObject(helper_process.value, 5000);
    if (wait != WAIT_OBJECT_0) {
        // Only the precisely launched synthetic Rust helper may be killed.
        TerminateProcess(helper_process.value, 99);
        WaitForSingleObject(helper_process.value, 3000);
        throw Failure{"rust-handoff-timeout", WAIT_TIMEOUT};
    }
    DWORD exit_code = 0;
    require(GetExitCodeProcess(helper_process.value, &exit_code) != 0
        && exit_code == 0, "rust-handoff-rejected");
    verify_created_child(child);
}

Observation observe(Child& child, bool reopen = false,
    const fs::path* rust_helper = nullptr) {
    // The child and fresh profile are permanently paired by launch().
    // Never accept a second caller-selected path or profile for this child.
    verify_created_child(child);
    const auto& profile = owned_profile(child);
    const auto path = profile.folder / (reopen ? L"observation-reopen" : L"observation");
    const ULONGLONG deadline = GetTickCount64() + 5000;
    while (GetTickCount64() < deadline) {
        Handle file(CreateFileW(path.c_str(), GENERIC_READ, 0, nullptr, OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT, nullptr));
        if (file.value != INVALID_HANDLE_VALUE) {
            BY_HANDLE_FILE_INFORMATION before{}, after{};
            require(GetFileInformationByHandle(file.value, &before) != 0,
                "fixture-file-identity");
            const DWORD not_regular = FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT;
            require((before.dwFileAttributes & not_regular) == 0
                    && before.nFileSizeHigh == 0
                    && before.nFileSizeLow == sizeof(Observation),
                "fixture-not-regular-eight-byte-file");
            Observation value{}; DWORD read = 0;
            require(ReadFile(file.value, &value, sizeof(value), &read, nullptr)
                    && read == sizeof(value) && value.version == 1,
                "read-observation");
            require(GetFileInformationByHandle(file.value, &after) != 0,
                "fixture-file-identity-recheck");
            require(before.dwVolumeSerialNumber == after.dwVolumeSerialNumber
                    && before.nFileIndexHigh == after.nFileIndexHigh
                    && before.nFileIndexLow == after.nFileIndexLow
                    && before.nFileSizeHigh == after.nFileSizeHigh
                    && before.nFileSizeLow == after.nFileSizeLow
                    && (after.dwFileAttributes & not_regular) == 0,
                "fixture-file-changed");
            verify_created_child(child);
            if (rust_helper) verify_rust_handoff(*rust_helper, child, file.value, reopen);
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
void stop(Child& child) {
    verify_created_child(child);
    const auto& profile = owned_profile(child);
    const char marker = 'x'; write(profile.folder / L"stop", &marker, 1);
    require(WaitForSingleObject(child.process.hProcess, 3000) == WAIT_OBJECT_0, "fixture-normal-stop");
    DWORD code = 0; require(GetExitCodeProcess(child.process.hProcess, &code) != 0 && code == 0, "fixture-normal-exit");
}
int parent(const fs::path& output, bool negative_control,
    const fs::path* rust_helper = nullptr) {
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
    // Profile objects were constructed before these borrowed owner leases;
    // explicit process-handle release precedes profile cleanup below.
    Child a, b;
    launch(a, binary, root, first, second);
    // Deliberately reuse token A only in the explicit red-capable synthetic control.
    launch(b, binary, root, negative_control ? first : second, first);
    require(token_matches(a.process.hProcess, first.sid) && token_matches(b.process.hProcess, second.sid), "actual-distinct-container-tokens");
    const auto left = observe(a, false, rust_helper);
    const auto right = observe(b, false, rust_helper);
    const DWORD required = OWN_FILE | PEER_DENIED | HOST_DENIED | MUTEX_PRIVATE
        | CUSTOM_ROAMING_OWN | CUSTOM_ROAMING_PEER_DENIED | CUSTOM_ROAMING_CREATED;
    require((left.flags & required) == required && (right.flags & required) == required,
        "synthetic-isolation-contract");
    require(WaitForSingleObject(a.process.hProcess, 0) == WAIT_TIMEOUT && WaitForSingleObject(b.process.hProcess, 0) == WAIT_TIMEOUT, "concurrent-fixtures");
    stop(a);
    require(WaitForSingleObject(b.process.hProcess, 0) == WAIT_TIMEOUT, "other-fixture-survives");
    stop(b);
    // Normal-stop sentinels belong only to this run. Remove them before a
    // second launch; otherwise the reopened child would immediately exit and
    // produce a false persistence/lifetime failure. Preserve both canaries.
    require(DeleteFileW((first.folder / L"stop").c_str()) != 0,
        "clear-first-fixture-stop");
    require(DeleteFileW((second.folder / L"stop").c_str()) != 0,
        "clear-second-fixture-stop");
    // Two fresh child processes, using the SAME synthetic AppContainer profile
    // identities, must independently read their own previous writes.
    Child reopened_a, reopened_b;
    launch(reopened_a, binary, root, first, second, true);
    launch(reopened_b, binary, root, second, first, true);
    require(token_matches(reopened_a.process.hProcess, first.sid)
        && token_matches(reopened_b.process.hProcess, second.sid),
        "reopened-distinct-container-tokens");
    const auto reopened_left = observe(reopened_a, true, rust_helper);
    const auto reopened_right = observe(reopened_b, true, rust_helper);
    const DWORD persisted = CUSTOM_ROAMING_OWN | CUSTOM_ROAMING_PEER_DENIED
        | CUSTOM_ROAMING_REOPENED;
    require((reopened_left.flags & persisted) == persisted
        && (reopened_right.flags & persisted) == persisted,
        "private-synthetic-roaming-persistence");
    require(WaitForSingleObject(reopened_a.process.hProcess, 0) == WAIT_TIMEOUT
        && WaitForSingleObject(reopened_b.process.hProcess, 0) == WAIT_TIMEOUT,
        "concurrent-reopened-fixtures");
    stop(reopened_a);
    require(WaitForSingleObject(reopened_b.process.hProcess, 0) == WAIT_TIMEOUT,
        "other-reopened-fixture-survives");
    stop(reopened_b);
    CloseHandle(reopened_a.process.hProcess); CloseHandle(reopened_a.process.hThread); reopened_a.process = {};
    CloseHandle(reopened_b.process.hProcess); CloseHandle(reopened_b.process.hThread); reopened_b.process = {};
    // Close process/thread handles before deleting only the two profiles created above.
    CloseHandle(a.process.hProcess); CloseHandle(a.process.hThread); a.process = {};
    CloseHandle(b.process.hProcess); CloseHandle(b.process.hThread); b.process = {};
    const bool cleaned_a = first.cleanup(), cleaned_b = second.cleanup();
    require(cleaned_a && cleaned_b, "fixture-profile-cleanup");
    // Both observations originate inside their own synthetic child tokens.
    // Fixture labels are NOT production PID/FILETIME or account identities.
    std::printf("{\"schema\":2,\"result\":\"PASS\",\"syntheticOnly\":true,\"distinctAppContainerTokens\":true,\"concurrentProcesses\":true,\"privateMutexNamespaces\":true,\"ownStorageReadable\":true,\"peerStorageDenied\":true,\"hostCanaryDenied\":true,\"independentNormalStop\":true,\"profilesRemoved\":true,\"localAppDataEnvironmentRedirected\":%s,\"localAppDataKnownFolderRedirected\":%s,\"roamingEnvironmentUnderContainer\":%s,\"roamingKnownFolderQuerySucceeded\":%s,\"roamingKnownFolderUnderContainer\":%s,\"syntheticPrivateRoamingWritten\":true,\"syntheticPrivateRoamingPeerDenied\":true,\"syntheticPrivateRoamingPersistsAfterRestart\":true,\"processPaths\":[{\"fixture\":\"A\",\"localKnownFolder\":\"%s\",\"roamingEnvironment\":\"%s\",\"roamingKnownFolder\":\"%s\"},{\"fixture\":\"B\",\"localKnownFolder\":\"%s\",\"roamingEnvironment\":\"%s\",\"roamingKnownFolder\":\"%s\"}],\"minecraftRoamingCompatibility\":\"NOT_EXECUTED\",\"minecraftAcceptance\":\"NOT_EXECUTED\"}\n",
        (left.flags & LOCAL_ENV) && (right.flags & LOCAL_ENV) ? "true" : "false",
        (left.flags & LOCAL_KNOWN_FOLDER) && (right.flags & LOCAL_KNOWN_FOLDER) ? "true" : "false",
        (left.flags & ROAMING_ENV_PRIVATE) && (right.flags & ROAMING_ENV_PRIVATE) ? "true" : "false",
        (left.flags & ROAMING_KNOWN_AVAILABLE) && (right.flags & ROAMING_KNOWN_AVAILABLE) ? "true" : "false",
        (left.flags & ROAMING_KNOWN_PRIVATE) && (right.flags & ROAMING_KNOWN_PRIVATE) ? "true" : "false",
        local_status(left), env_status(left), roaming_status(left),
        local_status(right), env_status(right), roaming_status(right));
    return 0;
}
// Application-hive FEASIBILITY only: no HKCU/HKU root handles are used.
// All registry state is written through HKEY values returned for two
// freshly created hive files under a new, fixture-owned directory.
// These two handles in ONE process do not establish peer-process denial.
struct AppHive {
    HKEY key = nullptr;
    AppHive() = default;
    AppHive(const AppHive&) = delete;
    AppHive& operator=(const AppHive&) = delete;
    ~AppHive() { if (key) RegCloseKey(key); }
    void close() {
        require(key != nullptr, "apphive-not-open");
        const auto held = key;
        key = nullptr;
        const auto result = RegCloseKey(held);
        if (result != ERROR_SUCCESS) throw Failure{"apphive-close", static_cast<DWORD>(result)};
    }
};
void apphive_status(LSTATUS result, const char* stage) {
    if (result != ERROR_SUCCESS) throw Failure{stage, static_cast<DWORD>(result)};
}
void load_fixture_apphive(const fs::path& file, AppHive& hive,
    REGSAM access = KEY_QUERY_VALUE | KEY_SET_VALUE) {
    require(hive.key == nullptr, "apphive-not-fresh-handle");
    // REG_PROCESS_APPKEY limits simultaneous loads by other callers,
    // but does NOT transparently virtualize the game's HKCU.
    apphive_status(RegLoadAppKeyW(
        file.c_str(), &hive.key, access,
        REG_PROCESS_APPKEY, 0), "apphive-load");
    require(hive.key != nullptr, "apphive-missing-root-handle");
}
constexpr wchar_t APPHIVE_MARKER_NAME[] = L"MivubiSyntheticMarker";
void write_apphive_marker(AppHive& hive, DWORD marker) {
    require(hive.key != nullptr, "apphive-write-without-root");
    apphive_status(RegSetValueExW(
        hive.key, APPHIVE_MARKER_NAME, 0, REG_DWORD,
        reinterpret_cast<const BYTE*>(&marker), sizeof(marker)), "apphive-set-marker");
}
bool apphive_marker_matches(const AppHive& hive, DWORD wanted) {
    require(hive.key != nullptr, "apphive-read-without-root");
    DWORD type = 0, count = sizeof(DWORD), value = 0;
    apphive_status(RegQueryValueExW(
        hive.key, APPHIVE_MARKER_NAME, nullptr, &type,
        reinterpret_cast<BYTE*>(&value), &count), "apphive-read-marker");
    return type == REG_DWORD && count == sizeof(DWORD) && value == wanted;
}
void verify_apphive_file(const fs::path& root, const fs::path& file) {
    require(file.parent_path() == root, "apphive-outside-owned-root");
    const auto attrs = GetFileAttributesW(file.c_str());
    require(attrs != INVALID_FILE_ATTRIBUTES
        && (attrs & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT)) == 0,
        "apphive-file-not-regular");
}
// This child has NO pathname argument. Its cwd is the NEW fixture hive
// directory chosen by the already trusted parent. It reads synthetic marker
// values only and never calls registry write APIs or ordinary HKCU/HKU.
int apphive_readonly_peer_child(const std::wstring& choice) {
    require(choice == L"A" || choice == L"B", "apphive-child-choice");
    const auto root = fs::current_path();
    require(root.filename() == L"mivubi-new-apphives"
        && root.parent_path().filename() == L"registry-fixture",
        "apphive-child-unowned-workdir");
    for (const auto& directory : {root, root.parent_path()}) {
        const auto attrs = GetFileAttributesW(directory.c_str());
        require(attrs != INVALID_FILE_ATTRIBUTES
            && (attrs & FILE_ATTRIBUTE_DIRECTORY) != 0
            && (attrs & FILE_ATTRIBUTE_REPARSE_POINT) == 0,
            "apphive-child-reparse-directory");
    }
    const auto a = root / L"synthetic-A.hiv";
    const auto b = root / L"synthetic-B.hiv";
    verify_apphive_file(root, a);
    verify_apphive_file(root, b);

    const bool choose_a = choice == L"A";
    const auto own = choose_a ? a : b;
    const auto peer = choose_a ? b : a;
    const DWORD marker = choose_a ? 0xA11CE001 : 0xB11CE002;
    AppHive own_hive;
    // The parent has closed the child's OWN hive. Loading/reading its
    // existing value must succeed without parent intervention or writes.
    load_fixture_apphive(own, own_hive, KEY_QUERY_VALUE);
    require(apphive_marker_matches(own_hive, marker), "apphive-child-own-marker");

    // The other hive is still held by the parent using REG_PROCESS_APPKEY.
    // Its refusal measures only process-scoped load exclusivity, not ACL
    // isolation, HKCU redirection or inaccessible account data.
    HKEY peer_key = nullptr;
    const auto result = RegLoadAppKeyW(
        peer.c_str(), &peer_key, KEY_QUERY_VALUE, REG_PROCESS_APPKEY, 0);
    if (peer_key) {
        RegCloseKey(peer_key);
        throw Failure{"apphive-peer-load-returned-handle", ERROR_INVALID_DATA};
    }
    if (result == ERROR_SUCCESS) {
        throw Failure{"apphive-peer-load-unexpected-success", ERROR_INVALID_DATA};
    }
    // Unknown failures cannot be misreported as an observed load lock.
    require(result == ERROR_ACCESS_DENIED || result == ERROR_SHARING_VIOLATION
        || result == ERROR_LOCK_VIOLATION || result == ERROR_BUSY,
        "apphive-peer-load-not-a-recognized-refusal");
    own_hive.close();
    return 0;
}

void run_apphive_peer_subprocess(const fs::path& root, const wchar_t* own) {
    wchar_t binary[32768]{};
    const auto length = GetModuleFileNameW(nullptr, binary, 32768);
    require(length != 0 && length < 32768, "apphive-peer-fixture-executable");
    const fs::path image(binary);
    // Child command line carries only a fixed A/B choice, not hive paths.
    std::wstring command = quote(image) + L" --apphive-peer-child " + own;
    STARTUPINFOW startup{};
    startup.cb = sizeof(startup);
    PROCESS_INFORMATION process{};
    require(CreateProcessW(image.c_str(), command.data(), nullptr, nullptr,
        FALSE, CREATE_NO_WINDOW, nullptr, root.c_str(), &startup, &process) != 0,
        "apphive-peer-create-child");
    Handle owned_process(process.hProcess), owned_thread(process.hThread);
    const auto observed = WaitForSingleObject(owned_process.value, 5000);
    if (observed != WAIT_OBJECT_0) {
        // Only this exact disposable test child is eligible for termination.
        TerminateProcess(owned_process.value, 99);
        WaitForSingleObject(owned_process.value, 3000);
        throw Failure{"apphive-peer-child-timeout", WAIT_TIMEOUT};
    }
    DWORD exit_code = 0;
    require(GetExitCodeProcess(owned_process.value, &exit_code) != 0
        && exit_code == 0, "apphive-peer-child-rejected");
}

int run_apphive_fixture(const fs::path& output, bool verify_peer_load = false) {
    const auto root = fs::absolute(output / L"mivubi-new-apphives");
    // Never adopt an existing tree or hive. Output is a new random workspace
    // directory created by the opt-in PowerShell fixture runner.
    require(fs::exists(output) && fs::is_directory(output)
        && !fs::is_symlink(output), "apphive-output-not-owned");
    require(fs::create_directory(root), "apphive-fresh-root-required");
    const auto a = root / L"synthetic-A.hiv";
    const auto b = root / L"synthetic-B.hiv";
    require(!fs::exists(a) && !fs::exists(b), "apphive-preexisting-file");
    constexpr DWORD MARKER_A = 0xA11CE001;
    constexpr DWORD MARKER_B = 0xB11CE002;
    {
        AppHive first, second;
        load_fixture_apphive(a, first);
        verify_apphive_file(root, a);
        load_fixture_apphive(b, second);
        verify_apphive_file(root, b);
        write_apphive_marker(first, MARKER_A);
        write_apphive_marker(second, MARKER_B);
        require(apphive_marker_matches(first, MARKER_A)
            && apphive_marker_matches(second, MARKER_B)
            && !apphive_marker_matches(first, MARKER_B)
            && !apphive_marker_matches(second, MARKER_A),
            "apphive-handle-separation");
        apphive_status(RegFlushKey(first.key), "apphive-flush-a");
        apphive_status(RegFlushKey(second.key), "apphive-flush-b");
        first.close();
        second.close();
    }
    // Both hive roots have been closed. This tests handle-scope restart
    // persistence, NOT a new Windows process, reboot or game account.
    {
        AppHive first, second;
        load_fixture_apphive(a, first);
        load_fixture_apphive(b, second);
        require(apphive_marker_matches(first, MARKER_A)
            && apphive_marker_matches(second, MARKER_B),
            "apphive-persistence-after-reload");
        first.close();
        second.close();
    }
    verify_apphive_file(root, a);
    verify_apphive_file(root, b);
    if (verify_peer_load) {
        // Test both directions: independently load the child's own hive
        // while the parent holds only the OTHER hive's process appkey lock.
        {
            AppHive parent_b;
            load_fixture_apphive(b, parent_b);
            run_apphive_peer_subprocess(root, L"A");
            parent_b.close();
        }
        {
            AppHive parent_a;
            load_fixture_apphive(a, parent_a);
            run_apphive_peer_subprocess(root, L"B");
            parent_a.close();
        }
        // A lock refusal is not a registry ACL or namespace isolation proof.
        std::printf(
            "{\"schema\":2,\"result\":\"PASS\",\"syntheticOnly\":true,"
            "\"twoNewAppHives\":true,\"separateHandleMarkers\":true,"
            "\"persistedAfterHandleReload\":true,"
            "\"bidirectionalProcessLoadLocks\":\"OBSERVED\","
            "\"peerProcessDenial\":\"NOT_EXECUTED\","
            "\"hostHKCUIsolation\":\"NOT_EXECUTED\","
            "\"minecraftAcceptance\":\"NOT_EXECUTED\","
            "\"isolatedLaunchAllowed\":false}\n");
        return 0;
    }
    // Do not claim cross-process denial or real registry redirection.
    std::printf(
        "{\"schema\":1,\"result\":\"PASS\",\"syntheticOnly\":true,"
        "\"twoNewAppHives\":true,\"separateHandleMarkers\":true,"
        "\"persistedAfterHandleReload\":true,"
        "\"peerProcessDenial\":\"NOT_EXECUTED\","
        "\"hostHKCUIsolation\":\"NOT_EXECUTED\","
        "\"minecraftAcceptance\":\"NOT_EXECUTED\","
        "\"isolatedLaunchAllowed\":false}\n");
    // Leave ONLY new fixture-owned files under the unique workspace output
    // for manual forensic review; never delete old account-bearing hives.
    return 0;
}

int wmain(int argc, wchar_t** argv) {
    // Silent, read-only subcommand: even an expected failure MUST NOT
    // contaminate the parent's single JSON receipt on inherited stdout.
    if (argc == 3 && std::wstring(argv[1]) == L"--apphive-peer-child") {
        try { return apphive_readonly_peer_child(argv[2]); }
        catch (...) { return 18; }
    }
    try {
        if (argc == 5 && std::wstring(argv[1]) == L"--child") return contained(argv[2], argv[3], argv[4], false);
        if (argc == 5 && std::wstring(argv[1]) == L"--child-reopen") return contained(argv[2], argv[3], argv[4], true);
        if (argc == 3 && std::wstring(argv[1]) == L"--run") return parent(argv[2], false);
        if (argc == 3 && std::wstring(argv[1]) == L"--apphive-fixture") return run_apphive_fixture(argv[2]);
        if (argc == 3 && std::wstring(argv[1]) == L"--apphive-peer-fixture") return run_apphive_fixture(argv[2], true);
        if (argc == 4 && std::wstring(argv[1]) == L"--run-rust-handoff") {
            const auto helper = fs::absolute(argv[3]);
            return parent(argv[2], false, &helper);
        }
        if (argc == 3 && std::wstring(argv[1]) == L"--negative-control") return parent(argv[2], true);
        throw Failure{"explicit-synthetic-run-required", ERROR_INVALID_PARAMETER};
    } catch (const Failure& failure) {
        std::printf("{\"schema\":1,\"result\":\"FAIL\",\"stage\":\"%s\",\"code\":%lu,\"syntheticOnly\":true,\"profilesRemoved\":%s}\n", failure.stage, failure.code, cleanup_failed ? "false" : "true");
    } catch (const std::exception&) {
        std::printf("{\"schema\":1,\"result\":\"FAIL\",\"stage\":\"filesystem-or-allocation\",\"syntheticOnly\":true}\n");
    }
    return 1;
}
