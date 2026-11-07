---
id: document.validation.lazy-tester-test-cases-standard
class: DOCUMENT
domain: validation
role: PROCEDURE
authority: WORKING
lifecycle: ACTIVE
---

# Lazy Tester — matriks pengujian produk standar

**Status seluruh kasus di bawah: NOT_EXECUTED.** Panduan ini bukan hasil
source tests, Windows build, installer, atau Minecraft gameplay. Baca
[panduan induk](lazy-tester-test-master.md) untuk urutan, approval, command,
definisi PASS dan evidence. Fitur AI native compile-OFF tidak diuji di
modul standar; lihat [matriks eksperimen](lazy-tester-test-cases-experimental.md).

**Disiplin:** satu case = satu hasil + exact Git SHA + observed effect pada
slot lain. P0 FAIL/INCONCLUSIVE/BLOCKED menghentikan peningkatan jumlah
Minecraft instance. Negative test yang melibatkan corrupt file, fake
process, stale token dan reparse point dilaksanakan terlebih dahulu di
fixture/disposable, **bukan** terhadap akun/world nyata.

## S1 — Repository, prerequisites, source, reviewed DLL (T00–T02)

| Case ID | Prioritas | Tindakan operator | PASS hanya apabila |
| --- | --- | --- | --- |
| STD-SRC-01 | P0 | Dari root, `git branch --show-current`, `git status --porcelain`, `git rev-parse HEAD` | Branch `Local`, source bersih, SHA 40 karakter disimpan |
| STD-SRC-02 | P0 | Bandingkan SHA dengan commit yang memang hendak diuji | Tidak memakai evidence dari revisi lebih lama atau berbeda |
| STD-SRC-03 | P0 | Inventory `App/`/`Data/` dan data akun/world sebelum perubahan | Baseline/backup aman atau use disposable Windows user |
| STD-SRC-04 | P0 | Pastikan Windows 10/11 **native x64** dan sesi interaktif | PowerShell 5.1 “64-bit” tidak dijadikan bukti arsitektur x64 |
| STD-SRC-05 | P1 | `.\DEV.cmd localcheck` | Prasyarat terlapor jujur tanpa download/launch; BLOCKED bukan PASS |
| STD-SRC-06 | P0 | `.\DEV.cmd bootstrap` dengan izin | Pinned toolchain benar di ignored `workspace/toolchains` dan exit code nyata |
| STD-SRC-07 | P0 | `.\DEV.cmd doctor` | Node/npm/Rust/PowerShell tepat seperti `Development/toolchain.json` |
| STD-SRC-08 | P0 | Periksa tracked `Cargo.lock` desktop/runtime-core dan desktop `package-lock.json` | Lockfiles cocok source, tidak diganti versi global sembarang |
| STD-SRC-09 | P0 | Periksa installed MSVC x64 `cl/link` dan Windows SDK x64 `rc/muirct` | Semua file/tool ada dan tidak menjalankan compiler palsu |
| STD-SRC-10 | P1 | Periksa Git for Windows, WebView2, interactive Explorer | Dapat digunakan di tahap UI; tidak membuka game otomatis |
| STD-SRC-11 | P0 | `.\DEV.cmd preflight` setelah izin | Semuanya dieksekusi/actual PASS, bukan hanya script tercantum |
| STD-SRC-12 | P1 | `.\DEV.cmd inspect --json`, `catalog --json`, `plan --json` | Output valid, tidak memicu game/startup otomatis |
| STD-SRC-13 | P0 | Validasi exact hash `bootstrap-compat.dll` dan `bootstrap-enroll.dll` | Dua file approved cocok constant source `REVIEWED_EDUCATION_BOOTSTRAP` |
| STD-SRC-14 | P0 | **FIXTURE ONLY** DLL absent/mismatched, symlink/unreviewed payload | Admission gagal sebelum profil dibuka atau game dijalankan |
| STD-SRC-15 | P0 | **FIXTURE ONLY** git HEAD berubah selama build/check | Source validator menolak dan tidak mencampur bukti |
| STD-SRC-16 | P1 | Periksa bahwa `preflight` tidak mengaktifkan AI input | Default feature OFF, bukan membangun hook C++ AI secara diam-diam |
| STD-SRC-17 | P0 | Audit hasil failure source secara lengkap | First failure, exit code dan owner dicatat; tidak skip test yang gagal |
| STD-SRC-18 | P0 | Pastikan tidak ada GitHub Actions/VMware G0 dan push `main` | Hanya `Local` dan manual on-demand; tidak menghidupkan route retired |

