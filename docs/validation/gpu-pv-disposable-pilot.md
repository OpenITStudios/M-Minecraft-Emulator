---
id: document.validation.gpu-pv-disposable-pilot
class: DOCUMENT
domain: validation
role: PROCEDURE
authority: WORKING
lifecycle: HISTORICAL
---

# App Sandbox GPU-PV: one disposable Windows guest

This procedure is an **isolated feasibility experiment**, not the production VMware client lifecycle. The one pilot guest is manually created using App Sandbox v0.1.9. The Rust CLI can then inspect, start, gracefully shut down and show its display. **No Rust VM-creation, deletion, hard-stop, snapshot or account-reset route exists in this pilot.** Source and Windows/GPU/game acceptance are separate.

## Host and identity boundaries

Use the exact pilot name `MVB-PILOT-01` (not Virtual-01/02/03). Its per-guest root must be:

`%ProgramData%\M-Bedrock\VirtualClients\pilots\MVB-PILOT-01\`

The VM's configured `diskDirectory` is the **parent** `%ProgramData%\M-Bedrock\VirtualClients\pilots`, and its active VHDX must be `MVB-PILOT-01\disk.vhdx` (no differencing branches). The guest is **Windows 11 x64**, and the planned initial App Sandbox allocation is exactly **4096 MB RAM**, **2 CPU cores**, **GPU Default (GPU-PV, gpuMode 1)**, **NAT (networkMode 1)**, with **zero snapshots/branches**. These are a conservative starting configuration, not proof that Minecraft works.

Create the disposable Windows guest through App Sandbox GUI or its documented API, **not by the MIVUBI CLI**. App Sandbox creation can start a Windows installation asynchronously. Complete Windows setup and power the guest down through its *graceful* shutdown mechanism before asking this pilot to start it. Only use a valid Windows guest evaluation or entitlement appropriate to the scenario. **An activated Windows 11 Enterprise host does not automatically license three Windows VM installations.** Minecraft Education accounts also require their own entitlements.

## Operator-controlled enrollment

Run the existing CLI command `virtual-clients app-sandbox-pilot inspect`. It reports a non-secret `enrollmentFingerprint`, derived from the canonical active VHDX path, VM name and persisted MAC in upstream `vms.cfg`, plus `operatorConsentValid=false` until explicitly enrolled. This is a marker for the disposable experiment, **not a trusted cryptographic attestation of VHDX contents, HCS attachment, or Windows licensing**.

After independently verifying on the local machine that the VHDX contains **no valuable account or world data**, an authorized operator may create the UTF-8 file `pilot-consent.json` **inside that specific pilot VM root**:

```json
{
  "schema": 1,
  "vm_name": "MVB-PILOT-01",
  "fingerprint": "<exact enrollmentFingerprint from inspect>",
  "disposable": true,
  "windows_entitlement_confirmed": true
}
```

The final boolean records **the operator's own licensing assertion**, not an entitlement check. Do not add this file, VM images, tokens, credentials or registry data to Git. A marker in the wrong directory, with a stale fingerprint, or inside a redirected/reparse directory is rejected. Replacing the active VHDX path or MAC invalidates the marker. **There is no automatic enrollment.**

## Explicit actions

The only pilot commands are:

```text
virtual-clients app-sandbox-pilot inspect
virtual-clients app-sandbox-pilot start --confirm-disposable-pilot
virtual-clients app-sandbox-pilot display --confirm-disposable-pilot
virtual-clients app-sandbox-pilot shutdown --confirm-disposable-pilot
```

`start` works only from `stopped` and only on the named, enrolled disposable VHDX; it uses the upstream `POST /v1/vms/MVB-PILOT-01/start` with `{}` and **never supplies a snapshot/branch selector**. `shutdown` uses upstream `/shutdown`, *not* `/stop` (which is forced power-off). `display` requires upstream `GET /display` readiness and the interactive App Sandbox daemon desktop session. HTTP `202 Accepted` is **only an acknowledgement**. The CLI now requires the VM to reach upstream **`online` (installed and App Sandbox agent connected)** before reporting `start` as observed complete; `installing` and `booting` are still pending. `online` does **not** prove an interactive user, correct graphics, Minecraft sign-in or working gameplay. For shutdown the target is `stopped`, and for display it is a reported open, ready window. The CLI rechecks the named VM configuration, MAC, consent, VHDX path, registry and the **unbranched** snapshot position after observing completion. A timeout, changed identity, or unknown status requires manual inspection, not an automatic retry, force-off or rollback.

The pilot also rejects multiple App Sandbox registry records that share a VM-directory path or active VHDX path. This check is a lexical safety invariant; it is not a physical HCS disk attachment certificate, and NTFS aliases/hardlinks are not independently attested. A direct VHDX file must exist, be non-reparse and nontrivially sized. **Before start only**, the CLI opens it for *reading* to check the 1 MiB VHDX header-section size and the official eight-byte `vhdxfile` identifier ([MS-VHDX §2.2.1](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-vhdx/23ba288b-4eda-47bf-bd48-6386489d12af)). This is basic format recognition, **not** a complete VHDX integrity or live HCS attachment guarantee; a running guest's disk is never opened for this check. The snapshot pointer must represent an unbranched base (`-1/-1` or `-2/-1`), with no snapshot or branch entries.

Before **start only**, the CLI additionally requires at least **10 GiB of currently available host RAM** (4 GiB guest allocation plus 6 GiB host reserve) and **12 GiB free on the ProgramData volume** as reported by App Sandbox. These are conservative admission floors, not gameplay performance forecasts or guarantees against dynamic VHDX growth. **Graceful shutdown remains permitted during memory or disk pressure**; neither resource check is a prerequisite for shutting down a running guest.

The pilot's host session is local-loopback authenticated, with no HTTP proxy or redirects. The Rust operation lock excludes concurrent product operations during explicit actions. Source code includes no auto-start, recurring VM polling or implicit app startup connection. Neither the normal desktop `snapshot` nor `Virtual-01/02/03` commands use this pilot route.

## Evidence required before any production cutover

Treat pilot running, IDD display ready and SSH `online` as **separate observations**. A viable G1 outcome also requires **actual hardware Direct3D 11 FL 11_0 and NVIDIA-backed OpenGL >=4.4 within Windows guest**, not WARP/llvmpipe, plus Minecraft Education licensed sign-in, world rendering and usable keyboard/mouse. Capture actual machine evidence before cloning any account-bearing VHDX. If one guest cannot pass, stop the three-guest scaling plan and diagnose the graphics/driver failure.

**Use the existing G1 collector with explicit provider attribution**, inside the interactive Windows pilot guest. The `ENVIRONMENT` and `INTERACTIVE` observations must use the same `-Provider APP_SANDBOX_GPU_PV` selection:

```powershell
.\collect-client-feasibility.ps1 -Provider APP_SANDBOX_GPU_PV -Stage ENVIRONMENT
# After personally verifying Minecraft Education and 10+ minutes of a playable world:
.\collect-client-feasibility.ps1 -Provider APP_SANDBOX_GPU_PV -Stage INTERACTIVE `
  -VmDesktopOpened YES -EducationInstalled YES -EducationSignedIn YES `
  -MenuRendered YES -WorldRendered YES -InputResponsive YES `
  -BlockingGraphicsError NO -CrashOrFreezeObserved NO `
  -ObservedMinutes 15 -EvidenceReference gpu-pv-pilot-01
