---
id: document.validation.acceptance
class: DOCUMENT
domain: validation
role: PROCEDURE
authority: CANONICAL
lifecycle: ACTIVE
---

# Virtual Clients Acceptance — historical VMware campaign

> **Historical VMware-only procedure.** This document describes the preserved
> guest-VM implementation and its G0–G4 evidence campaign, **not** the active
> Minecraft Instance Manager + Sandboxie Plus Client 02 workflow. Do not
> provision, start, delete or reset a VM as part of the current pilot.
> Active procedure: [Sandboxie two-client feasibility](sandboxie-two-client-feasibility.md).
> Its signed-in gameplay and resource measurements are still unverified.

## Historical VMware G0–G4 — five gates of the retired campaign

These five **sequential, outcome-based gates** are the sole indicators of progress **for the preserved VMware implementation**, not for the active native + Sandboxie experiment. Existing Cargo/Rust tests, Vitest, `verify:source`, package inspections, and Windows acceptance stages are **supporting evidence**, not additional product gates. No new runner, score, stored status schema, or readiness database is authorized.

| Gate | Product question / exit criteria | Evidence boundary | Stop / decision |
| --- | --- | --- | --- |
| **G0 — Source & Package Integrity** | One exact `Local` SHA passes source checks, Windows NSIS build, mandatory installed-payload inspection, and disposable install/repair/uninstall data-preservation checks | Executed source/package checks; details in [G0 Preparation](pre-local-test-readiness.md) | No runnable/preserved installer → do not promote candidate to game acceptance |
| **G1 — One Guest Feasibility** | One disposable Windows guest on the candidate provider boots, an independently licensed Minecraft Education client signs in, and an actual world renders with responsive input; GPU-PV also requires guest-measured hardware D3D11 FL11_0 and OpenGL 4.4+ | `CLIENT_FEASIBILITY` and host baseline; [single-guest procedure](client-feasibility.md) | Rendering, login, or input fails → stop before building account-bearing clones |
| **G2 — Provisioning & Identity** | A finalized, account-clean Base produces three isolated guests; VM/Windows identities, lineage, Guest Agent and per-VM tokens are unique; first-boot/account recovery points can be established without contaminating Base | `SINGLE_VM`, `GUEST`, `MINECRAFT`, `ACCOUNT`, `THREE_VMS`; detailed assertions below | Incomplete clone, identity collision, unsafe recovery or Base contamination → stop before four-client launch |
| **G3 — Four-Client End-to-End** | Native plus three guest Minecraft Education windows are simultaneously signed in with four distinct licensed accounts, responsive and independently controllable | `FOUR_MINECRAFT`, `FOUR_ACCOUNTS`; real visual/account observations | Any client unusable, account crossing, or four-client contention failure → do not claim product acceptance |
| **G4 — Recovery & Multiplayer** | Stop/start, suspend/resume, QA_READY reset and host reboot preserve independent account state; required Native↔Virtual and Virtual↔Virtual multiplayer paths work. Sustained-use observations support reliability, not a synthetic PASS | `RECOVERY`, `MULTIPLAYER`; `SOAK` supports performance/stability assessment | State loss, cross-client mutation, required multiplayer failure → release readiness fails |

**Stage mapping is canonical in [Windows Acceptance Campaign](../../Development/tooling/acceptance/windows/README.md).** `HOST` is a G1 baseline, not a separate product gate. An `SOAK` report supplements G4; it does not replace either recovery or multiplayer proof. Earlier documents' four source/package checks belong **inside G0**, not to independent G1–G4 gates.

### Gate evidence and decision rules

- Review each **gate on the same exact product SHA** and with its dependency evidence. `PASS` requires all mandatory outcomes observed and reviewed; an authored test, source-text assertion, successful CLI process, or operator declaration without reviewed gameplay evidence is insufficient.
- `FAIL` means a required outcome actually failed. `BLOCKED` means execution could not proceed due to an established prerequisite failure. No execution or absent prerequisites means `NOT_EXECUTED`, not `PASS` or `FAIL`; partial real evidence can be noted as `OBSERVED` without promoting the gate.
- The existing Windows recorder keeps its `NOT_RUN / OBSERVED / PASS / FAIL / BLOCKED` vocabulary and schema-1 records unchanged. For the five-gate **human-facing summary only**, `NOT_RUN` corresponds to `NOT_EXECUTED`. Individual stage PASS records cannot by themselves promote a whole gate.
- Run a **coherent engineering batch first**; then run only the contract regressions needed for its risk. Do not repeat a full product campaign after every small edit. A failed gate creates one root-cause issue at its first owning module; retest the affected gate and dependent gates when a genuinely executed candidate is available.
- This repository's working mode is ChatGPT + GitHub `Local` only. Executable builds, installers, Windows guests, license sign-in and real gameplay currently lack execution evidence; **all five gates remain `NOT_EXECUTED`**. Do not invent a machine, a test run or a passing result.