## S2 — Desktop UI sebelum Minecraft dibuka (T03)

| Case ID | Prioritas | Tindakan operator | PASS hanya apabila |
| --- | --- | --- | --- |
| STD-UI-01 | P0 | `.\DEV.cmd dev` | Satu Tauri Lazy Tester window benar-benar render atau error build nyata dicatat |
| STD-UI-02 | P0 | Buka manager tanpa game | Tidak membuka Minecraft, login Microsoft, atau input AI otomatis |
| STD-UI-03 | P0 | Hitung kartu Play | Tepat MC01–MC05 satu kali, urut konsisten |
| STD-UI-04 | P1 | Periksa status connection/backend | Connected hanya dengan verified native response, bukan window render saja |
| STD-UI-05 | P1 | Klik Refresh tanpa game beberapa kali | Read-only, tidak spawn game/controller kedua |
| STD-UI-06 | P1 | Main menu Play dan Settings | Navigasi benar, tidak ada HWND menutup form/toolbar |
| STD-UI-07 | P1 | Buka Data Storage untuk profil stopped | Path/status tersedia sesuai izin, tidak membuka raw akun |
| STD-UI-08 | P1 | Escape/Cancel keyboard pada review action | Cancel tidak menjalankan CloseOne/clear/mutasi |
| STD-UI-09 | P1 | Tab/Enter semua tombol penting | Fokus terlihat dan tindakan cocok label |
| STD-UI-10 | P1 | Resize manager, minimize/restore | UI tidak menampilkan data slot lain atau duplikasi worker |
| STD-UI-11 | P1 | Close manager tanpa game lalu reopen | Tidak ada auto login, normal exit, storage tidak direset |
| STD-UI-12 | P0 | **FIXTURE ONLY** backend native gagal/invalid response | “Unavailable/connection” alih-alih 5 STOPPED palsu siap launch |
| STD-UI-13 | P1 | Startup tiga kali tanpa melakukan tindakan lain | Tidak memunculkan duplicate controller atau lima profil baru tiap start |
| STD-UI-14 | P1 | Periksa tanpa MCP connected | Tidak ada klaim AI active/Testing PASS palsu |
| STD-UI-15 | P1 | Periksa pesan failure di banner/support code | Teks aman dan dapat ditindaklanjuti; tidak menampilkan token/account/raw path |
| STD-UI-16 | P2 | Refresh dan pindah tab saat UI busy | Tidak crash, queue tidak menembakkan dua aksi |
| STD-UI-17 | P1 | Tab/rail pada layar dengan ukuran berbeda | MC01–MC05 tetap terbaca dan dapat dipilih |

## S3 — Progresi game MC01 hingga MC05 (T04–T06)

**Skenario standar satu slot:** klik Launch hanya sekali; tunggu proses
asli dan window; login **manual** menggunakan akun berlisensi yang benar;
masuk world test; bergerak dan berinteraksi dengan objek aman; save/exit
normal; reopen; catat current PID/process-birth/Windows session/HWND dan
dampak peer. Jika tidak playable, STOP; jangan membuka slot berikutnya.

