# Historical Sandboxie Source Import Investigation

> **ARCHIVED — NOT AN ACTIVE IMPORT PROCEDURE.** This preserves
> v1.18.5 migration and early v1.18.6 import checkpoints. Use
> [one current source owner](../../docs/system/owned-source-import.md)
> and the [one native pipeline](../../tooling/isolation-source/README.md).
> External D: paths and old commands below are historical only.

## Historical v1.18.5 source procedure (superseded)

## Archived full-source-import checkpoint — 15 October 2026

The user-supplied v1.18.6 archive is now fully imported as **1,627 normal
editable Git files** at `engine/isolation-candidates/sandboxie-v1.18.6/`.
Its complete subtree is the exact official
`f690477c07a40810f479f53973334ceae7e0ab13` tree from commit
`cdba7096240b93d79c5373aa81a75a1c5c71b883`. The former 827-file checkpoint
has been completed with 800 files, including all remaining Plus UI,
resources, tools, metadata and installer source. Original licenses and
upstream source bytes are preserved. The v1.18.5 owned tree remains a
separate unchanged migration input; no active engine switch is implied.

On Windows the importer compares resolved repository paths, stages only
missing files via NUL-delimited stdin, and uses the candidate's original
attributes without inheriting parent automatic text normalization. Exact
tree equality remains mandatory. A resumed Windows fixture and idempotent
re-run passed. Source import is not a native build or performance fix.
The historical checkpoints below document earlier incomplete transfers;
current continuation is at the top of [development handoff](../../planning/development.md).

## Historical ownership procedure

The application is importing the authentic **Sandboxie Plus / Classic v1.18.5**
source as **ordinary writable Git blobs** at `engine/isolation-owned/`.
The target is the *complete tracked upstream source tree*, including the Sandboxie
kernel/service/DLL/launcher, SandboxiePlus/SandMan sources, resources, tools,
installer and **all original license files**. It is **not a Git submodule**:
editing `engine/isolation-owned/Sandboxie/core/drv/box.c` is an ordinary commit
to `M-Minecraft-Emulator/Local` and cannot change upstream Sandboxie.

This is a source-ownership change, not a separate replacement operating-system
kernel, an app install, a signed SbieDrv.sys, or a proof of Minecraft gameplay.
The active official Sandboxie Plus installation remains untouched.

## Legal boundary before redistribution

Read the original pinned `LICENSE.Plus`, `LICENSE.Classic`,
`Sandboxie/COPYING`, and `SandboxiePlus/QSbieAPI/LICENSE` after importing.
The GPLv3 Sandboxie core, LGPL libraries and custom-licensed Plus UI have
**different obligations**. In particular, the Plus source license governs
modified binary redistribution, name/attribution, commercial use and
certificates. All existing original headers, authorship and licenses must
remain intact. Do not rename/relicense the SandMan Plus application, remove
certificate checks, or treat the repository's core GPL metadata as an
all-components license. This work does not grant commercial distribution rights.

## One-time import into `Local`

Only use a **clean, authorized, full-history** checkout of
`sandboxie-plus/Sandboxie` pinned to original tag `v1.18.5`, commit
`abd4c240031cfb46666daa74ca47ed4e48b561e4`. The app already records
this upstream reference as a Git submodule. The import never touches a
Windows driver/service or any Minecraft account, settings, boxes or worlds.

From the **root of a full checkout of M-Minecraft-Emulator**, in a PowerShell
terminal with Git and Node.js:

```powershell
git switch Local
git submodule update --init -- engine/isolation-upstream/Sandboxie
$head = (git rev-parse HEAD).Trim()
node tooling/isolation-source/import-full-sandboxie.mjs --seed engine/isolation-upstream/Sandboxie --expect-head $head
```

The importer verifies the seed's tag, exact commit, origin, full Git history
and clean tree; it requires the application's exact `Local` HEAD and a clean
index. If the directory is empty, Git's `subtree add --squash` makes the first copy.
If it already contains a **clean, committed partial upstream copy**, the same
importer validates every existing blob SHA and file mode, copies only missing
pinned blobs, and refuses any mismatch or unknown file. It commits only after
the staged full source subtree SHA exactly equals the pinned upstream root tree
SHA. This is a one-time completion, not a destructive reset.
Unlike copying only the 11 modified source files, this retains **all** tracked
Plus/Classic sources. The `--squash` choice retains the pinned original commit
and Git subtree provenance without inserting decades of unrelated upstream
commit history into the application repository. The pinned submodule remains
as a read-only reference, not the engine's future source of truth.

## Apply existing native MIVUBI modifications to the owned COPY

```powershell
$head = (git rev-parse HEAD).Trim()
node tooling/isolation-source/apply-owned-sandboxie.mjs --seed engine/isolation-upstream/Sandboxie --expect-head $head
```

This second **one-time** command refuses a tree that no longer matches the
unmodified pinned upstream snapshot. It generates the already-reviewed
private driver/service/DLL/Start/SbieIni changes, applies them with
`git apply --index --directory=engine/isolation-owned`, verifies their exact
expected bytes, checks that only the approved source paths changed, then
commits the changes on `Local`. The upstream seed is never modified.
Later fixes should edit the actual owned files directly and be committed
normally, rather than creating another wrapper or regenerating the whole
overlay. The historical overlay remains only the initial migration source.

