# Lazy Tester Desktop — Current Product

This is the current Tauri 2 / Svelte 5 / TypeScript / Rust application.
Capture source candidates are capability-gated: selected-instance WebP
screenshot and independent-slot HEVC NVENC/MKV recording (RTX 3070-compatible codec target). Neither reviewed worker
binary is approved or shipped; controls remain unavailable until review.
Managed close (desktop or agent CLI) excludes active recording on selected
slots; a terminal unconfirmed-cleanup receipt disables subsequent starts.
Standalone settings saves exclude concurrent controller startup using its
existing lease. Audio, performance and storage settings share a bounded
opened-file reader with reparse-point refusal on Windows.
See [recording contract](../../../docs/system/recording.md) and
[GPU capture worker](../../tooling/agent/native-gpu-capture/README.md).
The mounted UI is `src/LazyTesterApp.svelte`, not the historical
`UnifiedManagerApp.svelte`. Play keeps the selected game in the workspace; manual window separation is not offered.
Native recovery remains available through the original controller. Healthy running
launchers use their preview and selection highlight; connection and exceptional
status remain visible when attention is needed. The AI & Testing disclosure
appears only for attention or terminal Probe results, not routine activity. General Settings prioritizes frame
limits and audio. All five launchers use the same reviewed Low graphics preset
at next launch; individual graphics controls are retired. Legacy graphics.json
and existing options backups are preserved. Data Storage retains Reset/Restore
and maintenance, without per-profile Texture Streaming or Improved Input Response controls.
Profile data-location inspection
runs on Data Storage navigation, not when opening General Settings.
The launcher action strip hosts selected-game close review and pending-close
status in normal flow and collapses when empty. Bottom/Right workspace layout
uses the existing geometry scheduler and falls back to Bottom when space is insufficient. Native close
admission and recovery remain unchanged. Every launcher reserves a bounded
health row so a warning does not resize the rail or shift native previews.
Unconfirmed launch receipts are scoped to the affected slot and block its
retry until a fresh session inspection succeeds.
Native Windows Minecraft Education has five
fixed profile identities and an explicit manual launch path; the old
Sandboxie/VMware provider route is retired.

## Canonical source owners

| Area | Owner |
| --- | --- |
| Play, Settings/Storage state and UI action dispatch | `src/LazyTesterApp.svelte` |
| General Settings forms and read-only product details | `src/GeneralSettings.svelte` |
| Data Storage, inventory and confirmation presentation only | `src/DataStorage.svelte` |
| Pure pending-action eligibility and inspection scheduling | `src/app/inspectionScheduling.ts` |
| UI workspace geometry snapshot comparison | `src/app/workspaceGeometry.ts` |
| App titlebar drag and normal exit request (no custom minimize/maximize buttons) | `src/WindowTitlebar.svelte` |
| Fixed five-launcher rail, visual actions only | `src/LauncherRail.svelte` |
| Shared visual tokens, shell, launcher and settings styles | `src/lazytester.css` |
| Health and recovery guidance | `src/SessionHealthNotice.svelte`, `src/app/presentation/` |
| Authored diagnostic categories, stable support codes and user error messages | `src/app/presentation/sessionFeedback.ts` |
| Selective desktop issue summary and checkpoint | `src/app/presentation/issueReport.ts`, `src-tauri/src/engine/issue_report.rs`, `src-tauri/src/engine/report_store.rs`; mounted feedback and General Settings Copy report |
| Validated desktop protocol | `src/app/bridge/lazyTester.ts`, `workspaceSurface.ts` |
| Only raw Tauri invoke boundary | `src/app/bridge/invokeRuntime.ts` |
| Game lifecycle / retained process identity | `src-tauri/src/engine/independent_bootstrap/session_controller.rs` |
| Authenticated same-user controller transport | `src-tauri/src/engine/native_clients_session.rs` |
| Native window recovery and DWM previews | `src-tauri/src/engine/workspace_surface.rs` and child modules |
| Profile data location, catalog and migration | `src-tauri/src/engine/independent_bootstrap/product_profiles.rs` |
| Installer and preservation policy | `../../engine/distribution/lazy-tester-package.json` |

An observed running process or registered thumbnail does **not** certify
gameplay, sign-in, visible pixels or multiplayer. Minecraft owns login,
worlds and rendering; Lazy Tester never stores credentials or force-closes
unverified games.

## Selective issue report

General Settings provides **Check issue report** on demand, changing to
**Copy report** after a UI-observed problem. This also exposes native-only
evidence without background availability checks. A healthy check returns
no report; unavailable native evidence is not reported as healthy. This is a local,
sanitized support summary, not a security audit log, crash dump or gameplay
acceptance. No new polling, background IPC, service, upload, raw error
text, screenshots, process identifiers, account or world data. Existing
feedback, recovery and native admission remain unchanged. Clipboard
failure leaves selectable JSON text; live copy requires General Settings.

