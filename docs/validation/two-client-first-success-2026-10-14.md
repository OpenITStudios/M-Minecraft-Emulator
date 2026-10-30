---
id: document.validation.two-client-first-success-2026-10-14
class: DOCUMENT
domain: validation
role: REPORT
authority: WORKING
lifecycle: HISTORICAL
---

# First two-client success and subsequent crash — 14 October 2026

## Outcome and revision boundaries

The operator confirms the first successful native Client 01 + Sandboxie
Client 02 test: distinct licensed Education accounts, concurrently playable
worlds, permitted multiplayer, and distinct accounts after Client 02 reopen.
Client 02 subsequently crashed while left idle, according to the operator.
This establishes basic feasibility, not stable or complete localtest acceptance.

The source baseline was GitHub Local `2a5552518179e9c8fe6a4b27fb09a19457ad0f9d`.
Full source preflight PASS applies only to
`588046b3e989e6c1aa4a0737a085600b25b2a098`, with 116 desktop tests,
frontend/Tauri/runtime checks and rustfmt; one symlink test was skipped.
The later approved machine configuration was applied at
`ef214ff60239ce88c02a479cc62afd43b8d015b8`. This report's parent is
`54654eca6d9c74d2c21ac3098e3cae89b71cb85e`; subsequent commits were handoff
documentation, not a fresh exact-HEAD source preflight or package validation.

| Result | Status | Evidence basis |
| --- | --- | --- |
| Source preflight at the specified source SHA | PASS | Executed manifest and checks |
| Two responsive game process/windows | PASS | Agent process/window observation |
| Client 02 completed Microsoft login | PASS | Operator report after compatibility change |
| Distinct accounts with simultaneous playable worlds | PASS | Operator report; no account inspection |
| Permitted multiplayer between both clients | PASS | Operator report |
| Account separation after Client 02 close/reopen | PASS | Operator report |
| Windows restart persistence | NOT_EXECUTED | Operator deferred restart |
| Stable sustained concurrent use | FAIL | Operator reports boxed Client 02 crash while idle |
| Completed 30–60-minute resource profile | NOT_EXECUTED | Duration unknown; no completed continuous collection |
| First crash cause / responsible component | UNKNOWN | No matching Minecraft Application event found in the queried two-hour window |
| Package/release or Client 03–04 acceptance | NOT_EXECUTED | Outside this test |

PASS entries above describe their individual observations. They do not erase
the later crash or establish the final two-client acceptance verdict.

## Environment and successful authentication configuration

- Windows 11 Pro, build 26100.
- Intel Core i7-14700F, 20 cores / 28 logical processors.
- NVIDIA GeForce RTX 3070; installed driver version `32.0.16.1062`.
- Official Microsoft-signed Minecraft Education Windows Desktop 1.26.3200.0.
- Sandboxie Plus 1.18.5; persistent box `MinecraftClient02`.
- Executed source toolchain: Node 24.21.0, npm 11.19.0, Rust 1.98.1 MSVC,
  PowerShell 7.6.5, installed MSVC/WebView2.
- Education account entitlement and suitable Sandboxie licensing are
  operator-confirmed; certificate contents and identities were not collected.

Standard-box Microsoft sign-in failed after email, before password, with
tag `7q6ch` / HRESULT `0x80070424`. A credential-free native C++/WinRT
provider-discovery probe reproduced that HRESULT in the box and passed on
the host. Host TokenBroker was present and running; activation across the
box boundary, rather than a missing host service, was implicated.

With explicit operator consent and normal game closure, the following
compatibility settings were applied using official Sandboxie tools:

```ini
NoSecurityIsolation=y
DropAdminRights=y
Template=OpenCOM
AutoDelete=n
AutoRemove=n
OpenCredentials=n
OpenProtectedStorage=n
```

OpenCOM was appended; existing templates were preserved. The native probe
then returned exit 0 in the actual game box. One game launch was requested
through Compact Manager, the password stage was observed, and the operator
completed manual sign-in. No account/token copying or authentication input
automation was performed.

Application Compartment deliberately reduces security isolation; host COM
access is part of the compatibility choice. File/registry virtualization and
OFF credential-sharing flags alone do not establish a separate Windows
security identity. The reported account separation is runtime evidence for
this specific test, not a universal isolation guarantee.

## Resource observation and crash qualification

The supplied cropped Task Manager image shows two Education rows:

| Screenshot row | CPU reading | Memory reading | Rightmost percentage |
| --- | --- | --- | --- |
| Upper | 9.0% | 2,093.2 MB | 2.1% |
| Lower | 33.7% | 1,885.1 MB | 19.5% |

