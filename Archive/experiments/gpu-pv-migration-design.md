# GPU-PV Migration Study — M-Minecraft-Emulator

Status: DESIGN DRAFT / DIRECTION APPROVED / NO RUNTIME MIGRATION
Decision date: 2026-10-11
Baseline repository: MIVUBI-STD/M-Minecraft-Emulator, GitHub `Local` @ `d568e48dbc9cc518eb29b5a536e12e431aaa0a7b`
Upstream examined: jamesstringer90/appsandbox, release `v0.1.9` (2026-09-22), `main` @ `6f3adb6aafd4fc819d7715bdfacf52ac87df26a6`
Proof ceiling: REMOTE_GITHUB and public vendor documentation. No Node, Rust, installer, GPU-PV, Minecraft sign-in, network, or 4-client execution.

This file is research/design only under `Archive/experiments/`, **not** a second implementation registry, committed runtime policy, test gate, compatibility guarantee or permission to delete old guest data. Existing `docs/system/implementation-map.md`, `AGENTS.md`, `GITHUB_RULES.md`, and actual source continue to describe the CURRENT VMware implementation until a coherent HCS migration is implemented and accepted. `planning/development.md` holds the single current handoff.

## User-selected product intent

- Keep **one Minecraft Education Native Windows client + three isolated Windows 11 x64 guests**, each with distinct, manually signed-in licensed Education accounts, on one personal Windows 11 computer. No cloud execution and no recurring **Development/engine/provider** fee.
- Target hardware: **Intel Core i7-14700F (20 cores / 28 threads, no iGPU), GeForce RTX 3070 (standard 8 GB GDDR6), 32 GB host RAM, NVMe SSD**. CPU scheduling is not the principal expected constraint; 8 GB shared GPU memory and 32 GB RAM are important resource risks. No automatic CPU affinity/power plan/GPU override based on conjecture.
- **Both GPU APIs required in the same x64 guest:** actual hardware-accelerated D3D11 device supporting feature level >=11_0 **AND** hardware OpenGL context >=4.4 with an identified RTX 3070 renderer. WARP/Basic Render Driver, Microsoft GDI Generic, software Mesa, host-only capability, VM setting, and version-string spoofing do not qualify.
- Minecraft Education must install/sign in, render an actual world, respond to keyboard/mouse and sustain interactive use. GPU API presence does not certify Education compatibility.
- App Sandbox (GPU-PV over Host Compute Service / Virtual Machine Platform) is the selected **new engineering direction**, not yet a production backend. The prior VMware architecture is **LEGACY / FROZEN FOR NEW FEATURES**, not deleted. Preserve Git history, account-bearing disks, snapshots, path contracts and old code until safe cutover. Do not create copied `legacy-v2` folders or two user-visible providers.
- No extra recurring App Sandbox/NVIDIA vGPU subscription. **Windows guest and Minecraft Education licensing is separate**; a Windows 11 host OEM/Pro license does not automatically license three Windows 11 guests. Verify already-held entitlement (e.g. qualifying Education/Enterprise/A3/A5 arrangements) before claiming zero additional license spending.

## Upstream technical facts; critical constraints

App Sandbox `v0.1.9` exposes a local **127.0.0.1** authenticated HTTP API from `appsandbox.exe --headless`. It discovers a per-run random token/port via `%ProgramData%\\AppSandbox\\host.json`, is admin-elevated, uses an interactive desktop session to open guest windows, and enforces a **single GUI-or-headless instance**. Guest display needs an online agent plus display driver readiness. A separate session-0 Windows service cannot display guest windows. App Sandbox owns VM mechanics; Rust may own **policy/admission, UI contracts and user-facing orchestration only**. Frontend `Svelte → Tauri → Rust` remains unchanged; **do not** connect the Svelte frontend directly to the local App Sandbox HTTP API or persist bearer tokens.

