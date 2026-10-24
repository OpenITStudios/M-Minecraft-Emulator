#pragma once
#include <windows.h>
#include <winternl.h>
#if !defined(_M_X64)
#error This self-test supports native AMD64 only
#endif

// Fixed self-test transport, not a production slot capability or shared-state ABI.
namespace probe {
constexpr DWORD Magic = 0x42535431;
constexpr DWORD Version = 6;
constexpr LONG Capacity = 16;
constexpr wchar_t SectionVariable[] = L"MIVUBI_BOOTSTRAP_TEST_SECTION";
using CompareHandles = BOOL (WINAPI*)(HANDLE, HANDLE);
using NativeCreate = NTSTATUS (NTAPI*)(PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES, BOOLEAN);
using NativeOpen = NTSTATUS (NTAPI*)(PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES);
using NativeFileCreate = NTSTATUS (NTAPI*)(PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES,
    PIO_STATUS_BLOCK, PLARGE_INTEGER, ULONG, ULONG, ULONG, ULONG, PVOID, ULONG);
using NativeFileOpen = NTSTATUS (NTAPI*)(PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES,
    PIO_STATUS_BLOCK, ULONG, ULONG);
using NativeDirectory = NTSTATUS (NTAPI*)(PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES);
enum Mode : DWORD { Baseline, Normal, FailAttach, Timeout, MutexCase };
enum Stage : LONG { Attach = 1, Armed, Rejected, Suppressed, Dependency, Tls, Main };
struct Event { LONG stage; LONG armed; };
struct Receipt {
    DWORD magic, version, pid;
    FILETIME created;
    DWORD mode;
    volatile LONG count, armed;
    Event events[Capacity];
    DWORD slot;
    volatile LONG mutexChecks, nativeChecks, fileChecks, registryChecks;
    DWORD fileRound, registryBaseline;
    volatile LONG earlyDenied, earlyAllowed;
    HKEY cachedHostKey;
    wchar_t registryUserPath[512];
    HANDLE fileLogicalRoot, fileOwnRoot;
    wchar_t logical[128], own[128], peer[128];
    HANDLE ready, go, done, finish;
};
static_assert(sizeof(Event) == 8);
static_assert(sizeof(Receipt) == 2048);

class View {
public:
    Receipt* value = nullptr;
    View() = default;
    View(const View&) = delete;
    View& operator=(const View&) = delete;
    ~View() { if (value) UnmapViewOfFile(value); }
    bool open() {
        wchar_t buffer[32];
        const DWORD length = GetEnvironmentVariableW(SectionVariable, buffer, 32);
        if (length == 0 || length >= 32) return false;
        ULONG_PTR raw = 0;
        for (DWORD i = 0; i < length; ++i) {
            if (buffer[i] < L'0' || buffer[i] > L'9') return false;
            const ULONG_PTR digit = static_cast<ULONG_PTR>(buffer[i] - L'0');
            if (raw > (~static_cast<ULONG_PTR>(0) - digit) / 10) return false;
            raw = raw * 10 + digit;
        }
        if (!raw) return false;
        value = static_cast<Receipt*>(MapViewOfFile(
            reinterpret_cast<HANDLE>(raw), FILE_MAP_READ | FILE_MAP_WRITE, 0, 0, sizeof(Receipt)));
        if (!value) return false;
        FILETIME created{}, exited{}, kernel{}, user{};
        const bool valid = GetProcessTimes(GetCurrentProcess(), &created, &exited, &kernel, &user)
            && value->magic == Magic && value->version == Version
            && value->pid == GetCurrentProcessId() && value->mode <= MutexCase && value->registryBaseline <= 1
            && value->created.dwLowDateTime == created.dwLowDateTime
            && value->created.dwHighDateTime == created.dwHighDateTime;
        if (!valid) {
            UnmapViewOfFile(value);
            value = nullptr;
        }
        return valid;
    }
};
inline void record(Receipt* receipt, Stage stage) {
    const LONG index = InterlockedIncrement(&receipt->count) - 1;
    if (index >= 0 && index < Capacity) {
        receipt->events[index] = {stage, InterlockedCompareExchange(&receipt->armed, 0, 0)};
    }
}
inline void canary(Stage stage) {
    View view;
    if (view.open()) record(view.value, stage);
}

inline void registryCanary(Stage stage) {
    View view;
    if (!view.open() || view.value->mode != MutexCase) return;
    auto& receipt = *view.value;
    wchar_t path[768]{};
    unsigned count = 0;
    const wchar_t* parts[]{receipt.registryUserPath, L"\\Software\\", receipt.logical + 6};
    for (const wchar_t* part : parts) {
        for (unsigned i = 0; part[i]; ++i) {
            if (count >= 767) return;
            path[count++] = part[i];
        }
    }
    const auto bytes = static_cast<USHORT>(count * sizeof(wchar_t));
    UNICODE_STRING name{bytes, bytes, path};
    OBJECT_ATTRIBUTES attributes{sizeof(OBJECT_ATTRIBUTES), nullptr, &name,
        OBJ_CASE_INSENSITIVE, nullptr, nullptr};
    using Open = NTSTATUS (NTAPI*)(PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES);
    using Query = NTSTATUS (NTAPI*)(HANDLE, PUNICODE_STRING, ULONG, PVOID, ULONG, PULONG);
    using Set = NTSTATUS (NTAPI*)(HANDLE, PUNICODE_STRING, ULONG, ULONG, PVOID, ULONG);
    const HMODULE ntdll = GetModuleHandleW(L"ntdll.dll");
    const auto open = reinterpret_cast<Open>(GetProcAddress(ntdll, "NtOpenKey"));
    const auto query = reinterpret_cast<Query>(GetProcAddress(ntdll, "NtQueryValueKey"));
    const auto set = reinterpret_cast<Set>(GetProcAddress(ntdll, "NtSetValueKey"));
    if (!open || !query || !set) return;
    HANDLE key = nullptr;
    constexpr DWORD sentinel = 0x13579bd;
    SetLastError(sentinel);
    const NTSTATUS opened = open(&key, KEY_READ, &attributes);
    const bool openError = GetLastError() == sentinel;
    if (key) CloseHandle(key);
    wchar_t marker[] = L"Marker";
    UNICODE_STRING valueName{12, 12, marker};
    struct Value { ULONG title, type, bytes, data; } value{};
    ULONG returned = 0;
    SetLastError(sentinel);
    const NTSTATUS queried = query(receipt.cachedHostKey, &valueName, 2, &value, sizeof(value), &returned);
    const bool queryError = GetLastError() == sentinel;
    constexpr DWORD host = 0x484f5354;
    DWORD write = receipt.armed == 1 && !receipt.registryBaseline ? 0xdeadbeef : host;
    SetLastError(sentinel);
    const NTSTATUS written = set(receipt.cachedHostKey, &valueName, 0, REG_DWORD, &write, sizeof(write));
    const bool setError = GetLastError() == sentinel;
    const LONG bit = stage == Dependency ? 1 : stage == Tls ? 2 : stage == Main ? 4 : 0;
    if (!openError || !queryError || !setError) return;
    constexpr auto denied = static_cast<NTSTATUS>(0xc0000022UL);
    if (opened == denied && queried == denied && written == denied && !key
        && returned == 0 && value.title == 0 && value.type == 0 && value.bytes == 0 && value.data == 0)
        InterlockedOr(&receipt.earlyDenied, bit);
    if (opened == 0 && queried == 0 && written == 0 && value.type == REG_DWORD
        && value.bytes == sizeof(DWORD) && value.data == host && write == host)
        InterlockedOr(&receipt.earlyAllowed, bit);
}
} // namespace probe
