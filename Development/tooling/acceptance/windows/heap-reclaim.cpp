#include <windows.h>
#include <psapi.h>
#include <tlhelp32.h>
#include <cstdio>
#include <cwchar>
#include <cstdint>

// One explicit allocator experiment; no hooks, timer, game-data reads or provider state.
struct HeapRequest {
    DWORD bytes, magic, pid;
    FILETIME birth;
    DWORD completed, error, mainHeapKind, heapKindKnown;
    SIZE_T beforeCommit, afterCommit;
    LONGLONG elapsed, frequency;
};
constexpr DWORD RequestMagic = 0x48525031;
bool sameBirth(FILETIME a, FILETIME b) {
    return a.dwLowDateTime == b.dwLowDateTime && a.dwHighDateTime == b.dwHighDateTime;
}
bool birthOf(HANDLE process, FILETIME& birth) {
    FILETIME exit{}, kernel{}, user{};
    return GetProcessTimes(process, &birth, &exit, &kernel, &user) != FALSE;
}

#ifdef HEAP_RECLAIM_DLL
extern "C" __declspec(dllexport) DWORD WINAPI MivubiReclaimHeap(void* parameter) {
    auto* request = static_cast<HeapRequest*>(parameter);
    FILETIME birth{};
    if (!request || request->bytes != sizeof(HeapRequest) || request->magic != RequestMagic
        || request->pid != GetCurrentProcessId() || request->completed
        || !birthOf(GetCurrentProcess(), birth) || !sameBirth(request->birth, birth))
        return ERROR_INVALID_PARAMETER;
    PROCESS_MEMORY_COUNTERS_EX before{}, after{};
    before.cb = sizeof(before); after.cb = sizeof(after);
    LARGE_INTEGER start{}, end{}, frequency{};
    if (!GetProcessMemoryInfo(GetCurrentProcess(), reinterpret_cast<PROCESS_MEMORY_COUNTERS*>(&before), sizeof(before))
        || !QueryPerformanceFrequency(&frequency) || !QueryPerformanceCounter(&start))
        return GetLastError();
    ULONG kind = 0;
    request->heapKindKnown = HeapQueryInformation(GetProcessHeap(), HeapCompatibilityInformation,
        &kind, sizeof(kind), nullptr) != FALSE;
    request->mainHeapKind = kind;
    HEAP_OPTIMIZE_RESOURCES_INFORMATION options{};
    options.Version = HEAP_OPTIMIZE_RESOURCES_CURRENT_VERSION;
    const BOOL result = HeapSetInformation(nullptr, HeapOptimizeResources, &options, sizeof(options));
    request->error = result ? ERROR_SUCCESS : GetLastError();
    QueryPerformanceCounter(&end);
    if (!GetProcessMemoryInfo(GetCurrentProcess(), reinterpret_cast<PROCESS_MEMORY_COUNTERS*>(&after), sizeof(after)))
        return GetLastError();
    request->beforeCommit = before.PrivateUsage; request->afterCommit = after.PrivateUsage;
    request->elapsed = end.QuadPart - start.QuadPart; request->frequency = frequency.QuadPart;
    request->completed = 1;
    return request->error;
}
BOOL WINAPI DllMain(HINSTANCE, DWORD, LPVOID) { return TRUE; }
#else
struct Handle {
    HANDLE value = nullptr;
    ~Handle() { if (value && value != INVALID_HANDLE_VALUE) CloseHandle(value); }
};
bool decimal(const wchar_t* text, unsigned long long& value) {
    if (!text || !*text) return false;
    value = 0;
    for (; *text; ++text) {
        if (*text < L'0' || *text > L'9' || value > (UINT64_MAX - (*text - L'0')) / 10) return false;
        value = value * 10 + (*text - L'0');
    }
    return true;
}
HMODULE remoteModule(DWORD pid, const wchar_t* path, DWORD* size = nullptr) {
    Handle snapshot{CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid)};
    if (snapshot.value == INVALID_HANDLE_VALUE) return nullptr;
    MODULEENTRY32W entry{}; entry.dwSize = sizeof(entry);
    for (BOOL found = Module32FirstW(snapshot.value, &entry); found; found = Module32NextW(snapshot.value, &entry)) {
        if (_wcsicmp(path, entry.szExePath) == 0) {
            if (size) *size = entry.modBaseSize;
            return entry.hModule;
        }
    }
    return nullptr;
}
LPTHREAD_START_ROUTINE remoteSystemFunction(DWORD pid, const char* name) {
    FARPROC function = GetProcAddress(GetModuleHandleW(L"kernel32.dll"), name);
    HMODULE owner = nullptr;
    if (!function || !GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS
        | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT, reinterpret_cast<LPCWSTR>(function), &owner)) return nullptr;
    wchar_t path[MAX_PATH]{};
    if (!GetModuleFileNameW(owner, path, MAX_PATH)) return nullptr;
    DWORD bytes = 0;
    const HMODULE remote = remoteModule(pid, path, &bytes);
    const auto offset = reinterpret_cast<ULONG_PTR>(function) - reinterpret_cast<ULONG_PTR>(owner);
    if (!remote || offset >= bytes) return nullptr;
    return reinterpret_cast<LPTHREAD_START_ROUTINE>(reinterpret_cast<ULONG_PTR>(remote) + offset);
}
bool run(HANDLE process, LPTHREAD_START_ROUTINE function, void* argument, bool& uncertain) {
    Handle thread{CreateRemoteThread(process, nullptr, 0, function, argument, 0, nullptr)};
    if (!thread.value) return false;
    if (WaitForSingleObject(thread.value, 10000) != WAIT_OBJECT_0) {
        uncertain = true; // Preserve live thread arguments/module; never terminate a game/thread.
        return false;
    }
    return true;
}
void print(const HeapRequest& request) {
    std::printf("{\"completed\":%lu,\"error\":%lu,\"mainHeapKindKnown\":%lu,\"mainHeapKind\":%lu,"
        "\"beforePrivateCommitBytes\":%llu,\"afterPrivateCommitBytes\":%llu,\"durationMs\":%.3f}\n",
        request.completed, request.error, request.heapKindKnown, request.mainHeapKind,
        static_cast<unsigned long long>(request.beforeCommit), static_cast<unsigned long long>(request.afterCommit),
        request.frequency > 0 ? 1000.0 * request.elapsed / request.frequency : 0.0);
}
int wmain(int argc, wchar_t** argv) {
    if (argc == 3 && !wcscmp(argv[1], L"--self-test")) {
        HMODULE library = LoadLibraryW(argv[2]);
        if (!library) return 20;
        const auto action = reinterpret_cast<DWORD (WINAPI*)(void*)>(GetProcAddress(library, "MivubiReclaimHeap"));
        HeapRequest request{};
        request.bytes = sizeof(request); request.magic = RequestMagic; request.pid = GetCurrentProcessId();
        if (!action || !birthOf(GetCurrentProcess(), request.birth)) { FreeLibrary(library); return 21; }
        request.magic = 0;
        if (action(&request) != ERROR_INVALID_PARAMETER || request.completed) { FreeLibrary(library); return 22; }
        request.magic = RequestMagic;
        auto* live = static_cast<BYTE*>(HeapAlloc(GetProcessHeap(), 0, 65536));
        if (!live) { FreeLibrary(library); return 23; }
        for (unsigned i = 0; i < 65536; ++i) live[i] = static_cast<BYTE>(i);
        const DWORD result = action(&request);
        bool intact = true;
        for (unsigned i = 0; i < 65536; ++i) if (live[i] != static_cast<BYTE>(i)) intact = false;
        const bool replayDenied = action(&request) == ERROR_INVALID_PARAMETER;
        HeapFree(GetProcessHeap(), 0, live); FreeLibrary(library);
        print(request);
        return result == ERROR_SUCCESS && request.completed && intact && replayDenied ? 0 : 24;
    }
    if (argc != 5 || wcscmp(argv[1], L"--process")) return 2;
    unsigned long long parsedPid = 0, parsedBirth = 0;
    if (!decimal(argv[2], parsedPid) || !parsedPid || parsedPid > MAXDWORD
        || !decimal(argv[3], parsedBirth) || !parsedBirth) return 2;
    const DWORD pid = static_cast<DWORD>(parsedPid);
    const FILETIME expected{static_cast<DWORD>(parsedBirth), static_cast<DWORD>(parsedBirth >> 32)};
    wchar_t library[MAX_PATH]{};
    const DWORD length = GetFullPathNameW(argv[4], MAX_PATH, library, nullptr);
    if (!length || length >= MAX_PATH || library[1] != L':' || library[2] != L'\\') return 2;
    Handle lockedFile{CreateFileW(library, GENERIC_READ, FILE_SHARE_READ, nullptr, OPEN_EXISTING,
        FILE_FLAG_OPEN_REPARSE_POINT, nullptr)};
    BY_HANDLE_FILE_INFORMATION file{};
    if (lockedFile.value == INVALID_HANDLE_VALUE || !GetFileInformationByHandle(lockedFile.value, &file)
        || (file.dwFileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT))) return 3;
    HMODULE image = LoadLibraryExW(library, nullptr, DONT_RESOLVE_DLL_REFERENCES);
    if (!image) return 3;
    const auto entry = GetProcAddress(image, "MivubiReclaimHeap");
    const auto offset = entry ? reinterpret_cast<ULONG_PTR>(entry) - reinterpret_cast<ULONG_PTR>(image) : 0;
    FreeLibrary(image);
    if (!offset) return 3;
    Handle process{OpenProcess(PROCESS_CREATE_THREAD | PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION
        | PROCESS_VM_WRITE | PROCESS_VM_READ | SYNCHRONIZE, FALSE, pid)};
    FILETIME birth{}; BOOL wow = TRUE;
    wchar_t target[MAX_PATH]{}; DWORD targetLength = MAX_PATH;
    if (!process.value || !birthOf(process.value, birth) || !sameBirth(expected, birth)
        || !IsWow64Process(process.value, &wow) || wow
        || !QueryFullProcessImageNameW(process.value, 0, target, &targetLength)
        || _wcsicmp(target, L"C:\\Program Files\\Microsoft Studios\\Minecraft Education\\Minecraft.Windows.exe")
        || WaitForSingleObject(process.value, 0) != WAIT_TIMEOUT || remoteModule(pid, library)) return 4;
    const auto load = remoteSystemFunction(pid, "LoadLibraryW"), unload = remoteSystemFunction(pid, "FreeLibrary");
    if (!load || !unload) return 4;
    void* remotePath = VirtualAllocEx(process.value, nullptr, (length + 1) * sizeof(wchar_t), MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    if (!remotePath) return 5;
    bool uncertain = false;
    if (!WriteProcessMemory(process.value, remotePath, library, (length + 1) * sizeof(wchar_t), nullptr)
        || !run(process.value, load, remotePath, uncertain)) {
        if (!uncertain) VirtualFreeEx(process.value, remotePath, 0, MEM_RELEASE);
        return uncertain ? 10 : 5;
    }
    VirtualFreeEx(process.value, remotePath, 0, MEM_RELEASE);
    DWORD moduleBytes = 0;
    const HMODULE module = remoteModule(pid, library, &moduleBytes);
    if (!module) return 6;
    if (offset >= moduleBytes) { run(process.value, unload, module, uncertain); return 6; }
    HeapRequest request{};
    request.bytes = sizeof(request); request.magic = RequestMagic; request.pid = pid; request.birth = birth;
    void* remoteRequest = VirtualAllocEx(process.value, nullptr, sizeof(request), MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    bool completed = false;
    if (remoteRequest && WriteProcessMemory(process.value, remoteRequest, &request, sizeof(request), nullptr)) {
        const auto action = reinterpret_cast<LPTHREAD_START_ROUTINE>(reinterpret_cast<ULONG_PTR>(module) + offset);
        completed = run(process.value, action, remoteRequest, uncertain)
            && ReadProcessMemory(process.value, remoteRequest, &request, sizeof(request), nullptr) != FALSE;
    }
    if (uncertain) { std::fputs("Action status unknown; live arguments preserved.\n", stderr); return 10; }
    if (remoteRequest) VirtualFreeEx(process.value, remoteRequest, 0, MEM_RELEASE);
    if (!run(process.value, unload, module, uncertain) || remoteModule(pid, library)) return 11;
    print(request);
    return completed && request.completed && !request.error ? 0 : 7;
}
#endif
