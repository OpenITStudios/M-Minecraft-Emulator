---
id: document.product.compact-manager-ux-spec
class: DOCUMENT
domain: product
role: SPECIFICATION
authority: WORKING
lifecycle: HISTORICAL
---


> **STATUS: SUPERSEDED DESIGN DRAFT.** Historical reference only; do not implement this document. The active product is [Lazy Tester five-profile workspace](unified-workspace.md), not Compact Manager. All dated Compact Manager specifications, including the former implementation draft, are historical and must not control current navigation, launch or recovery. Runtime safety contracts are unchanged.

# Minecraft Instance Manager — Compact Manager UX Specification

**State:** DESIGN DRAFT; not implemented and not a verification record.
**Date:** 2026-10-13
**Source examined:** `Local` at `580b4e386538aa4aa906bfe4c555e43a77135e1c`.
**Product:** a lightweight, everyday Windows desktop application for managing several legitimate Minecraft Education Desktop sessions. NOT a testing dashboard.
**Proof ceiling:** only a native Client 01 + isolated Sandboxie Client 02 experimental pilot exists; independent playability, separate sign-ins, client 03/04 and distribution remain unproven. Existing local test is PAUSED.

This is the proposed UX contract for review, not authority to change safety boundaries or unpause operations. The actual launch/inspection authorizations live in Rust and the typed IPC contract. Visual designs must never create an alternative backend state authority.

## 1. Product principles

1. **One-screen daily workflow.** Open the app to Instances and locate the primary action for every supported instance in one glance.
2. **Progressive disclosure.** PID, image signer, Sandboxie membership and credential-hook details belong in Advanced diagnostics, not the primary list.
3. **Truthful observation.** A launch request is not a launched window. A window/process candidate is not a verified native identity, account session, or playable world. A previous inspection is not live state.
4. **Safe uncertainty.** UNKNOWN never maps to Stopped, Empty, or launch permission. Ambiguous launch and unsupported requests do not trigger retries.
5. **Independent instances.** Missing isolated-client configuration must not prevent the separate native Open operation if that operation remains safe.
6. **Do not disturb gameplay.** Never terminate, reset, reprovision, move, or erase a game or sandbox without an explicit supported action; closing the manager must not be presented as stopping Minecraft.
7. **Native desktop quality.** Legible status, keyboard navigation, visible focus, disabled-action explanations, adaptive layouts and accessible semantics.
8. **Future scale through capabilities.** 03/04 appear as operable items only after corresponding provider contracts exist and are verified.

## 2. Information architecture

Primary:
- **Instances** — home, per-instance status and primary operation.
- **Window Layout** — preview and explicit placement when supported by backend.
- **Settings** — General, Instances, Advanced.

Secondary:
- **Help & Diagnostics** — opened contextually or from navigation footer.
- **Instance details** — an inline/details panel, not a permanent main destination.
- **Setup guidance** — appears only for missing prerequisites; no persistent Local Test page.

The preferred compact window is approximately 820–980 px wide. Proposed adaptive review targets: 980×680, 820×600, 680×540 and Windows 125%/150% scaling. These are design test cases, not verified application constraints.

## 3. Instances information hierarchy

One application header (product identity + Settings/Help access), one page heading (“Instances”), a short status line, one explicit “Check Status” action, then rows in stable order. No full-size dashboard KPI hero, no usage charts, no giant setup explanation.

Every instance row, from left to right:
- Fixed monochrome game/window glyph (not a simulated Minecraft frame).
- Clear name: `Minecraft 01`, `Minecraft 02`; secondary description: `This PC` / `Isolated`. The provider term is optional subordinate metadata.
- Status: concise text + optional icon; NEVER color alone.
- Exactly one primary action based on supported capability and last observation.
- Optional overflow `More actions` with only supported secondary commands.
- Selecting non-control area may open instance details; row as a whole must not compete with nested button keyboard semantics.

Rows: target 76–88 px; control height 36–40 px; main text 13–15 px; page heading 20–22 px; keyboard focus must remain visible. These are first-pass targets to confirm in a live Windows window.

