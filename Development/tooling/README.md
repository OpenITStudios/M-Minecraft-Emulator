# Developer tooling

The only active product is the Lazy Tester native Minecraft Education manager.
This directory contains source/structure checks, package preparation, Windows
native compatibility source, and retained historical evidence. Nothing here is
a second live game process/session owner.

| Path | Status |
| --- | --- |
| `repository/` | Active Git-tree, dependency, route, CI-disabled and documentation-link checks |
| `windows-toolchain/` | Source/build/package helpers when explicitly requested; no automatic local test |
| `dependencies/` | Dependency consistency review and maintenance |
| `acceptance/windows/bootstrap-ordering/` | **Active native DLL source** used by the five-session game bootstrap |
| `acceptance/` | Native and historical acceptance collectors; older Sandboxie/VM results are not current gameplay proof |
| `isolation-source/windows/` | Three retained embedded PowerShell helpers for the still-compiled legacy Rust compatibility adapter; all obsolete fork/provision tooling removed |
| `local-test/` | Optional source acceptance bookkeeping, not runtime evidence until executed |

The vendored Sandboxie trees, VMware Guest Agent resources, and retired fork
build/provision scripts have left `Local`, but historical collectors and the
three embedded legacy compatibility helpers remain
until their callers are removed. Do not run legacy service/VM scripts, grant
new game-launch authority or delete user data during source cleanup.
