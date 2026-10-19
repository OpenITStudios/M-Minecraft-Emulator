# Archived Sandboxie v1.18.5 source investigations

> **HISTORICAL / NON-EXECUTABLE GUIDE.** These notes come from the former
> `tooling/isolation-source/README.md` before the current v1.18.6
> source-only build route. The commands, hard-coded D: paths, test counts
> and deployment claims below are historical snapshots, NOT operator instructions.
> The sole current build path is
> [v1.18.6 isolation owner](../../tooling/isolation-source/README.md).

# Historical pinned v1.18.5 isolation baseline (superseded)

**Purpose:** reproduce Sandboxie-Plus **v1.18.5** core build inputs and record source/artifact provenance for the planned personal, educational/noncommercial **M-Isolation-Engine**. This is **not** an app installation routine, driver signing tool, fork repository, or migration command. The active Minecraft manager continues using official Sandboxie with the existing account-bearing box unchanged.

## Full-source import path (owned Git subtree, not a wrapper)

The new [owned source import procedure](../../docs/system/owned-source-import.md)
copies **every tracked upstream Sandboxie v1.18.5 file** from the exact pinned
Git seed into a writable `engine/isolation-owned/` directory of this
application. It then applies the existing native 11-file MIVUBI patch directly
to the copied source in a separate normal `Local` commit. Its four
disposable-Git integration tests passed; the real 1,624-file transfer has
not run on GitHub because unrelated repository tree objects cannot be
linked by the available GitHub API. Neither script modifies the official
installed Sandboxie or claims a compiled/verified Windows driver.

## What is pinned

- Full upstream repository: <https://github.com/sandboxie-plus/Sandboxie>
- Tag: `v1.18.5` → commit `abd4c240031cfb46666daa74ca47ed4e48b561e4`
- Main source contracts: `Sandboxie/{SandboxDll.sln,Sandbox.sln,SandboxDrv.sln}`, `Installer/copy_build.cmd`, `Sandboxie/ReadMe.md`, upstream `.github/workflows/main.yml`, `LICENSE.Plus`, `Sandboxie/COPYING`.
- File/build contract: [upstream-baseline.json](../../tooling/isolation-source/upstream-baseline.json). One source of truth for required artifacts and stage order.
- Read-only pre-build checkout verification: [verify-checkout.mjs](../../tooling/isolation-source/verify-checkout.mjs) and [six Git tests](../../tooling/isolation-source/verify-checkout.test.mjs). It rejects changed upstream remotes, shallow clones, missing/retargeted tags, detached/wrong branches, untracked modifications and symlinked checkout paths; a SHA string alone is not enough.
- Metadata validator: [validate-baseline.mjs](../../tooling/isolation-source/validate-baseline.mjs) with [six tests](../../tooling/isolation-source/validate-baseline.test.mjs). Checks source revision, stage order and report consistency.
- File-byte verifier: [verify-artifact-files.mjs](../../tooling/isolation-source/verify-artifact-files.mjs) with [five tests](../../tooling/isolation-source/verify-artifact-files.test.mjs). Independently rehashes every `PRESENT` file and confirms every `MISSING` file is absent; refuses symlinked and dangling-symlink file paths and changed files. Read-only; still **not** proof of signing, driver load or Minecraft gameplay. PowerShell invokes both verifiers after collection; Node.js is required.

## Pinned upstream source inside the application repository

The `Local` Git tree now contains a real Git submodule at `engine/isolation-upstream/Sandboxie`, pointing to the upstream Sandboxie **v1.18.5** commit `abd4c240031cfb46666daa74ca47ed4e48b561e4`. The actual upstream history remains in that Git repository, rather than copying kernel source into Rust or committing a ZIP. This is a **pinned source reference**, not a new MIVUBI GitHub fork and not a compiled engine.

For a checkout with GitHub access, the source-to-build route is:

~~~powershell
# From the M-Minecraft-Emulator repository root (Local):
git submodule update --init -- engine/isolation-upstream/Sandboxie

# Make a SEPARATE full-history checkout with a modifiable development branch.
# The target's parent must exist; the target itself must be absent.
node tooling/isolation-source/bootstrap-fork.mjs --destination 'D:\Source\M-Isolation-Engine' --local-seed 'engine/isolation-upstream/Sandboxie'

# Inspect the standalone, unmodified source before the Windows build.
node tooling/isolation-source/verify-checkout.mjs --source-root 'D:\Source\M-Isolation-Engine'
~~~

