# Developer tooling

The only active product is the Lazy Tester native Minecraft Education manager.
This directory contains source/structure checks, package preparation, Windows
native compatibility source, and retained historical evidence. Nothing here is
a second live game process/session owner.

| Path | Status |
| --- | --- |
| `repository/` | Active Git-tree, dependency, route, CI-disabled and documentation-link checks |
| `windows-toolchain/` | Source/build/package helpers, targeted frontend/Rust checks and opt-in two-pass timing; no automatic local test |
| [`developer/`](developer/README.md) | AI-facing `DEV.cmd` registry/inspection/planning and guarded dispatch; existing scripts remain execution owners |
| [`agent/`](agent/README.md) | Local stdio MCP, read-only visual planning, and OFF-by-default per-client native input candidate; not verified Minecraft gameplay |
| `storage/` | Read-only artifact inventory plus fail-closed retention planning for reviewed developer outputs; no deletion |
| `dependencies/` | Dependency consistency review and maintenance |
| `acceptance/windows/bootstrap-ordering/` | **Active native DLL source** used by the five-session game bootstrap |
| `acceptance/` | Native and historical acceptance collectors; older Sandboxie/VM results are not current gameplay proof |
| [`lazytester-particles/`](lazytester-particles/README.md) | Optional Particle Tuning v0.3 offline .mcpack builder; no automatic installation |
| [`lazytester-textures/`](lazytester-textures/README.md) | Optional block-only Texture Lite v0.1 offline .mcpack builder; entity/PBR extension retired |
| `acceptance/macos/lifecycle-fixture.swift` | Explicit synthetic AppKit launch/identity/normal-exit fixture; never launches Minecraft |
| `acceptance/macos/education-target.mjs` | Explicit offline Mac bundle metadata collector; no native launch or runtime admission |
| `acceptance/windows/education-target.ps1` | **Active** registered Minecraft Education executable resolver, moved from the retired isolation-source tree |
| `local-test/` | Optional source acceptance bookkeeping, not runtime evidence until executed |

Retired Sandboxie/VMware build and runtime source has left the active tree. The native process and per-profile controller remain the only active Minecraft launch authority. Historical notes remain in Git revisions.

## Developer disk inventory (read-only)

Use `DEV.cmd storage report` or `DEV.cmd storage explain` after bootstrap.
The Node CLI is `node tooling/storage/inventory.mjs report` from inside
`Development/`. It reports disjoint logical-byte categories with
`ABSENT`/`PARTIAL` markers and scans no profile/world/runtime data.
It does not delete or compact caches; first-time Rust profile changes may
temporarily increase target size before any separate retention review.
Synthetic tests: `node --test tooling/storage/inventory.test.mjs`.

`DEV.cmd storage plan` runs `storage/retention.mjs`, a second **read-only**
metadata review. It checks only the historical compiler-cache archive and
versioned Lazy Tester export candidates, preserving the newest three and
anything under 30 days. `REVIEW_ONLY` is never deletion permission;
malformed receipts and incomplete scans are `BLOCKED`. The planner does
not verify installer file hashes, perform cleanup or support `--apply`.
It is also available without PowerShell as
`node tooling/storage/retention.mjs` from `Development/`.
Synthetic tests: `node --test tooling/storage/retention.test.mjs`.

`storage/package-recycle.mjs` is the **only narrowly destructive developer
artifact owner**. It is called only by the successful native candidate
packager: preserve the newest two exports and, at most, delete eight older
verified three-file installer candidates after SHA-256 and path validation.
It never runs `cargo clean`, removes Cargo incremental artifacts, touches
private App backups, or exposes a generic delete CLI. A blocked scan is
reported, not bypassed. Synthetic tests:
`node --test tooling/storage/package-recycle.test.mjs`.

## Historical compiler cache retirement (explicit approval)

`storage/archive-retire.mjs` is an allowlisted backend for the AI task
`storage.retire-archive`. It uses the existing Cargo footprint inspector and
only permits a scoped, pinned `cargo clean --target-dir` against the retired
`workspace/archive/compiler-cache-20261026/target`, **never** the live
`workspace/builds/desktop/target`. The archive must have only recognized
Cargo layout, no links/hardlinks, a complete scan and an already populated
Windows-MSVC active target. Unknown or partial contents are blocked;
physical reclaimed bytes require Windows evidence. This task needs
`--approve storage.retire-archive` after explicit operator consent.
Synthetic tests: `node --test tooling/storage/archive-retire.test.mjs`.

