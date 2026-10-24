#pragma once
#include <windows.h>
#include <winternl.h>
#if !defined(_M_X64)
#error This self-test supports native AMD64 only
#endif

// Fixed self-test transport, not a production slot capability or shared-state ABI.
namespace probe {
constexpr DWORD Magic = 0x42535431;
constexpr DWORD Version = 5;
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
    DWORD fileRound;
    HANDLE fileLogicalRoot, fileOwnRoot;
    wchar_t logical[128], own[128], peer[128];
    HANDLE ready, go, done, finish;
};
static_assert(sizeof(Event) == 8);
static_assert(sizeof(Receipt) == 1000);

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
            && value->pid == GetCurrentProcessId() && value->mode <= MutexCase
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
} // namespace probe