| Case ID | Prioritas | Tindakan operator | PASS hanya apabila |
| --- | --- | --- | --- |
| STD-GAME-01 | P0 | Launch MC01 dari kartu | Original game process terikat MC01 dan window nyata tampil |
| STD-GAME-02 | P0 | Manual login akun A | Auth sah, tidak memakai akun slot lain |
| STD-GAME-03 | P0 | Buka world dummy MC01, gerak/kamera/interaksi | Renderer/input playable; PID saja tidak cukup |
| STD-GAME-04 | P0 | Save & exit MC01 normal | World berhasil disimpan dan tidak ada forced termination |
| STD-GAME-05 | P0 | Reopen MC01 | Saved world/profil retained, tidak direseed |
| STD-GAME-06 | P0 | Launch MC02 saat MC01 masih hidup | Dua process/window independent, bukan refocus MC01 |
| STD-GAME-07 | P0 | Manual login akun B | B benar-benar berbeda, data/hive tidak cross ke A |
| STD-GAME-08 | P0 | MC01 dan MC02 dimainkan bergantian | Input tidak silang, keduanya responsive |
| STD-GAME-09 | P0 | Launch MC03 setelah MC01/02 PASS | Tiga process dan tiga akun/game playable |
| STD-GAME-10 | P0 | Launch MC04 setelah MC03 PASS | Empat process dan empat akun/game playable |
| STD-GAME-11 | P0 | Launch MC05 setelah MC04 PASS | Lima process dan lima akun/game playable dalam waktu overlapping |
| STD-GAME-12 | P0 | Catat PID/birth/session/HWND setiap MCxx | Tidak ada window/process attribution milik peer |
| STD-GAME-13 | P0 | Double-click Launch saat STARTING/RUNNING | Tidak membuat game duplicate pada slot sama |
| STD-GAME-14 | P0 | Buka official Education yang tidak dikelola **disposable** | Controller menolak operasi ambigu, tidak adopt/kill foreign game |
| STD-GAME-15 | P0 | Restart MC03 saja, game peer tetap | MC03 process baru tidak mewarisi preview/window/input child lama |
| STD-GAME-16 | P0 | Lihat ATTENTION/NOT_RESPONDING untuk game windowless | Tidak optimistis menganggap game stopped/launchable |
| STD-GAME-17 | P0 | Setelah lima aktif, lakukan operasi world aman satu per satu | Semua benar-benar interactive, bukan menu/loading/5 splash processes |
| STD-GAME-18 | P1 | Satu MCxx normal exit saat empat peer hidup | Peer tidak ikut exit, world dan data peer utuh |
| STD-GAME-19 | P0 | Reopen semua slot dan cek state save | Per-slot world/account persists sesuai source; login manual bila diminta OS |
| STD-GAME-20 | P0 | Periksa tidak ada login/token copy otomatis | Identitas/credentials tidak dipindah antar-slot |
| STD-GAME-21 | P0 | MCxx login gagal karena lisensi/tenant di test account | Klasifikasikan identity/licensing blocker, tidak membenarkan auto bypass |
| STD-GAME-22 | P0 | Cek tidak ada game lain yang mendapat request CloseOne | Foreground/native ownership tidak bocor |
| STD-GAME-23 | P1 | Lanjut lima game untuk endurance | Waktu overlapping dan gameplay per slot dibuktikan secara objektif |
| STD-GAME-24 | P0 | Klasifikasikan “5 process”, “5 licensed”, “5 playable”, “5 multiplayer” | Empat hasil dicatat **terpisah**, jangan menjadikan salah satu PASS untuk semua |

## S4 — Play surface, DWM preview, window/focus (T07)

