# Current Development Handoff — v0.1

## P0 five-launcher registered-process observation — 27 October 2026

The source audit found a mismatched capacity contract: Lazy Tester supports
five independently managed Minecraft Education profiles but the canonical
registered-native-process collector and its JSON verifier rejected more
than four candidates. The existing observation owner now admits up to five
registered-image process candidates using a shared fixed cap, including
windowless processes with HWND 0 that can survive an incomplete shutdown.
Six or more candidates still fail closed, as do duplicate PID, HWND,
unbounded or forged fields. Nothing in this observation grants game
ownership, account evidence or a new launcher. Synthetic fixtures cover
five distinct identities (one windowless) and six-candidate refusal.
No local PC, Windows, Minecraft or source execution took place. Tests
are authored, NOT_EXECUTED.

## P0 source audit — 27 October 2026 (no Local PC testing)

The operator explicitly requires ChatGPT + GitHub-only work. Windows, installer,
Minecraft and Local PC tests are not requested, authorized or executed. Exact
runtime completion remains UNKNOWN.

The source audit identified an actual unsafe window-attribution path: the
controller used a generic visible, unowned top-level HWND as a game's main window.
After OGLES disappeared, an unowned #32770 Minecraft Education exit dialog could
be selected, renamed to Minecraft - N by label_slot_window (WM_SETTEXT), then fail
the exact native-dialog title check or receive WM_CLOSE meant for OGLES.
This is now blocked by the reviewed OGLES window-class filter, including retained
HWND rechecks. Full exit also skips automatic caption labeling, and the exit
prompt is revalidated in full just before submitting IDOK. A disabled OGLES
window while its modal dialog is in transition is now deferred within the
existing deadline rather than aborting all sessions immediately.

Source-level correctness and fixture cases were authored, not run. The
original MC02 hang is NOT_ACCEPTED and no universal thread/process termination
fallback was introduced. Remaining work: source-only lifecycle/HUD ownership
review until the operator separately authorizes runtime acceptance.

## P0 source correction — dialog command is not game exit (27 October 2026)

The CloseAll request now tracks a bounded per-session sequence: verified game
WM_CLOSE request, exact modal confirmation request and ongoing observation of
the retained **game process handle**. Posting WM_COMMAND/IDOK is recorded once
per exact observed dialog (HWND + owner-thread ID), NOT interpreted as Minecraft
exit, and cannot stop the remaining game's normal-close processing. A distinct
verified prompt may be handled at most four times per attempt; foreign/changed
dialogs refuse. A game HWND that disappears between enumeration and its guarded
close request is re-observed rather than immediately failing the entire attempt.
The transient modal-disabled path remains deferrable. The process is accepted
as exited only after the original process handle signals and reap completes.
The existing normal-close deadline and no-force-kill safeguards remain in place.

Typed UI/session contracts, protected account/world storage and native DLLs were
not changed by this increment. A source-level fixture now asserts no duplicate
IDOK for one live dialog, no action after its text changes and diagnostics that
never classify a sent command as process exit. No tests were executed in
ChatGPT/GitHub. MC02 real-game completion remains NOT_EXECUTED and UNKNOWN.

## P0 source correction — HUD worker lifetime on destroyed game HWND (27 October 2026)

The existing HUD worker could remain running after its owner game window was
destroyed while the game's UI thread was still alive: it only exited on owner
thread termination, explicit STOP, or an exception. In addition, most worker
exit paths left `workerFinished=false`, so a later bind might mistake a
finished but still joinable thread for a live HUD worker. The source now exits
on a genuinely destroyed owner HWND (including an unbound previous HWND),
rechecking the binding generation under the existing SRW lock to avoid
retiring a newly rebound owner, and publishes `workerFinished` on every
exit path. Existing policy, DLL ownership and frame-pacing semantics remain
unchanged. A synthetic hidden-window fixture covers HWND destruction while
the original owner thread remains alive, followed by a successful rebind.

