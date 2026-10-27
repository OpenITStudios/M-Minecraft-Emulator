# Current Development Handoff — v0.1

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

Next: continue source-only audit of native HUD worker lifetime and window
recovery helper ownership; once source corrections are coherent, audit frontend
close-failure recovery against the same single session controller.
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
