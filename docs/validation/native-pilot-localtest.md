---
id: document.validation.native-pilot-localtest
class: DOCUMENT
domain: validation
role: PROCEDURE
authority: WORKING
lifecycle: ACTIVE
---

# Minecraft Instance Manager — Windows local test (native + Sandboxie)

## Acceptance retracted: four independent accounts are not supported by proof

20 October 2026, reviewed `Local` HEAD
`35688ff9cf68debb22f2ed40a075e767deec3497` (implementation `dd1e404`).
The operator rejects the previous completion claim: 02–04 cannot be used as four
simultaneous clients and reuse the same account. **Product acceptance: FAIL.**
The earlier menu/tab observations remain historical bounded observations only;
they establish neither independent accounts nor four concurrent playable clients.

### Capability audit

| Required capability | Actual evidence | Verdict |
| --- | --- | --- |
| Distinct profile names | `minecraft-profiles.mjs` plans MivubiEdu02/03/04; read-only local receipt records profiles 03 and 04 configured. | Source/configuration only; not independent identities. |
| Four simultaneous usable games | Operator reports failure. Individual historical launch receipts for 02, 03 and 04 say RUNNING at separate times; the current process inventory did not contain four games. | FAIL by operator report; receipts cannot be combined into concurrency proof. |
| Stable presentation and control | Actual current workspace Diagnostics says `Workspace placement failed: Workspace foreground activation refused by Windows. Previous session preserved.` | FAIL for the observed presentation; process detection is insufficient. |
| Separate accounts | Operator reports the same account. Each canonical profile plan explicitly returns `accountIsolationVerified:false`. | FAIL by operator report; no independent account acceptance. |
| Persistence and cross-account noninterference | No final-source four-account close/reopen campaign, independent sign-out test, or independently controlled concurrent worlds. | NOT_EXECUTED. |

The account problem belongs first to the isolation/authentication boundary, not
the Svelte tab layout. Current settings intentionally use `NoSecurityIsolation=y`
and `Template=OpenCOM` for login compatibility. The profile source already warns
that `OpenCredentials=n` does not guarantee WAM/SSO isolation. Pinned owned source
`Sandboxie/core/dll/cred.c`, `Cred_CredReadW`, falls back to the host CredReadW
when a boxed item is absent; `Cred_CredEnumerateW` merges host credentials.
`Sandboxie/install/Templates.ini`, Template_OpenCOM, opens host RPC/COM paths.
These are confirmed source paths, **not a trace proving which path Minecraft used
for the reported shared account**. No credential values or tokens were read.

