# Current Development Handoff — Lazy Tester

`Local` is the only active product branch. Sole current handoff.
Full goal: modular five-profile native control center, manual launches, one active
viewport/four previews, storage/cache management, archival and safe packaging.
Renderer/RAM experiments remain PAUSED; GitHub Actions remains entirely OFF.

## Current bounded outcome — reconnect to the retained session controller

Source parent: 89b9ad0195e824fa4fd3b0b40bb6b7d89ae5e597.
The previous desktop integration and evidence are preserved at that exact commit.

The existing bootstrap still exclusively owns the five original game/config/
storage/FPS handles. Its actor now receives fixed-action requests through
native_clients_session/pipe.rs instead of a GUI-lifetime stdin connection.
Reopening the same application binary connects to the existing endpoint first.
Startup/refresh inspection never launches a controller or game.

The endpoint is scoped to the current executable file identity and Windows
logon/session, with a protected logon-SID DACL, local-only pipe, noninheritable
handles, one retained first instance, and no persistent readiness/capability file.
Both ends verify actual OS peer PID, retained creation time/liveness, user, logon,
authentication ID, session, integrity and pinned executable file identity before
the application request. Client SQOS allows identification, not delegation.
This is not an elevated/general broker or a security boundary against malicious
code already controlling the same Windows user.

Fresh explicitly requested launch/storage operations spawn a controller only if
the endpoint is absent. Busy, access-denied or changed peers never authorize a
replacement. If an incompatible/initializing controller still holds the shared
session gate, an absent endpoint is reported as unavailable, not five stopped
profiles. Controller startup has a bounded ready/error handshake. A timeout
retains the original worker, buffers and connection; it does not retry or spawn.
Further mutation requires successful inspection after that pending call settles.
An admitted launch is not canceled when the UI disconnects.

A detached controller retains games and policy until normal game exit. It can
accept the next GUI process and return its original slot attribution. A failed
transport also retains live game/storage/FPS ownership rather than dropping it.
Closing the UI still releases native presentation back to Windows.

## Concrete defect found and fixed

The workspace still had the historical four-lease guard. It now retains five
leases and refuses a sixth without replacing an existing game. The existing
interactive native-window regression was extended to five identities/round trips
and sixth rejection. It was compiled but NOT_EXECUTED because it manipulates
fixture windows/focus and the operator's working desktop must not be disturbed.

The first pipe fixture failed with access denied even though AccessCheck granted
the initial data-only access mask. The final client uses documented GENERIC_READ
plus FILE_WRITE_DATA, with a DACL granting only FILE_GENERIC_READ | FILE_WRITE_DATA.
Native connect/query/round-trip now succeeds. AccessCheck independently confirms
that FILE_CREATE_PIPE_INSTANCE (FILE_APPEND_DATA) is NOT granted. No Everyone,
Administrators, NULL DACL, generic-write grant or elevation fallback was added.

Primary API references:
- https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights
- https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-client
- https://learn.microsoft.com/en-us/windows/win32/ipc/impersonating-a-named-pipe-client

## Executed evidence for this change

- Windows Rust check PASS; existing 104 desktop warnings remain.
- Nine targeted Rust tests PASS: bounded request protocol; native logon membership;
  positive/negative native pipe AccessCheck; first-instance collision and oversized
  message rejection; same-process reconnect/contract round trips; mismatched image/
  session rejection with recovery; two successive distinct self-test client
  processes against the same retained server; incompatible-controller gate lifetime;
  the test-only client entry.
- Transport cancellation settled and joined in the disposable fixtures.
- All tests used only disposable pipe endpoints and our own test executables.
  No Minecraft, account/world files, UI control, trace/UAC or installer operation.
- Windows x64 release manager build with explicit preview overlay PASS.
  SHA-256: 9a77f6e2cbcc23782fd481ccea7a7925e7d7215111d8b3f0b3ee77f8d51ca32e.
- No dependency versions changed. Only existing windows-sys Win32 namespace
  features were enabled; the lock file is unchanged.
- Frontend code/contract are unchanged; previous 212-test/Svelte/frontend build
  evidence remains historical at 89b9ad0, not a newly executed frontend suite.

Actual GUI restart with five Minecraft games: NOT_EXECUTED.
Cross-version/cross-binary controller handoff: unsupported/unproven; image mismatch
refuses instead of adopting an old controller. Do not replace installed binaries
while retained game sessions remain active.

## Remaining full-goal work

- Four lower live previews; currently status/selection cards only.
- Automatic all-session adoption into the browser-like taskbar/workspace.
  Task Manager grouping remains unproven; game processes remain visible.
- Cache classification/safe deletion, free-space reporting and storage UX.
- Installed Lazy Tester branding/identity migration and native payload packaging.
  Current recipe/capabilities still bind to the reviewed local game/profiles.
- Physical legacy source relocation after shared consumers are retired.
  Native DLL source in bootstrap-ordering is active and must not be archived.
- Integrated UI/gameplay/restart/preview acceptance remains unexecuted.
- Primary D: checkout is still older; it was not reset or overwritten.

## Next — one action

Implement the four lower native live previews through the existing workspace
owner, preserving selected-game input, background 10-FPS policy and recovery.
Use native composition with bounded geometry/identity validation; do not introduce
a continuous screenshot/encoding pipeline or claim Minecraft preview acceptance
from API registration alone.

## Preserved baseline

Archive branch codex/archive-before-lazy-tester-20261026 still pins
d166b5c8b24e8435f2c84675cb00a61c753aeb6a. Only Local receives product work.
The previously accepted private five-client pilot had five licensed accounts in
one Education Host World and persistent login/world/FPS after normal restart.
New UI/controller acceptance is separate. Compatibility C++ DLL bytes, original
profile capabilities, accounts/worlds and the 144/10 FPS implementation were not
changed or operated by this increment. Approved automatic graphics presets stay
integrated, but no current options.txt was written during this work.