Important API/lifecycle semantics from `tools/headless-api/README.md`:
- `create` is async and automatically starts guest OS installation. Create guests sequentially to avoid staging races.
- `shutdown(name)` is **graceful ACPI shutdown**, `stop(name)` is **force-power-off**. Never map existing routine `Stop` to force power-off or invoke daemon exit with active VMs. `shutdown_daemon(force=false)` rejects live guests.
- `start` has snapshot/branch arguments; `snap_take` only works for STOPPED VMs. Snapshot/branch handling is **not equivalent** to old VMware `QA_READY` semantics. Do not claim reliable identity preservation, disk/account recovery, or safe reset without examining HCS VHDX branch behavior.
- There are multiple networking modes (`None/NAT/External/Internal`), display-open status and SSH capability. Minecraft peer-to-peer reachability from host to guest and guest-to-guest must be observed, not assumed from `networkMode=1`.
- Windows guest setup needs administrator privileges, enabled VirtualMachinePlatform and original image/driver prerequisites. The product must use the upstream dependency/snapshot/guest-agent mechanics rather than secretly running a second lifecycle manager.
- Upstream has a custom Mesa D3D12 Gallium route that may modify system `opengl32.dll` in guest Windows and can report OpenGL 4.6 **even via WARP software**. `v0.1.6` added **native NVIDIA OpenGL** paths. Prefer correct NVIDIA GPU hardware context; do not inject unreviewed DLLs into production guests or mistake "GL 4.6" for hardware acceleration. Follow upstream redistribution/signing/recovery obligations and third-party licenses.
- Upstream currently creates optional Windows autologin/SSH setup. Do not reuse a guest template after Education sign-in, embed Windows administrator credentials in Git/source, log API bearer credentials, use auto Microsoft sign-in, or expose guest account-bearing drives. App Sandbox admin process is a privileged security boundary.

## Resource baseline — hypotheses, not measured performance

| Budget | Initial design (reversible) | Do not assume |
| --- | --- | --- |
| Host CPU | i7-14700F 20C/28T; leave Windows scheduler in control | 28 dedicated vCPUs or automatic P/E affinity |
| Guests | 3 x **2 vCPU** to begin, staggered startup | that all three need 4+ vCPU |
| RAM | 3 x **4096 MB guest** = 12 GiB allocated; keep memory for host + Native, diagnostics and HCS overhead | this is guaranteed adequate for real Windows 11 + Minecraft Education |
| GPU | 1 shared RTX 3070 8 GB; hardware GPU-PV with actual NVIDIA renderer | 8 GB per guest or three separate GPUs |
| Display | Begin at approximately 1280x720/window, conservative game render/simulation distance, capped graphics features where supported | forced FPS or resolution knobs if upstream does not expose them |
| Storage | NVMe and dynamic guest VHDX; require free-space preflight, real allocation and snapshot-growth observation | allocated size equals actual on-disk usage |
| Networking | NAT first for internet; verify Native↔guest and guest↔guest join before preserving default | Internet connectivity means multiplayer success |

Host memory pressure and GPU VRAM contention are expected bottlenecks; no fixed FPS, minimum frame time, VRAM partition quota, or "eco mode" without actual device data. Avoid extra telemetry daemons, continuous GPU polling, forced power off, VM hibernation hacks, Hyper-V GPU-P undocumented tuning and security/driver workarounds.

## Target ownership boundaries

```text
Svelte product UI / Window Layout (Development/apps/desktop)
  -> typed Tauri IPC, current Rust public contract
  -> Rust runtime-core: one policy, one Doctor, one admission,
     one operation lock, one truthful evidence/readiness projection
  -> one provider implementation selected for production: App Sandbox HCS adapter
     -> App Sandbox headless API (single upstream VM authority)
       -> GPU-PV, VHDX, built-in guest tools/display and driver setup
  -> observed GPU/guest/game facts -> Rust projection -> UI

Native Minecraft Education stays in host Windows.
Three isolated Windows 11 x64 guests retain unique identity/account state.
```

No shadow lifecycle database. Provider status must be **observed from upstream** after command receipt, not inferred from `HTTP 202 Accepted`. App Sandbox `online` means its guest agent connected, not Minecraft Education signed in or game playable. Do not persist `online == QA_READY`; do not redefine existing public enum names without intentional versioned migration.

The existing `Provider` trait and Rust runtime are VMware-semantic today (`vmx`, `vmrun`, `guest_token`, `BaseState`, `QA_READY`, VMware window discovery and path conventions). **They cannot simply receive an App Sandbox class with no contract redesign.** Trace the actual use sites and replace only the first wrong owners in a coherent batch; prevent a permanent layer of fake VMX compatibility or duplicated setup/agent managers. Keep guest account disks and historical VMware implementation inspectable through Git history; do not rename/delete installed data and guest image resources as a folder tidy-up.

## M0 source mapping — Provider trait to App Sandbox v0.1.9 headless API

