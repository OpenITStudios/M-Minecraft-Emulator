---
id: document.validation.lazy-tester-test-cases-experimental
class: DOCUMENT
domain: validation
role: PROCEDURE
authority: WORKING
lifecycle: ACTIVE
---

# Lazy Tester — matriks uji MCP, AI input, visual evidence, Skills Probe

**Status seluruh kasus: NOT_EXECUTED.** Ini adalah rencana untuk
pengujian yang **masih memerlukan izin dan binary terpisah**, bukan
perintah menjalankan native hooks, Minecraft Education, AI control,
game-world commands atau CI. Baca [panduan induk](lazy-tester-test-master.md)
dan [AI Computer Use local test](ai-computer-use-localtest.md) dahulu.

**Batas:** MCP adalah local stdio adapter; satu retained Rust native
controller tetap pemilik semua game/process/input. Source/Vitest/Rust
fixtures, lima PID, IPC ACK, worker frame sequence maupun PNG **bukan**
bukti Minecraft menerima input. Standard Lazy Tester mengecualikan
input eksperimen (compile-OFF/default-OFF); reviewed AI DLL khusus belum
memenuhi hash admission, dan binary standar **tidak** boleh digunakan
sebagai pengganti. Perlu approval terpisah untuk build C++/ABI, input
live, gambar, player/admin world, dan semua akun Education. Jangan
mengubah source hash untuk membuat fitur yang diblokir tampak siap.

Mode **human-plus-four-ai**: manusia MC01 terlindungi, AI hanya MC02–MC05.
Mode **five-ai** adalah pilihan eksperimen yang berbeda dan eksplisit.
Mereka tidak boleh tercampur. Skenario di bawah menggunakan simulator/
synthetic test fixtures lebih dulu; live game baru sesudah gate khusus.

## 1. Gate X-A sampai X-F

| Gate | Prasyarat | Jika tidak terpenuhi |
| --- | --- | --- |
| X-A | Izin source tests, Node/Rust toolchain pinned dan SHA `Local` | Tests NOT_EXECUTED |
| X-B | C++/Rust experimental ABI, exact feature/source dan reviewed AI DLL digests | BLOCKED; **bukan** menggunakan reviewed standard DLL |
| X-C | Izin game input + world uji + 2 akun licensed | Tidak mengirim movement atau hooks |
| X-D | MC01 human + MC02 AI live movement, relative camera, attack/use, ReleaseAll dan human isolation benar-benar PASS | Tidak boleh eskalasi 4/5 AI |
| X-E | Consent 4/5 AI, 4/5 licensed accounts dan world yang diizinkan | Tetap hanya source/game manual |
| X-F | Approved original team Skills Probe ZIP/locks, admin Player dan owned QA world | QA source-only, gameplay NOT_VERIFIED |

Untuk detail feature, rujuk
[controlled AI test](ai-computer-use-localtest.md),
[MCP source](../../Development/tooling/agent/README.md),
[map QA](../system/map-aware-probe-qa.md),
[team source review](../system/education-team-source-updates.md)
dan [diagnostic archive](../system/map-diagnostics.md).
Jangan menganggap `DEV.cmd package` mengaktifkan input AI.

## 2. MCP default dan native ownership — source/test fixtures

| ID | Prio | Skenario | Expected / bukti |
| --- | --- | --- | --- |
| MCP-01 | P0 | MCP initialize/tool list default | Local stdio, tidak auto game, tidak membuka server HTTP |
| MCP-02 | P0 | Tool list tanpa opt-in | Tidak ada input experimental yang silently enabled |
| MCP-03 | P0 | Hybrid mode | MC01 human tidak boleh jadi AI target |
| MCP-04 | P0 | Five-ai opt-in mode | Hanya mode terpisah ini mengizinkan MC01 AI |
| MCP-05 | P0 | Dua experimental input modes sekaligus | Refuse; tidak dua controller |
| MCP-06 | P1 | `inspect_profiles` | Hanya lima status allowlisted, no account/world info |
| MCP-07 | P1 | `get_test_context` | Read-only native context, bukan Minecraft sign-in PASS |
| MCP-08 | P0 | `get_computer_use_target` MC01 hybrid | Refuse sebelum native capture/input |
| MCP-09 | P0 | `revalidate_computer_use_target` stale PID/HWND | Refuse, tidak retarget child baru |
| MCP-10 | P0 | Process birth berubah, PID tetap | Native identity mismatch |
| MCP-11 | P0 | Windows session berubah | Native identity mismatch |
| MCP-12 | P0 | MCP disconnect/restart dengan input lama | No automatic input replay; reconciliation hold |
| MCP-13 | P0 | Simultaneous mutator requests fixture | Exclusive admission, bukan dua native owners |
| MCP-14 | P0 | Pending IPC request timed out | Old worker/uncertain state retained |
| MCP-15 | P0 | Invalid MCP args/schema | Bounded sanitized error tanpa raw token/path |
| MCP-16 | P1 | `agent_capabilities` tanpa source report | UNKNOWN, tidak menyimpulkan no AI activity |
| MCP-17 | P0 | Explicit activity publication | UI hanya mirror unverified status |
| MCP-18 | P0 | MCP shutdown | Tidak kill Minecraft, tidak hapus private data |
| MCP-19 | P0 | Native lease expired/mismatched epoch fixture | Stale CLI tidak mengontrol game pengganti |
| MCP-20 | P0 | Wrong slot/native process owner | Refuse no cross-client action |