## G1 detailed feasibility procedure — Minecraft Education inside a VM

Before production Base registration, cloning or a four-client promise, follow [Single Guest Client Feasibility](client-feasibility.md) with one **disposable** Windows guest on the candidate virtualization provider. Collect its measured OpenGL context, VMware video adapter/driver, and real Minecraft menu/world rendering and keyboard/mouse interaction. Stop multi-instance escalation if real rendering fails.

This pilot does not need the manager's Base, Guest Agent or three-clone lifecycle and must not mutate production runtime state. Operator observations are not automatic game/VM support claims. Source-only development may continue independently even while target-machine pilot is `NOT_EXECUTED`.

### Three required observations within G1

Before cloning three account-bearing Windows guests, use [One Guest Feasibility](client-feasibility.md) and preserve actual machine evidence on the selected provider that:

1. One disposable Windows guest **boots into an interactive desktop** with graphics and input available; VMware requires VMware Tools, while GPU-PV additionally requires measured guest D3D11 hardware FL11_0 and NVIDIA-backed OpenGL 4.4+.
2. Minecraft Education **Windows desktop or Microsoft Store edition** is installed **inside that Windows guest** (never both), and an independent licensed organization account signs in to the actual game menu.
3. A playable Minecraft world **renders, accepts input and remains responsive** during an observed session with no blocking graphics error, crash or freeze.

The repo collector distinguishes machine observation from user-declared account/login/gameplay. It cannot autonomously prove Microsoft authentication or frame-rate quality; do not award PASS from a process counter, report filename or runtime configuration flag.

## G0 supporting source assertions

The canonical local source checkpoint must pass before target-machine acceptance:

```text
cargo check --locked --all-targets
cargo test --locked --all-targets
cargo fmt --all --check
```

Hosted source proof does not prove VMware, GPU, input, Microsoft sign-in, or Minecraft runtime behavior.

## Version authority proof

Before provisioning:

```text
doctor
→ Native Minecraft version detected
→ Base present + stopped
→ register-base
→ baseState = REGISTERED
→ runtimeProfile.parity = MATCH
→ C:\ProgramData\M-Bedrock\VirtualClients\finalize-base.ps1 -ConfirmGeneralize
→ Sysprep /generalize /oobe /mode:vm
→ Base shuts down
→ baseState = FINALIZED
→ doctor nextSetupAction = PROVISION_VIRTUALS
```

Changing/updating Native Minecraft without rebuilding/re-registering Base must produce:

```text
runtimeProfile.parity = MISMATCH
start/provision blocked
```

## Guest Agent proof

For every Virtual:

```text
start
→ VMware Tools ready
→ Guest Agent reachable
→ Guest Agent protocol supports Minecraft's fixed launcher request (status-only v1 is not accepted for Base registration/daily start)
→ live Minecraft version = Native version
→ versionParity = MATCH
```

A newly started mismatch must be stopped and batch rollback attempted.

Base lineage proof additionally requires:

- replacing/re-registering a Base at the same Minecraft version creates a new `baseGenerationId`;
- Virtuals from an older Base generation are rejected until reprovisioned from the current Base;
- application package-version changes alone do not invalidate a Base when the Guest Agent protocol remains compatible;
- unknown Guest Agent protocol versions remain blocked.

Base lifecycle proof also requires:

- `register-base` refuses to boot a Base already marked `FINALIZED`;
- `provision` and `reprovision` refuse any Base whose canonical state is not `FINALIZED`;
- `prepare-base.ps1` copies `finalize-base.ps1` into `C:\ProgramData\M-Bedrock\VirtualClients` so finalization does not depend on temporary setup media;
- the persisted `finalize-base.ps1` requires explicit `-ConfirmGeneralize`, a `REGISTERED` Base, clean Microsoft device registration, VMware Tools, and successful Sysprep;
- the pre-existing `guestinfo.virtualclients.baseState` value changes `REGISTERED → FINALIZING → FINALIZED` and remains `FINALIZED` after shutdown;
- a failed Sysprep attempt must not produce `FINALIZED`;
- missing/legacy Base state routes back to `REGISTER_BASE`;
- `FINALIZING` routes to `REBUILD_BASE` and must never be treated as safe to provision.

