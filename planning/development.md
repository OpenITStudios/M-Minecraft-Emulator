# Current Development Handoff — M-Minecraft-Emulator

`Local` is the only product branch. This is the single current handoff.
The preceding efficiency state is preserved at
[32b54d031056](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/32b54d0310566ac7716bae2dca48a4bc45f57809/planning/development.md).

## Current bounded work — automatic approved graphics preset

The operator requested that Minecraft3–5 receive the same preset as1–2 without
Computer Use. Read-only inspection confirmed the original pair already had
Simple/12 chunks/MSAA1/effects off, while3–5 still had Fancy/45 chunks/MSAA2.
This request resumes only approved-setting automation, not paused renderer/RAM
experiments. No quantitative resource saving is claimed.

The existing bootstrap now invokes its private
`independent_bootstrap/game_preset.rs` helper after exact-image/profile validation
and before creating any game processes, on both pair/five replay routes. It
refuses a running Minecraft, prepares every profile before replacing any file,
and enforces only the eight previously approved entries: Simple,12 chunks,
MSAA1 and the five decorative effects off. `options.txt` remains the sole settings
store; other bytes/settings, simulation, resolution, input and accounts are
preserved. Duplicate owned keys are made consistent; existing line endings and
UTF-8 BOM are retained. Unsupported encoding/NUL/oversize input fails closed.

Only changed files receive a unique `options.mivubi-*.bak` in their own private
options directory. Replacement is staged, flushed and renamed in the same
directory; files/ancestors are checked/pinned and reparse files are refused.
Unchanged presets are a no-op. Absent profiles' fixed directories/options can be
seeded; actual first-ever game consumption of a minimal options file is still
NOT_EXECUTED. A multi-profile I/O failure stops startup; already completed
per-file replacements are safe to retry, not a cross-profile transaction.

Three targeted Windows Rust tests passed, covering preservation/BOM/duplicates/
idempotence, invalid/oversize rejection, actual file replacement/backup/new-file
seeding, writer exclusion and refusing a replaced source file. Preview-manager
release build passed (104 existing warnings). New manager SHA256:
`03c93d0ab6a09486881f7d5438565e56fadfab62c91bd1bc87fa6ebfb14862df`.
The accepted login-fix DLL/enroller and five profile capability are unchanged.
The standard private five-client launcher is prepared for this candidate.

**Runtime application: NOT_EXECUTED while five games remain running.** The
normal-save/close question is pending; do not force-close or edit live settings.
No Computer Use, login automation, global browser/system changes or CI.

## Five-client native pilot — DONE within this scope

The operator requested five Minecraft Education clients on25 October2026.
The registry-fix candidate ran Minecraft - 1 through Minecraft - 5; the operator
confirmed all five distinct licensed accounts signed in together in the same
**Education Host World**, not a separate LAN server. After a normal host-world
save and five normal exits (each journal exitCode0), the same five profiles were
replayed. The operator confirmed all five accounts returned without password
entry, the saved world remained available, and FPS counters/active144-background10
behavior remained correct while switching windows. No account/token/world
content was inspected to infer this acceptance.

This closes the bounded five-client native pilot and blank-login repair.
It does not certify indefinite crash freedom, long soak, resource benchmarks,
installer/distribution or five-slot production-manager UI. The existing four-tab
public observation contract and isolated Launch/Provision admission remain
unchanged. Five-client startup currently uses the fixed private CLI candidate.

## Source owner and preserved data

`apps/desktop/src-tauri/src/engine/independent_bootstrap.rs` remains the sole
private preparation/launch/storage owner, reused for count-bound2/5 operation.
New private fixed routes: `--education-prepare-five-profiles-once` and
`--education-five-startup-observation-once`. The original pair route is retained;
extra arguments refuse. No second account/lifecycle manager or public readiness.

