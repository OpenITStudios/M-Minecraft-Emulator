---
id: document.system.data-layout
class: DOCUMENT
domain: system
role: POLICY
authority: CANONICAL
lifecycle: ACTIVE
---

# Lazy Tester application and data locations

New per-user installations default to the visible `%USERPROFILE%/Lazy Tester/App`.
The installer directory page permits another destination; a selected root receives
an App subdirectory. Registered existing flat installations are retained in place.
The application home is App's parent, or the executable's directory for an existing
flat candidate. The working directory is never used to find data.

```text
Lazy Tester/
  App/
    lazy-tester.exe
    independent-bootstrap/       two reviewed native DLLs
    distribution/               public package policy
    data-location.json          private runtime-created location pointer
  Data/
    profiles.json               private Windows-user/directory identities
    Profiles/
      Minecraft-1/              private Roaming, Local and Local/Temp
      Minecraft-2/
      Minecraft-3/
      Minecraft-4/
      Minecraft-5/
```

Data and the location pointer are never packaged. Uninstall preserves them; Data
is outside App in new installations. Backups and centralized Logs remain future
features, not automatically populated folders. Source checkouts ignore root
App/Data/Logs and structure checks reject tracking them. Developer output remains
governed by [artifact storage](artifact-storage.md).

## One product owner

`apps/desktop/src-tauri/src/engine/independent_bootstrap/product_profiles.rs` owns
location resolution, catalog validation, fresh store preparation and reopening.
The existing session controller alone owns live games, handles and launch policy.
The new catalog reuses the bootstrap's filesystem/HKCU constructor. The DLL ABI,
reviewed DLL bytes and original profile data are unchanged.

The relative default pointer `Data` follows the application home when the whole
folder is moved and its executable is opened from the new location. This does not
repair Windows shortcuts or installer registration after a manual application move.
A selected data folder is saved as an absolute path. Storage shows the actual root
and five profile paths, with Open folder, Locate data and initial Choose data folder
actions. The mounted UI and new errors are English.

## Setup and preservation

Startup first reopens product data. Without product data, the reviewed old pilot
can be discovered using bounded directory metadata/identity checks and adopted in
place, without finding developer launcher folders. Its original reviewed digest
and existing HKCU validation still apply. Invalid existing bindings are preserved,
never replaced with five empty profiles.

Only when neither product data nor a retained old binding exists may fresh setup
create Data/Profiles/Minecraft-1..5. It requires the supported registered Minecraft
image, reviewed DLLs, supported AppData layout and idle controller/game set. Each
slot receives its own storage and HKCU store. No game starts automatically; login
remains Minecraft-owned.

Catalog publication is create-once, after all five stores exist. Reopening validates
the current Windows-user digest, root/profile file identities, ordinary ancestors,
fixed slot paths, existing directories and HKCU stores; no missing store is recreated.
A valid catalog without its pointer repairs the pointer before admitting a game.
Corrupt metadata, unavailable saved data and changed identities fail closed. Partial
setup remains preserved for recovery, not automatically deleted/reseeded; an
interactive partial-setup recovery flow remains unfinished.

## Relocation support and limits

Locate data verifies an existing product Data directory before changing the pointer.
Same-volume moves by the same Windows user can retain the directory identities and
registry stores. Location changes refuse while Minecraft or its controller is active.
Dialog cancel changes no pointer; Choose cannot replace existing profile data.

This is location persistence and verified reattachment, not a Move data copy engine.
Cross-volume copies change identities and are rejected. Moving old pilot profiles
into the new layout, per-slot moves, backup/restore and destructive reset remain
unimplemented. No token export or account cloning is introduced.

Current payload limitations remain: bounded ASCII paths, ordinary local directories,
unsplit supported AppData and the exact reviewed Minecraft executable. Fixture and
installer-build results do not prove installed behavior, fresh-profile login,
gameplay, cross-PC portability or cross-volume account persistence.
