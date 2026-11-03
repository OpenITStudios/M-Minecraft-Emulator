---
id: document.validation.local-pc-first-run
class: DOCUMENT
domain: validation
role: PROCEDURE
authority: WORKING
lifecycle: ACTIVE
---

# First Windows PC test — Lazy Tester

**Scope:** the standard **Lazy Tester** desktop manager on Windows 10/11 x64,
from the GitHub `Local` source branch. This is an operator runbook, **not**
evidence that a build, installation, Minecraft session or test has run.
For experimental independent AI keyboard/mouse input use the separate
[controlled AI Computer Use guide](ai-computer-use-localtest.md); that
feature is compile-OFF/default-OFF and **not** in a normal package.

**Start here, rather than using legacy G0/VMware instructions.** The one
development command entry is the existing root `DEV.cmd`. No new
launcher, runtime owner, CI workflow or data directory is introduced.

## Readiness in one table

| Layer | Available in Git source | Necessary Windows observation |
| --- | --- | --- |
| Source and pinned locks | Yes, branch `Local` | Exact clean checkout SHA |
| Developer controller / build scripts | `DEV.cmd`, source and manifest | Pinned tools and Windows compiler prerequisites |
| App UI and native code | Svelte/Tauri/Rust source | Executed preflight and real desktop window |
| Native runtime payload | **No DLL binaries in Git** | Both reviewed `bootstrap-compat.dll` and `bootstrap-enroll.dll` must be provided and match exact source digests |
| Start shortcut | `RUN.cmd` source only | Root `Lazy Tester.exe` exists after an approved `DEV.cmd package` |
| Installer candidate | Packaging source only | Actual unsigned NSIS output, integrity and preservation checks |
| Minecraft and 5 profiles | Game/session source, historical pilot evidence | Fresh **same-revision** Windows + licensed Education gameplay acceptance |
| F3 automatic XYZ/Copy | **Not connected** | Native pixel source, correct game/window pairing and real Education recognition still missing |

The two native DLLs are not interchangeable with the **separate, unapproved
AI-input experimental DLL**. Never change source hash policy just to get a
local build to run.

## 1. Before running any developer command

Use a disposable Windows test environment or back up existing user-owned
`App/` and `Data/` first. Do not delete/reset existing Minecraft worlds,
profile directories, credentials, saved accounts or another installed
Lazy Tester copy. Use **Git for Windows**, a compatible **Visual Studio MSVC
x64 C++ build environment + Windows SDK**, and **WebView2**. The exact
portable versions for Node/npm/Rust/PowerShell belong to
[toolchain.json](../../Development/toolchain.json); do not substitute
global installs. Official Minecraft Education and distinct licensed
accounts are only needed at the later game test stage.

**Fresh clone** in a normal Windows PowerShell window:

```powershell
git clone --branch Local --single-branch https://github.com/OpenITStudios/M-Minecraft-Emulator.git
Set-Location .\M-Minecraft-Emulator
git branch --show-current
git status --porcelain
git rev-parse HEAD
```

**If this checkout already exists**, from its root instead:

```powershell
git switch Local
git fetch origin Local
git pull --ff-only origin Local
git status --porcelain
git rev-parse HEAD
```

**Read-only inspection before downloads:**
```powershell
.\DEV.cmd localcheck
```
This checks the clean Git checkout, tracked source lockfiles, six portable
tool files (including rustfmt), the SDK resource compiler, installed MSVC x64
compiler/linker paths, and source-pinned DLL hashes including directory links.
It checks file presence/integrity, **not actual compiler execution**. It needs only
Windows PowerShell/Git, does **not** change files, and normally reports
BLOCKED on a fresh clone until bootstrap and native DLLs are prepared.

The branch must read `Local`, `git status --porcelain` must be empty, and
the last command must return a full **40-character SHA**. Keep that SHA in
your test notes. If local work is dirty or the pull cannot fast-forward,
**stop** and review it; do not use `git reset --hard`, `git clean -fd`,
or overwrite private data. Each later Git commit invalidates older
exact-revision test results.

## 2. Prepare and verify the developer environment

When the operator authorizes downloads/toolchain setup, run **once**:

```powershell
.\DEV.cmd bootstrap
.\DEV.cmd doctor
.\DEV.cmd localcheck
```

The launcher now propagates bootstrap's actual failure exit code instead
of falsely succeeding. Stop if bootstrap or doctor fails. The read-only
localcheck can still report missing DLLs until those files are provided.

`bootstrap` provisions pinned portable Node, npm, PowerShell and Rust
inside ignored `workspace/toolchains/`. It **downloads files and changes
developer workspace state**, so it is not a read-only inspection. The
MSVC/Windows SDK, Git and WebView2 are external prerequisites. A successful
doctor confirms pinned development inputs, **not** compiler success,
an installer or Minecraft readiness.

Optional read-only controller views, after bootstrap:

```powershell
.\DEV.cmd inspect --json
.\DEV.cmd catalog --json
.\DEV.cmd plan --json
```

Do not use `DEV.cmd g0`: its older VMware route is legacy/blocked, not
the native Lazy Tester acceptance path.

## 3. Verify source before opening a desktop window

After permission to execute source tests and locked npm dependency
installation, from a **clean** `Local` checkout:

```powershell
.\DEV.cmd preflight
```

This checks the actual toolchain and committed Cargo/npm locks, reuses or
installs the locked frontend dependencies, executes the existing source
verification, and writes a bounded local manifest under ignored
`workspace/runs/`. Record the **actual exit code and full HEAD**. On
failure, **stop** and report the first error; do not skip checks or
mark the application ready. This step does **not** launch Minecraft,
compile the experimental AI-input path, or generate an installer.

