---
id: document.system.implementation-map
class: DOCUMENT
domain: system
role: GUIDE
authority: CANONICAL
lifecycle: ACTIVE
---

# Implementation Ownership Map

> **Retirement note (27 October 2026):** Older sections below contain historical
> Sandboxie/VMware navigation and revision-specific evidence. Vendored Sandboxie
> trees, fork-build sources, legacy Windows guest/installer resources and VMware
> runtime modules have now been retired from `Local`. Treat any older reference
> to these paths as Git-history provenance, not a current file or active owner.
> The current Lazy Tester native controller, window recovery and compatibility
> DLL source remain the product owners.

This is a **navigation guide**, not a task queue. Sandboxie v1.18.6 source/build tooling in [Isolation Source Tooling](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/4d790401823e066adb9c00790ba274b560cdcde2/Development/tooling/isolation-source/README.md) is retired product history, not the native engine. The only active goal, blocker and next action are in [planning/development.md](../../planning/development.md). Treat historical `Next` here as superseded; inspect owning source and consumers.

## macOS diagnostic shell feasibility inventory — 28 October 2026

| Boundary | Exact source finding | Proof ceiling |
| --- | --- | --- |
| Rust target | Desktop Win32 crates and session/window/process implementations use `cfg(windows)`; portable core is retained | Mac Rust/Tauri compilation NOT_EXECUTED |
| Startup/IPC | Tauri base config declares one `main` window; frontend asks `lazy_tester_backend_support` before profile IPC | Actual Mac WebView, native invoke and error UI NOT_EXECUTED |
| Game isolation | Profile binding, named-pipe sessions, DWM and close authority are unsupported on Mac | No Mac login/world/process/five-client capability |
| Window/exit | Default Tauri close is used; no native game-helper startup or Windows recovery cleanup | Mac event-loop exit/reopen behavior NOT_EXECUTED |
| WebView permissions | Existing inline Tauri capability is scoped to `main` with event listen/unlisten plus drag, maximize, minimize and close | No macOS privacy/Accessibility/Screen Recording entitlement approved |
| App instance | Windows `LOCALAPPDATA` exclusive marker is not compiled for non-Windows | Mac single-instance/second-window admission NOT_IMPLEMENTED |
| Packaging | Base is Windows NSIS with PNG/ICO icons; preview and product overlays contain Windows identities/hooks/DLLs | Mac app/dmg, icns, identifier, signing/notarization NOT_REVIEWED |

Source hardening: `build.rs` now refuses **both**
`native-workspace-preview` and `lazy-tester-package` Cargo features on
non-Windows targets, including debug builds. The original Windows release
identity and NSIS gates remain intact; no-feature diagnostic shell source
is still eligible for future compilation inspection, **not** distribution.
The existing native preview source verifier now pins the main-window
capability scope and Windows-only bundle targets. These checks are authored,
NOT_EXECUTED. Tauri's official configuration lists `nsis` for Windows and
`app`/`dmg` for Mac:
https://v2.tauri.app/reference/config/

Do not introduce a Mac packaging overlay, permissions, profile provider,
new instance coordinator or game helper until the separate host feasibility
and security review are authorized.

## Unsupported-platform shutdown boundary — 28 October 2026

The Windows `CloseRequested` confirmation, managed `CloseAll`,
native surface restoration and foreground hook cleanup remain available
only under `cfg(windows)`. Unsupported platforms leave `CloseRequested`
to normal Tauri window handling and do not claim to restore a Minecraft
window or terminate a managed game. The reserved `--workspace-surface`
helper argument is consumed immediately on non-Windows without starting
a helper, reading IPC, or opening a Tauri game view. Windows uses its
original verified helper and close handshake unchanged. This adds no
background worker, OS emulation, process manager, file mutation or native
Mac backend. Repository source checks are authored but NOT_EXECUTED;
actual Mac window close, process teardown and compiled availability still
require authorized target-environment evidence.

## Native process observation compile boundary — 28 October 2026

A source-graph audit of mounted `native_clients.rs`,
`native_clients_session.rs`, `instance_identity.rs`,
`workspace_surface.rs` and `previews.rs` found their Win32 process,
window and pipe implementations already target-gated; unsupported OS
calls return explicit `Unsupported` or `NotImplemented`. The only
remaining unconditional Windows-launch resource in the observer owner
was the Explorer launch mutex / 10-second cooldown. Its imports, state
and admission function are now `cfg(windows)` without changing the
actual Windows launch function or timeout. No Mac game process or
observation authority has been added.

Before claiming a macOS **diagnostic shell**, separately obtain a real
macOS Rust/Tauri compile with exact target configuration, verify the
`main` WebView plus the native capability IPC/error UI, and inspect
exit behavior and app-instance policy. Windows NSIS, DLLs, hard-linked
process roles and `lazy-tester-package` must remain blocked from Mac
packaging; any Mac artifact needs its own reviewed identifier/resources.
Actual multi-account Minecraft Education launch, world isolation,
window presentation, sign-in and gameplay require independent Mac
feasibility/acceptance and are explicitly **not** implied by a shell
compile. All these execution gates remain NOT_EXECUTED.

## Windows instance lease boundary — 28 October 2026

`engine/app_instance.rs` uses a Windows `LOCALAPPDATA`/`APPDATA`
instance marker, with exclusive file ownership of the desktop manager.
This must not run during unsupported-platform startup: macOS does not
promise those Windows environment variables, so calling `acquire`
could prevent Tauri from even showing the honest `Unsupported` UI.
The module import, declaration and startup lease are now `cfg(windows)`;
the Windows installed-product single-instance behavior remains byte-for-byte
unchanged. Non-Windows is currently a diagnostic shell with **no Minecraft
game/profile/window authority**, not a supported multi-instance manager.
A future macOS single-window contract must use reviewed platform app-data
and process ownership, not an invented Windows path. The repository
verifier checks these guards; no compiler or runtime proof was executed.

## Tauri startup platform capability — 28 October 2026

One read-only `lazy_tester_backend_support` Tauri IPC reports an enum
`WindowsNative` only for a compiled Windows x64 controller, or
`Unsupported` otherwise. This is **compiled backend presence**, never
installation, authentication, gameplay or five-session acceptance.
The mounted frontend checks it before binding, location or session IPC:
unsupported builds do not retry Windows profile discovery, never enable
launchers, and show an explicit platform support message rather than a
misleading connection-recovery prompt. Windows retains the original
binding/location retry and startup diagnostics. The native Windows-only
product WebView override cannot disable window creation on other OS targets;
a non-Windows Tauri setup verifies its automatically created `main` window
instead of starting another. Existing Windows package identifiers/payload
contracts remain unchanged, and attempted `lazy-tester-package` builds for
other OS targets refuse in `build.rs`. No macOS native/game backend is
implemented or proved to compile. Frontend/Rust/browser tests are authored
only; source/runtime and UI acceptance remain NOT_EXECUTED.

## Native profile readiness boundary — 28 October 2026

