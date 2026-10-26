# Current Development Handoff — Lazy Tester

Local is the only product branch. CI remains OFF; main is release-reserved.
Renderer/RAM experiments remain PAUSED. Physical checkout root/source organization is implemented. No new product
feature or data migration was included.

## Physical checkout organization

Source parent: 47a45f9c7d4a8ce6799d56997fd38e4b4df49ce7.
The operator approved one Development directory, with App and Data retained in
the checkout root. Development now owns apps, engine, tooling, package.json,
toolchain.json, VERSION and .node-version. Historical experiments are physically
under Archive/experiments. No source mirror or old-root junction is retained.
Root docs/planning and agent/Git/editor routing remain visible and authoritative.

The authorized Vite PID 31864 was revalidated against its exact script path and
stopped with the operator's explicit permission before source relocation. Other
processes were not stopped. The Sandboxie submodule was moved through git mv;
its revision remains abd4c240031cfb46666daa74ca47ed4e48b561e4. The second worktree
is preserved at its original path/revision. Retained upstream source/license
bytes must remain identical to the parent revision.

DEV.cmd remains the developer entry point; direct repository npm commands use
npm --prefix Development. Developer scripts distinguish Development source from
checkout-local workspace output and toolchains. Default new Cargo output is
workspace/builds/desktop/target. Existing compiler caches are preserved, including
Development/apps/desktop/src-tauri/target; cache removal is explicitly deferred.
Packaging supports --export-only to verify a new installer without refreshing App.

App still contains the four verified payloads from source
5b578653bd7d576ac92a90e56eb06137e97efb05. RUN.cmd uses that payload. Data remains
empty. Old profiles, HKCU, capability and account/world data are not migrated.
Toolchains and private workspace material remain at their original locations.
Unclassified workspace material is preserved rather than inferred disposable.
The stale generated Svelte checker tree was archived under
workspace/archive/layout-20261026/svelte-check-before-relocation; regenerated
typechecking reports zero errors/warnings. This is not compiler-cache cleanup.

## Executed verification and limits

Implementation/source verification SHA: 9a3d8c160e4a92ce61690b2492050b83b962911e.
The subsequent handoff commit changes documentation only; no newer binary proof
is implied. Before/after metadata, receipts and logs are preserved in ignored
workspace/runs/layout-20261026/before.json and after.json.

PASS: physical and tracked structure/link checks, source-relative URL path audit,
DEV.cmd doctor, repository/local-test/toolchain and all Node suites (four declared
platform skips), 30 frontend tests, Svelte zero errors/warnings, frontend build,
Rust desktop all-targets compilation, two product-profile fixtures within the
larger desktop run, and all 316 runtime-core tests.

The complete verify:source command is NOT PASS. Desktop Rust execution completed
180 passing tests and 18 declared ignores, with one DACL failure in
owned_disposable_directory_handle_has_stable_file_identity (broad group access
inherited by its disposable temporary directory). An isolated rerun reproduced
the same refusal. malformed_oversized_message_cannot_be_processed_as_valid_probe
stalled beyond 60 seconds; only our test executable PID 8364 was terminated.
Both source owners are byte-identical to the pre-move revision; baseline runtime
reproduction before the move was NOT_EXECUTED. No ACL/security checks were weakened.
Desktop/runtime rustfmt --check also reports existing formatting differences,
including unchanged build.rs/runtime sources; no broad formatter rewrite was made.

Actual Windows release/Tauri/NSIS construction and export-only packaging PASS
at the implementation SHA, with 104 existing release compiler warnings.
Export: workspace/packages/lazy-tester/0.1.0-9a3d8c160e4a-20261026T084317679Z.
Exactly three exported files; installer is 2,588,939 bytes, Authenticode NotSigned,
SHA256 0843bda228f1cf5d82e3e302c16f4b19894c7dd3ca918eb320ca3e95beae0fa8.
App payload hashes still match the before inventory; Data still has zero entries.
Retained owned/candidate/upstream Git trees and the second worktree are unchanged.

No product GUI, installer, Minecraft, UAC, driver/service or Computer Use was
launched. Only noninteractive/disposable source fixtures ran. Installed behavior
and Minecraft runtime acceptance remain NOT_EXECUTED. Root/source organization is
complete; unclassified loose workspace logs/scripts/screenshots/probe folders
remain deliberately preserved, not certified sanitized or safely disposable.
Cache deletion and profile migration remain deferred, with no space-reclaimed claim.

## Product limits retained

The prior five-account Education Host World pilot was operator-confirmed only
for the candidate tested then. Native DLL source remains in acceptance tooling.
Cross-volume data migration, partial-setup recovery, Settings completion,
final UI acceptance and signing/public release remain separate unfinished work.
No new lifecycle/storage owner, wire schema or installer identity is introduced.

## Next — one action

Resolve the recorded source-suite DACL/pipe and formatting failures as a separate
bounded diagnosis before claiming a fully green source suite. Do not resume
features, cache deletion or Minecraft/RAM experiments incidentally. STOP here
for the completed checkout-layout scope.