No sort, search, folder, tag or batch toolbar for two to four instances in V1. Names and order are stable; no silent drag reordering.

### Empty/setup/failure variants
- **Never inspected:** `Not checked` and `Check Status`; never `Stopped`.
- **Check in progress:** `Checking Windows and instance configuration…`; disable conflicting actions; keep prior evidence explicitly marked previous/not current rather than reinterpreting it.
- **Inspector fails:** `Couldn't check instance status` + `Try Again` + expandable details; no launch assumption.
- **Needs manual setup:** instance-specific `Set Up` with next exact step. Keep unaffected instances usable.
- **Launch requested:** `Open request sent. Check the game window, then check status.` Do not show `Running` or infinite `Starting…`.
- **Potential process seen, identity unclear:** `Window status uncertain` + `Check Status`; no second launch or Focus.
- **Focus eligible candidate:** `Window found (last check)`; Focus action revalidates target at execution time; not an account/gameplay claim.
- **No observed boxed candidate:** `No game process found at last check`, not `Stopped` as a live assertion.

## 4. Active backend capabilities — exact design boundary

Current active UI owner: `Development/apps/desktop/src/NativeFeasibilityApp.svelte`.
Current typed IPC bridge: `Development/apps/desktop/src/app/bridge/nativeFeasibilityApi.ts`.
Current owner: `Development/apps/desktop/src-tauri/src/engine/native_clients.rs`.
Current inspector: canonical on-demand Sandboxie collector; no automatic background polling or persistent daemon.

| Capability | Client 01 native | Client 02 isolated |
| --- | --- | --- |
| Open / launch request | `desktop_open_native_minecraft` | `desktop_request_sandbox_client_02` |
| Choose official executable | Not exposed on native path | `desktop_select_education_exe` |
| Inspect status | Combined `desktop_inspect_sandbox_client_02` gives candidates, **not verified host-native** | Same call gives box, membership and boxed candidate evidence |
| Focus | **Not currently available as safely attributed Client 01 operation** | `desktop_focus_observed_client_02` after unique observed candidate, rechecked by backend |
| Placement | **Not currently supported as Client 01 action** | `desktop_place_observed_client_02` LEFT or RIGHT, target rechecked |
| Stop / restart / delete | Unsupported | Unsupported |
| Layout preset / Start All | Unsupported | Unsupported |

Client 01 uses the official registered Start-app launch route. Do not label the action `Focus` or promise a new process. Observed outside-box PID(s) are candidates only, may belong to a different box or edition and MUST NOT be promoted to verified native identity.

For Client 02, `canRequestClient02` requires an observed interactive Windows session; box configured YES; box settings source BOX_SECTION_ENUMERATED; AutoDelete, AutoRemove, credentialHostAccess all OFF; membershipStatus OBSERVED; isolatedCandidateProcesses 0. Additional active UI gates: selected official Education Desktop EXE and explicit Sandboxie licence/persistence confirmation. The backend re-inspects before launch and can refuse. A zero count without reliable membership is never eligible.

Focus and placement require exactly one boxed window candidate with a non-null process creation FILETIME and HWND. The backend separately authenticates current file/process/window identity. UI should distinguish “Focus requested” from guaranteed foreground activation.

## 5. Status model — presentation projection, not a second authority

Use three orthogonal axes:
- `observation`: NOT_CHECKED / CHECKING / OBSERVED / UNKNOWN / ERROR
- `configuration`: NOT_CHECKED / ACTION_NEEDED / INSPECTED_ELIGIBLE / UNKNOWN
- `operation`: IDLE / REQUESTING / REQUEST_SENT / FAILED

These are ephemeral presentation labels derived from backend results and current request state, not persisted authoritative lifecycle records. A transient operation does not assert durable runtime state.

Priority order in row projection: pending operation > explicit error > unknown/untrusted inspection > setup blocker > observed window candidate > observed no candidate > not checked. However individual primary action eligibility is computed by backend capability + UI guards, never by translated labels alone.