## Cargo target attribution

`DEV.cmd storage cargo` uses the bounded read-only
`storage/cargo-footprint.mjs` scanner. It separates canonical versus
historical Cargo targets and provides debug/release and
deps/incremental/build size groups. It does **not** calculate NTFS
allocated bytes, delete data, run Cargo or authorize cache pruning.
Synthetic tests:
`node --test tooling/storage/cargo-footprint.test.mjs`.
The portable `DEV.cmd` environment rejects an ambient
`CARGO_TARGET_DIR` pointing outside the canonical target so new
compiler output is not silently split across extra directories.

## Build-check space evidence

`DEV.cmd bench:rust` uses the same `storage/cargo-footprint.mjs` owner
for read-only active-target measurements before and after each of its two
checks. The `--active-json` option exposes only schema, completeness
and aggregate bytes/counts. Partial scans make the disk delta `UNKNOWN`;
scanning is outside the benchmark stopwatch and never invokes Cargo or
deletes caches. `DEV.cmd` pins both Cargo target and Cargo intermediate
build-directory environment variables to the approved output root.
Direct Cargo from an unrelated shell is not governed by these settings.

## Resource-efficiency source previews — closed decision, reusable code

These **separate, explicit build tools are retained** for possible future
operator-approved testing; they are not runtime services and neither is
installed or activated by LazyTester. Their builders and dedicated fixtures
are outside the normal product npm/build/package and DEV.cmd task routes;
Python/Pillow is not a product prerequisite. The existing full developer
verification contract and pinned toolchain gate remain unchanged. The [current closed scope, proof limits,
and reopen gates](../../planning/development.md) override historical research.

From the repository root, optional **source-only synthetic checks** are:

```sh
python -m unittest discover -s Development/tooling/lazytester-textures -p 'test_*.py'
node --test Development/tooling/lazytester-particles/tune.test.mjs
```

The block checks require Python 3.12+, Pillow 10+ and Git; the particle checks
use Node.js. Follow each linked builder README for the exact explicit
`--source`/`--output` or `--official yes` procedure and original Mojang
sample version pins. Do not commit Mojang-derived output assets, merge the
separate packs, install packs in the original hosted world, or assume
individual Education profiles can enable them independently.

For a future request, **first pin the latest GitHub `Local` HEAD, read
`AGENTS.md`/`GITHUB_RULES.md` and the single planning handoff, then review
the existing builder owner and relevant tests**. Historical Git commits and
RAM studies remain references, not authorization to restart optimization.
Actual Minecraft/Windows compatibility and savings require new separate
approval and runtime evidence. No CI or monitoring is introduced here.

## macOS native artifact review (source prepared)

After a separately authorized Mac metadata phase, the existing Node toolchain
can inspect an explicit absolute path to an authorized, unchanged offline
`.app` copy with `node Development/tooling/acceptance/macos/education-target.mjs
"/absolute/path/Minecraft Education.app"` (one command from the checkout root).
Requires macOS standard tools and available `lipo` (Command Line Tools).
No install, mount, game launch, credential access or profile data scan occurs.
The collector requires the observed Education bundle ID `com.microsoft.minecraft-edu`
and executable `minecraft-edu`, not a filename-only match. These are metadata
expectations, not verified publisher pins. The report includes bundle metadata,
executable SHA-256 and architecture-specific
signature/entitlement summaries; it does not expose local paths or raw group
values. Empty entitlement output remains unknown. Codesign integrity is not a
Microsoft publisher pin, official DMG verification, loaded-image proof or
instance isolation. Use an unchanged offline copy; metadata checks are not
atomic against concurrent replacement. Refusal emits a generic JSON status.
The tool is not mounted in product, DEV.cmd, MCP or startup. See
[development handoff](../../planning/development.md). Source fixtures are
included in the existing native-feasibility test route, NOT_EXECUTED.

