---
id: document.validation.lazy-tester-test-master
class: DOCUMENT
domain: validation
role: PROCEDURE
authority: WORKING
lifecycle: ACTIVE
---

# Lazy Tester — panduan induk pengujian MC01–MC05

**Disusun:** 3 November 2026 · **Repository:** OpenITStudios/M-Minecraft-Emulator · **Branch:** `Local` · **Status seluruh pengujian:** **NOT_EXECUTED**. Pembuatan panduan ini **tidak** menjalankan Node/Vitest, Rust, Windows, installer, Minecraft, input AI, atau CI. Tes aktual memerlukan persetujuan operator sendiri.

**Catatan penggunaan:** katalog 306 kasus di bawah adalah cakupan *full
regression*, **bukan checklist wajib pada local test pertama**. Untuk
pengujian yang lebih efisien, jalankan [13 skenario core local smoke]
(efficient-local-testing.md) terlebih dahulu; pilih `DEV.cmd plan
--profile local-smoke --json` setelah toolchain siap. Prioritas P0 pada
katalog adalah *severity/risiko*, bukan perintah untuk mengeksekusi seluruh
P0 setiap kali. `check:quick` adalah screen source, bukan full-source PASS.

Panduan ini menjelaskan **urutan, command, persiapan, bukti dan syarat lulus**. Detail langkah per kasus berada di:
- [Matriks uji produk standar](lazy-tester-test-cases-standard.md) — source, desktop, MC01–MC05, window/DWM, lifecycle, penyimpanan, installer dan performa.
- [Matriks uji MCP, AI, QA](lazy-tester-test-cases-experimental.md) — terpisah, fitur experimental/source-only dan aturan pengujian khusus.

**Baca terlebih dahulu:** [first-run Windows](local-pc-first-run.md), [AI Computer Use khusus](ai-computer-use-localtest.md), [implementation map](../system/implementation-map.md), [data layout](../system/data-layout.md), [package contract](../../Development/engine/distribution/lazy-tester-package.json), [MCP docs](../../Development/tooling/agent/README.md), serta [current handoff](../../planning/development.md). Source di **HEAD `Local` pada saat test** selalu lebih tinggi daripada teks panduan ini.

## 1. Batas sistem — fitur yang benar-benar harus dibuktikan

**Produk standar:** Windows 10/11 x64 native, Tauri/Svelte/Rust, satu controller native, reviewed `bootstrap-compat.dll` dan `bootstrap-enroll.dll`, lima profil MC01–MC05, login Minecraft Education manual, Play/DWM preview, normal CloseOne/CloseAll, per-slot Data, setting, dan unsigned development installer.

**Fitur yang TIDAK boleh diasumsikan siap:** input AI game 4–5 klien, experimental hook/ABI C++, Skills Probe gameplay, F3 live XYZ/Copy, resource-pack loading aktual, lima akun sungguhan, efektivitas FPS policy, multiplayer, atau signed public release. Ini adalah bukti Windows/Minecraft yang harus didapat secara terpisah. Native AI input tetap compile-OFF/default-OFF dan tidak ada di package normal. Reviewed **AI DLL khusus** belum siap sebagai artefak runtime; jangan menggunakan standard DLL sebagai pengganti.

**Jalur lama yang tidak aktif:** VMware, Sandboxie, Guest Agent, legacy VMware G0–G4, `DEV.cmd g0`, driver alternatif, auto login, bypass lisensi, GitHub Actions. Jangan mengaktifkannya untuk membuat tes lulus.

## 2. Definisi hasil dan tingkat risiko

| Status | Artinya |
| --- | --- |
| `NOT_EXECUTED` | Belum pernah benar-benar dicoba; default semua ID dalam kedua matriks |
| `BLOCKED` | Belum ada izin, mesin, tool, reviewed DLL, lingkungan atau capability |
| `PASS` | Langkah benar-benar dijalankan pada SHA tercatat; hasil objektif sesuai |
| `FAIL` | Langkah dijalankan dan hasil melanggar expected |
| `INCONCLUSIVE` | Bukti ambigu, timeout, tidak dapat mengaitkan proses/game/akun secara tepat |
| `NOT_APPLICABLE` | Tidak relevan untuk mode yang dipilih; wajib beri alasan |