The native profile owner now distinguishes the reviewed installed Windows
product, the existing Windows source/preview fallback, and unsupported
non-Windows targets using mutually exclusive Rust `cfg` declarations.
`profile_binding` and `data_location` return `Unsupported` on non-Windows,
rather than returning a false `configured: true` report. Existing Svelte
binding-error UI already treats this as unavailable and disables launch;
the public profile-binding JSON schema, Windows behavior, session IPC and
installed data remain unchanged. The native session and workspace owners
already refuse unsupported game operations. No Mac account, game process,
window or profile backend is introduced; unit-test and source guards are
authored but NOT_EXECUTED.

## Native presentation target boundary — 28 October 2026

`workspace_surface.rs` now compiles its existing native HWND/DWM
`controller` module exclusively on Windows. Non-Windows targets use the
small `workspace_surface/unsupported_controller.rs` contract implementation:
no process, HWND, helper, state, startup or recovery operation is performed;
the same public functions fail with Unsupported and `is_active` is false.
Selected-close refusal reports no window mutation, and out-of-range slots
remain InvalidInput. Windows controller/recovery logic, Tauri IPC and
presentation schemas have not changed. Native Windows remains the only
implemented backend; this **does not** prove macOS compile readiness,
profile isolation, Minecraft login or multi-instance gameplay. Authored
non-Windows source tests and repository guard have not been executed.

## Windows-only Cargo dependencies — 28 October 2026

The desktop crate declares the same `windows` and `windows-sys` versions and
Win32 feature sets exclusively under `[target.'cfg(windows)'.dependencies]`,
matching the already target-scoped runtime-core manifest. Common Rust/Tauri
dependencies remain unchanged; the existing Windows controller, HWND/DWM,
named-pipe transport, Authenticode, audio, DLL and installer contracts are
unchanged. Cargo still resolves cross-target packages into its lockfile, so
this move does not require removing Windows packages from `Cargo.lock`.
`tooling/repository/verify.mjs` rejects unscoped/absent native bindings
and missing critical Win32 features; its negative regression fixtures are
authored, not executed. This is dependency hygiene, **not** a macOS build
claim: current `workspace_surface` controller and profile bootstrap remain
Windows-specific, and no Mac game/session backend exists.

## Read-only storage inspection policy — 27 October 2026

`runtime-core/src/inspection_budget.rs` owns the portable monotonic,
copyable inspection deadline. The Windows `session_controller.rs` uses one
two-second cooperative budget for all five slots' cache classifications and
profile metadata enumeration during one `InspectStorage` report, instead of
resetting separate two-second scans per profile. When exhausted, affected
results remain `partial` and cache clearance is disabled. The source still
inspects independent clear-confirmation/settings metadata; native OS calls
may block, so this is **not** a hard bound on the full request. Storage
mutations, Windows handle checks, account/world data, session lifecycle and
the user-visible wire schema are unchanged. A different OS can reuse the
budget policy without reusing Win32 filesystem code.

## Close-review inspection admission — 27 October 2026

A native titlebar CloseRequested now emits a UI-only
`lazy-tester-close-review` event when confirmation begins and when it
is canceled or a close attempt fails. The mounted Svelte view suppresses
background observation and preview layout while either full-exit review or
a selected game's CloseOne review/operation is pending. An epoch invalidates
any in-flight inspect result that predates the review; a selected close waits
for the previous refresh/resize task to finish before requesting native
closure. Existing `MANAGER_RESOURCE_LOCK` and the retained single
session controller remain the exclusive mutation authorities. This is
best-effort UI scheduling, **not** preemption of an already-running
native operation, a priority queue, or evidence of runtime improvement.

## Current Lazy Tester route — 26 October 2026

WindowTitlebar.svelte owns the shared product frame controls; main-window-only
Tauri permissions allow drag/minimize/maximize/close. workspace_surface.rs keeps
the manager opaque, preserving DWM frame rendering. The existing DWM preview worker
also renders the unfocused primary game; its ordinary click selects the verified
native HWND, with no input forwarding. Explicit presentation removes that primary
image before activating native gameplay. Manager focus/drag parks the independent
game while its DWM destination follows the manager; foreground and move events
reconcile native geometry through the existing coalesced worker and ordered helper.
The UI move callback does not queue an independent asynchronous game-window move:
such a move can arrive after an acknowledged selection and displace the active game.
Foreground changes also wake the same worker: it rechecks the active owned game's
current focus and geometry, including when a parked game receives foreground focus.
The primary preview is hidden only after the foreground game is visible at the
expected viewport; the worker refreshes previews after reconciling geometry.
Parking rechecks focus on completion to recover a concurrent focus change.
Failed switches attempt both parking the replacement and restoring the previous
view; incomplete recovery is reported and automatic surface updates are suspended.
release_for_close rejects reacquisition
while restoring games. Product CloseRequested then sends CloseAll to the retained
session controller: identity-checked WM_CLOSE to its own original game windows,
normal process-exit/reap wait, no force termination. Absent controller never starts
one for exit. Failure keeps the manager open, restores controls and reports the
pending game/dialog. Idle controller exits after the manager peer disappears.
The controller confirms only the exact English "Exit Minecraft Education?" native
prompt belonging to its original pinned game process; unknown prompts remain for
the user. No other application's dialog or account prompt is accepted.
Frontend separate-slot state releases only the selected
process; preview adoption receives only requested monitoring tiles. DWM destinations
follow manager visibility rather than foreground input; they retain ordinary owned
window stacking. These paths do not control Minecraft lifecycle or sign-in.

Project-local delivery has an icon-bearing native Lazy Tester.exe at checkout root.
It forwards only to App/lazy-tester.exe and owns no configuration/session state.
process_roles.rs admits only fixed named hard links to that same executable file
for Sessions and Window Recovery. Existing file-identity/peer checks remain in force.
windows/process-roles.json owns names; prepare-process-roles.mjs builds localized
Windows descriptions without changing runtime code. The stager and NSIS hooks
create/remove only these verified App links; en-US resources ship with the executable.
This identifies helper roles but does not control Windows/WebView Task Manager grouping.
product_profiles.rs owns schema2 Data/Launchers/Minecraft-1..5, each with private
Roaming/Local and Registry/HKCU.dat. Catalog validation binds user, directory and
hive-file identities. The DLL/enroller validate inherited application hives using
nonsecret directory/slot markers, retaining their ordinary-HKCU route and private
ABI. Missing hives are not recreated. Data privacy ACLs precede profile creation.
The fixed operator-only --lazy-tester-migrate-data copies the reviewed old five
stores opaquely into project Data, preserving sources and publishing the catalog
last. It never starts games or runs from packaging/installer hooks. See data-layout.md.
manager_settings.rs and app_instance.rs use Data/Settings; root resolution also
recognizes source execution within this checkout. New data locations stay in Data.

Launch validation stays in independent_bootstrap.rs. Initial admission resolves and
pins the official image, then rechecks its full Windows registration/signature.
The retained controller calls validate_registered_image against that original file
on each launch: one fresh full resolver, current named-file identity comparison and
exact reviewed digest. No current signature/registration/content check is skipped.
The ignored launch_preflight_measurement release test never opens profiles or
starts Minecraft.

