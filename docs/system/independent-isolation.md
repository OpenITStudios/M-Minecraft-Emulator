---
id: document.system.independent-isolation
class: DOCUMENT
domain: system
role: REPORT
authority: WORKING
lifecycle: ACTIVE
---

# Independent Minecraft session isolation

Research date: 21 October 2026. Source baseline:
`79b93ae76c03ff5d9ca128264511d2c145a0cde9` (`Local`). This is a design and
feasibility assessment, not a second readiness registry. Current continuation
belongs to [planning/development.md](../../planning/development.md); dependency
retirement belongs to [legacy-retirement.md](legacy-retirement.md).

**Product target:** four concurrently playable Education clients, each with its
own licensed account and persistent data, presented through one responsive tab
bar. Selecting a tab changes presentation; another session keeps running.
**Independent production provider: NOT_IMPLEMENTED.** No Windows-user experiment,
Education sign-in, four-world play or multiplayer acceptance is claimed here.

## Original engine architecture — operator decision, 23 October 2026

The operator selected **an independently implemented Minecraft-specific engine, using Sandboxie's architectural ideas as reference**. This supersedes a vendor-provided per-instance launch option as a prerequisite: our engine may supply transparent isolation, if a complete supported Windows implementation can be demonstrated. It does not select the retired fork, relax the unchanged-game/no-DLL-injection rule, or establish that an original driver is sufficient. PSEC-only rejection remains valid.

This section records design and responsibility, not runtime readiness. Current work and the next action live only in [planning/development.md](../../planning/development.md).
Source inspected: `e515fd5ab6d390d94f279dbe233414a2b3722126` (`Local`). Historical Sandboxie files below are **read-only reference**, not code dependencies or templates to translate into the new implementation.

### Reference architecture and existing module owners

Sandboxie's reference implementation combines user-mode translation, kernel enforcement and privileged launch assistance. Its [own overview](https://sandboxie-plus.com/sandboxie/howitworks/) describes intercepting and containing changes; the pinned source below identifies the actual division of work.

| Concern | Historical reference inspected | Existing independent owner / missing implementation |
| --- | --- | --- |
| Filesystem view | [DLL file translation](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/e515fd5ab6d390d94f279dbe233414a2b3722126/engine/isolation-owned/Sandboxie/core/dll/file.c) distinguishes true/copy paths and handles file operations; [driver file policy](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/e515fd5ab6d390d94f279dbe233414a2b3722126/engine/isolation-owned/Sandboxie/core/drv/file.c) enforces access. | `independent_engine/overlay.rs` is a bounded in-memory synthetic COW module; `storage.rs` classifies evidence. Neither redirects Windows I/O. A native adapter still needs process-bound private Roaming/Local resolution, enumeration, rename/delete and durable writes. |
| Registry view | [DLL key translation](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/e515fd5ab6d390d94f279dbe233414a2b3722126/engine/isolation-owned/Sandboxie/core/dll/key.c) hooks open/create/query/enumerate operations; [driver key policy](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/e515fd5ab6d390d94f279dbe233414a2b3722126/engine/isolation-owned/Sandboxie/core/drv/key.c) manages registry/hive access. | `independent_engine/registry.rs` reviews synthetic reports; `overlay.rs` models values only. Missing: effective per-game HKCU routing, key metadata/enumeration, notifications, persistent hive semantics and peer denial. |
| Objects and IPC | [DLL IPC translation](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/e515fd5ab6d390d94f279dbe233414a2b3722126/engine/isolation-owned/Sandboxie/core/dll/ipc.c) handles object/port names; [driver IPC policy](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/e515fd5ab6d390d94f279dbe233414a2b3722126/engine/isolation-owned/Sandboxie/core/drv/ipc.c) checks object access. | Existing pair-observation interfaces report evidence only. Missing: a justified namespace/IPC route for the actual game's singleton and helpers. File/registry filtering does not supply this. |
| Process lifecycle | [Driver process tracking](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/e515fd5ab6d390d94f279dbe233414a2b3722126/engine/isolation-owned/Sandboxie/core/drv/process.c) and [service launch assistance](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/e515fd5ab6d390d94f279dbe233414a2b3722126/engine/isolation-owned/Sandboxie/core/svc/ProcessServer.cpp) bind operations to sandbox membership. | Reuse `IndependentProviderPort`, desktop exact-process identity and the single `instance_catalog.rs` admission owner. The disabled provider remains disabled; no second session registry or readiness flag. |
| Authentication | [Credential interception](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/e515fd5ab6d390d94f279dbe233414a2b3722126/engine/isolation-owned/Sandboxie/core/dll/cred.c) contains explicit credential API handling. | These hooks are not imported or reproduced. `native_clients.rs` observes helper candidates only. The game's Microsoft-owned sign-in must work in the correct isolated context; no engine credential store, token/cache copying, broker impersonation or auth bypass. Classic credential hooks do not prove WAM compatibility. |
| Presentation and control | Isolation and launch do not themselves provide a working embedded renderer. | Keep `workspace_surface.rs` and existing workspace presentation as the owner of selection/restore. Selection must not stop, relaunch or change a session's identity. GPU/input/fullscreen and recovery require actual game evidence. |

The provider interface remains small: inspect a slot, launch it when admitted, and request normal close of an exact process. OS mediation belongs behind that interface, not in frontend controls or another generic manager. Domain policy stays in runtime-core; Windows coordination stays in the desktop adapter. The existing `independent-service/src/main.rs` is an inert, unbundled source host, not an active broker or permission to install a service.

### Native mediation candidate to assess

A newly authored **file minifilter plus registry-callback implementation** is a candidate for the missing file/HKCU mediation, not yet an accepted complete provider:

