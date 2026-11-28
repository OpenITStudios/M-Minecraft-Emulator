# Current Development Handoff — Lazy Tester

## Current status — cross-platform-native (MAC-03a)

Branch `Local`; Unreleased. Target: one native Lazy Tester for Windows and
macOS, retaining up to five independent licensed Education profiles
(MC01–MC05), one official installation per host, one UI, one retained
session authority, one catalog and one MCP/SDK operation core. ZLR-04 and
all native, execution and release holds remain. DESIGN_REVIEWED;
implementation BLOCKED at Mac profile isolation. MAC-02 candidate provenance
and mechanism review recorded below. Not SOURCE_READY_FOR_LOCAL.

### MAC-03a — shared UI session capability (source only)

Extract the existing positive Windows session check into the typed bridge's
`supportsManagedSessions` predicate. Only WindowsNative is accepted today;
Unsupported, null and forged/new backend labels do not enable sessions.
The existing wire enum, Rust authority, feature gates, slot catalog,
DWM/geometry/release guards, timing and one UI dispatcher are retained.
Use the predicate for common session observation, settings/navigation,
feedback, action dispatch and titlebar close wording. Adapt the existing
frontend negative fixtures and repository source witnesses; no new state,
workers, dependencies or native calls. Tests/typecheck NOT_EXECUTED.
This is a bounded shared-source preparation, not native Mac admission.
MAC-02 artifact and isolation blocker remains; no automatic fallback.

### MAC-02 — artifact candidate and isolation findings (2026-11-28)

