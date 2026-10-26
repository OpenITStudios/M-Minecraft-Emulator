# Minecraft Education macOS/Android and Multi-Instance Architecture — REA Research Boundary

**Date:** 2026-10-13. **Class:** source/external research only.
**Authority:** NONE for product runtime changes; current product remains Windows
Desktop native Client 01 + Sandboxie Client 02. This document neither adds a
new provider nor approves a platform migration or multi-client readiness claim.

## What is established versus unverified

1. Minecraft Education's [official Mac installation guide](https://edusupport.minecraft.net/hc/en-us/articles/360047118792-Mac-Installation-Guide)
   describes downloading the Mac installer and moving the Minecraft Education
   application to Applications. The [official system requirements](https://edusupport.minecraft.net/hc/en-us/articles/360047556591-System-Requirements)
   list Mac (macOS 14+) separately from Android (Android 8+).
   **OBSERVED_EXTERNAL:** there is an officially supported Mac application.
2. An Android **APK** is a package format used for Android; a macOS **.app**
   is an Apple application bundle. A Minecraft Education Mac download
   being installed as an Apple app does **not** by itself prove it shares or
   does not share C++ engine components with the Android build, nor does it
   identify its rendering backend.
   **NOT_PROVEN:** the claim that the Mac application is a repackaged Android APK.
3. The application's Mac executable, nested frameworks and its Android native
   libraries have **not** been supplied for REA analysis. No binary hash,
   loader path, executable architecture, shared native-library overlap,
   dynamic frame pipeline, launcher ABI or measured memory has been captured.
   **UNKNOWN:** internal code lineage and implementation strategy.
4. Minecraft Education's official system requirements explicitly caution that
   emulation of other platforms on Linux is unsupported. This does not
   establish support for Windows Android emulation, Sandboxie, or Windows VMs.
   An alternative client/runtime is not approved by inference.

## REA investigation when an authorized official artifact is available

REA supports application artifact inventory and platform-specific inspection.
The source-verified [Apple graph](https://github.com/morluto/rea/blob/main/docs/apple-application-analysis.md)
can project .app/.dmg/IPA inventory and nested-bundle roles. Its roles are
**path conventions**; they are not proof of native execution or parsed plist
semantics. Use explicit Evidence IDs, artifact hashes, tool/provider versions
and coverage bounds, following
[REA's evidence workflow](https://github.com/morluto/rea/blob/main/.agents/skills/reverse-engineer-anything/references/evidence-workflows.md).

When authorized, collect read-only evidence in this order:

1. Pin **both** official distribution packages at the same Education game
   version/build: platform, artifact SHA-256, license/inspection authorization,
   source URL, acquisition date and size. Do not commit licensed binaries,
   signed-in profiles, Microsoft tokens or game worlds.
2. For Mac: use REA's artifact inventory, Apple application graph, plist
   inspection and Mach-O/loader inspection (when the provider supports it).
   Identify CFBundleExecutable, supported architectures, dylib resolution,
   linked frameworks, code-signing identity and library fingerprints.
   An inventory alone does not prove any code path executes.
3. For Android: inspect the *actual* authorized APK/AAB-derived artifacts,
   Android manifest, DEX and native lib directories/architectures; use the
   Android/JADX bridge when available. Hash each .so/library and identify
   embedded Java/native boundaries. Do not infer runtime behavior from package
   names or guessed filenames.
4. Compare matching symbols, compiler metadata, native-library identities,
   signature/sections and Development/engine/graphics entry points across platforms.
   A matching string is weaker than matching binary/symbol/control-flow
   evidence; a shared engine does not mean identical OS or rendering behavior.
5. If permitted and technically supported, corroborate static results with
   instrumented runtime observations of process lifetime, GPU API, memory,
   startup, input, multiplayer and actual frame delivery. Keep static and
   runtime Evidence separate.
6. Produce a finding ledger:
   artifact hash, version, provider, evidence ID, source location,
   OBSERVED / INFERRED / UNKNOWN, alternate hypothesis, missing coverage,
   reproduction instructions and a product action or explicit DEFER.
   Do not claim REA decompiled an unsupplied file.

**Current execution:** REA provider, Mac/Android official binaries and a
native runtime host were unavailable in the present GitHub-only session;
**NOT_EXECUTED**. No REA Evidence IDs have been manufactured.

## External systems compared by mechanism, not reputation

| Upstream reference | Source-backed lesson | Relevance to current Windows pilot | Status |
| --- | --- | --- | --- |
| [Sandboxie Plus](https://github.com/sandboxie-plus/Sandboxie) | Per-box process isolation, read-only membership and startup semantics | Direct implementation source for current Client 02 collector | KEEP experiment; login separation NOT PROVEN |
| [Microsoft PowerToys / FancyZones](https://github.com/microsoft/PowerToys) | Multi-monitor layouts, zones, DPI/window behavior, UI testing | Adapt individual tested HWND/layout patterns only if proven deficiency | DEFER larger 2x2 layout |
| [PresentMon](https://github.com/GameTechDev/PresentMon) | ETW-backed frame present/delivery timing; not just CPU percentages | Independent graphics evidence during real gameplay | ADAPT as optional acceptance tool |
| [Windows Performance Toolkit](https://learn.microsoft.com/en-us/windows-hardware/test/wpt/) | ETW process, CPU scheduling, I/O and resource diagnosis | Only for actual reproducible host stalls/lag | DEFER until trace exists |
| [Android Emulator acceleration](https://developer.android.com/studio/run/emulator-acceleration) | VM and graphics acceleration are independent factors | Learn measurement design, not evidence of compatible Education client | RESEARCH ONLY |
| [scrcpy](https://github.com/Genymobile/scrcpy) | Native display and keyboard/input forwarding from a real Android device | Different problem from multiple native Windows logins; may add encoding/transport latency | REJECT as replacement |
| [Waydroid](https://github.com/waydroid/waydroid) | Android-in-Linux/container principles | Not a Windows Education Desktop solution and not proof of official app support | RESEARCH ONLY |
| [REA](https://github.com/morluto/rea) | Artifact inventory, function/call traces, explicit evidence/unknown bounds | Read-only validation of shipped artifacts, not an emulator | ADAPT audit discipline |

## Product decision and non-negotiable performance proof

Do not change the current Windows 01+02 client engine or add an Android/Mac
provider based on package terminology alone. The first real blocker is
two *independently playable*, *licensed* Minecraft Education sessions; an
official-looking icon, two PIDs or signed images cannot settle it.

The source now prepares privacy-constrained two-process CPU/working-set
sampling and a schema-validated offline report evaluator. Those measurements
must be paired with live frame-time (P95/P99 where available), GPU/VRAM,
manual input, account independence, startup/recovery and multiplayer evidence
on the same machine, app version and world settings. Missing evidence is
UNKNOWN; no arbitrary FPS threshold or RAM saving claim is an automatic PASS.
A two-client comparison does not establish four-client feasibility.

The **next architecture decision** must wait for actual Windows two-client
identity/gameplay evidence. If the isolation experiment fails account
independence or 3D rendering, research the first demonstrated cause before
evaluating alternative runtime providers. No new provider or source code
migration is authorized by this research note.
