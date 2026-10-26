/*
 * MIVUBI app-owned isolation / Sandboxie-derived runtime ABI probe.
 * Copyright (c) MIVUBI contributors. Upstream Sandboxie copyrights stay
 * in the upstream source and LICENSES/; no upstream component is relinked.
 *
 * Windows-only, read-only diagnostic for an isolated LAB Windows host.
 * NEVER installs, stops, starts or deletes any service, driver or box.
 * --abi performs an explicit, signed-DLL load ONLY after the SCM image-path
 * checks below; --offline does not load/invoke any upstream DLL.
 *
 * Upstream: sandboxie-plus/Sandboxie v1.18.5,
 * commit abd4c240031cfb46666daa74ca47ed4e48b561e4.
 * MY_ABI_VERSION 0x57230; SbieApi_GetVersionEx(WCHAR[16], ULONG*).
 */
#ifndef UNICODE
#define UNICODE
#endif
#ifndef _UNICODE
#define _UNICODE
#endif
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <winsvc.h>
#include <wintrust.h>
#include <Softpub.h>
#include <filesystem>
#include <string>
#include <array>
#include <algorithm>
#include <cwctype>
#include <vector>
#include <cstdint>
#include <cwchar>
#include <iostream>

// The same pinned source header is shared by SbieDrv/SbieSvc/SbieDll.
// Do not duplicate ABI numbers independently of the actual fork source.
#include "common/my_version.h"

#pragma comment(lib, "advapi32.lib")
#pragma comment(lib, "wintrust.lib")

