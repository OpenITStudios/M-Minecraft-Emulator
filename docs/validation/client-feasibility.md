---
id: document.validation.client-feasibility
class: DOCUMENT
domain: validation
role: PROCEDURE
authority: CANONICAL
lifecycle: ACTIVE
---

# Minecraft Education Windows desktop — one disposable guest feasibility

> **Historical VMware-only procedure.** This document describes the preserved
> guest-VM implementation and its G0–G4 evidence campaign, **not** the active
> Minecraft Instance Manager + Sandboxie Plus Client 02 workflow. Do not
> provision, start, delete or reset a VM as part of the current pilot.
> Active procedure: [Sandboxie two-client feasibility](sandboxie-two-client-feasibility.md).
> Its signed-in gameplay and resource measurements are still unverified.

## One selected Windows desktop architecture

This is the mandatory G1 feasibility boundary for **one Native Windows Minecraft Education client plus three isolated Windows guests**. VMware Workstation remains the *currently implemented production provider*. The approved *future direction* is App Sandbox GPU-PV, which has an explicitly separate disposable pilot and must not inherit VMware-only acceptance assumptions. **Prove one scratch Windows guest on the selected provider** opens, installs Minecraft Education, accepts a licensed login and plays responsively before creating any account-bearing production guests. The [four-account product contract](../product/four-concurrent-accounts.md) is the sole scope authority.

## Why this comes first

**Do not scale to three Virtual clients until one Minecraft Education Windows client is actually usable on the candidate provider.** Official Minecraft Education guidance distinguishes VM-hosted Dedicated Server (supported) from VM-hosted Minecraft Education Client (not officially supported). Minecraft Education's published PC minimum lists GPU OpenGL **4.4**; VMware Workstation/Fusion product documentation lists OpenGL guest **4.3** and DirectX 11. This difference is a feasibility warning, not proof that a particular Minecraft Education build cannot render through another graphics path. VMware's public product comparison still lists DX11 / OpenGL 4.3; OpenGL 4.4 is **not guaranteed in these guests** as of October 2026. Do not spoof driver versions, install speculative GPU replacements, or claim gameplay works without actual observations.

Sources:
- Minecraft Education: https://edusupport.minecraft.net/hc/en-us/articles/360047556591-System-Requirements
- Minecraft Education server/client VM support distinction: https://edusupport.minecraft.net/hc/en-us/articles/46913335157140-Dedicated-Server-System-Requirements
- VMware graphics capability: https://www.vmware.com/products/desktop-hypervisor/workstation-and-fusion
- Minecraft Education 64-bit requirement (February 2026): https://edusupport.minecraft.net/hc/en-us/articles/360047556591-System-Requirements
- VMware 25H2/26H1 OpenGL discussion (September 2026): https://community.broadcom.com/vmware-cloud-foundation/question/any-plans-for-opengl-44-support-in-guests

## Scope and isolation

This is a **disposable single-guest compatibility pilot**, not a new production VM lifecycle, state manager, installer, or account automation.

- One Windows 10/11 **64-bit** disposable guest on either the existing VMware Workstation path or the separately controlled App Sandbox GPU-PV pilot. Minecraft Education removed Windows 32-bit support in February 2026. This is a mandatory, observed guest prerequisite, not merely a preferred configuration. Use a scratch VM, **not** the immutable Base or any Virtual-01/02/03 whose account state matters.
- Test Minecraft Education obtained through its legitimate desktop installer or Microsoft Store, with VMware Tools installed and 3D acceleration configured. The official Windows install guide distinguishes those two installation variants.
- Nothing in this gate installs, modifies, provisions, snapshots, launches or destroys a VM. Testing Minecraft sign-in and gameplay remains an explicit interactive action with a legitimate licensed account where required.
- The intended full product target remains Native + 3 Virtuals. A single-VM pilot cannot establish multi-instance performance or networking.

## Ordered execution (when real Windows guest evidence becomes available)