| Observed case | User-facing primary text | Action |
| --- | --- | --- |
| Client 01, any non-busy state | Open Minecraft | Request official Windows Start-app opening; error is explicit |
| Client 02, never inspected | Check Status | Run one combined inspection |
| Client 02, configuration missing | Set Up | Open guided setup for exact missing prerequisite |
| Client 02, membership UNKNOWN / contradictory | Check Status | Show why result is ambiguous; no Start |
| Client 02, valid inspected empty box but EXE/licence not provided | Complete Setup | Choose EXE / confirm required licensing |
| Client 02, valid empty box + EXE/licence confirmed | Start | Backend repeats preflight; invalidate old observation |
| Client 02, exactly one boxed window candidate eligible for recheck | Focus | Backend revalidates; do not equate response with playable sign-in |
| Client 02, boxed process(es) but no unique focus candidate | Check Status / View Issue | Never Start duplicate |
| Any pending request | Working… | Block duplicate actions |
| Failed request | View Issue / Try Again when safe | Recovery-specific, no blind relaunch |

A global serialized operation guard currently exists; maintain it for first UI refactor. The user must be told which operation is ongoing. Do not manufacture per-instance concurrent launch promises.

**No inaccurate counters:** do not display `2 running` based only on name-matching candidates. Instead: `Last inspected: …` (only if an actual check completed in this session), and `Windows found: …` qualified as candidates when evidence permits. Prefer no numeric hero summary until verified lifecycle facts are available.

## 6. Daily interaction journeys

### J1. Open native Minecraft
1. Open app -> Instances (no modal).
2. User selects `Open Minecraft` on Client 01.
3. UI shows `Requesting Minecraft to open…`; disables duplicate request while pending.
4. Request succeeds -> `Open request sent. Check the Minecraft window.` User may manually Check Status.
5. Request fails -> human-readable error + installation help, safe retry after understanding failure.
6. Never claim native window, game rendering or login verified from the launch request alone.

### J2. Start Client 02
1. User inspects session with `Check Status`.
2. If configuration invalid: `Set Up` -> specific manual Sandboxie prerequisite; verification step rechecks when user returns.
3. If box membership ambiguous: no Start; show `Couldn't confirm box process status` and safe Check Status.
4. If empty box conclusively observed: request official executable selection (current pilot session-local), explicit licence/persistence confirmation.
5. Enable Start only when all gates are met; backend revalidates.
6. On success: `Launch requested`, invalidate previous snapshot; instruct user to check actual window and inspect again. No automatic retry.
7. Account sign-in and game playability are always manual inside the Minecraft app.

### J3. Focus existing Client 02
1. Only show active Focus for unique eligible last-observed window candidate.
2. Press -> backend re-check -> OS focus request.
3. If window disappeared or ownership changed, clear stale candidate and show `Window changed. Check Status again.`
4. If Windows does not bring it foreground, provide `Use Alt+Tab` fallback; do not claim gameplay.

### J4. Arrange windows
- Window Layout shows only arrangements actually owned by the active provider.
- Today: Client 02 Left / Right only when Focus target eligible; Client 01 is not moved.
- Future: monitor-aware previews, explicit Apply, window identity recheck, partial-success reporting, correct DPI geometry and no forced auto-rearrangement.
- Do not show 2×2 four-window controls as operable now.

### J5. Open app while game windows already exist
- Landing page must not interpret missing session inspection as zero running games.
- `Not checked` until a user-triggered combined inspection (current ceiling).
- After an inspection, row data says what was observed and when; it does not poll continuously.
- No auto-start, auto-stop, implicit focus or surprise window arrangement.

### J6. Close manager
- No UI behavior may imply Minecraft will be stopped as a side effect.
- The manager should clearly differentiate `Close manager` from any future `Stop client`.
- Confirm real window close/background-process behavior on Windows before promising “games keep running” as a certified guarantee; no system-tray persistence invented.

## 7. Instance details drawer