Compact combined export schema 2: `v` schema; `scope` desktop-process; `issues`
contains at most 48 distinct code/slot entries, first observed wall-clock
Unix seconds `t`, optional launcher slot `s`. State observations are
deduplicated until an explicit, scope-matched positive receipt; they do not have an attempt count.
Discrete recorded capture/preference failures include attempt count `n`
(saturating) and most recent observed time `l` when repeated. An overflow
sets `truncated:true`; no stored row is evicted or overwritten. Codes
describe unconfirmed outcomes or attention, not necessarily root causes.
No report is produced for normal success, polling, known preflight
blocks, unsupported platform or native exit rejected before release.

Existing LT codes are defined in `sessionFeedback.ts`; additional report
codes:

| Code | Observed problem |
| --- | --- |
| LT-GAM-05 | Game window unavailable/not responding |
| LT-CAP-01 / 02 / 04 | Screenshot / recording start / recording stop not confirmed |
| LT-CAP-03 / 05 | Recording status unavailable / observed Failed receipt |
| LT-AGT-01 / 02 | Native release unconfirmed / startup reconciliation needed |
| LT-AGT-03 / 04 / 05 | Probe checkpoint unavailable / Probe reported attention / earlier agent status stale |
| LT-SET-07 | Web cache preference save unconfirmed |

The optional `native` section is one explicit read of a 16-row, current-process Rust request-client summary. `native:null` means no retained native rows; `nativeUnavailable:true` means the read failed. Together with 48 UI rows, the report retains at most 64 distinct rows. Native `c` is the source-classified code, `op` is the actual admitted request name, optional `s` is the slot, optional `os` is the raw numeric OS error (not a message), `t`/`l` are observed Unix seconds (zero if unavailable), and `n` counts discrete failed attempts. Source owner: `native_clients_session.rs`, with `issue_report.rs` retaining bounded evidence. `IPC_WORKER_START_FAILED` means no worker started; `IPC_REPLY_TIMEOUT`, `IPC_REPLY_LOST`, `IPC_TRANSPORT_FAILED`, `IPC_INPUT_RECEIPT_INVALID`, `IPC_TARGET_RECEIPT_INVALID` and `IPC_INSPECT_RECEIPT_INVALID` preserve unconfirmed outcome; no automatic retry or inferred cause. UI and native rows are separate observations, not additive incident counts. This read cannot collect another CLI/controller process's memory. No exact build identity yet; no new session wire fields or periodic reads.

Optional `r` is the observed recovery/reconciliation time for the current issue episode; absence means no matching positive evidence, not a proved continuing failure. Optional `e` counts recurrences after confirmed recovery (saturating); recurrence clears `r`, updates `l` and never erases the first timestamp. On timeout the original request worker receives a best-effort synchronous I/O cancellation request; later admission may repeat cancellation, never the mutation. The pending worker stays retained until result or disconnect. The server bounds request-frame, reply-write and ACK phases to 5 s each, independent of admitted controller work; cancellation must settle before buffer reuse or reconnect. Native `r` is recorded only after the existing uncertainty gate accepts a full Inspect; this reconciles transport, not the prior mutation or gameplay. Worker-start failure is excluded from that reconciliation. Desktop recovery is limited to current successful Refresh, validated recording status, a non-cancelled screenshot receipt, confirmed selected-window restoration, and matched web-cache preference save. Recovery describes a later successful observation/operation at the same report scope, not success of the original failed action. Other codes remain without recovery evidence; UI warnings disappearing, partial replies and timeouts do not establish recovery. No extra polling or report rows for normal success.

### Bounded local checkpoint

Only the ordinary Windows product desktop enables writes after its existing
exclusive instance lease. Native issue changes checkpoint after releasing
the native request gate; UI submits one coalesced snapshot on new issues,
recovery, recurrence, first overflow or power-of-two repeat counts.
Copy retries current UI/native evidence (including a native-only checkpoint);
clean exit flushes native and last submitted UI counts. No new timer, service
or automatic retry loop. Copy indicates `storageUnavailable:true` if its
checkpoint attempt fails. Native publication is synchronous on the exceptional
request path after the session transport gate is released; an outer manager
admission gate may still be held. Disk delay and actual overhead are unmeasured.