## macOS offline package review from Windows

Use only a copy of the [official Minecraft Education download](https://aka.ms/meeclientmacos).
The current reviewed candidate is 1.26.33.0; see the development handoff
for the exact published package hash and source revision. The archive is
not executable on Windows and no installation or native test is needed.

1. Record the original DMG filename and SHA-256 using PowerShell
   `Get-FileHash -LiteralPath 'C:\path\Minecraft_Education_1.26.33.0.dmg' -Algorithm SHA256`
   (replace the example path with the actual file). Do not confuse the
   archive hash with a hash of an extracted executable.
2. If [7-Zip](https://www.7-zip.org/) is already available or separately
   chosen by the operator, open the DMG as an archive. It supports DMG,
   HFS and APFS; this specific archive has not been inspected. If a nested
   filesystem image is present, browse it as an archive. Extract only
   `minecraft-edu.app/Contents/Info.plist` (actual bundle name may differ)
   into a separate disposable directory; do not mount or execute content.
3. Supply the hash, original package filename and unchanged Info.plist
   as the first small evidence set. The main executable name will be
   derived from CFBundleExecutable, not guessed. Only if later needed,
   extract that exact file separately for offline Mach-O metadata review.
   Do not upload the whole package if too large or commit proprietary
   bundle bytes, game files, accounts or worlds to Git.

The operator can instead supply an unchanged authorized DMG copy if it
fits the available file transfer. Unknown archive layout or failed
extraction remains blocked; do not install random extractors or repair
the package. This is static artifact intake, not localtest or native
verification. The existing Mac artifact collector remains the owner
of later codesign checks; no new runtime or validation route.

### Small Mach-O metadata report without uploading the executable

When full-file transfer fails, reuse the existing Node 22+ toolchain and
the unchanged extracted `minecraft-edu` executable. Download the
[existing collector](acceptance/macos/education-target.mjs) if the
repository is not checked out. From PowerShell, with both files in the
current folder:

```powershell
node .\education-target.mjs --mach-o "$PWD\minecraft-edu" | Set-Content -Encoding UTF8 .\minecraft-metadata.json
```

Send only minecraft-metadata.json. This is explicit static artifact intake,
not localtest. The collector reads one fixed executable basename with
bounded random reads, hashes through small buffers, and reports declared
64-bit Intel/ARM slices, embedded CodeDirectory identity/team comparisons,
flags, CMS/DER blob presence, selected XML entitlement-key indications
and linked-library name indications. It does not disclose paths, raw
entitlements or certificates, run game code, install anything or validate
signatures. Unsupported layouts refuse. XML key indications are lexical,
not semantic/effective values; missing linked-library hints do not rule
out static or dynamic loading. No hard guarantee against concurrent
replacement. Source fixtures authored NOT_EXECUTED. The existing Mac
codesign route remains unchanged and required for actual signature proof.

## macOS synthetic lifecycle fixture (source prepared)

`acceptance/macos/lifecycle-fixture.swift` is a separately compiled, opt-in
acceptance program for a future approved Mac phase. It builds no game
adapter and is not part of the desktop, DEV.cmd, startup, MCP or package.
The fixture accepts no target path and refuses root. It supports no arguments
for the original single-instance lifecycle or the explicit `--profile-probe`
option for two synthetic instances of the same bundle, or the explicit
`--workspace-probe` mode for two synthetic visible windows. It copies
its own compiled executable into a fresh private temporary application bundle,
uses AppKit to request a new application, retains the original running-app
object, verifies bundle/executable identity and requests normal termination.
It waits for observed exit; timeout/refusal preserves the application and
fixture directory, with no force close, retries, adoption or recursive deletion.
Use the existing macOS developer tools, not a new runtime dependency.
A future authorized local agent may compile with `swiftc -swift-version 5
-parse-as-library` and an explicit private output path, then run the resulting
executable without arguments or with the explicit `--profile-probe` option.
The pair uses two fresh fixture-only HOME/TMPDIR directories and reads bounded
child receipts for environment HOME, Foundation home, Application Support,
Caches and temporary path resolution. No real preference/Keychain contents are
read or written. One normal close must leave the other original app alive.
Both observed exits and path results are synthetic only; shared or unresolved
paths are reported as a non-success status. Only booleans leave the parent
report, not local paths or PIDs. Preferences/authentication remain NOT_OBSERVED.
The workspace mode requires macOS 14+ and already-authorized screen capture
permission; it does not request or grant permission. It opens only its own
two windows and samples five interior pixels of each exact owned window with
ScreenCaptureKit. The operator manually clicks A then B; the expected color
changes in the chosen window only. No desktop capture, images written to disk,
input injection, Accessibility access or cross-user transport. Three pairs of
sequential screenshots and two bounded manual waits are not live simultaneous
preview or shared interactive workspace proof. Normal close A preserves B,
then close B. Only scoped statuses are emitted, always runtimeAdmission=false.
The explicit `--shared-workspace-probe` mode reuses those two windows and
adds one native AppKit panel with a selected snapshot and two preview tiles.
The operator clicks Select A/B in the panel, then physically clicks the
focused original window. AppKit activation is requested once and the exact
frontmost application must be observed. Captures remain bounded sequential
snapshots, not live streams; only the two latest preview images are displayed
with the selected image reused in the viewport. The panel is not an embedded
game window and does not send input events or access another user session.
No screenshots are persisted. Any unconfirmed selection, focus or marker
refuses; existing single/profile/workspace modes remain available.
The explicit `--preferences-probe` mode reuses the same synthetic bundle,
HOME/TMPDIR profile roots, exact launch/close helpers and receipt owner.
A writes a marker to a fresh UUID bundle's current-user/any-host
CFPreferences domain, B observes before writing its own marker, then A
and B each close and reopen while the other retained app remains live.
Reports contain only path and marker comparison booleans. The mode may
leave a small UUID-domain synthetic preference in the actual user's
Library if HOME redirection is ignored; no automatic deletion, no
existing preference domain, game, token or Keychain access. A positive
result applies only to this synthetic domain; authentication, game
storage, native workspace and runtime admission remain unverified.
This is future held acceptance source, not a localtest request.

The explicit `--fixed-home-probe` variant reuses the preference probe's
entire lifecycle and comparison. It additionally sets the process-local
`CFFIXED_USER_HOME` to each synthetic profile root. Historical public
CoreFoundation source prioritizes this value before the account home;
modern macOS behavior is not established, and this is not a supported
Minecraft profile parameter or a security boundary. The receipt labels
the recipe and reports whether the variable was present/matched on both
launches, together with the original path and preference checks. A
positive fixed-home result requires every private path, actual environment
match and independent preference persistence. The baseline remains
available separately; each invocation uses a fresh synthetic domain.
No daemon override, global environment change, game launch or credential
access. No additional product mode. NOT_EXECUTED.

The explicit `--rendezvous-probe` mode is a separate one-shot synthetic
cross-user feasibility witness in the same source file. It requires two
already-consented, distinct standard users in graphical login sessions,
both manually launching the fixture with `listen` or `connect`, the same
pre-provisioned disposable canonical directory under `/Users/Shared/`,
and the other user's expected numeric UID. The directory must belong to
the listener, disallow group/other writes, and permit only intended
cooperating access (including partner traversal/socket access); review
ACLs explicitly before any future execution. The source does not create
users/directories or alter ACLs. It binds one fresh fixed socket leaf,
uses nonblocking bounded I/O, reciprocal kernel getpeereid comparisons,
and exchanges only eight bytes per side (marker and reported security
session ID), verifying distinct graphical security sessions through
SessionGetInfo. The session report is NOT kernel-bound to the peer UID;
code identity, per-message sender, virtual desktop, game ownership,
capture, input and full concurrency remain NOT_VERIFIED. No credentials,
registration, TCP, retry, game, or permissions request. One 120-second
total deadline per invocation; socket leaf is retained for operator
cleanup, never blindly removed. Do not use this as product transport.
No execution requested during source phase.
All modes are source-authored and NOT_EXECUTED.
The temporary fixture bundle is retained and
may be removed by that operator after confirming exit. The result is scoped
to a synthetic app, never game isolation or gameplay proof. Source not compiled
or executed; main-run-loop behavior, bundle launch and exit remain unverified.