The local-seed mode checks the seed's real directory, remote URL, pinned Git SHA/tag, full history and clean working tree. It creates a full, independent Git checkout and sets its `upstream` remote back to the official public repository, without a second network clone. Eight disposable-Git tests passed in Linux Node 22; **the real submodule materialization and native Windows build remain NOT_EXECUTED here** because this runtime cannot resolve github.com through Git.

## Independent source checkout — full history preserved

The new bootstrap tool creates a complete, non-shallow Git checkout **outside** the application source and points an independent working branch at the pinned v1.18.5 commit. It verifies the tag, commit, clean working tree and history before promoting a temporary checkout into the chosen empty destination. It does **not** publish a GitHub fork or install a driver.

From the M-Minecraft-Emulator repository root, where Git and Node.js are available:

~~~powershell
node tooling/isolation-source/bootstrap-fork.mjs --destination 'D:\Source\M-Isolation-Engine'
~~~

The parent directory must already exist, and the requested destination must not exist yet. A failed clone refuses a partial result, cleans only its own temporary staging folder and leaves existing directories alone. It does not invoke game tools or touch `MinecraftClient02`. An upstream network connection is required for a real run; only the Git-local tests have executed here.

Equivalent manual full-history checkout (alternative operator route, **not executed by ChatGPT**):

~~~powershell
git clone https://github.com/sandboxie-plus/Sandboxie.git M-Isolation-Engine
cd M-Isolation-Engine
git switch --create isolation/baseline-v1.18.5 abd4c240031cfb46666daa74ca47ed4e48b561e4
git remote rename origin upstream
git rev-parse HEAD
git status --porcelain=v1 --untracked-files=all
git remote -v
~~~

Eight local Git regression tests are included in [bootstrap-fork.test.mjs](../../tooling/isolation-source/bootstrap-fork.test.mjs); they cover full-history cloning, wrong pin/tag refusal, existing destination preservation, dangling-symlink refusal, local pinned seed verification, wrong seed remote and dirty source using disposable repositories. Tests do not download Sandboxie.

**Do not** use GitHub's "Download ZIP" as your eventual fork provenance: it loses Git history. Creating a **new GitHub fork** requires a repository-fork operation not available through this session's connected GitHub actions. Do not claim that a fork has been published or that a local independent checkout is already a GitHub fork. A later approved remote fork must preserve the upstream history and its notices; pin this tag as the base of its working branch. Never delete/rename upstream GPL license notices.

## First build (unchanged upstream; not installed)

On a Windows build environment with Visual Studio 2022 C++, MSVC v143, MFC, Windows SDK, a compatible WDK extension, Git and PowerShell, execute the following from the **M-Minecraft-Emulator** checkout:

~~~powershell
# Collection-only check; no compilation or driver operations:
.\tooling\isolation-source\build-baseline.ps1 -SourceRoot 'D:\Source\M-Isolation-Engine' -ReportPath '.\workspace\isolation-baseline-inspection.json'

# Optional later build: a NEW report path, never overwriting the inspection.
# Run only after the required Windows toolchain is independently available.
.\tooling\isolation-source\build-baseline.ps1 -SourceRoot 'D:\Source\M-Isolation-Engine' -ReportPath '.\workspace\isolation-baseline-build.json' -Build
~~~

For source-only verification on any platform, run `npm run isolation-source:verify` and `npm run test:isolation-source` from the app repository root. To check an existing independent checkout before the Windows build, run `node tooling/isolation-source/verify-checkout.mjs --source-root 'D:\Source\M-Isolation-Engine'`. The PowerShell build procedure now requires that exact verification, too. These checks are read-only and have no build or Windows dependencies. The real file-byte verifier is invoked during `build-baseline.ps1` on the actual Windows checkout after artifact collection; metadata validation alone is insufficient.

A clean source tree with HEAD and tag identical to the manifest is mandatory. **Build mode also refuses pre-existing required binaries or symbols**, preventing stale output from being attributed to the new build; it never deletes them. Use a separate clean source checkout for another build. The script refuses modified source or an evidence report inside the upstream source checkout. It invokes upstream's **Win32 DLL/service → x64 core → x64 driver** MSBuild stages in the order of its v1.18.5 CI, logs each stage locally, hashes expected files, and records their signatures. It **never installs, starts, stops or replaces a service/driver**, launches Minecraft, reads account tokens, or copies a box. Unmodified locally compiled kernel binaries are normally **not** production-signed or guaranteed loadable. Report status `BUILD_ARTIFACTS_COLLECTED` means build outputs were found after these commands; it does **not** imply a loadable signed driver, working Sandboxie, successful Microsoft login or Minecraft gameplay.

