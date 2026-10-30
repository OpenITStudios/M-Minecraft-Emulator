---
id: document.validation.native-multi-instance-feasibility
class: DOCUMENT
domain: validation
role: PROCEDURE
authority: WORKING
lifecycle: HISTORICAL
---

# Minecraft Education native two-client feasibility — prior experiment

> **Historical two-client experiment.** This is not the active five-profile
> native Lazy Tester design. Old Sandboxie/VMware choices below remain only
> as provenance; use [current validation routes](README.md) for acceptance.


**Work:** emulator/native-multi-instance-feasibility. This experiment is not
a replacement production provider, automatic four-client support proof, or an
extension of the legacy VM G0–G4 PASS status. The existing VMware data,
installer identity and runtime remain untouched.

## Objective

Determine whether **one official Minecraft Education Windows Desktop
installation** can open two independently usable game windows and sustain
two separately licensed Microsoft Education sessions on one interactive
Windows desktop without a Windows guest VM or modified game binaries.

Official documentation distinguishes the Desktop and Microsoft Store
versions. Do not install both. Minecraft Education Desktop stores world
data under the normal per-user AppData location. Never change, move,
delete, inspect or copy that data during this pilot.

Sources:
- https://edusupport.minecraft.net/hc/en-us/community/posts/360072475972-Multi-World-Setup
- https://edusupport.minecraft.net/hc/en-us/articles/13106858087956-Windows-Installation-Guide
- https://edusupport.minecraft.net/hc/en-us/articles/4404785703316-Location-of-World-Files
- https://edusupport.minecraft.net/hc/en-us/articles/360061382672-Sign-Into-Minecraft-Education

**Support caveat:** Minecraft Education support explicitly said in 2019
that multiple app instances on one machine were not supported. This is a
historical platform statement, NOT a live 2026 compatibility verdict.
If the official app continues to restrict independent processes, record
that result; do not bypass its single-instance or licensing mechanisms.
The installed official Desktop and Store editions must not coexist as a
shortcut. Minecraft sign-in can also involve its own Windows authentication
helper; sandboxing file paths alone does not establish separate accounts.

## Source-to-localtest boundary

The active native pilot is a developer-mode experiment, **not** the legacy
NSIS installer. On a clean pinned `Local` checkout, source verification
is `npm --prefix Development run setup` followed by `npm --prefix Development run verify:source`; the experimental
desktop may then be exercised via `npm --prefix Development run dev`. Never use `DEV.cmd g0`,
`DEV.cmd package` or the existing NSIS bundle to install the native pilot:
the old VMware distribution contract is explicitly blocked. A successful
source check is not a Windows/Minecraft acceptance result.

Do not classify a second-game launch as NO-GO merely because the process
counter is zero. First distinguish a detector error from an actual failure
to open an independent, playable game. Evidence records no account secrets.

## Single-app preflight before the two-account experiment

The new native-first desktop offers two explicit actions:

1. **Request official Minecraft launch** makes a single launch request
   through Windows' registered official Start app only when a unique
   Desktop install is detected in the official 32-bit registry view, its
   Store counterpart is absent, and the current Windows session is
   interactive. Repeat only after observing the first game; it may focus
   the same instance again.
2. **Inspect Windows (read-only)** observes the current interactive
   Windows session once, showing candidate process and visible-window
   counts. This does not poll, enumerate email addresses, store world
   data, inspect executable paths, or read account tokens.

Requesting a launch invalidates the previous observation; inspect again
after the game has opened. The collector and active desktop now include
`Minecraft.Windows`, a process name used by Minecraft Education **and**
other Minecraft editions. Counts remain NAME_MATCH_CANDIDATE_ONLY, not
certified Education binaries, licensed accounts or actual game rendering.
Presence of unrelated Bedrock processes must be checked manually; neither
a positive count nor a zero count alone establishes game compatibility.

**Required verification order:** verify two distinct game windows;
manually sign in to two separate licensed Education accounts; load two
worlds and check input; try legitimate multiplayer; then check close,
reopen, PC restart and account separation before promoting this
architecture. The Windows evidence collector below is retained for
operator-attributed observations.

**STOP** if only one client opens, a second launch focuses the first
game, accounts cross, or multiplayer/authentication fails. Do not
copy or modify Minecraft EXEs, game state, Microsoft login broker,
guest VM disks or installed account profiles to force another instance.

## Windows experiment

Run from the repository root on a Windows desktop with Minecraft Education
legitimately installed. No elevation is needed for the collector itself.
It records only one registry version, registered Start-app uniqueness,
name-matched process IDs, main window handles, working sets and
explicitly declared gameplay flags. It never reads account details,
credentials, Microsoft tokens, process command lines or world files.

The default command is read-only:

    powershell.exe -NoProfile -File Development/tooling/acceptance/windows/collect-native-feasibility.ps1