Legacy native registry stores reopen through explicit current-user handles
in independent_bootstrap.rs. Missing stores
are never recreated as recovery. Product-only --lazy-tester-check-webview exercises
the actual compiled Svelte/IPC startup in a hidden WebView with inspection-only
handlers. startup_diagnostic_result emits bounded UI observations to stdout;
normal startup also emits them without exiting. It is not a cached ready flag.
Local delivery creates direct executable shortcuts, then verifies the normal UI
through verify-desktop-startup.ps1, dispatched by the interactive Explorer desktop.
It verifies Explorer -> root starter -> App ancestry, exact receipt/hashes, Connected status and five
enabled launcher buttons. Build-tool child processes can see a different virtualized
HKCU/AppData view despite the same user SID and logical registry root; their CLI
or hidden WebView success alone cannot establish Desktop readiness. Missing stores
must not be recreated as empty recovery. Existing-profile transfer needs explicit
operator approval; packaging never imports account-bearing registry trees.

The Svelte startup retries canonical launcher initialization/location reads before
offering recovery; it does not turn a first missing report into a permanent setup
screen. Product-only --lazy-tester-initialize uses the same profile_binding owner
without launching games. This fixed mode remains a diagnostic; local delivery no
longer uses builder-context initialization as a readiness gate. It adds no ready flag,
profile store or installer mutation route; export-only builds do not run it.

The Play viewport is a centered largest-fit 16:9 element inside a size-observed
stage. That same rectangle feeds existing native placement/aperture and preview
IPC; there is no independent aspect-ratio calculation in the Rust surface owner.
The five-session dock is width-bounded and preview destinations also use 16:9.
app_bootstrap.rs bounds the initial 1280x900 normal window to monitor work area,
frame and DPI; maximize remains an ordinary Windows action.

Automatic UI inspection is separate from manual Refresh feedback. Unchanged
snapshots do not mark native geometry dirty or disable launcher cards; navigation
and launch clicks during inspection are queued. Blocked setup never displays Ready.
Profile binding errors distinguish absent connection metadata from an existing
store that cannot be reopened. The fixed --lazy-tester-inspect-setup mode checks
existing product/legacy metadata only, without profile creation or locator writes.
Local App staging now creates verified Desktop/Start Menu shortcuts through
Development/tooling/windows-toolchain/create-app-shortcuts.ps1. They target the
executable directly, preserving existing unrelated shortcuts and all runtime data.

Play now owns launch and session selection together. Settings contains General
and Data Storage; Advanced options is an emphasized disclosure inside the selected
launcher's storage page. High/Low graphics selections are owned by the original
game_preset.rs writer and manager_settings.rs graphics.json (five selections).
ApplyGraphicsPreset is serialized by the same session actor; launch reapplies the
saved selection. All games must be stopped before manual preset writes. Cache cleanup requires
the selected launcher to be stopped; other verified managed games may continue.
Unknown external game identities still block cleanup through the session actor. The package stager preserves graphics.json with existing preferences.
game_preset.rs also owns reset/restore of only options.txt. A create-new backup is
flushed/read back and its fixed-name receipt atomically published before settings
replacement. Restore verifies bounded content/hash and backs up current settings.
The session actor admits only the stopped selected profile; UI reviews/cancels
before a request. Inspection creates no settings directories. Full selected-profile
clear belongs to product_profiles/clear.rs under the existing session actor. It
replaces only the pinned stopped profile with a fresh file-backed hive/directory,
updates its catalog identities and purges old data through verified handles. The
identity-bound transaction supports rollback before catalog commit and explicit
cleanup recovery afterward; read-only inspection never resumes deletion. UI review
requires the exact launcher name and an irreversible-deletion acknowledgement.
Windows sign-in is unchanged; world export remains unavailable.
Background audio uses session_audio.rs on the retained controller thread and
manager_settings.rs Data/Settings/audio.json. Default Keep makes no mixer changes.
Mute targets only exact live owned PID/birth identities and single-process render
sessions, excludes system sounds, preserves prior mute/volume and restores its own
changes on Keep/normal teardown. Existing controller wakeups sample foreground;
session/device discovery is limited to once per second and ownership changes.
Native errors are reported in General. Abrupt controller-crash recovery of mixer
changes and real Minecraft audio acceptance remain unverified.
Full product exit closes only its managed games normally through the
existing session owner; it never terminates unrelated games or reads their data.

Functional Settings uses the same lazy_tester_session adapter and retained
session actor: InspectSettings, SavePerformance and ApplyPreset. Existing
manager_settings.rs owns Data/Settings/performance.json; the original
game_preset.rs owns all graphics-option writes. Private presentation v3 passes
validated immutable FPS limits to the existing pacer at startup. New preferences default to120/5 with a5FPS minimum for saves; explicit older
preferences remain readable. Historical V2 payload defaults stay144/10. No parallel
limiter, live update worker or profile-state owner is added.

The mounted desktop is `Development/apps/desktop/src/LazyTesterApp.svelte`, consuming
`app/bridge/lazyTester.ts`. Thin commands in `src-tauri/src/commands/native_clients.rs`
route to `Development/engine/native_clients_session.rs` (reconnecting transport via its
`native_clients_session/pipe.rs` Windows adapter) and the existing bootstrap's
`independent_bootstrap/session_controller.rs`. That owner reuses the reviewed
five-profile capability, game/DLL verification, original process handles, FPS
policy channels, and preset writer. It is not a second Windows isolation engine.
Its sibling independent_bootstrap/profile_storage.rs owns bounded cache
classification, disk-space metadata and handle-based HTTP-cache deletion; the
session actor alone admits cleanup against game lifetime. No other storage owner
or periodic cleaner exists.
independent_bootstrap/profile_binding.rs owns the installed product's explicit
existing-profile import and private capability publication. It reuses the original
replay identity validator, stores no login/world contents and never starts games.
The UI consumes a separate typed binding report, not a game-ready/account flag.
Its sibling product_profiles.rs owns the Data/Settings location pointer, private
five-profile catalog, fresh store preparation and verified reopening/reattachment.
It reuses the same storage constructor and session owner. The new
lazy_tester_data_location command reports actual paths and gates native selection;
it does not copy accounts/worlds or introduce another lifecycle manager. See
[data layout](data-layout.md) for preservation and migration limits.
The frontend's only other runtime bridges are invokeRuntime, workspaceSurface and
observedProcess. The latter retains the shared validated OS metadata parser; it
does not import any retired provider API. All active frontend files are reachable
from main.ts. Old screens/VM schemas and exclusive tests are preserved in Git at
5601c354fb91db78a2697ceb6ec710590d7d17be; consult legacy-retirement.md.

Native packaging policy is Development/engine/distribution/lazy-tester-package.json; the
consumer is tauri.lazy-tester.conf.json with the lazy-tester-package feature and
windows/lazy-tester-hooks.nsh. scripts/prepare-native-payload.mjs derives reviewed
DLL hashes from the bootstrap and stages only those two files. The old preview
overlay remains for private source candidates; the default package command now
uses the Lazy Tester route. Neither installer owns game profile provisioning.

Native composition, five-window parking and recovery remain owned by
`workspace_surface.rs`. Its `workspace_surface/previews.rs` owns the four DWM
preview destinations, geometry checks, click notifications and visibility;
`workspace_surface/recovery_gate.rs` serializes original-checkpoint handoff across
GUI/recovery-helper exits. Neither module owns game lifecycle or profile storage.
Compatibility DLL code still lives in `Development/tooling/acceptance/windows/bootstrap-ordering/`;
its product-source relocation is pending. Archived UI and old provider/source paths
below are historical navigation. Current evidence and next work belong only to
[planning/development.md](../../planning/development.md).