| Case ID | Prioritas | Tindakan operator | PASS hanya apabila |
| --- | --- | --- | --- |
| STD-WIN-01 | P1 | Select MC01→02→03→04→05→01 | Surface tampil untuk original slot yang benar |
| STD-WIN-02 | P1 | Dengan 5 game aktif, periksa empat peer preview | Thumbnail benar/tidak bocor ke slot lain |
| STD-WIN-03 | P1 | Select melalui preview/kartu | HWND tervalidasi, tidak merestart proses |
| STD-WIN-04 | P1 | Ubah ukuran manager berulang | Native viewport/aspect dan click hit-testing tetap benar |
| STD-WIN-05 | P1 | Minimize/restore manager | Game state tersimpan; bukan auto-close game |
| STD-WIN-06 | P1 | Alt-Tab ke aplikasi lain lalu kembali | Tidak merebut foreground tanpa aksi pengguna |
| STD-WIN-07 | P1 | Multi-monitor atau DPI berbeda jika tersedia | Surface tidak salah koordinat atau z-order |
| STD-WIN-08 | P0 | Modal login/save Minecraft pada selected game | Dialog tetap accessible, tidak ditutup heuristik salah |
| STD-WIN-09 | P0 | Cancel modal exit game | Tidak diklaim exited, opsi manual recovery tetap |
| STD-WIN-10 | P1 | Separate window MC02, pilih kembali ke Play | Window original dipisah dan kembali tepat, peer tetap |
| STD-WIN-11 | P0 | Close original separated MC02 lalu launch ulang | New process tidak mewarisi status Separate yang lama |
| STD-WIN-12 | P1 | Peer preview error sementara | Error read-only; peer masih playable, tidak auto relaunch |
| STD-WIN-13 | P0 | **FIXTURE ONLY** PID lama reused, birth baru | Window lama tidak diadopsi untuk game baru |
| STD-WIN-14 | P0 | **FIXTURE ONLY** HWND changed/wrong session | Native present/release menolak identitas palsu |
| STD-WIN-15 | P1 | Layout result lama tiba setelah preview invalidation | Cache preview lama tidak menimpa info terbaru |
| STD-WIN-16 | P0 | Game masih hidup tanpa visible main HWND | Status NOT_RESPONDING/uncertain, bukan false crash |
| STD-WIN-17 | P1 | Refresh ketika CloseOne confirmation terbuka | UI close review tidak ditimpa refresh sebelumnya |
| STD-WIN-18 | P0 | Native release selesai setelah UI epoch berubah | Identitas released dihapus hanya jika same slot + PID/birth/session/HWND |
| STD-WIN-19 | P1 | Explorer restart **disposable dengan izin** | Shortcut/manager dapat dipakai, game asing tidak dikill |
| STD-WIN-20 | P0 | DWM preview tampak benar tetapi world belum input-tested | Tidak mengklaim gameplay PASS dari tile/pixel |

## S5 — Normal CloseOne / CloseAll / controller reconnect (T07)

**Jangan force-kill game sebagai metode normal.** Skenario hidden dialog,
unknown native reply, zombie controller lebih aman pada synthetic fixture.
Uji world yang sudah disave untuk close manual di PC nyata.

| Case ID | Prioritas | Tindakan operator | PASS hanya apabila |
| --- | --- | --- | --- |
| STD-LIFE-01 | P0 | Klik Close game MC02 | Hanya MC02 diminta normal close |
| STD-LIFE-02 | P0 | Cancel selected-close review UI | Tidak ada exit request native |
| STD-LIFE-03 | P0 | Confirm CloseOne, biarkan game modal belum selesai | Pending/uncertain, bukan success tanpa exit |
| STD-LIFE-04 | P0 | Confirm exit Minecraft normal | Child process benar-benar exited, bukan cuma dialog ACK |
| STD-LIFE-05 | P0 | CloseOne MC03 saat 4 peer masih aktif | 4 peer tetap, tidak menunggu journal unrelated |
| STD-LIFE-06 | P0 | **FIXTURE ONLY** selected CloseOne stale PID/HWND | Refused, tidak menutup replacement/foreign window |
| STD-LIFE-07 | P0 | Normal close timeout controlled fixture | Tidak force-kill; error/uncertain valid |
| STD-LIFE-08 | P1 | Refresh ketika CloseOne review dibuka | Snapshot lama dibuang, tidak mengganti dialog |
| STD-LIFE-09 | P1 | Native release completed saat review epoch berubah | Old UI binding dibersihkan tanpa menyentuh replacement |
| STD-LIFE-10 | P0 | CloseAll setelah save 5 world dummy | Request normal close untuk owned processes, bukan foreign |
| STD-LIFE-11 | P0 | CloseAll saat game modal belum dijawab | Tidak melaporkan all stopped sebelum exit dikonfirmasi |
| STD-LIFE-12 | P0 | Exit manager dengan game masih open | Tidak meninggalkan hidden game tanpa owner/restore aman |
| STD-LIFE-13 | P0 | Reopen manager dan game/controller asli masih hidup | Authenticated reconnect ke original retained controller |
| STD-LIFE-14 | P0 | **FIXTURE ONLY** controller absent + Education processes ada | Inspect tidak return five STOPPED palsu |
| STD-LIFE-15 | P0 | **FIXTURE ONLY** controller absent, tidak ada game | CloseOne tidak false success; CloseAll idle response dibedakan |
| STD-LIFE-16 | P0 | **FIXTURE ONLY** IPC timeout 45 detik | Original worker tetap, no automatic retry/second actor |
| STD-LIFE-17 | P0 | **FIXTURE ONLY** old response setelah reconnect | Stale reply tidak mengoverride new selection |
| STD-LIFE-18 | P0 | Game crash/normal exit tiba-tiba dalam test world | Peer unaffected; slot issue dan journal benar |
| STD-LIFE-19 | P1 | Normal reopen setelah CloseOne | Saved profile/workload retained |
| STD-LIFE-20 | P0 | **Disposable** second Lazy Tester instance | Conflict gate tidak mengambil alih 5 game |
| STD-LIFE-21 | P0 | **Disposable + after save/approval** Windows reboot | Data survive; jangan mengklaim game otomatis resume |
| STD-LIFE-22 | P0 | **FIXTURE ONLY** journal persistence write failure | Issue ditunjukkan; no overwrite/false complete |
| STD-LIFE-23 | P1 | CloseOne random MC01..MC05 pada dummy worlds | Tidak deadlock, cross-slot status, atau silent game restart |
| STD-LIFE-24 | P0 | Bandingkan Close receipt terhadap OS process exit | PASS hanya jika original child keluar |
| STD-LIFE-25 | P0 | Login modal lokal/unrecognized | Tidak klik arbitrary Windows dialog |