- Microsoft documents [minifilters](https://learn.microsoft.com/en-us/windows-hardware/drivers/ifs/about-file-system-filter-drivers) as a supported kernel mechanism for filtering/modifying file I/O. Use the Filter Manager model for any future implementation; do not copy legacy syscall hooks, patch kernel tables or change host junctions/profile directories to redirect every process.
- Microsoft documents [registry callbacks](https://learn.microsoft.com/en-us/windows-hardware/drivers/kernel/filtering-registry-calls) that can process or redirect operations. Callback availability does not implement a coherent private HKCU: key-object context, relative opens, output semantics, enumeration, reentrancy and cleanup all need design and proof.
- [Process notifications](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddk/nf-ntddk-pssetcreateprocessnotifyroutineex) can track creation/exit. They are not complete provenance for broker-launched helpers. Parent PID or executable name alone must not enroll a process.
- [Object callbacks](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-obregistercallbacks) cover process/thread/desktop handle operations, not a general mutex-name or ALPC remapping facility. [Windows namespaces](https://learn.microsoft.com/en-us/windows/win32/termserv/kernel-object-namespaces) include global and per-session objects. A file/registry driver therefore cannot be presented as a complete singleton/IPC/auth solution.
- No driver skeleton, new wire schema, privileged installer, service activation, signing-mode change or game injection is added by this design. PSEC may be reused later for a demonstrated subproblem, never as an automatic fallback or launch permit.

### Required behavior of the future adapter

A native adapter must bind trusted process membership to an exact live process object/lifetime and stable slot profile before game code accesses protected state. Children need verified membership; externally activated helpers remain outside until a supported attribution/context route exists. PID reuse or manager reconnect must never transfer another process into a slot.

For an admitted session, logical game paths and HKCU must have one coherent private effective view, including environment/Known Folder lookups and native operations. Private game writes persist across normal close/relaunch. Host account/cache/world data and peer overlays are not a read-through COW base; shared base reads are limited to explicitly allowed non-account installation/system resources. No implicit migration of old profiles is permitted.

A real file adapter must handle Windows Unicode/case behavior, relative handles, links/reparse escape, alternate streams, mapped I/O, enumeration, rename/delete and concurrent opens; the ASCII/value-only fixture is not its implementation. Failure to establish mediation blocks new launches; it must not fall back to host writes or kill unrelated clients. Manager failure must preserve recoverable session/data ownership; teardown must wait for active operations and owned processes before releasing mediation.

Isolation protects clients from unintended host/peer access. It does not promise secrecy from the Windows administrator or owning user. Acceptance must distinguish the contained game's access to host/peer data from the host administrator's authority.

### Acceptance implications

Source review can establish a concrete supported mechanism and its limits; it cannot certify runtime mediation. Before MC02 launch is exposed, require two official games with separate manual licensed sign-ins, independently persistent worlds/HKCU/auth behavior, no host/peer crossover, concurrent GPU/input, and independent close/relaunch. Then extend the same provider to four clients, multiplayer/recovery, and measured Sandboxie comparison. Synthetic I/O/registry controls may support native implementation checks, but cannot advance game acceptance.

**Evidence for this design:** source/reference and official Windows contract review only. Original minifilter/registry/IPC/auth adapter implementation, WDK compilation, driver signing/install, Minecraft launches and two/four-client acceptance are **NOT_EXECUTED / NOT_IMPLEMENTED**, as applicable.

## Microsoft PSEC/MXC native isolation candidate — 23 October 2026

Microsoft introduced generally available MXC in October 2026, with native
Process Security Environment (PSEC) capabilities on supported Windows builds.
This is a *candidate native enforcement foundation* potentially avoiding
a third-party kernel driver or full VM; it is NOT evidence of lower RAM/CPU
overhead than Sandboxie. The current supported processmodel.h PSEC API replaces
the older experimental SBOX-only CreateProcessInSandbox approach in MXC.
Refs: https://learn.microsoft.com/en-us/windows/win32/api/_processmodel/
and https://github.com/microsoft/mxc/blob/main/docs/process-container/os-version-support.md
and https://blogs.windows.com/windowsdeveloper/2026/10/07/microsoft-execution-containers-policy-driven-containment-for-ai-agents/

The existing Windows identity owner exposes an explicit, read-only capability
query for PSEC major-version support and reported OS policy flags. It loads
processmodel.dll only from System32 and does not allocate a security
environment, create a process or touch game/account data. **This probe cannot
approve MC02.** PSEC grants/denies file *access* but does not automatically
redirect the real Desktop game Roaming world directory, provide per-process
HKCU overlays, prove Microsoft ADAL/WAM isolation or solve the game's
multi-instance/GUI behavior. The prior unpackaged AppContainer Roaming NO-GO
still applies until a genuine game-aware mediation provider is established.
Do not add automatic PSEC fallback to weaker AppContainer/DACL policies.

## P0 feasibility decision — documented Minecraft data roots, 23 October 2026

**Engineering result: no supported, non-modifying four-client provider has
been demonstrated.** The independent native Windows source remains
diagnostic/fixture-only and MC02–MC04 Launch/Provision stays blocked. This
conclusion is a source and official-document review, NOT a claim that a
general solution is impossible, a passed Windows test or a new readiness
state. The previous Windows synthetic AppContainer fixture results are
revision-scoped and must not be promoted to unmodified Minecraft evidence.

### Official Minecraft Education edition-specific storage

Minecraft Education's published Windows clean-reinstall guide documents:

- Traditional **Desktop .exe** world data under
  `%APPDATA%\\Minecraft Education Edition\\games\\com.mojang\\minecraftWorlds`
  (Roaming AppData of the logged-on Windows user).
- **Microsoft Store appx** world data under
  `%LOCALAPPDATA%\\Packages\\Microsoft.MinecraftEducationEdition_8wekyb3d8bbwe\\LocalState\\games\\com.mojang\\minecraftWorlds`
  (the published Store package/user LocalState path).

Source: [Minecraft Education — Clean Reinstall on PC](https://edusupport.minecraft.net/hc/en-us/articles/360047118832-Clean-Reinstall-Minecraft-Education-on-PC).

These are official **edition-specific reference locations**, not
installer-independent runtime guarantees or an authorization to inspect,
copy, rewrite or delete user worlds. The Store and Desktop editions cannot
coexist according to the [official installation guide](https://edusupport.minecraft.net/hc/en-us/articles/13106858087956-Windows-Installation-Guide).
Never embed either path as a production routing shortcut.

The previously executed **unpackaged AppContainer synthetic fixture** reported
`APPDATA` and `SHGetKnownFolderPath(FOLDERID_RoamingAppData)` outside its
expected private container, despite private LocalAppData and synthetic
canaries. This specifically blocks that observed candidate as an assumed
replacement for the official Desktop world's Roaming data route. It does
NOT establish where the unmodified Education process actually wrote, how
its Microsoft login broker behaved or that *all* AppContainer configurations
are impossible. The existing evaluator already returns
`NO_GO_OBSERVED_ROAMING_BOUNDARY`; add no duplicate policy gate.

### Windows mechanism decision by necessary capability

| Candidate | Windows-supported mechanism | Product-specific gap / decision |
| --- | --- | --- |
| Unpackaged AppContainer | OS SID/DACL sandbox, scoped local data and named objects; actual two-process synthetic fixture exists | **NO-GO for the observed Desktop/Roaming candidate**: in-process known-folder/environment remained outside private profile. HKCU, Store behavior, Minecraft actual world access and auth remain UNKNOWN. |
| Full MSIX / official Store package | Documented virtualized-app AppData and HKCU behavior is **per user + per app/package**, with merged host reads under documented conditions | **Not a per-PID solution** for four sessions from one game package identity and one Windows user. Identical Package Family Name observations cannot prove per-client world/credential separation. Do not repackage or install a second edition. |
| Package identity with external location | Supported package identity for an existing **developer-controlled** application | **Not selected** for the unmodified third-party official game. A new identity package neither confers rights to modify/re-register the official distribution nor grants per-instance HKCU/WAM isolation. |
| Win32 app isolation (Windows 11 24H2+) | Microsoft-documented **preview**, includes AppContainer, file/privacy brokering and some app multi-instancing | **Research only**: requires packaging/developer integration and compatible supported OS. ShellExecute multi-instancing does not establish independent private worlds, HKCU or Microsoft broker contexts for the unmodified official Education app. |
| ProjFS | User-mode projected files under one explicit **virtualization root** | **Not a general game filesystem redirector**. A provider for a chosen directory does not by itself redirect the game's pre-existing Roaming path or HKCU. |
| RegLoadAppKeyW / RegOverridePredefKey | Handle-only app hive; latter overrides predefined keys **only for the caller process** | **Not transparent HKCU virtualization** for an external unmodified Minecraft process. Existing private hive fixture is useful but cannot authorize a game launch. |
| Windows UAC legacy registry virtualization | Redirects selected legacy HKLM writes into user VirtualStore | **Not a per-instance HKCU solution**; excluded for 64-bit processes and manifest-specified requested execution levels. Not applicable as a general independent game registry implementation. |
| RunAs alternate Windows user | Distinct Windows user profile and HKCU | **Not selected**: separate logon alone does not prove game singleton/display compatibility; Microsoft WAM API documentation explicitly rejects token acquisition under RunAs impersonation. Do NOT infer the actual Education client always uses WAM. |
| File minifilter + registry callbacks | Windows kernel APIs can observe/filter I/O and registry operations | **Not selected**: requires an original signed kernel implementation, robust identity/race/recovery design, and still does not solve external authentication brokers, UI or singleton behavior automatically. No driver code/install authorization. |

Windows primary-source references:

- [Packaged desktop virtualized AppData/HKCU](https://learn.microsoft.com/en-us/windows/msix/desktop/desktop-to-uwp-behind-the-scenes) — per-user/per-app writes and host-read fallback.
- [Win32 app isolation preview and minimum platform](https://learn.microsoft.com/en-us/windows/win32/secauthz/app-isolation-overview) and [release notes](https://learn.microsoft.com/en-us/windows/win32/secauthz/app-isolation-release-notes).
- [Package identity with external location](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/grant-identity-to-nonpackaged-apps-overview).
- [ProjFS virtualization root](https://learn.microsoft.com/en-us/windows/win32/projfs/provider-overview).
- [RegOverridePredefKey caller-only behavior](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regoverridepredefkey) and [application hive handles](https://learn.microsoft.com/en-us/windows-hardware/drivers/kernel/filtering-registry-operations-on-application-hives).
- [Legacy Windows registry virtualization limits](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-virtualization).
- [MSAL WAM active interactive session / RunAs limitation](https://learn.microsoft.com/en-us/entra/msal/dotnet/acquiring-tokens/desktop-mobile/wam).
- [Registry filter driver scope](https://learn.microsoft.com/en-us/windows-hardware/drivers/kernel/filtering-registry-calls).

### Product-evidence boundary and stop decision

For **any** future production candidate, require evidence that the unchanged
legitimately installed Education client — not a synthetic fixture — can
safely resolve private Local **and Roaming** world data, private effective
HKCU reads/writes and cross-session access denial while preserving normal
Microsoft-owned sign-in and singleton/window/input behavior. The user must
manually authenticate legitimate distinct accounts inside the game; collect
no account IDs, tokens, secrets or world files. Repeat across restart and
pair-wise clients before four-client acceptance. A native process count,
valid Authenticode signature, package family, token SID, private-looking
folder, created fixture canary or installed service cannot replace any of
these observations.

**STOP:** do not begin a production minifilter, package registration,
service activation, new login implementation, host-profile file rewrite,
or independent client launch on the strength of this source review.
The existing platform, provider and retired-fork admission gates remain
authoritative; no new lifecycle gate/readiness store is introduced.
Useful source work may continue independently of Windows runtime access,
but actual mechanism compatibility stays `NOT_EXECUTED` until observed.

**Next source-only task:** inspect the existing independent storage and
process evidence consumers for any implied promotion of *path/package*
metadata into game-private access; correct only a concrete leak if found.

## Bahan diskusi ChatGPT - 21 Oktober 2026

Operator ingin mendiskusikan keputusan berikutnya sebelum pekerjaan dilanjutkan.
Status jeda dan otoritas melanjutkan hanya dimiliki
[planning/development.md](../../planning/development.md). Bagian ini merangkum
konteks untuk diskusi, bukan persetujuan menjalankan eksperimen berikutnya.
Mulai dengan membaca `AGENTS.md`, `GITHUB_RULES.md`, implementation map dan HEAD
`Local` terbaru; jangan mengambil `Next` historis sebagai izin baru.

### Tujuan dan keputusan yang sudah disetujui

- Empat Minecraft Education berjalan bersamaan pada satu PC, dengan empat akun
  berlisensi berbeda dan data persisten masing-masing. Operator menyatakan empat
  akun tersedia untuk login manual; login ditunda sampai fondasi/UX layak diuji.
- Perilaku tab seperti browser: memilih tab hanya mengganti tampilan/kontrol;
  semua proses lain tetap hidup. Menutup game merupakan tindakan terpisah.
- Pensiunkan Sandboxie **dan fork turunannya**, bukan sekadar mengganti nama.
  Kemiripan ide arsitektur diperbolehkan; implementasi pengganti harus mandiri.
  Tidak kembali ke pembelian/aktivasi sertifikat fitur Sandboxie sebagai solusi.
- Gunakan satu pemilik lifecycle/admission; jangan menambah penyimpanan kredensial,
  implementasi login Microsoft, flag ready atau manager paralel. Login tetap di
  game. Tidak menyalin token, menginjeksi DLL atau memodifikasi binary Minecraft.
- Branch produk hanya `Local`; `main` dan rilis tidak disentuh. GitHub Actions
  tetap sepenuhnya dimatikan. VMware/guest legacy tidak dihidupkan kembali.

### Masalah semula dan koreksi klaim

Operator melaporkan area game kosong, kontrol duplikat/tidak jelas, tab lambat,
MC01 hilang ketika MC02 dibuka, kegagalan kembali dari maximize/fullscreen,
MC02–04 tidak operasional, serta akun yang belum benar-benar terpisah. Klaim
empat sesi stabil/terisolasi **tidak pernah memperoleh acceptance** dan ditolak
operator. Banyak proses atau status Running bukan bukti empat akun playable.

Pada engine lama, sumber fork menunjukkan App Compartment menjadwalkan terminasi
sekitar lima menit ketika fitur terkait tidak aktif. Query fitur aktual tidak
aktif dan satu exit code cocok dengan jalur terminasi; waktu/caller terminasi
setiap kegagalan lama tidak tertangkap. Ini bukan diagnosis tunggal untuk semua
blank window, fullscreen atau crash. Detail dan batas bukti ada di
[native pilot report](../validation/native-pilot-localtest.md).
Perbaikan swap/recovery lama dipertahankan, tetapi bukan bukti engine pengganti
atau jaminan fullscreen, DPI lintas monitor, input dan gameplay sudah stabil.

### Titik sumber dan bukti yang bisa dirujuk

| Commit `Local` | Hasil terbatas yang benar-benar dicatat |
| --- | --- |
| `79b93ae76c03ff5d9ca128264511d2c145a0cde9` | Admission menolak Launch/Provision melalui Sandboxie resmi dan fork; recovery/close sesi lama dipertahankan. Desktop verification 185 frontend + 78 Rust; 8 opt-in ignored; receipt unchanged regression lulus. Full root suite berhenti pada assembler `EPERM`; bukan PASS keseluruhan. |
| `cea5c365753efddafbba8ef8ccabf2b8f40b70fa` | Inspector konteks proses Windows tanpa Sandboxie; handle dan creation identity diverifikasi, hanya tiga boolean keluar melalui typed IPC/diagnostics. Desktop verification 188 frontend + 80 Rust, 8 ignored; dua regresi Windows nyata memakai proses tes/child sendiri. Bukan isolator atau launcher game. |
| `d823f68456e64298aedea38d879daabe3be4aa55` | Prototipe native AppContainer hanya untuk fixture sintetis; MSVC `/W4 /WX`, kontrol positif dan negatif nyata, serta pemeriksaan struktur/kontrak repo lulus. Desktop/root full suite tidak dijalankan ulang pada batch tooling ini. Bukan penerimaan Minecraft. |

Temuan kandidat terakhir: dua token AppContainer berbeda, proses bersamaan,
mutex bernama sama dalam namespace terpisah, canary sendiri terbaca, canary peer
serta host ditolak, dan penghentian normal satu fixture tidak menghentikan yang
lain. Dua profil sintetis dibersihkan. Kontrol negatif sengaja menyamakan token
kedua dan ditolak pada `actual-distinct-container-tokens`; FAIL tersebut adalah
hasil kontrol yang diharapkan, bukan acceptance produk yang gagal/disembunyikan.

`LOCALAPPDATA` environment dan known-folder beralih ke container pada host yang
diuji. `%APPDATA%` roaming dan known-folder roaming **tidak** berada di container.
Hasil token SID berbeda atau nama folder berbeda tidak membuktikan data game,
cache MSAL, broker WAM atau akun Microsoft terisolasi.

### Hal yang belum boleh diasumsikan

- AppContainer saat ini **kandidat sintetis**, bukan provider Minecraft produksi.
  Tidak ada peluncuran game, login, empat dunia, multiplayer, uji sesi panjang,
  rendering/input/fullscreen atau package acceptance untuk penggantinya.
- Changelog resmi membuktikan migrasi historis ADAL ke MSAL; belum membuktikan
  broker yang dipakai build terpasang. WAM mempunyai batas RunAs yang terdokumentasi;
  belum ada alasan menyatakan Minecraft selalu memakai WAM atau selalu bebas WAM.
- Pembatas proses kedua Minecraft belum teridentifikasi. Namespace mutex sintetis
  terpisah tidak membuktikan singleton game memakai mekanisme yang sama.
- Driver/service Sandboxie lama masih terpasang; pengujian pada Windows bersih
  tanpa driver itu belum dilakukan. Source fixture tidak memanggil API Sandboxie.
- Executable desktop yang sebelumnya terbuka tidak diganti oleh batch-batch ini;
  binary/UI terpasang tidak boleh dianggap sama dengan HEAD sumber terbaru.
- Profil akun/dunia lama tidak dimigrasikan atau dihapus. Profil yang kelak pernah
  menjalankan game harus dianggap berpotensi membawa akun/data dan **tidak boleh**
  mengikuti kebijakan auto-delete milik fixture sintetis.

### Pertanyaan untuk dibahas sebelum memilih langkah berikutnya

1. Apakah AppContainer layak diteruskan mengingat jalur roaming belum terarah?
   Bukti minimum apa yang membedakan data privat nyata dari perubahan environment
   yang tidak diikuti game/known-folder API?
2. Bagaimana memeriksa kompatibilitas MSAL/broker dan singleton game tanpa membaca
   cache akun, memodifikasi binary, menginjeksi kode atau membuka akses data host?
3. Apa kriteria NO-GO kandidat ini, sehingga kita tidak menumpuk launcher, custom
   driver atau shim sebelum ada mekanisme yang cocok dan terbukti?
4. Bila kandidat lolos, bagaimana urutan dua game hidup bersamaan, recovery/tab,
   dua login manual, lalu empat akun dan packaging/migrasi yang reversibel?

Pemilik sumber: lifecycle/policy `engine/runtime-core/`; Windows identity dan
presentation `apps/desktop/`; fixture pembuktian `tooling/acceptance/windows/`;
[arsitektur](architecture.md) untuk tanggung jawab sistem dan
[retirement register](legacy-retirement.md) untuk batas pensiun/data.
Log, hasil kompilasi, JSON runtime dan receipt mentah tetap di ignored `workspace/`;
repo menyimpan sumber, ringkasan bukti tersanitasi dan hash commit, bukan token,
akun, dunia, raw machine identity, signing material atau dump privat.

## Implementation direction

### Read-only OS Package Family comparison — 23 October 2026

The existing Windows process identity owner now has an **unmounted**
`inspect_process_package_family_pair_source` diagnostic for two DISTINCT,
exactly pinned PID/full creation-FILETIME processes. It uses Windows
`GetPackageFamilyName` on both retained, query-only process HANDLEs,
with the existing bounded `GetPackageFullName` status classifier.
An actual `APPMODEL_ERROR_NO_PACKAGE` is distinct from a query error.
Only four sanitized outcomes exist: both unpackaged, one packaged,
same package family or different package families. The UTF-16 OS package
family strings are discarded; no raw names, paths, package publishers,
account material or authentication data are exported. Tests authored
for invalid/stale process keys, both/mixed/same/different families and
a Windows-only self-process query consistency fixture; NOT_EXECUTED.

**Scope:** identical package family is a package identity relationship,
NOT proof of the same Windows user, private worlds, registry/HKCU data,
Microsoft WAM or a supported Education multi-instance launch. MSIX
state is scoped by package and user with merged reads of certain host
AppData/registry surfaces, rather than automatically isolated by PID.
Different package families do not attest that either is a legitimate
Minecraft Education build, that multiple variants are installable or
that independent licenses can authenticate simultaneously. No attempt
is made to repackage the official game or launch a second process.
The repo still blocks MC02–MC04 Launch/Provision.

Public OS reference:
[GetPackageFamilyName](https://learn.microsoft.com/en-us/windows/win32/api/appmodel/nf-appmodel-getpackagefamilyname)
and [MSIX flexible virtualization](https://learn.microsoft.com/en-us/windows/msix/desktop/flexible-virtualization).
**Next P0 evidence:** determine whether any supported, non-modifying
Windows mechanism truly provides per-game-process Roaming/HKCU/broker
separation before committing to a production provider or driver.

### Supported Windows mechanisms against the actual game requirements — 22 October 2026

Product target: four licensed, simultaneously playable official Education
processes on one interactive Windows desktop, preserving installation,
worlds, user login and the existing tab UX. No binary modification,
injected hooks, unreviewed service/driver or credential impersonation.

| Mechanism | Supported or observed Windows capability | Product-critical limitation | Decision |
| --- | --- | --- | --- |
| Unpackaged AppContainer | Earlier synthetic pilot showed distinct process/token namespaces and private LocalAppData | Default Roaming known folder and APPDATA were observed outside the private profile. HKCU, broker and real Education behavior are unverified | **NO-GO for observed candidate** |
| RegLoadAppKeyW | Fresh explicitly opened per-application registry hives | Keys are only reachable via the returned handle; they do NOT transparently replace normal HKCU used by an unmodified game | **Not a standalone HKCU solution** |
| RegOverridePredefKey | Can remap predefined registry keys for its **calling process** | An external manager's call does not affect a separate game process. Requiring a call within unmodified Minecraft conflicts with the current no-binary-changes requirement | **Not a viable external route** |
| Distinct Windows user with LOGON_WITH_PROFILE | Loads a different user's registry profile and user-context files | Microsoft WAM expects an active interactive user session and does not support arbitrary RunAs impersonation for token acquisition. Desktop/window and game singleton behavior also unresolved | **Not accepted for current single-desktop UX** |
| Official Store / MSIX package | Some virtualized packaged desktop app types redirect AppData writes and copy-on-write registry changes | Does not imply four private package identities, private reads, four WAM contexts or Minecraft multi-instance compatibility. Desktop and Store Education installers are mutually incompatible | **Investigation candidate only** |
| VM or new filesystem driver/broker | Potentially stronger OS isolation | VM overhead conflicts with target; an independent filter/driver cannot itself solve external Microsoft broker or game singleton compatibility, and needs signing/recovery proof | **Not selected** |

Official reference contracts:

- [Microsoft: AppContainer for legacy applications](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-for-legacy-applications-)
- [RegLoadAppKeyW — handle-only registry hive](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regloadappkeyw)
- [RegOverridePredefKey — calling-process scope](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regoverridepredefkey)
- [CreateProcessWithLogonW — interactive profile and desktop rights](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createprocesswithlogonw)
- [MSAL WAM requires active interactive Windows session](https://learn.microsoft.com/en-us/entra/msal/dotnet/acquiring-tokens/desktop-mobile/wam)
- [MSIX packaged desktop virtualized app behavior](https://learn.microsoft.com/en-us/windows/msix/desktop/desktop-to-uwp-behind-the-scenes)
- [Minecraft Education Windows installation guide](https://edusupport.minecraft.net/hc/en-us/articles/13106858087956-Windows-Installation-Guide)

Packaged identity alone cannot identify the manifest runtime behavior or
prove that this Education process receives filesystem and HKCU redirection.
MSIX private AppData writes do not prove isolation of existing host reads.
A synthetic independent registry hive does not substitute the game's
ordinary HKCU access. WAM compatibility must be demonstrated using normal,
game-owned sign-in; do not disable WAM, copy tokens or repackage the game.

Next engineering decision: keep MC02–04 Launch/Provision denied.
The only meaningful future provider trial must independently establish
Roaming/AppData **actual access**, HKCU effective isolation, singleton
behavior and distinct real login/broker contexts for the unchanged
official image, before any readiness or launch claim.


### Explicit Roaming Known Folder candidate NO-GO (22 October 2026)

The existing synthetic AppContainer evaluator now derives
`NO_GO_OBSERVED_ROAMING_BOUNDARY` when the *in-process*
`SHGetKnownFolderPath(FOLDERID_RoamingAppData)` fixture explicitly
reports a path outside its own profile. Successful independent synthetic
canaries and locally private directories cannot override this result.
An unobserved Known Folder or a path that merely looks private stays
`UNVERIFIED_NO_GAME_ACCEPTANCE`. Registry/HKCU, authentication broker
and unmodified Education game file access remain NOT_OBSERVED/NOT_EXECUTED.
This is an offline ephemeral candidate assessment, NOT a second lifecycle
state, proof that all AppContainer use is impossible, or permission to
enable a fallback.

Microsoft `RegLoadAppKeyW` application hives require the returned
root handle and are not mounted into normal HKCU. Do not mistake their
isolated fixture markers for transparent unmodified Minecraft registry
redirection. Likewise, MSIX package file/registry virtualization rules
cannot simply be ascribed to the existing unpackaged AppContainer process.
See the single source owner at
`tooling/acceptance/evaluate-appcontainer-isolation.mjs`.


Build the smallest independent, Minecraft-specific Windows session engine. Do not
copy Sandboxie source or make its driver, DLL, service, box settings, feature
certificate or scripts part of the replacement. Shared architectural ideas such
as identity-bound lifecycle operations and persistent per-instance data are
compatible with this direction. A rebranded fork is not the replacement.

Keep lifecycle/admission policy in `engine/runtime-core/`; keep Windows-native
calls and interactive windows in `apps/desktop/`. The existing workspace consumes
verified process identities. It must not become a second lifecycle manager, infer
ownership from a title, treat a PID as an account, or keep a separate ready flag.
Preserve PID/start-time/HWND checks and restore-on-manager-exit behavior. Existing
Sandboxie launch denial stays in place until an independent route is proven.

An opt-in read-only Windows identity comparison in the existing desktop
`instance_identity.rs` owner is the first implementation foundation. Retain the
process handle while checking token/profile/session identity, return only
sanitized comparison booleans, and do not serialize raw SIDs or profile paths.
This does not authorize adoption or a launch. Configured profile-directory
comparison does not inspect the game's actual file view or detect virtualization,
folder aliases, broker state or Microsoft accounts. Same Windows user does not
by itself prove absent isolation; different users do not prove successful isolation. **An alternate-user process is not
yet a viable production authentication design.** Do not
start a general-purpose kernel sandbox or promise a custom driver fixes the
constraints below before demonstrating that a driver is necessary and sufficient.

## What Windows actually provides

| Mechanism | Supported behavior and limit |
| --- | --- |
| Alternate-user process creation | `CreateProcessWithLogonW` creates a process under supplied Windows credentials. `LOGON_WITH_PROFILE` loads that user's registry profile; `LOGON_NETCREDENTIALS_ONLY` retains the caller's local token and cannot provide local account separation. A null environment is generated for the specified user. The caller must arrange permissions on the inherited window station and desktop. Automatic desktop permission is documented only for Windows XP. A successful call precedes application initialization, so it is not proof the game stayed alive. [Microsoft API contract](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createprocesswithlogonw). |
| Singleton objects | Windows named kernel objects have global or per-session namespaces. A different user in the same session does not automatically get a different session namespace. **Inference:** alternate-user launch alone cannot guarantee independent singleton behavior. The installed Education executable's actual singleton mechanism is UNKNOWN. Do not rename or suppress its objects. [Microsoft namespaces](https://learn.microsoft.com/en-us/windows/win32/termserv/kernel-object-namespaces). |
| Desktop access | Desktop access is checked against its security descriptor; desktop rights include creating, reading and enumerating windows. Giving another user access to the shared desktop is a deliberate permissions change, not private rendering isolation. Use narrowly scoped rights and a reviewed recovery strategy; do not grant broad Everyone access. [Microsoft desktop security](https://learn.microsoft.com/en-us/windows/win32/winstation/desktop-security-and-access-rights). |
| Authentication broker | WAM integrates with accounts known to Windows and manages authentication/token maintenance. Microsoft explicitly requires an active interactive user session and says WAM token acquisition through `runas` impersonation fails by design. **Consequently, a RunAs launcher is not an acceptable general WAM isolation solution.** This does not establish whether this Education build uses WAM for every sign-in path. [Microsoft WAM integration requirements](https://learn.microsoft.com/en-us/entra/msal/dotnet/acquiring-tokens/desktop-mobile/wam#integration-best-practices). |
| Window messaging/input | Messages are subject to UIPI and may target only equal or lower integrity processes; synthetic input has the same integrity restriction. Cross-user permissions and integrity must be tested separately. Do not request UIAccess, disable UIPI or elevate the whole UI as an assumed cure. Prefer ordinary user input into the selected window. [SendMessage](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendmessage), [SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput). |
| Embedding/focus | Cross-process `SetParent` can reset the child's DPI awareness. Foreground activation has OS restrictions and can be denied. Therefore successful window enumeration or reparenting does not establish usable fullscreen, focus or rendering. Keep presentation reversible and test the actual renderer. [SetParent](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setparent), [SetForegroundWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setforegroundwindow). |

Different `%APPDATA%` values, directories or JSON profiles do not establish any of
the token, namespace or authentication properties above. That is an inference from
the documented mechanisms, not a measured Education failure.

## Education-specific constraints

The official installation guide treats Desktop and Store as alternative delivery
methods and says they cannot coexist. A second installation method is therefore
not a supported way to obtain independent sessions. Windows Server is listed as
unsupported, so moving to an RDS server is not an accepted fallback.
[Education Windows Installation Guide](https://edusupport.minecraft.net/hc/en-us/articles/13106858087956-Windows-Installation-Guide).

The official sign-in guide describes signing in inside the game, remembering the
last account and switching accounts. It does not document four simultaneous
Windows sessions, a supported per-instance profile command line, or the Windows
broker used by the installed build. Keep those facts UNKNOWN; different Windows
SIDs are not proof of four separate Education licenses or accounts.
[Education sign-in guide](https://edusupport.minecraft.net/hc/en-us/articles/360061382672-Sign-Into-Minecraft-Education).

Our manager must not implement Microsoft sign-in, copy credentials/token caches,
modify game binaries or install both editions as a workaround. Preserve existing
account-bearing profiles and worlds during all experiments and migration.

## Native AppContainer candidate - 21 October 2026

A supported alternative to the rejected simple RunAs assumption is a native,
unpackaged AppContainer. Windows documents direct profile creation and launching
via `SECURITY_CAPABILITIES` / `STARTUPINFOEX` / `CreateProcess`, without repacking
the official game or installing our own driver. This is a selected **synthetic
feasibility candidate**, not an accepted Education launch provider.
[Unpackaged apps](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-for-legacy-applications-),
[launch contract](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer).

Each AppContainer has a private named-object path. Windows also documents
per-user/per-application credential identity; this supplies an investigation
basis, not proof that MSAL/WAM will function or maintain independent accounts.
[Named objects](https://learn.microsoft.com/en-us/windows/win32/api/securityappcontainer/nf-securityappcontainer-getappcontainernamedobjectpath),
[isolation model](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation).

The launch guide explicitly documents redirected `LOCALAPPDATA`, `TEMP` and
`TMP`. Do not generalize that to every roaming path, absolute path, external
broker, or package identity. An unpackaged AppContainer identity is not itself
an MSIX package registration. Our fixture checks environment and known-folder
paths separately; it does not inspect credentials or claim a usable private
Microsoft broker.

### Executed native fixture evidence

Source base: `cea5c365753efddafbba8ef8ccabf2b8f40b70fa`. The independent source
owner for this proof is `tooling/acceptance/windows/appcontainer-probe.cpp`, with
explicit runner `test-appcontainer-isolation.ps1`. The runner compiled the real
native code using installed x64 MSVC with warnings treated as errors and ran
both the positive proof and a deliberately broken identity control.

| Actual observation | Result / limit |
| --- | --- |
| Two native processes concurrently running with distinct actual AppContainer token SIDs | PASS; same underlying interactive Windows user, not two Minecraft accounts |
| Both create the same fixed synthetic mutex name while alive | PASS; demonstrates AppContainer namespace separation, not the game's unknown singleton mechanism |
| Each reads its own synthetic profile canary | PASS |
| Access to peer canary and host fixture canary | Denied as expected; no real account/profile contents opened |
| Normal stop of A while B remains alive, then normal stop of B | PASS; no rendering/input or manager-crash proof |
| Local AppData environment and known-folder paths | Both redirected under the respective container on this host |
| Roaming AppData environment and known-folder paths | Both remained outside the container; known-folder query succeeded. This is a material compatibility constraint, not a silent PASS for game data isolation |
| Cleanup | Both uniquely created synthetic profiles removed after process handles closed; no existing profiles/services removed |
| Negative control: deliberately give process B token A | Expected FAIL at `actual-distinct-container-tokens`; synthetic profiles removed. Detecting this is required for control success |

The final source was exercised with:
`./tooling/acceptance/windows/test-appcontainer-isolation.ps1 -VerifyNegativeControl`.
This is a Windows mechanism proof, not a production provider or four-client
acceptance. Only fixture copies were launched. No network capabilities, UI/input
operations, Microsoft sign-in, credential API calls, game modifications, DLL
injection, existing Windows user creation or service/driver installation occurred.
The historical driver remains installed on the host; retest without it on a clean
host remains NOT_EXECUTED. The code calls no Sandboxie APIs.

Profile names are fresh and collisions fail closed. Cleanup uses only profiles
successfully created in this invocation; no wildcard deletion. An ignored,
inspection-only moniker receipt is written before creation to aid review after
interruption. Abrupt termination can still leave synthetic profiles and must not
be reported as successful cleanup. Source and no account material are committed.

### Synthetic private roaming persistence source — 21 October 2026

A bounded extension to `tooling/acceptance/windows/appcontainer-probe.cpp`
now tests real file creation and isolation in two temporary profile-owned
`synthetic-roaming` directories and verifies canaries survive the normal exit
and relaunch of **both synthetic child processes**. The optional PowerShell
runner checks evidence fields and that Minecraft acceptance stays
`NOT_EXECUTED`. This is a **source change**, not a new measured PASS:
the updated MSVC fixture has not yet been executed on Windows.

This does **not** fix the earlier `APPDATA` or Known Folder roaming routing.
No host-roaming path is opened, no Windows profile is granted new permissions,
and no Minecraft/account data is inspected. If known-folder roaming still
points outside the container, the AppContainer candidate remains blocked for
unmodified Education regardless of successful synthetic private-directory tests.

### Strict offline AppContainer report review — 21 October 2026

`tooling/acceptance/evaluate-appcontainer-isolation.mjs` provides a pure
offline evaluation of the opt-in synthetic JSON report. A positive fixture
requires exact schema, all expected boolean fields, synthetic proof flags
and the explicit `NOT_EXECUTED` Minecraft fields. The reviewer can report
whether roaming environment and known-folder paths were observed inside
the synthetic profile, but it never claims file-access virtualization,
Microsoft authentication, game compatibility or new-session admission.
Even all-private path observations cannot authorize production launch.
Auth/known-folder effects and Windows fixture execution remain unknown
until tested. The result is not a durable state registry.

### Education-specific facts gathered in this batch

The official v1.19.53 changelog records migration from ADAL to MSAL. This is a
historical library fact, not proof that current Windows sign-in requires WAM.
[Wild Update changelog](https://edusupport.minecraft.net/hc/en-us/articles/13961617817236-What-s-New-in-The-Wild-Update-v1-19-53).
Current troubleshooting names `Minecraft.AdalServer.exe` during login; that
filename alone also cannot identify the current broker or cache isolation.
[Sign-in troubleshooting](https://edusupport.minecraft.net/hc/en-us/articles/4404976926356-Troubleshoot-the-Can-t-Connect-to-the-Service-Error).

Official Education deep links open the menu if the game is not already open;
they document content/join operations, not a separate-account profile launch
contract. No official supported multi-instance/profile runtime switch was found
in this bounded search; absence from search is not proof none exists.
[Deep-link contract](https://learn.microsoft.com/en-us/minecraft/creator/reference/content/deep-links?view=minecraft-bedrock-stable#education-edition).

Read-only installed-image metadata: registered Desktop `1.26.3200.0`, executable
version `1.26.32.0`, Valid Microsoft Corporation Authenticode signature, SHA-256
`04164c1c64458a5d984dbc1687d0e8023d1808c44ee7be03519cdf8355233eaa`.
No Minecraft process was running at the initial inspection. Import-table metadata
showed CEF/WinRT/network/security dependencies, but did not establish the runtime
authentication path, singleton object or command line. No game code was patched,
executed, disassembled or injected; no protected code was unpacked. The binary
and install path were not committed, and no account files were inspected.

**Next bounded proof:** determine whether an unmodified official Education build
can use a private persistent roaming-data location inside a native AppContainer
without granting host roaming access or injecting compatibility hooks. The
observed roaming paths must be treated explicitly; changing an environment
variable alone is not proof the game uses it. Any future game smoke-test profile
must be preserved as potentially account-bearing and must not reuse this
synthetic auto-delete policy. Auth compatibility, official image attribution,
window recovery, network capabilities, code-builder/COM behavior and sustained
multi-client acceptance remain unproven. Do not enable production launch yet.

## First experiment and acceptance order

These are investigation steps, not additional product progress gates. Existing
G0–G4 and four-client acceptance retain their authority.

1. **Read-only identity foundation:** inspect only an explicitly selected PID with
   an expected creation identity. Retain its handle, query token/profile/session,
   compare identities in memory and return sanitized booleans. Same-user fixture
   tests can verify comparison/error behavior without another account. Do not
   create users, request passwords, edit ACLs, start another process identity or
   launch Minecraft. A comparison informs the per-user candidate; it is not isolation proof.
2. **Singleton and auth feasibility:** establish the installed Education build's
   supported launch/authentication behavior without touching secrets. If WAM is
   required, reject the simple RunAs route for sign-in rather than disabling the
   broker or claiming a loaded registry hive supplies a real interactive session.
   If supported behavior cannot satisfy the target, record NO-GO and assess a
   different independent isolation mechanism before writing a production provider.
   The next decision must compare a genuinely supported OS route against the
   concrete work needed for our own file/registry/object/authentication isolation.
   Windows does not document a single SDK switch that gives an arbitrary game
   four private authentication brokers. Do not promise such virtualization is
   feasible without hooks, code injection or vendor cooperation. Separate Windows
   sessions also change presentation and Windows-edition constraints; they are
   not an automatic accepted fallback.
3. **Two independent game lifetimes:** only after the previous constraints have a
   justified solution, launch verified official game images and attribute each
   process by token/profile plus creation identity. Observe both concurrently;
   launcher exit zero and two process names are insufficient. No game process
   should be killed by selecting, hiding or restoring a tab.
4. **Presentation recovery:** measure repeated swaps, minimize/restore, fullscreen,
   resize, DPI/monitor moves, manager crash and game crash. Separate fast tab
   changes from expensive launch/profile checks; retain process authority and
   validate it before an operation. A failed swap must restore the prior visible
   session and preserve every other process. Timing and sustained survival must
   be measured, not inferred from a source test.
5. **Manual account acceptance:** only then ask the owner to sign into two licensed
   accounts inside the game. Check persistence and no crossing without reading
   account caches. Extend to four distinct accounts, concurrently playable worlds,
   legitimate multiplayer and sustained resource measurements before claiming
   the replacement works. Package migration/removal is a separate reversible step.

## Executed foundation evidence

At this source change based on `79b93ae`, `npm run check:desktop` passed:
188 frontend tests, 80 Rust tests, 8 existing opt-in Rust tests ignored;
Svelte typecheck (zero errors/warnings), frontend build, Cargo check and format.
Repository contract, structure and dependency metadata checks passed separately.
The complete root suite was not rerun in this change; the preceding revision's
historical assembler EPERM is not claimed resolved.

Two actual Windows regressions query the current test process and its own fixed
PowerShell child. They verify same user/configured profile/session comparisons,
full-width creation identity, stale/zero identity rejection, three-boolean-only
serialization, child survival after the observation handle is dropped, and dead
process rejection while an original handle remains retained. Only that test's
own child is terminated for cleanup. An initial Windows constant import error
was corrected before the final successful desktop run.

No different-user process, game authentication, fullscreen or account scenario
was exercised. The diagnostics command is registered and compiled; the new UI
was not visually exercised in the currently running old executable. Profile
paths and Windows SIDs stay in memory and are never serialized. No credential,
world or account-cache files are read, and no users, ACLs, services or drivers
are changed. This is a read-only foundation, not an installed replacement engine.

Singleton behavior, actual Education authentication, cross-user window control,
independent persistence and four-client performance remain UNKNOWN or
NOT_EXECUTED. The research grants no permission to revive legacy VMs, uninstall
the old driver or enable GitHub Actions.