Optionally request two ordinary launches through the registered official
Windows Start app (not through a copied executable):

    powershell.exe -NoProfile -File Development/tooling/acceptance/windows/collect-native-feasibility.ps1 -LaunchRequests 2 -ConfirmLaunch

**A launch request does not prove a second Minecraft process was created.**
The game may focus the first window instead. Do not attempt to bypass any
single-instance restriction. The tool does not install drivers, Sandboxie,
VMware, guest agents or change app authentication.

If two independent windows are actually available, log in **manually**
with two distinct legitimately licensed test accounts, load playable
worlds and observe input and gameplay for at least ten minutes.
Do not record identifiable account names or world titles.

Then run the observation separately, only with flags that were actually
observed (replace YES/NO with UNKNOWN wherever not verified):

    powershell.exe -NoProfile -File Development/tooling/acceptance/windows/collect-native-feasibility.ps1 -Stage TWO_CLIENT -DistinctAccounts YES -BothWorldsPlayable YES -IndependentInput YES -AccountCrossingObserved NO -MultiplayerJoined YES -ReopenAndRebootPersistent YES -ObservedMinutes 10 -EvidenceReference two-client-local-001

By default the report is saved as a fresh JSON under
%TEMP%\MinecraftInstanceManager\NativeFeasibility. This file is machine-observed process
candidate data **plus** operator statements, not verified accounts.

Review that specific JSON using Node:

    node Development/tooling/acceptance/evaluate-native-feasibility.mjs "<report.json>"

To execute evaluator unit tests (which do not run Minecraft):

    npm --prefix Development run test:native-feasibility

## Evidence interpretation: three levels, one evaluator

| Level | Output field | Meaning |
| --- | --- | --- |
| Process feasibility | processFeasibility | TWO_WINDOW_CANDIDATES_OBSERVED means two separate PID/HWND **candidates** in an interactive Windows session, not confirmed Minecraft gameplay |
| Account feasibility | accountFeasibility | OPERATOR_REPORTED_PLAYABLE requires a manually observed pair of separately signed-in playable worlds and independent input, plus duration/reference; it does not require multiplayer or reboot |
| Product acceptance | productAcceptance | NOT_EVALUATED always for the two-client pilot; four playable clients are a separate future verification |

The pre-existing top-level verdict remains conservative for compatibility
with older consumers: OPERATOR_REPORTED_TWO_CLIENTS requires the
operator-reported account result **plus** successful multiplayer and
separated accounts retained after reopening and reboot. Thus process
and account observations may advance while verdict remains INCONCLUSIVE.
A multiplayer failure must not be mislabelled a process-launch failure.
No level automatically verifies account ownership or official support.

A first TWO_CLIENT report can use MultiplayerJoined UNKNOWN and
ReopenAndRebootPersistent UNKNOWN while manually observing the two
licensed worlds. This may yield accountFeasibility
OPERATOR_REPORTED_PLAYABLE but no final compatibility verdict. Do not
invent positive values just to pass the stricter final criterion.

## Acceptance boundaries

The evaluator produces one of:
- INCONCLUSIVE: missing or insufficient evidence;
- OPERATOR_REPORTED_FAILURE: explicitly observed problem;
- OPERATOR_REPORTED_TWO_CLIENTS: two distinct process and visible-window
  candidates plus the required operator statements, including observed
  multiplayer success and retained independent sessions after reopening
  and a host reboot. Reports without either have an INCONCLUSIVE
  final verdict even if accountFeasibility is operator-reported playable.

Old schema-1 reports that lack reopen/reboot evidence remain readable,
but cannot be positively classified. Duplicate numeric HWNDs (even with
different hexadecimal spelling) and impossible Windows PIDs are rejected.

It never returns PASS or claims automated account verification. A human
must review the true game/window evidence and any sign-in or multiplayer
behavior before approving an architectural shift.

If one usable game remains the limit, mark native NO-GO and consult
the separate [Sandboxie Plus application-isolation pilot](sandboxie-two-client-feasibility.md),
which inspects licensed persistent box configuration and isolates observed
process membership without claiming Microsoft authentication compatibility. Do not patch, copy or inject
the game to circumvent its restrictions. Do not change the production
provider, uninstall VMware, or clear account-bearing VM disks based on
this pilot alone.

## Proof status

At authoring: REMOTE_GITHUB source only. The pilot release pipeline is
also intentionally blocked while it still carries VMware Guest Agent
resources in the old installed product identity. Use only source/build
verification, not the historical NSIS package, until a separate native
release contract and account-data preservation plan have been approved. Windows collector, two signed-in
game clients, four-account concurrency and persistence are NOT_EXECUTED.
Earlier VM installer/source G0 proof applies solely to its tested SHA.