## S6 — Settings, graphics, audio, FPS, manual XYZ/F3 (T08)

**Source defaults:** active 144/background 10 FPS adalah *configured
targets*, bukan nilai frame-rate game nyata. Preset `Preserve` tidak
mengembalikan nilai grafis sebelum High/Low; ia berhenti menulis
future options.txt.

**Retired desktop tests:** `STD-SET-12..16` remain listed for historical ID
traceability, but their former Paste XYZ / F3 Inspector screens were removed
by the operator. They are `NOT_APPLICABLE` to the current desktop and must not
be executed or counted as standard feature failures. Pure coordinate research
and the dormant native Saved Locations store do not expose replacement UI.

| Case ID | Prioritas | Tindakan operator | PASS hanya apabila |
| --- | --- | --- | --- |
| STD-SET-01 | P1 | Buka Settings tanpa game | Inspect tidak mengubah saved data |
| STD-SET-02 | P1 | Save active/background FPS valid lalu reopen | Preferences persists; next-launch behavior diperiksa |
| STD-SET-03 | P1 | Isi background FPS lebih tinggi dari active | Form/backend menolak |
| STD-SET-04 | P1 | Nilai di luar 5..1000 **fixture/form** | Ditolak tanpa overwrite |
| STD-SET-05 | P1 | Ukur measured FPS foreground dan background | Bedakan target 144/10 dengan actual |
| STD-SET-06 | P1 | Audio Keep/Mute 2+ clients | Hanya owned sessions yang boleh berubah |
| STD-SET-07 | P0 | **Disposable** audio device unavailable | Error aman, no foreign app mute |
| STD-SET-08 | P1 | Pilih Preserve pada stopped MCxx | Tidak overwrite Minecraft option saat next launch |
| STD-SET-09 | P1 | High/Low pada stopped MCxx | Preset slot tersebut saja, next launch |
| STD-SET-10 | P0 | Apply preset ketika satu game berjalan | Refused, no live options.txt edit |
| STD-SET-11 | P1 | Pilih preset MC01–MC05 satu per satu | Slot settings tidak tertukar |
| STD-SET-12 | P1 | RETIRED — Paste XYZ di UI desktop dihapus | NOT_APPLICABLE: tidak ada layar Paste XYZ |
| STD-SET-13 | P1 | RETIRED — hasil chunk/blok pada UI Paste XYZ dihapus | NOT_APPLICABLE: fungsi koordinat murni bukan UI desktop |
| STD-SET-14 | P1 | RETIRED — pilihan teks Paste XYZ dihapus | NOT_APPLICABLE: tidak ada aksi clipboard pada layar ini |
| STD-SET-15 | P0 | RETIRED — F3 Inspector desktop dihapus | NOT_APPLICABLE: tidak ada F3 Inspector dalam Play |
| STD-SET-16 | P0 | RETIRED — transisi saat F3 Inspector terbuka | NOT_APPLICABLE: F3 desktop tidak ada; slot identity diuji di STD-UI/WIN |
| STD-SET-17 | P1 | Missing/legacy options key | Tidak mengarang setting yang tidak dikenal |
| STD-SET-18 | P2 | Save/refresh UI bersamaan | Tidak duplicate submit atau freeze |
| STD-SET-19 | P1 | Reopen manager setelah audio/graphics/FPS changes | Last reported preferences benar dan no unapproved cross-slot writes |

