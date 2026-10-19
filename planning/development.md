# Current Development Handoff — M-Minecraft-Emulator

**Repository:** `OpenITStudios/M-Minecraft-Emulator`; source authority is the GitHub `Local` branch. Do not rely on historical Next/PASS claims.

## Current batch — explicit read-only discovery for all four slots

The active desktop adds one operator-requested `Inspect all four slots` action in Settings → Diagnostics. `instance_discovery.rs` projects the existing Windows process name-match observation and official Sandboxie Client 02 observation to four IDs from `instance_catalog.rs`. Both collector calls are serialized under the existing Rust manager lock; there is no new helper process kept resident, watcher, cache, periodic polling or provider mutation. If one collector fails, that slot becomes `UNKNOWN` with no candidate count, while the other successful observation remains available. The frontend validates a strict four-slot schema.

**Evidence ceiling:** 01 name matches are **not** attributed to Host 01: other editions and sandboxes may match. 02 returns only observed box process candidates, never sign-in or playable-world proof. 03/04 remain `PROVIDER_NOT_AVAILABLE`; zero candidates never proves that a game has completely stopped. Collector errors remain `UNKNOWN`, rather than yielding misleading empty instances. Neither collector may authorize launch. No action uses this overview to authorize Minecraft launch, account access, provisioning, process throttling or driver activation.

## Established source architecture

- Four immutable slot identities: `apps/desktop/src-tauri/src/engine/instance_catalog.rs`.
- One existing Rust IPC admission rule table, shared process identity parser and official 01/02 provider with fresh Sandboxie membership/identity verification.
- MIVUBI-owned Sandboxie v1.18.6 editable source: `engine/isolation-owned/`, immutable byte-exact reference: `engine/isolation-candidates/sandboxie-v1.18.6/`.
- Sole build-only engine procedure: [isolation-source README](../tooling/isolation-source/README.md). Older v1.18.5/VMware routes are historical, not an active fallback.
- One local Windows checkout: `DEV.cmd bootstrap` → `doctor` → `preflight` → `dev`. Project-controlled download/cache/evidence belongs under `workspace/`; OS services, C++ Build Tools, Minecraft and Microsoft user profiles remain external.

## Exact-SHA evidence

**New bounded resilience change:** read-only overview keeps independent
native/official Sandboxie candidate evidence when the other collector
fails. No collector failure is translated into zero candidates; no new
controller or watcher is introduced. Unit and TypeScript source contracts
cover both one-source and two-source failures. The last exact-SHA
`9f249656075c1fbcf3c2b8b6e9f088b719fa0368` CI is not assumed to
verify this new change; check its own Full Source result first.

The new discovery source on `92f164f78e2dc7be07ecf201e4f7412b12ad58ba`
[failed Full Source CI](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/runs/39265234252)
in the desktop Rust **test** compilation only: `result.slots.map(...)` moved
the non-Copy array before later assertions. The affected test now iterates
by reference. All other source stages, frontend tests, runtime Rust tests and
format checks on that SHA passed; this correction needs its own exact-SHA CI
before debug desktop compilation can be claimed.

The previous `Local` commit `106d925f92ecd2044f128635c58fa68e25fbf867` **passed** twelve Windows Full Source CI stages, compile-only debug desktop build and artifact upload: [run 39262164288](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/runs/39262164288). This new discovery batch needs its own matching SHA CI. Success in CI is not Windows UI startup, real Minecraft login, running v1.18.6 driver or stable four-client acceptance.

## Unresolved and next action

Actual owned v1.18.6 driver build/sign/install in an authorized clean lab, Client 03/04 provisioning and gameplay, four separate licensed sign-ins, Client 02 idle crash, and independently verified external FPS limiter remain **NOT EXECUTED / UNKNOWN** as appropriate. Do not alter Minecraft render/simulation distance, injection paths, game settings, official Sandboxie service, credentials or accounts.

**Next:** inspect this exact SHA's automatic Windows Full Source and desktop compile result, fix demonstrated regressions only, then extend authenticated provider-owned discovery when its native v1.18.6 capability is validated.

Historical design/CI/procedures: [planning/history](./history/development-through-2026-10-19.md).