**Audit basis:** current `Development/engine/runtime-core/src/provider/mod.rs`, `runtime.rs`, `lifecycle_admission.rs`, `doctor.rs`, `profile.rs`, `guest.rs`, `minecraft_runtime.rs`, `diagnostics.rs`; upstream `tools/headless-api/{README.md,asb.py}`, `src/backend_win/hcs_vm.c`, `src/backend_win/snapshot.c`. Source inspection only: no test execution or actual App Sandbox session. Mapping is a research tool, **not** a second provider interface.

### Provider trait method mapping

| Current Rust method | App Sandbox v0.1.9 API / facts | Treatment | Safety / first consumer |
| --- | --- | --- | --- |
| `id(), version(), detect()` | `GET /version` reports `product, version, apiVersion, hostOs, capabilities`; discovery at local `host.json` | ADAPT | Fail closed on absent/incompatible upstream; `current_platform_provider`, Doctor |
| `provision(client)` | `create` with `osType=Windows`, ISO/template, `ramMb/cpuCores/gpuMode/networkMode`; **async + auto-start** | DESIGN CHANGE | Existing `provision_inner` assumes stopped finalized VMware Base and synchronous clone; do not call until new image/identity lifecycle is explicit |
| `reprovision(client)` | `delete_vm` is destructive; upstream templates/snapshot branches differ from VMware cutover | BLOCK | Never delete account-bearing VM on path to recreate; old guard remains |
| `memory_limit_mb(client)` | `status(name).ramMb` | ADAPT | Configured guest limit, not observed working set |
| `host_working_sets_mb()` | `GET /host` has host RAM, free space, declared VM RAM and core sums, not VMware vmware-vmx process memory | NO EQUIVALENT | Do not present configured RAM as observed RSS; change resource evidence owner explicitly |
| `guest_tools_ready(client)` | `agentOnline`, display readiness has separate `idd_ready` | ADAPT | Upstream guest agent readiness is not Education or custom Minecraft launch readiness |
| `guest_ip_address(client)` | `ssh_info` provides loopback-forwarded SSH; `status` has `sshState` | NO EQUIVALENT | Existing `guest.rs` and `minecraft_runtime.rs` expect guest IP + authenticated TCP protocol. Do not silently substitute `127.0.0.1` or break isolation |
| `identity_key(client)` | `status.name` and upstream preserved MAC in v0.1.8; separate Windows machine identity proof required | REDESIGN PROVENANCE | A friendly name alone is not UUID/MAC/Windows identity or immutable disk provenance |
| `network_mode(client)` | `status.networkMode`: 0 none, 1 NAT, 2 external, 3 internal | ADAPT | Do not infer multiplayer reachable from network mode |
| `graphics_3d_enabled(client)` | `status.gpuMode`: 0 none, 1 default paravirtual, 2 try all | ADAPT AS CONFIG ONLY | GPU-PV config does NOT prove NVIDIA acceleration, Direct3D11 FL11_0 or hardware GL>=4.4 |
| `status(client)` | `GET /vms/{name}`: building/stopped/installing/booting/online/stopping plus flags | REDESIGN PROJECTION | Existing `ClientState` misses intermediary states; do not map building to RUNNING or `online` to Minecraft-ready |
| `start(client)` | `POST /vms/{name}/start`; HTTP 202 Accepted, then status/events | ADAPT ASYNC | Reobserve after acceptance; never treat 202 as successful completed startup |
| `suspend(client)` | No suspend operation in examined headless SDK/API | BLOCK/REMOVE IN MIGRATION | Don't simulate using snapshot, force stop or guest power-off |
| `stop(client)` | `shutdown(name)` graceful; `stop(name)` **hard power-off** | MAP TO SHUTDOWN ONLY | No destructive stop on normal UI action; preserve guest account disk |
| `restart(client)` | No direct headless `restart`; could request graceful shutdown then verified start | DEFER | Needs observed shutdown completion, idempotence and no automatic retries on unknown state |
| `set_ready(client)` | `snap_take` for stopped VM; branches use VHDX differencing | REDESIGN | Must not assume VMware `QA_READY` semantics. Snapshot with account state has privacy and guest-version implications |
| `has_ready(client)` | `GET /vms/{name}/snapshots` | DEFER | Named snapshot existence != verified account recovery |
| `reset(client)` | `start` can select a snapshot or branch | BLOCK | Could discard or redirect account-bearing disk lineage; design explicit safe recovery semantics first |
| `open(client)` | `POST /vms/{name}/display` | ADAPT | Interactive admin daemon session only; check `agentOnline` and `idd_ready`; window identity differs from VMware |
| `open_vm_ui(vmx)` | No VMX path; `open_display(name)` | REPLACE CALLERS | Base finalization and window arrangement directly depend on VMware VMX/window paths |
| `is_running_path(vmx)` | `status(name)`, upstream uses VM names | REPLACE CALLERS | Remove reliance on a path as identity; preserve old disks in legacy path |
| `start_validation_vm(vmx)` | `start(name)` async | REWRITE BASE FLOW | Current Base may be finalized/stopped and may not be booted; lifecycle must be re-proven for HCS templates |
| `stop_validation_vm(vmx)` | `shutdown(name)` | ADAPT GRACEFULLY | Wait for confirmed stopped state; no forced stop on timeout |
| `guest_ip_for_path(vmx)` | Not exposed as equivalent guest-IP API | BLOCK DIRECT PORT | Base registration currently calls Guest Agent via IP/token; redesign channel before admission |