## S7 — Data Storage, cache, settings recovery dan preservasi (T08)

**Full Clear benar-benar dapat menghapus data akun, world, cache, settings,
backup untuk launcher target.** Test yang mutatif hanya disposable/dummy
dengan explicit approval A6, bukan pada profile produksi. Stale token,
reparse point, manipulated hive dan interrupted clear **fixture only**.

| Case ID | Prioritas | Tindakan operator | PASS hanya apabila |
| --- | --- | --- | --- |
| STD-DATA-01 | P0 | Buka Data Storage kelima stopped profile | Per-slot folder/path/bytes/files tidak bercampur |
| STD-DATA-02 | P0 | Cek metadata keberadaan catalog dan hive tanpa isi privat | Original identity tetap |
| STD-DATA-03 | P0 | Reopen manager setelah storage inspect | Canonical Data pointer dan catalog tidak direset |
| STD-DATA-04 | P1 | Resource packs manifest inventory | Jumlah folder ber-manifest, bukan loaded packs |
| STD-DATA-05 | P1 | World pack declarations | Membaca hanya bounded world_resource_packs.json, bukan world DB |
| STD-DATA-06 | P1 | **FIXTURE ONLY** storage inspection budget exceeded | Partial/UNKNOWN, bukan false zero |
| STD-DATA-07 | P0 | Inspect game options ketika slot active/uncertain | Tidak membuka/mengubah data yang admission-nya tidak aman |
| STD-DATA-08 | P1 | Review Clear web cache → Cancel | Tidak ada deletion |
| STD-DATA-09 | P1 | Clear recognized cache dummy stopped setelah izin | Hanya target cache, worlds/accounts dan 4 peer preserved |
| STD-DATA-10 | P0 | Clear cache saat target aktif | Backend refuse |
| STD-DATA-11 | P1 | Reset dan Restore game settings dummy | Backup diciptakan, reversible sesuai contract |
| STD-DATA-12 | P1 | Enable/Disable Texture Streaming **hanya existing key** | Tidak mensintesis option baru |
| STD-DATA-13 | P1 | Disable Improved Input Response dummy berhenti | Per-slot dan backup; RAM effect tetap unverified |
| STD-DATA-14 | P0 | Review Full Clear lalu Cancel | Tidak ada profile/world deletion |
| STD-DATA-15 | P0 | **Disposable-only** Full Clear dummy MCxx exact name/ack | Hanya target yang clear; peer lain utuh |
| STD-DATA-16 | P0 | **FIXTURE ONLY** stale full-clear token | Rejected |
| STD-DATA-17 | P0 | **FIXTURE ONLY** interrupted clear/journal recovery | Status pending/recovery, bukan silent reseed |
| STD-DATA-18 | P0 | **FIXTURE ONLY** symlink/junction/path traversal | Menolak link ke luar root, tidak follow external targets |
| STD-DATA-19 | P0 | **Disposable** data choose/locate berbeda volume/user | Cross-volume/user identity unsupported refused |
| STD-DATA-20 | P0 | **Disposable-only** explicit legacy schema1 migration | Source tidak ditimpa, new schema catalog published last |
| STD-DATA-21 | P0 | Compare baseline bytes/files sebelum/sesudah safe operation | Tidak ada move/delete/copy account/world unauthorized |
| STD-DATA-22 | P0 | Restart Minecraft/manager setelah change dummy | Original identities and other slots preserved |
| STD-DATA-23 | P1 | Metadata pack tidak tersedia/parsial | UNKNOWN/partial honest |
| STD-DATA-24 | P0 | Clear/migrate dengan unmanaged games active | Refusal independent of UI button |
| STD-DATA-25 | P0 | **FIXTURE ONLY** corrupted/foreign catalog/hive | Fail closed, bukan membuat profil baru diam-diam |

