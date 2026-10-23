# Emulator Engine — One Active Owner per Function

The active desktop source offers an official Minecraft Education Client 01 launch and presentation of up to four **already-running** registered Windows game-window candidates without an isolation provider (Windows/Minecraft execution NOT_EXECUTED). Existing-window presentation is not new client provisioning or independent account proof. New Client 02–04 Launch/Provision remain **blocked** until an independent, game-compatible provider is demonstrated. Sandboxie, the owned fork and VMware are retired for new sessions; old account-bearing data and recovery source are preserved. The v1.18.6 fork is **not** the independent replacement.

| Source | Status and unique responsibility |
| --- | --- |
| [`isolation-owned/`](./isolation-owned/) | **Current editable** Sandboxie-derived v1.18.6 source |
| [`isolation-candidates/sandboxie-v1.18.6/`](./isolation-candidates/sandboxie-v1.18.6/) | **Immutable upstream reference** used only for source-byte comparison |
| [`isolation-native/`](./isolation-native/) | ABI probe/policy helpers; v1.18.5 overlay is **historical** and not a current engine |
| [`isolation-upstream/Sandboxie`](./isolation-upstream/Sandboxie) | Historical v1.18.5 upstream Gitlink; not the current source/build |
| [`runtime-core/`](./runtime-core/) | One Rust lifecycle, identity and provider authority, with legacy VMware operations blocked |
| [`guest/windows/`](./guest/windows/) | Inactive VMware guest/Base preparation and guest-agent source |
| [`distribution/`](./distribution/) | Packaging/release/update contracts; legacy NSIS package intentionally blocked |

**One current source/build route:** the
[Isolation Source Tooling README](../tooling/isolation-source/README.md)
owns v1.18.6 verification, optional unsigned native build and
`workspace/builds/` → `workspace/packages/` staging. The immutable
machine-readable component contract remains
`engine/isolation-native/owned-runtime/build-profile-v1186.json`.

**Historical source is preserved intentionally** for regressions, licenses,
provenance and possible future compatibility review. It is not an executable
fallback or a second active engine. Do not delete, rename or reinstall
source, Windows services, Minecraft accounts or sandbox data as a tidy-up.