**Additional upstream capabilities deliberately not promoted yet:** `edit` (guest must be stopped), `create` template, SSH key deployment, GPU selection, snapshot branch delete, force stop, `shutdown_daemon(force)`, and `events` stream. No speculative public VM feature panel.

### Cross-owner dependency cuts to make before executable provider work

1. **Backend provider selection and action admission:** `provider/mod.rs::current_platform_provider` currently selects VMware on Windows; `LifecycleFacts` and `runtime.rs::collect_lifecycle_facts` assume `current_base_vmx_path`, VMX finalized state and `QA_READY` on almost every operation. A new adapter class alone is insufficient. Preserve one admission policy while converting VMware-specific prerequisites to explicit provider capability and observed facts **only when actually needed**; no second manager.
2. **Image lifecycle and persistence:** `runtime.rs::register_base_inner/open_base_for_finalization_inner/provision_inner`, `doctor.rs::select_setup_action`, `profile.rs`, `base_preparation.rs` and `Development/engine/guest/windows` are VMware Base-specific. App Sandbox can create a Windows guest from an ISO or template, but its `create` immediately starts it. Need account-clean template, OS licensing, correct Windows identity and explicit provenance, before account-bearing three-guest creation.
3. **Minecraft launch boundary:** existing `guest.rs` expects a per-VM IP/token and custom Windows interactive launcher. Upstream App Sandbox agent covers transport/display, **not an API equivalent to launching Minecraft in the signed-in interactive Windows session**. First G1 can use manual launch; automate only after there is a source-proven secure user-session mechanism that doesn't duplicate upstream agent role.
4. **Recovery/data rights:** old VMware `QA_READY` is guest-specific and held by provider; App Sandbox snapshot branches are VHDX-backed. `lifecycle_admission.rs::Reset/SetReady` cannot reuse the old assumptions. Keep disabled until non-destructive behavior is proved and new VHDX lineage/account restoration is reasoned about.
5. **Desktop/windows:** existing `Development/apps/desktop/src-tauri/src/engine/window_arrangement.rs` discovers VMware windows and their HWND ownership; App Sandbox display windows use different process/title and interactive-session guarantees. Do not assume a selected GPU-PV guest has a separate HWND or that `online` means display-ready.
6. **Diagnostics and resource accounting:** `diagnostics.rs` reads VMX's `mks.enable3d`; `resources.rs` and `provider/mod.rs::host_working_sets_mb` collect VMware process memory; upstream `host()` has **declared**, not equivalent observed RSS or dedicated VRAM quota. Preserve `UNKNOWN` for measurements without owner.
7. **Distribution and installed identity:** `Development/engine/distribution`, Tauri package resources, `Development/apps/desktop/package.json`, and source verifier include VMware Guest Agent/setup and old name/path compatibility boundaries. They remain until approved coherent cutover; do not rename installed app or auto-delete VMware data.
8. **Privileged control:** App Sandbox elevated daemon owns HCS VM and `host.json` local bearer token. The Tauri UI must not open that file or bypass Rust typed IPC. `202` responses require observed completion; disappearance of daemon while guests are running must be surfaced, not treated as successful stop.

### Minimal adapter design to consider for M1

**Only one narrow read-only integration first, inside existing Rust `Development/engine/runtime-core/src/provider/` ownership.** Discover the trusted local App Sandbox daemon metadata, validate constrained `127.0.0.1` endpoint and token **in memory only**, query `GET /version` and `GET /vms` / `GET /vms/{name}`, validate API/schema/capabilities and return typed observations. No raw backend error/token in logs or diagnostics. Pin upstream v0.1.9 and fail closed if API changes. This is the planned implementation, **not code delivered by M0**.