## 3. Visual capture dan before-action PNG

Native original target harus cocok pada PID, HWND/windowId,
process birth dan Windows session **sebelum dan sesudah capture**.
Metadata read saja tidak berarti AI menerima PNG. PNG yang diambil
sebelumnya harus revalidated lagi ketika akan dikirim atau digunakan
sebagai dasar observed action. Seluruh capture tetap bounded/volatile.

| ID | Prio | Skenario | Expected / bukti |
| --- | --- | --- | --- |
| VIS-01 | P1 | `start_visual_observation` slot AI | Bounded sampler, no global screenshot / game input |
| VIS-02 | P0 | Request visual MC01 hybrid | Human MC01 protected |
| VIS-03 | P0 | Cua/GPU provider tidak configured | NOT_CONFIGURED, bukan synthetic image |
| VIS-04 | P1 | `get_multi_client_visual_state` metadata-only | No screenshot baru, no native input |
| VIS-05 | P0 | Metadata fresh tanpa PNG delivery | Source-ready saja, bukan observedActionEligible |
| VIS-06 | P0 | Explicit `read_visual_observation` includeImage | Native PID/HWND/birth/session revalidated sebelum PNG dikirim |
| VIS-07 | P0 | Original process replaced sebelum PNG read | Revoke obsolete cache hanya target slot |
| VIS-08 | P0 | Native target unavailable | Tidak mengirim cached PNG, fail closed |
| VIS-09 | P0 | Process berubah selama capture | Before/after target mismatch → screenshot discarded |
| VIS-10 | P0 | Frame maju selama awaited native check | Hanya exact verified sample dapat delivered |
| VIS-11 | P0 | Failed recheck tidak boleh menandai PNG “already delivered” | Delivery marker tidak tertulis untuk rejected frame |
| VIS-12 | P0 | WGC native worker sequence diulang | Tidak menghitung sample baru |
| VIS-13 | P0 | WGC source timestamp/sequence mundur | Refuse worker/frame invalid |
| VIS-14 | P0 | Cua frame terlambat > cached age policy | Not current; tidak promosi ke before/after evidence |
| VIS-15 | P0 | Satu slot replaced saat empat/five observers aktif | Peer identity/PNG tidak ikut dibersihkan |
| VIS-16 | P0 | Explicit stop/expiry | Cache dan delivered marker dihapus |
| VIS-17 | P0 | New frame after old PNG delivered | Old delivery receipt tidak berlaku pada current new frame |
| VIS-18 | P0 | Model transcript XYZ tanpa PNG current | Refuse, no live F3 verification |
| VIS-19 | P0 | PNG byte sama di dua sample | Bukan bukti freeze atau success/failure action |
| VIS-20 | P1 | Multi-client source clock/producer sequencing | Per-slot sample/identity terpisah |

## 4. Observed action source gate dan after-action evaluation

**Perlu diketahui:** `run_experimental_observed_actions` harus menerima
exact per-slot visual basis yang telah **delivered sebagai PNG**, cocok
dengan native identity **sebelum dan setelah async preflight**, dan masih
layak di original controller. Bahkan setelah lolos, native/game input
masih hanya dapat dicap sebagai **transport ACK**, bukan game PASS.