The fixture is authored but not executed. This correction prevents one
identifiable stale HUD-worker path, but does not prove that it caused the MC02
stuck process; process runtime/Windows/Minecraft remain NOT_EXECUTED.

## P0 user-facing recovery — no stale embedded-game claim (27 October 2026)

The GUI now listens to the already-existing product-close/close-error Tauri
events (without adding a new IPC command or game/session authority). During a
normal exit request it suspends UI inspection, launcher actions, navigation and
automatic native workspace reconciliation. On an incomplete exit, it clears
the stale embedded-window selection and queued launch/switch state, returns to
the Play page in separated-session mode and explicitly inspects the remaining
sessions without automatically recapturing them. A compact recovery panel shows
plain-language guidance and leaves diagnostic details expandable. The titlebar
uses short error text instead of duplicating a raw technical trace. A launcher
that has an error while still NOT_RESPONDING no longer leaves the pending Start
header indefinitely, and the card does not depict a new Launch.

This is bounded UX recovery after an acknowledged failed close, not a claim that
a surviving game has a valid HWND or that shutdown now succeeds. Existing
controller, profiles, game windows, and account/world data remain unchanged.
Svelte/Rust/Windows/game checks were NOT_EXECUTED; GitHub static review only.

## P0 source correction — complete recovery helper response ordering (27 October 2026)

The workspace's existing per-window recovery helper still uses bounded
commands, its own checkpoint and a reader-thread fallback that attempts
identity-checked native restore if the helper exits unexpectedly. Killing the
*helper* alone on timeout is not a Minecraft force-kill. A source inspection
found the stop path consumed exactly one remaining response while waiting for
its final reader End event: a late previous Placed/Restored/Error response
could be mistaken for incomplete recovery even if the reader later confirmed
actual restoration. The existing helper owner now drains earlier finite
responses within the same three-second deadline and only accepts its final
End(Result) as restoration evidence. No new helper, worker, process ownership
or timeout extension is introduced. Synthetic in-memory event-ordering tests
were authored, not executed. All real Windows/Minecraft recovery results
remain NOT_EXECUTED.

## P0 user-facing Exit confirmation — 27 October 2026

The Tauri CloseRequested owner now checks whether a retained controller
mutex or native workspace is present, and asks for explicit confirmation
before altering presentation or requesting CloseAll. The single native
boundary covers the titlebar X and Windows close shortcuts, not just a
Svelte click. Cancel returns without releasing/restoring a game window or
signaling Minecraft; the already-present concurrency gate resets. With no
controller/workspace, the manager closes without an unnecessary prompt.
The titlebar no longer sets its Closing state until it receives the
confirmed `lazy-tester-closing` event. rfd 0.17.2's OkCancel result
is compared explicitly against Ok; all other results preserve games.
These are source changes only, not a runtime UI acceptance claim.

## P0 selected normal close — retained five-slot session actor (27 October 2026)

A typed CloseOne {slot} (fixed slot 1..5) now routes through the same retained
Rust controller as CloseAll, sharing its bounded normal WM_CLOSE/exact dialog
and original game process-HANDLE exit proof. A one-slot selection mask excludes
all other sessions from close commands, timeout targets, and completion criteria;
after the request completes it restores the manager's nonterminal admission
state, unlike full CloseAll. An absent controller never launches a replacement
for CloseOne and no PID/HWND selection is accepted from the UI.

One Tauri action serializes inspection, release of only the native workspace
lease already attributed to the selected LAZY_TESTER_MANAGED_SESSION slot,
and CloseOne through the existing manager resource lock. Other game leases
are not released or closed; the preview compositor is cleared for refresh.
A failed release prevents a close request rather than leaving a partially
restored target. Fixed-mask/unknown-field/out-of-range Rust fixtures are
authored but not executed. This source change is not proof of MC02 exit.
All local PC tests and Minecraft acceptance remain prohibited pending
explicit operator authorization.

