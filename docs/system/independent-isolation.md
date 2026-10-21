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