P0 = data safety, original process/window ownership, identitas account/slot, input isolation, normal close, rollback. P1 = fungsi utama lima game dan kestabilan. P2 = UX/performance/kompatibilitas lanjutan. **Satu P0 FAIL/UNKNOWN menghentikan eskalasi** jumlah klien dan status GO.

**Level bukti tidak boleh disamakan:**
`REMOTE_GITHUB` → source review;
`SOURCE_EXECUTION` → actual Node/Svelte/Rust tests;
`PACKAGE_EXECUTION` → build, installed payload/repair/uninstall;
`TARGET_WINDOWS` → actual native window/process;
`MINECRAFT_RUNTIME` → login, rendered world, control, multiplayer;
`FIVE_CLIENT_ACCEPTANCE` → lima game berbeda playable bersamaan dengan lima akun berlisensi. Authored unit test, lima PID, IPC ACK atau PNG saja **bukan** gameplay PASS.

## 3. Persetujuan dan perlindungan data — wajib sebelum mulai

| Gate | Aksi | Persetujuan dan tempat |
| --- | --- | --- |
| A0 | Review source + tulis dokumentasi | GitHub/ChatGPT tanpa eksekusi PC |
| A1 | Git/`DEV.cmd localcheck` | Izin inspeksi read-only Windows |
| A2 | `bootstrap`, `doctor`, `preflight`, source tests | Izin download/install locked dependencies/build |
| A3 | `DEV.cmd dev`, menjalankan manager | Izin menjalankan aplikasi desktop |
| A4 | MC01–MC05, world, gameplay, multiplayer | Izin game runtime + lima akun sah |
| A5 | `DEV.cmd package`, `App/` staging, NSIS | Izin build/stage; install hanya di user/PC disposable |
| A6 | Reset, Clear cache, Full Clear, data migration, fault injection | Izin **per tindakan**; backup dan data dummy/disposable |
| A7 | AI native C++, MCP input, Skills Probe real world | Izin khusus + reviewed AI pair/ABI + pemilik world |

**Larangan:** jangan menghapus/memindah `App/` atau `Data/` untuk mencoba memperbaiki error; jangan pakai `git reset --hard`, `git clean -fd`, `cargo clean`, `Stop-Process` atau copy login token/WAM/registry/hive dari akun nyata sebagai jalan pintas. Jangan uji Full Clear pada world/account produksi. Jika Windows/gameplay belum diizinkan, tandai downstream `NOT_EXECUTED`.

### 3.1 Lingkungan dan test data

- [ ] Windows 10/11 **native x64**, sesi Desktop/Explorer interaktif, WebView2, Git for Windows, VS MSVC Hostx64/x64 tools, Windows SDK x64 `rc.exe` dan `muirct.exe` tersedia.
- [ ] Catat Windows version/build, OS architecture, CPU/GPU/driver/RAM, display/DPI, Minecraft Education Desktop version dan tanggal/timezone. Jangan upload serial hardware atau account IDs.
- [ ] **Utamakan Windows user/PC disposable** untuk package/installer, migration/clear dan negative fault scenarios. Jika PC mempunyai data lama, lakukan backup dengan bukti dapat dibaca sebelum T5/T6.
- [ ] Siapkan lima account berlisensi yang sah dan beri alias **A→MC01, B→MC02, C→MC03, D→MC04, E→MC05**. Login selalu manual; akun/tenant/version perlu kompatibel untuk multiplayer Education.
- [ ] Siapkan world test sendiri (contoh alias `W-TEST-1`) untuk rendering, movement, satu interaksi aman, save/reopen, join/rejoin. Jangan gunakan world produksi.
- [ ] Catat baseline **hanya metadata** keberadaan/ukuran file: `Data/`, `Data/Settings/`, `Data/profiles.json`, `Data/Launchers/Minecraft-1..5`, legacy `Data/Profiles`, legacy `App/data-location.json` bila ada. Jangan buka atau upload konten rahasia.
- [ ] Siapkan folder **privat dan diabaikan Git** seperti `workspace/` untuk hasil lokal. Screenshot/log disanitasi dari email, username, title world privat, paths pribadi, tokens, data account.
- [ ] Pastikan semua operator paham save dan **normal exit Minecraft**, cancel confirmation, dan cara menghentikan pengujian tanpa mematikan proses asing.