## P0 per-launcher close UI and safe retry status — 27 October 2026

Play now exposes a separate Close game control under each existing active
Minecraft card, not a nested button and not the global application Exit.
A one-slot review explicitly describes the target and preservation of peers;
Cancel issues no IPC. Confirmation calls the dedicated `lazy_tester_close_one`
Tauri action, which reuses the existing session actor and selective native
window lease release. A successful reply refreshes the canonical five-slot
snapshot, preserves unrelated sessions and changes Play selection only if
the closed slot was the selected one. A failed close reconciles the snapshot
and does not falsely claim the target still appears inside the viewport.
Unresponsive/starting slots may request normal close, but no process is ever
force-killed. Frontend slot validation and a negative regression fixture
were authored; no source runner, Windows or Minecraft test was executed.

## P0 close-one outcome safety — 27 October 2026

The dedicated `lazy_tester_close_one` command now emits a strict bounded
`SelectedCloseReport`: a preflight or native-restore error is still an
ordinary Err, while a subsequent session-close error returns a typed receipt
with `presentationReleased=true`, null snapshot and an issue. The Svelte
bridge strictly parses this receipt and rejects contradictory or forged
states. The frontend only clears its embedded active-window claim after the
backend has acknowledged restoration. A pre-release failure preserves
the original Play selection instead of inventing a detached window.
A post-release normal-exit timeout preserves games and reconciles status
through read-only Inspect. This corrects an identified misleading-UI path
without a new native lifecycle authority. Tests are authored but not
executed; runtime success remains NOT_EXECUTED.

## P0 preserved-peer presentation after selected close — 27 October 2026

When the currently embedded launcher receives an acknowledged native
presentation-release receipt, Play now selects another verified responsive
peer from the existing controller snapshot. This avoids leaving the other
session's still-retained parked native game window inaccessible after one
selected Minecraft exits. When selected exit remains uncertain, automatic
selection explicitly excludes the pending target; a different responding
peer may resume. With no verified responsive peer, Play remains in
separate-window/recovery mode, never claims to have a playable embedded
session, and does not force-kill or silently capture the unresolved target.
An unacknowledged pre-release error still preserves the original view.
Only the Svelte selection policy changed, not the game actor or ownership.

## P0 no false shutdown result after losing the controller (27 October 2026)

The previous transport returned five STOPPED profiles when CloseAll or
CloseOne found no session pipe/controller mutex, even though Minecraft
may survive its original controller. Source now inspects registered-game
process candidates through the existing read-only Windows Toolhelp
collector before returning a stopped result. Windowless game processes
are included. A nonzero count refuses success without adopting, killing
or mutating anyone's Minecraft session. An inspection failure also
fails closed. No candidates retains the existing idle success path.
Other unrelated registered games can conservatively block this Exit;
this limitation is explicitly retained rather than pretending they
are owned. Unit-source fixture covers zero, one and five candidates.
No Windows/Minecraft runtime tests were run; P0 remains unaccepted.

## State and operator request — 27 October 2026

The v0.1 checkpoint was saved and paused in Codex. The operator subsequently
resumed source development on GitHub Local, with the first priority explicitly
limited to diagnosing and correcting the MC02 normal-exit failure. The v0.1 tag
and backup branch remain untouched. Do not resume past runtime tests or close
existing processes from historical Next entries. This is still an incomplete
development checkpoint, not a release or a usable-product claim.
main remains untouched; GitHub Actions OFF; no release/installer publication.

Tag v0.1 points to this handoff; preceding implementation:
a788a06a9d9a7d4e944fc3b04505b185446ac6e3.
Do not repeat the same failed two-launcher test.
Diagnose the code before another runtime repetition.

## Priority and full scope retained