## Historical native-first transition — 21 October 2026

The mounted native Windows workspace now offers read-only discovery and
presentation of up to four **already-running** registered Minecraft Education
Desktop process/window candidates. This operates without an isolation provider,
but does not create or authenticate an independent MC02–04 session. The
presentation-only OS recheck is in `native_clients.rs` and the thin Tauri
command; ephemeral slot/window mapping belongs to desktop presentation.
New isolated Launch/Provision remains blocked; no provider admission is
inferred from observing four processes or switching tabs.

Sandboxie and the derived fork are retired for new product sessions. The old
inventory below is historical dependency navigation, not authorization to activate
the fork or enable certificates. See [retirement register](legacy-retirement.md).
`Development/apps/desktop/src-tauri/src/engine/instance_catalog.rs` blocks isolated Launch and
Provision in both old routes; existing-session recovery remains available.
**No independent replacement provider exists.** Its first read-only foundation is
`Development/apps/desktop/src-tauri/src/engine/instance_identity.rs` →
`desktop_compare_process_contexts` → `app/bridge/processContext.ts` → optional
Settings diagnostics. It compares pinned process contexts without the retired
provider, secrets, profile-file reads or launch authority. See
[architecture](architecture.md) and [feasibility](independent-isolation.md).

The existing unmounted desktop WinTrust observer now obtains a SHA-256 DER leaf
signer fingerprint before closing successful provider state. It is read-only
audit data; no official publisher policy, loaded image/file identity or
package trust is approved, and Launch/Provision stays blocked.

A new non-authorizing
`independent_engine/image_provenance.rs::review_image_provenance`
checks the gap between WinVerifyTrust on-disk success, a real
leaf-certificate SHA-256, an independently reviewed official
publisher policy, a proved loaded PE-section/file binding and
package provenance. The default signer allowlist is **empty**;
therefore a trusted generic signature cannot become Minecraft
publisher evidence. The result is always `PendingImageProvenance`
and never unlocks the existing Launch/Provision admission.

The existing desktop process identity owner now offers an
unmounted, read-only
`instance_identity.rs::inspect_process_image_trust_source`.
It pins PID/creation FILETIME, queries the executable pathname,
checks on-disk Authenticode using `WinVerifyTrust` without UI/network
revocation fetches, and revalidates process/file identity. The
result **always leaves the official publisher NotAttested**:
generic Authenticode trust cannot satisfy the Minecraft publisher
acceptance check or enable Launch/Provision. No account or world
data is accessed.

The pure domain now includes
`independent_engine/acceptance.rs::review_game_acceptance`,
a **stateless, non-authorizing manual Minecraft acceptance checklist**.
It requires MC01+MC02 for two-client assessment and all MC01–MC04
for four-client assessment, with process PID uniqueness, separate
client-local checks and simultaneous cross-client checks. It
rejects synthetic or Windows-token-only authentication claims,
requires joint results to bind the exact process set,
retains missing/failed checks and always returns
`PendingGameAcceptance`, never a production provider decision.
No login, game executable or OS process is launched or inspected
by the validator.

The existing Rust inspection owner now implements
`inspect_pair_boundaries` for two independently identified processes,
with singleton and WAM/MSAL broker observations evaluated separately.
Positive claims from Windows token metadata or synthetic fixtures are
not accepted as game compatibility proof. The desktop identity owner
now wires an unmounted **all-Unknown** pair observer; this adds no
account API calls or new production routes. All outcomes remain
pending rather than authorized.

`ScopedIsolationFindings` now rejects assigning a Windows PID to
more than one MC02–MC04 slot, including mismatched creation-time
attempts. Failed proof and tier-specific positive proof cannot be
rewritten within one process-bound review. The existing desktop-native
identity owner also contains an **unmounted read-only**
`inspect_independent_process_pair_source` that compares exact
Windows processes using existing pinned token/SID/profile context
checks and revalidates both live keys. Results are sanitized
diagnostics, not evidence of Minecraft WAM, HKCU, singleton,
world or official executable isolation.