## S8 — Package, root launcher, unsigned installer (T09)

Dibutuhkan izin A5. Semua install/repair/uninstall/staging failure
pertama kali pada disposable Windows user dan **dummy Data**. Standard
package tidak membawa Experimental AI binaries.

| Case ID | Prioritas | Tindakan operator | PASS hanya apabila |
| --- | --- | --- | --- |
| STD-PKG-01 | P0 | Tutup manager sebelum `DEV.cmd package` | Tidak ada manager memegang App yang ingin direfresh |
| STD-PKG-02 | P0 | Build candidate dengan standard reviewed DLL | Output nyata, exit 0, NSIS candidate |
| STD-PKG-03 | P0 | Periksa package-manifest + SHA256SUMS | Hash, sourceCommit, version dan branch cocok |
| STD-PKG-04 | P0 | Periksa policy requiredFiles | Main exe, 2 DLL, 2 role EXE, MUI, distribution policy |
| STD-PKG-05 | P0 | Periksa policy excludedPayloads | Tidak membundel private profiles/worlds/tokens/VM/Guest Agent |
| STD-PKG-06 | P0 | Verifikasi root Lazy Tester.exe + App/build-info.json | Binary receipt dan launcher SHA cocok |
| STD-PKG-07 | P1 | `RUN.cmd` setelah staging | Buka checked root exe, tidak build/game auto launch |
| STD-PKG-08 | P1 | Shortcut Desktop/Start Menu via Explorer | Target benar, tidak langsung App stale |
| STD-PKG-09 | P1 | Periksa WebView2 cache | Cache ada di project Data/Cache/WebView |
| STD-PKG-10 | P0 | **Disposable** install user-level NSIS | Tanpa menghapus Data dummy |
| STD-PKG-11 | P0 | **Disposable** repair/reinstall same candidate | Data/prefs preserved |
| STD-PKG-12 | P0 | **Disposable** uninstall/reinstall dummy | Tidak stop foreign Minecraft, Data tetap |
| STD-PKG-13 | P0 | **Disposable** coexistence previous Preview identity | com.mivubi.lazytester terpisah dan tidak migrate diam-diam |
| STD-PKG-14 | P0 | **FIXTURE ONLY** interrupted stage/upgrade | App lama/launcher backup preserved, Data untouched |
| STD-PKG-15 | P0 | **FIXTURE ONLY** tampered role/launcher hashes | Refuse, no execution of unreviewed image |
| STD-PKG-16 | P0 | Periksa unsigned/release status | Tidak diklaim signed/published |
| STD-PKG-17 | P1 | Reopen app setelah disposable reinstall | Lima fixed slots dan safe status retained |
| STD-PKG-18 | P0 | Cek legacy pointer dan dummy Data | Installer tidak diam-diam delete/move |
| STD-PKG-19 | P0 | App process/shortcut startup smoke source receipt | Source SHA and Explorer parent verified, not hidden build tool |
| STD-PKG-20 | P0 | Staging ketika package candidate sedang berjalan | Refusal, no forced kill |

## S9 — Multiplayer dan performa (T08/T09)

Durasi di bawah **rekomendasi metode test**, bukan SLA. Gunakan world
khusus testing, semua akun Education resmi, same version dan network/
tenant policy yang relevan. Catat metrik pada workload yang sama.