```

The collector now runs a **hardware-only D3D11 FL11_0 device probe** (no WARP/REFERENCE fallback) plus an interactive WGL OpenGL context. The reviewer also rejects contradictory diagnostics—for example, `UNKNOWN` Direct3D accompanied by a claimed hardware feature level, or `MEASURED` OpenGL accompanied by a probe-failure reason. NVIDIA vendor must have a recognizable NVIDIA-family renderer (`NVIDIA`, `GeForce`, `Quadro`, `Tesla`, `RTX`, or `GTX`) and must not be a known software/generic renderer. The unchanged schema-1 report includes `provider=APP_SANDBOX_GPU_PV`, `graphics.direct3D11`, and OpenGL `version/vendor/renderer`. The evaluator refuses `OPERATOR_REPORTED_PLAYABLE` for GPU-PV unless its guest observation has a hardware D3D11 FL11_0 device, NVIDIA vendor/renderer context at OpenGL 4.4+, and the existing 10-minute signed-in, responsive-game operator evidence. A host GPU name or App Sandbox `gpuMode=1` never fills a missing observation. Even an operator-reported playable result remains **not G1 PASS** until the evidence is separately reviewed.

The production G2–G4 validation and recovery workflows remain VMware-specific; they have **not** been migrated for App Sandbox. Source-only inspection and authored Rust regressions cannot make G0/G1 pass. All G0–G4 remain `NOT_EXECUTED` until executable evidence on the same candidate exists.

Do **not** use this disposable pilot as a recovery/reset mechanism for a production account. The pilot consent file is not proof of HCS disk attachment, VM identity continuity, or Microsoft account preservation.