| ID | Prio | Skenario | Expected / bukti |
| --- | --- | --- | --- |
| OBS-01 | P0 | Observed actions tanpa explicit opt-in | Refuse |
| OBS-02 | P0 | Semua cached frames metadata-only | TEAM_OBSERVATION_IMAGE_NOT_DELIVERED; no lease/input |
| OBS-03 | P0 | Satu dari 4 selected slots belum delivered PNG | Refuse seluruh cohort, tidak silently skip |
| OBS-04 | P0 | Satu dari 5 selected slots belum delivered PNG | Refuse seluruh cohort |
| OBS-05 | P0 | Semua current PNG delivered | Bisa lanjut ke native identity admission **saja** |
| OBS-06 | P0 | Baseline caller source sequence stale | TEAM_OBSERVATION_STALE_OR_REPLACED |
| OBS-07 | P0 | Baseline PID/HWND/birth/session mismatch | No writer lease/input |
| OBS-08 | P0 | Timestamp source missing/regressed | No observed action dispatch |
| OBS-09 | P0 | Native target berubah sebelum dispatch | Refuse, revoke old slot cache |
| OBS-10 | P0 | Native target unavailable saat preflight | Refuse, no optimistic cached fallback |
| OBS-11 | P0 | Frame maju selama awaited preflight | Recheck baseline menolak |
| OBS-12 | P0 | Sampler satu slot revoked saat action active | Whole cohort abort, recovery hold |
| OBS-13 | P0 | Nonbusy capture failure saat action | Abort, tidak blind retry |
| OBS-14 | P0 | One client unhealthy, peer sehat | Tidak otomatis mengecilkan cohort; requires explicit replan |
| OBS-15 | P0 | Writer lease conflict/old epoch | Original controller menolak |
| OBS-16 | P0 | IPC ACK tanpa observation Minecraft | Tetap NATIVE_TRANSPORT_ONLY_NOT_GAMEPLAY_EVIDENCE |
| OBS-17 | P0 | Neutral ACK uncertain | Hold sampai exact-source recovery |
| OBS-18 | P0 | Post-action PNG belum delivered tetapi model klaim perubahan | NOT_EVALUABLE, no positive visible-change count |
| OBS-19 | P0 | After PNG process replaced | NOT_EVALUABLE per slot, peer unaffected |
| OBS-20 | P0 | Delayed after frame source timestamp sebelum action finish | Tidak diakui newer post frame |
| OBS-21 | P0 | Same PNG bytes, model klaim movement | Report conflict, bukan movement proof |
| OBS-22 | P1 | No external model review diberikan | UNDETERMINED_NOT_REVIEWED |
| OBS-23 | P0 | Model review kategori/slot yang tidak diminta | Refuse, no foreign slot action |
| OBS-24 | P0 | Malformed/oversized model cue | Bounded refusal, no raw external text trusted |
| OBS-25 | P0 | MCP restart dan stale baseline | Old frame/ownership tidak dikembalikan |
| OBS-26 | P1 | Group summary tanpa images | No automatic image interpretation |
| OBS-27 | P0 | Before PNG delivered namun proses berganti sesudah capture | Live native recheck mendahului any writer lease |
| OBS-28 | P0 | Normal package active tanpa AI feature | Observed input tidak enabled oleh default |

## 5. Actual AI game input — khusus setelah approved AI binary dan izin

Jangan pernah mengaktifkan hook C++/native input pada regular build.
Review `REVIEWED_EDUCATION_AI_BOOTSTRAP`, matching C++/Rust feature/ABI,
native x64 dan per-client owner terlebih dahulu (lihat [AI test guide](ai-computer-use-localtest.md)).
Synthetic ABI compile, CLI ACK dan png frames bukan Minecraft movement.
Satu game human dan satu AI dulu; baru empat lalu lima AI dengan explicit consent.

| ID | Prio | Live scenario | Expected dari actual game |
| --- | --- | --- | --- |
| AI-01 | P0 | Matching feature/ABI/digest admitted | Tidak memakai default standard DLL sebagai AI test |
| AI-02 | P0 | MC01 human + MC02 AI | Human physical input pada MC01 tetap |
| AI-03 | P0 | AI MC02 movement | MC02 benar-benar bergerak, bukan hanya ACK |
| AI-04 | P0 | MC02 relative camera look | Camera target bergerak tanpa focus steal |
| AI-05 | P0 | MC02 jump/sneak/attack/use | Actual game outcome, bukan hanya animation screenshot |
| AI-06 | P0 | ReleaseAll/key-up | Tidak terus berjalan/menyerang ketika neutral |
| AI-07 | P0 | MC02 exit/relaunch saat active lease | Old input tidak menyasar replacement |
| AI-08 | P0 | Lost foreground/failed native identity | Refused safely, MC01 tidak terdampak |
| AI-09 | P0 | TTL/deadline expired, IPC uncertain | No unbounded held game input |
| AI-10 | P0 | Native neutral ACK hilang | Recovery hold, tidak auto-replay |
| AI-11 | P0 | Hybrid MC02–MC05 empat AI | Semua channel terikat per slot, MC01 human tetap |
| AI-12 | P0 | Five-ai mode explicit untuk MC01–MC05 | Tidak menyamakan consent hybrid/five-ai |
| AI-13 | P0 | Satu client drop ketika 5 active | Cohort hold/cancel; tidak silently continue |
| AI-14 | P0 | Stale CLI transaction epoch | Lease fencing menolak old writer |
| AI-15 | P1 | 4/5 clients join approved test world | Independent gameplay/multiplayer benar-benar diamati |
| AI-16 | P1 | PNG→reasoning→action→PNG | Semua source images validated/delivered; Minecraft effect terpisah |
| AI-17 | P0 | Physical human input saat AI peers bekerja | Tidak cross-slot, tidak merebut mouse/keyboard user |
| AI-18 | P0 | MCP restart while movement unresolved | No auto input, explicit neutral/release |
| AI-19 | P1 | Concurrent multi-slot timing | Per-slot action timestamps, errors dan impact dicatat |
| AI-20 | P0 | Shutdown semua experiments | Tidak ada stuck keys atau hooks pada standard package |

