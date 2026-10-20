---
id: document.validation.native-pilot-localtest
class: DOCUMENT
domain: validation
role: PROCEDURE
authority: WORKING
lifecycle: ACTIVE
---

# Minecraft Instance Manager — Windows local test (native + Sandboxie)

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