1. **Environment observation:** from an interactive Windows session *inside the scratch guest*, run the shipped read-only `collect-client-feasibility.ps1` with `-Provider VMWARE` (default) or `-Provider APP_SANDBOX_GPU_PV`, and `-Stage ENVIRONMENT`. Confirm that the guest OS is 64-bit, its video driver is detected and OpenGL reports a real context if possible. VMware needs VMware Tools; GPU-PV instead needs **measured hardware D3D11 FL11_0 and NVIDIA-backed OpenGL 4.4+**. Keep these signals separate.
2. **Three mandatory operator observations:** first prove the Windows guest boots into an interactive desktop, then install one official Minecraft Education Windows edition (desktop installer **or** Store, not both) and sign in using a separate legitimately licensed school/organization account. Finally, play a real Minecraft Education world for an initial **10-minute usability observation**, checking camera, keyboard, mouse, rendering, and whether the game freezes or crashes. These are not automatically measurable from Minecraft process existence.
3. **Rendering observation:** manually open Minecraft Education in that same guest; observe the actual application menu, load a playable scene/world (using a legitimate account if required), move the camera and use keyboard/mouse. Record whether there is a blocking renderer/black-screen error. Do not infer this from VMX flags, GPU names, installed packages or process existence.
4. **Capture interactive evidence:** run the collector with `-Stage INTERACTIVE` and operator-declared `YES`/`NO`/`UNKNOWN` for menu, world rendering, input and blocking graphics errors. Supply only a non-sensitive reference label for the supporting observation (e.g., `guest-proof-001`). This label is not a screenshot and does not independently prove it.
5. **Review:** transfer only privacy-reviewed JSON (no account or world content) to an authorized reviewer. Use `evaluate-client-feasibility.mjs` to check completeness and distinguish operator reports from actual graphics/process measurements. It **never** returns an official-support PASS.
6. **Decision:** keep `CLIENT_FEASIBILITY` stage `OBSERVED` until the reviewer has examined the actual interactive result. Only record `PASS` through the existing `new-stage-result.ps1` with a real evidence reference and reviewed observation; this certifies a *particular tested guest*, not official product support.

The first two collection runs can be performed without the emulator desktop application, Guest Agent, cloned Base, or the other two VMs. This allows early failure detection.

## Scripts

Run inside the single scratch Windows guest, not automatically as part of GitHub source verification. For the App Sandbox pilot, replace the default provider with `-Provider APP_SANDBOX_GPU_PV` **in both commands**. The evaluator then applies the GPU-PV hardware gates rather than VMware Tools:

```powershell
.\collect-client-feasibility.ps1 -Stage ENVIRONMENT
# After actually opening Minecraft Education and interacting with a world:
.\collect-client-feasibility.ps1 -Stage INTERACTIVE `
  -MenuRendered YES -WorldRendered YES -InputResponsive YES `
  -BlockingGraphicsError NO -VmDesktopOpened YES `
  -EducationInstalled YES -EducationSignedIn YES `
  -CrashOrFreezeObserved NO -ObservedMinutes 15 `
  -EvidenceReference guest-proof-001
