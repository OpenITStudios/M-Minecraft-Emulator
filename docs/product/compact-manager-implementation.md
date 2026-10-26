---
id: document.product.compact-manager-implementation
class: DOCUMENT
domain: product
role: SPECIFICATION
authority: CANONICAL
lifecycle: ACTIVE
---

# Minecraft Instance Manager — Compact Manager Implementation Specification

**Decision state:** UX DESIGN FROZEN for the current two-client surface, 2026-10-13. **Implementation state:** FRONTEND UI IMPLEMENTATION IN PROGRESS; source/Windows results remain unverified until run at the new exact HEAD. This document describes target behavior, not current runtime proof.
**Source baseline:** `Local` at `0dc41fd19a93581b59143045b45a14d1dffc52c1` (reviewed source and contracts).
**Scope:** Windows desktop, Minecraft Education. Long-term goal: four separately licensed, independent sessions. Active code: native Client 01 + one Sandboxie Plus isolated Client 02 experiment. Client 03/04, distinct game sessions and packaged native release remain UNPROVEN/UNIMPLEMENTED.
**Authority boundary:** Rust/native platform and the typed IPC decide eligibility. The frontend explains observations and requests supported actions; this spec does not authorize new providers, account actions, implicit polling, destructive changes, packaging, or resumed Windows localtests.

**Design authority:** This is the **only active Compact Manager UI specification**. The older `compact-manager-ux-spec.md` and `compact-manager-screen-design.md` are historical draft records and MUST NOT be treated as alternative instructions. System/runtime contracts override a design proposal where safety or compatibility conflicts.

## 1. User goal and firm non-goals

The application is an everyday, lightweight **Minecraft Instance Manager**, not a testing console. A user should open it, identify a Minecraft instance, request it to open, switch to a validated window if supported, and correct an issue without reading engineering terminology.

In the daily UI, DO NOT provide: separate Dashboard/Overview, Local Test, Window Layout page, Sidebar, VM/guest controls, 4-client dummy list with operative controls, profile/account login form, CPU/GPU dashboard, Stop/Restart/Clone/Delete, Start All, automatic credential handling, persistent background polling, tray service or decorative fake controls.

**Product clarity:** A compact interface is not permission to conceal essential blocking instructions. Explain a real user-owned Sandboxie requirement contextually. Leave low-level process IDs, file signer evidence, credential-hook flags and command output behind Advanced details.

## 2. Final navigation and composition

Preserve the **native Windows title bar** and its minimize/maximize/close controls. Do not duplicate the product title in the content.

- **Instances** — default/main screen, always shown immediately on app open.
- **Settings** — secondary screen opened by an explicitly labeled button in the Instances toolbar, with `Back to Instances`. No sidebar or nested navigation.
- **Help** — contextual link/secondary Settings entry, not main navigation.
- **Move window** — in the relevant instance's inline expanded details; not a standalone page or global Apply layout for this pilot.
- **Setup** and **Issue details** — inline expansion beneath the affected row; no blocking full-screen wizard.
- **Advanced details** — collapsed section of Settings/contextual issue. Never expose raw sensitive data.

Main order: [native title bar] -> one toolbar [Instances | Check status | Settings] -> flat list [Minecraft 01, Minecraft 02] -> optional "Last checked" qualifier. No hero paragraph, nested cards, "X running" counter, future-03/04 marketing card, duplicative refresh buttons or permanent "GAMEPLAY NOT VERIFIED" banner.

The page may show exactly two supported instance entries now. Future 03/04 require an implemented provider/capability source before any operational rows or buttons appear.

## 3. Visual contract

**References:** M-TranslateIT dark neutral surfaces and familiar desktop type hierarchy; M-LazyBuilder contextual error recovery; Microsoft Windows conventional window controls. Reuse visual principles, not their unrelated workflows.

| Semantic token | Initial value |
| --- | --- |
| background | #101316 |
| surface | #171b1f |
| raised surface | #1d2227 |
| border | #2b3238 |
| primary text | #f5f7fa |
| muted text | #a8b0b7 |
| primary button | #eef2f6 |
| primary ink | #11151b |
| positive, warning, error | restrained semantic accents WITH text |