P0: one normal Lazy Tester exit must close every original managed Minecraft session.
Observed failure: with two launched sessions, MC01 exits but MC02 loses its main
window and stays alive; manager/controller consequently remain open. Reproduced
before and AFTER the HUD lifetime change. Root cause remains UNKNOWN. At v0.1, the normal-close loop sent WM_CLOSE
only once per slot even if its verified main HWND changed, and a still-live
game without a unique HWND was represented as STARTING. The post-checkpoint
source correction tracks verified HWNDs per session, permits a bounded
WM_CLOSE request only for a newly verified window, and records the original
window thread's signaled/alive state together with whether a dialog was
submitted. Normal-close timeout evidence is attached to the affected session
for subsequent Inspect, which now reports NOT_RESPONDING instead of STARTING: the UI treats a
still-running game without a usable HWND as non-launchable and counts it as
an active process, without changing the existing session wire schema. A
frontend contract regression was authored for this case.
No force termination, account/world reads, new provider or background probe
is introduced. This is a diagnostic/close-retry correction, not proof that
the original MC02 hang is fixed; Windows and Minecraft tests are NOT_EXECUTED.

Other outstanding requirements:
- Active game should promptly reach the saved FPS limit; background games use the
  background limit. Defaults 120/5, new saves whole 5..1000, background <= active;
  preserve existing explicit settings. Actual focus-to-FPS behavior unverified.
- No blackscreen or loss of click input on selection; stable native geometry while
  dragging/resizing; shared Play/Settings frame; comfortable 16:9 main display.
- Persistent, useful background previews; smooth switching and efficient launch.
- Clear Task Manager roles/grouping for UI, controller, recovery, Minecraft and
  WebView2. Child process parentage and names do not prove one Windows group.
- Functional General/Data Storage: High/Low graphics, FPS, per-session audio,
  cache/reset/restore and confirmed per-launcher full local profile clear.
- Keep root EXE/shortcuts and project-local Data; preserve installation, peers,
  identity pins, file-backed hives, backups, submodule and second worktree.

Do not bypass sign-in/licensing, patch official game binaries, read account/world
contents, delete runtime data, revive VM/Sandboxie routes or resume RAM experiments.
Compiler caches were explicitly retained. No incidental cleanup is authorized.
Four independently licensed playable clients and gameplay/network acceptance are
separate obligations, NOT_EXECUTED here.

## Installed artifacts and latest actual failure

Installed App/root receipt source:
a788a06a9d9a7d4e944fc3b04505b185446ac6e3.
App/lazy-tester.exe SHA256:
decfba747d812bdcf2dce49ec35e0f724886467752c89d8e35f82245ff5ad1e2.
Native bootstrap-compat.dll SHA256:
e90b312b3f12915098a3571a647050d5e3a1d082a0f831242d92ffac5514653f.
Native bootstrap-enroll.dll SHA256:
51517528886483ba89f1b93b448c1ddd349f26f0e48e5322db434cbfe9c4de99.
App/build-info.json owns local root starter/helper/MUI receipts.

Latest local package:
workspace/packages/lazy-tester/0.1.0-a788a06a9d9a-20261027T051600372Z.
Installer SHA256:
13ddb8919be3053fe83b21d8d69d36014b46a27875dd7f8eb4a697abcba311c1.
NSIS construction and App staging succeeded. Signing NOT_VERIFIED; installer
install/repair/uninstall NOT_EXECUTED. No installer is published by this tag.

Latest actual two-session run on a788a06:
- Explorer startup: connected, five enabled launch controls, WebView cache in Data.
- MC01 visible-window observation ~7774 ms; MC02 ~5470 ms. This is not gameplay proof.
- One full-exit request timed out after the observer's 35-second bound.
- MC01 exited; MC02 and manager/controller remained. HUD change did NOT resolve it.
- Last metadata check: UI PID 53364, controller PID 47048, MC02 PID 53992.
  These are historical observations, not authority for a future PID-based action.
- No further input/exit testing after user requested source focus.
- Previous runs also had intermittent covered game/black screen and inaccessible
  input. Six-switch/drag geometry checks passed on some runs, but pixels/menu clicks,
  pacing and stable play are NOT_ACCEPTED.

