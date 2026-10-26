# Current Development Handoff — Lazy Tester

Local only; main is release-reserved and GitHub Actions remains OFF. Renderer/RAM
experiments remain PAUSED. Physical Development/Archive organization is retained.
Current bounded outcome: functional Settings and configurable active/background FPS.

## Implementation

Source parent: aac34f49c12f7faf569e20d8d227b5652c8ad96b.
The operator selected Settings/usability, then explicitly requested player-defined
FPS limits with 144/10 defaults and an efficient design.

Mounted English Settings now provides two numeric inputs, Save, Restore defaults,
unsaved/loading/success/error feedback, the existing graphics preset action and
a Storage shortcut. The existing typed session bridge handles InspectSettings,
SavePerformance and ApplyPreset. Settings metadata is an optional additive private
snapshot field; old ordinary snapshots remain accepted. Backend validation is
independent of UI validation. Background may not exceed active; both are integers
1..1000. No unlimited mode or per-profile overrides were introduced.

The existing manager_settings owner atomically saves only two numbers beside the
executable in performance.json. Missing settings mean 144/10 without a file write;
invalid existing files are preserved and reported. Root App refresh retains this
file and the data locator. No account/token/world/catalog contents are copied.

Save uses the retained session actor when connected; without a controller, it
writes preferences without preparing profiles or spawning a controller. Existing
incompatible-controller and uncertain-operation gates remain fail closed. Each
profile gets the saved pair at its next explicit launch. Running games retain
old limits; no force-close, live-update worker, settings watcher or extra limiter.

Native presentation V3 encodes the pair in the previously reserved DWORD of the
existing 120-byte private capability. V2 still means 144/10; malformed limits and
unknown versions are rejected. Storage/account ABI and process/event identity
validation are unchanged. The existing Pacer reads immutable integers, with no
new per-frame I/O/IPC/polling. Default-OFF RAM/heap experiments were kept OFF.
Newly built DLL SHA256 pins:
compatibility 9d1cf60673c1ebbc0d94c30c1e28e226aa4429653901e5abd6e95400457ef8cb
enroller 6e7c000bee2dcf043d372317c77a04078ca12451c7273a18bd54e8f89e62af2a

Apply to all profiles reuses the original game_preset writer and exact pinned
stores, under the sole session actor. Live/unattributed games or uncertain stores
refuse the action. All five inputs are prepared before writes; updates/backups
remain idempotent. Failures can leave some profiles updated; retry is safe, not a
cross-profile transaction. Simulation distance/unowned options remain untouched.

## Executed evidence so far

Logs and native build inputs/output: workspace/runs/settings-20261026 and
workspace/builds/settings-fps-20261026. MSVC Release /W4 /WX /permissive- builds
bootstrap-compat, bootstrap-enroll and frame-pacing-control successfully using the
existing clean pinned Detours source. Configurable-Pacer-only self-test PASS:
actual 60/5 deadlines, foreground promotion/stop, invalid limits, v2 defaults,
v3 decoding and unknown-version rejection. No Minecraft or window was used.

32 frontend tests PASS; Svelte zero errors/warnings; frontend build PASS.
Targeted Windows Rust tests PASS: performance persistence/validation (including
corrupt-file preservation), settings protocol/admission, five-profile graphics
updates/invalid-last-profile preservation/idempotent retry and existing writer
regressions. Exact revision Windows product/NSIS packaging remains to be recorded.

Do not claim a globally green source suite: the preceding work recorded a broad
DACL disposable-fixture refusal, oversized-pipe fixture hang and existing rustfmt
drift. Those unrelated failures were not changed or hidden by this work.

No product GUI, installer, Minecraft, UAC, service/driver or Computer Use launched.
No live preset/configuration was applied to real profiles. New default/custom FPS,
login/world persistence and gameplay on these DLL bytes remain NOT_EXECUTED.
Prior five-client pilot acceptance is historical, not acceptance of this revision.
Cache deletion and unclassified workspace cleanup remain deferred. Data migration,
partial-setup recovery and installed lifecycle acceptance remain unfinished.

## Next — one action

Build/export the exact clean Local Settings candidate, stage it only if App is
not running, then record source SHA, payload hashes and actual proof limits. Do
not run Minecraft or resume RAM experiments incidentally.
