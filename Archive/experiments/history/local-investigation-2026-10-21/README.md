# Arsip source investigasi lokal - 21 Oktober 2026

**HISTORICAL / ANALYSIS ONLY.** Diunggah atas permintaan operator untuk
pembelajaran dan diskusi ChatGPT. Arsip ini tidak melanjutkan pengembangan,
mengaktifkan Sandboxie, atau memberi izin menjalankan skrip lama. Status engine
masih **PAUSED** di [handoff](../../../../planning/development.md).
Audit dimulai pada `Local` `2668c21037451b6ece2497581092d86047c5c2d1` yang saat
itu sama dengan GitHub. Tidak ada perubahan tracked atau untracked biasa yang
belum dikirim; skrip di bawah sebelumnya tersembunyi oleh aturan ignore workspace.

## Rujukan source dan snapshot sebelum pengarsipan UI

Bridge/layar lama yang sudah dikeluarkan dari source aktif ditautkan ke snapshot
Git `5601c354`. Rujukan ini hanya untuk membaca histori, bukan menjalankan UI lama.

| Bagian | Source / catatan kanonis |
| --- | --- |
| Konteks diskusi, keputusan dan batas bukti | [Independent isolation](../../../../docs/system/independent-isolation.md) |
| Arsitektur dan tanggung jawab | [Architecture](../../../../docs/system/architecture.md) |
| Status jeda | [Development handoff](../../../../planning/development.md) |
| Batas pensiun dan perlindungan data lama | [Retirement register](../../../../docs/system/legacy-retirement.md) |
| Inspector identitas Windows | [instance_identity.rs](../../../../Development/apps/desktop/src-tauri/src/engine/instance_identity.rs), [typed bridge arsip](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/5601c354fb91db78a2697ceb6ec710590d7d17be/apps/desktop/src/app/bridge/processContext.ts), [tests arsip](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/5601c354fb91db78a2697ceb6ec710590d7d17be/apps/desktop/test/process-context.test.ts) |
| Native AppContainer sintetis | [appcontainer-probe.cpp](../../../../Development/tooling/acceptance/windows/appcontainer-probe.cpp), [runner](../../../../Development/tooling/acceptance/windows/test-appcontainer-isolation.ps1) |
| Tab dan recovery window | [workspace_surface.rs](../../../../Development/apps/desktop/src-tauri/src/engine/workspace_surface.rs), [NativeWorkspace.svelte arsip](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/5601c354fb91db78a2697ceb6ec710590d7d17be/apps/desktop/src/NativeWorkspace.svelte) |
| Penolakan launch route lama | [instance_catalog.rs](../../../../Development/apps/desktop/src-tauri/src/engine/instance_catalog.rs) |
| Source fork lama, sudah dipensiunkan | [engine/isolation-owned](../../../../Development/engine/isolation-owned/) dan histori Git; bukan implementasi engine mandiri |

Source produk, tests dan fixture native di atas sudah berada di repo sebelum
arsip ini. Jangan memakai salinan historis di bawah sebagai versi produk terkini.
Dua checkout build lokal juga diperiksa: tracked working trees bersih pada
`68ded21636ff486e973d757ce149f12ce05e1069` dan
`e2c80ada420b259ede58c2100d735a7bd73ff763`; keduanya sudah merupakan ancestor
HEAD `Local`, sehingga source-nya tersedia melalui histori Git. Tidak dibuat
mirror baru untuk source fork, dependensi, atau generated regression trees.

## Tambahan yang sebelumnya hanya lokal

Sebanyak **29 snapshot source**, dari file authored langsung di `workspace/`
dan `workspace/scratch/`, diarsipkan sebagai `.source.txt` agar mudah dibaca tanpa
menjadi entrypoint executable/build baru. Lingkup ini bukan pemindaian seluruh PC
atau isi direktori akun. Isinya:

- 8 harness browser/mock/UI lama; sebagian terhubung ke manager nyata dan dapat
  meminta launch. Selector, operasi, port atau asumsi readiness bisa sudah usang.
