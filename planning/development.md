# Current Development Handoff — Lazy Tester

## Current state — 30 October 2026

**State:** SOURCE UPDATED / FINAL VALIDATION PENDING. The active product remains
Lazy Tester for five independent, individually licensed Minecraft Education
Windows profiles, using the original official game installation, reviewed
native compatibility payload, one retained session controller and private
profile storage. Only branch Local receives product development; CI is off.

**Proof ceiling:** Current source and GitHub revisions can be inspected
remotely. The pinned Windows Node 24 / PowerShell / Cargo / Tauri suites,
installed NSIS candidate, account sign-in, live DWM previews and five-client
Education gameplay have **NOT_EXECUTED** acceptance at the exact new revision.
A successful static source review is not a Windows or Minecraft PASS.

## Consolidated source outcome

- DEV.cmd is the only developer entrypoint. Its existing Node controller
  routes inspected changes to one allowlisted check; task execution requires
  exact Local HEAD, and high-impact operations retain separate approval.
- The default automatic check uses working-tree changes or the latest
  committed delta. Explicit --base-sha selects the changed-path union
  from a verified ancestor through HEAD plus local edits, so final
  multi-commit development is not misrepresented as one-commit coverage.
  It introduces no new runner, CI, state file, process, or build engine.
- Desktop npm dependencies are reused only after pinned manifest/lock,
  installed dependency-graph and nonlinked-root checks. Missing or invalid
  ordinary dependencies fall back to frozen npm ci; uncertain links refuse.
- Desktop and runtime-core Rust checks use the existing shared Windows-MSVC
  Cargo target. Active incremental cache stays reusable. Neither a new target
  tree nor unscoped cargo clean is introduced.
- Successful package export may retire at most eight older verified installer
  candidates, retaining the newest two. Historic compiler archive retirement
  remains an **independently approved, fail-closed** action, never an automatic
  consequence of run auto, preflight, or packaging.

These are source implementation decisions, not measured timing/disk savings.
No Minecraft lifecycle, Microsoft account/world data, runtime DLL, installed
identity, game package or native preview behavior is changed in this stage.

## Unverified / deferred

- Exact Windows dependency-reuse behavior, source regression tests,
  desktop/compiler integration, signed/installed launch and full five-session
  gameplay remain unexecuted at this revision.
- The historical Cargo archive's real eligibility, linked-file risks and
  physical storage reclamation are UNKNOWN. Preserve all archive contents
  unless the separate approved retirement gate and actual evidence allow it.
- Fresh-profile gameplay, cross-volume data migration, live DWM pixels and
  reconnect/normal-exit paths need their existing target-Windows acceptance.
- Extra RAM/FPS or graphics optimization, macOS support, new native providers,
  hosted runners and CI remain outside the active development scope.

## Next — one validation boundary

After the bounded source changes are frozen, perform the existing
exact-revision Windows test:localtest and full-source/preflight validation,
then the actual installer and Minecraft five-profile acceptance as applicable.
Collect developer build timings and storage inventory before proposing any
further performance or cache change. Mark each unmet proof NOT_EXECUTED,
not PASS; do not block independent remote source work on unavailable PCs.

Historical work notes and prior Next proposals remain in immutable
Local Git history, not in a parallel current-status register.