Do not introduce a second frontend bridge, general-purpose HTTP relay, service manager, installer, dual-provider UI toggle, writable VM registry, daemon auto-bootstrap, GPU provisioning system or simulated readiness. If the existing `Provider` trait cannot represent read-only App Sandbox state without violating VMware invariants, first refactor that *single existing source owner* to separate neutral observation from provider-specific mutation, in the **same coherent migration**, not a second trait hierarchy retained forever.

### M0 outcome / next single bounded task

**Source map complete at method and primary-caller level.** Critical mismatches: non-existent suspend, `stop` hard-off hazard, asynchronous 202 start/create, no guest IP equivalence, VMware Base + VMX path facts, missing Minecraft per-user launcher route, non-equivalent QA_READY snapshot and upstream process/admin lifecycle. These are real architectural differences, not evidence that the GPU itself fails.

**Next:** design the precise **non-mutating Rust read-only adapter contract** (`version/host/list/status`, auth discovery and errors), including a fail-closed upstream schema and Tauri integration boundary, before any provisioning, snapshot, launch or Windows driver code is changed. Keep the current VMware implementation untouched until the new provider can meet the required product contract.

## M1 design complete — exact read-only App Sandbox contract

**Status:** CONTRACT DESIGNED / NOT IMPLEMENTED / UPSTREAM LIVE API NOT EXECUTED. Evidence: inspected App Sandbox source `src/app_win/headless.c` and `tools/headless-api/asb.py` at `6f3adb6aafd4fc819d7715bdfacf52ac87df26a6`, plus this repository's existing Rust `ureq = 2.9.7`, `serde_json` and public contract. No extra dependency, process, API endpoint, persisted state or GPU probe is approved.

### Single owner and adoption strategy

Use `Development/engine/runtime-core/src/provider/` as the only owner of App Sandbox host discovery and read-only API integration. Existing `Development/engine/runtime-core/src/runtime.rs` still owns lifecycle/admission. Keep Svelte → typed Tauri → Rust; UI must never read `host.json` or send HTTP to App Sandbox. Do **not** connect a partial App Sandbox implementation to the existing VMware `Provider` trait: it requires VMX paths, guest IP, VMware snapshot semantics and mutable VM actions. Instead, a narrowly scoped typed observation is consumed by the existing runtime **only when** a coherent neutral provider admission/diagnostics flow can use it without exposing broken lifecycle controls. Avoid permanent `asb-client` crate, second state registry, another Tauri public facade, and unconditional startup probes.

### Fixed upstream HTTP API

App Sandbox Windows headless creates `%ProgramData%\AppSandbox\host.json` with `endpoint`, `port`, `token`, `pid`, `version` and `apiVersion`. Its Python client appends `/v1` to the endpoint. Verified source: `src/app_win/headless.c::write_discovery` and `tools/headless-api/asb.py::Client/connect`.

