# Current Development Handoff — Lazy Tester

Local only; main/publication untouched; GitHub Actions entirely OFF.
VMware, Sandboxie and RAM/renderer experiments remain PAUSED.

## Current outcome — automatic launcher reconnection

Parent 259da6f6752bbf6a11eab98e3e97d83498138b7e. The user again observed
No saved launcher connection after opening the updated App and requires prepared
launchers without repeated setup. Existing App read-only inspection returned
configured=true from checkout and App working directories. The reviewed binding
predates the update. The new headless initializer also returned configured=true
through the canonical startup owner on a worker thread. No account/world contents
were read, copied or reset. No profiles were replaced.

Confirmed red reproducer: compiled Svelte receives a temporary missing binding
report followed by an available connection. It never retries binding because
unconfigured status disables session polling. The test timed out after seven
seconds waiting for Ready without operator setup. The specific first native miss
in the user's GUI was not reproduced; its origin remains UNKNOWN. A successful
CLI result does not prove that old GUI healthy.

The existing Svelte startup retries canonical initialization three times before
exposing recovery; transient data-location failures also retry. During startup
it displays Preparing launchers, without manual setup actions. Persistent failure
stays disabled and truthful, offering Retry connection and Data Storage rather
than instructions to create replacement profiles. Identity, catalog, registry,
reviewed-image and idle guards remain. No cached ready flag, second store,
account automation or automatic game launch is added.

Product-only --lazy-tester-initialize invokes the same native profile_binding
adapter on a worker, accepts no path/options, and never launches games. Default
local packaging executes it after guarded App staging and requires a configured
report before announcing readiness or creating shortcuts. Export-only builds do
not initialize runtime data; installer hooks and identities remain unchanged.

## Evidence

Typecheck and production frontend build PASS; 34 frontend tests PASS; five targeted
Rust binding tests PASS. The original red startup regression now recovers without
setup clicks. Repeated reload and transient binding/location cases require no
Choose/Locate/import/Launch. Compiled-Svelte regression also checks seven window
sizes, largest-fit 16:9, matching native IPC at simulated 100/125/150% browser DPI,
30-second quiet polling, queued actions, High/Low/FPS and existing-data recovery.
These mock-native tests do not prove actual game mouse/DPI/license acceptance.

A real Windows debug build ran canonical initialization successfully against the
existing reviewed connection. This validates connection metadata, not game login.
Clean Windows/NSIS candidate built from d2399c0acf7053a71b238a7136a95e3376925ce1:
workspace/packages/lazy-tester/0.1.0-d2399c0acf70-20261026T140100188Z.
Installer SHA256: 7fb7127065f05accdb605fd1676cfa2af1b22f3ef93453dfeecc0d99b27e72dd.
Log: workspace/runs/play-settings-implementation/automatic-reconnect-package.log.
The release initializer succeeded twice against the reviewed existing connection;
binding bytes remained identical and no game was launched. This is metadata proof.

The user confirmed normal closure. Process inspection found no running manager;
guarded staging then completed successfully without terminating any process.
App now contains d2399c0acf7053a71b238a7136a95e3376925ce1. Its actual initializer
returned configured=true, issue=null before delivery. All four staged payload
hashes and the candidate installer hash were verified. Desktop/Start Menu links
were recreated and verified with direct App executable, App working directory,
empty args and embedded icon. Previous App/data pointers remain preserved by the
canonical stager. No game or GUI was launched during this installation step.
Receipt: workspace/runs/play-settings-implementation/automatic-reconnect-staged.json.
The ignored stage-automatic-reconnect.mjs orchestrates the existing stager and
initialization/hash/shortcut checks; it is not a new runtime owner.

## Preserved App and prior accepted layout

The preceding App contained source 835f40d25a6017a6c5012d37defa15779a42ef43.
Its Windows/NSIS construction and guarded staging passed. All four payload hashes
and installer hash were verified. Normal Desktop/Start Menu shortcuts target
App/lazy-tester.exe directly with App working directory, empty args and LT icon.
Prior App and data pointers are preserved in ignored local backups by the stager.
Candidate: workspace/packages/lazy-tester/0.1.0-835f40d25a60-20261026T134529139Z.
Installer SHA256: e822d5b32a6587527b11d2a6ce027853cda952f60e49413c7b8d40ba36c7e532.
Log: workspace/runs/play-settings-implementation/comfortable-layout-package.log.

Play centers the existing native viewport at largest-fit 16:9. The same DOM bounds
feed native placement/aperture/input and preview IPC. The outer stage is observed
for position changes. Compact combined header and a five-session dock capped at
1320 logical pixels preserve game space; previews are 16:9. Text is larger.
The normal initial window targets 1280x900 logical pixels, constrained to monitor
work area/decorations/DPI by app_bootstrap.rs. Maximize stays available. Window
history and a separate interface-scale preference are not implemented.

General owns High/Low and saved 144/10 defaults with configurable FPS. The original
manager_settings.rs and game_preset.rs remain the preference/write owners. Manual
graphics changes and cache cleanup require all games stopped. Limits apply on
next launch. Data Storage uses bounded real metadata; there are no fabricated
world sizes. Quiet background polling must not flash controls or reposition
unchanged native geometry. Navigation/launch clicks during inspection queue.

## Boundaries and next action

Clear All Data, world export, reset/restore settings, background audio, automatic
game shutdown and data migration remain unavailable. Preserve all accounts/worlds,
preference pointers, native payload pins, initialized submodule and second worktree.
Signing NOT_VERIFIED; installer execution and new-layout native mouse interaction,
gameplay/license and multiplayer acceptance NOT_EXECUTED. Publication DISABLED.
User-reported earlier normal play is not independent four-client acceptance.

Next: observe startup through the normal shortcut on the user's display; verify
automatic reconnection and normal session use. The original first native missing
read remains UNKNOWN; successful initialization is not visual/gameplay acceptance.