OBSERVED_EXTERNAL: [official changelog](https://edusupport.minecraft.net/hc/en-us/articles/360047556451-Minecraft-Education-Change-Log)
identifies retail 26.33 (2026-10-28). The [pinned upstream Homebrew cask](https://github.com/Homebrew/homebrew-cask/blob/e4b5deed8c87229fd51ee3a71aff7f8679053419/Casks/m/minecraft-education.rb)
and [cask API](https://formulae.brew.sh/api/cask/minecraft-education.json)
identify the candidate below. The SHA is a third-party published
download checksum, NOT an independently computed hash, a signer pin,
or runtime admission.

- Candidate: `https://downloads.minecrafteduservices.com/retailbuilds/MacOS/Minecraft_Education_1.26.33.0.dmg`
- Expected DMG SHA-256 from cask:
  `80bb151b1db4e46edc503230bc8ece25210fff94b7d73d6774559a054398a91e`
- Bundle entry: `minecraft-edu.app`; Homebrew installs it as
  `Minecraft Education.app`. Names are discovery hints only.
- Cask lists both `com.microsoft.minecraft-edu` and
  `com.microsoft.minecraftpe` preference/cache paths; neither is proof
  of the current bundle ID. Preserve all candidate legacy data; do not
  invoke the cask's zap or uninstall rules.
- Official [Mac update guide](https://edusupport.minecraft.net/hc/en-us/articles/360047705032-Update-to-a-New-Version-of-Minecraft-Education)
  describes manual installation. Do not port Windows updater-task semantics.

| Mechanism | Evidence and decision |
| --- | --- |
| AppKit new instance | [New instance](https://developer.apple.com/documentation/appkit/nsworkspace/openconfiguration/createsnewapplicationinstance) supports a launch request; [LSMultipleInstancesProhibited](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/LaunchServicesKeys.html) can refuse it. Actual plist and internal game singleton unknown. Candidate only. |
| Per-process environment or argument | [Apple environment](https://developer.apple.com/documentation/appkit/nsworkspace/openconfiguration/environment) and [arguments](https://developer.apple.com/documentation/appkit/nsworkspace/openconfiguration/arguments) are ignored for sandboxed callers; [NSHomeDirectory](https://developer.apple.com/documentation/foundation/nshomedirectory%28%29) uses the user's or sandbox's home. No verified Education profile argument. Do not assume HOME redirects all files or identity. |
| Separate Microsoft accounts | [World locations](https://edusupport.minecraft.net/hc/en-us/articles/4404785703316-Location-of-World-Files) explicitly use the device login user, not the game account. [Manage Accounts](https://edusupport.minecraft.net/hc/en-us/articles/360061382672-Sign-Into-Minecraft-Education) can remember the last player or switch accounts; no concurrent isolated login guarantee. |
| App Sandbox / Keychain | [Apple sandbox](https://developer.apple.com/documentation/security/accessing-files-from-the-macos-app-sandbox) binds container to signed app identity on macOS 14+; [Keychain groups](https://developer.apple.com/documentation/security/sharing-access-to-keychain-items-among-a-collection-of-apps) bind protected items to authorized groups. Neither is a supported arbitrary per-instance profile namespace. Do not modify signature, entitlements or credentials. |
| Alternative Bedrock launcher | [mcpelauncher upstream](https://github.com/minecraft-linux/mcpelauncher-manifest) uses a separate launcher/Android compatibility stack; its own [paths](https://mcpelauncher.readthedocs.io/en/latest/faq/index.html) do not establish official Education native compatibility. Reject as replacement. Java launcher gameDir and Education Classroom Mode multi-instance are different products, not evidence. |

Smallest discriminating artifact review for a future authorized evidence
source: read the exact DMG checksum, app Info.plist (CFBundleIdentifier,
CFBundleExecutable, CFBundleVersion, CFBundleShortVersionString,
LSMinimumSystemVersion, LSMultipleInstancesProhibited if present), executable
Mach-O slices, signed bundle integrity, designated requirement and relevant
sandbox/app-group entitlement presence. Use macOS standard metadata
tools (shasum, plutil, lipo, codesign) on the authorized copy; no installer,
launch, credential enumeration, data migration, entitlement changes or
Gatekeeper override. Keep only sanitized metadata, no package bytes in Git.
[Apple designated requirement](https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements/)
is a signature identity, not proof of profile isolation. Missing plist key
is not proof that the game allows multiple instances.

Decision rule: a valid image and supported launch mechanism are necessary,
not sufficient. Before any managed Mac launch, establish a supported
per-instance game data AND account mechanism with no shared mutable roots,
including settings, caches, worlds and authentication broker behavior.
If no supported mechanism exists, keep the Mac route unsupported; a
single-client Mac variant or other-user workflow changes product acceptance
and requires a separate design decision. Do not open a second runtime
or call a normal user game session an isolated profile.

Proof: official documents, exact upstream cask source and existing
Windows source/unsupported gates reviewed. DMG bytes, actual Info.plist,
Mach-O, signature, entitlements and native login/storage behavior NOT
INSPECTED; no host or execution evidence. No new runtime module, dependency,
CLI, test run or version bump. MAC-02 advances provenance and narrows the
blocker, not Mac implementation readiness.

### Minimal architecture — proposed, not implemented

Keep UI -> typed bridge -> allowlisted Tauri commands -> retained Rust
controller -> native game. Bind OS-specific primitives at compile time;
no runtime provider chooser, fallback, second controller, VM, new service,
extra polling, credential store or game patch. Share policy only where
needed by a second concrete consumer; do not mount historical provider
traits as another runtime. Preserve Windows public commands, wire receipts,
slot IDs, private data and existing admission gates.

| Existing owner | Planned boundary |
| --- | --- |
| `engine/native_clients.rs` | Official target admission; Mac bundle/signature/version/architecture, unchanged Windows PE/DLL checks. |
| `independent_bootstrap/session_controller.rs` | One session lifecycle/state owner; extract neutral policy only as needed, retain native handles and uncertainty/recovery. |
| `native_clients_session.rs` + `session_protocol.rs` | Shared validation, framing and admission; Windows pipe, candidate Mac Unix socket with exact peer/image verification, no replay. |
| Existing product profile/storage owners | One five-slot catalog per host; verified private backing per OS; no automatic migration or credential copying. |
| `process_identity.rs` | Preserve Windows receipts; explicitly tagged/versioned Mac identity if needed; no invented FILETIME/HWND. |
| `workspace_surface.rs` | Same presentation contract; DWM or AppKit/Accessibility, optional ScreenCaptureKit after explicit permission and exact target validation. |
| Capture and agent core | Keep capability gates; no automatic capture/input/recording parity; NVENC is not a Mac requirement. |
| `LazyTesterApp.svelte`, `app/bridge/lazyTester.ts`, presentation/titlebar consumers | Replace OS-specific readiness comparisons with shared capability decisions only when real backend is admitted. Keep Unsupported fail-closed. |
| `agent-configuration.mjs`, `native-transport.mjs` | Same MCP/SDK core and same-image CLI; review .exe/path assumptions with real Mac executable. |
| `build.rs`, Tauri config, existing distribution/developer tooling | Preserve NSIS gates; later native .app/DMG and signing/notarization; no second build system. |

Naming: use `windows.rs` / `macos.rs` under the actual owner only when
needed; preserve clear existing names, no blanket rename or empty
scaffolding. Proposed target priority macOS 14+, Apple Silicon first;
Intel not excluded or declared compatible without artifact evidence.

### Isolation feasibility — blocking gate

Official [Mac installation](https://edusupport.minecraft.net/hc/en-us/articles/360047118792-Mac-Installation-Guide)
and [requirements](https://edusupport.minecraft.net/hc/en-us/articles/360047556591-System-Requirements)
confirm Mac 14+ support. Official [world locations](https://edusupport.minecraft.net/hc/en-us/articles/4404785703316-Location-of-World-Files)
show a device-user-relative Library/Application Support/Minecraftpe/Games/com.mojang
root, not a per-instance root. Apple [new instance](https://developer.apple.com/documentation/appkit/nsworkspace/openconfiguration/createsnewapplicationinstance)
and [launch environment](https://developer.apple.com/documentation/appkit/nsworkspace/openconfiguration/environment)
APIs do not establish independent Minecraft storage, singleton or login.
[App Sandbox](https://developer.apple.com/documentation/security/accessing-files-from-the-macos-app-sandbox)
and [Keychain access groups](https://developer.apple.com/documentation/Security/sharing-access-to-keychain-items-among-a-collection-of-apps)
are app/signing-associated, not arbitrary instance isolation.
[Fast user switching](https://support.apple.com/en-euro/guide/mac-help/mchlp2439/27/mac/27)
does not establish simultaneous shared-desktop interaction. Do not use as
fallback. No VM, duplicate game copies, global HOME/root swaps, ad-hoc
re-signing, security bypass, credential inspection/copying or invented flags.

Prefer a supported, verified per-instance profile mechanism. None established
in reviewed references. Exact official Mac bundle artifact, identifier,
signing requirement, Mach-O architectures, storage/authentication routing
and singleton behavior remain unknown. Preserve Unsupported and refuse
managed launch until a safe mechanism is established; two PIDs are not proof.

### Implementation and acceptance sequence

1. MAC-02: candidate package identified; actual artifact metadata and
   verified independent storage/account mechanism remain outstanding.
   Obtain sanitized image/architecture/signature and boundary evidence. No binary or secret in Git; no execution now.
2. MAC-03: implement narrow owner-to-consumer slice, one controller and
   two independent persistent profiles, then reuse slot path for five.
   Author fixtures for shared-root rejection, changed image, stale identity,
   unauthorized IPC peer, uncertain close and unsupported capability.
3. MAC-04: integrate shared UI and agent flow, explicit manual launch and
   sign-in, native presentation. Apple
   [NSRunningApplication](https://developer.apple.com/documentation/appkit/nsrunningapplication)
   and [ScreenCaptureKit](https://developer.apple.com/documentation/ScreenCaptureKit)
   are candidates, not integrated. No unverified permission/features.
4. MAC-05: target-specific packaging after source/admission review;
   [Tauri distribution](https://v2.tauri.app/distribute/) signing and
   notarization for external distribution. Release approval separate.
   Later separately authorized native evidence: two then five independent
   licensed playable accounts/worlds, separate persistent writes, normal
   close/relaunch/reconnect, permission-denied behavior, preservation,
   CPU/RAM/GPU/frame time/startup and Windows regression. No estimated
   performance claim or source-only PASS.

MAC-01 and MAC-02 change planning only; MAC-03a changes common frontend
source and matching fixtures only. No native runtime code, dependencies,
fake adapters, test execution or weakened gates. Builds, native/gameplay/performance,
packaging and tests NOT_EXECUTED. Next: BLOCKED on verified Mac artifact
and defensible isolation mechanism. Preserve all existing holds.

## Previous source closure — zero-legacy-runtime (unchanged)

Branch `Local`; Unreleased. Work: `zero-legacy-runtime`.
Approved goal: one retained native runtime, canonical profile storage, one agent
contract (MCP and SDK adapters), no extra worker/dependency or redundant owner.
Previous MOD/SIM-MOD source closure and exact history:
[15cd0072 handoff](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/15cd0072fcb0e78d74ae661a6092eb4a5eac8851/planning/development.md).

ZLR-01: retired executable pair/five startup and five-profile preparation CLI.
Retired flags fail before dispatch, old runner and legacy profile creation
removed; shared controller journal and test-only historical observer preserved.
Reviewed existing data/replay retained until safe storage cutover. Pair-only
historical data preserved, no automatic expansion. Source fixture NOT_EXECUTED.

ZLR-02: product runtime/binding/location admit verified schema 2 only; non-product
managed launch refused. Explicit existing migration command accepts verified
schema 1 or five-slot replay only, holds original manager/controller gates,
preserves source catalog/data and switches the canonical locator after five
target stores reopen. Schema 1 destination is selected-root/Migrated; old
replay uses unused default Data. Partial target refuses; explicit Locate of
verified complete schema 2 is recovery. Old root/binding presence prevents
fresh setup; no auto-import, fallback or pair expansion. Fixtures NOT_EXECUTED.
ZLR-03: one normal 23-tool catalog, same MCP/SDK tool names and core. Former
surface selectors and Probe prepare/read aliases refuse; trusted new
createAgentClient(core) derives methods from the original catalog, retains
budgets/closure and refuses experimental attachment. Existing explicit MCP
input gates, private checkpoint, full metadata and native authorities remain.
Explicit WebP requests share compact visual receipts; no auto-format fallback.
Source fixtures authored/adapted NOT_EXECUTED. Old host configurations require
operator review; no automatic permission widening.
ZLR-04: compare baseline 15cd0072 to 76cc5d77 (3 commits, 31 paths); review
selected active Rust storage/controller, MCP/SDK/type/catalog, fixtures,
package/verifier and recovery documentation consumers. Canonical logical
AppData validation no longer resolves a legacy replay parent; legacy HKCU
creation fixture-only. Migration no-op receipt reports 0, destination creation
and identity pins tightened, overlapping source/destination refused. One agent
catalog selected per core; Probe schema and private entry reconciled. Fixtures
NOT_EXECUTED. No new dependency/worker. Source phase closed at scoped proof;
external host configurations and exhaustive semantic review not verified.
Next: STOP (SOURCE_READY_FOR_LOCAL at scoped source proof only, not execution approval).
No compiler, test, CI, runtime, data migration or performance execution.
No release or version bump.

## Prior source work and continuity

UPD, RPT-01–07, CAP-01–05, HW-01, FIN-01–08, REV-01–08, BS-01–07,
FLOW-01–09, MCP-01–08, OPT (except OPT-07), E2E-01–06, BS-01–10,
SIM/EFF/DDR/NGE/FMS/DST/Rust-first and ICE-01–04/PLA scopes remain
source-closed, not executed acceptance. OPT-07, ICE-05 and VAL-01–06 held.
Preserve existing decisions and stricter safety constraints; do not reopen
closed work or treat historical Next fields as authority.
[Complete exact pre-MOD-08 handoff and decision history](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/f7fc983ead90964b40d99ec68c06213ab19e8215/planning/development.md).
Use [implementation map](../docs/system/implementation-map.md),
[architecture](../docs/system/architecture.md) and
[development discipline](../docs/system/development-discipline.md).

## Proof ceiling and unresolved gates

- Native presentation ABI v4 digest pair not admitted; standard dev/package/new launch held. Exact DLLs require independent review; never substitute arbitrary bytes.
- No current source test, Windows build, five-client/gameplay, UI visual, capture/recording or performance evidence.
- World/pack disposition, download cache root and seven-day log producer/root held. Existing web-cache admission unchanged; preserve accounts, worlds, private hives, snapshots and legacy roots.
- Client-native F3 `BLOCKED_ENGINE_ABI`: exact Education 26.33 image and input/player/dimension/camera/render interface required. No guessed offsets or fake toggle. [Design](../docs/system/minecraft-education-native-f3-client.md).
- Experimental input/AI/MCP holds unchanged: default OFF, exact writer/neutral/quarantine semantics, no unverified gameplay or Probe dispatch. [Prior decision](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/9b285c2110b7cdf91eda7e26c315a0b850eab82c/planning/development.md#separate-original-native-input-hold--unchanged).
- Candidate export not publication: extracted payload/legal notices, signed bytes, Authenticode chain/timestamp, clean/upgrade/uninstall and active/concurrent native session evidence held. Installer idle check is point-in-time. Repository visibility operator-managed.

## Deferred PC-local validation — not an authorization

Only a separately selected verified `LOCAL_PC_AGENT` with explicit
operator approval may execute the existing sequence. Re-pin actual Local
HEAD after the focused review; the old source-ready baseline is not current proof.
Use [Windows first-run guide](../docs/validation/local-pc-first-run.md),
[progressive smoke](../docs/validation/efficient-local-testing.md) and
the [full prior acceptance sequence](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/f7fc983ead90964b40d99ec68c06213ab19e8215/planning/development.md#source-to-local-handoff--source_ready_for_local).

1. Admission: actual Windows x64 PC/agent, clean checkout and exact SHA;
   read-only DEV.cmd localcheck. Preserve App/Data; bootstrap pinned tools
   only if missing and approved.
2. Source: approved DEV.cmd preflight once, default Cargo features;
   no redundant setup/quick/check or agent suite. Stop at first failure,
   correct canonical owner and re-pin; retain private receipts.
3. Native prerequisite: independently review exact ABI v4 DLL pair and
   admit approved digests in reviewed_bootstrap_payload.rs. Ordinary
   dev/package/new launch held until then; no old or placeholder hashes.
4. Product smoke only after matching native admission: manager then
   MC01/MC02, progressive escalation, preserve slots/data, normal close
   and reconnect. Packaging, capture/recording, AI/input separate gates.
5. RPT and UPD acceptance during approved smoke: follow the exact prior
   acceptance sequence above for disposable roots, checkpoint/receipt
   preservation, ordinary official update observation, generation pins,
   version/ABI refusal, no unreviewed task invocation or forced update.
6. Feedback: exact SHA, first failing stage, sanitized observed versus
   expected, canonical owner. No private logs/data committed, automatic
   repair/replay or repeat of unaffected successful checks.

| Gate | Evidence owner |
| --- | --- |
| VAL-01/02 | [Windows first-run](../docs/validation/local-pc-first-run.md), [native CMake controls](../Development/tooling/acceptance/windows/bootstrap-ordering/README.md) |
| VAL-03 | [Progressive smoke](../docs/validation/efficient-local-testing.md): one then five sessions, identity/layout/close/reconnect/preservation |
| VAL-04 | [Data layout](../docs/system/data-layout.md): exact producer/root, reference stability and exclusive disposition |
| VAL-05 | [Native collectors](../Development/tooling/acceptance/windows/README.md): matched workload, FPS, CPU/RAM/GPU, handles and timings; OPT-07 held |
| VAL-06 | [Distribution](../Development/engine/distribution/README.md): installer bytes, signatures/legal notices, lifecycle |

All execution deferred. No CI/cloud substitution, release or version bump.