- 4 probe shell/window lama; sebagian mengubah foreground. Bukan acceptance
  renderer/input, dan bukan rekomendasi cara computer-use saat ini.
- 9 wrapper operasional driver/profile lama dan 1 skrip transformasi source.
  Wrapper itu menyimpan prosedur historis, **bukan** izin aktivasi atau migrasi
  ulang. Transformasi source lama tidak boleh dijalankan pada HEAD sekarang.
- 7 sumber scratch terkait WAM/provider, transport collector dan kompilasinya.
  Opsi broker internal/vtable eksperimen tidak menjadi kontrak API didukung,
  solusi produksi, ataupun bukti login/akun terisolasi.

Path mesin/repository dan lokasi Playwright privat diganti dengan placeholder;
line ending dinormalkan ke LF. [Manifest](source-inventory.json) menyimpan path
asal relatif, SHA-256 byte asli, SHA-256 snapshot serta redaksi. Hash asli adalah
provenance byte lokal, bukan bukti acceptance. Snapshot adalah teks untuk analisis;
**tidak ada skrip arsip yang dijalankan, diperbaiki atau divalidasi ulang** dalam
batch upload ini. Tulisan PASS di dalam kode merupakan kondisi tes historis,
bukan hasil tes baru atau klaim bahwa source masih cocok dengan produk sekarang.

| Asal lokal | Snapshot untuk dibaca |
| --- | --- |
| `workspace/activate-and-review.ps1` | [activate-and-review.ps1.source.txt](sources/activate-and-review.ps1.source.txt) |
| `workspace/adopt-existing-tabs.cjs` | [adopt-existing-tabs.cjs.source.txt](sources/adopt-existing-tabs.cjs.source.txt) |
| `workspace/authorized-runtime-update.ps1` | [authorized-runtime-update.ps1.source.txt](sources/authorized-runtime-update.ps1.source.txt) |
| `workspace/check-native-open.cjs` | [check-native-open.cjs.source.txt](sources/check-native-open.cjs.source.txt) |
| `workspace/check-native-workspace.cjs` | [check-native-workspace.cjs.source.txt](sources/check-native-workspace.cjs.source.txt) |
| `workspace/check-shell-focus.ps1` | [check-shell-focus.ps1.source.txt](sources/check-shell-focus.ps1.source.txt) |
| `workspace/cycle-four-tabs.cjs` | [cycle-four-tabs.cjs.source.txt](sources/cycle-four-tabs.cjs.source.txt) |
| `workspace/cycle-two-tabs.cjs` | [cycle-two-tabs.cjs.source.txt](sources/cycle-two-tabs.cjs.source.txt) |
| `workspace/foreground-fact.ps1` | [foreground-fact.ps1.source.txt](sources/foreground-fact.ps1.source.txt) |
| `workspace/inspect-managed-shell.ps1` | [inspect-managed-shell.ps1.source.txt](sources/inspect-managed-shell.ps1.source.txt) |
| `workspace/owned-runtime-activate-run.ps1` | [owned-runtime-activate-run.ps1.source.txt](sources/owned-runtime-activate-run.ps1.source.txt) |
| `workspace/owned-runtime-client02-config-run.ps1` | [owned-runtime-client02-config-run.ps1.source.txt](sources/owned-runtime-client02-config-run.ps1.source.txt) |
| `workspace/owned-runtime-restore-run.ps1` | [owned-runtime-restore-run.ps1.source.txt](sources/owned-runtime-restore-run.ps1.source.txt) |
| `workspace/owned-test-boot-run.ps1` | [owned-test-boot-run.ps1.source.txt](sources/owned-test-boot-run.ps1.source.txt) |
| `workspace/owned-test-prepare-run.ps1` | [owned-test-prepare-run.ps1.source.txt](sources/owned-test-prepare-run.ps1.source.txt) |
| `workspace/post-reboot-activate.ps1` | [post-reboot-activate.ps1.source.txt](sources/post-reboot-activate.ps1.source.txt) |
| `workspace/prepare-four-tabs.cjs` | [prepare-four-tabs.cjs.source.txt](sources/prepare-four-tabs.cjs.source.txt) |
| `workspace/probe-live-tabs.cjs` | [probe-live-tabs.cjs.source.txt](sources/probe-live-tabs.cjs.source.txt) |
| `workspace/review-profile-data-upgrade.ps1` | [review-profile-data-upgrade.ps1.source.txt](sources/review-profile-data-upgrade.ps1.source.txt) |
| `workspace/scratch/build-wam-native.cmd` | [scratch__build-wam-native.cmd.source.txt](sources/scratch__build-wam-native.cmd.source.txt) |
| `workspace/scratch/collector-runner.rs` | [scratch__collector-runner.rs.source.txt](sources/scratch__collector-runner.rs.source.txt) |
| `workspace/scratch/inspect-collector-transport.mjs` | [scratch__inspect-collector-transport.mjs.source.txt](sources/scratch__inspect-collector-transport.mjs.source.txt) |
| `workspace/scratch/probe-wam-provider.ps1` | [scratch__probe-wam-provider.ps1.source.txt](sources/scratch__probe-wam-provider.ps1.source.txt) |
| `workspace/scratch/SbieMessagesProbe.cs` | [scratch__SbieMessagesProbe.cs.source.txt](sources/scratch__SbieMessagesProbe.cs.source.txt) |
| `workspace/scratch/WamNativeProbe.cpp` | [scratch__WamNativeProbe.cpp.source.txt](sources/scratch__WamNativeProbe.cpp.source.txt) |
| `workspace/scratch/WamProviderProbe.cs` | [scratch__WamProviderProbe.cs.source.txt](sources/scratch__WamProviderProbe.cs.source.txt) |
| `workspace/unified-visual-check.cjs` | [unified-visual-check.cjs.source.txt](sources/unified-visual-check.cjs.source.txt) |
| `workspace/update_tab_controller.py` | [update_tab_controller.py.source.txt](sources/update_tab_controller.py.source.txt) |
| `workspace/window-facts.ps1` | [window-facts.ps1.source.txt](sources/window-facts.ps1.source.txt) |