`Data/Reports/issues.json` holds the latest issue-bearing desktop lifetime:
64 rows maximum, 16 KiB hard cap. Healthy runs preserve it; the next
issue-bearing lifetime replaces it, without merging old recovery episodes.
Identical snapshots skip publication only after comparing the actual bounded file; a missing final is recreated. A fixed exclusive `issues.pending`
stage is flushed and moved within the pinned directory; interrupted or
unrecognized files block publication and require manual review, never
automatic deletion. Replacement requires the complete known schema,
including bounded UI/native rows, typed native operation and code vocabulary,
and no extra fields. This does not provide hostile same-user race attestation. At most two reporter-created files (16 KiB each).
Historical evidence is available on disk, not imported into live UI.
A crash before checkpoint or between repeat checkpoints may lose evidence.
CLI/controller/helper and startup diagnostic processes never write.

### Development feedback review

Use `DEV.cmd report --json` in the authorized local developer checkout to
review the latest fixed `Data/Reports/issues.json`. The existing pinned Node
developer controller calls `tooling/developer/report-review.mjs` only on
request; no installed-product dependency or runtime analysis. It returns up to
12 prioritized issue observations, grouped investigation routes to current
source owners, and explicit omitted/truncated/unknown coverage. UI/native
observations remain distinct, recovery does not prove the original action,
and the reported build and root cause remain UNKNOWN/NOT_ESTABLISHED.
No report means no saved evidence, not healthy. Source mapping is a review
starting point, not automated diagnosis or permission to execute a fix.

