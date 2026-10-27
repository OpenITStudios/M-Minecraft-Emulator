---
id: document.system.data-layout
class: DOCUMENT
domain: system
role: POLICY
authority: CANONICAL
lifecycle: ACTIVE
---

# Lazy Tester project data

The root native Lazy Tester.exe starts only App/lazy-tester.exe, with its working
directory set to App. It carries the application icon and no independent settings
or session authority. Desktop/Start Menu shortcuts use the same root entry point.
Data is resolved from the executable/project location, never the working directory.

```text
Minecraft-Emulator/
  Lazy Tester.exe
  App/                           application, two reviewed DLLs, build receipt
  Data/
    Cache/                       WebView data and process-local temporary files
    Settings/
      data-location.json         project-local location pointer
      performance.json           saved FPS limits
      graphics.json              five High/Low choices
      desktop-instance.json      exclusive manager marker
    profiles.json                private user/directory/hive file identities
    Launchers/
      Minecraft-1/
        Roaming/
        Local/Temp/
        Registry/HKCU.dat        private application hive and Windows sidecars
      Minecraft-2/ ... Minecraft-5/
  Development/                   canonical source/tooling
  workspace/                     developer builds, tools, cache and archives
```

All runtime files are ignored and never bundled or tracked. App refresh preserves
Data and the previous app/starter. project_profiles remains the existing
product_profiles.rs owner; manager_settings.rs owns root/preferences and the
original session controller alone owns games and admission. No parallel ready flag,
registry gateway or profile catalog is introduced.

Catalog schema2 binds the Windows-user digest, five directory identities and five
hive-file identities. Retained schema1 catalogs still use Profiles and ordinary
HKCU validation. Missing/replaced catalogs, directories and registry files fail
closed. Reopening checks that HKCU.dat exists before RegLoadAppKey can be called,
preventing accidental empty account stores. Data gets a protected user/System/
Administrators ACL before any profile contents are created.

The native enroller and compatibility DLL validate a nonsecret hive identity
marker against the retained private directory ID and slot. Foreign slot handles
are rejected before child code. Existing user/access checks and the private
storage ABI/version remain unchanged. Loader-lock initialization never opens or
loads a hive file; it checks metadata on the already inherited handle.

## Explicit migration

The fixed product-only --lazy-tester-migrate-data route requires the original
reviewed five-profile capability, supported image, reviewed payload, idle
controller and no games. It does not run from an installer hook or launch games.
Source trees are checked for links before creating target launchers.
Exception: an observed Local/Microsoft/Windows/INetCache/Content.IE5 directory
junction becomes an empty ordinary cache directory at the destination. Its
external target is never followed, copied or changed. Every other link is refused.
This preserves source account/world data and avoids importing global Windows cache.
CopyFileW copies the same slot's Roaming/Local files opaquely; SHCopyKeyW copies that slot's
registry values/subkeys while retaining the uniform destination hive descriptor.
No account values or world contents are printed, parsed or exported to logs.

Saved App preferences are copied into Data/Settings without replacing saved
settings. The complete catalog is published last and reopening is checked.
Original data remains intact. Existing/partial destinations are never cleared,
merged into replacement profiles or overwritten. Source preservation is recovery
protection, not another active or bidirectionally synchronized data location.
The earlier proposal to transfer into native HKCU remains unused; the user's
subsequent project-local Data instruction defines this migration's destination.

Old App location pointers are checked before fresh fallback. Outside-project
locations fail for explicit migration review rather than being ignored/reseeded.
New data selections stay under the project's Data subtree. Same-volume moves can
retain identities; arbitrary cross-volume copies are rejected. The explicit
reviewed migration creates and records new identities without mixing profiles.

Missing FPS settings default to 120/5; Save validates whole limits 5..1000,
background no higher than active. High/Low, atomic preference writes, next-launch
FPS application and all-games-stopped graphics/cache admission keep their owners.
Full clear replaces the selected verified local profile and hive after confirmation;
its recovery and isolation have synthetic fixture evidence only. Windows-managed
sign-in data is outside that operation, and a fresh account sign-in is not yet
verified. General backup/restore and cross-PC account portability remain absent.

Windows-managed Microsoft identity/DPAPI components, the licensed Minecraft
installation, Visual Studio/WDK and Git remain OS dependencies. Central Data does
not relocate them, clone accounts, bypass licensing or automate sign-in. Real
Minecraft sign-in, gameplay and multiplayer require their own observed evidence.