Typography: Segoe UI/system-ui, regular body 14 px, instance title ~14 px semibold, page title ~20 px, secondary labels 12–13 px, button text 13–14 px. Use a 4-px spacing rhythm, controls approximately 40 px high, row target ~84–92 px, subtle 8–10 px corners, thin row separators, and a restrained icon set. These are targets to verify in Windows, not existing app facts.

No sidebar, glassmorphism, large gradients, gaming ornaments, KPI cards, bright repetitive badges, 9-px critical text, or status presented by color alone. Button/icon affordances must be labeled except universally recognized native controls with accessible names.

The current Tauri settings are **1180×760 default and 920×620 minimum**. Design must pass both sizes and Windows 100/125/150% scaling without clipping; **960×640** is a *proposed future default*, not approved as an immediate Tauri config change. Center the content with a readable max width rather than making rows span a maximized screen.

## 4. English-only naming and copy dictionary

The **entire product UI is English-only**, including setup, status, errors, dialogs, help, accessibility labels and notifications. Developer discussion may be in other languages. No Indonesian text, runtime Language switch, translation catalog complexity or English/Indonesian mixtures in user-visible controls.

| Concept | Canonical UI wording | Avoid |
| --- | --- | --- |
| Window title | Minecraft Instance Manager | M-Bedrock Virtual Clients, Emulator |
| Main page | Instances | Overview, Minecraft, Dashboard |
| Instance names | Minecraft 01, Minecraft 02 | Client-1, Virtual-01 |
| Check both | Check status | Refresh, Rescan, Check both clients |
| Request start | Open | Start, Launch game, Run |
| Find validated game window | Switch to | Focus, Go to client |
| Missing setup | Set up | Configure, Build VM |
| Move a supported window | Move window | Arrange HWND |
| Expand problem | View issue | Bug report, Debug |
| Repeat inspection safely | Check again | Retry launch |
| Technology details | Advanced details | PID inspector |
| Secondary page | Settings | Configuration Center |
| Return | Back to Instances | Home / Dashboard |

Use sentence case, action-led 1–3 word button labels, concise statuses and one actionable instruction per issue. Named external dependencies such as **Minecraft Education** and **Sandboxie Plus** may appear in instructions where users must act in those products.

Identity remains canonical `Minecraft 01` and `Minecraft 02` throughout copy. User-defined alias is FUTURE ONLY and must never imply account identity. In the main rows no forced "Standard"/"Isolated" badge; environment appears inside details only when relevant.

## 5. Instances flat list: anatomy and decisions

Every row contains (1) small neutral monitor glyph, (2) stable instance name, (3) **one truthful status line**, (4) **at most one meaningful primary action**. Optional inline expansion is triggered by a clear text control; no permanent overflow `···` when there are no useful supported secondary operations.

Do not make the whole row a clickable nested button around inner buttons. Opening details must preserve keyboard order and return focus on collapse. Only one instance's contextual details needs to be expanded at a time; collapsing must not mutate Minecraft.

- **Minecraft 01**: primary `Open` (official Windows registered Start app request) even if no prior combined inspection, unless that operation is already in progress/blocked by backend. Do NOT offer `Switch to` or `Move window`: the current backend has no safe native-01 focus or placement command. Outside-box window/process candidates are not verified Client 01 identity.
- **Minecraft 02**: if unchecked, show `Not checked` and direct user to one GLOBAL `Check status`. Do not duplicate that button in the row. If setup is known missing, `Set up` expands targeted guidance. If all necessary inspection and UI preconditions are met, `Open` requests isolated launch, with Rust revalidation. If a **unique eligible boxed-window candidate** is available, `Switch to` requests backend-revalidated focus; the frontend does not declare it an authenticated game. If an ambiguous/non-unique candidate exists, DO NOT display a duplicate Open, and offer `View issue` or guidance to global Check status.
- For Client 02 `Move window` is a secondary action ONLY when target eligibility permits. Its inline UI offers `Left` / `Right`; preview/selection is inert; user presses `Apply`, then Rust revalidates target and requests placement. Do not move Client 01 as a side effect. Unsupported fullscreen/maximized/minimized states must be explained, not silently changed.
- **A single global Check status** invokes the existing on-demand combined inspector. Even when one user-facing row is affected, never add a redundant row-level check command.

