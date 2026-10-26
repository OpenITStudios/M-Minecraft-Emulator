#pragma once
#include "education-config.h"
#include <winternl.h>
#include <intrin.h>

// Private, local-only diagnostic bytes. Never provider readiness or authenticated evidence.
namespace startup_observation {
constexpr DWORD Magic = 0x4f425331, Version = 3, Capacity = 64;
constexpr wchar_t Variable[] = L"MIVUBI_EDUCATION_OBSERVATION_SECTION";
enum Kind : DWORD { CreateMutant = 1, OpenMutant, CreateProcess,
    Win32MutexW, Win32MutexExW, Win32MutexA, Win32MutexExA };
enum Namespace : DWORD { NamespaceUnknown, SessionBaseNamedObjects };
enum CallerModule : DWORD { CallerUnknown, MainImage, KernelBaseImage, Kernel32Image, SecureEngineImage };
enum Flags : DWORD { TruncatedName = 1, UnreadableArguments = 2, UnreadableResult = 4,
    ChildIdentityUnknown = 8, InitialOwner = 16, SingletonTranslated = 32 };
enum Loss : LONG { CapacityReached = 1, ReentrantLogger = 2, ClockOrLateReturn = 4 };
struct Record {
    volatile LONG committed;
    DWORD kind, thread, flags;
    ULONG_PTR caller;
    LONGLONG start, end;
    LONG status;
    DWORD lastError, argument, attributes;
    ULONG_PTR root, returned;
    DWORD nameBytes, childPid, childTid, namespaceKind;
    FILETIME childCreated;
    wchar_t name[192];
    DWORD callerModule, callerRva, callerImageBytes, reserved;
};
// Numeric failure shapes only: no path, file contents, account identifiers or call stacks.
enum PathStage : DWORD { PathStageUnknown = 0, AbsoluteRootConflict = 1, RelativeParent = 2, ComposedPath = 3 };
enum PathReason : DWORD { PathReasonUnknown = 0, TotalLength = 1, EmptyComponent = 2, TrailingDotOrSpace = 3,
    DepthLimit = 4, UnsupportedCharacter = 5, LeadingSpace = 6, InvalidUtf16 = 7, ComponentLength = 8, RootConflict = 9 };
struct PathRejection { DWORD stage = 0, reason = 0; };
struct StorageSample {
    volatile LONG claimed, committed;
    DWORD operation, access, options, disposition, pathLength;
    LONG status;
};
struct StorageDiagnostics {
    volatile LONG counts[5]; // success, unsupported path, access/options, parent open, backing call
    // Optional first unsupported-path detail: schema (0=unavailable, 1=defined), stage, reason.
    // Published by first[0].committed; old v3 journals keep zeros. Size/offsets stay unchanged.
    DWORD reserved[3];
    StorageSample first[4];
};
static_assert(sizeof(StorageDiagnostics) == 160);
static_assert(offsetof(StorageDiagnostics, reserved) == 20);
static_assert(offsetof(StorageDiagnostics, first) == 32);
struct Journal {
    DWORD magic, version, bytes, pid;
    FILETIME created;
    DWORD session, capacity;
    LONGLONG frequency, started;
    volatile LONG claimed, losses;
    DWORD durationMs;
    volatile LONG clockReady;
    Record records[Capacity];
    StorageDiagnostics storage;
};
static_assert(sizeof(Record) == 496);
static_assert(offsetof(Journal, records) == 64);
static_assert(sizeof(Journal) == 31968);
inline Journal* active = nullptr;
inline thread_local bool logging = false;
// Unlike the 30-second startup trace, these fixed counters cover the game lifetime.
inline void storageResult(unsigned reason, DWORD operation, ACCESS_MASK access, ULONG options,
                          ULONG disposition, unsigned length, NTSTATUS status,
                          PathRejection path = {}) {
    if (!active || reason > 4) return;
    auto& value = active->storage;
    InterlockedIncrement(&value.counts[reason]);
    if (!reason) return;
    auto& sample = value.first[reason - 1];
    if (InterlockedCompareExchange(&sample.claimed, 1, 0) != 0) return;
    sample.operation = operation; sample.access = access; sample.options = options;
    sample.disposition = disposition; sample.pathLength = length; sample.status = status;
    if (reason == 1) {
        value.reserved[0] = 1; value.reserved[1] = path.stage; value.reserved[2] = path.reason;
    }
    InterlockedExchange(&sample.committed, 1);
}

class View {
public:
    Journal* value = nullptr;
    ~View() { if (value) UnmapViewOfFile(value); }
    bool map(HANDLE section) {
        value = static_cast<Journal*>(MapViewOfFile(section, FILE_MAP_READ | FILE_MAP_WRITE,
            0, 0, sizeof(Journal)));
        return value != nullptr;
    }
    bool open(const education_bootstrap::Config& config) {
        wchar_t text[32]{};
        DWORD length = GetEnvironmentVariableW(Variable, text, 32);
        if (!length || length >= 32) return false;
        ULONG_PTR raw = 0;
        for (DWORD i = 0; i < length; ++i) {
            if (text[i] < L'0' || text[i] > L'9') return false;
            const ULONG_PTR digit = static_cast<ULONG_PTR>(text[i] - L'0');
            if (raw > (~static_cast<ULONG_PTR>(0) - digit) / 10) return false;
            raw = raw * 10 + digit;
        }
        const HANDLE section = reinterpret_cast<HANDLE>(raw);
        if (!raw || !SetHandleInformation(section, HANDLE_FLAG_INHERIT, 0) || !map(section)) return false;
        LARGE_INTEGER frequency{};
        return value->magic == Magic && value->version == Version && value->bytes == sizeof(Journal)
            && value->pid == config.pid && value->session == config.session && value->capacity == Capacity
            && value->created.dwHighDateTime == config.created.dwHighDateTime
            && value->created.dwLowDateTime == config.created.dwLowDateTime
            && value->durationMs == 30000 && QueryPerformanceFrequency(&frequency)
            && frequency.QuadPart > 0 && frequency.QuadPart <= MAXLONGLONG / 31
            && frequency.QuadPart == value->frequency;
    }
};
inline bool initialize(HANDLE section, const education_bootstrap::Config& config) {
    View view;
    if (!view.map(section) || view.value->magic != 0) return false;
    LARGE_INTEGER frequency{};
    if (!QueryPerformanceFrequency(&frequency) || frequency.QuadPart <= 0
        || frequency.QuadPart > MAXLONGLONG / 31) return false;
    ZeroMemory(view.value, sizeof(Journal));
    view.value->version = Version;
    view.value->bytes = sizeof(Journal);
    view.value->pid = config.pid;
    view.value->created = config.created;
    view.value->session = config.session;
    view.value->capacity = Capacity;
    view.value->frequency = frequency.QuadPart;
    view.value->durationMs = 30000;
    view.value->magic = Magic;
    return true;
}
inline bool attach(const education_bootstrap::Config& config) {
    SetLastError(ERROR_SUCCESS);
    if (!GetEnvironmentVariableW(Variable, nullptr, 0) && GetLastError() == ERROR_ENVVAR_NOT_FOUND)
        return true;
    View view;
    if (!view.open(config) || view.value->clockReady != 0 || view.value->claimed != 0) return false;
    LARGE_INTEGER now{};
    if (!QueryPerformanceCounter(&now)) return false;
    view.value->started = now.QuadPart;
    InterlockedExchange(&view.value->clockReady, 1);
    active = view.value;
    view.value = nullptr; // Process-lifetime mapping; no callbacks or active unload.
    return true;
}
inline bool within(const Journal& journal, LONGLONG now) {
    return journal.frequency > 0 && journal.frequency <= MAXLONGLONG / 31 && now >= journal.started
        && static_cast<ULONGLONG>(now) - static_cast<ULONGLONG>(journal.started)
            <= static_cast<ULONGLONG>(journal.frequency) * 30;
}
inline bool begin(Record& record, Kind kind, void* caller) {
    if (!active) return false;
    if (logging) { InterlockedOr(&active->losses, ReentrantLogger); return false; }
    logging = true;
    LARGE_INTEGER now{};
    const bool clock = QueryPerformanceCounter(&now) != FALSE;
    if (!clock) InterlockedOr(&active->losses, ClockOrLateReturn);
    const bool enabled = clock && within(*active, now.QuadPart);
    if (enabled) {
        record.kind = kind;
        record.thread = GetCurrentThreadId();
        record.caller = reinterpret_cast<ULONG_PTR>(caller);
        record.start = now.QuadPart;
    }
    logging = false;
    return enabled;
}
inline void objectArguments(Record& record, ACCESS_MASK access, const OBJECT_ATTRIBUTES* attributes) {
    record.argument = access;
    __try {
        if (!attributes || attributes->Length < sizeof(OBJECT_ATTRIBUTES)) {
            record.flags |= UnreadableArguments;
            return;
        }
        record.attributes = attributes->Attributes;
        record.root = reinterpret_cast<ULONG_PTR>(attributes->RootDirectory);
        if (!attributes->ObjectName) return;
        const auto& name = *attributes->ObjectName;
        record.nameBytes = name.Length;
        if (name.Length % sizeof(wchar_t) || (name.Length && !name.Buffer)) {
            record.flags |= UnreadableArguments;
            return;
        }
        const size_t count = name.Length < sizeof(record.name) ? name.Length : sizeof(record.name);
        if (name.Length > count) record.flags |= TruncatedName;
        if (count) memcpy(record.name, name.Buffer, count);
    } __except (EXCEPTION_EXECUTE_HANDLER) { record.flags |= UnreadableArguments; }
}
inline void objectResult(Record& record, NTSTATUS status, const HANDLE* handle) {
    record.status = status;
    if (status < 0) return;
    __try {
        if (handle) record.returned = reinterpret_cast<ULONG_PTR>(*handle);
        else record.flags |= UnreadableResult;
    } __except (EXCEPTION_EXECUTE_HANDLER) { record.flags |= UnreadableResult; }
}
inline void processResult(Record& record, BOOL result, const PROCESS_INFORMATION* info) {
    record.status = result;
    if (!result) return;
    PROCESS_INFORMATION copy{};
    __try {
        if (!info) { record.flags |= UnreadableResult; return; }
        copy = *info;
    } __except (EXCEPTION_EXECUTE_HANDLER) { record.flags |= UnreadableResult; return; }
    record.childPid = copy.dwProcessId;
    record.childTid = copy.dwThreadId;
    record.returned = reinterpret_cast<ULONG_PTR>(copy.hProcess);
    FILETIME exit{}, kernel{}, user{};
    if (!copy.dwProcessId || !copy.dwThreadId || GetProcessId(copy.hProcess) != copy.dwProcessId
        || GetProcessIdOfThread(copy.hThread) != copy.dwProcessId
        || GetThreadId(copy.hThread) != copy.dwThreadId
        || !GetProcessTimes(copy.hProcess, &record.childCreated, &exit, &kernel, &user))
        record.flags |= ChildIdentityUnknown;
}
inline void finish(Record& record, DWORD error) {
    logging = true;
    record.lastError = error;
    LARGE_INTEGER now{};
    if (!QueryPerformanceCounter(&now) || !within(*active, now.QuadPart)) {
        InterlockedOr(&active->losses, ClockOrLateReturn);
        logging = false;
        return;
    }
    record.end = now.QuadPart;
    LONG index = InterlockedCompareExchange(&active->claimed, 0, 0);
    while (index >= 0 && index < static_cast<LONG>(Capacity)) {
        const LONG actual = InterlockedCompareExchange(&active->claimed, index + 1, index);
        if (actual == index) break;
        index = actual;
    }
    if (index < 0 || index >= static_cast<LONG>(Capacity)) InterlockedOr(&active->losses, CapacityReached);
    else {
        // Each slot is written once; readers must wait for committed or original process exit.
        memcpy(&active->records[index], &record, sizeof(record));
        InterlockedExchange(&active->records[index].committed, 1);
    }
    logging = false;
}
} // namespace startup_observation
