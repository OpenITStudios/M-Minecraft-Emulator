# Contributing to M-Minecraft-Emulator

## Branch and operating mode

`Local` is active development; `main` is stable/release-only. Product source changes use the [Emulator Development procedure](.agents/skills/emulator-development/SKILL.md), [GitHub Rules](GITHUB_RULES.md) and [Development Discipline](docs/system/development-discipline.md).

ChatGPT's normal route is GitHub-first and source-verifiable. Do not make user-local Node/Rust/VMware execution a prerequisite for remote repository editing. In a fresh chat, a GitHub URL plus "amati" is read-only; recover ongoing work by topic, exact `Local` source, relevant commit `Work`/`State`/`Proof`, and `planning/development.md` rather than chat titles or an unrelated latest commit.

## Before any change

Find the first wrong canonical owner via `docs/system/implementation-map.md`. Identify its consumers, verify positive/negative impact, and preserve stable runtime identity and account-bearing data. Do not add a second path, helper, state store or compatibility shim for an existing responsibility.

## Code and tests

- Keep typed Tauri command adapters thin; lifecycle and provider policies belong to Rust.
- Follow canonical module responsibilities; native profile/world mutations require verified ownership and explicit authorization.
- Unit tests cover invariants; a written regression test is `NOT_EXECUTED` until actually run.
- Prefer targeted source verification over automatic expensive CI.
- Prefer one named implementation and Git history to `-new`, `-final`, `-v2` copies of internal code. Public schema versions remain explicit.

## Repository hygiene

Never commit VMware disks, guest/session images, Microsoft sign-in material, Minecraft worlds, signer keys, machine telemetry, build directories, generated support captures or `node_modules`. See `SECURITY.md`.

## Commit and handoff

Commit one coherent change to `Local`; cite exact SHA and proof level. Record significant ongoing work in `planning/development.md`. Stop when the requested outcome and its source-level acceptance are complete.
