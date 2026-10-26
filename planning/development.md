# Current Development Handoff — Lazy Tester

Local is the only active product branch. Sole current handoff.
Full goal: modular five-profile native control center, manual launches, one active
viewport/four previews, storage/cache management, archival and safe packaging.
Renderer/RAM experiments remain PAUSED; GitHub Actions remains entirely OFF.
Previous goal turn: progress (storage/cache source delivered at the parent below).

## Current bounded outcome — recover a safe failed startup without losing storage

Source parent: 87b6576a36bf75cdd1b024ae3ae8ab9eb4dbf276.
Storage UI, classified HTTP-cache policy and their evidence remain at that commit.
Native preview/recovery evidence remains at d36a536; same-binary controller
transport remains from 14ae0a9. Those milestones do not prove integrated gameplay.

The session owner previously took its sole inactive storage handle before process
creation. Any pre-resume failure could leave the slot unavailable until controller
restart, even when no process had been created or cancellation was confirmed.
It now retains the original validated store until the started transaction exists,
passing noninheritable duplicates of the exact same directory and local-registry
objects to the existing bootstrap. This reuses SlotStorage::duplicate, previously
test-only, without reopening paths, generating profiles or copying account data.

An optional per-attempt in-memory cleanup receipt follows only the original child.
It starts safe, becomes pending immediately before CreateProcess, and is confirmed
only on failed creation or confirmed cancellation/exit of a never-resumed child.
Preflight errors keep the original store. Enrollment/resume-preflight errors drop
the unstarted owner and restore retry only if cancellation confirms its exit.
A resumed or uncertain started transaction remains in the original running owner;
it is never automatically retried or terminated. An unconfirmed pre-resume cleanup
removes retry admission, reports uncertainty and preserves existing processes.
There is no controller reset, automatic retry, process reopen, path-based recovery,
persistent readiness flag, public protocol change or second lifecycle manager.

The existing configured creation entry remains a wrapper using no receipt;
historical callers retain their behavior. The fixture handoff's child initializer
explicitly uses no receipt. The accepted C++ compatibility payload, account/world
paths, preset writer and foreground144/background10 policy are unchanged.

## Executed evidence

- Three new targeted native Windows tests PASS:
  - relative-path preflight refusal and failed CreateProcess preserve retry;
    dropping the own suspended executable confirms exit before restoring retry;
  - real cancellation denial using a reduced-rights handle refuses retry; a
    simulated resume boundary also refuses it. Test teardown alone terminates
    only the retained own never-resumed fixture processes, not games;
  - duplicate private/logical/TEMP identities remain the same, a synthetic world
    sentinel survives failed creation, and another duplicate can be obtained from
    the retained original store. Only a newly created fixture registry leaf is
    removed during teardown.
- Two existing native cancellation/drop regressions PASS.
- All own executable fixtures remained suspended and never executed their entry
  points. No game, UI, installed profile or account-bearing registry key touched.
- Windows x64 release build PASS; existing 104 desktop warnings remain.
  Manager SHA-256: 45c08b28b3f3a64fabbd7bf7fc17aca775510ade524d40989775418c5062fe2b.
- Frontend/private wire schema unchanged. Prior 215-test frontend/Svelte/build
  evidence belongs to 87b6576, not a newly executed suite for this increment.
- No dependency versions/lockfile changed. No Minecraft, candidate GUI, installer,
  Computer Use, UAC or trace executed.

Actual Minecraft startup-failure/retry while another profile plays: NOT_EXECUTED.
Cancellation failure intentionally stays quarantined; no forced recovery is claimed.
The receipt is scoped to one attempt, not a security boundary or game-ready proof.

## Remaining full-goal work

- Integrated adoption/preview/storage/gameplay acceptance. Native DWM registration
  is not Minecraft pixels/input/FPS proof. Offscreen Education host simulation,
  dialog/DPI/minimize/crash/restart and real cleanup/login persistence remain
  NOT_EXECUTED. Metadata adoption may briefly expose standalone game windows.
- Taskbar/browser-shell acceptance and Task Manager grouping remain unproven;
  Minecraft still owns separate OS processes.
- Installed Lazy Tester branding/identity migration and native payload packaging.
  Current recipe/capabilities bind to reviewed local game/profiles.
- Physical legacy relocation after shared consumers retire. Native DLL source in
  bootstrap-ordering is active and must not be archived as retired test code.
- Primary D: checkout is older and has not been reset or overwritten.
- Same-binary controller reconnect is implemented; cross-binary/version handoff
  is unsupported. Do not replace an installed executable during live sessions.
- Cache cleanup remains explicit HTTP-only disk maintenance; unsupported caches
  and unknown helper quiescence do not justify broad TEMP/profile deletion.

## Next — one action

Implement the native Lazy Tester packaging/branding route through the existing
distribution and Tauri owners, with an explicit installed-identity/data migration
decision and verified payload inclusion. Keep local profile capabilities private
and refuse to package them as portable account/profile material.

## Preserved baseline

Archive branch codex/archive-before-lazy-tester-20261026 pins
d166b5c8b24e8435f2c84675cb00a61c753aeb6a. Only Local receives product work.
The prior five-client pilot was operator-confirmed with five licensed accounts in
one Education Host World, persistent login/world data and 144/10 FPS after restart.
New UI/controller acceptance remains separate. No account/world data was migrated,
reset or deleted. Approved presets remain integrated; no real options.txt was
written during this increment.