## 6. Presentation state and operation authority

There is **no new authoritative lifecycle database or "Running" state machine**. Compose an ephemeral view-model from:
- most recent inspection result and when obtained;
- explicit inspection request status (idle/pending/error);
- explicit operation request status (idle/pending/request sent/rejected/ambiguous);
- per-provider *supported* commands and backend eligibility evidence;
- temporary Client 02 official EXE selection and explicit Sandboxie licence/persistence confirmation.

Keep **last observation** and **action eligibility** distinct: a last-observed window may be displayed with `Last checked: …` but cannot become perpetual live authorization. When an operation invalidates evidence, retain old observations only as clearly historical text, and treat them as ineligible for dangerous actions. Do not represent old process data as fresh or replay it to authorize launch. Empty/unknown never means Stopped.

| Observed/operation condition | Public status | Primary behavior |
| --- | --- | --- |
| App opened, no current-session observation | Not checked | Global Check status; Client 01 may still Open |
| Inspector pending | Checking… | Disable conflicting operations while maintaining readable UI |
| Settings demonstrably incomplete | Setup required | Set up on affected instance |
| Inspected box eligible AND EXE/license provided | Ready to open | Open on Client 02 (Rust always rechecks) |
| Native/isolated launch call is pending | Opening… | Suppress duplicate requests; never infinite spinner after call resolves |
| Launch request returned success | Open requested | No fabricated Running, playable, login or window claim |
| Unique Client 02 boxed window candidate from last check | Window detected | Switch to only with revalidation at action time |
| Boxed process exists but no unique validated window target | Needs attention | View issue / Check status; NO second Open |
| Inspection impossible or process membership UNKNOWN | Couldn't check status | Global Check status and contextual cause, NO unsafe Open |
| Focus target replaced/disappeared | Window unavailable | Clear eligibility; Check status |
| Launch side effect may have occurred but command timed out | Couldn't confirm if Minecraft opened | Check status; never auto-retry |

Only Client 02 can obtain boxed-process attribution. An outside-box PID is a **candidate** and MUST NOT become a verified Client 01 window or signed-in user. Process/window detection is not Minecraft authentication or playability proof.

### Exact current admission boundary
The active typed bridge `Development/apps/desktop/src/app/bridge/nativeFeasibilityApi.ts` provides:
- `openOfficialMinecraft()`
- `selectEducationExe()`
- `requestSandboxClient02(executable, licensed)`
- `focusObservedClient02(target)`
- `placeObservedClient02(target, LEFT|RIGHT)`
- `inspectIsolatedClient02()` and an additional read-only native candidate inspection.

The existing `isSandboxieBoxPrepared()` / `canRequestClient02()` checks include an interactive Windows session, configured and enumerated persistent Sandboxie box, AutoDelete/AutoRemove/credentialHostAccess OFF, observed membership and zero boxed process candidates. For Client 02, the UI ALSO needs selected official Education EXE and explicit licence/persistence confirmation. **Rust is final authority and refreshes evidence on launch**; UI eligibility is advisory.

Focus/placement requires a unique last-observed boxed candidate with PID, window handle and process creation time. Rust verifies current Sandboxie membership, Microsoft-signed product image, HWND ownership and liveness before focus/movement. The frontend may offer `Switch to` as a request, never assert guaranteed foreground activation.

## 7. Launch outcome and error copy contract

**Critical:** Current Tauri launch/focus/placement commands return `Result<(), String>`, NOT a typed enum differentiating blocked-before-side-effect from uncertain-after-side-effect. Do not parse arbitrary Rust/PowerShell message substrings as if they were a reliable machine contract.

Present an operation outcome conservatively:
- pending: `Opening…`;
- call returned success: `Open requested` + optional "Check the Minecraft window.";
- explicit known safe setup blocker: contextual `Setup required` + user action;
- failure with uncertain side effect: `Couldn't confirm if Minecraft opened` + "Check status before trying again.";
- Focus revalidation refusal: `Window unavailable` + "Check status and try switching again.";
- OS foreground refusal: `Couldn't switch to Minecraft` + "Select its window or use Alt+Tab.";
- rejected/invalid executable: `Minecraft installation needs attention` + `Select Minecraft` or contextual guide.