Explicit preparation reopens/pins the original216-byte pair capability and its
identities, creates only three empty additional stores under the same profile
root, and writes a separate288-byte/version2 five-slot capability. Completed
reruns reopen the same stores. Interrupted partial creation refuses rather than
adopting/deleting/resetting data. Config stays version2/1136bytes; slots1–5 have
distinct A–E registry/singleton suffixes. Roaming/Local/TEMP/HKCU remain per-slot.
Original accounts/worlds and approved render>=12/simulation/resolution settings
are preserved. No account cache/token/world copying or game-file modifications.

## Blank login — cause and correction

Fresh3–5 sign-in initially showed a white window. The loaded browser path was
Windows MSHTML/WinINet; Chromium removal was not established as its cause.
Public discovery HTTPS returned200 in both own-process private and unmediated
controls, giving no basis for TLS/security downgrades or copying browser settings.

An initial folder-only candidate still failed the operator's login check. Its
valid API corrections are retained: existing-directory root FILE_CREATE returns
Windows NAME_COLLISION/183 rather than NOT_SUPPORTED/50; selected Known Folder
CREATE creates/verifies private Roaming/Local children while preserving LastError,
foreign-user rejection and reparse checks. These alone did not fix sign-in.

The narrower offline HTMLDocument control reproduced E_UNEXPECTED with private
HKCU, while unmediated Windows and file-only mediation succeeded. Scoped metadata
in this synthetic process identified rejected opens of Internet Settings\Zones\
and Lockdown_Zones\. The registry router reused filename validation and rejected
a terminal separator that Windows accepts. It now validates the preceding
components and forwards the original counted registry name. Filesystem rules,
interior-empty/parent-component restrictions, peer-store rejection and no host
fallback remain unchanged. No broad browser configuration seeding was needed.

The configured-storage self-test now creates a real offline MSHTML document and
checks its body in both private slots/generations. It failed before this registry
correction and passes afterward. Native/Win32 trailing-key opens also check
private values and LastError. These controls perform no authentication/network/UI;
actual Minecraft sign-in was separately confirmed by the operator afterward.

## Evidence and provenance

Source parent: `32b54d0310566ac7716bae2dca48a4bc45f57809`.
C++ MSVC /W4 /WX builds, configured-storage/MSHTML and mutex regressions, and
slot1–5/A–E/bounds control passed. Rust five-slot tests2 and original-pair replay
regression1 passed during expansion. Final frontend Vite build passed. Preview
release manager builds passed with104 existing warnings; these are source builds,
not installer verification. Both heap experiments remain OFF.

Actual accepted runtime payload SHA256:

- Manager: `e860054ff9de4d040380716df497c9cf71f5cd446263e8552fad8f3e46556480`
- Compatibility DLL: `f65f263b6cbf76054a86d67e50ba4d02d3c5b15fc98a70e627ea19a73eab2edf`
- Enroller: `73f9038364b2b853a50e6d058613049234568fccd054ce9a758f572531707368`

Five named responding windows loaded the pinned compatibility DLL. Runtime
acceptance applies to these actual payloads and operator reports. During final
consumer review an unused expansion of the public observer/parser was removed:
the production tab consumer still supports four IDs, so returning five there
would be incomplete. That GUI path was not exercised by the accepted fixed CLI;
five-tab production UI and exact published full-GUI runtime are NOT_EXECUTED.
The bootstrap/login DLL used for acceptance is unchanged by that source cleanup.

RAM/renderer efficiency work stays **PAUSED** by operator request. CI remains
entirely off; VMware/Sandboxie stay retired. Computer Use was stopped with Esc;
no further screen/input control was used. Do not force-close games or automate
authentication. Last accepted games are left open; do not close them incidentally.

**Next:** once the operator confirms normal game closure, launch the prepared
five-profile candidate and verify the eight approved settings in each profile.
If restart is deferred, leave current games/settings untouched; the prepared
launcher applies the preset on the next ordinary launch.
