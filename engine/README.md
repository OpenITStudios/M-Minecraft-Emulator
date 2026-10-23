# Emulator Engine — One Active Owner per Function

## Native-only Windows preview installer source — 23 October 2026

This repository now has a **separate opt-in Windows NSIS preview build
route** for the native Minecraft Education window manager. It allows release
compilation of the MC01 native launcher and the existing window presentation
helper without selecting any legacy isolation engine. It does not create an
independent MC02–04 provider.

Source build: `npm --prefix apps/desktop run build:preview`.
Configuration-only check:
`npm --prefix apps/desktop run check:preview-package`.
Both are separate from the blocked original `build:app` path.

The preview uses an independent Tauri name, app identifier, and executable
filename; per-user NSIS installation; RFC 7396 removal of every inherited Guest Agent/VMware
resource and installer hook, and no external VM sidecar. The existing
manager window recovery helper checks its parent process. The release
build enables that helper only with the dedicated Cargo feature and a
build.rs check of its distinct preview identity and scrubbed payload;
ordinary legacy release attempts remain blocked.

**Proof limit:** no Windows/MSVC compiler, NSIS build, signing certificate,
installed preview, Minecraft Education gameplay or four-account acceptance
was executed. No user game data, credentials, releases or `main` branch were
touched. This is a preview source route, not a signed public product release.

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