## Hasil yang aman dibagikan

[appcontainer-observations.json](appcontainer-observations.json) menyalin hanya
field allowlist dari hasil positif dan kontrol negatif sintetis yang sudah ada.
Tidak ada SID, nama akun, path profil, token, dunia atau material signing. JSON
runtime aslinya tidak mengikat Git SHA secara kriptografis; batas provenance itu
tertulis di file. Percobaan tidak diulang untuk upload ini. Positive PASS berlaku
pada mekanisme dua fixture, sementara expected FAIL pada kontrol token-sama
membuktikan pemeriksaan dapat menolak kasus yang rusak. Empat akun Minecraft,
roaming isolation game dan gameplay tetap belum terbukti.

Ringkasan hasil desktop/runtime lama dan kegagalannya sudah ada pada
[native pilot report](../../../../docs/validation/native-pilot-localtest.md) dan
[independent isolation](../../../../docs/system/independent-isolation.md). Hasil dari
SHA lama tidak dinaikkan menjadi acceptance HEAD saat ini.

## Material yang tetap lokal

Tidak diunggah: credentials/token/cache akun, private worlds, VM disks/snapshots,
receipt mesin mentah, manager settings lokal, sertifikat/private keys, dump,
screenshot privat, log mentah, binary hasil kompilasi/installer, official game
binaries, downloaded toolchains, node_modules dan target/cache build. Generated
source/fixture copies yang dapat direproduksi dari source tracked tidak diduplikasi.
Artefak tersebut tidak diperlukan untuk membaca source; mengunggah seluruh
workspace tanpa seleksi justru dapat membocorkan data. File lokal asli tidak
dihapus atau dipindahkan oleh pengarsipan ini. GitHub Actions tetap mati.
