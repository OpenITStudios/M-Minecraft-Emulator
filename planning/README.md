# Planning — Current vs Historical

- [`development.md`](development.md) is the **single current source of
  operator intent, evidence boundary and stop/reopen decision**. As of
  29 October 2026, the temporary LazyTester RAM/resource-efficiency source
  stage is **DONE**, with **no active next action** for that stage; runtime
  RAM/FPS/VRAM and five-client acceptance are deferred, not marked PASS.
- [Historical RAM research](../docs/system/native-memory-efficiency-research.md)
  and the [inactive implementation reference](../docs/system/native-efficiency-implementation-playbook.md)
  provide context only. Their older "Active", "Current" and "Next" language
  is not current authorization.
- `operations.md` and `projects.md` describe distinct scopes; neither
  overrides the active branch's source or the current development decision.
- [`history/`](history/development-through-2026-10-19.md) contains non-authoritative
  archived context. The full prior 29 October handoff is available
  [in Git history](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/5b59c5d693c151428cd15424c3daee30fb63e631/planning/development.md).
  Do not follow historical plans, launch commands or archived `Next` fields.

Only `Local` is development-authoritative; `main` remains release-reserved.
Current source, root `AGENTS.md` and `GITHUB_RULES.md` win over stale prose.
No CI, local-PC test or automatic monitoring was authorized by this
documentation cleanup.
