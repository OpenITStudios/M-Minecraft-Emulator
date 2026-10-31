// Optional MC02 in-process virtual keyboard/cursor polling implementation.
// It reuses the existing private bootstrap DLL and controller-owned inherited
// read-only mapping. No OS/global keyboard or mouse events are synthesized.
// This is NOT a working claim for Minecraft Raw Input/GameInput, and is
// entirely absent from normal builds until BOOTSTRAP_MC02_VIRTUAL_INPUT=ON.
#include "mc02-virtual-input.h"
#include <detours.h>
#include <atomic>
#include <cstdint>
#include <cstring>
#include <initializer_list>
#include <cwchar>

namespace {
constexpr std::uint32_t Magic=0x4c544932, Version=3;
struct alignas(8) SharedInput {
    std::uint32_t magic,version,bytes,slot,pid,session;
    std::uint64_t birth;
    std::atomic<std::uint64_t> packet;
    std::atomic<std::uint64_t> cursorRevision;
    std::atomic<std::int64_t> cursorX,cursorY;
    std::atomic<std::uint64_t> cursorEpoch;
    std::atomic<std::uint64_t> humanWindow,aiWindow;
};
static_assert(sizeof(SharedInput)==88 && alignof(SharedInput)==8);
static_assert(std::atomic<std::int64_t>::is_always_lock_free);
static_assert(std::atomic<std::uint64_t>::is_always_lock_free);
const SharedInput* view=nullptr;
std::atomic<bool> isolated{false}; // latched after first valid AI command
std::atomic<bool> cursorInitialized{false};
std::atomic<LONG> virtualX{0},virtualY{0};
// Only GetCursorPos consumes pending virtual pointer movement. Its
// non-blocking per-process guard prevents two game threads applying the
// same cumulative deltas twice. This is NOT a second input controller.
std::atomic_flag cursorReadBusy=ATOMIC_FLAG_INIT;
std::uint64_t seenRevision=0,seenEpoch=0;
std::int64_t seenX=0,seenY=0;
using KeyCall=SHORT (WINAPI*)(int);
using KeyboardStateCall=BOOL (WINAPI*)(PBYTE);
using CursorCall=BOOL (WINAPI*)(LPPOINT);
using SetCursorCall=BOOL (WINAPI*)(int,int);
using ClipCall=BOOL (WINAPI*)(const RECT*);
using FocusCall=HWND (WINAPI*)();
KeyCall originalAsync=nullptr,originalKey=nullptr;
KeyboardStateCall originalKeyboard=nullptr;
CursorCall originalCursor=nullptr;
SetCursorCall originalSetCursor=nullptr;
ClipCall originalClip=nullptr;
FocusCall originalForeground=nullptr,originalActive=nullptr,originalFocus=nullptr;

bool parseHandle(HANDLE& handle) {
    wchar_t text[32]{};
    const DWORD count=GetEnvironmentVariableW(
        L"MIVUBI_EDUCATION_MC02_INPUT_SECTION",text,32);
    if (count==0) return false;
    if (count>=32) return false;
    std::uint64_t raw=0;
    for (DWORD i=0;i<count;++i) {
        if (text[i]<L'0'||text[i]>L'9') return false;
        const auto digit=static_cast<std::uint64_t>(text[i]-L'0');
        if (raw>(UINT64_MAX-digit)/10) return false;
        raw=raw*10+digit;
    }
    if (!raw) return false;
    handle=reinterpret_cast<HANDLE>(static_cast<ULONG_PTR>(raw));
    return true;
}
std::uint64_t currentPacket() {
    if (!view) return 0;
    const std::uint64_t word=view->packet.load(std::memory_order_acquire);
    const DWORD now=GetTickCount();
    const DWORD until=static_cast<DWORD>(word>>32);
    const auto ahead=static_cast<LONG>(until-now);
    if (!word || ahead<=0 || ahead>500) return 0;
    // Human focus is not merely checked when the controller writes.
    // If MC01 loses the REAL Windows foreground before the child polls,
    // no AI keys/cursor deltas are accepted, and virtual buttons release.
    // Use the unhooked trampoline so fake foreground reads in this game
    // cannot bypass the physical human-host focus requirement.
    if (!originalForeground) return 0;
    const auto host=reinterpret_cast<HWND>(
        static_cast<ULONG_PTR>(view->humanWindow.load(std::memory_order_seq_cst)));
    if (!host || originalForeground()!=host) return 0;
    return word;
}
// A background game may skip its Win32 input polling loop after comparing
// GetForegroundWindow/GetActiveWindow/GetFocus to its own game HWND.
// Return the pinned MC02 OGLES window only inside THIS opted-in child,
// only for an unexpired AI packet while MC01 retains REAL foreground.
// Never SetForegroundWindow, SetFocus, attach a message queue or touch MC01.
HWND ownedGameWindow() {
    if (!view) return nullptr;
    const auto raw=view->aiWindow.load(std::memory_order_seq_cst);
    if (!raw) return nullptr;
    const HWND hwnd=reinterpret_cast<HWND>(static_cast<ULONG_PTR>(raw));
    DWORD pid=0;
    const DWORD thread=GetWindowThreadProcessId(hwnd,&pid);
    wchar_t name[16]{};
    if (!thread || pid!=GetCurrentProcessId() ||
        GetClassNameW(hwnd,name,16)!=5 ||
        std::wcscmp(name,L"OGLES")!=0 ||
        GetAncestor(hwnd,GA_ROOT)!=hwnd ||
        GetWindow(hwnd,GW_OWNER)!=nullptr ||
        !IsWindowVisible(hwnd) || IsIconic(hwnd)) return nullptr;
    return hwnd;
}
HWND WINAPI observedForeground() {
    const DWORD previous=GetLastError();
    const HWND real=originalForeground();
    const auto word=currentPacket();
    const HWND owned=word ? ownedGameWindow() : nullptr;
    if (owned) isolated.store(true,std::memory_order_release);
    SetLastError(previous);
    return owned ? owned : real;
}
HWND WINAPI observedActive() {
    const DWORD previous=GetLastError();
    const HWND real=originalActive();
    const auto word=currentPacket();
    const HWND owned=word ? ownedGameWindow() : nullptr;
    // GetActiveWindow is scoped to the caller's thread, unlike the
    // process-wide GetForegroundWindow query.
    const bool ownThread=owned &&
        GetWindowThreadProcessId(owned,nullptr)==GetCurrentThreadId();
    if (ownThread) isolated.store(true,std::memory_order_release);
    SetLastError(previous);
    return ownThread ? owned : real;
}
HWND WINAPI observedFocus() {
    const DWORD previous=GetLastError();
    const HWND real=originalFocus();
    const auto word=currentPacket();
    const HWND owned=word ? ownedGameWindow() : nullptr;
    // GetFocus is thread-local too. Never pretend that a worker thread
    // owns the Minecraft game window's message queue.
    const bool ownThread=owned &&
        GetWindowThreadProcessId(owned,nullptr)==GetCurrentThreadId();
    if (ownThread) isolated.store(true,std::memory_order_release);
    SetLastError(previous);
    return ownThread ? owned : real;
}
// MC02 must never poll MC01's physical key/cursor state while MC01 is
// foreground. Before the first AI command, allow local interaction with
// MC02 only when MC02 itself is the foreground window (login/menu).
// After the first command, virtual input remains latched until process
// close; expiration releases the virtual buttons, not the isolation.
bool virtualMode(std::uint64_t word) {
    if (word) isolated.store(true,std::memory_order_release);
    // Always query the REAL desktop foreground. Using the new game-local
    // GetForegroundWindow detour here would make background MC02 appear
    // physically focused and could leak MC01's real key states.
    const HWND foreground=originalForeground ? originalForeground() : nullptr;
    DWORD owner=0;
    const bool ownForeground=foreground!=nullptr &&
        GetWindowThreadProcessId(foreground,&owner)!=0 &&
        owner==GetCurrentProcessId();
    const bool useVirtual=isolated.load(std::memory_order_acquire)||
        !ownForeground;
    if (useVirtual && !cursorInitialized.exchange(true,std::memory_order_acq_rel)) {
        POINT point{};
        if (originalCursor && originalCursor(&point)) {
            virtualX.store(point.x,std::memory_order_release);
            virtualY.store(point.y,std::memory_order_release);
        }
    }
    return useVirtual;
}
unsigned maskFor(int code) {
    switch(code) {
        case 'W': return 1u;
        case 'S': return 2u;
        case 'A': return 4u;
        case 'D': return 8u;
        case VK_SPACE: return 16u;
        case VK_SHIFT: case VK_LSHIFT: case VK_RSHIFT: return 32u;
        case VK_LBUTTON: return 64u;
        case VK_RBUTTON: return 128u;
        default: return 0;
    }
}
bool virtualDown(int key,std::uint64_t word) {
    return (static_cast<unsigned>(word)&maskFor(key))!=0;
}
SHORT WINAPI observedAsync(int key) {
    const DWORD previous=GetLastError();
    const auto word=currentPacket();
    if (!virtualMode(word)) {
        SetLastError(previous); return originalAsync(key);
    }
    const SHORT result=virtualDown(key,word)?static_cast<SHORT>(0x8000):0;
    SetLastError(previous);
    return result;
}
SHORT WINAPI observedKey(int key) {
    const DWORD previous=GetLastError();
    const auto word=currentPacket();
    if (!virtualMode(word)) {
        SetLastError(previous); return originalKey(key);
    }
    const SHORT result=virtualDown(key,word)?static_cast<SHORT>(0x8000):0;
    SetLastError(previous);
    return result;
}
BOOL WINAPI observedKeyboard(PBYTE state) {
    const DWORD previous=GetLastError();
    const auto word=currentPacket();
    if (!virtualMode(word) || !state) {
        SetLastError(previous); return originalKeyboard(state);
    }
    std::memset(state,0,256);
    for (int key : {'W','S','A','D',VK_SPACE,VK_SHIFT,VK_LBUTTON,VK_RBUTTON}) {
        if (virtualDown(key,word)) state[key]=0x80;
    }
    if (virtualDown(VK_SHIFT,word)) state[VK_LSHIFT]=state[VK_RSHIFT]=0x80;
    SetLastError(previous);
    return TRUE;
}
void applyPendingCursor() {
    if (!view || cursorReadBusy.test_and_set(std::memory_order_acquire)) return;
    const auto before=view->cursorRevision.load(std::memory_order_seq_cst);
    if (before!=0 && !(before&1) && before!=seenRevision) {
        const auto x=view->cursorX.load(std::memory_order_seq_cst);
        const auto y=view->cursorY.load(std::memory_order_seq_cst);
        const auto epoch=view->cursorEpoch.load(std::memory_order_seq_cst);
        if (view->cursorRevision.load(std::memory_order_seq_cst)==before) {
            // Same epoch: take the difference in cumulative totals, even
            // when multiple fresh commands arrived before one game poll.
            // New epoch: the previous, expired cursor backlog is dropped.
            const auto dx=epoch==seenEpoch ? x-seenX : x;
            const auto dy=epoch==seenEpoch ? y-seenY : y;
            // Reject implausible arithmetic rather than wrap LONG and
            // teleport MC02's virtual pointer across the screen.
            if (dx>=-32767 && dx<=32767 && dy>=-32767 && dy<=32767) {
                virtualX.fetch_add(static_cast<LONG>(dx),std::memory_order_acq_rel);
                virtualY.fetch_add(static_cast<LONG>(dy),std::memory_order_acq_rel);
            }
            seenEpoch=epoch;seenRevision=before;seenX=x;seenY=y;
        }
    }
    cursorReadBusy.clear(std::memory_order_release);
}
BOOL WINAPI observedCursor(LPPOINT point) {
    const DWORD previous=GetLastError();
    const auto word=currentPacket();
    if (!virtualMode(word) || !point) {
        SetLastError(previous); return originalCursor(point);
    }
    // Do not consume an expired command. The next fresh native command
    // opens a new epoch, so stale look deltas cannot be replayed.
    if (word) applyPendingCursor();
    point->x=virtualX.load(std::memory_order_acquire);
    point->y=virtualY.load(std::memory_order_acquire);
    SetLastError(previous);
    return TRUE;
}
BOOL WINAPI observedSetCursor(int x,int y) {
    const DWORD previous=GetLastError();
    const auto word=currentPacket();
    if (!virtualMode(word)) {
        SetLastError(previous); return originalSetCursor(x,y);
    }
    // MC02's cursor recentering never physically moves MC01's pointer.
    virtualX.store(x,std::memory_order_release);
    virtualY.store(y,std::memory_order_release);
    SetLastError(previous);
    return TRUE;
}
BOOL WINAPI observedClip(const RECT* rect) {
    const DWORD previous=GetLastError();
    const auto word=currentPacket();
    if (!virtualMode(word)) {
        SetLastError(previous); return originalClip(rect);
    }
    // A background AI window must not clip the physical human desktop.
    SetLastError(previous);
    return TRUE;
}
} // namespace
namespace mc02_input {
bool attach(const education_bootstrap::Config& config) noexcept {
    // This optional experiment must NEVER reject or hook MC01/MC03–MC05.
    if (config.slot!=2) return true;
    wchar_t raw[2]{};
    // No inherited section => optional feature stays inert even in a
    // development binary. Environment is supplied only by the owner.
    if (GetEnvironmentVariableW(L"MIVUBI_EDUCATION_MC02_INPUT_SECTION",raw,2)==0)
        return true;
    HANDLE handle=nullptr;
    if (!parseHandle(handle)) return false;
    auto* mapped=MapViewOfFile(handle,FILE_MAP_READ,0,0,sizeof(SharedInput));
    if (!mapped) return false;
    const auto* state=static_cast<const SharedInput*>(mapped);
    if (state->magic!=Magic||state->version!=Version||state->bytes!=sizeof(SharedInput)||
        state->slot!=2||state->pid!=GetCurrentProcessId()||
        state->session!=config.session||
        state->birth!=(static_cast<std::uint64_t>(config.created.dwHighDateTime)<<32 |
                        config.created.dwLowDateTime)) {
        UnmapViewOfFile(mapped);return false;
    }
    HMODULE user=GetModuleHandleW(L"user32.dll");
    if (!user) {UnmapViewOfFile(mapped);return false;}
    originalAsync=reinterpret_cast<KeyCall>(GetProcAddress(user,"GetAsyncKeyState"));
    originalKey=reinterpret_cast<KeyCall>(GetProcAddress(user,"GetKeyState"));
    originalKeyboard=reinterpret_cast<KeyboardStateCall>(GetProcAddress(user,"GetKeyboardState"));
    originalCursor=reinterpret_cast<CursorCall>(GetProcAddress(user,"GetCursorPos"));
    originalSetCursor=reinterpret_cast<SetCursorCall>(GetProcAddress(user,"SetCursorPos"));
    originalClip=reinterpret_cast<ClipCall>(GetProcAddress(user,"ClipCursor"));
    originalForeground=reinterpret_cast<FocusCall>(
        GetProcAddress(user,"GetForegroundWindow"));
    originalActive=reinterpret_cast<FocusCall>(
        GetProcAddress(user,"GetActiveWindow"));
    originalFocus=reinterpret_cast<FocusCall>(
        GetProcAddress(user,"GetFocus"));
    if (!originalAsync||!originalKey||!originalKeyboard||!originalCursor||
        !originalSetCursor||!originalClip||!originalForeground||
        !originalActive||!originalFocus) {
        UnmapViewOfFile(mapped);return false;
    }
    view=state;
    // Called inside the existing bootstrap-compat Detours transaction.
    return DetourAttach(reinterpret_cast<PVOID*>(&originalAsync),
                reinterpret_cast<PVOID>(observedAsync))==NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalKey),
                reinterpret_cast<PVOID>(observedKey))==NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalKeyboard),
                reinterpret_cast<PVOID>(observedKeyboard))==NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalCursor),
                reinterpret_cast<PVOID>(observedCursor))==NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalSetCursor),
                reinterpret_cast<PVOID>(observedSetCursor))==NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalClip),
                reinterpret_cast<PVOID>(observedClip))==NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalForeground),
                reinterpret_cast<PVOID>(observedForeground))==NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalActive),
                reinterpret_cast<PVOID>(observedActive))==NO_ERROR
        && DetourAttach(reinterpret_cast<PVOID*>(&originalFocus),
                reinterpret_cast<PVOID>(observedFocus))==NO_ERROR;
}
} // namespace mc02_input