Guest Agent security proof also requires:

- the agent has no fallback/shared token source; Windows guests read only `guestinfo.virtualclients.token`;
- cloned Virtual VMX files rotate the inherited Base token during the production hardware-policy path;
- Base preparation rejects Microsoft Entra joined or Workplace joined source VMs;
- Guest Agent exposes only a SHA-256 fingerprint of Windows MachineGuid, never the raw identifier;
- `Base`, `Virtual-01`, `Virtual-02`, and `Virtual-03` do not share a Guest Agent token;
- invalid or missing tokens receive no status payload;
- the Windows firewall rule is scoped to the installed Guest Agent executable, TCP 47831, and `LocalSubnet`;
- the scheduled task has no finite execution-time ceiling for the long-running agent;
- Base preparation registers the startup task but does not launch the agent before host-side guestinfo token injection.

## Target-machine proof

```text
provision
→ start 3
→ verify-identities
→ vmIdentity = UNIQUE for all three
→ windowsIdentity = UNIQUE for all three
→ doctor nextSetupAction = CREATE_READY_SNAPSHOTS
→ Virtual-01 manually playable
→ configure account once
→ verify signed-in Minecraft menu
→ stop Virtual-01
→ set-ready Virtual-01
→ repeat account setup + stop + set-ready for Virtual-02 / Virtual-03
→ doctor nextSetupAction = READY
→ reset Virtual-01
→ verify account session remains usable
→ suspend/resume Virtual-01 without signing in again
→ stop Virtual-01
→ start 1 and verify account session remains usable
→ resources 3
→ start 3
→ open Native manually
→ control Native + 3 Virtual clients
→ host/join Minecraft Education multiplayer
→ disconnect/reconnect one Virtual
→ suspend one Virtual
→ verify stop-all rejects the suspended member before stopping other Virtuals
→ explicitly resume that Virtual
→ stop all gracefully without automatic hard power-off
→ start 3 again
```

Required observations:

- Base contains no Microsoft/Minecraft user session;
- each Virtual uses its own account session and does not inherit another Virtual's session;
- `start`, `stop`, `restart`, `suspend`, resume and `open` preserve the selected Virtual's guest/account state;
- repeated daily starts do not require sign-in again while the Microsoft/Minecraft session remains valid;
- `verify-identities` persists only identity fingerprints into client provenance, never account/session secrets;
- `set-ready` rejects a Virtual without verified VM + Windows identity provenance;
- `QA_READY` is created after account setup and reset restores that configured checkpoint;
- `reset` restores `QA_READY`, boots the selected Virtual, and returns `RUNNING`;
- `reprovision` is explicitly destructive to the selected Virtual's saved account session and requires explicit destruction confirmation;
- no Microsoft password/token/session secret is written into Virtual Clients runtime/profile data;
- all four clients remain manually controllable;
- all clients run exactly the same Minecraft Education version;
- accounts are licensed and in the same Microsoft 365 tenant;
- network path supports peer-to-peer multiplayer;
- Base remains unchanged;
- clone UUID/MAC identities remain unique;
- with Virtual-01/02/03 running together, each reports `windowsIdentity = UNIQUE`;
- 4 GB is treated as a ceiling, not measured resident use;
- host working-set telemetry is plausible;
- `startDecision = BLOCK` blocks new starts without killing existing clients;
- suspend/resume remains usable;
- QA_READY reset affects only the selected Virtual;
- reprovision affects only the selected fully stopped Virtual;
- rollback failures are surfaced explicitly.

## Four-client concurrency and sign-in acceptance

The product target is one Native Minecraft Education client plus Virtual-01,
Virtual-02 and Virtual-03. Treat that as an unproven target-machine capability
until every layer below passes together.

### Interactive sign-in

For each Virtual independently:
- Minecraft must launch into the interactive signed-in Windows desktop, not
  session 0 or a SYSTEM-only desktop;
- selecting Minecraft Education sign-in may open an embedded sign-in surface,
  system browser window/tab, or another Microsoft authentication surface. The
  product must not assume one fixed presentation;
- any browser/window spawned for authentication must remain inside that
  Virtual's Windows session and be manually controllable through that VM;
- redirect/deep-link completion must return authentication to the same
  Virtual's Minecraft instance;
- MFA, Conditional Access, tenant branding, consent, password reset and
  federation redirects must remain usable;
