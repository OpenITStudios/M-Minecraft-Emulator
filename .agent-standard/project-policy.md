# M-Minecraft-Emulator — Project Policy

**Scope:** repository-local restrictions and domain routing layered under byte-identical AgentSkill shared rules. `.agent-standard/profile.json` selects this policy. Shared governance Skills are not additional Minecraft execution controllers.

## Branch and held validation
- `Local` is the only development branch; `main` is release-reserved. Do not promote, tag, create branches, merge or deploy without explicit separate authority.
- GitHub Actions CI is fully disabled, including any old main-only exception. Do not add workflows or dispatch/reintroduce verification. Source-test execution is on hold. ChatGPT + GitHub authors and reviews source/fixtures only; local tests require an actual user-PC agent and separately selected validation phase.
- Never infer Windows/Minecraft readiness from source, prior VM/Sandboxie evidence, or a GitHub branch called Local. Native product runtime, five Minecraft Education profiles, independent instance control and installed-package behavior require matching real native observations.

## Product source and Skill owner
- Current product is **Lazy Tester**, not the retired VMware/Sandboxie implementation. Preserve the five independently managed Minecraft Education Windows profile design and actual native session controller.
- Keep `.agents/skills/emulator-development/SKILL.md` as the sole project engineering Skill. `oits-*` files are shared governance procedures, not alternative native/map producers or a new local runtime.
- Resolve current source owner via `docs/README.md` and `docs/system/implementation-map.md`; use `CONTEXT.md` for stable facts and `planning/development.md` for the active workstream. Source has priority over stale planning/history.
- Preserve Rust runtime core, Tauri desktop, installer identity, data formats, profile/world storage, saved snapshots, compatibility and user data. Never silently remove any old account-bearing disk or protected Minecraft material.
- Preserve the native Lazy Tester installation and source acceptance boundaries. Only an actually connected target can prove native gameplay, installed app or UI acceptance.


## Related Lazy-family task routing

When an explicitly authorized task involves another Lazy Minecraft product, consult this repository's `README.md` **Related Lazy-family handoff** section to locate the actual artifact and evidence owner. This is a navigation pointer, not another agent/router. Read the other repository's current selected ref and own local policy only when the handoff is genuinely required. Preserve exact asset/world identity, edition/version, user approval, evidence tier and every local test/CI hold. A relationship never grants automatic transfer, cross-repository writes, native host access or gameplay approval.

## Security, provenance, preservation
- No account/Microsoft credentials, OAuth tokens, signing keys, machine identity values, private worlds, snapshots or secrets in Git/source/logs.
- Do not run a CLI, installer, source tests, cloud runner or GitHub Action merely to prove this standards migration. Never manufacture source/native PASS.
- Exact prior root `AGENTS.md` and `GITHUB_RULES.md` are retained in `.agent-standard/history/` as non-active historical material. Consult only relevant original domain sections if a preservation question remains; applicable stricter project constraints survive, but old generic policy cannot supersede the shared core. Do not replace existing nested instructions or the `emulator-development` Skill.