## 4. Urutan gerbang yang tidak boleh dilompati

| Tahap | Mulai jika | Harus terjadi sebelum lanjut |
| --- | --- | --- |
| T00 Checkout | A1 | `Local`, SHA 40 karakter, clean status, backup direncanakan |
| T01 Source | A2 + T00 | Quick source screen PASS untuk local smoke; full preflight PASS untuk release/full regression |
| T02 Reviewed DLL | Binary resmi disediakan | Dua digest sesuai source standar; bukan test/AI DLL |
| T03 UI tanpa game | A3 + T02 | Satu manager dan lima kartu, tidak auto-launch game |
| T04 MC01 | A4 + T03 | World interaktif, manual login, normal close/reopen, data utuh |
| T05 MC02 | T04 PASS | Dua akun/game/profile berbeda yang playable simultan |
| T06 MC03–MC05 | T05 PASS | Tambahkan bertahap, lima game benar-benar playable |
| T07 Workspace/lifecycle | T06 | DWM/focus, CloseOne/All, timeout/reconnect, isolasi peer |
| T08 Settings/storage/perf | T07 | Mutation aman, data utuh, angka performa benar-benar diukur |
| T09 Package/installer | A5; workstation disposable | Candidate/manifest, stage, install/repair/uninstall preserves Data |
| X0 MCP source | Izin khusus | Source fixtures dan tool contract; bukan input game |
| X1 AI live | Reviewed AI ABI/binary + A7 | Human MC01 + AI MC02 berhasil dalam game |
| X2 4/5 AI, Probe/map | X1 PASS + A7 | Hasil gameplay dan original Probe report terbukti secara terpisah |

Angka T00–T09 hanya label panduan ini; **bukan** menghidupkan kembali VMware G0–G4.

## 5. Command yang berlaku (Windows PowerShell)

### 5.1 Clone baru

~~~powershell
git clone --branch Local --single-branch https://github.com/OpenITStudios/M-Minecraft-Emulator.git
Set-Location .\M-Minecraft-Emulator
git branch --show-current
git status --porcelain
git rev-parse HEAD
.\DEV.cmd localcheck
~~~

Untuk checkout yang sudah ada, dari root repo:

~~~powershell
git switch Local
git fetch origin Local
git pull --ff-only origin Local
git branch --show-current
git status --porcelain
git rev-parse HEAD
.\DEV.cmd localcheck
~~~

**PASS T00:** `Local`, source clean, SHA valid. Jika source berisi perubahan
lokal atau tidak bisa fast-forward, **STOP** dan review; jangan reset/hapus.
`localcheck` adalah pemeriksaan read-only dan mungkin mencetak BLOCKED pada
clone baru; itu tidak berarti build/test dijalankan.

### 5.2 Bootstrap, doctor, quick source atau full preflight (setelah A2)

~~~powershell
.\DEV.cmd bootstrap
.\DEV.cmd doctor
.\DEV.cmd localcheck
.\DEV.cmd inspect --json
.\DEV.cmd catalog --json
.\DEV.cmd plan --json
.\DEV.cmd plan --profile local-smoke --json
.\DEV.cmd setup
.\DEV.cmd check:quick
# Untuk full regression/release dengan izin terpisah:
.\DEV.cmd preflight
~~~

`bootstrap` mengunduh dan menyiapkan local pinned tools di
`workspace/toolchains`. `preflight` dapat menginstal locked npm dependencies
dan benar-benar menjalankan source verification. Simpan exit code/first error
dan full SHA. `Development/toolchain.json` saat penyusunan menyebut Node
24.22.0, npm 12.2.0, Rust 1.99.1, PowerShell 7.6.7; **baca nilai terbaru
pada HEAD saat test**, bukan bergantung pada angka di panduan.

Subset test hanya jika dipilih dan diizinkan (tidak menggantikan full PASS):