- closing/reopening Minecraft after successful Remember-me/session persistence
  should not require a new sign-in unless Microsoft policy/session expiry does;
- Virtual Clients must never capture, proxy, copy or persist Microsoft
  passwords, MFA codes, OAuth tokens, cookies or Minecraft session secrets;
- each Virtual must be configured with its own licensed organizational account
  where the intended multiplayer scenario requires distinct players.

Current source warning: the Guest Agent startup task runs as SYSTEM. A
SYSTEM/session-0 agent is not accepted as proof that Minecraft or its
authentication UI can appear in the interactive user session. Auto-launch is
not target-machine-ready until this boundary is proven or corrected.

### Four simultaneous Minecraft clients

Prove Native + Virtual-01 + Virtual-02 + Virtual-03 concurrently, not one at a
time:
- all four reach an interactive Minecraft menu concurrently;
- each Virtual retains its own Windows/Minecraft account identity;
- no Minecraft instance exits because another instance started;
- no Store/Desktop package single-instance behavior redirects launch into a
  different Windows session;
- VMware 3D acceleration remains available in all three guests concurrently;
- keyboard/mouse focus can move between all four without stuck capture;
- audio does not block launch or make the workflow unusable;
- host CPU, committed memory, resident working sets, GPU memory/3D usage and
  disk pressure remain observable while all four are active;
- `startDecision = BLOCK` blocks additional starts without killing already
  running clients;
- suspend/resume one Virtual does not terminate or corrupt the other three;
- restarting/resetting one Virtual does not move another account/session into
  that VM;
- Window Layout discovers exactly the intended Native + three VMware windows
  after all four Minecraft clients are running.

### Multiplayer/network concurrency

Prove the real multiplayer path with four distinct clients:
- all four have network connectivity at the same time;
- DNS, Microsoft authentication, Minecraft services and required multiplayer
  endpoints are reachable from every guest;
- NAT/VMware networking gives each guest a usable independent network identity;
- one client can host and the other three can join as intended;
- repeat with Native as host and with one Virtual as host;
- peer discovery/join is not assumed to work merely because Internet access
  works;
- host firewall, guest firewall, VMware NAT/bridged mode and router isolation
  must be recorded for the accepted configuration;
- disconnect/reconnect one Virtual while the other clients remain in session;
- suspend/resume one joined Virtual and record whether Minecraft reconnects,
  returns to menu, or requires manual rejoin;
- duplicate account use, license rejection, tenant mismatch and multiplayer
  permission errors must be distinguishable from network failures.

### Persistence/isolation

After all four are configured:
- stop/start all three Virtuals and verify each returns to its own account;
- restore QA_READY independently on each Virtual and verify no account crossing;
- prove Base remains free of Microsoft/Minecraft user sessions;
- prove reprovision is the only normal operation that intentionally destroys
  the selected Virtual's saved account state;
- repeat after host reboot and Virtual Clients app restart.

## Minecraft auto-launch acceptance

Daily client lifecycle now treats a usable Virtual as VM + Minecraft, while
keeping those responsibilities separate internally.

Required proof:
- `start-client` and batch `start` start/resume the VM, verify Guest Agent /
  lineage / Windows identity, request Minecraft Education launch, then open the
  VM console window;
- Minecraft launch is idempotent: if Minecraft is already running, no second
  Minecraft process/window is created;
- Resume preserves an already-running Minecraft process and launches it only
  when it is absent;
- Restart and QA_READY reset wait for guest compatibility, launch Minecraft and
  reopen the VM console;
- first-time `start-setup` does NOT auto-launch Minecraft during Windows OOBE;
- launch failure leaves the verified VM running and reports specifically that
  Minecraft could not be opened; it must not roll the VM back merely because
  the guest application failed;
- console-open failure after successful launch reports that VM + Minecraft are
  running but the client window could not be opened;
- a running VM with `minecraftRunning=false` remains a VM/runtime observation and may offer Launch Minecraft when backend action admission allows it; `minecraftRunning=true` is presented only as Minecraft process running, never as signed-in or multiplayer proof;
- Guest Agent protocol v2 is required for launch capability. Protocol v1 remains
  status-readable for diagnosis/migration but is not sufficient for a current
  Base/client lineage;
- `POST /minecraft/launch` accepts no executable, path, command or arguments
  from the host and remains authenticated with the existing per-VM token;
- launch waits are bounded and an absent Start-menu/App identity produces a
  recoverable error rather than arbitrary shell execution;
- normal launch never automates Microsoft/Minecraft account sign-in and never
  reads account/session secrets.