namespace fs = std::filesystem;
namespace {
static_assert(MY_ABI_VERSION == 0x57230, "Unexpected pinned Sandboxie ABI");
static_assert(VERSION_MJR == 5 && VERSION_MIN == 73 && VERSION_REV == 5,
              "Unexpected pinned Sandboxie release");
#if defined(MIVUBI_EXPECT_PRIVATE_SOURCE) && !defined(MIVUBI_PRIVATE_ISOLATION_BUILD)
#error A_private_source_probe_requires_the_owned_fork_header
#endif
#if defined(MIVUBI_PRIVATE_ISOLATION_BUILD)
static_assert(MIVUBI_PRIVATE_ISOLATION_BUILD == 0x4D495655, "Unexpected fork marker");
static_assert(MIVUBI_PRIVATE_ISOLATION_ABI == MY_ABI_VERSION, "Private fork ABI mismatch");
constexpr bool kPrivateForkSource = true;
#else
constexpr bool kPrivateForkSource = false;
#endif
constexpr unsigned long kPinnedAbi = MY_ABI_VERSION;
constexpr wchar_t kPinnedVersion[] = TEXT(MY_VERSION_STRING);
constexpr wchar_t kLabDirectory[] = L"M-Isolation-Payload";

// Driver version query API from Sandboxie/core/dll/sbieapi.h;
// x64 only, never call another vendor's DLL via PATH searching.
using GetVersionExFn = LONG (*)(wchar_t*, unsigned long*);

struct ModuleGuard {
    HMODULE module = nullptr;
    ~ModuleGuard() { if (module != nullptr) FreeLibrary(module); }
    ModuleGuard() = default;
    ModuleGuard(const ModuleGuard&) = delete;
    ModuleGuard& operator=(const ModuleGuard&) = delete;
};

bool OrdinaryFile(const fs::path& file) {
    const auto attr = GetFileAttributesW(file.c_str());
    return attr != INVALID_FILE_ATTRIBUTES && (attr & FILE_ATTRIBUTE_DIRECTORY) == 0 &&
        (attr & FILE_ATTRIBUTE_REPARSE_POINT) == 0;
}
bool OrdinaryDirectory(const fs::path& directory) {
    const auto attr = GetFileAttributesW(directory.c_str());
    return attr != INVALID_FILE_ATTRIBUTES && (attr & FILE_ATTRIBUTE_DIRECTORY) != 0 &&
        (attr & FILE_ATTRIBUTE_REPARSE_POINT) == 0;
}
bool NoReparseAncestors(const fs::path& target) {
    fs::path parent;
    for (const auto& component : target) {
        parent /= component;
        // "C:" is a drive-relative path, not yet a rooted directory.
        if (!parent.has_root_directory()) continue;
        const auto attr = GetFileAttributesW(parent.c_str());
        if (attr == INVALID_FILE_ATTRIBUTES || (attr & FILE_ATTRIBUTE_REPARSE_POINT))
            return false;
    }
    return true;
}
bool SamePath(const std::wstring& a, const std::wstring& b) {
    return _wcsicmp(a.c_str(), b.c_str()) == 0;
}
std::wstring UnquoteExact(std::wstring s) {
    if (s.size() > 2 && s.front() == L'"' && s.back() == L'"')
        return s.substr(1, s.size() - 2);
    return s;
}
// Fail closed on command-line switches/extra parameters in ImagePath.
// This deliberately does not normalize arbitrary NT paths (\SystemRoot, \??)
// or environment expansion. A future fork must establish its own path proof.
bool ServiceImageMatches(const wchar_t* serviceName, const wchar_t* expected,
                         DWORD desiredType) {
    SC_HANDLE manager = OpenSCManagerW(nullptr, nullptr, SC_MANAGER_CONNECT);
    if (!manager) return false;
    SC_HANDLE service = OpenServiceW(manager, serviceName, SERVICE_QUERY_CONFIG);
    if (!service) { CloseServiceHandle(manager); return false; }
    DWORD length = 0;
    QueryServiceConfigW(service, nullptr, 0, &length);
    bool match = false;
    if (length > sizeof(QUERY_SERVICE_CONFIGW) && length < 32768) {
        // QueryServiceConfigW stores pointers in its output buffer.
        // uint64_t storage guarantees pointer-aligned backing memory on x64.
        std::vector<std::uint64_t> storage(
            (length + sizeof(std::uint64_t) - 1) / sizeof(std::uint64_t));
        auto* config = reinterpret_cast<QUERY_SERVICE_CONFIGW*>(storage.data());
        if (QueryServiceConfigW(service, config, length, &length) &&
            (config->dwServiceType & desiredType) == desiredType &&
            config->lpBinaryPathName != nullptr) {
            match = SamePath(UnquoteExact(config->lpBinaryPathName), expected);
        }
    }
    CloseServiceHandle(service);
    CloseServiceHandle(manager);
    return match;
}
bool AuthenticodeValid(const fs::path& file) {
    WINTRUST_FILE_INFO fileInfo{};
    fileInfo.cbStruct = sizeof(fileInfo);
    fileInfo.pcwszFilePath = file.c_str();
    WINTRUST_DATA data{};
    data.cbStruct = sizeof(data);
    data.dwUIChoice = WTD_UI_NONE;
    data.fdwRevocationChecks = WTD_REVOKE_NONE;
    data.dwUnionChoice = WTD_CHOICE_FILE;
    data.pFile = &fileInfo;
    data.dwStateAction = WTD_STATEACTION_VERIFY;
    data.dwProvFlags = WTD_CACHE_ONLY_URL_RETRIEVAL;
    GUID policy = WINTRUST_ACTION_GENERIC_VERIFY_V2;
    const LONG result = WinVerifyTrust(nullptr, &policy, &data);
    data.dwStateAction = WTD_STATEACTION_CLOSE;
    WinVerifyTrust(nullptr, &policy, &data);
    // Valid signature is necessary but NOT sufficient for kernel driver
    // signing/trust or permission to modify the account-bearing host.
    return result == ERROR_SUCCESS;
}
void Result(const char* state, bool layout, bool servicePaths, bool signatures,
            bool abiMatch = false) {
    std::cout << "{\"schema\":1,\"state\":\"" << state
        << "\",\"upstreamCommit\":\"abd4c240031cfb46666daa74ca47ed4e48b561e4\""
        << ",\"layoutObserved\":" << (layout ? "true" : "false")
        << ",\"driverServicePathsMatched\":" << (servicePaths ? "true" : "false")
        << ",\"validAuthenticodeObserved\":" << (signatures ? "true" : "false")
        << ",\"driverAbiMatchesPinned\":" << (abiMatch ? "true" : "false")
        << ",\"privateForkSourceMarked\":" << (kPrivateForkSource ? "true" : "false")
        << ",\"driverSigningPolicyAccepted\":false"
        << ",\"installerReady\":false,\"gameplayVerified\":false"
        << ",\"providerSelected\":false}\n";
}
}