Do **not** invent a REJECTED vs OUTCOME_UNKNOWN classifier in TS by regex on free-form backend error strings. Until Rust exposes explicit structured outcomes, ambiguous cases must be conservatively surfaced; a future typed error/result contract is a **separate backend enhancement**. A command timeout or launcher helper exit does not prove Minecraft failed to appear. Do not use a blind "Try again" launch button for ambiguous results.

Check status failures use a single actionable inline issue near Client 02 and the existing global check control; do not duplicate multiple buttons. Advanced details can show bounded, redacted technical description, not account secrets/world data.

Feedback is quiet: action spinner only while IPC request pending, brief "request sent" acknowledgement rather than large success popup, errors persist until action/dismissal, no fabricated percentages or auto-disappearing critical instructions.

## 8. Setup rules

Setup belongs to the affected instance, not a global blocking wizard. Minecraft 01 remains independently usable if Client 02 is not configured.

A fresh user may require: choose official Minecraft Education Desktop executable (for the isolated path), prepare properly licensed Sandboxie Plus persistent box `MinecraftClient02`, review explicit required settings, then run Check status. Show only the **current actionable unmet condition**, not a permanent 1/3 progress wizard.

No automatic sandbox creation, AutoDelete/AutoRemove changes, account duplication or privileged configuration. Naming Sandboxie Plus is acceptable when the user needs to open that application. Keep credential-hook technical flags out of ordinary copy; do not suggest they prove account separation. User must sign into each officially licensed Minecraft Education session inside Minecraft.

EXE selection and licence checkbox are CURRENTLY session-local; do not imply preferences survive reopen. Future non-secret path/alias/layout persistence requires a separate reviewed data owner and must never cache account credentials or waive Rust admission checks.

### Source failure-path review — Focus and Move (2026-10-14)

Focus and Move already invalidate their Client 02 observation when the
backend rejects the operation, preserve a scoped error for **View issue**,
mark the last check historical, and unblock the single-operation UI guard.
No new failure-state manager or automatic retry is required. A targeted
source regression covers these invariants for both IPC paths.

The Move error copy previously presumed every rejection meant the Minecraft
window needed restoration. Rust can instead reject stale PID/creation time/
HWND evidence, changed Sandboxie membership, unsigned executable attribution,
or Win32 refusal. The guidance now asks for an explicit **Check status**
first, and suggests restoring a normal window only *if* the game is currently
minimized/maximized. This does not classify free-form backend messages and
does not imply that Windows actually moved the window. Local Windows,
Svelte/Vitest and Minecraft acceptance remain NOT_EXECUTED.

## 9. Window management rules

Current capability: **Client 02 only**: `Move window` -> choose `Left`/`Right` -> `Apply`. This adjusts the selected isolated window within its current monitor work area; it does not arrange Client 01.

- No fullscreen, maximized or minimized coercion. If window isn't normal, say "Restore the Minecraft window first."
- No automatic movement when changing preview, opening manager, reconnecting monitors or finding a candidate.
- Monitor or HWND can change between inspection and action; backend revalidates. Failure returns contextual guidance.
- The current backend uses asynchronous SetWindowPos: a successful return means **move request accepted**, not final position/render confirmed. Say `Move requested` instead of `Window moved successfully`.
- Multi-monitor selection, saved layouts, 2×2 grid and batch Arrange windows remain **future** pending provider ownership and geometry verification.

## 10. Application lifecycle and progress

- **Open manager:** show Instances immediately. The current inspector is user-triggered and bounded; future one-shot startup inspection is a separate performance/security review, not part of this UI refactor. Never gate first paint behind inspection or introduce persistent polling.
- **One operation at a time:** preserve the existing frontend busy guard and backend Client 02 launch serialization. Disable conflicting commands but keep existing information, Settings and Help readable.
- **Minimize:** use native Windows behavior; never stop/reposition Minecraft.
- **Close (legacy two-client surface):** no Minecraft Stop function. Current Lazy Tester behavior is superseded by the27October2026 full-exit decision in [the workspace contract](unified-workspace.md): normal closure of owned sessions before manager exit, with pending saves/dialogs explained and no forced termination.
- **Second manager launch:** currently prevented by exclusive desktop instance lock; it may show a dialog. Bringing existing window to front is a FUTURE product enhancement, not an implemented feature.
- **Reopen or Windows restart:** no stale process result as live status. Nonsecret preferences may be restored only after a reviewed persistence contract exists.
- **No system tray, hidden resident launcher or automatic multi-client startup** in first version.

