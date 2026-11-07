---
id: document.validation.local-pc-first-run
class: DOCUMENT
domain: validation
role: PROCEDURE
authority: WORKING
lifecycle: ACTIVE
---

# First Windows PC test — LazyTester

**Scope:** the standard **LazyTester** desktop manager on Windows 10/11 x64,
from the GitHub `Local` source branch. This is an operator runbook, **not**
evidence that a build, installation, Minecraft session or test has run.
For experimental independent AI keyboard/mouse input use the separate
[controlled AI Computer Use guide](ai-computer-use-localtest.md); that
feature is compile-OFF/default-OFF and **not** in a normal package.

For a practical **first day** with no need to traverse 306 cases, follow the
[13-scenario local smoke](efficient-local-testing.md). It selects a
quick source screen and only MC01/MC02 first. Full-source and five-client
acceptance remain separate; no tests are executed by reading these guides.

For a full operator-facing checklist beyond this short command runbook,
see the [master test guide](lazy-tester-test-master.md),
[198 standard cases](lazy-tester-test-cases-standard.md), and
[108 separately gated MCP/AI/QA cases](lazy-tester-test-cases-experimental.md).
These are planned **NOT_EXECUTED** scenarios, not a report of successful
Windows, Minecraft or installer tests.

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
| Retired F3/Coordinates desktop controls | **Not part of current UI** | Verify both are absent; no unsupported live-coordinate or Copy action is advertised |

The two native DLLs are not interchangeable with the **separate, unapproved
AI-input experimental DLL**. Never change source hash policy just to get a
local build to run.

## Local root and storage consistency (read before first run)

Use one ordinary, writable, nonsynchronized checkout folder for the
source-only localtest. Source, `DEV.cmd`, `RUN.cmd`, `Development/`,
`workspace/`, prepared `App/`, and private `Data/` remain siblings
inside that one checkout. `App/` and `Data/` may be absent on a fresh
clone; they are not prerequisites for recognizing the approved dev root.
The reviewed DLL directory still needs its separate explicit environment
variable and hash check.

The native developer binary may execute from
`workspace/builds/desktop/target/`, but must resolve the canonical
`<checkout>\\Data\\`, **never**
`workspace\\builds\\desktop\\target\\...\\Data\\`.
If a previous dev run left a build-local Data folder, startup must BLOCK
rather than hide/copy/migrate private accounts/worlds. Keep the original
files untouched and report the exact failure. Verify root/data paths using
read-only metadata only; do not resolve this by `git clean`, deleting a
directory, symlinking, copying hives or bypassing native identity checks.

An approved standalone NSIS installation has its own `App/` + `Data/`
under its selected Windows location and must not be mixed with a source
checkout for the first localtest. Desktop and Start Menu links are OS
shortcuts, not duplicate installations. Windows-managed Microsoft
authentication and the licensed game are intentionally outside the repo.

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
tool files (including rustfmt), both SDK resource compilers (`rc.exe` and `muirct.exe`), installed MSVC x64
compiler/linker paths, and source-pinned DLL hashes including directory links.
The PowerShell 5.1 check can confirm a 64-bit Windows host, but cannot
reliably prove native x64 rather than emulated x64 on ARM64. The existing
PowerShell 7 native gate now also runs before `dev` and `package`.
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

For an **efficient initial local smoke** after authorized toolchain and locked
npm setup, from a **clean** `Local` checkout:

```powershell
.\DEV.cmd setup
.\DEV.cmd plan --profile local-smoke --json
.\DEV.cmd check:quick
```

`setup` installs locked npm dependencies and must be explicitly authorized;
skip it if already prepared. The `plan` is read-only. `check:quick` runs
repository/lock checks, repository fixtures, frontend typecheck/Vitest and
desktop/runtime Rust compilation, but no full Rust test matrix or installer.
The existing `tooling/local-test/session.mjs` writer records a private
metadata-only `*-quick-*/manifest.json` receipt with actual result and
elapsed wall time. `quickSourceScreen` alone may pass; full
`sourceVerification` and `windowsGameplay` stay `NOT_EXECUTED`.
Record its actual result and full SHA. A failure stops this baseline; no
gameplay PASS can be inferred from a source check.

For **full regression / release verification**, separately authorize and run:

```powershell
.\DEV.cmd preflight
```

Full `preflight` checks the pinned tools and committed locks, reuses or
installs locked dependencies, runs all existing source verification, and
writes a metadata-only run manifest in ignored `workspace/runs/`.
Neither the quick route nor full preflight launches Minecraft, compiles
experimental AI input by default, or creates an installer. Do not bypass
the reviewed DLL gate in the next section.

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
Until the variable is set, `localcheck` deliberately exits **BLOCKED** even
when the old `App/` DLL hashes match: the build's staging script will not
implicitly reuse those binaries.
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

On Play, confirm the retired F3 Inspector and Coordinates & Saves controls
are absent. Navigate via Settings to General and Data Storage using keyboard
focus. Inspect the selected profile's graphics/FPS labels and read-only
storage status without saving, clearing, restoring, or changing any game data.
A native Minecraft coordinate display is not a LazyTester desktop feature.

### 5.1 UI interaction failure checks (manual, same-SHA, no destructive commit)

Before gameplay, check mouse and keyboard navigation on 920×620 and
1280×900 desktop sizes, then 125–200% Windows DPI. Verify that every
visible action is either clickable with a truthful result or clearly
disabled with a reason. Do not treat a synthetic screenshot or a native
process ID as a functional PASS.

- Open and **Cancel** a running game's CloseOne review. While open,
  other launcher cards, Close buttons, Refresh, Separate window and
  page navigation must not replace the review or launch another game.
  After Cancel, relevant controls and keyboard focus must recover.
- In Data Storage, open and **Cancel** cache cleanup and full-clear
  review without submitting a destructive action. Refresh, folder,
  recovery and other clear actions must be unavailable until Cancel.
  During an Advanced recovery review, the detail section must not
  collapse and hide its own Cancel control.
- Inspect unsaved Graphics/FPS drafts, save/failed-save feedback,
  invalid FPS limits and background-audio selection. A native refusal
  must not leave the audio checkbox falsely changed. For actual
  mutations use only separately approved disposable profiles.
- Check Tab/Shift+Tab, Enter, Escape, focus return, long errors,
  high contrast, mixed MC01–MC05 states and unavailable backend.
  Preserve all original native/IPC guards and stop on any P0 data or
  process-identity mismatch.

These are **acceptance checks to execute later**, not tests passed
by this document or the source-only regression fixtures.

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
| 7 | Settings and storage UX | Play/General/Data Storage navigation, readable status and keyboard focus; no retired desktop F3/Coordinates controls |
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
and experimental AI input are **NOT_VERIFIED**. Retired desktop F3/Coordinates
controls are not an acceptance requirement. Stop at the
first real blocker and repair its original owner; do not bypass it.