A secondary panel (drawer on wide screens, full-content overlay on narrow screens):
- title / source type / last observation;
- one concise cause/action if setup blocked;
- supported secondary actions such as `Place Left`, `Place Right` for Client 02 only when backend allows;
- `View technical details` expands candidate evidence, without secrets;
- explicit close and Escape, returning keyboard focus to the triggering row;
- no editable Microsoft account credentials, no screenshot capture, no worlds inspection;
- stable labels, no unsupported Restart, Delete, Clone, VM controls.

Overflow menus must be keyboard navigable, anchored to the correct row, close on Escape/outside click and never contain destructive operations that the backend does not support.

## 8. Setup guidance as a nonblocking task

Do not create a permanent `Setup` destination. Show `Set Up` from an affected instance only.

Flow:
- Detect/choose installed official Education executable where applicable;
- Explain Sandboxie Plus box `MinecraftClient02`, persistence and required setting state;
- Inspect configuration on explicit user request;
- If ready, return to instance row and show allowed action;
- Keep native Client 01 operation independent from boxed Client 02 blockers;
- Never ask for Microsoft username/password/token or duplicate an account profile.

Current EXE selection and licence confirmation are transient UI state. Persisting any nonsecret choice is a separate reviewed feature; licence assertion cannot silently persist as a waiver of later backend checks.

## 9. Window Layout, Settings and Help (screen contracts)

**Window Layout**
- Purpose: manage screen positioning, not game session state.
- V1 only show Client 02 Left/Right available as the current actions with accurate limitation. Do not imply multi-window global presets or monitor APIs exist.
- Future preset proposal: 1 centered, 2 side-by-side, 3 main+2, 4 grid/columns/focus+3; conditional on real backend capability and actual HWND ownership proof.
- Preview must never mutate window state; Apply initiates explicit request with failure feedback.

**Settings**
- General: language/theme/close behavior (settings can be view-only or clearly planned until persistence exists).
- Instances: official executable and provider requirements; show sensitive path only when needed, avoid logging personal directories.
- Advanced: inspection fields, build/runtime diagnostics; error details redacted and copyable without secrets.
- Do not display CPU/RAM knobs or auto-launch switches unless implemented.

**Help & Diagnostics**
- Context-specific recovery first, expandable raw technical details second.
- Maintain `What happened / What it means / What to do next`.
- Never equate collector error with empty box; never expose credentials or private world data.

## 10. Interaction and accessibility rules

- Enter/Space activates the focused button, Escape closes a drawer/menu and returns focus, Tab order matches visual reading order.
- Focus remains visible for all interactive controls; disabled primary actions have an adjacent explanation available without hover.
- Status must include text/icon meaning, never color alone.
- Dynamic progress uses a polite accessible announcement; failed operations use an alert region. Do not announce repeated unchanged messages.
- A collapsed menu never hides essential recovery instruction.
- No scrolling the entire page to find a blocked launch reason; show it by the affected instance.
- Avoid auto-dismiss notifications while user is reading; dismissible persistent messages are acceptable.
- Respect high-contrast, 125%/150% scaling and reduced motion where supported.

Responsive rules:
- Wide >= 900 px: 160–184 px labeled left navigation, rows in main panel, optional right-side details.
- Medium 680–899 px: reduce secondary metadata and panel widths, keep labels visible and one action per row.
- Narrow < 680 px: navigation can collapse to labeled top bar/dropdown; details become a full-screen panel. Never reduce readable font or hit target simply to fit.
- The renderer must fit the actual Tauri minimum window dimensions; breakpoints above are proposed for UI review, not a verified Tauri configuration.

## 11. Design tokens — initial proposal

Visual baseline from M-TranslateIT: dark neutral `#101316`, sidebar `#0d1013`, raised `#1d2227`, border `#2b3238`, main text `#f5f7fa`, muted `#a8b0b7`.
Use semantic status colors sparingly and retain text/icon for meaning. Minimum normal text contrast target 4.5:1, keyboard accessibility and visible focus. Component token owners should be in a single desktop styling layer; do not duplicate conflicting UI kits.

