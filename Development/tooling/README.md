# Developer tooling

The only active product is the Lazy Tester native Minecraft Education manager.
This directory contains source/structure checks, package preparation, Windows
native compatibility source, and retained historical evidence. Nothing here is
a second live game process/session owner.

| Path | Status |
| --- | --- |
| `repository/` | Active Git-tree, dependency, route, CI-disabled and documentation-link checks |
| `windows-toolchain/` | Source/build/package helpers when explicitly requested; no automatic local test |
| `storage/` | Read-only artifact inventory plus fail-closed retention planning for reviewed developer outputs; no deletion |
| `dependencies/` | Dependency consistency review and maintenance |
| `acceptance/windows/bootstrap-ordering/` | **Active native DLL source** used by the five-session game bootstrap |
| `acceptance/` | Native and historical acceptance collectors; older Sandboxie/VM results are not current gameplay proof |
| [`lazytester-particles/`](lazytester-particles/README.md) | Optional Particle Tuning v0.3 offline .mcpack builder; no automatic installation |
| [`lazytester-textures/`](lazytester-textures/README.md) | Optional block-only Texture Lite v0.1 offline .mcpack builder; entity/PBR extension retired |
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

## Resource-efficiency source previews — closed decision, reusable code

These **separate, explicit build tools are retained** for possible future
operator-approved testing; they are not runtime services and neither is
installed or activated by LazyTester. The [current closed scope, proof limits,
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