Column headings and PIDs are absent. The rows cannot be reliably assigned
to native/boxed clients from the image alone. The rightmost column appears
to be GPU in context, but its heading/engine was not captured. These are
instantaneous readings, not a time series, frame-time comparison, or proof
of a CPU bottleneck or disabled hardware acceleration.

A separate machine-local sample observed both earlier game candidates
responding, with working sets approximately 2.42 GB and 2.15 GB. Its CPU
values were cumulative CPU seconds, not instantaneous CPU percentages;
memory counters are not interchangeable with the cropped Task Manager view.
No FPS, GPU-engine/VRAM series, thermal data or new acceptance threshold
was established.

The operator started manual sustained observation and then reported Client 02
crashing while left idle. At a subsequent read-only inspection only the
previous native game process remained. Process disappearance supports the
report but does not supply an exception, stack, or first causal owner.
The two-hour Application event query for IDs 1000/1001/1002 returned no
matching Minecraft records; this is not proof of the absence of a crash.
The proposed background resource recorder had not been launched before
the prior turn was interrupted. No continuous monitor result is claimed.

Ignored local evidence is retained in
`workspace/runs/20261013T191850Z-588046b3-preflight-22609f43/`.
Git receives only redacted metadata/prose, not the screenshot, configuration
file, identities, worlds, raw authentication events or crash dumps.

## Integration discussion — proposals, not implementation approval

**Subsequent user decision:** the user chose the fork/owned-engine direction,
requested REA assistance and moved continuation to ChatGPT + GitHub.
The official-tools recommendation below records the earlier discussion, not
the selected future direction. The [current development handoff](../../planning/development.md)
owns the selected research scope, prerequisites and next source task.
No replacement engine or distribution was implemented by that decision.

The existing backend already launches through official Start.exe and
inspects through the canonical collector. A smoother setup can extend this
existing owner: detect a supported signed installation, check prerequisites,
preview creation/configuration of the one persistent box, apply an explicitly
approved compatibility profile, and retain rollback metadata for affected
keys. License entry and Microsoft sign-in remain manual. Never reset an
existing account-bearing box to simplify onboarding.

This orchestration route does not require a new isolation engine or importing
SandMan UI code. Bundling an official dependency is a separate installer,
distribution/license, update and uninstall-preservation decision; the current
legacy packaging hold remains. Rebuilding/forking Sandboxie's driver, service
and injected DLLs would introduce Windows driver signing, compatibility and
security maintenance responsibility without demonstrated performance gain.

Sandboxie licenses are per component: core GPLv3, QSbieAPI LGPL and SandMan
custom terms. The 1.18.5 Plus license permits free redistribution of original
unmodified binaries subject to its terms and separately conditions modified
build distribution and commercial use. This is not blanket permission to
rename or relicense the entire product. No redistribution design is approved.

## Performance/crash investigation proposed for discussion

Ranked falsifiable hypotheses, not diagnosed causes:

1. Different scene, focus state, resolution, render/simulation distance or
   frame pacing accounts for the resource difference. Match workload and
   settings, swap which client is foreground, and compare frame times.
2. Sandboxie hooks or a compatibility interaction adds overhead or causes
   the idle exit. Run the same manual idle scenario separately native/boxed;
   correlate the first failing module/stack and generic Sandboxie messages.
3. The graphics driver/render path fails. Capture actual GPU Development/engine/adapter
   and frame-time evidence; a low aggregate GPU percentage alone is insufficient.
4. An idle/background transition triggers a Minecraft failure independently
   of peak load. Compare both-foreground, background and minimized states
   separately while preserving saved worlds and accounts.

CPU simulation, chunk preparation and game logic cannot simply be moved to
the GPU by the manager. UI integration by itself will not reduce game engine
work. Affinity, priority, credential-sharing changes, forced relaunch and
speculative driver/template changes are not approved optimizations here.

The next engineering decision should follow attribution of the first crash
owner and a matched native/boxed workload comparison. No source/API/schema,
installer, driver or resource-policy change is made by this report.

## Primary references

- [Sandboxie 1.18.5 component/license terms](https://github.com/sandboxie-plus/Sandboxie/blob/v1.18.5/LICENSE.Plus).
- [Application Compartment compatibility and isolation model](https://sandboxie-plus.github.io/sandboxie-docs/PlusContent/compartment-mode/).
- [Official Start.exe interface](https://sandboxie-plus.github.io/sandboxie-docs/Content/StartCommandLine/).
- [Microsoft: GPU engine readings in Task Manager](https://devblogs.microsoft.com/directx/gpus-in-the-task-manager/).
- [Microsoft: kernel driver signing requirements](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/kernel-mode-code-signing-requirements--windows-vista-and-later-).