The companion Windows x86 binaries `SbieSvc.exe`/`SbieDll.dll` matter even for x64 packaging; `LowLevel` is embedded by upstream project resources. The source tree includes upstream Classic UI and optional helpers because the first build must verify **complete dependency closure before removing any targets**. Required artifact inventory is based on `Installer/copy_build.cmd`; missing optional PDBs do not masquerade as a successful debug-symbol capture.

## Private binary payload assembly (no installation)

The new [assemble-engine.mjs](../../tooling/isolation-source/assemble-engine.mjs) assembles a **private, unmodified upstream engine candidate** only after the successful pinned Win32/x64 build and full file-byte checks. Its structure follows upstream runtime packaging: x64 driver/service/DLL and helpers in the root, 32-bit companions in `32/`, required templates/manifests and upstream copyright/licenses under `LICENSES/`, optional PDB symbols under `_debug/`. Each copied artifact is SHA-256 checked again and recorded in `engine-provenance.json`.

Alternative to the preceding `-Build` command: perform **one** clean
build with staging in a fresh checkout. Do not run both `-Build` examples
sequentially against the same source tree; the second must refuse the
previous binary/PDB outputs. Use new report, log and destination paths
for each separate build. Optional Windows build-and-stage command,
**NOT_EXECUTED here**:

~~~powershell
.\tooling\isolation-source\build-baseline.ps1 -SourceRoot 'D:\Source\M-Isolation-Engine' -ReportPath '.\workspace\isolation-baseline-build.json' -Build -StageOutput 'D:\Source\M-Isolation-Payload'
~~~

The destination's parent must exist and the destination must not exist yet. The result is labeled `ARTIFACTS_STAGED_NOT_INSTALLABLE`: it is **not** a signed, installed, loadable or gameplay-tested independent isolation engine. Seven executable assembly tests passed with disposable fixture binaries and injected source/byte-verification dependencies. Actual upstream build, staging of genuine binaries, signing and Minecraft acceptance remain NOT_EXECUTED.

## Source-first disposable Windows driver/service transaction

The [provision-disposable.mjs](../../tooling/isolation-source/provision-disposable.mjs) module now defines an **offline source-only** provisioning and rollback contract. It uses the pinned upstream's actual `KmdUtil` semantics (`SbieDrv` kernel minifilter, altitude `86900`; `SbieSvc` user service, `UIGroup`) and produces seven sequenced installation checks/actions and five reverse-order rollback checks/actions. It requires an empty `SbieSvc`/`SbieDrv` namespace in an interactive session, a correct noninstallable payload manifest and an operator-declared dedicated disposable VM without accounts or official Sandboxie.

The CLI can generate a plan without touching Windows services:

~~~text
node tooling/isolation-source/plan-disposable.mjs --payload <engine-provenance.json> --host <read-only-host.json> --target <disposable-target.json>
~~~

**Important:** the output always declares `SOURCE_PLAN_ONLY_DRIVER_INSTALL_BLOCKED`, `installationPermitted=false`, and `codeSignatureVerified=false`. Merely declaring a VM disposable does not attest that it is one. The script cannot load/start/delete services, does not turn off driver signing, and is deliberately disconnected from the active Tauri/Client 02 routes. An actual privileged Windows executor will be a separate later source batch with signed artifact verification and ownership-protected rollback. The 9 pure transaction tests have run in the available V8 context; full Node24/PowerShell/kernel execution remains NOT_EXECUTED.

Upstream anchors used for correct semantics: `Sandboxie/common/my_version.h` (driver/service names and `FILTER_ALTITUDE=86900`), `Sandboxie/install/SandboxieVS.nsi` (KmdUtil install/start/delete ordering), and `Sandboxie/install/kmdutil/kmdutil.c` (supported options), all from pinned tag v1.18.5.

## Durable source-first transaction executor and Windows preflight

