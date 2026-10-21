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