Record actual Minecraft Education process/AppID behavior on the canonical
Windows image before broadening any process matching or launch fallback.

## Window Layout acceptance

After Native and Virtual-01/02/03 are open, verify the desktop orchestration
without changing Minecraft worlds or guest configuration:

- default Arrange uses Grid on the saved display, or the current primary display
  when the saved display is no longer available;
- Grid places 1–4 discovered windows without an unused three-window quadrant;
- Focus gives the selected Main window the large region and places the remaining
  discovered windows in the side column;
- Columns divides the selected display working area across discovered windows;
- Apply persists the selected layout/display/Main-window/Screen-Overlay
  presentation preference; Cancel does not persist draft changes;
- Reset to defaults changes only the dialog draft until Apply;
- Screen Overlay is click-through and does not take keyboard focus from
  Minecraft;
- default overlay shows Screen number + label and custom labels are limited to
  32 characters;
- Identify screens temporarily enlarges identification and returns to the latest
  normal overlay without restoring an older layout;
- disabling Screen Overlay and applying the layout removes existing overlays;
- overlay failure is reported as a warning and does not misreport successfully
  arranged Minecraft windows as unarranged;
- closing Virtual Clients removes its native overlay windows;
- manually moving a Minecraft window is not continuously overridden; Arrange
  explicitly reapplies both window and overlay positions;
- missing/crashed clients are reported as missing and no background auto-reflow
  daemon is introduced;
- mixed-DPI displays keep Window Layout and overlay coordinates aligned;
- Native window discovery does not claim an unrelated window with a matching
  title; ambiguous identity fails closed.

Exercise Grid, Focus and Columns on the primary display. When a second display
is available, repeat Apply there, disconnect/reconnect that display, and verify
the saved-display fallback. Record actual Minecraft/VMware process names and
window titles as acceptance evidence before strengthening identity matching.

## Support boundary

Virtualization itself is not treated as officially supported by Minecraft Education until this target-machine campaign proves the complete workflow.

Windows + VMware Workstation is the primary target.

macOS/Fusion remains a separate acceptance campaign.

## Distribution proof

Windows distribution acceptance requires:

- exact source version equals Cargo/package version;
- release binaries built with `--locked`;
- current-user installer builds from exact source SHA;
- uninstall preserves Runtime data by default;
- provenance records source SHA, version, runtime schema and installer SHA-256;
- update check performs no install or runtime mutation;
- staged installer URL matches the configured GitHub release channel;
- staged installer SHA-256 matches manifest metadata;
- trusted build embeds the expected publisher certificate thumbprint;
- staged installer Authenticode status is Valid and signer thumbprint matches both embedded publisher identity and manifest metadata;
- update apply remains unavailable while any Virtual is RUNNING or SUSPENDED;
- update staging never changes Native/Base/Virtual Minecraft lineage;
- self-update remains disabled until an explicit apply implementation is reviewed and target-machine accepted; staging signature verification alone does not authorize execution.


## Windows acceptance execution

Use the installed evidence collector instead of manually copying CLI output:

```powershell
& "$env:LOCALAPPDATA\Programs\M-Bedrock Virtual Clients\acceptance\windows\collect-acceptance.ps1" `
  -VirtualClients "$env:LOCALAPPDATA\Programs\M-Bedrock Virtual Clients\virtual-clients.exe"
```

After all three Virtual instances are running and first-boot Windows setup has completed:

```powershell
& "$env:LOCALAPPDATA\Programs\M-Bedrock Virtual Clients\acceptance\windows\collect-acceptance.ps1" `
  -VirtualClients "$env:LOCALAPPDATA\Programs\M-Bedrock Virtual Clients\virtual-clients.exe" `
  -VerifyIdentities
```

The collector writes timestamped evidence outside the repository by default and captures `doctor`, `status`, `diagnostics`, `resources 3`, optional `verify-identities`, and one `summary.json`. It never reads or stores Microsoft credentials, account names, worlds, or gameplay content.

First-boot acceptance after Sysprep also requires:

- each newly provisioned Virtual completes Windows specialization/OOBE only during initial setup;
- first boot does not modify the immutable Base;
- after initial setup, stop/start does not return the Virtual to OOBE;
- Guest Agent becomes reachable after first-boot setup;
- `verify-identities` returns `vmIdentity = UNIQUE` and `windowsIdentity = UNIQUE` for all three;
- account sign-in happens only after identity verification;
- later daily starts preserve the configured Windows and Minecraft account state.
