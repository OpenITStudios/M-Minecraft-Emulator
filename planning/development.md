# Current Development Handoff — Lazy Tester

Local is the only product branch. CI remains OFF; main is release-reserved.
Renderer/RAM experiments remain PAUSED. Current bounded work is physical checkout
organization, before any new product feature or data migration.

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

## Verification in progress

Metadata-only before/after inventory and execution logs belong to ignored
workspace/runs/layout-20261026. Initial relocated checks: structure PASS,
15 repository/local-test/toolchain tests PASS, 30 frontend tests PASS,
Svelte typecheck zero errors/warnings, DEV.cmd doctor PASS. Remaining source and
package construction checks must be recorded against the committed revision.

No GUI, installer, Minecraft, UAC, driver/service or Computer Use was launched.
Installed behavior and Minecraft runtime acceptance remain NOT_EXECUTED.
Build results never imply account persistence, migration or gameplay acceptance.

## Product limits retained

The prior five-account Education Host World pilot was operator-confirmed only
for the candidate tested then. Native DLL source remains in acceptance tooling.
Cross-volume data migration, partial-setup recovery, Settings completion,
final UI acceptance and signing/public release remain separate unfinished work.
No new lifecycle/storage owner, wire schema or installer identity is introduced.

## Next — one action

Finish consumer/path verification and Windows candidate export for this layout,
then record exact source SHA and physical before/after evidence. Do not resume
features, cache deletion or Minecraft/RAM experiments as incidental work.