The import script is idempotent on the complete unmodified upstream tree; the native patch script is not idempotent and refuses a patched tree. Neither command pushes automatically. Review the resulting commits
and push `Local` by the normal repository-approved Git workflow.

## Acceptance boundary

1. **Source imported:** exact upstream subtree hash matches (portable Git test).
2. **Owned source patched:** exact approved 11-file modifications are present
   as regular changes to the MIVUBI repository, original upstream unchanged.
3. **Windows build:** compile Win32, x64 user-mode, and WDK x64 kernel from
   `engine/isolation-owned/Sandboxie/` using the upstream documented MSBuild
   order; examine genuine compiler/linker errors. **Not executed here.**
4. **Runtime:** sign/load private driver in an authorized isolated Windows
   environment, verify service/DLL/launcher operation and rollback. Private
   core retains upstream service/device names and must not coexist with the
   official account-bearing installation. **Not executed here.**
5. **Minecraft:** test two separate Education sign-ins, stable graphics and
   multiplayer, then Clients 03–04 with distinct legitimately licensed
   accounts, preserving user data. **Not executed here.**

REA, if installed alongside a native analysis provider, may inspect genuine
built binaries; it does not replace Windows/MSVC/WDK or driver signing.

## Current GitHub state (15 October 2026)

At the first checkpoint `1752e98`, `Local` contained **213 of 1,624** pinned upstream Git blobs at
`engine/isolation-owned/`: 89 driver, 117 DLL and seven license/metadata
files. No existing blob SHA or file mode differs from upstream. A later GitHub transfer increased the matching owned snapshot to **314 of
1,624** files, including full service, low-level, launcher and ini groups.
The
partial import remains an ordinary Git directory, not a subtree-complete snapshot.

The resumed importer is prepared for **the committed partial tree**. The
actual full-source GitHub transfer, source patch and native Windows build
remain separate proof gates. Transferring blobs via GitHub API must be
followed by exact subtree SHA comparison before asserting completion.
A GitHub object SHA from another repository is not by itself sufficient
proof that the destination can resolve that object. Never delete, replace or
reinitialize this directory as a shortcut.

The ChatGPT container has no direct GitHub network route; GitHub connector
writes can transfer individual blobs but cannot atomically fetch/copy the
entire external upstream Git tree. A full checkout with the pinned upstream
seed can use the resumed importer if direct API transfer remains incomplete.
No user account, box, Microsoft login, signed driver or existing official
Sandboxie installation is touched by source operations.

## Verified user-provided upstream v1.18.6 ZIP (15 October 2026)

The uploaded `Sandboxie-1.18.6.zip` is a full **1,627-file** source archive.
Its SHA-256 is `22d8b004d4fe8dcf05654c4704a8b121c730352c1b406d2f1dad564dea451b33`.
Extracted into a disposable Git repository, the source produces exact
upstream tag `v1.18.6` commit `cdba7096240b93d79c5373aa81a75a1c5c71b883`
tree `f690477c07a40810f479f53973334ceae7e0ab13`.

The `Local` branch now has a **separate partial** versioned candidate at
`engine/isolation-candidates/sandboxie-v1.18.6/`: initially 827 original
source files, now completed to all 1,627 with exact Git tree parity. The
active older partial copy at `engine/isolation-owned/` is left unchanged.
This complete candidate is source only, NOT built or active.

`tooling/isolation-source/import-v1186-archive.py` is an executable Python
standard-library importer for exactly this ZIP. It accepts an existing,
clean and committed candidate, copies only missing files, and commits only if
the entire final candidate tree matches the pinned official upstream SHA.
A local disposable Git test using the authentic ZIP completed the full
1,627-file import and verified a clean, idempotent re-run. This is NOT
remote GitHub completion evidence.

Typical usage with an authorized checkout containing the archive:

```sh
python tooling/isolation-source/import-v1186-archive.py \\
  --repo-root . --archive /path/to/Sandboxie-1.18.6.zip \\
  --expect-head "$(git rev-parse HEAD)"
```

The `Sandboxie-Plus-x64-v1.18.6.exe` upload is an **installer**, not source.
It was not executed or redistributed. No driver, service, account or Minecraft
world data is touched by the source-only migration.

### Exact artifact audit — 15 October 2026

- ZIP SHA-256: `22d8b004d4fe8dcf05654c4704a8b121c730352c1b406d2f1dad564dea451b33`. A fresh disposable local Git reconstruction of all 1,627 tracked archive files returned the official tag tree `f690477c07a40810f479f53973334ceae7e0ab13` (PASS). ZIP extraction alone does not push to GitHub.
- Installer `Sandboxie-Plus-x64-v1.18.6.exe`: size 24,914,432 bytes; SHA-256 `3ad29e1ec18618ee9aa4ef4e21ab0548cf2188b94c05c41cb7a07b5d2d0270e2`, identical to the digest of the x64 v1.18.6 asset published at https://github.com/sandboxie-plus/Sandboxie/releases/tag/v1.18.6. It is a signed-release **reference artifact**, not an executable of the customized MIVUBI fork; no install or execution took place.
- The earlier **827/1,627** checkpoint has been completed to **1,627/1,627** source files with exact upstream tree parity. Legacy non-UTF-8 source bytes are preserved through the archive importer rather than text transport. Complete source is not proof that build dependencies, the MIVUBI overlay or native binaries are ready.
- Do not commit the third-party installer to the application runtime or point production launches to it; preserve original licenses and old source until license/build/driver/acceptance review.