The transaction plan can now be exercised in an **exclusive simulation journal** using `execute-disposable.mjs` and `simulate-disposable.mjs`. This is executable, tested source for ordering, write-ahead intent, unknown/failed results, and protection against retry/replay. It only accepts an `IN_MEMORY_SIMULATION` adapter. There is no channel from this module to the active desktop process, installer, or `kmdutil.exe`.

Example after a plan has been created:

~~~text
node tooling/isolation-source/simulate-disposable.mjs --plan <plan.json> --journal-directory <brand-new-local-directory>
~~~

The journal directory must not exist beforehand; each intent is stored as `UNKNOWN` before a simulated stage runs. A failed/ambiguous stage stops without retry, and later manual reconciliation must determine whether anything occurred. The simulation NEVER reports a loaded driver or an installed provider. Source-level tests and execution evidence for this module are separately recorded in the development handoff.

For Windows-only read-only preflight, `tooling/isolation-source/windows/probe-disposable-readiness.ps1 -PayloadRoot <candidate-folder>` checks registration names, signatures, and heuristic virtualization metadata. It **does not** prove a dedicated disposable host, source owner, supported signer, production signing, kernel loadability or game readiness. Every response has `installationPermitted=false`. Do not feed it into a privileged executor as authorization.

**Still not implemented:** a privileged, host-attested Windows SCM/KmdUtil transport and service-ownership-proven rollback. Those must be written and reviewed as separate source that cannot target the existing `MinecraftClient02` host without independent safety evidence.

## Windows KmdUtil transport interface (source-first)

The source now includes [transport-commands.mjs](../../tooling/isolation-source/windows/transport-commands.mjs) for deterministic **argv compilation** from the existing seven-stage v1.18.5 plan, including `SbieDrv` kernel minifilter `type=kernel start=demand altitude=86900`, the `SbieSvc` user-mode service, and stop/delete rollback commands. Rollback descriptors require a claim of an owned transaction receipt. **These claims must still be independently verified by Windows SCM image-path and journal signatures; the compiler alone cannot prove ownership.**

The complementary [kmdutil-transport.ps1](../../tooling/isolation-source/windows/kmdutil-transport.ps1) supports a **read-only Probe mode**. It revalidates exact argument sequences, image hash, Authenticode metadata, and SCM registration. Its `Apply` mode is deliberately sealed until an independently approved lab-attestation signer and service ownership validation are implemented. At present there is **no enabled privileged transport or executable driver installation**. The PowerShell source is not connected to Tauri, the active installer or any auto-update route.

Seven argument/rollback/unsafe-source scenarios passed against the exact GitHub JS source in the available V8 test context. PowerShell parsing and Windows command semantics have NOT_EXECUTED status. The upstream command semantics were crosschecked against pinned `Sandboxie/install/SandboxieVS.nsi`, `Sandboxie/install/kmdutil/kmdutil.c`, and `Sandboxie/common/my_version.h`.

Do not confuse a parsed/validated command with approval to use it. Every emitted transport specification includes `executionAllowed=false`, `driverSigningVerified=false`, `disposableHostAttested=false`, and `rollbackAuthorized=false`. No code automatically deletes a registered Windows service.

## Signed disposable-lab attestation and Windows service image ownership

The source now contains [attestation-gate.mjs](../../tooling/isolation-source/windows/attestation-gate.mjs), [service-ownership.mjs](../../tooling/isolation-source/windows/service-ownership.mjs) and the read-only [probe-owned-service-image.ps1](../../tooling/isolation-source/windows/probe-owned-service-image.ps1). Attestations use **Ed25519** signatures over exact bounded JSON bytes and expire within a ten-minute window. Signed claims are bound to an individual machine-fingerprint SHA-256, VM snapshot UUID, one transaction and the precise plan/journal hashes.

The lab trust public key is intentionally **NOT configured**. A caller cannot provide an arbitrary certificate key to the production API; test-only source functions verify ephemeral keys without exposing private signing material in Git. Even a successful test signature and matching SCM ImagePath return *correlation only*, not independent Windows disposable-host attestation, service ownership, driver-signing approval or permission to delete/stop a service.

The KmdUtil command compiler no longer accepts merely self-reported rollback booleans as sufficient to emit STOP/DELETE descriptors. `kmdutil-transport.ps1 -Mode Probe` also reports the observed SCM path and whether it exactly matches the expected binary root (still untrusted as ownership proof). `Apply` remains sealed.

To collect a read-only Windows service-image observation (source written, Windows execution **NOT_EXECUTED**):