Old test games were force-stopped after approval; an old manager was also force-stopped. No approval carries to current/reused PIDs. Current sessions remain running.

## Source owners, changes and proof limits

app_bootstrap.rs prevents close, releases/restores surfaces via workspace_surface.rs, then sends CloseAll. Failure keeps UI open.

independent_bootstrap/session_controller.rs retains process/birth pins, verifies normal WM_CLOSE and exact English exit dialog, waits 20s for startup/exit, reaps process exit and tolerates identity races. No automatic force kill. Real MC02 still FAILED; missing HWND misleadingly maps to STARTING.

Key recent commits:
- 2685a08 / 79af35e: owned exit-dialog checks survive missing main HWND.
- 64f2af6: wait for starting game HWND within existing close deadline.
- e735b7e: tolerate game exit racing close checks.
- d39544a: bounded retries of unstable foreground sampling.
- 57b6842 / ba40d73: HUD worker lifetime and owner-thread validation.
- a788a06: pin rebuilt HUD native payload; packaging digest fixture updated.

Native HUD owner: Development/tooling/acceptance/windows/bootstrap-ordering/fps-hud.cpp pins owner thread, exits worker when it ends and reuses/restarts it on bind. hud-lifetime-control.cpp (main thread ExitThread) hung before and exits after; hidden HUD self-test passed eight reuse cycles. Actual MC02 still fails, so causation is NOT_PROVEN; review owner-thread replacement races.

FPS: instance_focus_events.rs publishes identity-bound events; frame-pacing.h consumes limits. Synthetic promotion ~0.06ms and policy fixtures passed; real focus/FPS unproven. Changes apply next launch. Preview geometry/recovery changes passed synthetic checks; blackscreen/input remain unresolved.

Process roles: process_roles.rs and windows/process-roles.json use pinned hard links
and MUI descriptions. WebView/controller parentage observed; Task Manager one-group
result not achieved. Official Minecraft binary identity is preserved.

Settings/Data Storage: queued saves, stable review dialogs, admission checks. 39 frontend tests, typecheck and synthetic storage/reset/clear/audio fixtures passed at their implementation revisions. No real profile deletion/reset. Full clear excludes Windows identity services; fresh-account sign-in unverified. Audio crash recovery and Minecraft mute/volume unverified.

## Continuation, build and preservation

Next: continue source-only review for any unhandled close-on-error window
identity and normal-close UX gaps, then stop with an exact-head static handoff.
No local PC testing or Windows/Minecraft runtime execution until authorized.
No request to use a Local PC, no Windows or Minecraft execution and no
installer publication until the operator explicitly authorizes them.
The detailed game process/thread/dialog evidence remains an unexecuted
acceptance obligation, not a reason to stop source work.
Keep every remaining requirement above open until its own actual evidence exists.

Build: source Development/tooling/windows-toolchain/local-env.ps1 and call Set-LocalDeveloperEnvironment -Root <checkout>. Use pinned native CMake/Detours assets. Rust --locked --offline, feature lazy-tester-package; existing GetSecurityInfo declaration warnings remain. Candidate: npm --prefix Development run package:windows -- --export-only, with workspace CARGO_TARGET_DIR and reviewed LAZY_TESTER_NATIVE_PAYLOAD_DIR. Without --export-only it stages and opens startup verification. Source/structure checks: node Development/tooling/repository/verify.mjs --g0 and node Development/tooling/repository/check-structure.mjs.

Ignored local artifacts remain on this PC: App, Data, packages, caches/toolchains,
private recovery originals and workspace/runs logs. They are intentionally not in Git.
Safe observed facts are recorded here because runtime logs are not portable proof.
Second worktree remains detached at 47a45f9c7d4a8ce6799d56997fd38e4b4df49ce7;
submodule at abd4c240031cfb46666daa74ca47ed4e48b561e4. Do not reset either incidentally.
