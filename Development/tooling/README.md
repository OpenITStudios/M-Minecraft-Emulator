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
| `isolation-source/` | **Retired Sandboxie build/provision research**; some embedded helper scripts are still included by legacy Rust code and must be migrated before this directory can disappear |
| `local-test/` | Optional source acceptance bookkeeping, not runtime evidence until executed |

The complete vendored Sandboxie fork/reference trees and gitlink have left
`Local`, but historical collectors and inherited code dependencies remain
until their callers are removed. Do not run legacy service/VM scripts, grant
new game-launch authority or delete user data during source cleanup.
