# Native Isolation — Source Ownership

**Current owner:** the editable native Sandboxie v1.18.6 source is
`engine/isolation-owned/`. This directory contains the auxiliary ABI
probe, source policy helpers and **historical** v1.18.5 fork overlay,
not a second active native engine or a separate deployed runtime.

For the only current v1.18.6 build, exact source verifier, stage/output
contract and workspace paths, use the
[Isolation Source Tooling README](../../tooling/isolation-source/README.md).
The machine-readable profile is
`engine/isolation-native/owned-runtime/build-profile-v1186.json`.

**Status:** source patch and source regressions exist; actual v1.18.6
Windows MSVC/WDK compilation, signing, SCM ownership, service installation
and owned-engine Minecraft gameplay are **NOT EXECUTED**. The official
account-bearing Sandboxie Client 02 pilot is not replaced or migrated.

The v1.18.5 fork overlay, private MSBuild scripts and ABI probe are retained
for explicit historical research or isolated tests only. They must not be
used as a second current build route. Source research is archived in
[Native Isolation Development Notes](../../planning/history/isolation-native-development-notes.md).