## 11. Settings and accessibility

Settings is a compact secondary page with only implemented items: Minecraft setup access, contextual window placement access where available, Help, app/technical information. Do not show active Language, Theme, RAM/GPU, Startup, Backup, auto-start, Stop or account settings without actual support.

Keyboard: logical Tab/Shift+Tab, Enter/Space to activate, Escape closes inline menus/panels where relevant, focus returns to expander, and native Alt+F4 works normally. Inline details use correct expanded semantics, not an unnecessary modal focus trap. Screen reader must announce operation status/failure appropriately, not repeatedly. All primary controls have text; hover is never the only explanation. Minimum 4.5:1 contrast for normal text. Test at 100%, 125%, 150% scaling and actual 920×620 min viewport.

## 12. Architecture & migration mapping

Only ONE mounted Svelte application shell. Avoid Redux/event bus, duplicate stores, a second Rust service, persistent polling and copy-pasted provider rules.

| Owner | Decision |
| --- | --- |
| `Development/apps/desktop/src/main.ts` | Eventually mount **one** Compact Manager App Shell |
| `Development/apps/desktop/src/NativeFeasibilityApp.svelte` | Refactor/retire from entry only after replacement and tests; no parallel live apps |
| `Development/apps/desktop/src/App.svelte` legacy | Leave unmounted, do not re-enable |
| `Development/apps/desktop/src/app.css` | Replace pilot-specific styling with a single semantic token layer |
| presentation helpers | Pure state/status/action projections, unit-testable |
| Svelte components | toolbar, flat instance list, row, inline issue/setup, window placement, Settings |
| `nativeFeasibilityApi.ts` | Preserve strict typed IPC parser and existing admission projections |
| Rust native client engine + collector | Remain sole process/isolation authority |
| Legacy VMware + existing guest/account data | Untouched, inactive |
| Tauri/NSIS package identity | Preserve legacy compatibility; native packaging remains BLOCKED |

