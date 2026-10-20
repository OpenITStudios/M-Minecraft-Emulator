# Isolation Source Tooling — Active Route (v1.18.6)

This README is the **single operator entrypoint for native isolation source
verification/build**. The active desktop pilot continues to use the
*official installed* Sandboxie Plus for `MinecraftClient02`; **it does not
load, sign, install or switch to the owned engine**.

## Exactly one current source/build path

| Role | Canonical owner | Allowed use |
| --- | --- | --- |
| Unchanged comparison source | `engine/isolation-candidates/sandboxie-v1.18.6/` | Read-only byte/tree reference (1,627 files) |
| Editable native source | `engine/isolation-owned/` | One owned v1.18.6 tree (1,627 files; 11 C/C++ changes plus four exact build-project path fixes) |
| Build component contract | `engine/isolation-native/owned-runtime/build-profile-v1186.json` | Exactly 3 stages, 16 native binaries, 5 resources and 4 original licenses |
| Source verification | `tooling/isolation-source/verify-owned-v1186.mjs` | Read-only exact-source and upstream provenance check |
| Windows build (optional, build-only) | `tooling/isolation-source/build-owned-v1186.ps1` | Compile only; 3 MSBuild stages, WDK required |
| Staging (optional, NOT-INSTALLABLE) | `tooling/isolation-source/assemble-owned-v1186.mjs` | One immutable payload and provenance manifest |
| Desktop inspector | `apps/desktop/src-tauri/src/engine/owned_isolation.rs` | Read-only package inspection; cannot activate the provider |

**Read-only source check:**

```powershell
npm run isolation-source:verify
```

**Optional native Windows compiler build** (not required for first UI
smoke or normal `DEV.cmd dev`, and not proof of working Minecraft):

```powershell
# From the repository root; use a new report/package name each run.
New-Item -ItemType Directory -Force .\workspace\builds, .\workspace\packages | Out-Null
$runId = Get-Date -Format 'yyyyMMdd-HHmmss'
$report = Join-Path (Get-Location) "workspace\builds\owned-$runId"
$payload = Join-Path (Get-Location) "workspace\packages\owned-$runId"
.\tooling\isolation-source\build-owned-v1186.ps1 -ReportDirectory $report -PackageDirectory $payload
```

Never reuse an existing report or package directory. The builder and
assembler refuse paths outside `workspace/builds/` and
`workspace/packages/`, respectively, and enforce clean source,
bounded artifact provenance and reparse-point checks. Neither command
installs or starts a driver, replaces official Sandboxie, creates boxes,
changes accounts or proves multiplayer.

## Current Windows build evidence — 20 October 2026

The owned-engine campaign prioritizes build/signing before switching the desktop.
At `f529d59fda778be50cd3e874374a5546c32dce29`, all three actual Windows compiler
stages pass and all 16 binaries plus five resources/four licenses stage successfully.
MFC v143, WDK 26100.6584 and its VS2022 integration are installed after operator
authorization. The package is **MIVUBI_V1186_CORE_STAGED_NOT_INSTALLABLE**.
Production signing, activation, owned runtime operations and gameplay remain
unexecuted; no driver was installed and no security policy changed.

The builder explicitly disables WDK's default test signing and checks the driver
is actually `NotSigned`. An earlier implicit test-signed attempt was rejected and
its generated temporary certificate/private key removed. The final clean unsigned
candidate is `workspace/packages/owned-20261020-150022/`, with its source/hash-bound
report in `workspace/builds/owned-20261020-150022/`. Resources normalize checkout
CRLF only if the result matches exact committed Git blob bytes; edited configs
or licenses remain rejected. Preserve the successful candidate; do not install it.

Four reviewed project changes quote MASM, import-library and message-compiler
paths, including checkout paths containing spaces. The source verifier reconstructs
these exact transformations from pristine upstream separately from the eleven
C/C++ security/identity modifications. Unexpected edits still fail verification.
Standalone C/C++ policies pass actual MSVC compilation and all three CTest tests:

```powershell
npm run isolation-native:test-owned-start
```

That command does not compile or load the real kernel driver. Full campaign
results and remaining work live in the existing
[Windows validation procedure](../../docs/validation/native-pilot-localtest.md).

## Explicit real-payload desktop inspection

The desktop's existing Rust inspector can be exercised against a selected real
v1.18.6 payload, without opening the app, loading a DLL/driver or changing services:

```powershell
$env:MIVUBI_TEST_OWNED_PAYLOAD = (Resolve-Path 'workspace/packages/owned-20261020-150022').Path
try { npm run isolation-source:inspect-candidate }
finally { Remove-Item Env:MIVUBI_TEST_OWNED_PAYLOAD -ErrorAction SilentlyContinue }
```

Select an existing absolute payload directory from the matching build report;
the example is the completed unsigned candidate, not an automatic latest selector.
The command runs one opt-in Rust test, checks all 25 declared artifacts with the
production inspector, and observes the current SCM namespace using its existing
read-only probe. It asserts provider selection, driver-load/gameplay proof and
installation/activation remain false. Printed service-name registration is not
service-owner authentication. A successful checksum inspection never grants
permission to replace an occupied namespace or load an unsigned driver.

The real-payload test is ignored by default because source verification must not
depend on machine-local build artifacts; explicit invocation fails if the variable
is missing or the payload is invalid. This is developer verification, not a new
installed-app CLI, IPC schema, runtime provider or readiness database.

## Not active — historical implementations

The pinned v1.18.5 Git submodule
`engine/isolation-upstream/Sandboxie`, v1.18.5 fork overlay,
`bootstrap-fork.mjs`, `assemble-private-engine.mjs`,
`build-private.ps1`, source migration and disposable service/driver
prototypes remain **historical research/test fixtures only**.
They are **not fallback build commands** for the current v1.18.6 engine.
Do not remove these sources indiscriminately: regressions, source
provenance and licenses still refer to them.

Archived investigation details:
[legacy v1.18.5 isolation notes](../../planning/history/isolation-v1185-source-notes.md);
[native overlay history](../../planning/history/isolation-native-development-notes.md).
See the [source ownership map](../../docs/system/implementation-map.md)
and [active Windows localtest](../../docs/validation/native-pilot-localtest.md)
before selecting a command.
