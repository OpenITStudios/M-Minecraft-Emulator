---
id: document.validation.owned-isolation-upstream-inventory
class: DOCUMENT
domain: validation
role: REPORT
authority: WORKING
lifecycle: HISTORICAL
---

# Owned Isolation — pinned upstream component, license and build inventory

> **HISTORICAL v1.18.5 design/inventory snapshot.** The only current
> owned v1.18.6 source/build route is [Isolation Source Tooling](../../Development/tooling/isolation-source/README.md).
> External checkout paths, old fork procedures and pending tests below are
> not current `Local` commands.


**Review date:** 2026-10-14. **Product source:** `MIVUBI-STD/M-Minecraft-Emulator` `Local` at `8f58f26b800fcd89188224efb862930b6f2e0ec9`.
**Upstream:** [sandboxie-plus/Sandboxie v1.18.5](https://github.com/sandboxie-plus/Sandboxie/tree/abd4c240031cfb46666daa74ca47ed4e48b561e4), exact tag commit `abd4c240031cfb46666daa74ca47ed4e48b561e4`. This is SOURCE evidence; installed Sandboxie 1.18.5 has not been proven byte-for-byte identical to an independently compiled artifact. No upstream binary or modified driver was compiled or executed in this batch.

## User outcome and precise independence boundary

Personal, educational/noncommercial use. Build an **app-owned isolation distribution** which, after independent build, driver signing/loading and Minecraft acceptance, can operate without requiring a separately installed official Sandboxie-Plus / SandMan manager. The first engine will remain a **copyrighted Sandboxie-derived implementation**, NOT an independently written sandbox. The separate, later aspiration of no Sandboxie-derived code requires a different design/clean-room implementation and is NOT achieved by a fork.

The proven two-client baseline stays unchanged: one official native Minecraft Education and Client 02 in persistent box `MinecraftClient02`. Evidence of independently playable accounts and multiplayer is operator-confirmed; Client 02 idle crash cause is UNKNOWN. No client/account/world migration, new driver installation, license bypass or box deletion is authorized by this inventory.

## Component boundary — not a single drop-in library

| Source group at tag | Role and evidence | Fork decision |
| --- | --- | --- |
| `Sandboxie/core/drv` (`SboxDrv.vcxproj`) | Kernel SbieDrv: process and box policy, files, registry, IPC; WDK driver | Required, high risk; preserve matching service/DLL ABI |
| `Sandboxie/core/svc` (`SboxSvc.vcxproj`) | SbieSvc privileged service, process start, COM/RPC, IPC and broker-related helpers | Required; no service/privilege policy changes initially |
| `Sandboxie/core/dll` (`SboxDll.vcxproj`) | SbieDll, injected hooks, Win32/COM, filesystem/registry redirection | Required; maintain 32/64-bit build compatibility |
| `Sandboxie/core/low` (`LowLevel.vcxproj`) | LowLevel injection support; `lowlevel.rc` embedded in SbieDll and service support | Required |
| `Sandboxie/common`, `Sandboxie/msgs` | Shared code, headers, messages, generator | Retain until actual build graph validates minimal closure |
| `Sandboxie/apps/start` | Start.exe: existing app launches Client 02 with `/box:MinecraftClient02`; project links SbieDll and Common | Preserve for baseline parity; native adapter replacement later |
| `Sandboxie/apps/ini` | SbieIni.exe: current collector queries persisted box configuration; project links SbieDll | Preserve for baseline parity; migrate config adapter only after proof |
| `Sandboxie/apps/com/*` | RpcSs, DcomLaunch, Crypto, BITS, WUAU wrappers | Build/behavior inclusion CONDITIONAL; COM/Broker compatibility forbids speculative pruning |
| `Sandboxie/install/kmdutil` | Driver/service installation control helper | Build/installation candidate; installation policy not approved |
| `Sandboxie/SboxHostDll` | Optional host-process injection path, upstream describes Office-oriented use | Build-solution dependency; runtime exclusion for Education must be demonstrated, not assumed |
| `SandboxiePlus/QSbieAPI` | LGPLv3 user-mode IPC-based API implementation, Qt-linked | API/reference candidate; do not embed Qt only to access one operation |
| `SandboxiePlus/SandMan`, Classic `apps/control` | Upstream user interfaces | Not part of app UI; Tauri + Svelte remain canonical |
| Plus installer, Qt utilities, shell integration | Upstream packaging/UX add-ons | No wholesale import; independently inspect needed runtime and license assets |

### Observed build edges

- `Sandboxie/Sandbox.sln` includes SboxSvc, Start, Common, SboxMsg, COM wrappers, SboxDll, LowLevel, SboxDrv, SbieIni, Parse, KmdUtil, SboxHostDll and Classic Control, among others.
- `Sandboxie/SandboxDll.sln` includes SboxSvc, SboxDll, LowLevel, SboxMsg, Parse and SboxHostDll.
- `Sandboxie/SandboxDrv.sln` includes SboxDrv plus message projects.
- `apps/start/Start.vcxproj` declares `SbieDll.lib` and `Common.lib`; `apps/ini/SbieIni.vcxproj`, `core/svc/SboxSvc.vcxproj` and COM projects also reference `SbieDll.lib`. These are concrete link edges, not a complete compiled dependency graph.
- Source build instructions and upstream x64 CI both build Win32 `SandboxDll.sln` BEFORE x64 `Sandbox.sln`, THEN x64 `SandboxDrv.sln`. Do not silently omit Win32 components for an x64-only product.
- Upstream project files select MSVC v143 for release user-mode code; driver project uses the WindowsKernelModeDriver10.0 toolset. Windows SDK/WDK, VS2022 C++, MFC and project-specific targets need a Windows build checkpoint. Product's earlier Node/Rust preflight does NOT certify these tools.

### Reproducible upstream baseline (commands for future approved Windows source build; NOT EXECUTED)

Run from an **independent immutable checkout of upstream exact SHA** in the upstream-recommended VS2022 environment:

~~~bat
msbuild /t:build Sandboxie\SandboxDll.sln /p:Configuration="SbieRelease" /p:Platform=Win32 -maxcpucount:8
msbuild /t:build Sandboxie\Sandbox.sln /p:Configuration="SbieRelease" /p:Platform=x64 -maxcpucount:8
msbuild /t:build Sandboxie\SandboxDrv.sln /p:Configuration="SbieRelease" /p:Platform=x64 -maxcpucount:8
~~~

Before any patches record source SHA, subproject list, compiler/WDK/SDK versions, target architectures, output filenames, per-artifact SHA-256, version resources, symbols, binary signatures and warnings. A successful compilation is NOT proof of supported/signed driver installation, isolation, login, rendering, stability, or independent deployment.

## Licensing for personal educational noncommercial use

At the pinned source:
- `Sandboxie/COPYING` and core: GPLv3.
- `SandboxiePlus/QSbieAPI/LICENSE`: LGPLv3.
- `SandboxiePlus/SandMan/LICENSE` and root `LICENSE.Plus`: custom; private noncommercial source modification/build permitted per written conditions; commercial usage and redistribution have separate restrictions. SandMan is deliberately excluded.
- Other modules/assets each need a file-level copyright/license pass **before** actual import/distribution. GPL obligations and original authorship notices remain in force; using an app-owned name for our app does not relicense upstream code.
- A private local build does not waive Windows kernel signing requirements; `NoSecurityIsolation=y`/OpenCOM is a reduced-isolation compatibility setting, not an authorization to bypass Microsoft authentication.

The current record covers source/license inventory only. Not a legal opinion; distribution needs a new dedicated review.

## Existing product integration owners

- Runtime launch, signed Education candidate, launch admission, focus/place: `Development/apps/desktop/src-tauri/src/engine/native_clients.rs`.
- Canonical read-only collector: `Development/tooling/acceptance/windows/collect-sandboxie-feasibility.ps1`.
- Thin Tauri IPC adapter and typed UI: `Development/apps/desktop/src-tauri/src/commands/native_clients.rs`, `Development/apps/desktop/src/app/bridge/nativeFeasibilityApi.ts`.
- Compact Manager presentation: `Development/apps/desktop/src/CompactManagerApp.svelte`.
- Legacy VMware `Development/engine/runtime-core/src/provider/` is PAUSED, not a reusable active isolation provider.

The currently active collector searches signed `Start.exe`/`SbieIni.exe`, examines `SbieSvc`, obtains sandbox process membership by a bounded CLI or trusted `SbieDll` query, and separately authenticates the targeted signed Education process for window actions. A new engine adapter must preserve at least equivalent provenance and fail-closed semantics.

## First fork boundary recommendation

1. Create a **separate upstream-history-preserving source fork** at the pinned tag (working name `M-Isolation-Engine`), not an import of GPL driver code into the Tauri desktop.
2. Preserve the mutually compatible GPL core/service/DLL and required 32/64-bit tools/helpers at first. Minimize by **excluding frontends/packaging**, not by deleting unknown runtime dependencies.
3. Treat official external Sandboxie 1.18.5 as the only LIVE backend until baseline fork artifacts work in an isolated, disposable Windows test environment; do not load two incompatible kernel services on the account-bearing host.
4. Define an application-owned versioned interface at the existing Rust desktop owner. First adapter delegates to official tools, second is for an **independently validated runtime** only. Do not claim merely renaming Start.exe grants independence.
5. Once the second adapter, Windows driver deployment, compatibility, matched performance/stability and data-preserving rollback pass, remove reliance on **an external Sandboxie installation**. Historical source attribution remains.

## Source anchors

- [Pinned upstream root license](https://github.com/sandboxie-plus/Sandboxie/blob/abd4c240031cfb46666daa74ca47ed4e48b561e4/LICENSE.Plus)
- [Pinned GPL core](https://github.com/sandboxie-plus/Sandboxie/blob/abd4c240031cfb46666daa74ca47ed4e48b561e4/Sandboxie/COPYING)
- [Pinned LGPL API license](https://github.com/sandboxie-plus/Sandboxie/blob/abd4c240031cfb46666daa74ca47ed4e48b561e4/SandboxiePlus/QSbieAPI/LICENSE)
- [Pinned AGENTS component guide](https://github.com/sandboxie-plus/Sandboxie/blob/abd4c240031cfb46666daa74ca47ed4e48b561e4/AGENTS.md)
- [Pinned upstream core build guide](https://github.com/sandboxie-plus/Sandboxie/blob/abd4c240031cfb46666daa74ca47ed4e48b561e4/Sandboxie/ReadMe.md)
- [Pinned CI sequence](https://github.com/sandboxie-plus/Sandboxie/blob/abd4c240031cfb46666daa74ca47ed4e48b561e4/.github/workflows/main.yml)
- [First gameplay success and crash report](two-client-first-success-2026-10-14.md)

**Evidence:** exact referenced public GitHub source and current product source review. **NOT_EXECUTED:** compiled fork, dependency trimming, driver load, driver signing, QA crash attribution, Minecraft regression, account migration, release bundle.