Model modules by responsibility, not \`v2\`/`-new` duplicates. Keep environment-specific code behind IPC, not in Svelte. Names of modules are an implementation suggestion, not a demand to rename old compatibility paths.

## 13. Acceptance checklist / honest proof

Design review must be followed by actual executable tests on a pinned source SHA; **this document does not assert PASS**.

**Static and component checks:**
1. User sees Instances immediately with no sidebar and **no Indonesian UI strings**, duplicate navigation, false 03/04 controls or pilot banner.
2. Both rows remain readable at 920×620 and Windows 100/125/150% scale; no clipped actions or horizontal overflow.
3. Single global Check status; correct UNKNOWN, no-candidate, candidate, stale, pending, setup, error and ambiguous-launch projections.
4. Client 01 Open remains available independently of Client 02 setup, while no native Focus/Move is shown.
5. Client 02 Open requires every UI prerequisite and safe Rust recheck; existing boxed candidate blocks duplicate launch; Focus/Move require unique eligible candidate and fresh Rust identity verification.
6. Concurrent double clicks and stale async responses cannot replace newer truth, enable duplicate actions or mislabel an in-flight request.
7. Inline setup/issue expansion works by mouse and keyboard, with focus and screen reader semantics; advanced information remains discoverable but collapsed.
8. Error copy is concise and contextual; no auto-retry on uncertain outcomes, false `Running` or `Window moved successfully` claims.
9. No unsupported controls, persistence claims, multi-window layout presets or unreviewed settings.
10. Close and reopening logic does not itself send Stop/reset, erase sandbox/world data or re-enable VMware commands.

**Windows acceptance later, only on explicit user instruction:** exact-SHA dev startup and smoke, real official installed Education app behavior, Sandboxie preparation/membership checks, two independent licensed sessions (manual login), window focus/left-right actions, Fullscreen/maximize/minimize, DPI/monitor changes, crash/timeout ambiguity, manager close/reopen. Source build and browser prototype never prove gameplay or package compatibility.

### Source-only executable selection pending state — 2026-10-14

Client 02's presentation previously displayed "Ready to open" while the
executable-picker IPC request was in flight. Its Open button was disabled by
the existing global busy guard, so this was not an authorization bypass, but
the status was misleading while the selected file and licence acknowledgement
could change. During `busy === "select"`, only Client 02 now displays
"Selecting…" with no primary action or Move eligibility; native Client 01
is not falsely marked as selecting. On successful choice of a **different**
file, the existing UI clears the licence checkbox and the resulting source
projection is Setup required until explicit reconfirmation. An unchanged path
does not silently alter the previous checkbox, but Rust still repeats its
signed official-installation, licencing and Sandboxie preflight before launch.
A failed/cancelled picker still preserves the appropriate previous issue.
These are presentation and authored source regressions, not proof of an actual
Minecraft sign-in, file integrity or executed Svelte build.

### Source-only executable-picker error resolution — 2026-10-14

An explicitly successful executable selection resolves only a previous
`UiIssue.origin = "executable-picker"` UI error. The error tag is set by
the file-picker catch handler, not inferred from free-form backend messages.
A cancelled/empty selection preserves previous errors, and any ambiguous
Open, inspection, Focus or Move error remains until its own relevant check.
Different executable selection continues to require licence confirmation;
a file choice does not establish Microsoft signing, successful launch or
playable gameplay. This is source-level presentation behavior with authored
regressions, not Windows/Vitest/Svelte execution evidence.

### Source-only inline Move navigation revision — 2026-10-14

The contextual **Move window** button inside Client 02 About details previously
set `detailMode = "move"` without advancing the existing UI navigation
revision. If a prior IPC operation was still pending, its late error or success
could treat that newly selected panel as the original one and auto-replace it.
This button now uses a guarded handler in the mounted Svelte shell which
increments the same ephemeral revision as other user-owned navigation.
Eligibility remains advisory; backend focus/placement still revalidates fresh
PID/FILETIME/HWND and Sandboxie membership. This changes neither backend
admission nor the current no-polling/no-credential policy. Runtime UI and
Minecraft outcomes remain NOT_EXECUTED.

### Source-only cross-client details on IPC completion — 2026-10-14

After feedback was scoped per instance, the shared inline panel could still
be dismissed by Client 02 inspection while Client 01's issue was expanded,
or replaced by an error from a different client. Now only the Client 02
issue panel may auto-close on Client 02 inspection success, and an issue
may auto-open only if no panel is expanded or the current panel belongs to
the same instance. The other client's error remains available via View issue.
No backend, account, or Minecraft game behavior changed. Regression tests
are authored, not a compiled UI acceptance claim.

### Source-only client-specific feedback — 2026-10-14

The active Compact Manager keeps one global UI operation guard, but now stores
temporary issue and notice feedback separately for Minecraft 01 and 02.
Native Open clears only native feedback; Check status/Open/Focus/Move for the
isolated client clear only isolated feedback. A Sandboxie-only check cannot
resolve an earlier native launch warning. A Client 02 check also no longer
labels Client 01 "Checking…" since it does not actually inspect native sign-in.
Per-client feedback remains UI-only, does not authorize actions, and does not
introduce another lifecycle state machine, polling, credential storage or
backend command. Source regression tests have been authored; full Svelte/Windows
runtime evidence is still NOT_EXECUTED.

### Source-only asynchronous outcome navigation safety — 2026-10-13

The active Compact Manager now increments one ephemeral UI navigation revision
when the operator opens/closes details or changes pages. Each IPC operation
records the revision on request. Late success or error responses still update
their appropriate operation result, but cannot automatically close a newer
Client 02 panel or replace newer contextual navigation with the issue panel.
The issue remains accessible through **View issue**, without parsing backend
error strings into authoritative status. A cancelled executable picker does not
clear a preexisting launch/inspection issue. This is presentation-only
concurrency safety: it does not cancel a game operation, retry, persist a
status machine, or prove a Minecraft window has opened. Source regressions
are authored; Svelte/Windows execution remains NOT_EXECUTED.

### Source-only Client 02 issue/detail routing — 2026-10-13

The Client 02 inline panel now renders the **explicitly selected detail mode**.
Previously a stored Client 02 issue forced the issue panel even when the user
requested Set up from Settings or tried to open About details. The row may
continue showing the unresolved issue, and View issue still explicitly selects
the issue panel, but a stale issue must not hijack contextual navigation.
The change does not clear errors, assert that launch failed, bypass process
admission, or enable Move window without a validated candidate. Source/test
changes are **not** evidence of a compiled Svelte build or actual Windows UI
interaction.

### Source-only Settings focus handling — 2026-10-13

The active Svelte shell now transfers keyboard focus to the mounted Settings
Back button when opening Settings, and to Check status when returning to
Instances. Settings' contextual setup/move actions focus the Client 02 inline
region (programmatic focus only, no modal trap). Closing an inline panel after
an asynchronous request never focuses an unmounted Instances control while
Settings remains open and does not override another expanded panel. This is
source behavior with authored assertions, **not** a Windows UI or screen-reader
acceptance result. Navigation focus callbacks also confirm that the destination
page and intended target are still mounted when deferred focus runs; they
do not resurrect obsolete details or steal focus after rapid navigation.

### Source accessibility hardening — 2026-10-13

The active Compact Manager now records the user-triggering button for inline details,
restores keyboard focus after the panel is removed (using a Check status fallback if
the previous button disappeared), and supports Escape to dismiss the visible details.
"View issue" always reveals the issue, even when the panel is already open; a
separate Close details action handles dismissal. Expanded buttons reference their
detail region with ARIA attributes. These are source changes and test assertions,
**not** Windows keyboard/screen-reader acceptance evidence.

### Implementation notes — 2026-10-13

The first frontend batch mounts `CompactManagerApp.svelte` from `src/main.ts`, and adds a pure projection module and tests. The legacy `NativeFeasibilityApp.svelte` and inactive VMware `App.svelte` remain in Git but must not be mounted. There is no new Tauri command, native process monitor, persistence owner or packaged release. The UI uses on-demand combined inspection and conservative launch outcomes. The new branch state requires exact-revision source verification and later explicit Windows usability approval; do not infer it from older SHA preflight.

## 14. Execution plan and ownership boundary

- **Batch 1: Foundation** — one active shell, title bar, toolbar, compact tokens, flat rows, Settings shell; frontend-only.
- **Batch 2: Truthful Instances wiring** — existing typed IPC, status/action projections, no duplicated authority, event serialization, error copy and tests.
- **Batch 3: Setup and placement** — contextual setup, Client 02 Left/Right/Apply, advanced help, focus/accessibility and edge-case tests.
- **Batch 4: Review and verification** — build/source checks, responsive/keyboard accessibility, scenario matrix; separate actual Windows localtest only with approval.

A batch can be grouped into a meaningful development commit, but never misstate a passing build/test on HEAD. Stop at failed safety checks. Do not change the packaging guard, native installed identity, Microsoft account handling or paused test state as part of cosmetic UI work.

**Design freeze means: do not redesign navigation/screens in implementation without a reviewed UX decision. It is NOT a runtime production approval.**

## 15. Explicitly deferred, not silently omitted

Future work, with separate backend decisions and proof: 03/04 providers, persistent instance aliases and safe non-secret preferences, auto one-shot startup inspection, native Client 01 window identification and Switch to, global multi-window layout, structured Rust result/error codes, focus existing manager on second launch, Close-during-in-flight behavior, native installer migration, and all independent Minecraft playability acceptance.

## 16. Existing source references

- `Development/apps/desktop/src/main.ts` and `NativeFeasibilityApp.svelte` — mounted pilot.
- `Development/apps/desktop/src/app/bridge/nativeFeasibilityApi.ts` — on-demand typed bridge.
- `Development/apps/desktop/src-tauri/src/engine/native_clients.rs` — Windows native/boxed launch, Focus and placement.
- `Development/apps/desktop/src-tauri/src/engine/app_instance.rs` — exclusive desktop lock, not auto-refocus.
- `Development/apps/desktop/src-tauri/tauri.conf.json` — real current window sizes and legacy package contract.
- `docs/system/legacy-retirement.md` — authoritative inactive VM/package boundary.
- `planning/development.md` — localtest pause/evidence handoff; documentation/design commits do not transfer execution PASS to new HEAD.