~~~powershell
.\tooling\isolation-source\windows\probe-owned-service-image.ps1 -ServiceName SbieSvc -ExpectedImagePath 'D:\Lab\M-Isolation-Payload\SbieSvc.exe'
~~~

Seven Node tests for Ed25519 evidence and six for ownership correlation are authored. The KmdUtil compiler's seven pure scenarios and eight ownership-logic scenarios were checked against exact source blobs in the available JavaScript runtime; full Node cryptographic test, PowerShell Windows execution and actual signer provisioning remain NOT_EXECUTED.

## Durable operator review and service ownership ledger

The same journal now supports **reopening an existing transaction for review** without restarting an installation or an earlier simulation. `execute-disposable.mjs` rejects removal/rewriting of previous provisioning steps and SCM observations. `windows/ownership-ledger.mjs` binds an explicit operator acknowledgment to the current transaction and appends at most two read-only SCM observations with SHA-256-linked entries.

Use the developer-only CLI `windows/review-ownership.mjs` to inspect a journal:

~~~text
node tooling/isolation-source/windows/review-ownership.mjs inspect <plan.json> <existing-journal-directory>
~~~

To explicitly record a **review-only** acknowledgment:

~~~text
node tooling/isolation-source/windows/review-ownership.mjs acknowledge <plan.json> <existing-journal-directory> <transaction-id> "REVIEW ONLY - NEVER INSTALL STOP OR DELETE"
~~~

To record a read-only SCM result:

~~~text
node tooling/isolation-source/windows/review-ownership.mjs observe <plan.json> <existing-journal-directory> SbieSvc <read-only-scm.json>
~~~

The CLI does not call KmdUtil, Windows SCM, or the active Tauri runtime. Its operator acknowledgment is not authorization; the event hash chain is not a digital signature. Every operation remains blocked and no service can be stopped/deleted from these records. An additional ten Node filesystem test cases are authored and V8 pure state/chain checks have been exercised; Windows/Node24 integrated validation remains NOT_EXECUTED.

## Source-first SCM and signed lab reconciliation

The [reconcile-ownership.mjs](../../tooling/isolation-source/windows/reconcile-ownership.mjs) source connects three existing owners: the **actual canonical journal file** (from `execute-disposable.mjs`), its `ownership-ledger.mjs` hash-chain records, and the current Windows SCM observation produced by `probe-owned-service-image.ps1`. Both `SbieDrv` and `SbieSvc` must independently match pinned package image paths, SHA-256 values, observed signatures and one hashed host identifier.

The new developer-only read-only CLI accepts an existing plan and journal with both SCM observations:

~~~text
node tooling/isolation-source/windows/reconcile-ownership-cli.mjs --plan <plan.json> --journal-directory <existing-journal-directory> --payload-root <Windows-payload-root> --driver-scm <driver-observation.json> --service-scm <service-observation.json>
~~~

The output is strictly `LOCAL_EVIDENCE_CORRELATED_NOT_AUTHORIZED` when all local evidence matches. It includes the SHA-256 of the **persisted journal bytes**, so a separately signed lab statement could bind exactly that file. The separate programmatic signed reconciliation function uses only the **source-pinned** Ed25519 key, which is not yet configured; it therefore currently refuses every production signature. These are not installation/rollback interfaces, and even verified local evidence cannot stop or delete Windows services.

Seven new Node 22 filesystem tests were executed with lightweight stand-ins for upstream dependency modules and the exact new reconciliation module bytes (7/7 PASS); the full repository version with actual dependencies remains NOT_EXECUTED on this revision. Neither an existing standalone upstream source checkout nor signed-driver/Windows service/gameplay acceptance is proven.

## Read-only rollback reconciliation report

The existing `windows/reconcile-ownership.mjs` now also produces a **non-authorizing rollback report**. Unlike strict two-service correlation, it diagnoses partial registrations, missing or unknown SCM results, image/path drift, absent creation steps, and ambiguous simulated operations. It checks the existing operator/ownership ledger, exact pinned binary hashes, the host fingerprint and the actual bytes of the canonical transaction journal. No additional lifecycle database or service controller was created.

The existing CLI supports `--rollback-report`:

~~~text
node tooling/isolation-source/windows/reconcile-ownership-cli.mjs --rollback-report --plan <plan.json> --journal-directory <existing-journal-directory> --payload-root <Windows-payload-root> --driver-scm <read-only-driver.json> --service-scm <read-only-service.json>
~~~

