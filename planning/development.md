# Current Development Handoff — Lazy Tester

## Active scope — AI developer flow and bounded build artifact retention (30 October 2026)

**State:** SOURCE IMPLEMENTATION IN PROGRESS; real Windows execution, source-packaging acceptance and physical disk savings NOT_EXECUTED.

**Product:** Native-first Lazy Tester controls up to five independent Minecraft Education profiles. Active game lifecycle, Win32 window recovery, checked controller IPC, reviewed native DLL identity and the unsigned local candidate packager are unchanged. Retired Sandboxie/VMware routes and GitHub Actions remain disabled. Source writes belong only on GitHub branch `Local`.

**Decision:** The single root `DEV.cmd` remains the only developer entrypoint. It uses a pinned portable Node controller, a fixed 17-task allowlist, and existing PowerShell/npm/Cargo/Tauri owners. The AI `plan` and `run auto` path uses changed paths or the newest committed delta to select one scoped source validation (docs/frontend/Rust/full). Unknown or mixed changes escalate to full verification; no auto-mode task requires operator permission to install, package, mutate lockfiles or delete files. Explicit `--expect-sha` remains mandatory; all existing task safety, exclusive output lock and consent gates stay in force.

**Artifact lifecycle:** The native package owner may, **only after a completed and verified new candidate**, prune older verified three-file installer exports under `workspace/packages/lazy-tester/`. Protect the latest two candidates. Each deletion requires fixed naming, full receipt agreement, actual streamed installer SHA-256, regular singly-linked files, ordinary directories, bounded discovery and unchanged identities. Never invoke unscoped `cargo clean` or delete the active Cargo target, historical compiler archive, toolchains, `App/`, `Data/`, `workspace/private/`, worlds, accounts, evidence or backup material. The historical `storage plan` remains read-only and retains its conservative review default; the post-build turnover policy is separate and explicit. A blocked retention scan does not invalidate the already successful package but must report its limitation.

**Efficiency:** Cargo retains one canonical developer target with lean dev/test profiles and incremental compilation. Global Cargo dependency garbage collection remains Cargo-owned. Targeted validation avoids unnecessary Rust checks for frontend-only changes, while full source verification and packaging gates are retained. Do not add another Rust cache, agent/daemon, CI runner or duplicate build engine.

**Proof ceiling:** Read-only remote GitHub review and isolated Node.js 22 synthetic fixtures establish source/test correctness only. Pinned Windows Node 24, PowerShell 7, Tauri, installed app, native DLL and Minecraft gameplay verification are NOT_EXECUTED. Logical artifact sizes are not physical freed bytes. No storage reduction can be claimed without observed target-Windows evidence.

**Unresolved:** Rust `target/` contains potentially reusable Cargo intermediate artifacts; Cargo stable does not automatically garbage-collect that directory by build version. The separately archived older compiler cache and private App backups are not automatically disposable. Windows junction/ACL/race behavior, actual disk distribution and performance trade-offs remain UNKNOWN.

**Next:** Run the pinned local-test suite and observe a complete Windows `DEV.cmd run auto` and candidate build with retained/deleted candidate counts; collect `storage report` and `storage cargo` to determine whether additional narrowly scoped compiler-cache retirement is safe.

---

Historical decisions, earlier phases, benchmark notes and prior proof ceilings remain accessible via the immutable `Local` Git commit history. This file intentionally contains only the current owner, decisions, evidence, unresolved items and one next action.