Proof readiness observations are now **scoped separately per
isolated slot (MC02–MC04) and exact PID/creation FILETIME** at
\`Development/engine/runtime-core/src/independent_isolation.rs\`.
The pure diagnostic matrix requires native evidence for the actual
Roaming Known Folder and registry view, and game-runtime evidence
for Microsoft broker account continuity. No synthetic
\`RegLoadAppKeyW\` lock may satisfy those checks. Reused PIDs,
process restarts and a successful result from another slot cannot
become four-client evidence. No second lifecycle owner or
production provider has been introduced.

The optional `-VerifyRegistryAppHivePeer` source route now
tests `REG_PROCESS_APPKEY` **load-lock behavior across two separate
synthetic processes**. One hive is unloaded and readable by the
child while the other is held open by the parent; then the roles
reverse. Only a recognized load refusal counts, and the JSON
receipt continues to mark **real peer registry ACL isolation and
HKCU/Minecraft separation as NOT_EXECUTED**. No host registry keys,
accounts or game worlds are accessed.

An opt-in synthetic application-hive phase has been added to
`Development/tooling/acceptance/windows/appcontainer-probe.cpp`, reachable
through `-VerifyRegistryAppHive` in its existing PowerShell runner.
It creates two new registry hive files under a disposable workspace
and uses only returned `RegLoadAppKeyW` handles to seed/read canaries
and recheck them after close/reload. **Cross-process denial and HKCU
redirection remain NOT_EXECUTED**; no host HKCU/HKU key or Minecraft
data is accessed. The application-hive phase is not a provider.

The documented Windows `RegLoadAppKeyW` facility is the next
**candidate** for an opt-in, temporary application hive experiment:
it returns an `HKEY` rooted in an application hive instead of
writing an ordinary HKCU/HKU path. Its private-handle semantics
do **not** redirect unmodified Minecraft's normal HKCU calls.
See the canonical independent engine architecture for stop
conditions and proof requirements.

A new pure Rust synthetic registry reviewer in
`independent_engine/registry.rs` refuses host HKCU,
unowned/preexisting hives, cross-slot ownership and stale/reused
process identities. It accepts only fixture-hive canary reports and
never approves live registry isolation or Minecraft launch.
The route inspector now separates path-outside-profile
`path_risks` from shared-access `dangerous` findings.
No Windows registry I/O is performed or authorized.

The read-only routing inspector is now
`independent_engine/inspection.rs::inspect_storage_route`. It applies
the canonical route policy before Windows observation, checks exact
process identity/token category before and after storage inspection,
and returns only **Denied** or **PendingNativeAcceptance** with explicit
required/unresolved/dangerous surfaces. The desktop's existing
`instance_identity.rs` exposes an unmounted source-only composition
with the current all-Unknown storage adapter, plus a separate Win32
read-only query for legacy UAC token virtualization flags. Those
flags are NOT proof of HKCU registry isolation or official Minecraft
sign-in, and the isolated Launch/Provision gate stays blocked.

Independent filesystem and registry route **preflight** now lives
in the existing `independent_engine/storage.rs` owner. Pure Rust
`preflight_storage_route` explicitly rejects host/account data,
cross-slot owners, unknown targets, stale process keys, and writes to
the common official installation. MC01 remains under its existing
native owner. Remaining private local/roaming/registry/namespace
requests return **NeedsNativeVerification** with distinct required
OS proof surfaces; no runtime filesystem or registry mapping is
implemented and desktop Launch/Provision remains blocked.

An opt-in, **test-only Windows HANDLE handoff** is implemented by
the existing C++ AppContainer fixture plus
`Development/engine/runtime-core/src/bin/synthetic_fixture_handoff.rs`. The
owner duplicates only child-process QUERY/SYNCHRONIZE and fixed
record-read handles into an explicitly allowlisted Rust helper.
The helper verifies exact PID/FILETIME, live token, file identity and
fixture frame and returns only an exit status. Requires explicit
`-VerifyRustHandoff`; not part of the Tauri app or isolated
Minecraft Launch. Runtime Windows execution still NOT_EXECUTED.

The unbound Windows Rust fixture record reader now checks the
underlying **file HANDLE** using `GetFileInformationByHandle` before
and after its eight-byte read, rejecting invalid file type,
reparse attributes, length and changed file identity. It remains
non-authoritative: only the original C++ fixture owner can establish
the fresh profile + handle provenance, and no cross-language
capability has been issued.

The synthetic C++ fixture now uses one `Child` owner binding:
a freshly created `Profile` is paired with its original process
handle/FILETIME during `launch`. `observe(child)` and `stop(child)`
derive the exact fixture profile from that owner rather than accepting
a second, potentially mismatched profile argument. This is
synthetic-parent ownership only, not a Rust cross-process handoff
or an independent Minecraft runtime provider.

The original native C++ fixture parent now pins self-created process
creation FILETIME and rechecks exact AppContainer SID/liveness around
fixture record reads. Its open record handle must refer to a regular,
non-reparse, exactly eight-byte file; identity and size are rechecked.
The Rust decoder rejects missing core synthetic control flags. This is
fixture-side hardening, not a trusted Rust file-to-child transport.

A bounded *unbound* fixture record reader is now implemented under the
existing desktop process-identity owner:
`Development/apps/desktop/src-tauri/src/engine/instance_identity.rs`.
It takes an already-opened file handle, checks PID/FILETIME and a
Windows AppContainer token before/after one eight-byte read, and returns
`UnboundFixturePaths`. The reader does **not** prove that the file was
written by this process; it is not a production collector, session provider,
profile manager, new Tauri route or game authentication evidence.

The existing C++ test fixture's **eight-byte Observation record** now has
a strict pure-Rust decoder at
`Development/engine/runtime-core/src/independent_engine/fixture_wire.rs`.
It rejects malformed version/length/flags, decodes independent path states,
and requires A/B records to come from the same fixture run phase. The
result is explicitly `UnboundFixturePaths`, **not** a verified
`ProcessPathSnapshot`: there is still no live Rust transport or privileged
process launch. Exact-handle provenance is the next platform boundary.

The cooperating-process path contract is
`independent_engine/paths.rs::inspect_cooperative_paths`, consuming
`CooperativePathObserverPort` and the current `ProcessObserverPort`.
It verifies exact PID/FILETIME before and after a path probe, rejects
mismatched fixture binding and preserves path-only uncertainty. No A/B
fixture label can independently establish process ownership. No native
collector or desktop launch route is registered.

The agreed production language split is **Rust-first hybrid**:
`Development/engine/runtime-core/` and `Development/apps/desktop/` own independent Rust logic and
user-mode Windows APIs; C++ remains for synthetic fixtures, with any
separately authored kernel C/C++ component strictly conditional on
demonstrated need. See [Implementation language and ownership policy](independent-engine-architecture.md#implementation-language-and-ownership-policy---21-october-2026).
No code conversion, service, or driver is authorized by this decision.

The synthetic native AppContainer fixture emits separate path-query outcomes
for each cooperative A/B process (JSON schema 2), with strict offline review.
No raw process identifier or profile path is serialized; A/B labels are
not authoritative PID/FILETIME evidence. Existing game launch remains blocked.

The new `independent_engine/paths.rs` performs source-only, exact
slot/PID/FILETIME review of three process-local path outcomes. It
distinguishes `PathOnly`, `PathOutsideProfile` and `Unknown`
without calling host filesystem APIs or treating path strings as
successful isolation. No production Windows collector is wired to it yet.

Bound native storage evidence mapping is owned by
`Development/engine/runtime-core/src/independent_engine/evidence.rs`. It binds
each fact to exact slot/PID/FILETIME, probe method, and evidence source
before an observation can be recorded. Synthetic positive flags and
path-only data cannot assert actual Minecraft storage or broker separation.
Only pure Rust contracts/tests are added; no Windows I/O or launch path.

The independent `StorageObserverPort` now accepts exact
`ProcessIdentity` along with the slot, rather than slot alone. The
existing pinned Windows process observer also categorizes its actual
AppContainer token without emitting raw SID. Storage remains all-Unknown
and the verified token category grants no launch or game readiness.

The existing desktop identity owner
`Development/apps/desktop/src-tauri/src/engine/instance_identity.rs` now supplies
a read-only `WindowsProcessObserver` and `inspect_independent_source`.
They reuse the pinned PID/FILETIME Windows handle check and leave image,
window and all storage/broker evidence explicitly unknown. These are
unmounted Rust functions, not an alternate Tauri handler or launch path.

A read-only orchestration closure, `independent_engine/inspection.rs`,
joins the process-observer and storage-observer ports. It rejects mismatched
PID/FILETIME observations before inspecting any storage boundary; no Tauri
command, provider admission or persisted status has been created.

The modular, independently authored source architecture is
[Independent Engine Architecture](independent-engine-architecture.md).
The domain contracts live in `Development/engine/runtime-core/src/independent_engine/`
(`identity.rs`, `storage.rs`, `operations.rs`, `ports.rs`). They describe
the four fixed slots, full-width PID/start-time identity, the storage-surface
evidence matrix, effect classification, and abstract Windows adapter ports.
The deliberately disabled provider rejects all actions. Nothing registers
a new desktop launch route, state store, service or driver; old admission
remains fail-closed.

The independently written proof-classification policy lives in
`Development/engine/runtime-core/src/independent_isolation.rs`: pure in-memory findings and
tier-separated evidence gaps for synthetic, native Windows, two-game, and four-game
claims. It is NOT provider readiness, lifecycle persistence, or launch admission.
The existing desktop launch block remains authoritative.

The desktop's existing pinned-process identity owner now also provides a separate
`compare_appcontainer_identities` read-only Rust API. It queries actual Windows
AppContainer tokens and compares SIDs strictly in memory, returning only two
booleans; this is neither a credential comparison nor a provider launch path.
No Tauri/UI command is registered for the new API.

The synthetic AppContainer experiment now also contains a two-process,
two-phase **process-restart persistence** check for private
`synthetic-roaming` canaries. Actual `RoamingAppData` API redirection is
still separately unproven; this is not a game-ready storage provider.
The Windows runner `test-appcontainer-isolation.ps1` validates the
synthetic-only output fields and an explicit negative token-control path.

Offline diagnostics for the synthetic native fixture are owned by
`Development/tooling/acceptance/evaluate-appcontainer-isolation.mjs` and its Node tests.
The reviewer reports only path observations, rejects unexpected private
fields, and cannot authorize a new game launch even if synthetic data passes.
It is not a second lifecycle state store or runtime provider.

The current isolation experiment is a native AppContainer fixture in
`Development/tooling/acceptance/windows/appcontainer-probe.cpp`, explicitly compiled/run by
`test-appcontainer-isolation.ps1`. It launches only itself, validates two native
token namespaces and storage canaries, and leaves game acceptance unexecuted.
Roaming-path redirection remains a material unresolved compatibility question.

## Locate work by problem

**Historical fork source (before retirement on 21 October 2026):** the editable Sandboxie-derived
v1.18.6 implementation is **complete as source** in `Development/engine/isolation-owned/`
(1,627 files, 15 directly modified C/C++ files plus four reviewed project-path
quoting fixes). The unmodified reference is
`Development/engine/isolation-candidates/sandboxie-v1.18.6/`. The active exact-source
verifier is `Development/tooling/isolation-source/verify-owned-v1186.mjs` and the
**build-only** Windows entrypoint is `Development/tooling/isolation-source/build-owned-v1186.ps1`.
The full native build and staging pass on 21 October 2026 at `e2c80ad`:
three Windows stages, 16 unsigned-build binaries, five resources and four licenses.
The resulting 25-file owned package remains NOT_INSTALLABLE. A separate locally
test-signed copy was activated after the operator's reboot/UAC approval;
production signing and four-account gameplay remain unexecuted. See the
[current Windows evidence](../validation/native-pilot-localtest.md).
Standalone C/C++ policy tests compile and pass with MSVC; they are not the
real driver build. Developer driver activation is observed; owned-engine gameplay
acceptance remains unexecuted.
See [current implementation sequence](../../planning/development.md).
The pinned v1.18.5 Gitlink, independent fork scripts, overlays, assemblers
and Windows transaction prototypes listed elsewhere in this guide are
**historical/reference-only** until they are explicitly migrated to v1.18.6;
they must never be chosen automatically as current build or activation code.

| Question / fault | Primary implementation owner | Consumer / companion |
| --- | --- | --- |
| Classic credential fallback, Protected Storage host fallback and compartment bypass | `Development/engine/isolation-owned/Sandboxie/core/dll/cred.c`, `pst.cpp`, `ipstore_impl.cpp` and `core/svc/pstoreserver.cpp` | Exact current delta in `Development/tooling/isolation-source/owned-credential-policy.mjs`, reconstructed by the v1.18.6 verifier; compiled synthetic routine regression and build-only evidence in [native pilot report](../validation/native-pilot-localtest.md). WAM/SSO, game cache and account acceptance remain unproven. |
| Four-tab native composition, shell eligibility and window recovery | `Development/apps/desktop/src/NativeWorkspace.svelte` → `app/bridge/workspaceSurface.ts` → `src-tauri/src/engine/workspace_surface.rs`, gated by existing owned provider | [Workspace contract](../product/unified-workspace.md); retained checkpoints, same-executable recovery helpers, coalesced manager foreground updates in `app_bootstrap.rs`; playable rendering/input and single-taskbar acceptance pending |
| Archived Unified dashboard | Git snapshot `5601c354` → old `UnifiedManagerApp.svelte` | Historical location/provider UX; the current Lazy Tester binding/session owners are listed above |
| Explicit debug-manager owned engine selection and four-slot operations | `Development/apps/desktop/src-tauri/src/engine/owned_provider.rs` → embedded `Development/tooling/isolation-source/windows/owned-manager.ps1` | Existing catalog/resource lock, typed `ownedProviderApi.ts`, mounted `OwnedManagerPanel.svelte`; one-shot Create-only helper in the same executable; no production activation or service installation |
| Diagnose Minecraft present-interval stutter without adding a game hook or daemon | `Development/tooling/acceptance/analyze-presentmon.mjs` + `Development/tooling/acceptance/analyze-presentmon.test.mjs` | Offline operator-supplied PresentMon CSV, one selected Minecraft PID and swap chain, p50/p95/p99 `MsBetweenPresents` only; output excludes PID/account data, never claims FPS/VRAM/gameplay PASS. See `Development/tooling/acceptance/windows/README.md` |
| On-demand GPU counter hint plus stable CPU/RAM report writing | `Development/tooling/acceptance/windows/collect-native-performance.ps1` (`-GpuSnapshot`) → `Development/tooling/acceptance/evaluate-native-performance.mjs` | Default schema-1 remains; optional schema-2 is skipped after PID identity loss; complete UTF8-no-BOM JSON published through exclusive temporary-file move, historical PowerShell 5.1 BOM read; no FPS/VRAM or gameplay claim, see `Development/tooling/acceptance/windows/README.md` |
| Opt-in Windows foreground event counter (zero Minecraft changes) | `Development/apps/desktop/src-tauri/src/engine/instance_focus_events.rs` → existing `commands/native_clients.rs` | One out-of-context WinEvent hook on a dedicated blocking message thread, manual Start/Read/Stop in Settings Diagnostics, same-thread unhook on exit; count-only, no PID/title storage, no client attribution or FPS cap |
| Avoid overlapping high-cost manager PowerShell, dialog, game launch and package hashing | `Development/apps/desktop/src-tauri/src/commands/native_clients.rs` (`MANAGER_RESOURCE_LOCK`) | Existing Svelte `isResourceOperationBusy` in `app/presentation/compactManager.ts` and `CompactManagerApp.svelte`, on-demand only; game CPU/GPU/FPS quotas not implemented |
| Select a single owned Minecraft virtual slot (plan-only, 02/03/04) | `Development/engine/isolation-native/owned-runtime/minecraft-profiles.mjs` (`planOwnedMinecraftVirtual`) | Same persistent Sandboxie setting sequence, with no driver activation; UI readiness explanation belongs to `app/presentation/compactManager.ts` and existing Rust host probe |
| Inspect candidate Sandboxie section names, including disabled boxes (read-only; no ownership proof) | `Development/tooling/acceptance/windows/collect-virtual-boxes.ps1` → `Development/apps/desktop/src-tauri/src/engine/virtual_inventory.rs` → existing Tauri `desktop_discover_existing_virtuals` | `nativeFeasibilityApi.ts` and mounted `CompactManagerApp.svelte`; inspection UI lives in Settings → Diagnostics, official-pilot section candidates only, no Create/Configure/driver activation or account proof |
| User-facing Virtual Manager and internal isolation readiness | `Development/apps/desktop/src/CompactManagerApp.svelte` + `Development/apps/desktop/src/app/bridge/nativeFeasibilityApi.ts` | One mounted UI: Minecraft 01–04 overview first, technical Sandboxie/owned-engine checks collapsed under Settings → Diagnostics; existing Rust inspector (`Development/apps/desktop/src-tauri/src/engine/owned_isolation.rs`); v1.18.6/25 and v1.18.5/24 metadata agree, Create Virtual remains blocked until a signed owned provider exists |
| Compile actual **v1.18.6 MIVUBI-owned** native core without installation | `Development/engine/isolation-owned/` + `Development/tooling/isolation-source/build-owned-v1186.ps1` | `Development/engine/isolation-native/owned-runtime/build-profile-v1186.json`, `verify-owned-v1186.mjs` and `build-owned-v1186.test.mjs`; three-stage real Windows build passes at `e2c80ad`; not installed |
| Native PID/HWND candidates without accounts, tokens or VM | `Development/apps/desktop/src-tauri/src/engine/native_clients.rs`, `Development/apps/desktop/src/app/bridge/nativeFeasibilityApi.ts` | [Native feasibility](../validation/native-multi-instance-feasibility.md); candidate-only |
| Active desktop and blocked legacy VM routing | `Development/apps/desktop/src/main.ts`, `Development/apps/desktop/src/CompactManagerApp.svelte`, `Development/engine/runtime-core/src/command.rs` | [Legacy Retirement](legacy-retirement.md); old Tauri handlers not registered |
| Prove two Minecraft Education Desktop sessions can run natively without VMs | `Development/tooling/acceptance/windows/collect-native-feasibility.ps1`, `Development/tooling/acceptance/evaluate-native-feasibility.mjs` | [Native feasibility](../validation/native-multi-instance-feasibility.md); historical alternative, not selected launch architecture |
| Build/package the current owned v1.18.6 engine (unsigned, not installable) | `Development/engine/isolation-native/owned-runtime/build-profile-v1186.json` → `Development/tooling/isolation-source/build-owned-v1186.ps1` → `Development/tooling/isolation-source/assemble-owned-v1186.mjs` | One exact profile, actual compiler exit/16 binary hashes/5 resources/4 licenses; package remains NOT_INSTALLABLE; Windows build passes at `e2c80ad` |
| Inspect the owned v1.18.6 payload in the desktop (read-only) | `Development/apps/desktop/src-tauri/src/engine/owned_isolation.rs` | Existing Settings → Diagnostics candidate inspection accepts owned schema-2 package but never selects or installs the engine |
| Historical private v1.18.5 assembler (superseded) | `Development/tooling/isolation-source/assemble-private-engine.mjs` → `assemble-engine.mjs` | Retained only for regression/reference; not an active source build or package route |
| Apply a coherent private-source build marker and ABI compiler guards across seven pinned upstream owners in a separate Git fork checkout | `Development/engine/isolation-native/fork-overlay/apply-private-source.mjs`, `build-private.ps1` | Private exclusive fork build route, keeps upstream namespace/ABI intact, private build report distinct from unmodified baseline; no install |
| Compile a native Windows executable that can compare an isolated candidate driver/service/DLL ABI to pinned Sandboxie source | `Development/engine/isolation-native/src/abi_probe.cpp`, `Development/engine/isolation-native/CMakeLists.txt` | Pinned upstream `my_version.h`, read-only offline mode, explicit lab-only `SbieApi_GetVersionEx` runtime handshake; never installed or wired into Tauri |
| Diagnose simulated creation steps against partial/unknown current SCM registrations for manual rollback review | `Development/tooling/isolation-source/windows/reconcile-ownership.mjs` via `reconcile-ownership-cli.mjs --rollback-report` | Per-service state, drift and provenance, never grants stop/delete or driver install |
| Reconcile the canonical on-disk journal hash with current driver/service SCM evidence and production Ed25519 bindings | `Development/tooling/isolation-source/windows/reconcile-ownership.mjs` and `reconcile-ownership-cli.mjs` | Read-only cross-check of both pinned image hashes, actual persisted journal bytes and same-host fingerprint; production signed gate sealed and all action flags false |
| Record operator review intent and append read-only SCM observations in the canonical transaction journal | `Development/tooling/isolation-source/execute-disposable.mjs`, `windows/ownership-ledger.mjs`, `windows/review-ownership.mjs` | One canonical journal, no automatic resume or service deletion; chained events aid reconciliation but do not prove ownership |
| Cryptographically correlate short-lived disposable-lab claims and read-only SCM service image evidence without enabling rollback | `Development/tooling/isolation-source/windows/attestation-gate.mjs`, `service-ownership.mjs`, `probe-owned-service-image.ps1` | Ed25519 envelope verification with production key intentionally unconfigured, journal+host+snapshot binding, verified path/hash review; ALWAYS nonauthorizing |
| Compile pinned Windows KmdUtil operation arguments and inspect driver/service signatures without enabling installation | `Development/tooling/isolation-source/windows/transport-commands.mjs` and `kmdutil-transport.ps1` | Validated v1.18.5 install/stop/delete transport descriptors; Apply sealed pending signer/host/ownership acceptance |
| Journal disposable engine provisioning attempts before any future privileged side effect | `Development/tooling/isolation-source/execute-disposable.mjs`, `simulate-disposable.mjs` | Exclusive write-ahead simulation ledger; source hook for future Windows transport, no Tauri integration |
| Read-only signing and service-registration diagnostics for a candidate owned Windows runtime | `Development/tooling/isolation-source/windows/probe-disposable-readiness.ps1` | File signature and collision evidence; virtual-machine detection heuristic NOT host attestation, install always blocked |
| Design a disposable-only driver/service provisioning and rollback transaction before any Windows mutation | `Development/tooling/isolation-source/provision-disposable.mjs`, `plan-disposable.mjs` | Deterministic upstream KmdUtil install/rollback sequence; always `SOURCE_PLAN_ONLY_DRIVER_INSTALL_BLOCKED`; not an active Tauri runtime |
| Check Windows `SbieSvc`/`SbieDrv` registration without PowerShell/WMI startup | `Development/apps/desktop/src-tauri/src/engine/owned_isolation.rs` (native SCM `OpenSCManagerW`/`OpenServiceW`) | Same Tauri/UI host-inspection contract; no service mutation, denies unknown/occupied names; `Development/tooling/isolation-source/windows/probe-host-service-names.ps1` is retained as historical read-only reference only |
| Inspect a staged owned Sandboxie-derived engine candidate without installing or selecting it | `Development/apps/desktop/src-tauri/src/engine/owned_isolation.rs` via `native_clients.rs` | Read-only Tauri command `desktop_inspect_owned_engine_candidate`, typed desktop bridge; staged payload is NOT_INSTALLABLE |
| Request Client 02 launch in one persistent MinecraftClient02 Windows Sandboxie box | `Development/apps/desktop/src-tauri/src/engine/native_clients.rs` → Tauri native command | [Sandboxie Client 02 experiment](../validation/sandboxie-two-client-feasibility.md); explicit, non-destructive launch request only |
| Attribute signed Education image candidates without claiming login or native ownership | `Development/tooling/acceptance/windows/collect-sandboxie-feasibility.ps1` → `Development/apps/desktop/src-tauri/src/engine/native_clients.rs` | Explicit **Check status** (`-ImageProof`) alone invokes the per-PID `SbieApi_QueryProcess` attribution; a failed CLI enumeration can load a trusted vendor DLL in any on-demand inspection. Focus/Place use only a targeted boxed PID Authenticode check, not `QueryProcess`. Launch remains a fast signed-EXE preflight |
| Inspect current Client 02 box membership, credentials and persistent settings, without touching accounts | `Development/tooling/acceptance/windows/collect-sandboxie-feasibility.ps1`, `Development/tooling/acceptance/evaluate-sandboxie-feasibility.mjs` | [Sandboxie feasibility](../validation/sandboxie-two-client-feasibility.md); active collector/evaluator, candidate observations are not proof of login |
| Focus or place Client 02 window using fresh PID/FILETIME/HWND, Sandboxie membership and selected signed Education file evidence | `Development/apps/desktop/src-tauri/src/engine/native_clients.rs` → canonical PowerShell collector with `-BoxedImageProofPid` | Only one requested boxed PID gets Authenticode proof at action time; name-only UI cannot authorize a move, and Win32 may refuse foreground; no legacy VM route |
| Embedded collector execution without oversized Windows command lines | `Development/engine/runtime-core/src/provider/process.rs` bounded stdin runner → `Development/apps/desktop/src-tauri/src/engine/native_clients.rs` | Source is streamed as UTF-8 from the embedded collector into a short-lived PowerShell child. No temporary script, daemon, or schema change; Windows execution still unproven |
| Resource-efficient native/isolated management, no VM or new daemon | `Development/apps/desktop/src-tauri/src/engine/native_clients.rs` and [Architecture](architecture.md) | `Development/apps/desktop/src/CompactManagerApp.svelte`; one on-demand read-only Sandboxie candidate observation, not independent Client 01 authentication, no governor |
| Prove one disposable Windows guest can boot, sign in and render Minecraft Education on VMware or App Sandbox GPU-PV, with provider-specific graphics evidence | `Development/tooling/acceptance/windows/collect-client-feasibility.ps1`, `Development/tooling/acceptance/evaluate-client-feasibility.mjs` | [Client Feasibility](../validation/client-feasibility.md); no production VM mutation |
| VM status, start/stop/suspend/restart/reprovision and rollback | `Development/engine/runtime-core/src/runtime.rs` | `lifecycle_admission.rs`, provider implementation |
| Lifecycle eligibility / guard decisions | `Development/engine/runtime-core/src/lifecycle_admission.rs` | `runtime.rs`, `client.rs` |
| Legacy VMware client-owned setup wizard (inactive) | `Development/apps/desktop/src/app/setupFlow.ts` and `Development/apps/desktop/src/App.svelte` | Historical `shouldShowClientControls` regression; not mounted in native pilot |
| Legacy VMware first-run setup decisions (inactive) | `Development/engine/runtime-core/src/doctor.rs` | Unmounted `src/app/setupFlow.ts`, `src/app/surfaces/SetupSurface.svelte` |
| Base provenance, immutable finalized image and parity | `Development/engine/runtime-core/src/profile.rs`, `base_preparation.rs` | `Development/engine/guest/windows/prepare-base.ps1`, `finalize-base.ps1` |
| VMware Workstation/Fusion operations and VMX | `Development/engine/runtime-core/src/provider/` | `runtime.rs`, `diagnostics.rs` |
| Explicit disposable App Sandbox pilot (`inspect/start/graceful shutdown/display`) | `Development/engine/runtime-core/src/provider/app_sandbox_pilot.rs` | [GPU-PV Disposable Pilot](../validation/gpu-pv-disposable-pilot.md), CLI only; no production VM controls |
| Identity and guest disk isolation | `Development/engine/runtime-core/src/identity.rs`, `provider/vmx.rs` | `profile.rs`, `Development/engine/guest/windows/` |
| Guest Agent authenticated status/launch | `Development/engine/runtime-core/src/guest.rs`, `src/bin/virtual-guest-agent.rs` | `Development/engine/guest/windows/install-guest-agent.ps1` |
| Native Minecraft detection and executable constraints | `Development/engine/runtime-core/src/minecraft_runtime.rs`, `profile.rs` | `guest.rs`, `doctor.rs` |
| RAM/CPU resource pressure and start admission | `Development/engine/runtime-core/src/resources.rs`, `policy.rs` | `runtime.rs`, `capacity.rs` |
| State schema, durable profile and recovery | `Development/engine/runtime-core/src/schema.rs`, `persistence.rs` | `journal.rs`, `runtime.rs` |
| Update policy, signed-asset manifest, installer integrity | `Development/engine/runtime-core/src/update.rs`, `Development/engine/distribution/` | Tauri package configurations, package smoke |
| Open host-native Windows Minecraft / inspect candidate HWNDs without claiming account or gameplay proof | `Development/apps/desktop/src-tauri/src/engine/native_clients.rs`, native Tauri adapter | Active `Development/apps/desktop/src/app/bridge/nativeFeasibilityApi.ts`; legacy `virtualClientsApi.ts` unmounted |
| Active desktop navigation / transient visual state | `Development/apps/desktop/src/LazyTesterApp.svelte` | `Development/apps/desktop/src/main.ts` mounts Lazy Tester; old App/Unified/Compact/feasibility components are archived in Git, absent from active source |
| Typed frontend/backend IPC and contract validation | `Development/apps/desktop/src/app/bridge/` | `Development/apps/desktop/src-tauri/src/commands/` |
| Display DPI bootstrap (active); legacy VMware layout/overlay (inactive) | `Development/apps/desktop/src-tauri/src/engine/dpi.rs`; unmounted `window_arrangement.rs` and `screen_overlay.rs` | Active Client 02 **Move window** is in `native_clients.rs`; legacy layout Tauri commands are unregistered |
| Windows package contents/install lifecycle | `Development/engine/distribution/package-contract.json`, `install-lifecycle-policy.json` | `Development/apps/desktop/src-tauri/tauri*.json`, `scripts/prepare-package-binaries.mjs` |
| Target Windows machine acceptance collector | `Development/tooling/acceptance/windows/` | [Validation](../validation/README.md) |
| Root scripts, tooling pin/metadata and source checks | `Development/package.json`, `Development/toolchain.json`, `Development/tooling/repository/verify.mjs` | `Development/tooling/windows-toolchain/dev.ps1`, `.github/workflows/` |

## Domain boundaries

```text
ACTIVE: Svelte UI → typed bridge → thin Tauri command
→ native_clients.rs → official Windows launcher / Sandboxie Plus
→ bounded process + Sandboxie observation → candidate-only UI

PAUSED: Rust VM runtime → VMware provider / bounded Guest Agent
→ old VM status schema and desktop surfaces (unmounted)
```

UI never decides or stores whether the Base, guest or signed-in session is ready. The runtime does not own Windows window layout presentation. Documentation never replaces either.

## Route by work type

| Work | First navigation |
| --- | --- |
| Change engine behavior | [Runtime Core](runtime-core.md) → listed Rust owner → source tests |
| Improve desktop | `Development/apps/desktop/README.md` → affected Svelte/Tauri owner |
| Base image or guest Windows concern | [Base Image](../product/base-image.md) → `Development/engine/guest/windows/` |
| Package/release safety | `Development/engine/distribution/README.md` → contract JSON → Tauri consumer |
| Investigate VM/game acceptance | [Acceptance](../validation/acceptance.md) → [Failure Matrix](../validation/four-client-failure-matrix.md) |
| Continue unfinished repo work | `planning/development.md` + current GitHub `Local` HEAD |

## Exact lookup rule

Identify one first wrong owner and only its caller/test dependency closure. Search the entire repository only when a concrete cross-domain reference remains unresolved. Source, machine-readable contracts and actual runtime evidence take precedence over overview text. Keep obsolete implementation names in Git history, **not** duplicated source folders.
