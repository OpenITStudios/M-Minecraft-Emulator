---
id: document.system.owned-source-import
class: DOCUMENT
domain: system
role: PROCEDURE
authority: WORKING
lifecycle: ACTIVE
---

# Complete Sandboxie source in the MIVUBI repository

## Current owned v1.18.6 native source — 15 October 2026

The byte-exact **upstream reference** is the complete original 1,627-file
`engine/isolation-candidates/sandboxie-v1.18.6/` tree,
SHA `f690477c07a40810f479f53973334ceae7e0ab13`.
The complete **modifiable MIVUBI source owner** is now
`engine/isolation-owned/`, with all 1,627 files, original
license/source attribution, and precisely 11 edited native source files.
The selected fork is an *exclusive* `SbieDrv`/`SbieSvc` runtime:
it is NOT co-installable with official Sandboxie.

To inspect the exact committed source-byte transformation (read-only):

```sh
npm run isolation-source:verify
npm run test:isolation-source
```

The verifier compares both complete Git inventories, requires the
upstream tree to remain unchanged, rebuilds the existing 24-hunk private
transform using the original 11 source files, and checks the committed
modified bytes. The v1.18.5 import/patch scripts are **historical test fixtures only**, not a second current authority. Edit only `engine/isolation-owned/` for reviewed v1.18.6 changes. The [previous source-import notes](../../planning/history/owned-source-import-notes.md) are archived separately.

**Application-specific component boundary:** see
`engine/isolation-native/owned-runtime/build-profile-v1186.json`.
Keep the full source tree, compile all required core and 32-bit helpers
first, and package a minimal verified MIVUBI runtime rather than
shipping SandMan/Classic Qt UIs. Do not remove uncertain COM, RPC,
injection or build-time dependencies based on source assumptions.

**Unproven:** MSVC x86/x64 compile, WDK x64 kernel build,
signing/installation, service namespace ownership, WAM sign-in,
multi-instance graphics, two-client crash recovery, and four-client
performance. Existing game accounts/boxes and official Sandboxie
are unchanged. An earlier installer reference is not a signed, approved owned engine.
Use the [one current native build procedure](../../tooling/isolation-source/README.md).