int wmain(int argc, wchar_t* argv[]) {
    if (argc == 2 && wcscmp(argv[1], L"--selftest") == 0) {
        const bool pinned = kPinnedAbi == 0x57230UL &&
            wcscmp(kPinnedVersion, L"5.73.5") == 0;
        Result(pinned ? "PINNED_SOURCE_ABI_SELFTEST_NOT_RUNTIME_PROOF" :
                        "PINNED_SOURCE_ABI_INCONSISTENT",
               false, false, false);
        return pinned ? 0 : 11;
    }
    if (argc != 3 || (wcscmp(argv[1], L"--offline") != 0 &&
                       wcscmp(argv[1], L"--abi") != 0)) {
        Result("INVALID_ARGUMENTS", false, false, false);
        return 2;
    }
    const bool requestAbi = wcscmp(argv[1], L"--abi") == 0;
    try {
        const fs::path root = fs::absolute(fs::path(argv[2])).lexically_normal();
        if (!OrdinaryDirectory(root) || !NoReparseAncestors(root) ||
            !SamePath(root.filename().wstring(), kLabDirectory)) {
            Result("REJECTED_NON_DISPOSABLE_PATH", false, false, false);
            return 3;
        }
        // Never inspect the official Sandboxie installation via this utility.
        std::wstring foldedRoot = root.wstring();
        std::transform(foldedRoot.begin(), foldedRoot.end(),
            foldedRoot.begin(), [](wchar_t c) {
                return static_cast<wchar_t>(std::towlower(c));
            });
        if (foldedRoot.find(L"sandboxie") != std::wstring::npos ||
            foldedRoot.find(L"program files") != std::wstring::npos ||
            foldedRoot.find(L"\\windows\\") != std::wstring::npos) {
            Result("REJECTED_OFFICIAL_INSTALL_ROOT", false, false, false);
            return 3;
        }
        const std::array<fs::path, 10> required = {
            root / L"SbieDrv.sys", root / L"SbieSvc.exe", root / L"SbieDll.dll",
            root / L"Start.exe", root / L"SbieIni.exe", root / L"SbieMsg.dll",
            root / L"kmdutil.exe", root / L"32" / L"SbieSvc.exe",
            root / L"32" / L"SbieDll.dll", root / L"NOT-INSTALLABLE.txt",
        };
        bool layout = true;
        for (const auto& path : required)
            layout = OrdinaryFile(path) && layout;
        if (!layout) {
            Result("INCOMPLETE_ENGINE_PAYLOAD", false, false, false);
            return 4;
        }
        // Offline explicitly NEVER opens SCM or executes injected DLL code.
        if (!requestAbi) {
            Result("OFFLINE_CORE_FILES_PRESENT_NOT_VALIDATED", true, false, false);
            return 0;
        }
        // Lab runtime probe: the exact service+kernel image paths must point
        // into the same candidate payload before any upstream DLL is loaded.
        const bool ownership =
            ServiceImageMatches(L"SbieSvc", (root / L"SbieSvc.exe").c_str(),
                                SERVICE_WIN32_OWN_PROCESS) &&
            ServiceImageMatches(L"SbieDrv", (root / L"SbieDrv.sys").c_str(),
                                SERVICE_KERNEL_DRIVER);
        if (!ownership) {
            Result("REJECTED_SERVICE_IMAGE_PATHS", true, false, false);
            return 5;
        }
        const bool signatures = AuthenticodeValid(root / L"SbieDrv.sys") &&
            AuthenticodeValid(root / L"SbieSvc.exe") &&
            AuthenticodeValid(root / L"SbieDll.dll");
        if (!signatures) {
            Result("REJECTED_UNSIGNED_COMPONENT", true, true, false);
            return 6;
        }
        ModuleGuard module;
        module.module = LoadLibraryExW((root / L"SbieDll.dll").c_str(), nullptr,
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS);
        if (module.module == nullptr) {
            Result("FAILED_LOADING_PINNED_DLL", true, true, true);
            return 7;
        }
        // LoadLibrary can return an already-loaded basename collision on
        // some Windows loader paths; verify the actual mapped module path.
        std::array<wchar_t, 32768> loadedPath{};
        DWORD loadedLength = GetModuleFileNameW(module.module,
            loadedPath.data(), static_cast<DWORD>(loadedPath.size()));
        if (loadedLength == 0 || loadedLength >= loadedPath.size() ||
            !SamePath(loadedPath.data(), (root / L"SbieDll.dll").c_str())) {
            Result("REJECTED_LOADED_DLL_PATH_MISMATCH", true, true, true);
            return 8;
        }
        auto* getVersion = reinterpret_cast<GetVersionExFn>(
            GetProcAddress(module.module, "SbieApi_GetVersionEx"));
        if (getVersion == nullptr) {
            Result("MISSING_PINNED_DRIVER_ABI_EXPORT", true, true, true);
            return 8;
        }
        std::array<wchar_t, 16> version{};
        unsigned long abi = 0;
        const LONG status = getVersion(version.data(), &abi);
        version.back() = L'\0';
        const bool coherent = status >= 0 && abi == kPinnedAbi &&
            wcscmp(version.data(), kPinnedVersion) == 0;
        Result(coherent ? "PINNED_DRIVER_DLL_ABI_RESPONDS_UNACCEPTED" :
                          "DRIVER_DLL_ABI_MISMATCH", true, true, true, coherent);
        return coherent ? 0 : 9;
    } catch (...) {
        Result("INCONCLUSIVE_NATIVE_ERROR", false, false, false);
        return 10;
    }
}
