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
#include <limits>
#include <mutex>
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
// A rejected controller publication can contain unread rising-edge
// counters and accumulated relative look. A later HEARTBEAT can increment
// the revision while retaining that SAME cursor epoch and backlog.
// Reject the entire contaminated epoch, not just one packet revision:
// only an original controller ReleaseAll/expiry followed by a genuinely
// new epoch can admit active input again. This is a process-local safety
// floor, not an ABI field, independent driver, or additional lease owner.
std::atomic<std::uint64_t> rejectedThroughEpoch{0};
std::atomic<bool> isolated{false}; // latched after first valid AI command
std::atomic<bool> cursorInitialized{false};
std::atomic<LONG> virtualX{0},virtualY{0};
// Only virtual cursor position is synchronized here. The lock never
// guards a real Win32 call, native IPC, Detours trampoline or diagnostic.
// It prevents a delayed first real-pointer sample from overwriting a
// later AI recenter, and prevents GetCursorPos seeing mixed X/Y updates.
// The separate cursorReadBusy still owns cumulative look read cursors.
std::mutex cursorPositionMutex;
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
// Different packet revisions in one shared-memory epoch retain the SAME
// cumulative mouse and button history. A rejected key-down at revision 2
// must not reappear as a "new" short click in a look-only revision 4.
// The original Rust writer advances cursorEpoch on ReleaseAll/expiry,
// giving a clean re-admission boundary without any new IPC contract.
constexpr bool freshNativeInputEpoch(std::uint64_t epoch,
                                     std::uint64_t rejectedThrough) noexcept {
    return epoch!=0 && epoch>rejectedThrough;
}
static_assert(freshNativeInputEpoch(2,1));  // new independent epoch
static_assert(!freshNativeInputEpoch(1,1)); // heartbeat in revoked epoch
static_assert(!freshNativeInputEpoch(1,2)); // older epoch
static_assert(!freshNativeInputEpoch(0,0)); // uninitialized channel
// A coherent, nonzero native packet means this original AI process has
// entered its isolated-input epoch, EVEN when the packet is rejected by
// deadline, focus, or root-window admission. A rejected first command
// must never let subsequent GetRawInputBuffer/KeyState reads fall back
// to a human's physical input after the AI window regains foreground.
// A neutral-only initial recovery write (packet=0) does not latch the
// experimental input mode, so the original user-driven login remains usable.
constexpr bool observedCommittedActivePacket(std::uint64_t word,
    std::uint64_t initialRevision,std::uint64_t finalRevision) noexcept {
    return word!=0 && initialRevision!=0 && !(initialRevision&1) &&
        initialRevision==finalRevision;
}
static_assert(observedCommittedActivePacket(1,2,2));
static_assert(!observedCommittedActivePacket(0,2,2)); // neutral
static_assert(!observedCommittedActivePacket(1,2,4)); // changed
static_assert(!observedCommittedActivePacket(1,3,3)); // mid-write
static_assert(!observedCommittedActivePacket(1,0,0)); // uninitialized
// Multiple original game threads can observe failures concurrently.
// Never let a late rejection of an older epoch decrease the fence.
// Record only a coherent publisher snapshot: an in-flight writer is not
// evidence that the next packet/epoch itself should be rejected.
void rejectActiveEpochIfCurrent(std::uint64_t revision,std::uint64_t epoch) {
    if (!view || !revision || !epoch ||
        view->cursorRevision.load(std::memory_order_seq_cst)!=revision ||
        view->cursorEpoch.load(std::memory_order_seq_cst)!=epoch) return;
    auto previous=rejectedThroughEpoch.load(std::memory_order_seq_cst);
    while (previous<epoch &&
           !rejectedThroughEpoch.compare_exchange_weak(
               previous,epoch,std::memory_order_seq_cst,
               std::memory_order_seq_cst)) {}
}