Microsoft documents that [WAM integrates accounts known to Windows and can sign
in silently](https://learn.microsoft.com/en-us/entra/identity-platform/scenario-desktop-acquire-token-wam).
This makes host broker reuse a hypothesis to distinguish from inherited
game-local cache. Separate process/box names and credential-write virtualization
do not prove either path isolated. The historical official-provider two-account
operator observation at a different revision does not establish current owned
engine capability.

### Required engineering boundary before another success claim

1. Identify whether the real game obtains the reused identity from game-local
   file/registry state, classic credential fallback, or the host broker/COM path.
   Use bounded credential-free metadata/probes; no token/account-store dumps.
2. Correct the identified owner, then prove per-profile read/write independence
   with synthetic nonsecret markers under the actual owned engine. A speculative
   OpenCOM toggle, manual sign-out, or deleting caches is not an isolation fix;
   restricting COM previously broke login and must preserve its supported path.
3. Prove launch/presentation independently of authentication: each slot starts
   once, controls recover from failure, and another client's operation cannot
   hide or strand a usable session. Existing global operation serialization
   explains disabled controls while Working; it proves no four-client capability.
4. Only then accept four distinct licensed game accounts concurrently, with
   responsive input and close/reopen/noninterference evidence at one exact SHA.
   Synthetic tests and a configured/readable profile cannot substitute for this.

This audit makes **no authentication fix or capability guarantee**. No login,
logout, cache deletion, profile rewrite, driver/service change or VM operation
was performed. The existing 15 profile-contract tests were executed and pass;
their assertions deliberately keep account isolation false. They are source
evidence, not account/gameplay acceptance. There is no justified one-setting fix
until the actual authentication path is distinguished.

## Client 01 recovery after opening 02 — 20 October 2026

Source `dd1e4047034a2c73e7456b06f6f57746ea285bb8`, baseline `ac68612`.
**Reported irreversible 01 loss: bounded recovery PASS in the final sequence.
General foreground stability and full gameplay acceptance remain unaccepted.**

Two independent defects were reproduced and corrected in the existing owned
provider's canonical PowerShell window/launch owner:

1. The still-running native game had both an `OGLES` surface and a large hidden
   `Chrome_WidgetWin_0`. Size/owner filtering counted both and threw the screenshot's
   `Ambiguous Education top-level windows`. All-client Inspect, launch preflight
   and display revalidation consequently failed; 01 was hidden, not exited.
   Discovery now selects the exact game class within the freshly attributed PID:
   native `OGLES`, or `Sandbox:<verified canonical profile>:OGLES` for owned games.
   Hidden retained tabs remain eligible. Unknown classes return no window; two
   matching game surfaces still fail closed. Class names do not replace existing
   PID/start/session/image/profile ownership validation.
2. The initial real 02 launch reached its menu while the manager remained Working
   after PowerShell exited. With `UseShellExecute=false`, a synthetic long-lived
   GUI inherited the observation pipes: the production policy made collection
   wait 4,138 ms for its four-second lifetime. The same regression passes with
   shell execution, which separates the launched application from those pipes.
   This matches the documented [ProcessStartInfo standard-stream inheritance](https://learn.microsoft.com/en-us/dotnet/api/System.Diagnostics.ProcessStartInfo).

Both regressions failed before correction. Final `npm run verify:source` passes
**712 checks** (Node 282, frontend 177, desktop Rust 76, runtime Rust 177), with
four existing file-symlink skips and eight opt-in desktop tests ignored. The
focused ten-test Windows provider suite and rebuilt Windows debug executable pass.
No frontend or general process-runner changes were needed for this correction.

Final-build actual sequence: adopt existing 01 and 02, display 02, then select 01
and visibly recover its existing world. Next select 02 at its menu, request
**Close normally**, approve its ordinary exit dialog, refresh to observe it empty,
and recover 01. Open 02 once using the final launcher; at the approximately
23-second observation the manager was Connected, 02's menu occupied the viewport,
and controls were enabled. Select tab 01: its same world returned without a
restart, while 02 remained detected. Two game processes remained alive. No input
was sent into the world, no login was automated and no credentials were copied.

The old coverage/foreground error still appeared during manager interaction and
loading; explicit selection/retry recovered it. This change does not claim
continuous 01 visibility throughout launch, reliable mouse/keyboard capture,
distinct licensed accounts, two concurrent playable worlds, four clients or safe
true child hosting. Those remain separate acceptance gaps. The developer manager
is left on the recovered 01 world with 02 available as a retained tab.

## Audit correction: focus failure and recovery hold — 20 October 2026

Baseline `5f8460dacdcfeb849d8611bdcfbbf76c3904d67d`; retained source
`106bd5917f800558cf832b40cc56f03c62ea34d2`. **Primary workspace acceptance
remains FAIL. Recovery hold is a bounded PASS.** This supersedes any reading of
the previous menu observations as general focus/input acceptance.

### Why previous fixes did not close the reported issue

| Finding | Evidence and implication |
| --- | --- |
| Presentation is still two independent top-level windows | `workspace_surface.rs` deliberately avoids parenting/injection. Aligning a game rectangle and suppressing its taskbar entry does not create interactive child hosting. The product's single-shell requirement remains unproven. |
| Foreground handoff remains unreliable | Actual composed native menu → activate manager and Tab reproduced a blank stage. Diagnostics: game foreground false, manager foreground true, center covered by manager true, game visible/enabled true. Game appeared later at its old rectangle. This is a presentation/focus failure, not proof of process exit or a missing installation. |
| Error handling had two different retry policies | `NativeWorkspace.svelte` latched `updateFailed`, while `app_bootstrap.rs` continued forwarding native foreground/resize events to `controller::update`. The backend could show/place the game after the banner changed viewport geometry. Thus the delayed game covered recovery controls. |
| The previous proof was too narrow | Menu pixels, helper readbacks, mocked UI and offline test success did not exercise composed gameplay input/capture. This turn a standalone Tab visibly changed menu focus, but the composed toolwindow could not be targeted by the available UI tool. No composed input or four-client PASS follows. |

The exact cause of the foreground handoff failure remains **UNKNOWN**. Ranked
probes considered pending cross-queue activation, manager focus reclaim, and a
slow game message thread. Microsoft documents asynchronous activation across
input queues and a bounded `WM_NULL` acknowledgement
([primary explanation](https://devblogs.microsoft.com/oldnewthing/20161118-00/?p=94745)).
A two-second acknowledgement experiment passed its synthetic timeout/recovery
test but **still reproduced the real failure**. Both experiment and synthetic
extension were discarded. Increasing a wait is not retained as a solution.

### Retained correction and executed proof

The existing controller now holds failed active presentation using its existing
`suspended` state, asks a live helper to hide it, and preserves the first error.
Later native updates do nothing while held; an explicit, freshly validated
presentation or Restore remains available. Ended helpers retain their existing
standalone checkpoint recovery. A hide failure is appended to the original
error, not reported as success. General hung-window restoration is still unproven.

- `cargo test ... stalled_window_reports -- --ignored --test-threads=1
  --nocapture` failed before the fix: **Failed update was not held for explicit
  recovery**. After the fix it verifies the held state, a harmless follow-up
  update, original style/geometry recovery, first native-stage error and a live
  synthetic client. The complete two-test actual-helper suite passes.
- Final `npm run verify:source`: **711 passed**, four existing file-symlink
  skips, eight opt-in desktop tests ignored by default. Windows debug build and
  formatting pass. No frontend code changed in this continuation.
- Final-build actual native menu → manager Tab reproduced the original coverage
  error, but the game stayed hidden and the complete recovery banner remained
  visible. Clicking **Retry display** returned the native menu to the correct
  viewport without an alert. This proves that recovery sequence only.
- The earlier system-menu/resize probe also left a blank stage without a clearly
  observed error transition; Restore successfully returned the standalone game.
  It is not counted as stable resize or keyboard-capture acceptance.

**Remaining:** true hosting/single-shell behavior, reliable manager/game focus,
composed keyboard/mouse capture and escape, world gameplay, unexplained exits,
general timeout recovery, then two/four licensed playable sessions. No world,
authentication, VM, driver, installer, CI or `main` operation was performed.
The rebuilt developer manager remains open after the successful native-menu retry.

## Foreground-owner correction and recovery UI — 20 October 2026, 22:14–22:51 WIB

Baseline: `6593776cd2ace947f1501c857cb340e1a03e6d21`. Retained implementation:
`9a6999c24b3e7505c4fb3acd5ee9cc3ed45a066a`.
**Native menu presentation: bounded PASS for the final-build observations below.
Overall four-client/gameplay acceptance remains FAIL/unaccepted.** This section
supersedes the earlier binary warning and native-menu observations only; it does
not erase historical hangs, unexplained exits or untested requirements.

The operator explicitly resumed implementation without their participation at
the PC. The running manager was the old experimental executable: Diagnostics
reported `Workspace foreground handoff refused by Windows`, absent from retained
source. Games were restored and the manager closed normally before rebuilding.
No game was force-closed, no world opened and no authentication automated.

### Reproduced defects and correction

- Startup errors persisted globally after successful presentation. A mounted
  regression failed, then passed with presentation errors owned by the workspace
  and related parent errors cleared after confirmed recovery.
- Later native events overwrote the first failure with helper completion. A
  mounted two-error replay failed, then passed after preserving the first error
  until explicit recovery. Stage-only helper progress shares the original
  three-second deadline; it contains no PID, account or window-title data.
- Actual manager → existing Task Manager → manager reproduced a blank stage.
  The preserved first error was `Surface helper failed at place/position:
  Workspace position update: Access is denied. (os error 5)`. Background z-order
  preservation alone removed that error in the observed retry but still failed
  coverage. Asynchronous promotion from the manager also failed and was discarded.
- The retained helper owns geometry/recovery with `SWP_NOZORDER`. The manager
  calls `SetForegroundWindow` only when its own workspace is foreground; an
  unrelated foreground application is left alone. Native identity and modal
  guards remain, with a bounded 500 ms coverage observation. No parenting,
  topmost flag, input-queue attachment or foreground-lock override. Obsolete
  focus parameters were removed from the private same-executable protocol;
  public Tauri schemas are unchanged.
- **Retry display** works after initial admission or active update failure and
  revalidates through the existing provider. Failed replacement does not commit
  selection. Tab/card guards agree on stale, pending, ambiguous and unresponsive
  observations; keyboard navigation skips disabled tabs. The engine badge says
  Connected. Consolidated viewport sizing, readable labels and bounded recovery
  banners replace conflicting layout overrides. The rail is controls, not previews.

### Executed evidence

| Check | Result | Boundary |
| --- | --- | --- |
| Final `npm run verify:source` | PASS | 711: Node 281, frontend 177, desktop Rust 76, runtime Rust 177; four existing file-symlink skips; eight opt-in desktop tests ignored by default. Typecheck zero errors/warnings, frontend build, Rust checks and formatting pass. |
| Windows debug executable | PASS | Rebuilt from retained implementation after final frontend build. Abandoned experiments are absent; no installer constructed. |
| Actual-helper synthetic suite | PASS | Two opt-in Windows tests run separately: normal/maximized release, pipe loss, helper crash, retained tabs/minimize/modal/stale identity cases; a four-second synthetic position stall reports `place/position`, restores original state and leaves the synthetic client alive. Diagnostic regression failed on the original generic timeout, then passed. |
| Mounted UI | PASS | Startup/initial/active retry, first-error preservation, failed suspend/release, retired target reconciliation, tab/card parity and disabled-tab keyboard navigation; 1366×900, 1180×760, 920×620 without page overflow. Mocked bridge, no game rendering proof. |
| Final-build real native menu | PASS, bounded | Startup visibly renders the menu; manager → existing Task Manager → manager returns to a rendered menu; one Settings round-trip and one minimize/restore round-trip return to the menu without an alert. Earlier candidate repetitions are supporting observations, not extra final-SHA cycles. |
| First retained full source attempt | FAIL, rerun PASS | Existing private-engine assembler fixture hit EPERM on temporary payload rename. Isolated and complete reruns pass without assembler changes. File-lock cause UNKNOWN. |
| Wider runtime | NOT_EXECUTED | Direct world/input, mouse capture, authentication, four-client campaign, multiplayer, endurance, FPS, mixed-DPI/Explorer restart and standalone installer acceptance. A sent Alt-Tab chord is not a separately isolated Alt-Tab campaign. |

The Vite development server also logged ResizeObserver undelivered-notification
warnings during repeated mocked viewport/error transitions. The final mounted
harness completed without a page-error verdict; this does not establish that
the warning is absent in every runtime. No speculative observer change retained.

The final retained debug manager remains open with the native menu displayed;
no loopback CDP listener was enabled. Historical timeout causes outside this
reproduced placement/foreground failure remain UNKNOWN. Synchronous fallback
restoration and simultaneous manager/helper failure still lack complete bounded
or durable recovery proof. The game remains an independent top-level window,
not true child embedding. Prove native direct input/recovery before four-slot
escalation. `main`, Actions, drivers, services and profiles are unchanged.

Local supporting logs are ignored: `workspace/workspace-*-red.log`,
`workspace/workspace-recovery-ui-final.log`,
`workspace/workspace-recovery-retained-source-rerun.log`,
`workspace/workspace-recovery-retained-build.log`,
`workspace/workspace-recovery-retained-synthetic.log`. Sanitized observations above
are portable; no account-bearing screenshots or raw machine identity enter Git.

## Full check and cross-chat handoff — 20 October 2026

Baseline: `cf3089c1e0ff8893b9939bdb5062e3facab7b2fd`. Retained source/test
corrections: `cd382b4353b2c53761927551d91c71d8e09e62a1`.
**Overall acceptance: FAIL. The manager is not yet reliably usable.** This section
supersedes earlier optimistic UI observations for current acceptance. The user
requested recording unresolved work before moving to a new chat; do not resume
interactive tests merely from this historical handoff.

### Required experience and architecture boundary

One automatically configured English manager, four sessions (01 native, 02–04
owned), browser-like switching, one manager taskbar entry, responsive rendering
and input, safe Alt-Tab/minimize/restore/Settings navigation. No executable or
engine folder chooser in the normal flow. Accounts remain manually signed in,
separate and licensed. The operator accepts normal Windows APIs and windows but
does not want Windows Sandbox or an external sandbox application dependency.
Current presentation is native top-level window composition, not true child
embedding. Previous SetParent experimentation was rejected because parent failure
could destroy the game HWND. Do not conceal this architectural limitation or
claim that window flags prove a working embedded renderer.

### Unresolved issues, evidence and continuation

| Priority | Issue | Actual evidence / next investigation |
| --- | --- | --- |
| P1 | Alt-Tab recovery fails | Actual manager → existing Task Manager → manager sequence produced `Surface helper timed out`; blank play area. UI automation activation also changed window geometry, so pure Alt-Tab versus restore/resize is not isolated. Trace helper command/reply and native call duration before changing policy. Never kill Minecraft to recover. |
| P1 | Returning from Settings can leave game behind manager | Actual error: `Workspace game is covered; presentation was not confirmed`; sampled game foreground false, manager foreground true, center covered by manager true, visible/enabled true. The asynchronous foreground-race fix below does not resolve this stable-foreground failure. |
| P1 | Process detected is incorrectly easy to interpret as usable | Native process remained responsive, but blank workspace persisted. Running/process identity, WM_NULL, rectangle and visibility readbacks do not prove displayed frames or keyboard/mouse input. Keep rendering/input acceptance failed/unexecuted until directly established. |
| P1 | Reported Minecraft self-exits / four-client unresponsiveness | User previously reported game exit and all four nonresponsive. At full-check start only native process was alive; previously displayed owned 02 was absent. No matching Application 1000/1001/1002 record was found in the queried hour. Exit cause UNKNOWN, not a clean-exit conclusion. Collect sanitized termination/helper timing evidence. |
| P1 | Four tabs and one taskbar entry not accepted | Earlier shell flag readbacks are supporting evidence only. Stable switching for 02–04, actual visual taskbar/Alt-Tab behavior, Explorer restart, minimize/restore, DPI and monitor changes remain unaccepted. Do not hide windows merely to make taskbar screenshots pass. |
| P2 | Recovery may block on synchronous native calls | Helper reply has a three-second deadline and helper-only termination, but manager-side checkpoint restoration still makes synchronous Win32 calls. A hung game UI may block recovery; this is a source risk, not the demonstrated timeout root cause. Investigate bounded restoration in the existing owner. |
| P2 | Checkpoint durability gap | Saved window state is in memory. Simultaneous manager/helper failure has no proven durable recovery. Preserve original styles/placement and process identity; do not introduce another lifecycle manager. |
| P2 | Error/status UX can remain stale | A root startup alert was observed persisting after manual window restoration. Global startup errors and local presentation errors need consistent clearing after confirmed recovery. No background game-exit notification is established; explicit fresh Check now reconciles retired targets. |
| P2 | Tab/card operation guards differ | Static review: tab activation does not share every card pending/current-observation guard. Backend ownership/locking checks remain, but avoidable denied operations are possible. Reproduce before changing shared command semantics. |
| Separate scope | Engine independence incomplete | Developer test fork still borrows SbieDrv/SbieSvc identities. Namespace separation, independent installer/uninstall and clean Windows without official Sandboxie remain unproved. Unsigned production payload stays NOT_INSTALLABLE. No driver/service/security changes were performed in this full check. |
| Separate acceptance | Gameplay/performance | Four distinct licensed playable sessions, account separation, multiplayer native↔owned and owned↔owned, staggered restarts, 60-minute endurance and displayed 60 FPS remain NOT_EXECUTED for this revision. No official background FPS cap established. |

### Retained fixes and verification

Four reproduced issues were corrected in the existing desktop owner, with
regressions: stale native-resize geometry now waits for authoritative webview
geometry (explicit invalid geometry still fails); Escape from profile setup
resumes the suspended game; fresh Inspect clears an exited selected target only
after safe release (failed release retains recovery controls); a foreground switch
during placement no longer falsely applies foreground-only coverage rejection.
Each targeted regression failed before its correction and passed afterwards.

| Check | Result | Scope / limitation |
| --- | --- | --- |
| Final `npm run verify:source` after removing experiment | PASS | 709 tests: Node 281, frontend 175, desktop Rust 76, runtime Rust 177; four existing symlink skips, seven opt-in desktop tests ignored by default. Log `workspace/full-check-confirmed-source.log`; source committed as cd382b4. |
| Svelte/TypeScript and Windows debug build | PASS | Zero checking errors/warnings; debug build before foreground experiment passed. Current on-disk executable may still be the experimental build; rebuild before next UI test. No installer constructed. |
| Mounted workspace UI regression | PASS | Three viewport sizes, setup Escape, fresh exited-target reconciliation and failed-release recovery. Mocked UI is not Minecraft rendering proof. |
| Real-helper Windows synthetic regression | PASS | Stale geometry red/green plus normal/maximized release, pipe-loss and helper-crash recovery. Synthetic fixture is not game compatibility proof. |
| Four other opt-in Windows checks | PASS | Owned three-slot read-only inspection; automatic setup validation/preferences; registered native launcher matches chosen installation; staged 25-file package remains not installable. Owned-game launch/focus opt-ins NOT_EXECUTED in this batch. |
| Staged package first invocation | FAIL, corrected invocation PASS | First check incorrectly pointed to payload subfolder; corrected package root passed. Invocation error, not evidence of a defective payload. |
| Dependency audit / provenance | PASS with warnings | npm audit zero vulnerabilities; Cargo audit retains known cross-platform glib 0.18.5 RUSTSEC-2024-0429 and proc-macro-error 1.0.4 RUSTSEC-2024-0370 warnings; both absent from explicit Windows target graph. No dependency changes. npm outdated remains informational; latest-everything is not claimed. Isolation source verifier passed. |
| Clean npm installation | NOT_EXECUTED | Existing locked installation used; no npm ci in this batch. |
| Actual app switching / Settings return | FAIL | Residual P1 failures above; source test success does not override them. |
| Current revision world/input/taskbar acceptance | NOT_EXECUTED | Native menu was incidentally visible; no account details retained. No gameplay input or login automated. |

An exact-child `AllowSetForegroundWindow` experiment compiled and passed synthetic
checks but did not demonstrate a live improvement. It was removed from committed
source; no focus-lock/security bypass retained. The running/on-disk experimental
binary must not be mistaken for cd382b4. The last attempted UI observation was
stopped by the operator's physical Escape key. No further Computer Use was
performed, and no final window/service cleanup is claimed. Reinspect live state
read-only at the next explicitly resumed session; use normal restoration/close,
never force-close games or delete profiles/worlds.

Local raw logs are under ignored `workspace/full-check-*.log` and may not travel
with a new checkout. This sanitized report and Git source/tests are the portable
handoff. First continuation: reproduce the native helper timeout and identify the
first blocking native call using bounded diagnostics in
`apps/desktop/src-tauri/src/engine/workspace_surface.rs`; its UI consumers are
`NativeWorkspace.svelte`, `OwnedManagerPanel.svelte`, and `UnifiedManagerApp.svelte`.
Do not ask the user to validate tiny speculative patches. Rebuild the retained
source and run meaningful manager navigation/recovery checks before requesting
manual licensed gameplay acceptance. `main`, Actions, accounts and game data stay
unchanged.


## Hidden viewport failure isolated — 20 October 2026, 21:22–21:35 WIB

Baseline `a916e3e587e4d12f6fdcd45d2ecfe552f3a22926`; product correction
`e26500c`, lifetime diagnostic/test integration `dcf779e`.
The operator approved the focused stability design. **Overall gameplay acceptance
remains FAIL/unaccepted**; this batch fixes a reproduced Settings bug, not every
reported blank-frame, focus or process-exit failure.

The new mounted regression reproduced **FAIL**: opening Settings hid the native
viewport, but ResizeObserver queued zero-sized geometry. When busy cleared, that
update generated a false error and latched off later resize updates. The desktop
now suspends observation before requesting native suspension, discards pending
hidden geometry, and resumes after successful presentation. Failed suspension
restores the previous observation state. No duplicate readiness/provider store,
automatic game retry, foreground-lock bypass or force-close was added.

| Verification | Result | Scope |
| --- | --- | --- |
| Hidden Settings geometry | PASS after reproduced FAIL | Mounted mocked UI; explicitly checks hidden-stage errors rather than visible alerts only. Failed suspend also retains the client page and permits subsequent resize updates. |
| Actual manager Settings | PASS | New Windows build: three native Settings suspend/resume cycles with hidden-stage error checks, same process identity and bounded window response retained. No rendered input/world proof. |
| Status contract | PASS after reproduced FAIL | An observed HWND is labelled `Window detected`; absent/nonresponsive observations cannot be labelled playable/running. Native and owned Play buttons refuse a nonresponsive observation. No `Ready to play` state is inferred. |
| Full source | PASS | 708 passed: Node 281, frontend 175, desktop Rust 75, runtime Rust 177; four existing symlink skips, seven opt-in Rust tests not run by default. Typecheck 0 errors/0 warnings, production frontend build, Rust checks and formatting pass. |
| Windows debug build | PASS | MSVC executable rebuilt with Settings fix and contextual occlusion diagnostics. No installer/driver/signing operations. |
| Mounted error layouts | PASS | 1366×900, 1180×760 and 920×620; preserves recovery controls. |
| Process lifetime collector | PASS | Exact creation time plus retained process handle; Windows fixture rejects stale identity, distinguishes timeout from exit, and preserves known exit code 17. JSON omits PID/path/title/account data. |
| Native lifetime observation | PASS, bounded | 60 seconds, original process did not exit. This is not a responsiveness/rendering claim. |
| Owned 02 lifetime observation | PASS, bounded | Two distinct 60-second intervals covering normal restoration/close/reopening (first includes rebuild/debug reopen; second normal reopen without CDP). Original owned process did not exit in either interval. These are not a continuous soak; earlier four-client exits remain UNKNOWN. |
| Two-client selection campaign | PASS, bounded | Twelve selections retain both PID/birth/session/HWND identities and response; median acknowledgement 3123 ms, max 3316 ms. Switching delay remains a UX limitation. |
| Normal restart shell readback | PASS, bounded | Two original game processes/root windows, two TOOLWINDOW, zero APPWINDOW, exactly one visible after normal manager restart. Loopback debug listener removed. Visual taskbar/Alt-Tab proof is not inferred. |
| Rendered input, actual Alt-Tab, Explorer restart, four playable clients, multiplayer/soak/FPS | NOT_EXECUTED | These unresolved acceptance requirements are not replaced by source/WM_NULL/style checks. |

One earlier startup on this baseline was rejected by the foreground-center
occlusion check, leaving a retained process but no selected viewport. A later
startup succeeded; the transient cause remains **UNKNOWN**. Contextual diagnostics
now distinguish focus requested, current game/manager foreground, manager coverage,
visibility and disabled state, without recording external window titles or IDs.
No speculative permission or z-order workaround was enabled. Likewise, no exit
was observed in the two bounded process-handle observations; that does not explain
the earlier closures, identify who closed a game, or prove a long soak.

The existing read-only `watch-workspace-process.ps1` collector is documented in
the Windows tooling guide. It is optional acceptance instrumentation, not a
product lifecycle monitor or readiness authority. Four-client escalation remains
deferred until visible interactive presentation and focus transitions are proven.

## Four retained tabs and shell eligibility — 20 October 2026, 21:08–21:16 WIB

### Subsequent restart failure and partial-startup correction, 21:17–21:20 WIB

**FAIL: restart acceptance.** After the successful campaigns and normal restoration/
manager restart, only two game processes remained with ordinary separate-window
shell flags; the next provider observation found only the native game. No matching
Application 1000/1001/1002 record was found in the inspected 15-minute interval.
Exit cause and whether any closure was external remain **UNKNOWN**. The earlier
four-window PASS is a point/campaign observation, not stable restart acceptance.
No visual one-taskbar or crash-resolution claim is made.

Source correction `451e1387a05975940158b00bbe3661941c403a49` fixes a separately
reproduced startup blindspot: one failed admission aborted all later sessions.
The mounted mock injected a first-session exit and timed out waiting for the next
tab (**FAIL**); per-session failure handling then admitted the next tab (**PASS**).
That error banner also exposed viewport overflow; flex allocation now preserves
the play area at all three tested sizes, including error state. Typecheck has
0 errors/0 warnings, 175 frontend tests and the rebuilt Windows debug executable
PASS. Rust product code is unchanged from `aada461`; the full 707-test result
above belongs to that prior implementation, not a new full-suite run at `451e138`.
The latest build was started normally with no CDP listener on port 9223. Only a
responsive native process was subsequently observed; automatic real-game adoption,
four clients, rendered input and restart stability remain unaccepted at this revision.
This partial-startup fix does not explain or fix the observed game exits.

Implementation revision: `aada461865317ec8059b3becf4c46b9fcb9357b5`, based on
`1d21cb7b89238e3ac7405d2194ff4a54215e6e89`. This section supersedes the historical
single-lease descriptions below. The collector's delivery follow-up waits for
startup adoption to finish; no product behavior changes after this implementation.
The operator's blank/unresponsive screenshots remain **FAIL** for usable gameplay;
the narrower machine observations below do not overturn them.

The existing desktop presentation owner retains four original checkpoints,
hides background tabs, suspends Settings, restores only the chosen tab for normal
close, and restores all windows on detach/manager exit. Hidden window enumeration
does not interpret `.MainWindowHandle == 0` as game exit. PID, creation FILETIME,
Windows session, HWND and provider/profile checks remain. Shell adoption hides
before changing APPWINDOW/TOOLWINDOW bits; recovery restores the original bits
without overwriting live modal flags. A manager-only foreground event hook queues
coalesced updates, without polling external applications. Foreground placement
also rejects a covered viewport center instead of accepting geometry alone.

| Check | Result | Actual boundary |
| --- | --- | --- |
| Full `npm run verify:source` | PASS | 707 passed: Node 280, frontend 175, desktop Rust 75, runtime Rust 177; 4 existing symlink skips, 7 opt-in desktop tests ignored. Typecheck, frontend build, checks and formatting pass. |
| Intervening full source attempt | FAIL | Windows EPERM renaming a temporary payload in the existing owned assembler fixture; complete rerun PASS without changing that owner. Transient cause UNKNOWN. |
| Windows debug executable build | PASS | Actual MSVC build, including final foreground occlusion check; existing dead-code warnings retained. No installer/signing change. |
| Actual recovery helper with synthetic windows | PASS | Six normal/maximized release/pipe-loss/helper-crash cases, two retained tabs, shell restoration, minimize, Settings, modal flags, stale identity and closed-client replacement. Synthetic windows are not Minecraft. |
| Mocked mounted UI | PASS | Selection, Settings suspend/resume, error recovery and 1366×900 / 1180×760 / 920×620 geometry. |
| Actual native Education baseline and Settings | PASS | 20-second idle identity/response baseline, manager adoption, three suspend/resume cycles and 10-second response check. No sign-in/world/input automation. |
| Actual four-slot manager selection | PASS | Two 12-switch campaigns retained four PID/birth/session/HWND identities and bounded WM_NULL responsiveness. Final campaign includes final occlusion-check build. First measured campaign median 3982 ms, maximum 4091 ms per switch; this remains a UX limitation. |
| Actual shell-style readback | PASS | Four game root windows: TOOLWINDOW=4, APPWINDOW=0, visible=1 after final four-tab campaign. These flags are not visual taskbar acceptance. |
| First final collector startup | FAIL | Inspector collided with startup adoption's existing operation lock. Collector now waits for enabled tabs; repeated final campaign PASS. No retry/lock bypass added to product. |
| Automated foreground transition | NOT_EXECUTED | Windows denied SetForegroundWindow; no focus-lock bypass or simulated Alt-Tab. The foreground watcher is not proof actual Alt-Tab is fixed. |
| Visual one-taskbar button, Explorer restart, game dialogs | NOT_EXECUTED | Shell readback alone cannot prove these behaviors. A brief pre-adoption launch icon is not eliminated/proven absent. |
| Rendered game, mouse/keyboard, four licensed worlds, multiplayer, 60-minute soak, displayed FPS | NOT_EXECUTED | No credentials, game input, private worlds or account state were automated. Previous acceptance FAIL remains unresolved. |

At resumption all four game processes were absent while the old manager retained
a selected tab and helper-ended error. Cause of those exits is **UNKNOWN**.
No event evidence links them to a particular owner; do not claim the recorded
Education hang fixed. Tests above subsequently launched through the real manager
without forced game termination, token copying or profile/data deletion.

Before rebuilding, all four game windows were restored and the manager closed
normally. Developer CDP was loopback-only and process-scoped, never saved in
settings or release config; the manager is restarted normally after collection.
The existing test-signed fork still borrows SbieDrv/SbieSvc registrations;
standalone namespaces/installer and simultaneous manager/helper recovery remain
unproven. Main, CI, driver registrations and security settings were not changed.

Next: reproduce and verify visible interactive presentation and real focus
transitions against this build before declaring one-shell acceptance or advancing
to multiplayer/soak. Avoid incremental manual tests after each code edit; reserve
manual licensed login/gameplay for a consolidated acceptance campaign.

## Browser-like shell target and live window flags — 20 October 2026

Baseline `5ce53a254bd598a76995510f979ecae7c58049a2`. The operator clarifies
the target as one taskbar entry with four game tabs, not hiding game processes
in Task Manager. The product contract now records the current single-lease
limitation and adoption/recovery requirements. One-taskbar/multitab operation
remains **NOT_EXECUTED / not implemented**; no shell-style or ownership hack was
enabled on the game.

The continuing source diagnosis found that placement/restoration replayed the
saved whole window style, overwriting live game-controlled bits such as
`WS_DISABLED`. A synthetic modal-state regression reproduced **FAIL**: placement
re-enabled the main window. The fix changes only the frame mask and preserves
other current bits during placement and restoration. The extended controller/
helper test now **PASS** on two consecutive final runs, including modal-state
preservation, minimize/restore and six normal/maximized recovery cases. Frame and
position errors now include the failed operation in diagnostics. Two intervening
runs failed with generic Access Denied before operation-context diagnostics were
added; that transient cause is **UNKNOWN**, not explained by the diagnostic change.
The prior modal assertion failure was deterministic and is not waived.

This is not evidence that the recorded Minecraft hang is fixed. A real native
baseline, controlled composition, rendering/input, shell eligibility and game
dialog acceptance are still required. No game launch, world access, driver/service
operation, forced game termination or taskbar manipulation occurred in these tests.

## Workspace switching investigation — 20 October 2026

Baseline: `0819b3d17a5bb104fb19d39033e6d6ea33cffbb4`. Operator reports
unstable app switching, disappearing presentation and a crash. Acceptance remains
**FAIL**. Task Manager showing a separate Minecraft process is expected; it is
not evidence of a Windows Sandbox dependency or of a crash. Application/WER
inspection did not provide a usable matching crash record. The precise reported
crash cause remains **UNKNOWN**, not attributed to the following bugs without proof.

**Later operator clarification and matching Windows evidence:** the game closed
itself, rather than remaining alive behind the manager. Read-only Application
event inspection found Event ID **1002** at **20 October 2026, 20:22:23 local**:
`Minecraft.Windows.exe` **1.26.32.0** stopped interacting with Windows and was
closed; `HangType` is `Top level window is idle`. This supersedes the earlier
absence of a matching record. It establishes a game hang/closure, not its cause
or an exception fault module. No contemporaneous dump was found; existing dumps
dated September/10 October are historical and were not used as proof for this run.
Source review finds no game force-termination in the presentation owner (its
kill calls target its own helper or synthetic fixture), but does not exclude
window/style/focus changes triggering a game hang. Real-game acceptance remains
**FAIL**; escalating to two/four clients is deferred. Next runtime investigation
must compare a normal native window baseline with controlled composition,
including foreground loss, minimize/restore and original-style restoration;
preserve any new crash evidence and stop on the first hang. No test may be
declared successful solely because the process/HWND exists.

Correction revision: `f406e7718685ce7582664993414927841a597073`.
Final executed checks: full `npm run verify:source` **PASS**, 705 tests
(Node 279, frontend 175, desktop Rust 74, runtime Rust 177; four existing symlink
skips, seven opt-in Rust tests ignored), Windows debug build **PASS**, opt-in
synthetic helper/minimize/restore regression **PASS**, and persisted mock browser
regression **PASS**. One preliminary overlapping test run encountered Windows
linker LNK1104 on its in-use test executable; the final sequential run exited 0.
These results are not actual Minecraft app-switching or crash-recovery acceptance.

| Investigation | Result / action |
| --- | --- |
| Resize error discards UI selection but retains backend lease | Reproduced FAIL in mounted mocked UI; corrected to retain target and Restore/Close controls, suspend automatic resize updates after failure; regression PASS |
| Webview update while manager minimized | Reproduced FAIL in real controller regression (`Workspace layout changed`); update now checks current native minimized state under the surface lock; hide/restore regression PASS |
| Stale move/minimize/focus snapshots and worker growth | Native events coalesce through one worker with one latest pending window; visibility is read when executed, not captured when queued |
| Silent native event errors | Errors now reach UI via `workspace-surface-error`; main window gets only listen/unlisten permissions; subscription errors are visible; mocked event regression PASS |
| Helper/manager loss, invalid replacement, PID reuse, foreign session | Existing six normal/maximized synthetic recovery scenarios and attribution checks retained; these do not establish real-game acceptance |
| True containment, one taskbar group, Alt-Tab, dialogs, relative mouse, fullscreen, mixed DPI and real Minecraft crash recovery | NOT_EXECUTED / not proven; current independent composition cannot promise true containment or one taskbar entry |
| Direct reparenting / owner reassignment | Not enabled; prior destructive HWND recovery failure remains a rejection. Removing the taskbar icon would not establish safe containment |

The optional persisted browser harness is
`tooling/acceptance/workspace-ui-regression.mjs` (mock bridge, no game control).
It covers selection, settings release, failed replacement, three viewport sizes,
resize failure with retained recovery controls, and native error delivery.
The native regression additionally checks minimize→webview update→restore.
No game, account, driver, service or official Sandboxie registration is changed.

The operator now asks for genuine in-app hosting after composition instability.
This is an unmet architectural requirement. Stabilization corrections here are
not a declaration that the overlay is equivalent to hosting or production-ready.
A replacement must first prove rendering, direct input, dialogs, app switching
and host-failure recovery; capturing video alone is not interactive hosting.
Microsoft documents cross-process DPI risks for
[SetParent](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setparent)
and owner/child destruction behavior in
[window features](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features).
No unsafe parent/owner workaround, input injection or engine fallback was added.

## Approved native workspace integration — 20 October 2026

**Second user acceptance: FAIL; live backend cause reproduced and corrected.**
The operator reported `Game` / `Unknown owned profile`, reproduced by the real
read-only `actual_owned_session_inspects_three_slots` regression. PowerShell
converted the native `$null` argument to an empty C# string, triggering the
owned-profile allowlist. `NativeGame` now supplies null inside C# and shares the
same PID/start/session and two-query attribution checks. Empty/foreign owned
profile names remain rejected; no security predicate was weakened.
After correction, real active-engine inspection **PASS** (three slots; no launch,
create or window operation), eight PowerShell/provider tests **PASS**, and Windows
debug build **PASS**. A regression executes the production native call against
the compiled query type, requiring it to reach DLL loading rather than fail the
profile allowlist. Real-game workspace visibility/input still awaits acceptance;
this backend PASS does not supersede the user's failed UI acceptance.

**Subsequent user acceptance: FAIL.** The operator screenshot showed an empty
play area after Open 01, with the game opening separately and a refresh-required
notice. Source confirmed that the native launch handler stopped after starting
the registered launcher; it never observed the new process or called presentation.
The correction observes up to eight fresh provider snapshots, admits only one
responsive HWND for the requested slot, then presents automatically. Owned launches
also use this bounded observation when their first result precedes a game window.
No automatic relaunch or background polling was introduced. Existing native processes
are inspected and presented rather than launched again. Timeout retains a recovery
message and requires a fresh check before retrying.

Correction checks: Svelte/TypeScript **PASS** (zero errors/warnings), frontend
**175 tests PASS**, frontend build **PASS**, Windows debug build **PASS**. A mounted
browser regression with mocked Tauri commands **PASS**: clicking Open 01 observes
a delayed native window and calls presentation exactly once without refresh.
Two unit regressions cover delayed-window admission and bounded rejection of a
native process for an owned slot. These are source/UI evidence only; corrected
real-game visibility/input remains **NOT_EXECUTED** pending operator acceptance.

**ACTIVE.** The operator approved independent native composition instead of
direct child-window embedding. Implementation revisions: `d3a2a3c` (backend)
and `f5d81be` (mounted UI). The game remains an unowned top-level Windows
window, aligned with the manager play area. This uses ordinary Windows APIs;
Windows Sandbox is not a dependency. The existing owned provider still borrows
the Sandboxie service namespace; standalone installation is not complete.

| Check | Result | Evidence / limit |
| --- | --- | --- |
| Full `npm run verify:source` on the integrated source | PASS | 702 passed: Node 278, frontend 173, desktop Rust 74, runtime Rust 177; four existing Node symlink skips and seven opt-in Rust tests ignored |
| Windows debug executable | PASS | `cargo build --locked`; not a production installer or signing acceptance |
| Actual helper with synthetic normal/maximized windows | PASS | Six scenarios: normal release, manager pipe loss and helper crash for each initial window state; original geometry/style restored, identity preserved; invalid replacement rejected |
| Maximized-window regression during implementation | FAIL then PASS | Strict geometry checking exposed retained old dimensions; restoring from maximized state before applying presentation style fixed the six-case regression |
| Mock browser UI | PASS | 1366x900, 1180x760 and 920x620: layout, selection, restore, Settings release and failure handling; no Minecraft rendering/input claim |
| Helper caller restriction | PASS | Foreign PowerShell caller rejected; helper accepts the same manager executable, plus the debug test harness for regression only |
| Flicker prevention in placement source | PASS, source | Repeated updates do not reapply an unchanged window frame; updates serialize rather than incorrectly reporting concurrent resize as session failure. Helpers use hidden/no-console creation |
| Minecraft visibility, mouse, keyboard, dialogs, minimize and reopen on this layout | NOT_EXECUTED | Earlier user-reported normal gameplay predates this composition and does not establish its acceptance |
| Three live previews, four-client gameplay, multiplayer, 60 minutes, displayed FPS | NOT_EXECUTED | Current rail contains session controls, not live previews |
| Standalone engine services, installer and clean Windows without official Sandboxie | NOT_EXECUTED | Separate namespace/package work remains; no system, driver or account changes made in this integration |

The synthetic fixture may briefly appear while explicit engineering tests run.
It is not part of normal product startup. It is neither Minecraft nor Windows
Sandbox. Reduced redraw is not proof that the reported real-game flicker is fixed.

The existing resource lock and provider remain the lifecycle authority. Presentation
checks PID, birth time, session and HWND before native changes. Recovery saves
style/geometry in memory; simultaneous failure of manager and recovery helper,
mixed-DPI displays and full-screen transitions are not proven. No forced game
termination, token migration or account handling was added. The only dependency
configuration change enables a Windows ToolHelp API feature; versions/locks are
unchanged. `main` and GitHub Actions remain untouched.

## Independent native-surface proposal — 20 October 2026

**ACTIVE; design deviation awaits operator decision.** Source:
`5369417bc015a9e722d7189bd72ca93dd270cc20`. This resolves the feasibility question
for an alternative, not the original embedded-host acceptance requirement.
No live game or mounted product behavior was changed.

| Check | Result | Evidence / limit |
| --- | --- | --- |
| Normal / partial failure / protected UI crash | PASS | Three existing synthetic cases still pass after shared-fixture changes |
| Independent surface survives host crash | PASS, synthetic | Game stand-in remains an unowned top-level HWND; frame/caption temporarily removed and rectangle aligned; after host termination the same process/start/HWND survives, original style/geometry restores and normal close exits 0 |
| Direct-parent recovery comparison | FAIL, retained | Same final compiled fixture still reproduces lost HWND; combined diagnostic command exits nonzero rather than waiving this failure |
| Residual synthetic processes | PASS, observed | Zero after completed comparison; no game, account or service was targeted |
| Compilation / repository contracts / links / whitespace | PASS | Existing Windows x64 compiler and relevant manual source checks |
| Full Svelte/Rust pipeline | NOT_EXECUTED | No product, frontend, Rust or dependency changes; historical pipeline results are not promoted to this revision |
| Minecraft / native input / live previews / dialogs / z-order / DPI / taskbar grouping | NOT_EXECUTED | Separate integration acceptance still required |
| Simultaneous recovery-coordinator failure | NOT_EXECUTED | Test coordinator retains the original window state; do not infer recovery if it is also lost |

The alternative never assigns a manager parent/owner or synthesizes input.
It is window composition rather than embedding. The approved design explicitly
rejects silently substituting placement for embedding, so the proposal remains
opt-in tooling and is not wired into the product. The operator was asked whether
to accept that explicit change. The desired main view/three previews is still a
target, not a claim of implemented UI or a single taskbar entry.

The first alternative harness run could not read ExitCode from a process object
discovered by IPC after it exited. Retaining its OS process handle before exit
corrected the fixture; the final positive and negative results above are from
the corrected comparison. No production process-lifecycle behavior changed.
Run the proposal with `-IndependentSurfaceProposal`; add
`-IncludeUnsafeDirectCrash` to retain the failing direct-parent comparison.

## Workspace crash boundary — 20 October 2026

**ACTIVE, synthetic evidence only.** Source:
`ef250c57dc7db52ada5fd334a14c0f97d359bc68`, following `f91f742`.
No product UI, engine, game, service, profile or account code is changed.

| Check | Result | Actual evidence / limit |
| --- | --- | --- |
| Synthetic normal attach/detach | PASS | Original parent/style/geometry restored; normal close request accepted and child exits 0 |
| Synthetic partial attachment failure | PASS | Same restoration after injected failure; child normally closes with exit 0 |
| Direct manager-parent crash recovery | FAIL, reproduced | Terminating the synthetic manager destroys the client's HWND. Its process may remain alive without a usable window; independent coordinator cannot restore the lost HWND. Optional diagnostic still returns FAIL/nonzero |
| Separate presentation owner; synthetic UI killed | PASS | Fake UI and fake game are sibling children of a surviving owner. Game PID/start/HWND remains valid after UI exit; parent/style/geometry restored and normal close exits 0 |
| Residual synthetic processes | PASS, observed | Zero fixture processes after the completed comparison. Cleanup is restricted to exact synthetic executable/start identity, never real games |
| C# compilation and Windows execution | PASS | Existing Framework64 compiler; normal/partial/protected cases execute; explicit direct-parent comparison executes and keeps its failure verdict |
| Structure / repository contract / whitespace | PASS | Relevant manual checks; no workflow definitions introduced |
| Full frontend/Rust pipeline | NOT_EXECUTED for this revision | No frontend/Rust/package/dependency changes; prior 696-test run remains evidence for its earlier revision only |
| Separate presentation-owner failure | NOT_EXECUTED | The protected case keeps that owner alive; it cannot establish survival if that process also crashes |
| Actual hosted Minecraft / prompts / input / previews / four-client acceptance | NOT_EXECUTED | No game is attached to this experimental arrangement |

The first crash-harness revision also exposed a local fixture error: disposing
the pipe wait handle before EndWaitForConnection caused ObjectDisposedException.
That harness error was corrected before recording the final direct-parent FAIL.
Earlier stranded synthetic fixtures were cleaned up by their exact executable
paths; a thread-based 20-second safety exit now covers a stranded Forms timer.
The optional negative diagnostic remains `-IncludeUnsafeDirectCrash`, explicitly
separate from the three passing synthetic scenarios. It is not a production
fallback or a waived recovery requirement.

**Decision:** reject direct parenting into the manager/UI process. A separate
presentation owner is a narrower promising seam, not an accepted product host.
Resolve its own failure behavior before attaching Minecraft. Keeping a game
process alive without a usable HWND does not satisfy recovery or user comfort.

## Unified workspace foundation — 20 October 2026

**ACTIVE; workspace hosting is not implemented.** The operator resumed work and
confirmed that the previously launched Minecraft was running normally. This is
human-reported runtime evidence for the earlier launch/presentation work at
`a1a147bbc5fd4fc68531a18cf5ee58410281a511`, not four-client or embedded-game proof.
The [approved workspace contract](../product/unified-workspace.md) records one
interactive main view plus three previews, automatic setup and recovery gates.

Source `52c38a3c09c87b9a65d6b2b2d49621cd0d439b9c` removes executable/engine
pickers from the mounted UI. Settings has automatic detection/status and Check
again, routed through the existing authenticated startup preparation. Paths and
canonical routing are read-only under collapsed Diagnostics. Existing backend
validation, account/profile data and service registrations are unchanged.

Source `e80ef10f4046aa60767104d89790b7a02e465bc8` adds only the approved design
and a synthetic cross-process window-hosting probe. It is not a product game
host. Its two independent processes use the same PMv2 DPI context and verify
ownership, parent, style, geometry restoration and normal close. The second
case deliberately fails after attachment and verifies restoration.

| Check | Result | Evidence / limit |
| --- | --- | --- |
| Previous Minecraft visible/normal | PASS, operator report | User: Minecraft could run normally; no new game launched or moved during this foundation work |
| Automatic Settings and retry | PASS, source/browser fixture | No chooser calls in mounted UI; technical paths collapsed; startup uses existing provider; saved paths alone do not establish readiness |
| Browser layout/interaction | PASS, mocked bridge | 1180×760 and 920×620; four cards, no horizontal overflow, visible keyboard focus; compatibility consent and pending-launch navigation guard remain intact; not actual Tauri/game acceptance |
| Synthetic probe first attempt | FAIL → PASS | Initial detach readback occurred before restoring WS_CHILD/style and produced an incorrect failure; readback moved after style restoration, then both modes pass |
| Synthetic cross-process attach/detach | PASS, actual Windows | Parent, original style and screen rectangle restored; child remains alive through restoration and accepts normal close |
| Injected partial attach failure | PASS, actual Windows | Original presentation restored and fixture normally closed after injected exception |
| Source pipeline | PASS | 696 tests: 278 Node + 171 Vitest + 247 Rust; four existing Node symlink skips and six opt-in Rust tests ignored; Svelte 0 errors/0 warnings, production frontend build, Windows Cargo checks and formatting pass |
| New documents and tracked fixture structure | PASS | Repository contract and internal documentation links pass with all new files staged |
| Actual Minecraft hosted inside manager | NOT_EXECUTED | No product hosting module is exposed; synthetic result does not establish graphics or relative mouse support |
| Host crash / mixed DPI / prompts / capture previews | NOT_EXECUTED | Required before native-host promotion; current probe deliberately cannot target external windows |
| Four clients / multiplayer / soak / displayed FPS / independent installer | NOT_EXECUTED | Previous acceptance limits remain |

Only synthetic fixture processes created by this probe were cleaned up after
the failed probe; no Minecraft, unrelated shell, service or account process was
terminated. Evidence stays in ignored local workspace logs/JSON/screenshots;
no raw HWND, PID, account, credentials or private world data is committed.
The main Clients page still uses the existing cards and external game windows.
Next gate: synthetic host-failure recovery before connecting a live game surface.

## Launch failure and flashing PowerShell repair — 20 October 2026

**ACTIVE.** Source `337e137b4c0501139224fef53d95d24472f6d306`, following
`a01a056ca7a8c2785bf2f67eb6b4fb2866bfab78`. The operator reported that Open 02
showed PowerShell briefly but no game. The existing receipt recorded exit 1,
zero attributed games and an empty profile. No stale powershell.exe processes
were present in the initial inventory, so no unrelated process was terminated.

The existing bounded runner now uses CREATE_NO_WINDOW on Windows. Its timeout,
output limits and direct-child-only cleanup remain intact; no job kills game
descendants. The privileged Create helper still uses normal Windows UAC.
Private Start.exe is now started with ProcessStartInfo/UseShellExecute=false,
keeping the verified executable, exact arguments and working directory.
The bounded observation waits for an attributed window (RUNNING), not a PID alone.

| Check | Result | Actual evidence / limit |
| --- | --- | --- |
| Original manager-backend launch repro | FAIL reproduced | Opt-in real launch returns exit 1 and zero games, matching the operator symptom; no automatic retry |
| Console regression from a GUI host | FAIL before → PASS after | Test compiles a Windows GUI host using the actual bounded runner; helper GetConsoleWindow changes from nonzero to zero after CREATE_NO_WINDOW |
| Bounded process tests | PASS | 15 tests cover console suppression, pipes, size bounds, timeout and direct-child reaping |
| Broker communication diagnosis | PASS, read-only | Private DLL GET_USER returns status 0; no account/user-section contents retained |
| Controlled direct-launch experiment | PASS, backend execution | Same opted-in manager backend creates one owned Minecraft with launcher exit 0 after direct process startup; this experiment precedes the subsequent wait-for-window refinement |
| Actual post-launch identity/window | PASS | Signed game, private driver 5.73.6-MIVU/ABI 0x57230, twice-queried MivubiEdu02, stable PID/start time, interactive Windows session, window present and responsive; raw identifiers stay local |
| Residual helpers after game launch | PASS, observed inventory | No powershell.exe or Start.exe remains; Minecraft remains running. No blind global process cleanup performed |
| Window observation regression | PASS | Actual production loop with controlled snapshots does not stop at a game PID before a window appears |
| Complete source pipeline at source SHA above | PASS | 694 tests: 277 Node, 170 Vitest, 247 Rust; four existing Node symlink skips, five opt-in Rust tests ignored by default; Svelte check/build and Windows checks pass |
| First full verification attempt | FAIL → PASS after fixture correction | Existing parallel candidate tests collided on timestamp-only directory names. Test-local atomic suffix fixes the fixture; production integrity verification is unchanged |
| New manager UI Open/normal close/reopen | NOT_EXECUTED after fix | Current game was launched through the real backend regression, not a new operator UI click. Reopen was deliberately not retried while the game is active |
| Manual login/world/input after fix | NOT_EXECUTED | Operator asked to check the already-open game; no result received yet |
| Four clients / multiplayer / soak / FPS / independent installation | NOT_EXECUTED | No change to previous acceptance limits |

The controlled experiment changed direct process startup after console suppression
had also been fixed. The exact contribution of ShellExecute versus inherited
console state is not independently isolated; the observed launch outcome is the
claim, not a guessed upstream failure mechanism. No payload, driver, service,
account or world was replaced. The previous unsigned package and signing status
remain intact. On the running manager, a fresh check should show the existing
Minecraft 02 and offer Focus; do not launch another copy while it remains active.

## Unified manager and automatic setup — 20 October 2026

**ACTIVE; actual manager gameplay remains NOT_EXECUTED after this change.**
Implementation: `1591b6165eb605a2ab064c8f0babd56e328723f1` on Local,
following baseline `89af69acf97f4fa215983206c5b6fba70835f5be`.
The operator requested one coherent English application and then explicitly
requested automatic setup instead of folder selection and repeated checkboxes.

The sole mounted `UnifiedManagerApp.svelte` shows 01 native and owned 02–04.
Legacy Compact Manager/pilot source and all existing profile/account/world data
remain preserved and unmounted. The active UI never falls back to official Sandboxie.
Canonical slot/provider/profile definitions live in
[`minecraft-targets.json`](../../engine/isolation-native/owned-runtime/minecraft-targets.json).
The existing legacy diagnostic catalog projects those identities onto its historical
pilot metadata; that projection is not the unified dashboard's provider selection.

Startup derives one candidate from the currently registered SbieSvc image, bounded
to developer test storage, then runs the existing full owned-provider verification.
An official installation, unknown package or foreign service cannot be auto-adopted.
Only successful automatic verification saves locations in the existing
`%LOCALAPPDATA%\MinecraftInstanceManager\NativePilot\manager-settings.json`.
Saved preferences do not replace live verification. The game resolver checks registry
installation, Microsoft signature and the native launcher's resolved Known Folder path.
There is no directory hunt, service replacement, driver installation or login at startup.

**Current operator path:** open the manager → **Open** on Minecraft 02 → manual
sign-in/world test. No engine picker or licensing checkbox is required on this host.
For a missing owned profile, Open asks compatibility consent once, creates through
the same bounded Create-only helper/readback, then launches only after success.
UAC remains Windows-controlled. Focus/placement/normal close revalidate identity;
no forced game termination, account copying or automatic launch retry is added.

| Verification | Result | Evidence / limit |
| --- | --- | --- |
| Default full source pipeline | PASS | 692 tests: 276 Node, 170 Vitest, 246 Rust; zero failed, four existing Node symlink skips and four opt-in Rust tests ignored by default |
| Real registered game / native launcher identity | PASS | Opt-in Rust test validates actual signed installation and resolved registered Start app target without launch |
| Actual automatic engine preparation and remembered preferences | PASS | Opt-in Rust calls the same prepare owner twice; full owned verification succeeds and saved locations match on reread; no profile/game mutation |
| Actual three-owned-slot inspection | PASS | Opt-in existing Rust test verifies the running private engine and profile 02 settings |
| PowerShell module failure in actual native subprocess | FAIL reproduced → PASS after fix | Security module auto-load failed under inherited environment; resolver explicitly imports trusted system modules; actual Rust target inspection then passes |
| Concurrent external inspection | BLOCKED as designed | First automatic-setup test overlapped the dev app's startup inspection; exclusive operation admission rejected overlap. Idle rerun passes; no automatic launch retry added |
| Frontend checking / production assets / Windows developer executable | PASS | Svelte 0 errors/warnings, Vite production build and actual Tauri debug compilation/startup |
| Layout, keyboard and error presentation | PASS, browser fixture only | 1180×760 and 920×620; four cards, no horizontal overflow; keyboard outline visible; technical error remains collapsed in Diagnostics |
| First-open consent and launch UI | PASS, browser fixture only | Cancel creates nothing; Open issues exactly one request; backend-denied retry stays disabled after check and Settings navigation |
| Actual manager Open 02, focus, place, close and reopen | NOT_EXECUTED | Operator asked to click Open in the automatically prepared app; no new playable-session result received yet |
| Four accounts / multiplayer / 60-minute soak / displayed FPS | NOT_EXECUTED | Earlier one-client CLI world proof does not establish new manager acceptance |
| Independent installer / clean Windows | NOT_EXECUTED | Engine still uses authorized borrowed SbieDrv/SbieSvc registrations; namespace and installer separation remain subsequent work |

The launcher now journals exit code, whether the launcher remains alive, boxed
process count and attributed game count in the existing local receipt. The recent
UTC cooldown bug is fixed; any remaining cold-launch failure must be diagnosed
from a new actual attempt, not hidden by retry. The UI asks Check again and exposes
technical details separately. All identifiers in repository evidence are sanitized;
raw local receipts and screenshots remain ignored workspace artifacts.

Historical observations below keep their original revision/proof scope.

## Four-slot manager integration — 20 October 2026, active continuation

**ACTIVE; four-client acceptance is NOT_EXECUTED.** Implementation source
`fd064eb9ba9a30dec11f54c2134d7b30acae8121` follows the operator's approved
sequence: integrate the actual manager first, validate 01 native plus owned
02–04, then separate the engine namespace/installation and repeat acceptance.
Four different licensed accounts are operator-confirmed available; login stays
manual. This is an explicit **developer mode**, not production activation or
proof of an installation independent of official Sandboxie.

The backend verifies the selected activated session, 25 artifact hashes and
signatures, original source manifest/resources, closed signed-copy inventory,
actual SCM images, private driver/ABI, kernel Test Mode and current Windows
session. The old official pilot remains a separate explicit route; errors do
not fall back to it. The existing catalog/resource lock owns admission; no
second lifecycle service/readiness database is added. Profile settings come
from the existing canonical owner, not a second preset.

| Check / behavior | Result | Revision-scoped evidence |
| --- | --- | --- |
| Full source pipeline | PASS | 683 passed: 274 Node, 166 Vitest, 243 Rust; zero failed; four existing Node symlink skips, two explicit actual-payload/session Rust tests ignored by default |
| Live Rust selected-provider inspection | PASS | Opt-in actual Windows test validates all three owned slots, including adopted Client 02; no create, launch or window mutation executed |
| Svelte/TypeScript / frontend production build | PASS | Zero check errors/warnings; Vite build completed |
| Actual Tauri developer app | PASS, startup only | Real debug build finished and a responsive HWND in the interactive session observed; startup is not button/IPC acceptance |
| System PowerShell module loading | PASS | Actual Rust subprocess initially lacked Get-FileHash; fixed by importing Utility/Security/Management from system PSHOME, disabling inherited module auto-loading; live inspection then passes |
| Profile mutation boundaries | PASS, source/regression | Existing/foreign/partial/orphan profiles refused; disabled first, query after each write, enabled last; Create-only helper in same executable with normal UAC |
| Launch / window boundaries | PASS, source/regression | Nonempty profile/live prior launcher/cooldown prevents duplicate requests; bounded observation with no automatic retry; fresh PID/start/session/HWND required; WM_CLOSE only |
| Actual UI session selection | PASS, operator/screenshot | Selected test engine; four slots shown, 02 STOPPED and 03–04 ABSENT |
| UI launch cooldown | FAIL observed, source fixed | UTC DateTime was parsed to local time before subtraction, incorrectly extending the 30-second guard on UTC+7. DateTimeOffset now compares instants; four focused Windows regressions and live Rust inspection pass. Actual UI relaunch/gameplay remains NOT_EXECUTED after fix |
| Actual creation of profiles 03–04 through UI | NOT_EXECUTED | No new accounts/profiles created by source verification |
| Four distinct playable sessions / multiplayer / reopen | NOT_EXECUTED | Four accounts available, not yet logged in/tested through the new manager |
| Four-client 60-minute soak / displayed FPS | NOT_EXECUTED | No soak or adequate displayed-frame measurement |
| Independent namespaces / installer / clean Windows | NOT_EXECUTED | Intentionally follows manager acceptance; current test still borrows SbieDrv/SbieSvc registrations |

Operator UI sequence: Settings → Select local engine test → select the active
session directory containing session.json and payload → Back to Virtuals.
Confirm the licensing checkbox and explicitly approve the reduced-containment
compatibility profile before creating new virtuals. First open 02 through its
button and verify manual world/input; then add native 01, then owned 03–04,
stopping on the first failure. Verify focus, placement, normal close and reopen
through actual buttons, with world progress saved normally. A native/owned
process observation is not an account/gameplay claim. An inconclusive Open
requires Refresh and an explicit guarded retry; never repeatedly click Open.

Next evidence is the operator's mounted UI result. Preserve the current game
profiles and original service recovery receipts. Do not uninstall official
Sandboxie or switch namespaces while the four-client manager test is pending.
After successful manager acceptance, follow the approved standalone namespace,
rebuild, packaging and fresh-profile/manual-login stage; do not promote this
developer adapter to production simply by changing a readiness flag.

## First playable owned client — 20 October 2026, latest result

**PASS for the first bounded goal: one playable Minecraft Education on the built
owned fork, including its driver.** Source/tooling revision
`3cff60738ecd0cbc19eca95b2c23b13fe5361464`; native binaries remain the real
`f529d59fda778be50cd3e874374a5546c32dce29` build, explicitly test-signed in a
separate local copy. Earlier checkpoint sections below retain historical results,
not the current activation/gameplay status. This is not production signing or
four-client acceptance.

| Evidence | Result | Attribution / limits |
| --- | --- | --- |
| Private driver loaded | PASS | Machine observed 5.73.6-MIVU, ABI 0x57230, actual kernel Test Mode active |
| Owned service namespace | PASS | Guarded ImagePath takeover, verified binary hashes/signatures, stopped/start/readback; no service registrations deleted |
| First unload attempt | FAIL | Official KmdUtil exceeded 60 seconds while SandMan remained open; no ImagePath changed |
| Recovery from pre-redirect failure | PASS | Original driver remained loaded; original user service restarted and original identity verified without repeating the failed unload |
| Explicit activation retry | PASS | Operator exited SandMan normally; second activation unloaded the old driver, started owned services and passed private ABI checks; previous receipt retained in runtimeHistory |
| New MivubiEdu02 profile | PASS | No preexisting name; canonical 18 guarded write/readback steps, disabled first/enabled last; persistent App Compartment/OpenCOM mode explicitly approved |
| Education process attribution | PASS | Microsoft-signed desktop 1.26.32.0; own API twice confirms MivubiEdu02 and matching Windows session; PID/start time stable before/after; local identifiers not published |
| Actual injected hook | PASS | Live Education module list points to the test copy's SbieDll.dll; recorded signed hash matches |
| Window/input/world | PASS, operator reported | Window presence and responsiveness machine observed; operator explicitly reports successfully entering a world and being able to play after manual login |
| Original unsigned payload | PASS | Independent final recheck: zero hash mismatches across all 25 original artifacts |
| Full source verification | PASS | 672 passed: 271 Node, 161 Vitest, 240 Rust; zero failed; four existing Node symlink skips and one default-ignored actual-payload Rust test |
| Runtime regressions | PASS | Six runtime tests cover ownership, busy/unknown inventory, failed-unload recovery, canonical occupied-profile handling, parser validity and PID-reuse rejection before DLL loading |
| Full restoration after an active owned driver | NOT_EXECUTED | Pre-redirect recovery was executed; stopping/reverting the currently active owned driver has not been exercised |
| Two/four clients, account separation, multiplayer, reopen | NOT_EXECUTED | One operator-confirmed playable owned session; no inference about other accounts or peers |
| 60-minute soak / displayed 60 FPS / background cap | NOT_EXECUTED | No duration or FPS acceptance measurement |

The first launch request returned without an immediately observed game process.
A controlled retry, after independently confirming zero boxed/game processes,
returned private launcher exit 0 and produced the attributed responsive window.
The cold-launch discrepancy remains UNKNOWN; repeated first-click reliability
is not accepted or claimed fixed. No game executable, account, token, license
check or driver signing enforcement was patched to obtain this result.

Current Windows state: the owned test driver/service and game remain active,
Test Mode and the local test certificate remain configured. Saved original service
paths/configuration and all local receipts remain available. Do not remove the
test certificate or restore boot settings while its driver is still in use.
For restoration, first save/close the owned game normally; the documented Restore
stage refuses active sandbox processes or ambiguous images. Profiles/worlds are
preserved. The manager UI's owned-provider integration remains unimplemented;
this successful session uses the explicit developer test route.

## Post-reboot runtime preparation — 20 October 2026

Source revision `38b19e603134f2846bca9259fc08036afa7d8cd6` adds the explicit
v1.18.6 test activation/restoration path. It changes only the existing services'
ImagePath through SCM, preserving their registrations, startup types, accounts,
dependencies, security descriptors and minifilter configuration. Authenticated
upstream helpers perform bounded stop/start; original image paths/hashes/signers
are journaled before redirection. This replaces the earlier proposed
delete/reinstall approach with a smaller reversible operation. No legacy Apply
gate or desktop provider is enabled. Commands are in
[Isolation Source Tooling](../../tooling/isolation-source/README.md).

| Check | Result | Actual evidence |
| --- | --- | --- |
| Reboot | PASS | OS boot time is newer than preparation's recorded boot |
| Actual Test Mode | PASS | NtQuerySystemInformation SystemCodeIntegrityInformation reports TESTSIGN active |
| Signed copy integrity | PASS | All 25 recorded SHA-256 values match; driver signature Valid |
| All-session idle inspection | PASS | Verified official SbieDll API queried twice, null box filter/allSessions=true; both counts zero |
| Current driver identity | PASS observation | Official driver 5.73.2, ABI 0x57230; not our private driver |
| Minecraft Education install | PASS observation | Desktop EXE 1.26.32.0, signature Valid, Microsoft Corporation; no game launched |
| Source verification | PASS | verify:source: 669 passed, zero failed; four existing Node symlink skips and one default-ignored actual-payload Rust test |
| New runtime regressions | PASS | Three tests cover parser validity, service-preserving SCM contract, ownership/hash/signer rejection, active/unknown process rejection and all-session probe contract |
| Elevated activation | NOT_EXECUTED | Normal RunAs UAC pending; receipt has no runtime snapshot and servicesChanged=false |
| Own driver load, service restoration, profile and gameplay | NOT_EXECUTED | Existing official SbieDrv/SbieSvc remain running; no takeover has begun |

First next action is resolving the normal UAC request, then observing the actual
private driver load/ABI result. Do not interpret the preparation receipt's old
REBOOT_REQUIRED label as current boot evidence; the fresh read-only probe above
independently confirms the reboot and actual kernel Test Mode. If activation
fails, retain its first error and use only verified transaction ownership for
restoration. Do not proceed to profile/game creation until private identity and
ABI pass. Manual login and playable-world acceptance remain outstanding.

## Local developer activation preparation — 20 October 2026

**ACTIVE, not gameplay-ready.** Source revision
`d99f2a47392bc976de717349a5289a6b30f6e7e7` adds the explicit v1.18.6 developer
preparation procedure described in [Isolation Source Tooling](../../tooling/isolation-source/README.md).
The operator now authorizes local developer test signing on this Windows host,
manual reboot checkpoints and a later controlled, reversible replacement of the
official service namespace. This supersedes the **local testing** production-only
signing restriction below; it does not approve production distribution. The first
goal is one playable owned client, followed by native plus owned; four clients
and the 60-minute performance campaign follow only after these succeed.

The selected new profile may use `NoSecurityIsolation=y` and `Template=OpenCOM`.
This explicit compatibility choice weakens containment and does not establish
WAM/SSO or account separation. Login remains manual with a licensed account.

| Check / operation | Result | Evidence / remaining work |
| --- | --- | --- |
| Branch and starting remote | PASS | Local and origin/Local both `01e21e110defb940dc7c755dad15289b0f043821` before changes |
| Source verification | PASS | `npm run verify:source`: 666 passed (265 Node, 161 Vitest, 240 Rust), zero failed; four existing Node symlink skips and one default-ignored actual-payload test |
| Preparation regressions | PASS | Three tests: complete PowerShell parsing, preparation/activation boundaries, real bounded native execution including spaced arguments, exit 7 and quote refusal |
| Tracked structure | PASS | 3,558 tracked files after implementation commit; no workflow or generated payload added |
| Existing services | PASS observation only | SCM reports official-location SbieSvc and SbieDrv running; this does not prove all sandbox processes idle |
| Security observations | PASS observation only | Current unelevated CIM reports no running HVCI service; registry Secure Boot/HVCI values are zero; firmware, BitLocker and boot preflight still require elevation |
| Elevated preparation | PASS | Normal RunAs retry succeeded at source revision e411fd7380668220e2f220b675528a03c0b065b1; session test-20261020-a |
| Test-copy signing / certificate trust | PASS | 16 copied binaries have Valid signatures; all 25 copied artifact hashes recorded; independent original-artifact recheck has zero mismatches |
| TESTSIGNING / reboot | PASS / NOT_EXECUTED | Explicit EnableTestMode completed with boot readback Yes; receipt REBOOT_REQUIRED_NO_SERVICE_CHANGES; manual reboot outstanding |
| Service takeover / rollback | NOT_EXECUTED | Needs all-session idle proof, full restoration review and the subsequent v1.18.6 activation implementation; preparation exports alone are not an executable service rollback |
| Driver load / ABI / owned profile / Minecraft | NOT_EXECUTED | No owned driver, service replacement, profile creation or game launch |

Actual commands/logs remain in ignored workspace storage. The first normal UAC
request ended canceled without a visible dialog, as reported by the operator.
A second normal RunAs request succeeded; no UAC bypass was used. At source
revision `e411fd7380668220e2f220b675528a03c0b065b1`, elevated preparation and
EnableTestMode both returned PASS. The local receipt is
`workspace/owned-tests/test-20261020-a/session.json`, state
`REBOOT_REQUIRED_NO_SERVICE_CHANGES`. All 16 copied binary signatures are Valid;
independent verification found zero original artifact hash mismatches. Firmware,
HVCI and BitLocker preflight passed without changing their settings. The recorded
boot entry now has TESTSIGNING Yes; no reboot has occurred.

No official service was stopped/replaced, no owned driver was loaded, and no
profile/game was launched. The script supports preparation and pre-takeover
cancellation only; historical v1.18.5 Apply stays sealed. Production signing
remains separate. To cancel before takeover, use the documented elevated
RestoreTestMode stage with this exact session directory, then reboot manually.

Next: operator saves work and manually reboots Windows. Resume by checking the
same local receipt, actual boot state and signature hashes. Complete the reviewed
v1.18.6 service takeover/restoration path, including all-session idle proof and
private ABI checks, before loading the driver or launching owned Minecraft.

## Owned Windows engine campaign — 20 October 2026

**Build/staging COMPLETE; activation, owned operations and four Minecraft NOT_EXECUTED.**
The operator selected owned v1.18.6 first: one native Client 01 and three owned
persistent profiles 02–04. The official two-client desktop pilot below remains
the mounted implementation, not an automatic fallback for this campaign.
No owned provider has been activated.

Executed source revision: `f529d59fda778be50cd3e874374a5546c32dce29`.
Baseline: `8c1c6b459b8d6a69bb5683200d1d1ef120247460`.
This section owns campaign results; the dependency report retains dependency scope.

| Check | Result | Actual evidence / reason |
| --- | --- | --- |
| Remote and checkout identity | PASS | Clean Local matched remote baseline before edits |
| Exact source provenance | PASS | 1,627 files; pristine upstream unchanged; 11 C/C++ changes plus four exact project quoting fixes |
| Full source pipeline | PASS | 663 passed: 262 Node, 161 Vitest, 240 Rust; 0 failed, 4 existing file-symlink skips |
| Native C/C++ policy executables | PASS | MSVC Release build and three CTest tests: launcher parser, core identity/ABI matching, kernel box policy |
| MFC v143 x86/x64 setup | PASS | User authorized system setup; installer exit 0 and afxres.h present |
| WDK 26100.6584 setup | PASS | Microsoft signature Valid; installer exit 0, kernel headers present, no reboot |
| WDK integration into VS2022 | PASS | Official RunAs retry after operator clarification exits 0; kernel toolset registered; no UAC/security-policy bypass |
| Native Win32 engine build | PASS | Real canonical builder stage exit 0; both required Win32 binaries present |
| Native x64 user-mode build | PASS | Real canonical builder stage exit 0; thirteen required x64 user-mode binaries present |
| Native x64 kernel build | PASS | Real canonical stage exit 0; driver Authenticode status NotSigned |
| Sixteen binaries / five resources / four licenses staged | PASS | 25 artifact records bound to source revision, actual build report and SHA-256; state MIVUBI_V1186_CORE_STAGED_NOT_INSTALLABLE |
| Dependency audit | PASS with warnings | Prior source batch: zero vulnerability-class findings; manifests unchanged; two GTK informational warnings retained |
| Four-process telemetry | PASS | Prior batch at 8928602: actual 30-second disposable Node captures on PowerShell 7/5.1; six samples each; exited Client 03 stays UNKNOWN |
| Production signing / activation / owned operations | BLOCKED | Candidate deliberately unsigned; no Windows-accepted production signature or active owned adapter; no driver/service/account changes |
| Owned gameplay: one, two, then four | NOT_EXECUTED | Runtime unavailable; no login, world, independent input, recovery or multiplayer observation |
| Four-client 60-minute soak, active 60 FPS | NOT_EXECUTED | No four-client game session or displayed-frame capture |
| Background cap near 10 FPS | NOT_EXECUTED | No supported, measured vendor/API route selected; no FPS mutation added |

The full engine attempt used VS2022 Build Tools 17.14, MSVC 14.44.35207,
MSBuild 17.14.40.60911 and Windows SDK 10.0.26100.0. The earlier RC1015 missing-MFC
failure is resolved. MFC and WDK setup were explicitly authorized by the user and
completed. VS2022's WindowsKernelModeDriver10.0 integration is now installed;
all three native build stages pass. Production signing credentials and Windows
driver acceptance were not assessed; the unsigned candidate cannot be activated.

Installed components: MFC v143 x86/x64
(`Microsoft.VisualStudio.Component.VC.ATLMFC`) in the existing VS2022 Build Tools
and WDK 26100.6584 (bundle version 10.1.26100.6584; SDK/WDK path build 26100).
The WDK bootstrapper SHA-256 is
`ed82c46bd98e0f1d07fd6e5075900e42aedc3fa5e68c06a8764b3dc5303cff1b`;
Authenticode was Valid with Microsoft Corporation as signer before execution.
The VS component `Component.Microsoft.Windows.DriverKit.BuildTools` is installed.
After the earlier canceled consent attempt, the operator reported no visible
dialog; a normal official RunAs retry completed with exit 0. No UAC bypass,
manual system-file copying, driver installation, reboot or security-policy change
was performed.
Microsoft lists this kit for VS2022; no VS2026 toolchain migration was attempted.
Sources:
[Microsoft WDK downloads](https://learn.microsoft.com/en-us/windows-hardware/drivers/other-wdk-downloads)
and [Build Tools component IDs](https://learn.microsoft.com/en-us/visualstudio/install/workload-component-id-vs-build-tools?view=vs-2022).
System-component authorization does not include reboot, driver installation,
replacing Sandboxie, changing signing policy or accessing game accounts.

Builder repairs capture VS discovery status in a fresh strict PowerShell session,
wait for the compiler rather than inherited descendants, and retain the process
handle so Windows PowerShell 5.1 preserves its real exit code. Actual exit-7 and
VS discovery regressions pass. The verifier reconstructs all four project fixes
from pristine source; the eleven C/C++ transformation checks remain active.
The standalone identity test uses a bounded copy without suppressing warnings.
No upstream snapshot or driver runtime policy was modified.

### Native-build findings resolved in the final candidate

The first fully compiling attempt exposed WDK's implicit test-sign default:
it generated a temporary test certificate and test-signed the local driver.
That output was rejected for final use, was never installed or loaded, and its
generated certificate plus private key were removed. Targeted checks found no
remaining matching certificate in the user/machine My, Root or TrustedPeople
stores. No real signing credential was accessed or removed.

The builder now passes `SignMode=Off` and `EnableTestSign=false` to every stage
and checks the resulting driver's actual Authenticode status is `NotSigned`
before issuing a successful compile-only receipt. The final clean build meets
that check. A test-signed local binary is not production signing evidence.

Staging also exposed Git autocrlf expansion in configuration/license files.
The assembler now accepts original bytes or CRLF→LF normalization only when
the resulting Git blob ID exactly matches committed HEAD; it writes those exact
bytes and retains the final copied-blob check. Modified configuration or licenses
still fail even after an earlier clean-status observation. Regression tests cover
both accepted CRLF checkout resources and tampering. No tracked resource or
license content was rewritten to accommodate staging.

The existing sampler accepts four distinct explicit targets, pins creation times,
keeps lost identities UNKNOWN, and publishes sanitized schema-3 CPU/RAM reports.
Schemas 1/2 remain compatible. It neither authenticates owned Minecraft targets
nor measures FPS. Usage belongs to the
[Windows tooling guide](../../tooling/acceptance/windows/README.md).

Successful build logs and sixteen hashes are ignored under
`workspace/builds/owned-20261020-150022/`; the matching 25-file artifact manifest is
`workspace/packages/owned-20261020-150022/engine-provenance.json`.
The package is local and NOT_INSTALLABLE; it is not a desktop installer or release.
Earlier rejected output is preserved in `workspace/builds/owned-20261020-145802/`.
WDK setup and new source/policy logs are under
`workspace/dependency-maintenance/wdk-26100-install*`, `owned-toolchain-*`
and `owned-unsigned-*`;
earlier telemetry remains under `owned-four-*`. Earlier failed compiler outputs
were preserved beneath their build-report directories before clean retries.
No binary, account identity, private world, token, certificate or raw host report
is committed.

### Signing and real desktop-inspector review — 20 October 2026

Review source: `f6c0b2308dcf06ae19df777cbec12b7874fc8b4d`.
The reviewed unsigned artifact remains the successful build at
`f529d59fda778be50cd3e874374a5546c32dce29`; this review did not rebuild or replace it.
Observed host OS: Windows 11 Pro, build 26100. No host/account identifiers are stored.

| Review | Result | Evidence / boundary |
| --- | --- | --- |
| Existing desktop Rust inspector on actual staged payload | PASS | Explicit isolation-source:inspect-candidate test verifies all 25 files and asserts provider/load/gameplay/install claims remain false |
| Existing native SCM host probe | OBSERVED / activation BLOCKED | SbieSvc and SbieDrv registrations PRESENT; state BLOCKED_SERVICE_NAMESPACE_IN_USE; names alone do not authenticate an owner |
| SDK kernel signing verification | FAIL, expected for unsigned candidate | SignTool verify /kp /v exits 1: No signature found; no signing operation executed |
| Upstream reference INF qualification | FAIL | InfVerif 10.0.26100.6584 /w exits 1627; source reference is not a usable current submission package |
| Full source pipeline at review source | PASS | 663 passed, 0 failed; four existing Node file-symlink skips and one default-ignored real-payload Rust test |
| Explicit real-payload Rust test at review source | PASS | The default-ignored test is separately executed successfully against the real build; no artifact dependency added to routine source verification |

The root command uses the existing production candidate inspector and SCM probe;
it adds no app IPC, installer switch, lifecycle manager or readiness database.
It reads the path explicitly selected with MIVUBI_TEST_OWNED_PAYLOAD, checks the
actual payload and current service namespace, and never loads its DLL/driver,
starts/stops services or changes account state. The selected path is cleared from
the test process environment afterward. Command usage belongs to the
[native source guide](../../tooling/isolation-source/README.md).

**Signing requirements checked against current Microsoft documentation:** new
kernel drivers use the Hardware Developer Program signing path; an EV certificate
alone is not a Windows kernel signature. Microsoft recommends HLK-tested dashboard
signing for certified distribution. Its attestation route is described for testing,
requires dashboard enrollment/EV credentials, and does not establish compatibility
or Windows certification. It must not be treated as general production acceptance.
References: [kernel signing policy](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/kernel-mode-code-signing-policy--windows-vista-and-later-),
[signing offerings](https://learn.microsoft.com/en-us/windows-hardware/drivers/dashboard/driver-signing-offerings),
[attestation submission requirements](https://learn.microsoft.com/en-us/windows-hardware/drivers/dashboard/code-signing-attestation),
and [SignTool verification policy](https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool).

The 25-file runtime staging manifest is **not a signing submission CAB**. It does
not ship an INF, CAT or debug PDB. The upstream source reference
`engine/isolation-owned/Sandboxie/core/drv/SbieDrv.inf` is not referenced by the
current driver project/install helper. Its stale DriverVer=5.23.5, absent decorated
primitive-install sections/PnpLockdown, DIRID requirements and placeholder
ServiceBinary=examplefilesystem.sys prevent using it as a current submission INF.
The six observed verifier errors are 1423, 1420, 1324, 1217, 1322 and 1421.
No source INF was stamped or rewritten, and no alternate INF-based installer was
introduced alongside the existing KmdUtil design. Submission packaging needs its
own review against the actual service/driver installation contract and legal
publisher identity; upstream copyright notices remain intact.

No partner account was accessed, no certificate store/private key was scanned,
and no upload, enrollment, purchase or signing was performed. Availability of the
operator's Hardware Developer Program access and EV certificate is **NOT AVAILABLE**,
explicitly confirmed by the operator in this continuation. Official signing and
owned-driver activation are therefore externally blocked under the selected
no-bypass route. Unsigned binaries and occupied service names independently block
activation. Source-only work can continue; four owned clients cannot be accepted
from this unsigned build. A future signing result must
be separately verified and bound to new artifact hashes; it must not overwrite
this unsigned candidate or turn its installation flags on.

Review logs are local ignored outputs under
`workspace/dependency-maintenance/owned-signing-*`. These are observations, not an
authorization to install a driver, replace a service or claim playable clients.

### Remaining implementation and acceptance sequence

1. Preserve this unsigned candidate and its exact source/build hashes for signing
   review. Native build/staging are complete; do not reinstall or rebuild merely
   to repeat a passing check. Any new source change requires fresh bound evidence.
2. Verify the complete candidate's hashes, ABI, Windows-accepted signatures,
   service provenance and recovery instructions. Review a concrete activation
   proposal before driver/service changes. Never silently replace official
   SbieDrv/SbieSvc. Reduced-containment settings require explicit review and do
   not prove WAM/account isolation.
3. Implement owned create/launch/inspect/stop, fresh provider/profile/PID/start-time/
   session attribution, cross-process admission and partial-failure handling in
   existing runtime/desktop owners. **Those operations and owned typed bridge
   integration are not implemented by this build/telemetry batch.** Slot admission
   stays blocked; caller flags, checksums or process names cannot authorize it.
4. Prove one owned client, then two, then four with manually signed-in distinct
   licensed accounts, responsive worlds and independent input. Test native↔owned
   and owned↔owned multiplayer, rejoin and sequential client restart without
   session loss or cross-account contamination. Stop escalation on failure.
5. Run four clients for 60 minutes with matching version, graphics and world
   conditions. Rotate focus through each client and capture displayed FPS during
   focused 60-second post-loading windows to evaluate the active 60-FPS target.
   Inter-present data alone cannot award displayed-FPS PASS. Record CPU/RAM,
   available GPU/VRAM diagnostics, crash/freeze and memory growth; no arbitrary
   RAM/GPU threshold is added. Background clients must remain responsive; a supported
   cap near 10 FPS is optional and reported separately.

**Delivery is partial, not full plan completion.** Build/signing/runtime/gameplay
prerequisites remain. GitHub Actions stays OFF, main untouched and VMware paused.

**Scope:** developer-mode local feasibility check for **one native Minecraft Education Client 01 + one Sandboxie Plus Client 02**, on a **single Windows 10/11 x64 PC**. Does not install/replace the legacy VMware manager. Does not start/delete/reset VMware guests and does not automate Microsoft sign-in. The goal of four independently playable licensed sessions is **NOT PROVEN**.

## Historical CI debug binary — UI-only smoke

**CI is disabled.** The following describes historical artifacts only and is not
a current download/build route. Use the existing manual toolchain for source work;
never restore an Actions workflow to obtain a binary.

A successful automatic Windows Full CI run for `Local` can upload a `localtest-debug-<SHA>` **unsigned debug executable** after all twelve source stages and compile-only pass. This is the **shortest first localtest**: no portable developer toolchain or MSVC/WDK build is needed to *attempt* UI startup. Use [CI Debug Candidate](./ci-debug-candidate.md) to download the exact-SHA artifact into `workspace/downloads/`, extract only the expected three files into a new `workspace/packages/debug-<SHA>/`, verify the manifest SHA-256, and try opening/closing the manager on Windows 10/11 x64 with WebView2. It is not an installer, signed release, Sandboxie distribution or Minecraft gameplay proof. Do **not** bypass OS security warnings. The normal full developer localtest route remains `DEV.cmd bootstrap` → `DEV.cmd doctor` → `DEV.cmd preflight` → `DEV.cmd dev`.

## Quick start — first Windows session

Choose one **short NTFS checkout root on a drive you control**, such as
`D:\M-Minecraft-Emulator`. Install Git for Windows as an OS prerequisite.
Open a normal Windows PowerShell or Command Prompt at that drive's parent
directory, then run:

```powershell
git clone --branch Local --single-branch https://github.com/OpenITStudios/M-Minecraft-Emulator.git
cd M-Minecraft-Emulator
.\DEV.cmd bootstrap
.\DEV.cmd doctor
.\DEV.cmd preflight
```

The `bootstrap` command runs through the built-in Windows PowerShell,
downloads the pinned Node/npm, PowerShell, and Rust prerequisites into
`workspace/downloads/`, and extracts/configures them only beneath
`workspace/toolchains/`. Dependency caches and temporary files belong
to `workspace/cache/` and `workspace/tmp/`. No separate `C:\Source`,
`D:\Source`, global Node installation, or global Rust installation is
required by this developer launcher. **Git and Microsoft C++ Build Tools
remain OS-level prerequisites.** Before opening the UI, inspect the paths
printed by `doctor`. Do not change Windows service/driver or installed
Minecraft/Sandboxie locations to make them appear portable.

For an existing checkout, inspect `git status --short` first, preserve
local edits, then use `git switch Local` and `git pull --ff-only origin
Local`. Never run `git reset --hard` as a shortcut. Execute the same
`bootstrap`, `doctor` and `preflight` sequence.

**Stop at the first failed stage.** If preflight passes, run the developer
UI in a terminal at the same repository root:

```powershell
.\DEV.cmd dev
```

This developer mode does not install/replace the old VMware manager.

## A. Before starting (stop on failures)

1. Use **branch `Local`** at a known commit (prefer latest reviewed `Local`). Confirm the checkout is clean with `git status --short` and record `git rev-parse HEAD`. Source changes or untracked edits can invalidate exact-head verification.
2. Run `DEV.cmd bootstrap` and `DEV.cmd doctor` from the checkout root. The portable setup uses the Node/npm/Rust/PowerShell versions in [toolchain.json](../../toolchain.json), including rustfmt and `x86_64-pc-windows-msvc`, under the checkout's `workspace/toolchains/`. Do **not** install these globally just for this project. Git, [MSVC Desktop development with C++ and WebView2](https://v2.tauri.app/start/prerequisites/) remain external Windows prerequisites for the developer-mode build. The `doctor` check proves tool versions/paths, not linker compatibility or that a Windows desktop opened successfully.
3. Use an [official Minecraft Education Windows Desktop installation](https://edusupport.minecraft.net/hc/en-us/articles/13106858087956-Windows-Installation-Guide), not copied binaries or mixed Microsoft Store + Desktop editions. Have **two distinct properly entitled Education accounts** ready for manual sign-in. Do not place usernames, tokens, passwords, world files or screenshots exposing identities in Git.
4. Use a legitimately licensed [Sandboxie Plus installation](https://sandboxie-plus.com/downloads/) in the applicable environment. Check the vendor's business/educational certificate requirements where applicable. Create **one clean, persistent, enabled** box named exactly `MinecraftClient02` in Sandboxie Plus. Keep **AutoDelete**, **AutoRemove** and credential-bypass/protected-storage settings disabled; do not clone existing account data. The manager does **not** provision the box or its license. Check effective settings in Sandboxie Plus, including templates.

### Source verification (repository root)

```powershell
.\DEV.cmd bootstrap
.\DEV.cmd doctor
.\DEV.cmd preflight
```

For runs that reach pinned Node/toolchain checks, preflight automatically creates
a metadata-only report at `workspace/runs/<UTC>-<shortSHA>-preflight-<suffix>/manifest.json`.
The console prints its location. The report records the complete Git commit
and whether source checks passed or which phase failed. Raw terminal output,
account information and Minecraft worlds are **never captured automatically**.
Earlier Windows/Git/toolchain failures may not create a report. An interrupted
`RUNNING` report is not a PASS. Keep reports local and share only redacted
details; see [Artifact Storage Policy](../system/artifact-storage.md).

**Stop if this command fails.** `DEV.cmd preflight` prints four numbered stages, rejects unsupported Windows hosts, and confirms a clean committed `Local` SHA, checks pinned developer tools and the tracked Cargo lockfile, installs locked desktop npm dependencies, runs root `npm run verify:source`, and rechecks that tracked source/HEAD did not change. It does not open the desktop or package the application. If a stage fails, keep the first error and SHA; do not retry by disabling checks. The source verifier runs repository rules, evidence evaluator tests, frontend Svelte/TypeScript/Vitest/build checks, Rust checks/tests/format. A Git commit or an authored test is *not* evidence of a passing command. A clean verification result does not prove Minecraft playability.

**Do not use** `npm run package:windows`, `npm run build:app`, `DEV.cmd package` or `DEV.cmd g0` to obtain an installer: those legacy VMware packaging paths are deliberately blocked while this native pilot is active.

### Start the developer-mode application

```powershell
.\DEV.cmd dev
```

This runs the desktop development UI, **not** the release installer or a virtual machine. Leave the developer console visible; record only generic error classifications and timings. Do not edit game executables, profiles, or credential storage.

## B. Two-client manual acceptance sequence

Start with a short smoke test: verify that the manager opens, inspect the Client 02 Sandboxie box, open Client 01, then request Client 02 only after its setup is eligible. Observe two independent windows, two distinct licensed sign-ins and playable worlds before attempting multiplayer, restart persistence or sustained performance. **Check status** inspects Client 02 Sandboxie/process candidates; it does not establish Client 01 identity, account authentication or game playability. The mounted UI uses **Check status**, **Set up**, **Browse…**, **Open**, **Switch to** and contextual **Move window** (Left/Right → Apply).

| Step | Action | Expected observation | Decision |
| --- | --- | --- | --- |
| 1 | Open **Instances** and select **Check status** on the Minecraft 02 row (or the toolbar **Check status · 02**) before setup or launch | Sandboxie configuration and PID membership are observed, or an actionable UNKNOWN/refusal is shown | UNKNOWN is **never** evidence of an empty box; check Sandboxie service, box settings and inspection access |
| 2 | On Minecraft 02 choose **Set up** → **Browse…**, select the installed Education Desktop EXE (other EXE filenames/unsafe paths are rejected by the picker) and confirm Sandboxie licensing | **Open** is offered only after a positively observed empty box and complete UI setup | Correct any indicated setting and use **Check status** again; do not bypass rejected or ambiguous setup |
| 3 | On Minecraft 01 select **Open**, then inspect the actual Education window and interact with its menu; after any **Open requested** or error, review **Details** | Native game window responds to input; edition is correct | The manager cannot confirm Host 01 is closed or playable. Before any later launch request, check Windows for an existing window and explicitly clear the UI message; this does not terminate the game |
| 4 | On Minecraft 02 select **Open**, then **Check status** again | An independent candidate window is observed inside `MinecraftClient02` | Sandboxie Start.exe success is only a launch request. If the result is ambiguous, inspect first and **do not auto-retry** |
| 5 | Manually sign in to two distinct licensed Education accounts | Both sessions retain different intended identities without crossing | Stop if authentication cannot be independent; do not access or copy tokens |
| 6 | Open two playable worlds; optionally use **Switch to** and **Details** → **Move window** → Left/Right → Apply for an eligible Client 02 window | Both worlds render, accept separate input and remain responsive | Focus/movement is not proof of gameplay; if either game freezes or window identity changes, mark the result unverified or failed |
| 7 | Test approved multiplayer (only with valid permissions) | Both clients join and interact as intended | Record outcomes without account names, tokens, IP addresses or world contents |
| 8 | Close/reopen Client 02, restart Windows, select **Check status** and repeat the independent-account check | Sessions persist without account crossing or unexpected data deletion | Do not claim two-client acceptance if persistence is missing or UNKNOWN |
| 9 | Optional 30–60-minute sustained-use observation | Note candidate processes, CPU/RAM/GPU/VRAM trends, responsiveness and crashes | Do not invent a RAM/FPS threshold; measure the same PC and Minecraft version |

Host 01 does not use the Virtual 02 **Check status** probe. Its **Open requested** or failed-launch feedback remains until the user opens **Details** and, after checking Windows for an existing Education window, selects **I checked Windows · Clear message**. This is only a local UI acknowledgement that restores the explicit Open control. It does **not** check the running process, confirm gameplay, stop a game or launch another copy automatically. Do not clear the message and retry blindly when the native request outcome remains ambiguous.

The file picker checks only the selected executable's path, expected Education filename and existence. It does **not** prove publisher authenticity: the launch path performs the Microsoft product/signature check independently. An invalid selection clears any previous selected path and licence consent; cancelling the dialog preserves the prior selection. Do not treat “File selected” as proof of signing or gameplay.

When opening **Set up Minecraft 02** from Settings, an inconclusive box/process check or an observed isolated process intentionally hides the executable picker and licence checkbox. Use the explicit **Check status · 02** button in that panel to inspect again; do not assume an unknown result means a stopped game. An error panel also offers a read-only status recheck; do not retry **Open** after an ambiguous launch without verifying current process membership. MIVUBI does not create, delete, enable or reconfigure the official account-bearing `MinecraftClient02` box.

The optional **Switch to** and **Move window** controls apply only to an eligible observed Client 02 window. Windows may reject foreground focus; Alt+Tab is the manual fallback. This manager does not verify Client 01 sign-in, or read any account credentials.

A successful **Move window** request leaves a non-launch `Move requested` notice; the existing **Switch to** action stays available only while the last inspected Client 02 observation still identifies a unique candidate window. The backend always rechecks PID, window and executable identity before another focus/move request. A Client 02 **Open requested** notice persists through a quick check with no matching game process or UNKNOWN membership. A read-only empty-box snapshot is not proof the previous game will never appear. An observed boxed game candidate resolves the notice; otherwise, after a fresh positively observed safe-empty check, open **Details**, inspect actual Windows/Sandboxie windows, and explicitly select **I checked Windows · Clear message** to restore Open. This acknowledgment neither stops Minecraft nor proves it closed; the backend repeats launch admission and enforces its retry gap. An inspection error also retains the launch safeguard. **A failed or timed-out Client 02 Open request is also uncertain**: Minecraft might already have started. The error stays visible with the same pending Open safeguard, which survives empty/UNKNOWN status checks until a boxed game candidate is observed or an operator reviews a fresh safe-empty inspection and explicitly clears the notice. Never infer game readiness from either notice.

## C. Read-only collector (optional diagnostics)

```powershell
powershell.exe -NoProfile -File tooling/acceptance/windows/collect-sandboxie-feasibility.ps1 -Stage INSPECT -BoxName MinecraftClient02
```

The collector emits candidate/process and box-setting information, **not** verified account or game status. Some snapshots may correctly return `UNKNOWN` if Sandboxie processes exit mid-inspection or Windows blocks discovery. Source changes now reconcile every reported box PID against a complete Windows process snapshot; inaccessible or missing IDs do not authorize Client 02 launch.

For a manually confirmed two-game report, follow the existing commands in [Sandboxie two-client feasibility](./sandboxie-two-client-feasibility.md). **Only enter YES/NO for outcomes actually observed.** An operator-reported compatible outcome is still not automated login evidence or four-client acceptance.

## D. Failure classification and next action

| Symptom | Record / action |
| --- | --- |
| `DEV.cmd preflight` fails | Save the first failing command, compiler message, git SHA and tool versions; do not proceed to gameplay. |
| `Check status` fails or membership UNKNOWN | Report Sandboxie version, collector error category, whether the box exists, and whether its service is running; do **not** keep clicking launch. |
| Start Menu / Desktop edition rejected | Inspect official Windows installation/Start app mapping; record language and install-path category, **not** private path or credentials. Do not relax signer verification just to pass. |
| `Open` requested but no second window | Check whether Minecraft focused the first instance, an unsigned/unattributed process appeared, or Sandboxie reported an error; record result as inconclusive/failure as appropriate. |
| Distinct sign-ins fail | Stop the experiment; do not patch the executable, broker, tokens or license checks. |
| Game crashes / input broken / account crossing | Stop scaling; preserve existing worlds/profiles and record the first reproducible failure and hardware summary. |
| Installer request | Existing package path is deliberately blocked. Only developer mode is supported for this pilot. |

### Client 02 idle-crash isolation (one controlled session at a time)

The 14 October 2026 pilot proved two concurrently playable Minecraft Education sessions but subsequently reported Client 02 disappearing while idle. The first cause is **UNKNOWN**: the old two-hour Application event lookup returned no matching 1000/1001/1002 record, and the current manager has no continuous Minecraft monitor or idle-stop action. Do not patch an unproven Sandboxie/graphics cause or report the crash resolved.

Use the **same official Education installation, Windows user, licensed Client 02 account, world/scene, graphics options, idle duration (initially 30 minutes)** and Sandboxie box in these comparisons. Close Minecraft normally between runs, save world progress, and do not reset or recreate the persistent box.

1. **A — Sandboxie without MIVUBI manager:** launch Minecraft 02 using the already installed official Sandboxie Plus interface, then close the MIVUBI manager. Observe whether the boxed game stays responsive while idle. This isolates manager involvement, not Sandboxie from the game.
2. **B — Sandboxie with MIVUBI manager:** with the same closed/empty box and verified setup, request Client 02 from the manager, leave the manager running but do not click repeated checks, and repeat the same idle state. An accepted launcher request is not proof of gameplay.
3. **C — native control:** if A or B fails, run one authorized native Education session outside Sandboxie with the same scene and settings. Compare an equally long idle interval. Do not run this control simultaneously with A/B if doing so changes resource pressure.

For each run record the **start/end time including local timezone, whether the game exited, froze or remained responsive, and whether the Windows desktop/other processes remained alive**. After any exit, review Windows **Reliability Monitor** (`perfmon /rel`) and Event Viewer → Windows Logs → Application near that **exact time**. Check Application Error (1000) and Windows Error Reporting (1001), if present. Retain only the application name, faulting-module name, exception code and event time in the handoff; redact names, paths and account/world information. An absent matching event **does not** prove a clean exit or exclude a crash. Do not enable full crash dumps or persist raw reports without a separate privacy review.

Interpretation is conditional: **A and B fail, C passes** → prioritize Minecraft–Sandboxie compatibility; **only B fails** → investigate manager-launched process/working directory/interaction; **A, B and C fail** → investigate game, Windows or graphics baseline first. Other combinations and missing event data are **INCONCLUSIVE**. These are diagnostic routing rules, not proof of root cause. Only a reproduced failure with matching Windows/module evidence justifies a targeted code or configuration patch. Do not change driver signing, account isolation, templates or live profile data speculatively.

## E. Evidence handoff — no secret data

When reporting the **first** error, send a short manually redacted report instead of raw logs or account details:

```text
COMMIT SHA:
WINDOWS: 10/11 x64
PREFLIGHT: PASS / FAILED / NOT_RUN
FAILED STAGE: 1/4, 2/4, 3/4, 4/4 or NONE
FIRST ERROR: <failing command + redacted error>
DEV UI OPENS: YES / NO / NOT_TESTED
CHECK STATUS (CLIENT 02): OBSERVED / UNKNOWN / ERROR / NOT_TESTED
CLIENT 01 PLAYABLE: YES / NO / NOT_TESTED
CLIENT 02 WINDOW: YES / NO / UNKNOWN / NOT_TESTED
SEPARATE ACCOUNT SESSIONS: YES / NO / UNKNOWN / NOT_TESTED
```

Do not share usernames, access tokens, IP addresses, full filesystem paths containing personal names, private worlds, or screenshots with visible account identities. A source preflight PASS does not establish successful multi-instance gameplay.



Record the exact SHA, Windows version, official Minecraft version, Sandboxie version, app/toolchain versions, elapsed time for Check status and Open requests, observed window count, world responsiveness, distinct-account outcome as **YES/NO/UNKNOWN** (no identities), multiplayer result, persistence result, approximate CPU/RAM/GPU measurements, reproducible error message, and whether the observation is **machine-observed** or **operator-reported**.

**Proof ceiling:** Source checks at a different commit do not verify this revision. Until the exact-SHA preflight and Windows two-client tests actually execute, `SOURCE_EXECUTION`, `TARGET_WINDOWS`, `MINECRAFT_RUNTIME` and four-client acceptance remain **NOT_EXECUTED** for that revision. Review [REA-aligned audit](./rea-source-audit-2026-10-13.md) for still-open source risks. This document does not add a second state machine, test gate or service.