- HTTP boundary: **GET only**, fixed paths `/v1/version`, `/v1/host`, `/v1/vms`, and later `/v1/vms/{canonical-name}`. Never expose raw user-selected URL, arbitrary path, POST, DELETE, SSH or query parameters.
- Discovery: read once for an explicit operator-initiated observation, max ~4 KiB; reject symlink/reparse/path redirection, missing/oversized/non-UTF8/invalid JSON, non-uint `pid`, invalid port (1–65535), missing token, mismatched `endpoint`/`port` and invalid version. Restrict to **literal** `http://127.0.0.1:<port>`, no DNS aliases, credentials, fragments, query strings or alternate host. No env-driven arbitrary discovery path; use actual Windows ProgramData system path.
- Authentication: v0.1.9 source emits **40 lowercase hexadecimal characters** of random bearer token (do not confuse with our VMware Guest Agent's 64-hex token). Keep token in the Rust call stack only, never serialize, persist, print, place in error context or send to a non-loopback address. Verify the daemon response with authenticated GET after the **unauthenticated** `/v1/version` product check; mere product/version strings do not prove process identity.
- Process lifetime: a stale discovery file can survive a crash; even matched PID can be reused. Valid discovery is **not** an active daemon claim. Reachability, product/API/host checks and authenticated response must succeed on the current request. Avoid caching a bearer token across restarts.
- Transport: reuse existing Rust HTTP/JSON dependency if its transport can be forced to direct loopback, **no proxy**, **no redirects**, short connect/read timeouts (proposed 2s connect, 3s overall for a normal status read), capped response bytes (proposed 256 KiB maximum, smaller for discovery), no verbose errors or raw URL/body logging. If existing HTTP client cannot enforce these, use bounded std TCP within the same owner; **do not add an HTTP framework**. No polling loop in Svelte and no SSE/event manager for this read-only slice.
- Pin exact tested upstream release `0.1.9` and API version `v1` (source constant `ASB_API_VERSION`) for first integration. Require `product == "AppSandbox"`, `hostOs == "Windows"`, expected version and typed capability booleans. Reject incompatible version/API before reading VM state. Unknown additive fields may be ignored; absent or malformed consumed fields are not silently defaulted.

### Typed facts and sources

| External GET | Required source fields | Safe representation and meaning | Reject/unknown |
| --- | --- | --- | --- |
| `/v1/version` | `product: "AppSandbox"`, `version`, `apiVersion: "v1"`, `hostOs: "Windows"`, `capabilities.snapshots: bool`, `capabilities.templates: bool` | Supported upstream release/capabilities, **not VM readiness** | Missing/incompatible fields → incompatible/unknown provider, no VM mutation |
| `/v1/host` | `hostCores`, `hostRamMb`, `freeGb`, `vmCores`, `vmRamMb`, `vmHddGb` non-negative bounded numbers | Host total and provider-declared guest allocations; `freeGb` is space on upstream's ProgramData drive, NOT necessarily VM disk target | Negative, overflow or missing → unknown; no baseline capacity PASS |
| `/v1/vms` | Array of VM records under `vms`, all VM objects have distinct `name` | Read-only inventory, select exactly provisioned tracked guest identities only after ownership binding | Duplicate names, unexpected record type, unknown required fields → fail closed; do not manage other App Sandbox users' VMs |
| `/v1/vms/{name}` | `name`, `osType`, `state`, `running`, `agentOnline`, `installComplete`, `building`, `ramMb`, `cpuCores`, `gpuMode`, `networkMode`, `displayOpen` | Observed VM process/guest signals; no Microsoft sign-in, Minecraft rendering, snapshot/identity guarantee | 404 is absent/not provisioned only after a validated inventory; authentication/malformed/transport errors must remain UNKNOWN, never STOPPED |

Per `src/app_win/headless.c::derive_state` the current Windows states are `building`, `stopped`, `stopping`, `installing`, `booting` and `online`; some transitional combinations share `running=true`. **Preserve the upstream state internally** and validate relationships (`online` needs `running` and `agentOnline`, `building` cannot mean playable, `stopped` means not running). Do not reinterpret `online` as signed-in game or accept a game-ready claim from `gpuMode`. A guest Windows OS type is required for Minecraft usage; Linux VM records are other App Sandbox resources, not Minecraft Virtual-01/02/03. Return only a **filtered non-sensitive projection**, never `diskDirectory`, `gpuId` device path, passwords, raw provider error bodies, or token.

App Sandbox `src/app_win/headless.c` also adds `gpuId` and `gpuName` to a VM response, but `gpuMode=1` and a GPU name are **configured hints only**. The public headless doc mentions `gpuMode=2` yet Windows source input validation accepts only 0/1: require only 0 or 1 for initial Windows create planning, do not expose `Try All` as a confirmed option. No VM-creation path is enabled during M1.

### Error and reconciliation semantics

| Observation | Required handling |
| --- | --- |
| Discovery absent | `PROVIDER_NOT_CONFIGURED` as diagnostic evidence; no auto-start, no VM-create |
| Stale PID/connection refused | `PROVIDER_UNREACHABLE`, not "no VMs", not healthy |
| Auth status 401 | `PROVIDER_AUTH_FAILED`; never print token |
| API version/product/OS mismatch | `PROVIDER_INCOMPATIBLE`; no fallback to VMware silently |
| HTTP 404 on named status | `VM_NOT_FOUND` only after provider handshake; no automatic recreation |
| HTTP 409/500/501 | `PROVIDER_ERROR/UNSUPPORTED`; never translate to a stopped state |
| Bad field, duplicate tracked VM name, invalid JSON, oversized response | `INVALID_PROVIDER_RESPONSE`; refuse mutation, preserve original guest data |
| Request timeout / daemon exits mid-request | `UNKNOWN`; do not assume a VM changed state or retry mutation |
| Concurrent guest startup | Not in read-only scope; future POST `202 Accepted` means admission only, then bounded status reconciliation before declaring success |

The symbolic labels above are **design vocabulary**, not new public enum values or an extra state database. Map only proven cases to the single existing `ErrorCode` family at implementation. If public status semantics need a new enum, do a versioned explicit migration rather than smuggling new meanings into schema 1.

### Consumer boundary, security and efficient resource use

- No App Sandbox request from static module import, initial UI page load, every Svelte render or global interval. Preferred eventual consumer is the existing **explicit diagnostics / setup action** once its backend admits the selected provider. Use on-demand single request when requested; do not generate continuous traffic or a new persistent cache.
- Admin App Sandbox daemon owns HCS/GPU-PV and visible Windows guest display; do not run it silently in a Windows service (Session 0), duplicate a full server, or send admin bearer token to the frontend. If daemon is unavailable, explain it as one reason in the existing Doctor instead of hiding an installation behind setup.
- Support bundle and journal must exclude discovery file, token, absolute upstream disk directories, Windows accounts and raw provider response strings. The old JVM/VMX update paths and guest disks must remain untouched during M1.
- Business/policy owner stays Rust runtime. Exact validation is **read-only**; application GUI may eventually display provider availability/capabilities but must not claim API feature readiness as Minecraft runtime evidence.
- Device budgets stay hypotheses: i7-14700F/RTX3070 8GB/32GB RAM, initial three guests at 4 GB + 2 vCPU; no fake VRAM quota or automatic GPU tuning until G1/G3 observation.
- Windows 11 **Enterprise edition confirmed by user**, *Windows virtualization entitlements unknown*. `slmgr /dli` shows activation channel/status, not the underlying right to multiple Windows VM installations.

### Minimum implementation acceptance for the next coherent batch

Implement inside **existing** Rust backend owner and simultaneously wire its one legitimate existing diagnostics consumer (without a competing VMware lifecycle). Cases to represent in targeted source regressions once code exists: valid v0.1.9 response and three named VM statuses; stale discovery; external URL/port mismatch/redirect/proxy prevention; wrong API/product; 401; response over cap; duplicate VM entries; malformed state relation; one VM 404; offline daemon; token redaction; Windows/Linux VM separation. These are targeted source tests in the existing suite, **not a new G gate** and not proof that they have run.

**M1 engineering next action:** Make one complete, read-only provider inspection path in `Development/engine/runtime-core/src/provider/` and its existing diagnostics consumer **only after** exact calls can be added without causing normal snapshot refresh to perform heavyweight polling. No speculative public CLI/Tauri command, no independent provider registry, no provider switch, no writes to VMs, no release. If no consumer can be safely added in one coherent change, keep this specification as the completed batch and do not commit unused transport code.

## Coherent migration phases (not new acceptance gates)

**M0 — Research and source ownership design (NOW).** Freeze VMware feature work and preserve historical commit/SHA. Inventory all VMware-specific provider, profile, admission, doctor, window, package, Guest Agent, VMX/QA_READY and tests. Map App Sandbox API semantics and external source risk. Decide how the existing Rust Public Contract can be kept or version-migrated. Finish on GitHub with a single source-grounded plan; no VM mutation.

**M1 — Non-destructive App Sandbox boundary.** Design a thin Rust local loopback client consuming the upstream `version/host/list/status/capabilities` API, errors, token discovery and version pin, without authoring a second daemon, frontend HTTP access, or storing token. Provider status must remain read-only while unknown or no daemon. Address elevated daemon ownership and GUI-session display in an explicit operator workflow; do not automatically install drivers or create guests during application startup. This is SOURCE WORK before M2; do not call M1 accepted without executed evidence.

**M2 — One disposable GPU-PV guest, before account-bearing clones.** Use supported Windows 11 x64; validate RTX 3070 hardware D3D11 FL11_0, hardware OpenGL >=4.4, vendor/renderer (not WARP), guest display/input and real Minecraft Education installation/manual login and world rendering. A source-written probe or upstream rotating-cube demo is supporting evidence **only**. No "PASS" until real G1 review. If either API or game fails, STOP product scaling and diagnose actual driver/renderer first.

**M3 — Single guest daily operation and identity.** Integrate Rust's observation, admission and graceful stop with upstream `start/shutdown`. Trace VM identity, Windows identity, persistence, OS update/driver compatibility, host crash handling, guest shutdown, snapshot retention, and branch behavior. Explicitly handle force-stop as recovery-only and **never** delete a VM/account disk during setup. Separate App Sandbox agent from any Minecraft-specific launch helper, and keep the host/guest authority count minimal.

**M4 — Three guests + Native capacity.** Sequential guest provisioning with account-clean template; manual independent Windows/ Education sign-in; unique IDs and MACs; expose windows on the interactive desktop; native + three simultaneous worlds; verify GPU/CPU/RAM/VRAM pressure, VHDX I/O and all required Native↔guest, guest↔guest multiplayer paths. Keep 4 GB / 2 vCPU starting budget until actual evidence dictates changes. If 32 GB host memory or 8 GB RTX VRAM proves insufficient, mark hardware capacity limit honestly instead of introducing fake resource features.

**M5 — Coherent production switch and clean-up.** Only after a viable new path exists: migrate canonical backend/domain contracts, UI window discovery, build/package payloads, source router, product/docs and validation terminology in **bounded reviewable batches**. Preserve user-installed identity and old VMware VMs/backups by explicit migration policy. Remove old VMware implementation from active source only when no consumers remain; Git history is its archive. Do not keep dual provider modes, fork App Sandbox without necessity, publish a release or alter `main` without explicit authorization.

Only the repository's existing **G0–G4** are product test gates; M0–M5 are engineering delivery phases, not extra test-progress gates. Execute proportional source checks after a coherent change, and G0/Windows/G1/G2/G3/G4 only at their genuine proof context. No local PC use is needed to finish GitHub-verifiable source work. Runtime/GPU facts are `NOT_EXECUTED` here.

## Key blocking decisions before claiming no usage cost / maturity

1. **Windows guest rights:** App Sandbox is free/MIT; three guest Windows installations might require separate licenses or qualifying existing virtualization rights. Minecraft Education itself needs proper four-account entitlement. Never call operating costs zero without confirming both.
2. **Privileged process and secrets:** App Sandbox currently admin-elevated, loopback bearer token, single instance and optional auto-login/SSH. Decide how to run it visibly and minimize privilege exposure; protect `host.json` and guest passwords.
3. **Graphics correctness:** NVIDIA native GL vs Mesa D3D12 fallback must be distinguished. D3D11/GL both on *same* guest, same host driver, same GPU, before Minecraft gameplay.
4. **Recovery and snapshot semantics:** HCS VHDX templates/branches are not VMware linked clones. Maintain old disks untouched and do not wire destructive VM deletion/branch/reset until proven.
5. **API stability:** Upstream App Sandbox is still a relatively young 0.1.x product. Pin tested release, audit schema/commit/API, and tolerate unsupported/missing fields and `202` asynchronous outcomes.
6. **Sustained capacity:** 4 clients share one RTX 3070 8 GB; 3 Windows guests share 32 GB RAM. High CPU core count does not resolve VRAM or Windows 11 memory pressure.

## Further reading (externally owned, no code copied yet)

- App Sandbox source: https://github.com/jamesstringer90/appsandbox
- v0.1.9 release: https://github.com/jamesstringer90/appsandbox/releases/tag/v0.1.9
- API/lifecycle/security: https://github.com/jamesstringer90/appsandbox/blob/main/tools/headless-api/README.md
- Mesa/OpenGL caveats: https://github.com/jamesstringer90/appsandbox/blob/main/tools/win-mesa-gl/README.md
- NVIDIA native OpenGL change: https://github.com/jamesstringer90/appsandbox/releases/tag/v0.1.6
- Intel i7-14700F specs/no iGPU: https://www.intel.com/content/www/us/en/products/sku/236854/intel-core-i7-processor-14700f-33m-cache-up-to-5-40-ghz/specifications.html
- NVIDIA RTX3070 8GB specs: https://www.nvidia.com/id-id/geforce/graphics-cards/30-series/rtx-3070-3070ti/
- Education requirements: https://edusupport.minecraft.net/hc/en-us/articles/360047556591-System-Requirements
- Windows VM licensing: https://www.microsoft.com/licensing/guidance/Windows-11-Licensing-for-Virtual-Desktops

**Next single source task:** complete a read-only mapping from existing Rust `Provider` methods and VMware-bound runtime call sites to App Sandbox `v0.1.9` headless API capabilities, classify `preserve / adapt / remove / missing / high-risk`, and design one canonical provider boundary. Do not write executable VM mutation paths or delete VMware code until that map is reviewed.