| Case ID | Prioritas | Tindakan operator | PASS hanya apabila |
| --- | --- | --- | --- |
| STD-PERF-01 | P0 | MC01 host world uji, MC02 join manual | Dua pemain terlihat dan responsive |
| STD-PERF-02 | P0 | MC03/04/05 join satu per satu | Lima pemain nyata, tidak hanya proses |
| STD-PERF-03 | P1 | Input/interaksi per client | Game world benar dan tidak salah account |
| STD-PERF-04 | P1 | Host/peer keluar/rejoin normal | Perilaku world nyata dicatat, bukan claim auto resume |
| STD-PERF-05 | P0 | MC03 exit saat empat peer online | Peer lain tidak ikut close |
| STD-PERF-06 | P1 | Save/reopen host world dan satu peer | Persistence benar |
| STD-PERF-07 | P1 | CPU/RAM/working-set/private-memory pada 1–5 client | Angka measured dan sample interval dicatat |
| STD-PERF-08 | P1 | FPS foreground/background serta frame-time | Target 144/10 tidak diasumsikan actual |
| STD-PERF-09 | P1 | Contoh soak 10 menit 2 game dan 20 menit 5 game | Crash/freeze/memory trend/input/network diobservasi |
| STD-PERF-10 | P1 | Switch focus + DWM saat semua game hidup | Tidak mempengaruhi game lain atau retarget input |
| STD-PERF-11 | P2 | Ubah resolusi/monitor/DPI selama workload sama | Hasil dibandingkan secara adil, bukan dari satu screenshot |
| STD-PERF-12 | P1 | Audio active/background during 5 games | Tidak mute foreign process |
| STD-PERF-13 | P0 | Minecraft atau network error | Status FAIL/BLOCKED/INCONCLUSIVE dijaga, tidak false PASS |
| STD-PERF-14 | P1 | Read-only storage/cargo footprint report | Tidak melakukan cleanup cache atau world |
| STD-PERF-15 | P1 | Catat session startup time setiap slot | Waktu per-slot, bukan total satu PC saja |
| STD-PERF-16 | P1 | Lima game aktif sekaligus selama test | Ada overlap waktu, bukan game dijalankan bergantian |

## S10 — Security/negative evidence dan akhir test

| Case ID | Prioritas | Tindakan aman | PASS hanya apabila |
| --- | --- | --- | --- |
| STD-SEC-01 | P0 | **FIXTURE** wrong PID + same HWND | Not authorized |
| STD-SEC-02 | P0 | **FIXTURE** PID same, birth differs | Original identity fence prevents retarget |
| STD-SEC-03 | P0 | **FIXTURE** HWND invalid, other Windows session | No present/close of unrelated process |
| STD-SEC-04 | P0 | **FIXTURE** missing controller with live Education | No fake STOPPED/recovery |
| STD-SEC-05 | P0 | **FIXTURE** 45s IPC timeout/delayed reply | No duplicate worker/new game owner |
| STD-SEC-06 | P0 | **FIXTURE** read-only snapshot after close review | Old async state discarded |
| STD-SEC-07 | P0 | **FIXTURE** old DWM preview after identity change | Cache entry refused, other slots unaffected |
| STD-SEC-08 | P0 | **FIXTURE** stale data deletion token | No deletion |
| STD-SEC-09 | P0 | **FIXTURE** storage symlink/reparse point | No follow/delete outside safe data root |
| STD-SEC-10 | P0 | **FIXTURE** package DLL/exe swap | Mismatch refused |
| STD-SEC-11 | P0 | Private log/screenshot review | No personal account IDs or token upload |
| STD-SEC-12 | P0 | No auto source/test/CI/data cleanup | Operator remains in control |
| STD-SEC-13 | P0 | Unknown native/game outcome | Recorded INCONCLUSIVE/BLOCKED not PASS |
| STD-SEC-14 | P0 | Validate exact same SHA for all reported results | No old PASS mixed with new HEAD |

## Operator worksheet MC01–MC05

**Semua kolom saat ditulis adalah NOT_EXECUTED.** Isi setelah benar-benar
diobservasi:

| Slot | Alias akun berlisensi | PID/birth/HWND | Manual sign-in | World/render/input | Save/reopen | CloseOne/peer isolation | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| MC01 | A | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED |
| MC02 | B | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED |
| MC03 | C | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED |
| MC04 | D | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED |
| MC05 | E | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED | NOT_EXECUTED |

**Tidak boleh memberi FIVE_CLIENT_ACCEPTANCE PASS** tanpa lima akun yang
sah, lima proses sebenarnya, lima world playable **pada waktu bersamaan**,
persistensi/normal close dan tidak ada P0 fail. Jika fixture/source test
lulus tetapi gameplay tidak berjalan, klasifikasi hanya SOURCE_EXECUTION,
bukan gameplay.

**Source owner jika FAIL:** [implementation map](../system/implementation-map.md).
**Catat project status hanya di:** [planning/development.md](../../planning/development.md).
