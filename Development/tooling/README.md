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
| `lazytester-particles/` | Optional, source-pinned LazyTester Particle Tuning .mcpack builder; never auto-installs or modifies Minecraft |
| `acceptance/windows/education-target.ps1` | **Active** registered Minecraft Education executable resolver, moved from the retired isolation-source tree |
| `local-test/` | Optional source acceptance bookkeeping, not runtime evidence until executed |

Retired Sandboxie/VMware build and runtime source has left the active tree. The native process and per-profile controller remain the only active Minecraft launch authority. Historical notes remain in Git revisions.