### Copy vocabulary
- Preferred English UI labels for current implementation examples: `Instances`, `Window Layout`, `Settings`, `Check Status`, `Open Minecraft`, `Set Up`, `Start`, `Focus`, `View Issue`.
- Indonesian localization can be introduced through one translation catalog, not scattered conditional strings. Product language default and persistence remain a product decision.
- Use “instance” consistently in user-facing copy; `Client 01/02` remain stable backend compatibility identifiers.

## 12. Minimum UI acceptance cases (not yet executed)

1. **Initial open:** supported instances visible, no fake zero/running count, no splash wizard for already-supported native action.
2. **Check unknown:** undefined/failed/contradictory inspection blocks isolated Start and cannot show `Stopped`.
3. **Setup partial:** Client 02 missing Sandboxie does not hide Client 01 Open.
4. **Empty ready box:** Client 02 Start requires real freshness-preserving revalidation, EXE and explicit licence confirmation.
5. **Existing boxed candidate:** no duplicate Start and no Focus unless exactly one eligible window candidate.
6. **Stale focus:** process/window replacement causes safe rejection and observation invalidation.
7. **Launch request succeeds:** UI says request sent, not “Running” or “Game Ready”.
8. **One-operation serialization:** rapid double clicking cannot issue competing launch requests.
9. **Responsive:** core action + status visible at all agreed widths/scaling values; secondary details adapt.
10. **Accessibility:** keyboard navigation, focus return, accessible button names, screen reader status and contrast.
11. **No unsupported operations:** no active Stop, Restart, Clone, Delete, Start All, or four-window Apply.
12. **Legacy safety:** VMware and packaging legacy controls remain blocked, with no account/world/VM mutation.

Tests should include pure projection assertions + rendered component interaction tests; source checks do not establish real Windows behavior or game compatibility. Do not claim pass until each test is actually run at a pinned source SHA.

## 13. Implementation sequence after UX review

**Batch A — presentation-only:** one active shell, design tokens, navigation, Instances rows, static detail/Help surfaces. Preserve typed bridge and Rust. No new provider capabilities.

**Batch B — truthful runtime wiring:** connect existing combined inspection and explicit native/isolated commands; derive action projections in pure selectors; test all UNKNOWN, busy, stale, failure boundaries. Keep operations serialized and on-demand.

**Batch C — setup and lightweight layout:** contextual Client 02 setup and existing Left/Right actions; keyboard focus management and responsive/accessibility review.

**Batch D — real Windows usability:** verify startup, manual Check, official native Open, boxed Client 02 request, Focus/Left/Right, uncertainty failures and no unintended account/world mutations on Windows. Pause remains until user explicitly authorizes local tests.

**Future (separate decisions):** provider support for 03/04, comprehensive layout, stop/restart, automatic one-shot startup inspection, preference persistence, native packaging and full four-account acceptance. No premature UI toggles.

## 14. Open product decisions (must be explicit before they become behavior)

1. Product default language (English-first vs Indonesian-first); design remains localization-ready.
2. Startup inspection: current V1 stays explicit/on-demand; a one-shot startup check needs separate runtime review and user acceptance.
3. Close manager behavior: needs native Windows evidence before claims/setting switch.
4. Future naming and customization of `Minecraft 01–04`: do not add rename persistence until supported.
5. Provider transition and account isolation: only real observations can justify scaling from two to four clients.

## 15. Reference patterns

- Microsoft, “Design for productivity in WinUI LOB apps”: https://learn.microsoft.com/en-us/windows/apps/get-started/line-of-business/design-for-lob
- Microsoft, Windows app design guidelines: https://learn.microsoft.com/en-us/windows/apps/design/guidelines-overview
- BlueStacks, Multi-instance Manager: https://support.bluestacks.com/hc/en-us/articles/360052834092-How-to-create-and-manage-instances-using-the-Multi-instance-Manager-on-BlueStacks-5
- NN/g, Visibility of system status: https://www.nngroup.com/articles/visibility-system-status/
- Existing project contracts: `docs/product/four-concurrent-accounts.md`, `docs/system/legacy-retirement.md`, `docs/validation/native-pilot-localtest.md`, `planning/development.md`.

**Do not use this draft to assert readiness or override the pause on local runtime testing.**
