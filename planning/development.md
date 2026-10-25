# Current Development Handoff — M-Minecraft-Emulator

`Local` is the only product branch. This is the single current handoff.
The preceding efficiency state is preserved at
[32b54d031056](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/32b54d0310566ac7716bae2dca48a4bc45f57809/planning/development.md).

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

**Next action: none for this completed repair/pilot.** Any production-manager
integration, distribution, soak or renewed optimization is a separately bounded
follow-up, not automatic work from historical full-access permission.
