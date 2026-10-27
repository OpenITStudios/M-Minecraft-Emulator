# Retired isolation-source tooling

The Sandboxie/owned-fork source, build, driver/service, import and provision
workflows have been removed from active `Local`. Exact historical sources are
recoverable through Git history. They are not a selectable Minecraft provider.

Only three **legacy compile-time embedded** scripts remain in `windows/`:

- `education-target.ps1`: registered Minecraft Education executable resolver,
  also referenced by native startup and preview checks.
- `owned-manager.ps1` and `probe-v1186-runtime.ps1`: compatibility inputs of
  the still-compiled but unregistered `owned_provider.rs` path.

Do not run either historical owned-provider helper or treat it as an active
launcher. Remove these remaining scripts only after all Rust `include_str!`
consumers are updated or retired. This README is a compatibility pointer for
old documentation links, not an active build or test owner.
