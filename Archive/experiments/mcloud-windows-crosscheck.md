# MCloud and Windows Multi-Instance Cross-check

**Review date:** 2026-10-13  
**Baseline:** `Local` commit `37f4290d26a4d4ee1e36196a59b03d8ffbc9dd20`  
**Authority:** RESEARCH ONLY (`Archive/experiments/`), not an approved provider or new runtime policy.
**Product:** Minecraft Education **Windows Desktop**; four distinct licensed playable clients remain the outcome, but only Client 01 native + Client 02 in Sandboxie Plus are under active investigation.  
**Execution:** No Windows, Rust, Svelte, Node, installer or Minecraft test was run for this review.

## Finding: "M-cloud" is not a Windows Minecraft engine

- [Zebrunner MCloud](https://github.com/zebrunner/mcloud) is a Docker/OpenSTF/Appium-backed device farm for Android/iOS devices, not a Windows game multi-instance provider. Inspect the upstream `README.md` and `docker-compose.yml`: it defines STF app/auth/API/websocket/processors, RethinkDB and Selenium Grid services. **Borrow operator patterns only**: explicitly named clients, bounded observations, clear activity/status, diagnostic drilldown. Do **not** port services, remote ports, DB, container fleet or Appium into this single-PC app.
- [Zebrunner MCloud agent](https://github.com/zebrunner/mcloud-agent) manages phone/device connectivity, not Windows Education identities. **Defer** integration.
- [Zebrunner MCloud Redroid](https://github.com/zebrunner/mcloud-redroid) concerns Linux-hosted Android. Its repository is archived. [Redroid documentation](https://github.com/remote-android/redroid-doc) describes Android containers, ADB, and optional GPU rendering. **Reject** as the runtime for Windows Education.
- [DeviceFarmer STF](https://github.com/DeviceFarmer/stf) is useful for device inventory and explicit actions. **Adapt** display/diagnostics concepts only; no streaming or remote service.
- [MEmu Cloud](https://www.memucloud.com/) is a UI/operator reference for multi-instance experiences. Its Android compatibility and resource usage are **not evidence** for Minecraft Education Windows.

## Higher-authority Windows and application facts

| Source | What can safely be concluded | Existing source owner / decision |
| --- | --- | --- |
| [Minecraft Education Windows installation guide](https://edusupport.minecraft.net/hc/en-us/articles/13106858087956-Windows-Installation-Guide) | Desktop and Store editions are different installation paths and incompatible as co-installed alternatives. | `native_clients.rs`: **KEEP** exact Desktop choice, no Store workaround. |
| [Minecraft Education sign-in guide](https://edusupport.minecraft.net/hc/en-us/articles/360061382672-Sign-Into-Minecraft-Education) | Remember Me and Switch Accounts are account-management features, **not** simultaneous multi-login evidence. | **BLOCKED BY RUNTIME EVIDENCE**: WAM/account independence cannot be certified from source. |
| [Minecraft Education licensing/getting started](https://edusupport.minecraft.net/hc/en-us/articles/360047557271-Getting-started-with-Minecraft-Education) | Each distinct user needs an applicable license/entitlement. | **KEEP** explicit operator licensing assertion; do not store or copy credentials. |
| [Sandboxie isolation mechanism](https://sandboxie-plus.github.io/sandboxie-docs/Content/IsolationMechanism/) and [OpenCredentials](https://sandboxie-plus.github.io/sandboxie-docs/Content/OpenCredentials/) | File/registry/process isolation and credential-hook settings do not prove WAM/SSO independence. | `collect-sandboxie-feasibility.ps1`: **KEEP** UNKNOWN and bounded credential-option checks; do not auto-modify templates. |
| [Sandboxie DLL API](https://github.com/sandboxie-plus/sandboxie-docs/blob/main/docs/Content/SBIEDLLAPI.md) | `SbieApi_QueryProcess` attributes a sandboxed PID when it succeeds; failure is not a native-process attestation. | **KEEP** the short-lived, explicitly requested vendor DLL probe; never infer native from nonzero status. |
| [Sandboxie Start CLI](https://sandboxie-plus.github.io/sandboxie-docs/Content/StartCommandLine/) | `/listpids` is a box-specific PID query; GUI Start.exe has stdout/redirection caveats. | **KEEP** UNKNOWN when output cannot be observed; not an empty-box claim. |
| [Sandboxie known conflicts](https://sandboxie-plus.github.io/sandboxie-docs/Content/KnownConflicts/) and [AppContainer tokens](https://sandboxie-plus.github.io/sandboxie-docs/Content/AppContainerTokens/) | Microsoft Store/UWP compatibility is materially different from normal Desktop EXEs. | **REJECT** Store-edition fallback into Sandboxie. |
| [Application Compartment](https://sandboxie-plus.github.io/sandboxie-docs/PlusContent/compartment-mode/) | Relaxing security for compatibility weakens the isolation model. | **REJECT** automatic compartment/NoSecurityFiltering switch as a login fix. |
| [SetWindowPos](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos), [MonitorFromWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-monitorfromwindow), [MONITORINFO](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-monitorinfo) | Window placement can be requested asynchronously and should use a particular monitor's work area; success does not attest final game rendering. | `native_clients.rs`: **KEEP** fresh PID/HWND ownership guard, async placement and no automatic reflow. |
| [PowerToys FancyZones architecture](https://github.com/microsoft/PowerToys/blob/main/doc/devdocs/modules/fancyzones.md) and [user guide](https://learn.microsoft.com/en-us/windows/powertoys/fancyzones) | Multi-monitor layouts, DPI and discoverable presets are sound design references. | **DEFER** complex/custom layouts until more than one trusted window can be attributed; **ADAPT** clearer user guidance now. |

## Source-level disposition — one owner, no speculative engine

1. **Adapt now — operator guidance:** `Development/apps/desktop/src/NativeFeasibilityApp.svelte` can display actionable next steps directly from its already-validated observation: interactive session, persistent box, known credential-bypass settings and PID membership. Keep one **presentation-only derivation**; the existing Rust preflight is still the only launch authority. Move lower-level configuration facts to expandable diagnostics. Do not display any gameplay PASS.
2. **Keep — process integrity:** `native_clients.rs` already re-observes Sandboxie membership and checks PID creation time/HWND immediately before Focus/Place. `SbieApi_QueryProcess` remains manual-only, signed and isolated in a short-lived collector. **Do not** introduce a parallel inspector/daemon or copy the operator data into another state DB.
3. **Keep — explicit resource policy:** no idle polling, game streaming, screen capture, artificial FPS governor or auto relaunch. The Windows/Minecraft game owns resource costs; no empirical capacity numbers can be derived from architecture diagrams.
4. **Defer — smart multi-window layouts:** current placement can move only a freshly attributed Client 02. A two/four-window operation must not silently grab an unknown outside-box process or a separate edition merely to fill a grid. No stored geometry or auto-restoration without clear user intent and measurable hardware evidence.
5. **Unresolved, not a code-only bug fix:** whether the official Windows Education Desktop launcher opens a distinct instance in Sandboxie; Windows broker isolation; licensed simultaneous login; gameplay; network/multiplayer; graphics/input; long-duration CPU/RAM/GPU; signed release installer. These require an environment and are **NOT_EXECUTED**, not PASS or FAIL.

## Non-goals and evidence boundary

No automated account/token transfer, modified game binaries, bypassing single-instance restrictions, cross-user logon tricks, unlicensed Windows VMs or new Android provider. Do not run layered microtests to claim progress. Source-only edits must be reviewed for exact ownership and consumers, with no gameplay claim. Actual Windows acceptance is **postponed by the operator**.

The smallest coherent source change from this research is **a guided, honest setup UI**. Larger state-management rewrites have no demonstrated payoff until the active two-client environment can be observed.