~~~powershell
.\DEV.cmd check:frontend
.\DEV.cmd check:rust
.\DEV.cmd test
.\DEV.cmd check
~~~

`DEV.cmd plan --json` hanya membantu rekomendasi; jalur task optional
`DEV.cmd run <task-id> --expect-sha <full-sha>` boleh dipakai **hanya** jika
task tersebut memang terdaftar dalam `DEV.cmd catalog --json`. Jangan
membuat ID atau command baru secara spekulatif.

### 5.3 Reviewed standard DLL gate

Standard `dev`/`package` memerlukan kedua file:
`bootstrap-compat.dll` dan `bootstrap-enroll.dll`. Mereka **tidak ada
di Git**. Hash harus cocok tepat dengan constant
`REVIEWED_EDUCATION_BOOTSTRAP` pada
`Development/apps/desktop/src-tauri/src/engine/independent_bootstrap.rs`.
Bukan experimental AI pair. Jangan buat placeholder, mengubah hash, atau
mengganti sumber approved DLL agar bisa membangun aplikasi.

~~~powershell
$env:LAZY_TESTER_NATIVE_PAYLOAD_DIR = 'C:\path\to\reviewed-native-payload'
.\DEV.cmd localcheck
.\DEV.cmd dev
~~~

Path di atas **placeholder**. Gunakan direktori absolut file asli dan
**PowerShell session yang sama**. Jika local `App/independent-bootstrap`
berisi reviewed DLL yang hash-nya benar, variabel tetap harus diset ke
path absolut folder tersebut sebelum `dev/package`. Tanpa DLL/hash yang
disetujui: **BLOCKED: REVIEWED_NATIVE_PAYLOAD_MISSING**.

### 5.4 Candidate package, bukan release (setelah A5)

Di user/PC **disposable**, manager sudah ditutup normal dan data test
diamankan:

~~~powershell
.\DEV.cmd package
.\RUN.cmd
~~~

`package` membangun unsigned NSIS candidate, membuat
`workspace/packages/lazy-tester/` (manifest/checksums), staging `App/`,
root `Lazy Tester.exe`, shortcut dan source-defined Desktop startup
observation. Ini **bukan** memasang installer atau menerbitkan rilis;
staging dapat memodifikasi `App/`. `RUN.cmd` hanya menjalankan existing
root executable dan tidak build. Actual NSIS install/repair/uninstall
adalah langkah terpisah pada disposable environment.

## 6. Kapan harus berhenti: aturan P0 stop

STOP sebelum menambah game/AI ketika terjadi salah satu:
- Hash reviewed DLL salah, source/HEAD berganti, toolchain mismatch atau
  source check gagal.
- MC01/02 tidak benar-benar playable; launch berikutnya malah memilih
  instance lama; account/profile/world menyeberang slot.
- Proses/window di luar controller terpengaruh, wrong HWND atau PID-birth,
  CloseOne menutup peer, modal terjawab otomatis, saved data berubah tanpa izin.
- Input game/macOS/cross-user/AI dievaluasi tanpa capability dan consent.
- IPC ACK/transport treated as movement/interactive PASS; frame gambar
  atau Probe source-only dianggap game PASS.
- Full clear/migration menargetkan akun nyata atau recovery journal
  gagal tanpa backup; tidak ada privilege untuk melanjutkan.
- Log/screenshot berisi akun, tokens atau private path yang akan dipublikasi.

Jika blocked/unknown, rekam kondisi **apa adanya**. Jangan bypass admission
dan jangan melompat ke test kategori berikutnya.

## 7. Bukti dan format laporan — satu baris per case ID

Untuk setiap kasus dalam dua matriks, salin template:

~~~text
CASE ID            : <STD-... atau MCP-/AI-/QA-...>
SOURCE BRANCH/SHA  : Local / <full 40-character SHA>
EXECUTION DATE     : <YYYY-MM-DD HH:mm +time zone>
SYSTEM             : Windows build/native x64; CPU/GPU/RAM/driver; game version
PRODUCT/MODE       : dev | unsigned package | standard | experimental opt-in
NATIVE DLL GATE    : reviewed+hash match | missing | blocked | N/A
APPROVAL           : A1/A2/A3/A4/A5/A6/A7
CLIENT/WORLD ALIAS : MC01(A) ... MC05(E), W-TEST-1 (no raw account IDs)
PREREQUISITES      : list actually PASS cases, or first blocked prerequisite
TEST ACTION        : exact command or manual UI/game step performed
EXPECTED           : what source/runbook requires
OBSERVED           : what was actually seen, without guessing
PROCESS IDENTITY   : sanitized slot/PID-birth/HWND binding where applicable
DATA BEFORE/AFTER  : preserved | changed as approved | uncertain
MEASURED METRICS   : tool/interval/duration for FPS/RAM/CPU/GPU if applicable
RESULT             : NOT_EXECUTED | BLOCKED | PASS | FAIL | INCONCLUSIVE
SEVERITY           : P0 | P1 | P2
EVIDENCE REF       : sanitized private file ID/log reference
FIRST WRONG OWNER  : path in docs/system/implementation-map.md
NEXT ACTION        : one exact blocker or fix; do not invent success
~~~

**Laporan 5 client**: catat MC01–MC05 terpisah (akun alias, distinct PID,
birth/HWND, login manual, rendered world + input, save/reopen, normal
close/reconnect). Pisahkan empat klaim: **5 proses**; **5 licensed accounts**;
**5 world interaktif simultan**; **5 pemain multiplayer**. Lima window
splash hanya berarti belum ada bukti gameplay.

## 8. GO/NO-GO — klaim akhir yang boleh dibuat

| Klaim | Harus sudah benar-benar teramati |
| --- | --- |
| SOURCE_EXECUTION | Full exact-SHA source, Svelte/Node/Rust checks executed, PASS |
| PACKAGE_EXECUTION | Candidate/manifest/hash dan actual disposable installed payload, repair/uninstall |
| TARGET_WINDOWS | Real desktop/Explorer/WebView2, one controller, process/window state and recovery |
| MINECRAFT_RUNTIME | Manual sign-in, rendered interactive world, input, save/reopen, network join |
| FIVE_CLIENT_ACCEPTANCE | Lima Education client playable bersamaan + lima licensed accounts + isolation + P0 PASS |
| EXPERIMENTAL_AI_INPUT | Reviewed AI DLL/ABI, actual movement/camera/neutral/human isolation dalam game |
| MAP_QA | Skenario map benar-benar diuji, bukan sekadar original Probe all-PASS di log |

P0 belum selesai → **NO-GO** untuk klaim penuh. `NOT_EXECUTED` bukan FAIL,
tetapi juga bukan PASS. Live automatic F3 XYZ/Copy, AI native input, dan
signed public release saat authoring tidak boleh dipaksa menjadi kriteria
PASS standar bila fitur/capability tersebut memang belum tersedia.

## 9. Urutan operator paling ringkas pada hari-H

1. Tentukan izin, mesin disposable/backup, source SHA dan world uji.
2. `localcheck` → `bootstrap/doctor` → `preflight` sesuai persetujuan.
3. Dua reviewed standard DLL cocok, env path eksplisit.
4. `dev`: manager-only, tepat lima fixed slots, no auto Minecraft/login.
5. MC01 playable/save/reopen → MC02 paralel → MC03/04/05 satu per satu.
6. Window/DWM/focus → CloseOne/CloseAll → reconnect → data preservation.
7. Settings/storage non-destruktif → measured FPS/RAM/workload.
8. `package`/NSIS install/repair/uninstall **disposable terpisah**.
9. **Hanya jika approval khusus**, ikuti jalur MCP/AI/Probe pada matriks eksperimen.
10. Catat case IDs dan evidence, kemudian laporkan first failing owner di
    [implementation map](../system/implementation-map.md); project status
    di [planning/development.md](../../planning/development.md), bukan
    database/status file kedua.

**Catatan permanen:** seluruh langkah dokumen ini **NOT_EXECUTED** saat
ditulis. Pengujian yang akan datang harus tetap memakai bukti nyata dan
sumber HEAD yang sama.