A result such as `MANUAL_REVIEW_UNATTRIBUTED_SERVICE_NAME`, `MANUAL_REVIEW_AMBIGUOUS_CREATION_RECORD`, or `MATCHED_SIMULATION_ONLY_NOT_SERVICE_OWNERSHIP` is diagnostic information—not proof that the service belongs to MIVUBI. Every result has `canInstallDriver=false`, `canStopService=false`, `canDeleteService=false`, and `rollbackPermitted=false`. No KmdUtil, SCM mutation, or Tauri path is invoked.

Eight pure JavaScript decision scenarios were executed against the new GitHub source blob in the available V8 runtime. Seven additional Node integration tests were authored but **NOT_EXECUTED** with actual filesystem, PowerShell or Minecraft processes in this revision.

## Private fork source patch and native build

The [engine source patch](../../engine/isolation-native/fork-overlay/apply-private-source.mjs)
and separate [private core build script](../../engine/isolation-native/fork-overlay/build-private.ps1)
now support a MIVUBI-specific source checkout that retains the pinned upstream
kernel/service/DLL ABI and ensures the seven selected source owners are patched
together. Only a **separate** Git checkout may be changed; the upstream Gitlink
and running official Sandboxie installation are never modified.

This is an **exclusive** private engine route: it does not attempt to
coinstall alongside another `SbieDrv`/`SbieSvc` instance. ABI probe CMake
now accepts `-DMIVUBI_PRIVATE_FORK_ROOT=<patched checkout>` and refuses a
mismatched source marker or missing upstream ancestry. The private MSBuild
route creates a new source-specific report rather than asserting the modified
fork binaries are unmodified v1.18.5. No signed driver or deployed runtime
exists yet. See [native README](../../engine/isolation-native/README.md).

## Private fork runtime package handoff

After the **exclusive** private core build succeeds, the same payload layout owner
(`assemble-engine.mjs`) now supports an explicitly separate private-build route
through [assemble-private-engine.mjs](../../tooling/isolation-source/assemble-private-engine.mjs). This does
**not** mislabel modified fork binaries as an unmodified upstream build.
It validates the exact seven source changes and Git ancestry, stage ordering,
source-specific build report, actual binary SHA-256/size, upstream resource files
and copyright/licenses before staging an atomic, new-folder payload.

Run the private build and request staging explicitly with:

~~~powershell
.\engine\isolation-native\fork-overlay\build-private.ps1 -ForkRoot 'D:\Source\M-Isolation-Engine' -ReportPath '.\workspace\private-build.json' -StageOutput 'D:\Source\M-Isolation-Payload'
~~~

Or stage an existing genuine private build report without recompilation:

~~~powershell
node tooling/isolation-source/assemble-private-engine.mjs --fork-root 'D:\Source\M-Isolation-Engine' --manifest tooling/isolation-source/upstream-baseline.json --private-report '.\workspace\private-build.json' --output 'D:\Source\M-Isolation-Payload'
~~~

The result uses `PRIVATE_CORE_STAGED_NOT_INSTALLABLE`, contains
`privateCoreBuild=true`, the fork HEAD and the seven source owners,
and explicitly sets `canCoexistWithOfficialSandboxie=false`. The existing
Rust read-only candidate inspector accepts this provenance only when its
full file SHA-256 checks and private marker expectations match.
It returns the **same NOT_INSTALLABLE status** to the UI; no provider
switch, service command, driver loading or account migration occurs.

Seven private-assembly Node tests are authored; the source-level private
report acceptance/rejection contract was evaluated in V8 (8/8 cases),
but the complete file-copy and Rust test suites are NOT_EXECUTED for
the new revision. Windows build, driver signing and gameplay remain unproven.

## Product separation, attribution and acceptance

Raw outputs remain in the upstream checkout; a private candidate may be staged into an explicitly requested separate folder. Neither is committed to `Local`, merged into Rust, or installed under the existing `M-Bedrock Virtual Clients` identity. Do not try a custom driver next to the official Sandboxie service/driver in the same account-bearing Windows installation: device/service/IPC namespaces may collide. The future cutover needs a dedicated disposable Windows environment, signing and crash-attribution evidence, game compatibility, persist/reboot and rollback tests, plus a new license/distribution decision. A fork is **deployment independence** once proved, not source independence from Sandboxie.