## 4. Supply the reviewed native payload — hard prerequisite

The standard `DEV.cmd dev` and `DEV.cmd package` both run
`prepare-native-payload.mjs`. **A clone alone is insufficient.**
They require an operator-reviewed directory with exactly the necessary
native binaries:

```text
<reviewed-native-payload-directory>/
  bootstrap-compat.dll
  bootstrap-enroll.dll
```

The source authority for their approved SHA-256 values is
`REVIEWED_EDUCATION_BOOTSTRAP` in
[the native controller](../../Development/apps/desktop/src-tauri/src/engine/independent_bootstrap.rs).
The existing staging script verifies **both** hashes before use.
Obtain the matching reviewed pair from an authorized build/artifact
process; if unavailable, report **BLOCKED: REVIEWED_NATIVE_PAYLOAD_MISSING**,
not a missing feature in Git. If a previous local `App/` folder contains
native DLLs, `DEV.cmd localcheck` will check their SHA-256 against the
current Rust policy; matching binaries can be reused by pointing the
environment variable to that `App/independent-bootstrap` folder.
Do **not** copy accounts, profile data, worlds or arbitrary DLLs, override
the hashes, create placeholders, or enable experimental AI input.

Only after the reviewed pair is genuinely available on the PC, in the
**same PowerShell session**, set its actual absolute path:

```powershell
$env:LAZY_TESTER_NATIVE_PAYLOAD_DIR = 'C:\path\to\reviewed-native-payload'
.\DEV.cmd localcheck
.\DEV.cmd dev
```

The path above is an **example placeholder**, not a repository folder.
`dev` builds and opens the existing Svelte/Tauri desktop in developer
mode. Failure to compile, locate the DLLs or open the window is a
**failed test**, not a reason to bypass admission.

## 5. Check the desktop before Minecraft

Verify the manager itself without launching a game: one window opens,
five stable MC01–MC05 slots appear, selection works, Refresh remains
read-only, the app does **not** auto-launch Minecraft or sign into Microsoft,
and closing/reopening the manager does not modify game data. Check the
presented status against reality; a passive DWM preview or a PID is not
proof that a game is playable.

For the existing **manual** coordinate feature, paste sample XYZ and
check decimal preservation, negative values and floor-based block/chunk
display. The F3 Inspector must label unknown live XYZ/rotation/dimension
as unavailable; **do not expect one-click live Copy Coordinate** from
this revision. Any manually supplied screenshot of real Education
Show Coordinates is separate, explicitly reviewed F3 evidence.

## 6. Optional package and root launcher test — only after steps 1–5

With separate permission for a Windows build and candidate staging,
with the same valid DLL environment variable, a clean source checkout
and **Lazy Tester itself closed**:

```powershell
.\DEV.cmd package
.\RUN.cmd
```

`package` builds an **unsigned development candidate**, exports an NSIS
installer/checksum/manifest under ignored
`workspace/packages/lazy-tester/`, stages the checked local application
under ignored `App/`, creates the root `Lazy Tester.exe` launcher,
and performs its source-defined desktop startup check. It may refresh
`App/`; use a disposable test environment and inspect backups first.
It must not remove `Data/` or installed worlds/accounts. `RUN.cmd` only
opens an already prepared root executable; it does **not** build one.
Do **not** interpret the unsigned installer as release-ready or install
it over an account-bearing environment for first acceptance.

If `package` fails, capture its actual first failure. No installer or
gameplay PASS can be claimed from source review alone.

## 7. Real Windows/Minecraft smoke test order

Execute only when the operator separately approves live game sessions.
Use real, independently licensed Education accounts, manual game launch,
and observe each outcome against the **same** source SHA.

| Order | Check | Required observed evidence |
| --- | --- | --- |
| 1 | Manager only | App starts/closes; no automatic Minecraft launch or login |
| 2 | MC01 | Manual launch, signed-in interactive world, normal CloseOne and reopen |
| 3 | MC02 | Separate account and private profile; both remain usable together |
| 4 | MC03–MC05 | Add one at a time; five distinct licensed playable sessions, no profile collision |
| 5 | Workspace | Correct selected window, peer previews, focus, resize, minimize/reopen |
| 6 | Recovery | CloseOne, CloseAll, manager exit/reopen, reconnect without data loss |
| 7 | F3/manual XYZ | Manual paste works; F3 unavailable fields are honest; automatic Copy remains blocked |
| 8 | Package preservation | **Separate** disposable install/repair/uninstall review; `Data/` survives |

Experimental multi-client AI keyboard/mouse input and
[Skills Probe/Map-Aware QA](../system/map-aware-probe-qa.md) have **separate
approval and evidence gates**. They are not silently enabled by this
desktop smoke test. A past five-profile pilot at another SHA does not
satisfy a new revision.

## 8. Report one result, not a manufactured PASS

For each actually executed stage, record **full `Local` SHA**, Windows
version/architecture, actual command, exit status, observed application
version, available native binary hashes, and concise PASS/FAIL/BLOCKED/
NOT_EXECUTED evidence. Keep private credentials, account IDs, game
worlds, token files, private paths and raw captures out of GitHub.
Use the existing ignored `workspace/` for local-only diagnostics where
appropriate. The only current development state/next-action owner remains
[planning/development.md](../../planning/development.md); do not create
a second state database or turn source-only documentation into test proof.

**Expected conclusion before Windows execution:** checkout/runbook
`SOURCE_PREPARED`; toolchain, payload, UI, installer, game, five profiles,
F3 live Copy and experimental AI input are **NOT_VERIFIED**. Stop at the
first real blocker and repair its original owner; do not bypass it.