```

Run the pure-Node reviewer on the captured (privacy-reviewed) JSON from an authorized development environment:

```bash
node Development/tooling/acceptance/evaluate-client-feasibility.mjs path/to/client-feasibility-interactive.json
node --test Development/tooling/acceptance/evaluate-client-feasibility.test.mjs
```

The output defaults to a timestamped temporary directory *outside the repository*. The probe reads only OS name/version/architecture, physical memory total, VMware Tools service status, video adapter display names and driver versions, guest OpenGL version/vendor/renderer when measurable, Minecraft install versions, and candidate Minecraft-related process count. It does **not** inspect usernames, tokens, account identities, world files, machine GUIDs, process command lines or PIDs.

The OpenGL context probe uses a Windows user32/gdi32/opengl32 context in the executing user's interactive session. Compilation, display-context creation or driver failure yields `UNKNOWN`, never a fabricated version. It may report a generic software renderer. Actual Minecraft rendering remains the final observable requirement.

## App Sandbox GPU-PV evidence interpretation

The existing schema-1 report is backward-compatible: historical records without `provider` remain `VMWARE`. App Sandbox collection declares `provider=APP_SANDBOX_GPU_PV`, adds `graphics.direct3D11`, and preserves OpenGL `version`, `vendor`, `renderer` plus the existing operator fields. The Direct3D probe creates only a temporary **D3D_DRIVER_TYPE_HARDWARE** device requesting FL11_0; it never falls back to WARP, software or reference rendering. OpenGL is measured using an interactive Windows WGL context. Missing/failed probes are `UNKNOWN`, not invented PASS. The reviewer rejects self-contradictory GPU-PV graphics records (`MEASURED` with a probe failure, `UNKNOWN` with claimed hardware measurements) and does not accept a generic/software OpenGL renderer as NVIDIA hardware merely because a vendor string says NVIDIA. Some legitimate NVIDIA contexts use `GeForce` or `RTX` renderer names without repeating the `NVIDIA` word; these are recognized with an NVIDIA vendor, not mistaken for generic software.

For GPU-PV only, `OPERATOR_REPORTED_PLAYABLE` requires measured hardware D3D11 FL11_0, NVIDIA OpenGL vendor/renderer at version 4.4+, and all the original operator-reported gameplay conditions. An OpenGL value reported by Microsoft GDI Generic, software/llvmpipe/WARP or an unverified graphics backend cannot satisfy this requirement. These readings are **necessary evidence**, not a promise that Minecraft Education uses the same device or is officially supported in a VM. One pilot never proves concurrency, account recovery or G2–G4.

For the intentionally bounded, manually prepared App Sandbox pilot, see [GPU-PV Disposable Pilot](gpu-pv-disposable-pilot.md). **Do not treat VMware G2–G4 operations, QA_READY or VMware Tools as interchangeable with HCS VM lifecycle semantics.**

## Outcome interpretation

| Result | Meaning / response |
| --- | --- |
| `INCONCLUSIVE` | Source alone, candidate process, display flag, unknown OpenGL or incomplete operator observation; gather more evidence |
| `OPERATOR_REPORTED_FAILURE` | User actually observed failed menu, missing rendered world, unresponsive input or blocking graphics error. Stop scaling and classify first causal boundary |
| `OPERATOR_REPORTED_PLAYABLE` | All three mandatory gates (Windows boot, Minecraft Education install/sign-in, responsive world for at least 10 observed minutes) are explicitly declared, with an interactive session, candidate process and reference label. **Still an operator report**, not official VM-client support or a multi-VM PASS |
| `WINDOWS_64_BIT_UNVERIFIED` risk | The guest reports 32-bit Windows or its architecture is unavailable. This prevents an `OPERATOR_REPORTED_PLAYABLE` verdict even if gameplay observations were entered; use a supported 64-bit Windows guest. |
| `OPENGL_BELOW_PUBLISHED_4_4_REFERENCE` risk | OpenGL string reports below published minimum. Treat as priority compatibility investigation, even if DX11/another render path appears usable |
| `OPENGL_UNMEASURED` risk | Probe unavailable. Requires visual app proof and independent graphics diagnosis; absence of result is not OpenGL 1.1 or PASS |

Stop if the single client cannot render, and diagnose **driver/render path first**. Do not paper over it with RAM/vCPU tweaks, replacing the VM provider, shader tweaks or a second lifecycle framework without representative evidence. If it works, preserve exact Windows/VMware/driver/Minecraft versions and test two then three guests as separate later milestones.

## Proof ceiling and governance

This guide and scripts are source deliverables. Neither the Windows guest script nor the app source has been executed by adding them to GitHub. `main` remains untouched. Actual Windows/VMware/Minecraft proof remains `NOT_EXECUTED` until observed. Do not turn source-level checks, mocks, screenshots of VM settings, or a successful Guest Agent request into real game-compatibility claims.
