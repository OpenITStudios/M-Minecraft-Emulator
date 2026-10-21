# Emulator Stable Context

Product direction updated: 2026-10-14. This file contains **stable product and repository facts only**. Current tasks, progress and evidence status belong in `planning/development.md`.

## Product and target

**Approved product outcome (2026-10-12):** Four concurrently playable Minecraft Education Windows Desktop clients using four distinct, properly licensed accounts on one PC. **Historical implemented route:** official Windows Desktop installation; Client 01 native and Client 02 through a persistent licensed Sandboxie Plus box named `MinecraftClient02`. This remains a two-client pilot, not four-client support. See the [first-success and subsequent-crash report](docs/validation/two-client-first-success-2026-10-14.md) for revision-scoped observations; it does not establish stable full acceptance or resource savings. Do not modify game binaries, bypass sign-in or copy tokens/profiles.

**Current direction (2026-10-21):** retire Sandboxie and the Sandboxie-derived
owned fork as launch providers. An owned fork is not an independent engine.
Desktop admission rejects new fork profile creation and isolated launches.
Existing session observation, window recovery and normal close remain available
to preserve data. No independent four-session provider is implemented. The independent direction
is a Minecraft-specific Windows engine; the first read-only context inspector
uses Windows process/token/profile APIs without Sandboxie. Per-user RunAs is not
selected as the authentication solution; see
[independent architecture](docs/system/architecture.md) and
[primary-source feasibility](docs/system/independent-isolation.md).
Do not resume fork activation, certificate enablement or VM routes as an implied
fallback. See [retirement register](docs/system/legacy-retirement.md) and
[current continuation](planning/development.md).

The legacy implementation path is:

```text
Host Native Minecraft Education
+ VMware Virtual-01
+ VMware Virtual-02
+ VMware Virtual-03
```


## Branch and execution authority

```text
Local  → GitHub development/source authority
main   → reviewed stable/release authority

REMOTE_GITHUB source work
→ exact-head bounded source evidence
→ unresolved executable/Windows/Minecraft evidence explicitly separated
```

ChatGPT + GitHub work does not require a user PC. A committed test or written acceptance procedure is not equivalent to a passed test.

## Canonical repository ownership

```text
apps/desktop/           UI, Tauri bridges, desktop-native window behavior
engine/runtime-core/    Rust lifecycle, provider, identity, version, readiness, recovery
engine/guest/windows/   Windows Base/Guest Agent setup
engine/distribution/    release, package and installer policy
docs/                   durable product, system and validation explanation
planning/               current development and operations intent
tooling/                developer checks, optional Windows entrypoint, acceptance collectors
workspace/              project-scoped working handoff; sensitive captures ignored
experiments/            non-authoritative bounded research
.agents/                bounded agent procedure
```

One implemented behavior has one owner and one source of truth. The Rust runtime owns VM lifecycle and setup decisions. Svelte presentation may project runtime truth but never duplicate it.

## Stable runtime contracts

- Tauri 2 + Svelte 5 desktop, thin Tauri commands, Rust canonical runtime-core.
- VMware Workstation/Fusion and App Sandbox GPU-PV remain **inactive legacy implementation and guarded research sources**, not active client launch providers. The old VMware-oriented NSIS package is intentionally **blocked**, and no separate native distribution is approved.
- Native Minecraft Education version is the version authority; Base and guest version/lineage must match before unsafe lifecycle actions.
- Base lifecycle: REGISTERED → FINALIZING → FINALIZED. Clone/provision requires a stopped finalized Base.
- Normal stop/start/suspend/restart must not replace a Virtual's account-bearing guest disk. Reprovision is explicitly destructive.
- Guest Agent protocol and tokens remain authenticated and isolated per VM; Windows interactive session is not SYSTEM session 0.
- VM RUNNING, agent reachable, Minecraft process RUNNING, signed-in menu usable and multiplayer READY are distinct observations.
- Virtual profile policy currently caps a guest at 4096 MB RAM and 2 vCPUs; this is not a measured real-world performance guarantee.
- Installed **M-Bedrock Virtual Clients** identity, release asset name, persisted Windows runtime path, Guest Agent protocol and external schemas are compatibility boundaries. Source folder renames do not alter them.
- Self-apply update remains disabled. Release publication/signing requires separate authorization.

## Navigation

```text
README.md → docs/README.md → domain documentation
AGENTS.md → GITHUB_RULES.md → docs/system/implementation-map.md → canonical source
planning/development.md → current work and next single task
docs/validation/acceptance.md → target-machine proof obligations
```

Source provenance: MIVUBI-STD/M-Lazy-Developer Experimental @ 89cb8691f6757d287707bb8cae2d0d810b751404.
