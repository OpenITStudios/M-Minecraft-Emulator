// Optional per-process MC01-MC05 AI keyboard/cursor polling candidate.
// It reuses the existing private bootstrap DLL and controller-owned inherited
// read-only mapping. No OS/global keyboard or mouse events are synthesized.
// This is NOT a working claim for Minecraft Raw Input/GameInput, and is
// entirely absent from normal builds until BOOTSTRAP_MC02_VIRTUAL_INPUT=ON.
#include "mc02-virtual-input.h"
#include <detours.h>
#include <atomic>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <initializer_list>
#include <cwchar>
#if defined(BOOTSTRAP_MC02_RAW_BUFFER_POLL)
#include <array>
#endif

namespace {
constexpr std::uint32_t Magic=0x4c544932, Version=4;
struct alignas(8) SharedInput {
    std::uint32_t magic,version,bytes,slot,pid,session;
    std::uint64_t birth;
    std::atomic<std::uint64_t> packet;
    std::atomic<std::uint64_t> cursorRevision;
    std::atomic<std::int64_t> cursorX,cursorY;
    std::atomic<std::uint64_t> cursorEpoch;
    std::atomic<std::uint64_t> humanWindow,aiWindow;
    std::atomic<std::uint64_t> pressCounts;
};
static_assert(sizeof(SharedInput)==96 && alignof(SharedInput)==8);
static_assert(std::atomic<std::int64_t>::is_always_lock_free);
static_assert(std::atomic<std::uint64_t>::is_always_lock_free);
// SharedInput is ABI v4 and shared with the Rust native controller.
// Size/alignment alone cannot detect a reordered field of equal width.
// Pin every field offset in the C++ game reader and Rust writer.
static_assert(sizeof(std::atomic<std::uint64_t>)==8 &&
              sizeof(std::atomic<std::int64_t>)==8);
static_assert(
    offsetof(SharedInput,magic)==0 &&
    offsetof(SharedInput,version)==4 &&
    offsetof(SharedInput,bytes)==8 &&
    offsetof(SharedInput,slot)==12 &&
    offsetof(SharedInput,pid)==16 &&
    offsetof(SharedInput,session)==20 &&
    offsetof(SharedInput,birth)==24 &&
    offsetof(SharedInput,packet)==32 &&
    offsetof(SharedInput,cursorRevision)==40 &&
    offsetof(SharedInput,cursorX)==48 &&
    offsetof(SharedInput,cursorY)==56 &&
    offsetof(SharedInput,cursorEpoch)==64 &&
    offsetof(SharedInput,humanWindow)==72 &&
    offsetof(SharedInput,aiWindow)==80 &&
    offsetof(SharedInput,pressCounts)==88 ,
    "SharedInput v4 offsets must match the Rust native writer");
const SharedInput* view=nullptr;
std::atomic<bool> isolated{false}; // latched after first valid AI command
std::atomic<bool> cursorInitialized{false};
std::atomic<LONG> virtualX{0},virtualY{0};
// Strictly one-shot hook-return evidence. A positive virtual return means
// only that THIS game process polled an opted-in API: it does not prove
// Minecraft applied the input to movement, camera or attack.
std::atomic<bool> sawVirtualKeyboard{false},sawVirtualButtons{false},
    sawVirtualLook{false};
void noteVirtualRead(std::atomic<bool>& seen,const wchar_t* tag) {
    const DWORD previous=GetLastError();
    if (!seen.exchange(true,std::memory_order_relaxed)) OutputDebugStringW(tag);
    SetLastError(previous);
}
void noteVirtualKeys(unsigned mask) {
    if (mask & 0x3fu) noteVirtualRead(sawVirtualKeyboard,
        L"[LT_MC02_INPUT][VIRTUAL_KEY_RETURNED]\n");
    if (mask & 0xc0u) noteVirtualRead(sawVirtualButtons,
        L"[LT_MC02_INPUT][VIRTUAL_BUTTON_RETURNED]\n");
}
// Select exactly ONE relative-camera delivery path at compile time:
// the Win32 cursor polling route OR the explicit buffered Raw Input
// experiment. Never apply the same look deltas to both.
#if !defined(BOOTSTRAP_MC02_RAW_BUFFER_POLL)
std::atomic_flag cursorReadBusy=ATOMIC_FLAG_INIT;
std::uint64_t seenRevision=0,seenEpoch=0;
std::int64_t seenX=0,seenY=0;
#endif
// Per-process read cursor for 8 key/mouse rising-edge counters. The
// original Rust Channel now rejects the 256th rise of ANY 8-bit field
// in one live cursor epoch BEFORE publishing a new packet. Byte-value
// equality here and in RawPlan therefore cannot alias a wrapped 0 after
// 256 new presses. Several presses before one poll STILL coalesce; this
// does not certify how many game clicks or attacks occurred.
// Separate nonblocking guard from cursor movement: async key and camera
// reads may run on different Minecraft threads without sharing a lock.
std::atomic_flag asyncReadBusy=ATOMIC_FLAG_INIT;
std::uint64_t seenPressEpoch=0,seenPressCounts=0;
using KeyCall=SHORT (WINAPI*)(int);
using KeyboardStateCall=BOOL (WINAPI*)(PBYTE);
using CursorCall=BOOL (WINAPI*)(LPPOINT);
using SetCursorCall=BOOL (WINAPI*)(int,int);
using ClipCall=BOOL (WINAPI*)(const RECT*);
using FocusCall=HWND (WINAPI*)();
#if defined(BOOTSTRAP_MC02_RAW_BUFFER_POLL)
using RawBufferCall=UINT (WINAPI*)(PRAWINPUT,PUINT,UINT);
RawBufferCall originalRawBuffer=nullptr;
std::atomic_flag rawBufferBusy=ATOMIC_FLAG_INIT;
std::uint64_t rawSeenEpoch=0,rawSeenCounts=0;
std::int64_t rawSeenX=0,rawSeenY=0;
unsigned rawHeldMask=0;
#endif
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
// The original Rust v4 packet holds a 32-bit GetTickCount deadline.
// Unsigned modular subtraction respects the 500-ms limit even when the
// uptime counter wraps. This is read-time validity, NOT game consumption.
constexpr bool packetLiveAt(std::uint64_t word,std::uint32_t now) noexcept {
    if (!word) return false;
    const auto until=static_cast<std::uint32_t>(word>>32);
    const auto remaining=static_cast<std::uint32_t>(until-now);
    return remaining>0 && remaining<=500;
}
// Compile-time negative fixtures; authored but NOT_EXECUTED.
constexpr auto DeadlineSample=(std::uint64_t{501}<<32)|1u;
static_assert(packetLiveAt(DeadlineSample,1u));     // 500 ms
static_assert(packetLiveAt(DeadlineSample,500u));   // 1 ms
static_assert(!packetLiveAt(DeadlineSample,501u));  // expired
static_assert(!packetLiveAt(DeadlineSample,502u));  // past expiry
static_assert(!packetLiveAt(DeadlineSample,0u));    // >500 ms
constexpr auto RolloverSample=(std::uint64_t{3}<<32)|1u;
static_assert(packetLiveAt(RolloverSample,0xfffffffEu));
static_assert(!packetLiveAt(RolloverSample,4u));
static_assert(!packetLiveAt(0,0u));

// The controller verifies its exact original child and root HWND when
// publishing a packet. The game window can nevertheless be destroyed,
// hidden or replaced before an opted-in Win32 key/cursor poll. A current
// time and revision alone must never authorize the old packet.
// Reuse the existing same-process OGLES-root validator, without caching
// a HWND across separate polls or admitting a different window.
HWND ownedGameWindow(bool permitHiddenForRelease=false);
constexpr bool activePacketAdmitted(bool originalRoot,bool sameRevision,
                                   bool stillLive) noexcept {
    return originalRoot && sameRevision && stillLive;
}
static_assert(activePacketAdmitted(true,true,true));
static_assert(!activePacketAdmitted(false,true,true));
static_assert(!activePacketAdmitted(true,false,true));
static_assert(!activePacketAdmitted(true,true,false));

std::uint64_t currentPacket() {
    if (!view) return 0;
    // Rust publishes packet, host HWND and input state inside one ABI-v4
    // odd/even cursorRevision transaction. Sampling packet separately
    // from humanWindow could expose a partly published heartbeat or
    // briefly accept an old pressed key during ReleaseAll.
    // All virtual Win32 key/focus/cursor hooks share this fail-closed gate.
    const auto revision=view->cursorRevision.load(std::memory_order_seq_cst);
    if (!revision || (revision&1)) return 0;
    const auto word=view->packet.load(std::memory_order_seq_cst);
    const auto hostRaw=view->humanWindow.load(std::memory_order_seq_cst);
    const DWORD now=GetTickCount();
    if (!packetLiveAt(word,now) || !originalForeground) return 0;
    // Human focus is not merely checked when the controller writes.
    // Use the real Win32 trampoline: a spoofed game-local focus read
    // must never authorize background MC02 against physical MC01 input.
    // A nonzero host HWND denotes hybrid and demands the real MC01
    // foreground. Zero is reserved for an explicitly opted-in five-AI
    // channel; no global foreground or physical input is synthesized.
    if (hostRaw) {
        const auto host=reinterpret_cast<HWND>(
            static_cast<ULONG_PTR>(hostRaw));
        if (originalForeground()!=host) return 0;
    }
    // KeyState/AsyncKeyState/KeyboardState and cursor hooks previously
    // skipped the exact AI root check used by focus and buffered Raw Input.
    // A live packet from an old HWND must not drive a newly replaced or
    // hidden game window, even though PID/birth and the input lease match.
    // Perform Win32 identity calls ONLY for an active packet, then sample
    // the same publication revision again to reject concurrent updates.
    // This performs no OS input and cannot raise/activate any window.
    const bool originalRoot=ownedGameWindow()!=nullptr;
    const bool sameRevision=
        view->cursorRevision.load(std::memory_order_seq_cst)==revision;
    if (!activePacketAdmitted(originalRoot,sameRevision,
                             packetLiveAt(word,GetTickCount()))) return 0;
    return word;
}
// A background game may skip its Win32 input polling loop after comparing
// GetForegroundWindow/GetActiveWindow/GetFocus to its own game HWND.
// Resolve only the pinned MC02 OGLES root in THIS opted-in child.
// Active input and focus-spoof hooks must still require a visible,
// non-minimized game window. The buffered Raw Input reader MAY also
// resolve a hidden/minimized window solely to complete pending key-up
// and mouse-button UP records when currentPacket()==0. This never
// activates that window, changes focus or routes physical MC01 input.
HWND ownedGameWindow(bool permitHiddenForRelease) {
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
        (!permitHiddenForRelease &&
         (!IsWindowVisible(hwnd) || IsIconic(hwnd)))) return nullptr;
    return hwnd;
}
HWND WINAPI observedForeground() {
    const DWORD previous=GetLastError();
    const HWND real=originalForeground();
    const auto word=currentPacket();
    const HWND owned=word ? ownedGameWindow() : nullptr;
    const bool liveOwned=owned && packetLiveAt(word,GetTickCount());
    if (liveOwned) isolated.store(true,std::memory_order_release);
    SetLastError(previous);
    return liveOwned ? owned : real;
}
HWND WINAPI observedActive() {
    const DWORD previous=GetLastError();
    const HWND real=originalActive();
    const auto word=currentPacket();
    const HWND owned=word ? ownedGameWindow() : nullptr;
    // GetActiveWindow is scoped to the caller's thread, unlike the
    // process-wide GetForegroundWindow query.
    const bool ownThread=owned &&
        GetWindowThreadProcessId(owned,nullptr)==GetCurrentThreadId() &&
        packetLiveAt(word,GetTickCount());
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
        GetWindowThreadProcessId(owned,nullptr)==GetCurrentThreadId() &&
        packetLiveAt(word,GetTickCount());
    if (ownThread) isolated.store(true,std::memory_order_release);
    SetLastError(previous);
    return ownThread ? owned : real;
}
// An AI client must never poll another human's physical input while MC01 is
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
// GetAsyncKeyState returns the high bit for 'currently held' and the
// low bit for 'pressed since the previous call'. A last-packet key mask
// alone cannot report a short click followed by a release between polls.
// The original Win32 low bit is unreliable across callers, but this
// optional MC02-only read cursor is independent from MC01 hardware.
// Read under the existing writer's revision and reject torn snapshots.
bool virtualPressEdge(unsigned keyMask,std::uint64_t word) {
    if (!view || !packetLiveAt(word,GetTickCount()) || !keyMask ||
        asyncReadBusy.test_and_set(std::memory_order_acquire)) return false;
    bool pressed=false;
    const auto revision=view->cursorRevision.load(std::memory_order_seq_cst);
    if (revision!=0 && !(revision&1)) {
        const auto counts=view->pressCounts.load(std::memory_order_seq_cst);
        const auto epoch=view->cursorEpoch.load(std::memory_order_seq_cst);
        const auto packet=view->packet.load(std::memory_order_seq_cst);
        if (packet==word &&
            view->cursorRevision.load(std::memory_order_seq_cst)==revision &&
            packetLiveAt(word,GetTickCount())) {
            if (epoch!=seenPressEpoch) {
                seenPressEpoch=epoch;
                seenPressCounts=0; // discard expired previous-epoch clicks
            }
            unsigned shift=0;
            for (unsigned bits=keyMask;bits>1;bits>>=1) shift+=8;
            const std::uint64_t mask=0xffull<<shift;
            if ((seenPressCounts&mask)!=(counts&mask)) {
                seenPressCounts=(seenPressCounts&~mask)|(counts&mask);
                pressed=true;
            }
        }
    }
    asyncReadBusy.clear(std::memory_order_release);
    return pressed;
}
SHORT WINAPI observedAsync(int key) {
    const DWORD previous=GetLastError();
    const auto word=currentPacket();
    if (!virtualMode(word)) {
        SetLastError(previous); return originalAsync(key);
    }
    // virtualMode can query Windows after the initial packet sample.
    const auto live=packetLiveAt(word,GetTickCount())?word:0;
    const unsigned keyMask=maskFor(key);
    const bool held=virtualDown(key,live);
    const bool newlyPressed=virtualPressEdge(keyMask,live);
    const SHORT result=static_cast<SHORT>(
        (held ? 0x8000u : 0u) | (newlyPressed ? 1u : 0u));
    if (live && result) noteVirtualKeys(keyMask);
    SetLastError(previous);
    return result;
}
SHORT WINAPI observedKey(int key) {
    const DWORD previous=GetLastError();
    const auto word=currentPacket();
    if (!virtualMode(word)) {
        SetLastError(previous); return originalKey(key);
    }
    const auto live=packetLiveAt(word,GetTickCount())?word:0;
    const SHORT result=virtualDown(key,live)?static_cast<SHORT>(0x8000):0;
    if (live && result) noteVirtualKeys(maskFor(key));
    SetLastError(previous);
    return result;
}
BOOL WINAPI observedKeyboard(PBYTE state) {
    const DWORD previous=GetLastError();
    const auto word=currentPacket();
    if (!virtualMode(word) || !state) {
        SetLastError(previous); return originalKeyboard(state);
    }
    const auto live=packetLiveAt(word,GetTickCount())?word:0;
    std::memset(state,0,256);
    for (int key : {'W','S','A','D',VK_SPACE,VK_SHIFT,VK_LBUTTON,VK_RBUTTON}) {
        if (virtualDown(key,live)) state[key]=0x80;
    }
    if (virtualDown(VK_SHIFT,live)) state[VK_LSHIFT]=state[VK_RSHIFT]=0x80;
    if (live) noteVirtualKeys(static_cast<unsigned>(live)&0xffu);
    SetLastError(previous);
    return TRUE;
}
#if !defined(BOOTSTRAP_MC02_RAW_BUFFER_POLL)
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
                if (dx || dy) noteVirtualRead(sawVirtualLook,
                    L"[LT_MC02_INPUT][VIRTUAL_LOOK_RETURNED]\n");
            }
            seenEpoch=epoch;seenRevision=before;seenX=x;seenY=y;
        }
    }
    cursorReadBusy.clear(std::memory_order_release);
}
#endif
BOOL WINAPI observedCursor(LPPOINT point) {
    const DWORD previous=GetLastError();
    const auto word=currentPacket();
    if (!virtualMode(word) || !point) {
        SetLastError(previous); return originalCursor(point);
    }
    // Do not consume an expired command. The next fresh native command
    // opens a new epoch, so stale look deltas cannot be replayed.
    // In the separate Raw-buffer mode, camera deltas are published ONLY
    // through GetRawInputBuffer; GetCursorPos retains menu cursor state.
#if !defined(BOOTSTRAP_MC02_RAW_BUFFER_POLL)
    if (packetLiveAt(word,GetTickCount())) applyPendingCursor();
#endif
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
#if defined(BOOTSTRAP_MC02_RAW_BUFFER_POLL)
// This is ONLY a consumer-side adapter for games that *already poll*
// GetRawInputBuffer. It does not post WM_INPUT, create HRAWINPUT handles,
// supply a kernel HID, invent device registration or intercept GetRawInputData.
// Synthetic RAWINPUTHEADER.hDevice is intentionally null: games that
// demand a physical device handle are NOT supported by this candidate.
// A separate OFF-by-default CMake option is required in addition to
// BOOTSTRAP_MC02_VIRTUAL_INPUT; no extra DLL/controller/process is created.
// At worst six keyboard actions and two mouse buttons each need
// release/down/release records, plus one relative mouse movement: 25.
using RawBatch=std::array<RAWINPUT,28>;
void rawHeader(RAWINPUT& event,DWORD kind) {
    std::memset(&event,0,sizeof(event));
    event.header.dwType=kind;
    event.header.dwSize=static_cast<DWORD>(sizeof(RAWINPUT));
    event.header.hDevice=nullptr;
    event.header.wParam=RIM_INPUTSINK;
}
void rawKey(RawBatch& batch,UINT& count,USHORT vk,USHORT scan,bool down) {
    if (count>=static_cast<UINT>(batch.size())) return;
    auto& e=batch[count++];
    rawHeader(e,RIM_TYPEKEYBOARD);
    e.data.keyboard.VKey=vk;
    e.data.keyboard.MakeCode=scan;
    e.data.keyboard.Flags=down?RI_KEY_MAKE:RI_KEY_BREAK;
    e.data.keyboard.Message=down?WM_KEYDOWN:WM_KEYUP;
}
void rawMouse(RawBatch& batch,UINT& count,USHORT buttons,LONG dx=0,LONG dy=0) {
    if (count>=static_cast<UINT>(batch.size())) return;
    auto& e=batch[count++];
    rawHeader(e,RIM_TYPEMOUSE);
    e.data.mouse.usFlags=MOUSE_MOVE_RELATIVE;
    e.data.mouse.usButtonFlags=buttons;
    e.data.mouse.lLastX=dx;
    e.data.mouse.lLastY=dy;
}
// If a once-active packet becomes zero AFTER the physical queue drain,
// only re-plan RELEASE records that were previously copied as DOWN.
// A new active packet, another thread or no held state is not permission
// to replay stale presses, cursor movement, or a different game window.
constexpr bool rawCanReplanRelease(std::uint64_t previous,
    std::uint64_t current,bool originalGameThread,unsigned held) noexcept {
    return previous!=0 && current==0 && originalGameThread && held!=0;
}
static_assert(rawCanReplanRelease(1,0,true,1));
static_assert(!rawCanReplanRelease(1,2,true,1));
static_assert(!rawCanReplanRelease(0,0,true,1));
static_assert(!rawCanReplanRelease(1,0,false,1));
static_assert(!rawCanReplanRelease(1,0,true,0));

struct RawPlan {
    RawBatch events{};
    UINT count=0;
    unsigned held=0;
    std::uint64_t epoch=0,counts=0;
    std::int64_t cursorX=0,cursorY=0;
    std::uint64_t revision=0; // inherited ABI-v4 writer transaction
    bool coherent=false;
};
RawPlan nextRawPlan(std::uint64_t word) {
    RawPlan plan{};
    plan.held=word?static_cast<unsigned>(word&255):0;
    plan.epoch=rawSeenEpoch;
    plan.counts=rawSeenCounts;
    plan.cursorX=rawSeenX;plan.cursorY=rawSeenY;
    LONG pendingLookX=0,pendingLookY=0;
    // A zero/expired packet can still need key-up/mouse-up output.
    // Do not synthesize those releases while a native ReleaseAll or
    // replacement input write has an ODD shared-memory revision.
    // The old neutral path bypassed this guard entirely.
    if (!view) return plan;
    const auto revision=view->cursorRevision.load(std::memory_order_seq_cst);
    if (!revision || (revision&1)) return plan;
    plan.revision=revision;
    if (word) {
        const auto counts=view->pressCounts.load(std::memory_order_seq_cst);
        const auto epoch=view->cursorEpoch.load(std::memory_order_seq_cst);
        const auto x=view->cursorX.load(std::memory_order_seq_cst);
        const auto y=view->cursorY.load(std::memory_order_seq_cst);
        if (view->cursorRevision.load(std::memory_order_seq_cst)!=revision ||
            view->packet.load(std::memory_order_seq_cst)!=word) return plan;
        plan.epoch=epoch;plan.counts=counts;
        const auto baseX=epoch==rawSeenEpoch?rawSeenX:0;
        const auto baseY=epoch==rawSeenEpoch?rawSeenY:0;
        const auto dx=x-baseX,dy=y-baseY;
        // One call cannot move a virtual pointer by an unbounded amount.
        // Keep the unconsumed remainder for the next buffered poll.
        const auto moveX=dx>32767?32767:dx< -32767?-32767:dx;
        const auto moveY=dy>32767?32767:dy< -32767?-32767:dy;
        plan.cursorX=baseX+moveX;
        plan.cursorY=baseY+moveY;
        pendingLookX=static_cast<LONG>(moveX);
        pendingLookY=static_cast<LONG>(moveY);
    }
    // All eight virtual buttons share ONE ordering rule. First release
    // every previously held key/button that is now up or must be
    // re-triggered, then synthesize new press edges. This matters when
    // GetRawInputBuffer can return only one record: a new W key-down
    // must not keep an older attack mouse-up pending in MC02.
    // A full short press/release between game polls still emits a
    // down/up pair; multiple cycles coalesce to one (never fabricated).
    struct Button {
        unsigned bit,index;
        USHORT vk,scan,mouseDown,mouseUp;
    };
    constexpr Button buttons[]={
        {1u,0u,'W',0x11,0,0},{2u,1u,'S',0x1f,0,0},
        {4u,2u,'A',0x1e,0,0},{8u,3u,'D',0x20,0,0},
        {16u,4u,VK_SPACE,0x39,0,0},{32u,5u,VK_SHIFT,0x2a,0,0},
        {64u,6u,0,0,RI_MOUSE_LEFT_BUTTON_DOWN,RI_MOUSE_LEFT_BUTTON_UP},
        {128u,7u,0,0,RI_MOUSE_RIGHT_BUTTON_DOWN,RI_MOUSE_RIGHT_BUTTON_UP},
    };
    const auto oldCounts=plan.epoch==rawSeenEpoch?rawSeenCounts:0;
    const auto freshlyPressed=[&](const Button& button) {
        const auto shift=button.index*8u;
        return static_cast<std::uint8_t>(plan.counts>>shift)!=
            static_cast<std::uint8_t>(oldCounts>>shift);
    };
    const auto emit=[&](const Button& button,bool down) {
        if (button.bit&0xc0u)
            rawMouse(plan.events,plan.count,
                down?button.mouseDown:button.mouseUp);
        else rawKey(plan.events,plan.count,button.vk,button.scan,down);
    };
    // Phase 1: drain ALL outstanding releases (key-up / mouse-up)
    // before any new press, independent of button index.
    for (const auto& button:buttons) {
        const bool prior=(rawHeldMask&button.bit)!=0;
        const bool now=(plan.held&button.bit)!=0;
        if (prior && (freshlyPressed(button) || !now))
            emit(button,false);
    }
    // Phase 2: new down transitions. A short, already released click
    // produces down/up in this phase. If only a prefix fits, the
    // reader's rawHeldMask + press-count cursor reconstructs its suffix.
    for (const auto& button:buttons) {
        const bool prior=(rawHeldMask&button.bit)!=0;
        const bool now=(plan.held&button.bit)!=0;
        if (freshlyPressed(button) || (!prior && now)) {
            emit(button,true);
            if (!now) emit(button,false);
        }
    }
    // A caller can provide space for only ONE RAWINPUT record. When
    // fresh relative look data arrives on every poll, placing the mouse
    // movement first would postpone pending key-up/mouse-up indefinitely.
    // Emit all button transitions first (preserving each button's
    // release/press/release order); append one-shot camera motion last.
    // rawSeenX/Y are advanced only when this final record is copied.
    if (pendingLookX || pendingLookY)
        rawMouse(plan.events,plan.count,0,pendingLookX,pendingLookY);
    // The synthetic batch must not be derived from a torn neutral
    // snapshot either. A subsequent poll retries any undelivered
    // releases from rawHeldMask; no read cursor advances on failure.
    if (view->cursorRevision.load(std::memory_order_seq_cst)!=plan.revision)
        return plan;
    plan.coherent=true;
    return plan;
}
// A caller may have room for just ONE RAWINPUT structure. Consume only
// that prefix; never move rawHeldMask/cursors past records not copied.
constexpr UINT rawCopiedRecordCount(UINT planned,UINT capacityBytes) noexcept {
    const UINT available=capacityBytes/static_cast<UINT>(sizeof(RAWINPUT));
    return planned<available?planned:available;
}
static_assert(rawCopiedRecordCount(8,sizeof(RAWINPUT))==1);
static_assert(rawCopiedRecordCount(8,sizeof(RAWINPUT)-1)==0);
static_assert(rawCopiedRecordCount(0,sizeof(RAWINPUT))==0);
static_assert(rawCopiedRecordCount(8,2*sizeof(RAWINPUT))==2);

// Updating the game's per-key and cursor read cursors is justified only
// by at least one completed output record. In particular, a zero-record
// poll with a newly published epoch must not consume its future presses.
constexpr bool rawMayAdvanceReadCursor(UINT copied) noexcept {
    return copied!=0;
}
static_assert(!rawMayAdvanceReadCursor(0));
static_assert(rawMayAdvanceReadCursor(1));
static_assert(rawMayAdvanceReadCursor(2));

// The final Raw Input admission is sampled only AFTER any potentially
// delayed Win32 window-owner lookup. An expired/changed active word cannot
// pass merely because the original snapshot revision was stable BEFORE
// that lookup. Zero-word pending UP records still require the same gate.
constexpr bool rawFinalCopyAllowed(bool coherent,bool sameWindow,
    std::uint64_t plannedWord,std::uint64_t sampledWord,
    std::uint64_t plannedRevision,std::uint64_t sampledRevision) noexcept {
    return coherent && sameWindow && plannedRevision!=0 &&
        !(plannedRevision&1) && plannedRevision==sampledRevision &&
        plannedWord==sampledWord;
}
// Source-only compile-time fixtures: active/zero successes, changed
// word after window sampling, changed/odd revision, invalid window or
// incoherent plan. AUTHORED_NOT_EXECUTED.
static_assert(rawFinalCopyAllowed(true,true,1,1,2,2));
static_assert(rawFinalCopyAllowed(true,true,0,0,2,2));
static_assert(!rawFinalCopyAllowed(true,true,1,0,2,2));
static_assert(!rawFinalCopyAllowed(true,true,0,1,2,2));
static_assert(!rawFinalCopyAllowed(true,true,1,1,2,4));
static_assert(!rawFinalCopyAllowed(true,true,1,1,3,3));
static_assert(!rawFinalCopyAllowed(true,false,1,1,2,2));
static_assert(!rawFinalCopyAllowed(false,true,1,1,2,2));

// GetRawInputBuffer accepts partial batches: the caller may provide room
// for one RAWINPUT even when the current virtual plan has many transitions.
// Advance ONLY the state represented by successfully copied records.
// Re-planning after a partial read then emits the remaining transitions
// without repeating an already delivered press or cursor delta.
void consumeRawPress(unsigned bit,std::uint64_t latestCounts) {
    unsigned shift=0;
    for (unsigned bits=bit;bits>1;bits>>=1) shift+=8;
    const std::uint64_t field=0xffull<<shift;
    rawSeenCounts=(rawSeenCounts&~field)|(latestCounts&field);
}
void consumeRawRecord(const RAWINPUT& event,const RawPlan& plan) {
    if (event.header.dwType==RIM_TYPEKEYBOARD) {
        const unsigned bit=maskFor(event.data.keyboard.VKey);
        if (!bit) return;
        if (event.data.keyboard.Flags&RI_KEY_BREAK) rawHeldMask&=~bit;
        else {
            rawHeldMask|=bit;
            consumeRawPress(bit,plan.counts);
        }
        return;
    }
    if (event.header.dwType!=RIM_TYPEMOUSE) return;
    const USHORT flags=event.data.mouse.usButtonFlags;
    if (flags&RI_MOUSE_LEFT_BUTTON_DOWN) {
        rawHeldMask|=64u;consumeRawPress(64u,plan.counts);
    }
    if (flags&RI_MOUSE_LEFT_BUTTON_UP) rawHeldMask&=~64u;
    if (flags&RI_MOUSE_RIGHT_BUTTON_DOWN) {
        rawHeldMask|=128u;consumeRawPress(128u,plan.counts);
    }
    if (flags&RI_MOUSE_RIGHT_BUTTON_UP) rawHeldMask&=~128u;
    if (event.data.mouse.lLastX || event.data.mouse.lLastY) {
        rawSeenX=plan.cursorX;rawSeenY=plan.cursorY;
    }
}
// The game-provided output buffer belongs to the virtual input consumer.
// Never pass it to the genuine OS Raw Input reader: even a temporary
// physical-keyboard write exposes MC01's real events to the MC02 caller.
// Drain the actual queue using a separate, bounded, aligned scratch buffer.
// A large HID packet or continuous queue that cannot be drained fails
// closed instead of returning unfiltered events or inventing success.
bool discardPhysicalRawInput(UINT headerBytes,DWORD& failure) {
    alignas(8) std::array<RAWINPUT,64> scratch{};
    constexpr unsigned MaxDrainPasses=16;
    for (unsigned pass=0;pass<MaxDrainPasses;++pass) {
        UINT capacity=static_cast<UINT>(sizeof(scratch));
        const UINT count=originalRawBuffer(scratch.data(),&capacity,headerBytes);
        if (count==UINT(-1)) {
            failure=GetLastError();
            return false;
        }
        if (count==0) return true;
    }
    failure=ERROR_BUSY; // Bounded fail-closed: never pretend queue is empty.
    return false;
}
UINT WINAPI observedRawBuffer(PRAWINPUT data,PUINT size,UINT headerBytes) {
    if (!view) return originalRawBuffer(data,size,headerBytes);
    const DWORD previous=GetLastError();
    auto word=currentPacket();
    if (!virtualMode(word)) {
        SetLastError(previous);
        return originalRawBuffer(data,size,headerBytes);
    }
    // If the opted-in game resumed polling just as its input TTL
    // expired, build the zero-word release plan NOW. Preserve the
    // existing rawHeldMask and delayed key-up/mouse-up events.
    if (!packetLiveAt(word,GetTickCount())) word=0;
    // Once this opted-in AI child is isolated, malformed buffer arguments
    // must never bypass the virtual reader and expose the physical queue
    // through the genuine GetRawInputBuffer. Refuse without touching caller
    // memory or advancing the virtual release/button cursors.
    if (!size || headerBytes!=sizeof(RAWINPUTHEADER)) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return UINT(-1);
    }
    // An explicit ReleaseAll (or expiry/MC01 focus loss) clears the
    // effective input word, but retains the exact MC02 HWND. Permit
    // a hidden/minimized owner ONLY for pending release records. The
    // focus-spoof hooks still use the strict visible-window selector,
    // and an active word can never inject into a hidden game window.
    const HWND window=ownedGameWindow(word==0);
    // A buffered raw read consumes the calling thread's queue, not all
    // windows' queues. Only the verified MC02 OGLES window thread can
    // see synthesized buffered events.
    const bool gameThread=window &&
        GetWindowThreadProcessId(window,nullptr)==GetCurrentThreadId();
    if (rawBufferBusy.test_and_set(std::memory_order_acquire)) {
        *size=0;SetLastError(previous);return 0;
    }
    auto plan=gameThread?nextRawPlan(word):RawPlan{};
    if (!data) {
        // Win32 size query never consumes the action. The next call
        // with a real buffer must still see the same queued transitions.
        *size=plan.count?static_cast<UINT>(sizeof(RAWINPUT)):0;
        rawBufferBusy.clear(std::memory_order_release);
        SetLastError(previous);return 0;
    }
    const UINT capacity=*size;
    // The caller's data and size must remain untouched by real physical
    // Raw Input. Suppression uses a private fixed-capacity scratch buffer.
    DWORD physicalFailure=ERROR_SUCCESS;
    if (!discardPhysicalRawInput(headerBytes,physicalFailure)) {
        *size=0;
        rawBufferBusy.clear(std::memory_order_release);
        SetLastError(physicalFailure);
        return UINT(-1);
    }
    // The bounded real-queue drain may outlast a live virtual packet.
    // If its newest read is ZERO, the previous plan's presses/camera
    // must be discarded. However, already-delivered DOWNs still need
    // key-up/mouse-up records. Build a fresh release-only plan on THIS
    // poll, with a new even-revision check, without touching an
    // unverified game thread or unconsumed release cursor.
    const auto afterDrain=currentPacket();
    if (rawCanReplanRelease(word,afterDrain,gameThread,rawHeldMask)) {
        word=0;
        plan=nextRawPlan(0);
    }
    // All other changes (including a NEW active packet) still refuse
    // this old plan. Never copy stale down/camera actions to the game.
    // A command may expire while the genuine raw queue is being drained.
    // Never commit synthetic mouse/key actions after the deadline or
    // after MC01 has lost real Windows foreground.
    // A native write can begin WHILE the real Raw Input queue is being
    // discarded. Refuse a stale plan even if the 64-bit packet happens
    // to be unchanged (e.g. a zero packet during a release transaction).
    // After a failed read, held state and per-key/camera cursors are
    // intentionally unchanged; the next game poll can finish the release.
    // The retained OGLES window revalidation uses Win32 calls which
    // can return after an active packet has expired. Finish THAT
    // potentially slow check before sampling the final live packet.
    // The last revision read follows currentPacket(), so a native
    // ReleaseAll or replacement write begun during the final sample
    // also cannot authorize copying stale DOWN/camera events.
    const bool sameWindow=plan.coherent &&
        (plan.count==0 || ownedGameWindow(word==0)==window);
    const auto latestWord=sameWindow?currentPacket():0;
    const auto latestRevision=sameWindow?
        view->cursorRevision.load(std::memory_order_seq_cst):0;
    if (!rawFinalCopyAllowed(plan.coherent,sameWindow,
            word,latestWord,plan.revision,latestRevision)) {
        // An invalidated plan must not consume any per-button or
        // cursor state. Next actual game poll can recover pending UPs.
        *size=0;
        rawBufferBusy.clear(std::memory_order_release);
        SetLastError(previous);return 0;
    }
    const UINT blockBytes=static_cast<UINT>(sizeof(RAWINPUT));
    // Win32 GetRawInputBuffer returns the number of whole records that
    // fit. ERROR_INSUFFICIENT_BUFFER applies only when even one required
    // record cannot fit, not when a larger batch must be split.
    if (plan.count!=0 && capacity<blockBytes) {
        *size=blockBytes;
        rawBufferBusy.clear(std::memory_order_release);
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return UINT(-1);
    }
    const UINT delivered=rawCopiedRecordCount(plan.count,capacity);
    const UINT bytes=delivered*blockBytes;
    if (bytes) {
        // A native ReleaseAll or replacement packet can arrive even
        // BETWEEN the P41 final admission and this bounded memcpy.
        // Copy first, then validate the *same* packet and even revision
        // once more before reporting any records or committing read
        // cursors. A failed post-copy check withdraws the entire prefix.
        std::memcpy(data,plan.events.data(),bytes);
        const auto postCopyWord=currentPacket();
        const auto postCopyRevision=
            view->cursorRevision.load(std::memory_order_seq_cst);
        if (!rawFinalCopyAllowed(plan.coherent,sameWindow,
                word,postCopyWord,plan.revision,postCopyRevision)) {
            // The output caller observes no synthetic records (return
            // count 0). Scrub the provisional bytes and NEVER advance
            // rawHeldMask, press counters or camera read coordinates.
            std::memset(data,0,bytes);
            *size=0;
            rawBufferBusy.clear(std::memory_order_release);
            SetLastError(previous);return 0;
        }
    }
    *size=bytes;
    // A newly observed epoch is not consumption: reset per-key and
    // camera cursors ONLY when one or more records were copied and
    // the post-copy source identity remained stable.
    if (rawMayAdvanceReadCursor(delivered) && rawSeenEpoch!=plan.epoch) {
        rawSeenEpoch=plan.epoch;rawSeenCounts=0;
        rawSeenX=0;rawSeenY=0;
    }
    // Do not count size queries or undelivered plan suffixes. The read
    // cursor advances after each actually copied record only.
    for (UINT i=0;i<delivered;++i) {
        const auto& event=plan.events[i];
        consumeRawRecord(event,plan);
        if (event.header.dwType==RIM_TYPEKEYBOARD)
            noteVirtualRead(sawVirtualKeyboard,
                L"[LT_MC02_INPUT][VIRTUAL_KEY_RETURNED]\n");
        else if (event.header.dwType==RIM_TYPEMOUSE) {
            if (event.data.mouse.usButtonFlags)
                noteVirtualRead(sawVirtualButtons,
                    L"[LT_MC02_INPUT][VIRTUAL_BUTTON_RETURNED]\n");
            if (event.data.mouse.lLastX || event.data.mouse.lLastY)
                noteVirtualRead(sawVirtualLook,
                    L"[LT_MC02_INPUT][VIRTUAL_LOOK_RETURNED]\n");
        }
    }
    rawBufferBusy.clear(std::memory_order_release);
    SetLastError(previous);
    return delivered;
}
#endif // BOOTSTRAP_MC02_RAW_BUFFER_POLL
} // namespace
namespace mc02_input {
bool attach(const education_bootstrap::Config& config) noexcept {
    // Every opted-in game gets its OWN inherited read-only channel.
    // No mapping in the child => its native hooks remain entirely inert.
    if (!education_bootstrap::validSlot(config.slot)) return false;
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
        state->slot!=config.slot||state->pid!=GetCurrentProcessId()||
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
#if defined(BOOTSTRAP_MC02_RAW_BUFFER_POLL)
    originalRawBuffer=reinterpret_cast<RawBufferCall>(
        GetProcAddress(user,"GetRawInputBuffer"));
    if (!originalRawBuffer) {UnmapViewOfFile(mapped);return false;}
#endif
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
                reinterpret_cast<PVOID>(observedFocus))==NO_ERROR
#if defined(BOOTSTRAP_MC02_RAW_BUFFER_POLL)
        && DetourAttach(reinterpret_cast<PVOID*>(&originalRawBuffer),
                reinterpret_cast<PVOID>(observedRawBuffer))==NO_ERROR
#endif
        ;
}
} // namespace mc02_input