std::uint64_t currentPacket(std::uint64_t* admittedRevision=nullptr) {
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
    const auto epoch=view->cursorEpoch.load(std::memory_order_seq_cst);
    // The first coherent ACTIVE publication latches physical-input
    // isolation before *any* deadline/focus/HWND refusal. Previously a
    // rejected initial command returned zero without setting isolated,
    // letting a later foreground AI game poll real keys/Raw Input.
    // Do not latch for an incomplete writer transaction or neutral-only
    // startup reconciliation.
    if (observedCommittedActivePacket(word,revision,
            view->cursorRevision.load(std::memory_order_seq_cst)))
        isolated.store(true,std::memory_order_release);
    // A revision-only fence would admit the next heartbeat in the same
    // epoch, potentially replaying presses/look that were rejected earlier.
    // Deny the full epoch until the controller publishes a fresh one.
    if (!word) return 0;
    if (view->cursorRevision.load(std::memory_order_seq_cst)!=revision)
        return 0;
    if (!freshNativeInputEpoch(epoch,
         rejectedThroughEpoch.load(std::memory_order_seq_cst))) return 0;
    if (!packetLiveAt(word,GetTickCount()) || !originalForeground) {
        rejectActiveEpochIfCurrent(revision,epoch);
        return 0;
    }
    // Only the genuine Win32 foreground is authoritative for the hybrid
    // human MC01; five-AI has no artificial human foreground requirement.
    const HWND host=hostRaw?reinterpret_cast<HWND>(
        static_cast<ULONG_PTR>(hostRaw)):nullptr;
    if (host && originalForeground()!=host) {
        rejectActiveEpochIfCurrent(revision,epoch);
        return 0;
    }
    // Check the exact root AFTER focus, and recheck focus AFTER potentially
    // delayed Win32 HWND/class lookups. Loss of either condition rejects
    // the entire accumulated input epoch. A later same-epoch heartbeat
    // cannot replay pending key/button/cursor deltas on focus restoration.
    const bool originalRoot=ownedGameWindow()!=nullptr;
    const bool sameRevision=
        view->cursorRevision.load(std::memory_order_seq_cst)==revision;
    const bool stillLive=packetLiveAt(word,GetTickCount());
    const bool humanFocusIntact=!host || originalForeground()==host;
    if ((!originalRoot || !stillLive || !humanFocusIntact) && sameRevision)
        rejectActiveEpochIfCurrent(revision,epoch);
    if (!activePacketAdmitted(originalRoot&&humanFocusIntact,
                              sameRevision,stillLive) ||
        !freshNativeInputEpoch(epoch,
         rejectedThroughEpoch.load(std::memory_order_seq_cst))) return 0;
    if (admittedRevision) *admittedRevision=revision;
    return word;
}
// A controller ReleaseAll or later active packet can publish AFTER the
// first currentPacket read but BEFORE a Win32 hook returns its result.
// Both the word and its 64-bit even publication revision must still
// match, with fresh native focus/OGLES/expiry/epoch admission. Equal
// key masks in successive native packets never justify stale return.
constexpr bool sameActivePublication(std::uint64_t initialWord,
    std::uint64_t finalWord,std::uint64_t initialRevision,
    std::uint64_t finalRevision) noexcept {
    return initialWord!=0 && initialWord==finalWord &&
        initialRevision!=0 && !(initialRevision&1) &&
        initialRevision==finalRevision;
}
static_assert(sameActivePublication(1,1,2,2));
static_assert(!sameActivePublication(1,0,2,4)); // neutral
static_assert(!sameActivePublication(1,1,2,4)); // identical word, newer revision
static_assert(!sameActivePublication(1,2,2,2)); // different word
static_assert(!sameActivePublication(1,1,3,3)); // incomplete publication
static_assert(!sameActivePublication(0,0,2,2)); // neutral is not active
bool sameLiveActivePublication(std::uint64_t word,std::uint64_t revision) {
    if (!word || !revision) return false;
    std::uint64_t latestRevision=0;
    const auto latest=currentPacket(&latestRevision);
    return sameActivePublication(word,latest,revision,latestRevision);
}
// Lightweight LAST fence after a potential DebugString/keyboard-array
// delay. The first and second currentPacket calls already verified real
// focus and the exact OGLES root; this final check only ensures that
// the original Rust writer has not started or finished a replacement,
// ReleaseAll, epoch change or expiry since those full Win32 checks.
// Do not pay for another HWND/class/foreground lookup on every key poll.
constexpr bool nativePublicationStillCurrent(bool sameRevision,
    bool sameWord,bool epochAllowed,bool live) noexcept {
    return sameRevision && sameWord && epochAllowed && live;
}
static_assert(nativePublicationStillCurrent(true,true,true,true));
static_assert(!nativePublicationStillCurrent(false,true,true,true));
static_assert(!nativePublicationStillCurrent(true,false,true,true));
static_assert(!nativePublicationStillCurrent(true,true,false,true));
static_assert(!nativePublicationStillCurrent(true,true,true,false));
bool sameNativePublicationAtReturn(std::uint64_t word,
                                   std::uint64_t revision) {
    if (!view || !word || !revision) return false;
    const auto before=view->cursorRevision.load(std::memory_order_seq_cst);
    if (before!=revision) return false;
    const auto packet=view->packet.load(std::memory_order_seq_cst);
    const auto epoch=view->cursorEpoch.load(std::memory_order_seq_cst);
    const bool safe=nativePublicationStillCurrent(
        before==revision,packet==word,
        freshNativeInputEpoch(epoch,
            rejectedThroughEpoch.load(std::memory_order_seq_cst)),
        packetLiveAt(word,GetTickCount()));
    return safe && view->cursorRevision.load(std::memory_order_seq_cst)==revision;
}
// SetCursorPos can legitimately recenter an isolated game menu after
// native ReleaseAll (packet 0), without re-enabling physical OS input.
// Such a recenter must not cross an in-flight/new active publication.
// Revision 0 + packet 0 is also the initial unmoved channel.
constexpr bool sameNeutralPublication(std::uint64_t before,
    std::uint64_t after,std::uint64_t packet) noexcept {
    return !(before&1) && before==after && packet==0;
}
static_assert(sameNeutralPublication(0,0,0));
static_assert(sameNeutralPublication(4,4,0));
static_assert(!sameNeutralPublication(4,6,0)); // newer native write
static_assert(!sameNeutralPublication(5,5,0)); // in-flight native write
static_assert(!sameNeutralPublication(4,4,1)); // newly active command
bool neutralPublicationAtReturn(std::uint64_t revision) {
    if (!view) return false;
    const auto before=view->cursorRevision.load(std::memory_order_seq_cst);
    if (before!=revision || (before&1)) return false;
    const auto packet=view->packet.load(std::memory_order_seq_cst);
    return sameNeutralPublication(before,
        view->cursorRevision.load(std::memory_order_seq_cst),packet);
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
    std::uint64_t revision=0;
    const auto word=currentPacket(&revision);
    const HWND owned=word ? ownedGameWindow() : nullptr;
    // The owned-root lookup can outlast ReleaseAll, a new native packet,
    // TTL expiry or a hybrid MC01 foreground change. Reuse the exact
    // live-publication gate already required for virtual key/cursor reads:
    // stale game-local focus must never be returned after neutralization.
    const bool liveOwned=owned && sameLiveActivePublication(word,revision);
    if (liveOwned) isolated.store(true,std::memory_order_release);
    SetLastError(previous);
    return liveOwned ? owned : real;
}
HWND WINAPI observedActive() {
    const DWORD previous=GetLastError();
    const HWND real=originalActive();
    std::uint64_t revision=0;
    const auto word=currentPacket(&revision);
    const HWND owned=word ? ownedGameWindow() : nullptr;
    // GetActiveWindow is scoped to the caller's thread, unlike the
    // process-wide GetForegroundWindow query. A stopped/replaced original
    // channel cannot spoof active status while that thread check runs.
    const bool ownThread=owned &&
        GetWindowThreadProcessId(owned,nullptr)==GetCurrentThreadId() &&
        sameLiveActivePublication(word,revision);
    if (ownThread) isolated.store(true,std::memory_order_release);
    SetLastError(previous);
    return ownThread ? owned : real;
}
HWND WINAPI observedFocus() {
    const DWORD previous=GetLastError();
    const HWND real=originalFocus();
    std::uint64_t revision=0;
    const auto word=currentPacket(&revision);
    const HWND owned=word ? ownedGameWindow() : nullptr;
    // GetFocus is thread-local too. Never pretend that a worker thread
    // owns the Minecraft game window's message queue, nor return stale
    // virtual focus after native ReleaseAll or a human focus change.
    const bool ownThread=owned &&
        GetWindowThreadProcessId(owned,nullptr)==GetCurrentThreadId() &&
        sameLiveActivePublication(word,revision);
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
    if (useVirtual && !cursorInitialized.load(std::memory_order_acquire)) {
        // Read the physical initial point OUTSIDE the position lock.
        // Another original game thread can SetCursorPos while this
        // trampoline is delayed; the later recenter wins that race.
        POINT point{};
        const bool sampled=originalCursor && originalCursor(&point);
        std::lock_guard<std::mutex> lock(cursorPositionMutex);
        if (!cursorInitialized.load(std::memory_order_relaxed)) {
            if (sampled) {
                virtualX.store(point.x,std::memory_order_release);
                virtualY.store(point.y,std::memory_order_release);
            }
            cursorInitialized.store(true,std::memory_order_release);
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
// Read only a tentative edge snapshot under the existing per-process
// async-key cursor lock. Commit counts AFTER verifying the same complete
// native publication at return time; a revoked/neutral packet must not
// consume a press which the game never received from this polling call.
SHORT virtualAsyncKeyResult(int key,std::uint64_t word,
                            std::uint64_t revision) {
    if (!word || !revision) return 0;
    const unsigned keyMask=maskFor(key);
    bool cursorHeld=false,coherent=false;
    std::uint64_t counts=0,epoch=0,priorEpoch=0,priorCounts=0;
    if (keyMask && !asyncReadBusy.test_and_set(std::memory_order_acquire)) {
        cursorHeld=true;
        // Both the cursor snapshot and the later tentative commit/rollback
        // belong to this same original asyncReadBusy critical section.
        // Never read non-atomic seenPress* if another key poll owns it.
        priorEpoch=seenPressEpoch;
        priorCounts=seenPressCounts;
        const auto before=view->cursorRevision.load(std::memory_order_seq_cst);
        if (before==revision) {
            counts=view->pressCounts.load(std::memory_order_seq_cst);
            epoch=view->cursorEpoch.load(std::memory_order_seq_cst);
            coherent=view->packet.load(std::memory_order_seq_cst)==word &&
                view->cursorRevision.load(std::memory_order_seq_cst)==revision;
        }
    }
    // Revalidation must follow the potentially delayed shared-state and
    // window reads. No per-key state is consumed if it fails.
    const bool admitted=sameLiveActivePublication(word,revision);
    // Restore the locked snapshot if a native neutral/replacement
    // arrives during the optional one-shot diagnostic.
    bool pressed=false;
    if (admitted && coherent && cursorHeld) {
        if (epoch!=seenPressEpoch) {
            seenPressEpoch=epoch;
            seenPressCounts=0;
        }
        unsigned shift=0;
        for (unsigned bits=keyMask;bits>1;bits>>=1) shift+=8;
        const std::uint64_t mask=0xffull<<shift;
        if ((seenPressCounts&mask)!=(counts&mask)) {
            seenPressCounts=(seenPressCounts&~mask)|(counts&mask);
            pressed=true;
        }
    }
    const SHORT result=admitted?static_cast<SHORT>(
        (virtualDown(key,word)?0x8000u:0u) | (pressed?1u:0u)):0;
    if (result) noteVirtualKeys(keyMask);
    const bool finalAdmitted=admitted && (!result ||
        sameNativePublicationAtReturn(word,revision));
    if (!finalAdmitted && cursorHeld) {
        seenPressEpoch=priorEpoch;
        seenPressCounts=priorCounts;
    }
    if (cursorHeld) asyncReadBusy.clear(std::memory_order_release);
    return finalAdmitted?result:0;
}
SHORT WINAPI observedAsync(int key) {
    const DWORD previous=GetLastError();
    std::uint64_t revision=0;
    const auto word=currentPacket(&revision);
    if (!virtualMode(word)) {
        SetLastError(previous); return originalAsync(key);
    }
    const SHORT result=virtualAsyncKeyResult(key,word,revision);
    SetLastError(previous);
    return result;
}
SHORT WINAPI observedKey(int key) {
    const DWORD previous=GetLastError();
    std::uint64_t revision=0;
    const auto word=currentPacket(&revision);
    if (!virtualMode(word)) {
        SetLastError(previous); return originalKey(key);
    }
    const bool admitted=sameLiveActivePublication(word,revision);
    SHORT result=admitted && virtualDown(key,word)?
        static_cast<SHORT>(0x8000):0;
    if (result) {
        noteVirtualKeys(maskFor(key));
        // Diagnostic hooks can delay this thread. Never return a held
        // button from a packet the original native actor has neutralized.
        if (!sameNativePublicationAtReturn(word,revision)) result=0;
    }
    SetLastError(previous);
    return result;
}
BOOL WINAPI observedKeyboard(PBYTE state) {
    const DWORD previous=GetLastError();
    std::uint64_t revision=0;
    const auto word=currentPacket(&revision);
    if (!virtualMode(word) || !state) {
        SetLastError(previous); return originalKeyboard(state);
    }
    // Populate into caller-owned memory, then retract every virtual key
    // if the controller published ReleaseAll/replacement while filling.
    // This hook must never forward the physical human keyboard once
    // its original AI channel was isolated.
    std::memset(state,0,256);
    for (int key : {'W','S','A','D',VK_SPACE,VK_SHIFT,VK_LBUTTON,VK_RBUTTON}) {
        if (virtualDown(key,word)) state[key]=0x80;
    }
    if (virtualDown(VK_SHIFT,word)) state[VK_LSHIFT]=state[VK_RSHIFT]=0x80;
    if (!sameLiveActivePublication(word,revision)) {
        std::memset(state,0,256);
    } else {
        noteVirtualKeys(static_cast<unsigned>(word)&0xffu);
        // Revoke even an already-populated 256-byte virtual keyboard
        // array if ReleaseAll/replacement began while the diagnostic ran.
        if (!sameNativePublicationAtReturn(word,revision))
            std::memset(state,0,256);
    }
    SetLastError(previous);
    return TRUE;
}

// Both optional camera consumers (virtual GetCursorPos and buffered
// Raw Input) read the ORIGINAL Rust writer's saturating i64 totals.
// Cap the next step BEFORE subtracting: x-baseX can overflow a signed
// i64 at opposite saturation limits. A failure in buffered planning
// must never prevent already-held key-up/mouse-up transitions.
// The unconsumed suffix remains available within the current epoch.
constexpr std::int64_t boundedVirtualLookStep(std::int64_t target,
                                             std::int64_t consumed) noexcept {
    constexpr std::int64_t limit=32767;
    constexpr auto maximum=std::numeric_limits<std::int64_t>::max();
    constexpr auto minimum=std::numeric_limits<std::int64_t>::min();
    if (target>=consumed) {
        if (consumed<=maximum-limit && target>consumed+limit) return limit;
    } else if (consumed>=minimum+limit && target<consumed-limit) {
        return -limit;
    }
    return target-consumed;
}
static_assert(boundedVirtualLookStep(40000,0)==32767);
static_assert(boundedVirtualLookStep(40000,32767)==7233);
static_assert(boundedVirtualLookStep(-40000,0)==-32767);
static_assert(boundedVirtualLookStep(-40000,-32767)==-7233);
static_assert(boundedVirtualLookStep(0,0)==0);
static_assert(boundedVirtualLookStep(
    std::numeric_limits<std::int64_t>::max(),
    std::numeric_limits<std::int64_t>::min())==32767);
static_assert(boundedVirtualLookStep(
    std::numeric_limits<std::int64_t>::min(),
    std::numeric_limits<std::int64_t>::max())==-32767);
static_assert(boundedVirtualLookStep(32768,0)==32767);
static_assert(boundedVirtualLookStep(32768,32767)==1);
static_assert(boundedVirtualLookStep(-32768,0)==-32767);
static_assert(boundedVirtualLookStep(-32768,-32767)==-1);

#if !defined(BOOTSTRAP_MC02_RAW_BUFFER_POLL)
void applyPendingCursor(std::uint64_t word,std::uint64_t revision) {
    if (!view || !word || !revision ||
        cursorReadBusy.test_and_set(std::memory_order_acquire)) return;
    const auto before=view->cursorRevision.load(std::memory_order_seq_cst);
    if (before==revision) {
        const auto x=view->cursorX.load(std::memory_order_seq_cst);
        const auto y=view->cursorY.load(std::memory_order_seq_cst);
        const auto epoch=view->cursorEpoch.load(std::memory_order_seq_cst);
        // A partially consumed large delta must remain eligible even
        // when the controller has not published another revision.
        // On a new epoch, discard the earlier expired backlog entirely.
        const auto baseX=epoch==seenEpoch?seenX:0;
        const auto baseY=epoch==seenEpoch?seenY:0;
        if (before!=seenRevision || epoch!=seenEpoch ||
            x!=baseX || y!=baseY) {
            // The initial full admission checks the pinned game HWND,
            // real human focus (hybrid), input expiry and rejected epoch.
            // Recheck the same native publication after reading totals;
            // no read cursor is advanced on a late ReleaseAll.
            if (sameLiveActivePublication(word,revision) &&
                sameNativePublicationAtReturn(word,revision)) {
                const auto stepX=boundedVirtualLookStep(x,baseX);
                const auto stepY=boundedVirtualLookStep(y,baseY);
                bool committed=false;
                {
                    // Serialize the two virtual coordinates with a
                    // concurrent game-local SetCursorPos or GetCursorPos.
                    // Do not expose either axis until both are updated.
                    std::lock_guard<std::mutex> lock(cursorPositionMutex);
                    const LONG oldX=virtualX.load(std::memory_order_relaxed);
                    const LONG oldY=virtualY.load(std::memory_order_relaxed);
                    virtualX.fetch_add(static_cast<LONG>(stepX),
                        std::memory_order_acq_rel);
                    virtualY.fetch_add(static_cast<LONG>(stepY),
                        std::memory_order_acq_rel);
                    // ReleaseAll may have landed in the tiny interval
                    // between the earlier admission and these writes.
                    // Undo BOTH provisional axes while SetCursorPos and
                    // GetCursorPos are excluded; do not consume the delta.
                    committed=sameNativePublicationAtReturn(word,revision);
                    if (!committed) {
                        virtualX.store(oldX,std::memory_order_release);
                        virtualY.store(oldY,std::memory_order_release);
                    }
                }
                if (committed) {
                    seenEpoch=epoch;
                    seenRevision=before;
                    // Consume only the delivered bounded prefix. Its
                    // unconsumed remainder survives within this epoch.
                    seenX=baseX+stepX;
                    seenY=baseY+stepY;
                    if (stepX || stepY) noteVirtualRead(sawVirtualLook,
                        L"[LT_MC02_INPUT][VIRTUAL_LOOK_RETURNED]\n");
                }
            }
        }
    }
    cursorReadBusy.clear(std::memory_order_release);
}
#endif
BOOL WINAPI observedCursor(LPPOINT point) {
    const DWORD previous=GetLastError();
    std::uint64_t revision=0;
    const auto word=currentPacket(&revision);
    if (!virtualMode(word) || !point) {
        SetLastError(previous); return originalCursor(point);
    }
    // In the separate Raw-buffer mode, camera deltas are published ONLY
    // through GetRawInputBuffer; GetCursorPos retains menu cursor state.
#if !defined(BOOTSTRAP_MC02_RAW_BUFFER_POLL)
    if (word) applyPendingCursor(word,revision);
#endif
    {
        // Paired snapshot: no half-updated cursor while another game
        // thread recenters or consumes a relative camera delta.
        std::lock_guard<std::mutex> lock(cursorPositionMutex);
        point->x=virtualX.load(std::memory_order_acquire);
        point->y=virtualY.load(std::memory_order_acquire);
    }
    SetLastError(previous);
    return TRUE;
}
BOOL WINAPI observedSetCursor(int x,int y) {
    const DWORD previous=GetLastError();
    std::uint64_t revision=0;
    const auto word=currentPacket(&revision);
    // currentPacket intentionally does not expose a revision for neutral
    // or refused active input. Only a real zero packet (or the unmoved
    // initial mapping) can authorize a neutral menu recenter.
    const auto neutralRevision=(!word && view)?
        view->cursorRevision.load(std::memory_order_seq_cst):0;
    if (!virtualMode(word)) {
        SetLastError(previous); return originalSetCursor(x,y);
    }
    // A full target/focus check belongs OUTSIDE the position lock.
    // A native mutation can still arrive while waiting for that lock,
    // so the final lightweight publication fence is checked inside it.
    const bool activeAdmitted=word && sameLiveActivePublication(word,revision);
    {
        std::lock_guard<std::mutex> lock(cursorPositionMutex);
        const auto stillCurrent=[&]() {
            return word ? activeAdmitted &&
                sameNativePublicationAtReturn(word,revision) :
                neutralPublicationAtReturn(neutralRevision);
        };
        if (stillCurrent()) {
            const LONG oldX=virtualX.load(std::memory_order_relaxed);
            const LONG oldY=virtualY.load(std::memory_order_relaxed);
            virtualX.store(x,std::memory_order_release);
            virtualY.store(y,std::memory_order_release);
            // Native ReleaseAll or a replacement packet may begin between
            // these two axis writes. Undo both provisional coordinates
            // without exposing a mixed pair to another game thread.
            if (!stillCurrent()) {
                virtualX.store(oldX,std::memory_order_release);
                virtualY.store(oldY,std::memory_order_release);
            } else {
                cursorInitialized.store(true,std::memory_order_release);
            }
        }
    }
    // A stale recenter must never fall back to the real Windows cursor.
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
        // Reuse the polling cursor's overflow-safe bounded delta.
        // Never evaluate signed x-baseX/y-baseY before the clamp:
        // the Rust writer can legitimately saturate either total
        // and RawPlan must still be able to emit held-button releases.
        const auto moveX=boundedVirtualLookStep(x,baseX);
        const auto moveY=boundedVirtualLookStep(y,baseY);
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
// Size-only GetRawInputBuffer calls do not consume records. They must
// never advertise an incoherent/torn plan as a pending down/up record.
// Only a presently admitted plan with at least one transition needs
// caller storage; the next real read rechecks everything independently.
constexpr UINT rawSizeQueryBytes(bool admitted,UINT count) noexcept {
    return admitted && count!=0?static_cast<UINT>(sizeof(RAWINPUT)):0u;
}
static_assert(rawSizeQueryBytes(false,1)==0u);
static_assert(rawSizeQueryBytes(false,25)==0u);
static_assert(rawSizeQueryBytes(true,0)==0u);
static_assert(rawSizeQueryBytes(true,1)==sizeof(RAWINPUT));
static_assert(rawSizeQueryBytes(true,25)==sizeof(RAWINPUT));

// A numeric HWND can be destroyed and reused while the genuine physical
// queue is being drained, or between provisional copy and final readback.
// The initial gameThread check alone is insufficient: revalidate the
// SAME owned OGLES root AND the current caller thread before accepting
// either a DOWN record or a pending key-up/mouse-up record. Only ReleaseAll
// may use the original root while hidden; it never admits a new active DOWN.
constexpr bool rawSameOriginalGameThread(bool sameHwnd,bool sameThread) noexcept {
    return sameHwnd && sameThread;
}
static_assert(rawSameOriginalGameThread(true,true));
static_assert(!rawSameOriginalGameThread(true,false)); // HWND reused on another thread
static_assert(!rawSameOriginalGameThread(false,true)); // replacement root, same thread
static_assert(!rawSameOriginalGameThread(false,false));
bool rawOriginalThreadStillOwns(HWND original,bool releaseOnly) {
    if (!original) return false;
    const HWND current=ownedGameWindow(releaseOnly);
    return current && rawSameOriginalGameThread(
        current==original,
        GetWindowThreadProcessId(current,nullptr)==GetCurrentThreadId());
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
        // A size-only query MUST NOT advertise a DOWN/camera record
        // from an incoherent plan after ReleaseAll or a replacement
        // native publication. Keep the full same-HWND/game-thread and
        // same-word/revision gate, without draining physical input or
        // consuming any virtual key-up/press/camera read cursor.
        bool admitted=false;
        if (plan.coherent && plan.count && gameThread &&
            rawOriginalThreadStillOwns(window,word==0)) {
            const auto latestWord=currentPacket();
            const auto latestRevision=
                view->cursorRevision.load(std::memory_order_seq_cst);
            admitted=rawFinalCopyAllowed(plan.coherent,true,
                word,latestWord,plan.revision,latestRevision);
        }
        *size=rawSizeQueryBytes(admitted,plan.count);
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
    // Even if Windows recycled the SAME numeric handle into a different
    // OGLES thread, that replacement must never inherit this old thread's
    // pending key-up/mouse-up or virtual cursor movement. Zero-record polls
    // have no synthetic effect and need no extra Win32 lookup.
    const bool sameWindow=plan.coherent &&
        (plan.count==0 || (gameThread &&
            rawOriginalThreadStillOwns(window,word==0)));
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
        // An HWND/thread ownership change after the first check can
        // occur during memcpy. Scrub the copied prefix and leave all raw
        // cursors untouched unless this very same original game thread
        // still owns the pinned root at post-copy admission.
        const bool postCopySameWindow=
            rawOriginalThreadStillOwns(window,word==0);
        const auto postCopyRevision=
            view->cursorRevision.load(std::memory_order_seq_cst);
        if (!rawFinalCopyAllowed(plan.coherent,postCopySameWindow,
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