Source references and license findings: [inventory](../../docs/validation/owned-isolation-upstream-inventory.md); product isolation/rollback design: [owned isolation design](../../docs/system/owned-isolation-design.md). Store local logs/manifests under ignored `workspace/`; never commit credentials, Minecraft worlds, machine-specific paths or raw dumps.

**Current evidence ceiling (2026-10-14):** GitHub accepted a pinned upstream v1.18.5 Gitlink. Bootstrap/local-seed tests 8/8 PASS, and private-payload assembly tests 7/7 PASS on disposable fixtures using Node22. Real upstream source checkout, Windows driver build, payload staging from genuine binaries, driver loading and Minecraft gameplay are NOT_EXECUTED.

## Committed private-fork source identity — mandatory before compilation

The first seven-file overlay is intentionally **staged but not committed**.
Before running `build-private.ps1`, review and commit it in the **separate**
fork checkout, never in this app repository:

~~~powershell
git -C 'D:\Source\M-Isolation-Engine' diff --cached --check
git -C 'D:\Source\M-Isolation-Engine' diff --cached --stat
git -C 'D:\Source\M-Isolation-Engine' commit -m "build: mark MIVUBI private isolation core ABI"
node engine/isolation-native/fork-overlay/apply-private-source.mjs --verify-committed --fork-root 'D:\Source\M-Isolation-Engine'
~~~

The canonical source verifier requires one clean direct-child commit from
upstream v1.18.5, exactly seven source files, and exact expected modified blob
bytes. The source identity is carried as Git HEAD, Git tree and SHA-256 patch
digest through the Windows build report, staging assembler and read-only Rust
candidate inspector. A marker-only match or HEAD still equal to upstream is
refused. These checks never touch installed Sandboxie, Microsoft accounts or
Minecraft worlds; no Windows kernel driver is installed or started.

## Complete upstream artifact manifest (2026-10-14)

`validate-baseline.mjs` now validates the FULL pinned v1.18.5 inventory,
not just the five core DLL/service/driver files. The version-locked closure
has 16 required binaries (including helpers), nine optional symbol paths,
five source resources and three source license files. It rejects removal,
reordering and injection in any group. This is used by the existing
unmodified baseline builder, the private build's PRE-MSBuild source check
and the private staging assembler. A reduced manifest can no longer produce
a misleading complete binary report.

An upstream update requires reviewing the upstream pin, manifest, complete
validator closure, consumer payload map, tests and licensing together.
Optional symbol files may remain absent, but their declared locations are
immutable for the pinned version. This is only source/build provenance:
compiler output, signing and native driver functionality remain untested.

## Baseline/private evidence preservation (2026-10-14)

Before launching MSBuild, both Windows source build scripts now refuse
pre-existing required binaries **and optional PDB symbols**. Existing
`baseline-*.log` / `private-*.log`, existing JSON report paths and existing
payload destinations are refused before the expensive compilation begins.
Log and report writes additionally use PowerShell `Out-File -NoClobber`.
Staging to an ancestor/descendant of the source checkout is rejected.
The builder does **not** delete or overwrite a previous run to make a new
attempt appear clean.

This protects previous diagnostic evidence and reduces wasted compilation,
but is not an atomic cross-process reservation or a substitute for testing
against an actual Windows filesystem. Native MSBuild/PowerShell execution
remains NOT_EXECUTED from the ChatGPT/GitHub context.

## MIVUBI private core DLL/service startup identity — 15 October 2026

The current private-fork overlay modifies **eight** pinned source files,
now including `Sandboxie/core/svc/DriverAssistStart.cpp` in addition to
the existing `DriverAssist.cpp`. The private kernel version marker is
checked by the existing `SbieApi_GetVersionEx` DLL path even for callers
passing `NULL` output pointers. The service's asynchronous driver startup
rejects a specific foreign-driver revision mismatch *before* any
`NtLoadDriver` attempt; absence of a driver follows the original
Windows startup route. The existing 32-bit/64-bit private build and
assembler use the new exact eight-file source contract, matching the
Rust read-only inspector.

This behavior is source-implemented, **NOT Windows compiled, signed or
runtime verified**. It cannot coexist with an installed official
`SbieSvc`/`SbieDrv` namespace. Do not activate private service/driver
against the active Client 02 environment. The shared version/ABI guard
does not constitute cryptographic driver attestation or prove that all
other DLL ioctl paths are individually authenticated.
