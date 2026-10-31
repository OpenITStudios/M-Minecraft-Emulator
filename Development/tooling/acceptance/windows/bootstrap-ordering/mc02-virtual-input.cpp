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
// Per-process read cursor for 8 key/mouse rising-edge counters. Separate
// nonblocking guard from cursor movement: async key and camera reads may
// run on different Minecraft threads without sharing a blocking lock.
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
// Resolve only the pinned MC02 OGLES root in THIS opted-in child.
// Active input and focus-spoof hooks must still require a visible,
// non-minimized game window. The buffered Raw Input reader MAY also
// resolve a hidden/minimized window solely to complete pending key-up
// and mouse-button UP records when currentPacket()==0. This never
// activates that window, changes focus or routes physical MC01 input.
HWND ownedGameWindow(bool permitHiddenForRelease=false) {
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
// GetAsyncKeyState returns the high bit for 'currently held' and the
// low bit for 'pressed since the previous call'. A last-packet key mask
// alone cannot report a short click followed by a release between polls.
// The original Win32 low bit is unreliable across callers, but this
// optional MC02-only read cursor is independent from MC01 hardware.
// Read under the existing writer's revision and reject torn snapshots.
bool virtualPressEdge(unsigned keyMask,std::uint64_t word) {
    if (!view || !word || !keyMask ||
        asyncReadBusy.test_and_set(std::memory_order_acquire)) return false;
    bool pressed=false;
    const auto revision=view->cursorRevision.load(std::memory_order_seq_cst);
    if (revision!=0 && !(revision&1)) {
        const auto counts=view->pressCounts.load(std::memory_order_seq_cst);
        const auto epoch=view->cursorEpoch.load(std::memory_order_seq_cst);
        const auto packet=view->packet.load(std::memory_order_seq_cst);
        if (packet==word &&
            view->cursorRevision.load(std::memory_order_seq_cst)==revision) {
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
    const unsigned keyMask=maskFor(key);
    const bool held=virtualDown(key,word);
    const bool newlyPressed=virtualPressEdge(keyMask,word);
    const SHORT result=static_cast<SHORT>(
        (held ? 0x8000u : 0u) | (newlyPressed ? 1u : 0u));
    if (word && result) noteVirtualKeys(keyMask);
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
    if (word && result) noteVirtualKeys(maskFor(key));
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
    if (word) noteVirtualKeys(static_cast<unsigned>(word)&0xffu);
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
    if (word) applyPendingCursor();
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
    if (!view||!size||headerBytes!=sizeof(RAWINPUTHEADER))
        return originalRawBuffer(data,size,headerBytes);
    const DWORD previous=GetLastError();
    const auto word=currentPacket();
    if (!virtualMode(word)) {
        SetLastError(previous);
        return originalRawBuffer(data,size,headerBytes);
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
    const auto plan=gameThread?nextRawPlan(word):RawPlan{};
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
    // A command may expire while the genuine raw queue is being drained.
    // Never commit synthetic mouse/key actions after the deadline or
    // after MC01 has lost real Windows foreground.
    // A native write can begin WHILE the real Raw Input queue is being
    // discarded. Refuse a stale plan even if the 64-bit packet happens
    // to be unchanged (e.g. a zero packet during a release transaction).
    // After a failed read, held state and per-key/camera cursors are
    // intentionally unchanged; the next game poll can finish the release.
    if (!plan.coherent || currentPacket()!=word ||
        view->cursorRevision.load(std::memory_order_seq_cst)!=plan.revision ||
        (plan.count!=0 && ownedGameWindow(word==0)!=window)) {
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
    const UINT available=capacity/blockBytes;
    const UINT delivered=plan.count<available?plan.count:available;
    const UINT bytes=delivered*blockBytes;
    if (bytes) std::memcpy(data,plan.events.data(),bytes);
    *size=bytes;
    // A command's epoch resets stale cursor/click counts, but does not
    // erase key-down state already observed by the game's Raw Input reader.
    if (rawSeenEpoch!=plan.epoch) {
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
