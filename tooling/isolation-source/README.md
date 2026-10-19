# Isolation Source Tooling — Active Route (v1.18.6)

This README is the **single operator entrypoint for native isolation source
verification/build**. The active desktop pilot continues to use the
*official installed* Sandboxie Plus for `MinecraftClient02`; **it does not
load, sign, install or switch to the owned engine**.

## Exactly one current source/build path

| Role | Canonical owner | Allowed use |
| --- | --- | --- |
| Unchanged comparison source | `engine/isolation-candidates/sandboxie-v1.18.6/` | Read-only byte/tree reference (1,627 files) |
| Editable native source | `engine/isolation-owned/` | One owned v1.18.6 C/C++ source tree (1,627 files, 11 reviewed modifications) |
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
