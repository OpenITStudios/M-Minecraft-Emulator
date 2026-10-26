# Current Development Handoff — Lazy Tester

Local only; main/publication untouched; GitHub Actions entirely OFF.
VMware, Sandboxie and RAM/renderer experiments remain PAUSED.

## Active goal — reliable Desktop startup with centralized application data

The user rejected repeated setup and premature completion claims. Current App
source is 09eaca9eb7c2a7593e316848be833d8179f3cb84, with executable hash
473a8363714ad5dcbadbee5427aa98d74f0c53b95cef2597271c68c691469541.
Desktop and Start Menu shortcuts target this App correctly. Updating the same
executable alone does not solve the observed storage-context failure.

Current source parent: ec929a97957aed029611dcb27e866a4189c6d825.
The previous handoff's completion claim is overturned by actual Explorer startup.

## Reproduction and confirmed discrepancy

verify-desktop-startup.ps1 dispatches the Desktop shortcut through the interactive
Explorer desktop's Document.Application, verifies Explorer parentage and reads the
actual UI with Windows accessibility APIs. Shell.Application alone or Start-Process
can inherit the development context; they were insufficient reproduction routes.

Executed RED with the installed App: PID38812, Explorer parent PID4764, five cards,
zero enabled, connection unavailable and Minecraft - 1: saved registry store cannot
be opened (Windows error 2). Exact receipt is ignored under
workspace/runs/play-settings-implementation/desktop-startup-red.log.

Read-only comparison: same Windows user SID hash, same logical native HKCU root
hash, neither elevated; development context sees1264 pilot subkeys, Explorer
context sees none in both32/64 views. Development CLI reports configured=true;
Explorer CLI reports absent binding. Canonical binding metadata is also present
under the development package's redirected LocalCache/Local directory. These are
separate effective HKCU/AppData views despite the same user and executable.
No registry values, account contents or worlds were inspected or printed.
The earlier predefined-HKCU regression demonstrates a separate source defect;
it did not establish the cause of the user's Desktop failure.

## Source correction and verification boundary

package-lazy-tester.mjs now creates the normal shortcuts, then requires real
Explorer/Desktop startup evidence, exact staged source/hash, Connected status and
five enabled launcher buttons. Builder-context initializer/hidden WebView checks
are no longer delivery readiness gates. Existing product diagnostic flags remain
available; missing registry stores still fail closed and are never recreated as
empty recovery. No runtime identity, gameplay, graphics or layout change is made.

A bounded opaque registry transfer was prepared privately and tested with a
new disposable fixture across the two process contexts. Preflight of the original
five stores passed: source handles readable, all five native destinations absent.
The fixture alone was transferred and removed. No original profile was transferred.
Private scripts/metadata remain ignored under workspace/private/desktop-registry-repair/.
Original binding and registry stores remain intact. No account/world export exists.

Permission was requested because CONTEXT.md prohibits copying tokens/profiles.
The user did NOT approve that transfer; instead they requested all application data
be centralized under the project root. Do not run the prepared Apply operation or
infer approval from the request for a central data layout.

## Latest direction — one authoritative project Data directory

User wants data needed by Lazy Tester together and understandable, rather than
scattered across AppData, registry contexts and arbitrary folders. Proposed target:
App for application binaries; Data for runtime settings, per-launcher data and
backups; Development/workspace for development only. One active source of truth,
not continuous bidirectional copies. Existing data is retained until any explicitly
approved migration has completed and actual Desktop reopening is verified.

A file-backed per-launcher registry hive would require changes in the existing
storage/bootstrap owner and actual registry isolation/reopen tests. It is a design
candidate, not implemented. Windows-managed Microsoft sign-in components and the
official Minecraft installation cannot be promised portable or relocated merely
by choosing a project Data folder. No additional ready-state/configuration store.

Next: finalize the central Data ownership and safe migration boundary with the user,
then implement the existing owner's storage changes and verify repeated Desktop
startup. The immediate connection issue remains UNRESOLVED pending that work.

## Preservation and other limits

Keep existing16:9 viewport/native-input rectangle, five-session dock, normal
1280x900 bounded window, High/Low presets and configurable144/10FPS defaults.
All games must be stopped for cache cleanup or manual graphics changes.
Full clear, world export, settings reset/restore, background audio, automatic game
shutdown and data migration remain unavailable. No game/installer GUI or sign-in
was automated. Updater was preserved; only verifier-owned managers closed normally.
Initialized submodule and second worktree remain untouched. Signing is NOT_VERIFIED;
installer execution/gameplay/license/multiplayer acceptance remain NOT_EXECUTED.
Publication remains DISABLED. Do not call this repair complete yet.
