#pragma once
#include <windows.h>
#include <cstddef>
#include <cstring>

// Private native pilot transport, independent of the self-test receipt.
// StorageAndRegistryOnly deliberately carries no singleton/broker readiness.
namespace education_bootstrap {
constexpr DWORD Magic = 0x45444231;
constexpr DWORD Version = 2;
constexpr DWORD MaxSlots = 5;
constexpr bool validSlot(DWORD slot) { return slot >= 1 && slot <= MaxSlots; }
constexpr DWORD StorageAndRegistryOnly = 1;
constexpr DWORD StorageRegistrySelectedMutex = 2;
constexpr wchar_t SectionVariable[] = L"MIVUBI_EDUCATION_BOOTSTRAP_SECTION";
struct Config {
    DWORD magic, version, bytes, slot, pid, session, scope, singletonCallerRva;
    FILETIME created;
    HANDLE logicalRoot, privateRoot;
    HKEY registryRoot;
    FILE_ID_INFO logicalId, privateId;
    wchar_t userKeyPath[512];
};
static_assert(sizeof(Config) == 1136);
static_assert(offsetof(Config, logicalId) == 64);
static_assert(offsetof(Config, userKeyPath) == 112);

inline bool validScope(const Config& config) {
    return (config.scope == StorageAndRegistryOnly && config.singletonCallerRva == 0)
        || (config.scope == StorageRegistrySelectedMutex && config.singletonCallerRva != 0);
}

inline bool present() {
    SetLastError(ERROR_SUCCESS);
    const DWORD size = GetEnvironmentVariableW(SectionVariable, nullptr, 0);
    return size != 0 || GetLastError() != ERROR_ENVVAR_NOT_FOUND;
}

inline bool read(Config& config) {
    wchar_t buffer[32]{};
    const DWORD length = GetEnvironmentVariableW(SectionVariable, buffer, 32);
    if (!length || length >= 32) return false;
    ULONG_PTR raw = 0;
    for (DWORD i = 0; i < length; ++i) {
        if (buffer[i] < L'0' || buffer[i] > L'9') return false;
        const ULONG_PTR digit = static_cast<ULONG_PTR>(buffer[i] - L'0');
        if (raw > (~static_cast<ULONG_PTR>(0) - digit) / 10) return false;
        raw = raw * 10 + digit;
    }
    if (!raw) return false;
    const HANDLE section = reinterpret_cast<HANDLE>(raw);
    if (!SetHandleInformation(section, HANDLE_FLAG_INHERIT, 0)) return false;
    // A target receives a read-only capability, never the parent's writable handle.
    void* writable = MapViewOfFile(section, FILE_MAP_WRITE, 0, 0, sizeof(Config));
    if (writable) { UnmapViewOfFile(writable); return false; }
    const void* view = MapViewOfFile(section, FILE_MAP_READ, 0, 0, sizeof(Config));
    if (!view) return false;
    bool copied = false;
    __try { memcpy(&config, view, sizeof(config)); copied = true; }
    __except (EXCEPTION_EXECUTE_HANDLER) {}
    UnmapViewOfFile(view);
    if (!copied) return false;
    FILETIME birth{}, exited{}, kernel{}, user{};
    DWORD session = 0;
    return config.magic == Magic && config.version == Version && config.bytes == sizeof(Config)
        && validSlot(config.slot) && validScope(config)
        && config.pid == GetCurrentProcessId()
        && config.userKeyPath[511] == 0
        && GetProcessTimes(GetCurrentProcess(), &birth, &exited, &kernel, &user)
        && birth.dwLowDateTime == config.created.dwLowDateTime
        && birth.dwHighDateTime == config.created.dwHighDateTime
        && ProcessIdToSessionId(GetCurrentProcessId(), &session)
        && session != 0 && config.session == session;
}
} // namespace education_bootstrap