## 6. Skills Probe/Team source, map-aware QA, diagnostics

Tim Skills Probe adalah pemilik real in-game command dan foundation
results. Lazy Tester hanya mempersiapkan/review source dan ringkas report.
Tanpa approved ZIP/lock, admin Player/world dan verified live log,
`PROBE_REPORTED_PASS` tetap bukan map gameplay PASS.
Rujuk [map QA](../system/map-aware-probe-qa.md),
[team source update](../system/education-team-source-updates.md),
[diagnostics](../system/map-diagnostics.md).

| ID | Prio | Skenario | Expected / bukti |
| --- | --- | --- | --- |
| QA-01 | P0 | `probe_qa` tanpa approved ZIP | BLOCKED, bukan auto install/QA PASS |
| QA-02 | P0 | Team ZIP lock mismatch/different source | Refuse; no automatic module copying |
| QA-03 | P1 | First `probe_qa` source/capture preflight | Bounded checkpoint, no Player command sent |
| QA-04 | P0 | Admin Player/world not verified | Automatic dispatch DISABLED |
| QA-05 | P1 | Manual original Probe execution with approval | Original START/RESULT/DONE marker from team |
| QA-06 | P0 | Incomplete/rotated/malformed Probe log | UNKNOWN/partial, bukan full PASS |
| QA-07 | P0 | Original Probe all-PASS reported | PROBE_REPORTED_PASS, not Minecraft/map PASS |
| QA-08 | P1 | Repeat checkpoint read | No duplicate Probe run |
| QA-09 | P0 | Competing MCP owner reserves same Testing slot | Original native owner refuses takeover |
| QA-10 | P0 | `probe_qa inspectRun` after reconnect | Read-only correlation, token equality not operator proof |
| QA-11 | P0 | Orphan RESERVED run | No automatic transfer/forced reset |
| QA-12 | P1 | Approved immutable Data/Testing archive | Checksum and run correlation, no raw world/token |
| QA-13 | P0 | Map-aware JSON plan | All unexecuted map checks NOT_TESTED |
| QA-14 | P0 | Foundation Probe PASS but map checks missing | No map gameplay acceptance |
| QA-15 | P0 | Explicit Data/Diagnostics import | Private/local, no automatic capture of Education logs |
| QA-16 | P1 | Issue candidates/detail cursor | Bounded sanitized output |
| QA-17 | P1 | Team source update inspection | Changes reviewed only, no auto sync |
| QA-18 | P1 | Data-only `/function help` pack | Displays team commands, no second telemetry engine |
| QA-19 | P0 | Optional desktop status/report mirror | Clearly unverified, no account/world claims |
| QA-20 | P0 | MCP process exits/restarts | Stale report not presented as newly verified |

## 7. Cara mengartikan hasil

- Source tests benar-benar PASS hanya pada **source execution dan SHA**
  yang benar. Jangan menandai real Windows/Minecraft acceptance.
- `get_multi_client_visual_state` adalah **metadata-only**; tidak
  menjadi izin input hanya karena sample fresh. PNG delivery adalah
  syarat provenance, **bukan bukti model benar-benar memahaminya**.
- Post-frame yang tidak cocok PID/birth/HWND/session, belum delivered,
  atau timestamp tidak valid = **NOT_EVALUABLE**, bukan GAME FAIL/PASS.
- Native CLI writer ACK = bukti transport, bukan game movement. Periksa
  movement/camera/attack/use dan key release langsung di Minecraft.
- Jangan menampilkan raw account credentials, world names, private log,
  process command lines, atau tokens di GitHub/public report.
- Jika MC01 human menerima unwanted input, proses salah ditargetkan,
  game input tidak netral atau DLL/ABI tidak verified, **STOP semua
  pengujian eksperimen** tanpa force-kill.

Gunakan form hasil di [panduan induk](lazy-tester-test-master.md).
**Semua kasus di atas NOT_EXECUTED saat dokumen ditulis.**
