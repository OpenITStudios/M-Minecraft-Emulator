# Current Development Handoff — Lazy Tester

## Active work: canonical Rust build outputs and approved historical cache retirement (30 October 2026)

**State:** DONE for bounded source implementation and synthetic safety checks. Windows Node 24/PowerShell 7, Rust/Cargo/Tauri execution, actual cache cleanup and NTFS bytes reclaimed remain **NOT_EXECUTED**.

**Product and authority:** Lazy Tester manages five independently launched native Minecraft Education Windows profiles. Runtime/session ownership, reviewed native DLL hash admission, installed identity, App/Data profiles and Minecraft worlds remain unchanged. Only branch `Local` may receive product development changes. GitHub Actions are disabled.

**Efficiency — one shared target:** Both the desktop and standalone runtime-core Cargo `check` and `test` scripts now select `--target x86_64-pc-windows-msvc` like the existing quick checks and Tauri dev path. This prevents unnecessary host-default **output directory** duplication on supported Windows. Cargo features and dev/test/release profiles can still produce legitimately different artifacts; a one-time rebuild may be required. Incremental compilation and the single canonical `workspace/builds/desktop/target` remain enabled; never run unscoped `cargo clean` on the active target. Cargo's automatic global cache GC remains Cargo-owned.

**Efficiency — AI validation:** `DEV.cmd` remains one launcher with `inspect/catalog/plan/run` and a fixed task registry. Documentation accompanying only frontend or Rust source no longer forces full validation; standalone Cargo manifests, lockfiles, build.rs, native/CMake, unknown or mixed source still require full source verification. The `run auto` path is SHA-pinned and cannot initiate setup, packaging, archive retirement or other consent-gated tasks.

**Old generated artifact lifecycle:** Post-success installer packaging already retains the latest two verified installer candidate copies and removes bounded old verified copies only. The older compiler archive `workspace/archive/compiler-cache-20261026` remains outside this automatic package turnover. `storage cargo` now measures its `target/` subtree separately from the active target. A **distinct approval-gated task** `storage.retire-archive` can use pinned Cargo `clean --target-dir` solely for `workspace/archive/compiler-cache-20261026/target` after checking recognized, complete nonlinked old Cargo layout and nonempty replacement Windows MSVC artifacts. Unknown entries, partial scans, symlinks, hardlinks, missing Cargo, and suspect paths block. This operation is **not scheduled or run automatically** and cannot access active Cargo, other archives, registry/toolchains, protected App/Data/worlds or backups.

**Safety and evidence:** The new archive retirement module is fail-closed and exposes no arbitrary shell/deletion path. Its exact proposed Git blob passed 8/8 synthetic V8 mocked-filesystem/Cargo scenarios (including scoped target args, unknown/hardlinked contents, no active replacement, failed Cargo, and no false reclaimed-space claims). New Node filesystem fixture tests and source-routing regression assertions were authored but the full pinned Windows source test suite remains **NOT_EXECUTED**. No old archive contents or real Windows disk totals were available in this remote GitHub session. Source changes cannot be promoted into actual runtime/cleaner proof.

**Unresolved:** Whether the historical archive matches the recognized structure, has hardlinks, or is actually responsible for the expected ~60 GB usage is UNKNOWN. Cleanup may block entirely; no real bytes have been freed. True disk-allocated bytes differ from logical file sizes. Direct Cargo outside `DEV.cmd` may still make new target trees. Windows ACL/junction/race behavior and rebuilt cache performance have not been measured.

**Next:** Run the pinned Windows `test:localtest` suite, collect `DEV.cmd storage report` and `DEV.cmd storage cargo`, and inspect the actual archive eligibility before explicitly approving scoped retirement. Compare build timings and physical free space afterward.

---

Historical phase descriptions, earlier decisions and prior proof classifications are retained in immutable `Local` Git commits, not accumulated in this active handoff.