The summary does not include success denominators, unobserved native
failures, exact root causes or historical process exits. It cannot
measure performance, availability or failure rates. Existing developer
performance collectors and private evidence remain their original owners.
Reference patterns: [OWASP logging guidance](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html),
[Google SRE monitoring](https://sre.google/sre-book/monitoring-distributed-systems/),
and [OpenTelemetry recording errors](https://opentelemetry.io/docs/specs/semconv/general/recording-errors/).
No new SDK or logging framework is installed.

## Profile layout and legacy compatibility

Minecraft version compatibility is checked for every managed launch. An
unreviewed version leaves existing configured launcher inspection and Settings available, with
the stopped profiles marked as needing attention. Launch remains blocked
until its exact image/callsite is reviewed. The idle controller retains no
game executable handle; each launch resolves the current signed installation
again. Running games retain their original image pins until they exit.
The first managed launch after idle hashes the game executable; later launches
in the same active cohort reuse the already reviewed recipe only when every
original child still retains the same immutable image pin and all recipes
agree. No idle cache or persisted admission; a replaced file or last exit
requires full review again. Fresh signed registration, updater state, exact
file identity and DLL admission remain mandatory for every launch.
The official Windows Desktop updater remains the only game update provider.
The existing two launch resolver calls check the updater state before and
after taking the executable pin (no additional PowerShell process or
background polling). Known busy state before pinning avoids a new pin.
Queued/running or unobservable updater state defers new launch; a missing
or disabled task may be intentional organization policy and is not changed.
This observation is not an installer lock. Active managed games retain
their original image pins; a different installation generation is refused
until the original sessions close normally. Unknown executable hashes remain
unsupported; recipe data stays in the reviewed build, not an unsigned
download. No automatic uninstall/reinstall or world/account migration.

Official guidance: [Update to a new version](https://edusupport.minecraft.net/hc/en-us/articles/360047705032-Update-to-a-New-Version-of-Minecraft-Education)
and [Windows installation](https://edusupport.minecraft.net/hc/en-us/articles/13106858087956-Windows-Installation-Guide).
General Settings shows only the last observed Minecraft version and, when
needed, one actionable notice. The internal updater state and reviewed-image
match remain validation data, not permanent UI rows or a positive readiness
claim. Settings entry observes installation independently before the
session settings request, so a native payload hold does not itself prevent
the installation check. The report is refreshed on Settings entry or
explicit header Refresh, not passive session inspection, app startup or
a new timer. The explicit observation reads updater state with the first
signed resolver result; a known busy updater skips pinning and hashing.
Otherwise it rechecks registration and identity under a temporary pin;
if the updater became busy, it skips hashing and drops the pin. The UI
receives no path or hash, and a deferred review is not an unsupported image.
Manual Refresh serializes the existing UI and background observation lanes. A match does not prove native ABI or gameplay;
a failed check clears the displayed receipt without invalidating unrelated
session state. No action starts the updater or changes task policy.
Minecraft Education support documents that opening a game can cancel an
automatic update: keep games closed after Windows sign-in until the official
update finishes. See [support explanation](https://edusupport.minecraft.net/hc/en-us/community/posts/39644204051732-This-v1-21-91-Update-Has-Really-Messed-Things-Up).
Do not expose a manual task-run button until exact installed task provenance,
permissions and safe coordination with active/unmanaged games are verified.
A task name or Ready state is not that proof; no elevation or task-policy
change is authorized.
This does not promise automatic compatibility with future Minecraft builds.
Task visibility, installer coordination and runtime acceptance remain
NOT_EXECUTED; no task is started, enabled, disabled or created by Lazy Tester.

The reviewed Education 1.26.33.0 image selects its own observed mutex caller;
the earlier reviewed image retains its original recipe. The existing one-shot
`--lazy-tester-agent-cli` adapter also accepts `close <1..5>` and `close-all`,
using the retained controller's normal close path. These commands never force
terminate Minecraft, sign in, reset profiles or replace their data.

- Current schema 2 profiles: `Data/Launchers/Minecraft-1..5`.
- Catalog: `Data/profiles.json`.
- Canonical pointer: `Data/Settings/data-location.json`.
- Legacy schema 1 profiles: `Data/Profiles/Minecraft-1..5` are migration-only, never active launch storage.
- Legacy flat App pointer: `App/data-location.json`, consulted only when
  the canonical pointer is absent and retained by packaging preservation.
- Preferences: `Data/Settings/`, with first-run 144/10 FPS limits.

Runtime and Data Storage open only verified schema 2. The existing explicit
`--lazy-tester-migrate-data` route copies verified schema 1 to an unused
`Migrated/Launchers` child of the selected catalog root, or reviewed replay to
an unused default Data root, then publishes the canonical pointer after all
five stores reopen. Original sources and incomplete destinations are preserved.
These are storage identities, not permission to run a migration. Read
[Data layout](../../../docs/system/data-layout.md) before any migration.

## Developer commands

Use only the pinned versions in `../../toolchain.json`.
With genuine dependencies and a compatible developer environment:

```bash
npm ci
npm run typecheck
npm test
npm run check:frontend    # typecheck + Vitest only; no Cargo
npm run check:rust:target # Windows-target Cargo check without all-targets
npm run build:frontend
npm run check:lazy-package
npm run verify:source
```

`npm run dev:app` uses the current Lazy Tester native configuration.
Full Cargo checks and tests for desktop and runtime-core use the
`x86_64-pc-windows-msvc` target, consistent with the scoped Windows developer
check and Tauri development layout. This reduces duplicate host-default
cache trees, but different feature sets and profiles can still have separate
compiled artifacts; no active `cargo clean` is implied.

Node/Vitest/Svelte verification does not prove Windows app integration or
Minecraft Education playability. Installed signing, Windows desktop
acceptance, and five-client gameplay remain separate evidence gates.
No CI runs are enabled; the only development branch is `Local`.

See the [implementation map](../../../docs/system/implementation-map.md),
[current handoff](../../../planning/development.md), root `AGENTS.md`
and `GITHUB_RULES.md`. Retired designs remain in Git history, not active UI.

Workspace placement and preview IPC share one captured geometry per layout pass.
Position, size, target identity and pixel ratio are rechecked after actual native awaits;
without a selected game there is no redundant pre-preview geometry read.
The bridge borrows the captured tiles. Native preview admission compares all
destinations in one fresh manager geometry sample; actual Park/Place and DWM
updates retain their fresh native checks. Ancestor scroll and event-driven
display-scale changes reuse the existing dirty scheduler. The native conversion
rounds shared physical edges consistently.

Play-wide preview feedback uses the existing fixed launcher action strip;
redundant below-rail guidance is removed. The existing Refresh action remains
available for explicit preview recovery.

Selection/preview action errors with a retained, available workspace reuse the
fixed launcher action strip. Connection and restoration uncertainty stay global;
selected close confirmation takes precedence over noncritical display feedback.

Healthy Play relies on existing automatic inspection; the header Refresh is
shown only for actionable exceptions or an in-flight manual refresh. Settings
and Data Storage retain explicit inspection controls.

Data Storage shows Project Data and the connected launcher data location together,
with the existing native Open action and each selected launcher's verified folder.
It does not move data or treat a path as proof of account readiness.
Data Storage separates global preferences from per-launcher management. The
verified temporary web-cache preference is persisted (default enabled); eligible
post-exit maintenance uses the existing controller, without an extra worker.
Selective Cleanup reviews worlds, resource/behavior packs and download cache.
World/pack disposition and download/log cleanup remain held; unknown cache
location is not zero usage. The seven-day log policy is source-only. Pending
full-clear recovery revokes both cache admissions for that launcher. Existing
full-clear confirmation, backup and data-preservation boundaries remain.

### Native payload review gate

The selected-only HUD source uses presentation ABI v4. The previously reviewed
DLL pair is not approved for this ABI. Source packaging and new native launches
remain blocked until a matching DLL pair is built and explicitly reviewed on the
approved native validation surface. Existing installed files and profile data are
not changed by this source gate.
