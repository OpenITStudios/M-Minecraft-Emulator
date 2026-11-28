# Current Development Handoff — Lazy Tester

## Current status — macOS development PAUSED (operator decision)

Branch `Local`; Unreleased. Target: one native Lazy Tester for Windows and
macOS, retaining up to five independent licensed Education profiles
(MC01–MC05), one official installation per host, one UI, one retained
session authority, one catalog and one MCP/SDK operation core. ZLR-04 and
all native, execution and release holds remain. DESIGN_REVIEWED;
native launch remains BLOCKED at Mac profile isolation; Mac source work PAUSED. MAC-02 candidate provenance
and mechanism review recorded below. Not SOURCE_READY_FOR_LOCAL.

### MAC-HOLD — operator pauses macOS development

Operator explicitly closes the Mac workstream for now because a Mac is
not accessible. Plan A only is retained as historical selected direction;
do not resume Mac work automatically on a generic continuation request.
Resume only when the operator explicitly reopens this workstream.
Preserve all source, fixtures, evidence, unresolved game storage/account
isolation and main executable artifact limitations. Mac runtime admission
remains false. No rollback, runtime edit, packaging change or release.

Windows preservation audit: compare pre-Mac convergence base
10ed48c6ce6592ccef680623ffc643e8cd6aa9ec to
e5c014b79f500aa0c5048223c966f47f2ce244c6 (39 commits, 39 changed paths),
including the last 11 commits since b5c8b5c2a52ec27242be301b1a353793d8de0232.
Reviewed the shared controller/IPC, capability, identity, UI scheduling,
icon, package and dependency diffs. Windows source is not byte-identical:
status projection/close selection/slot validation moved to shared core,
UI capability predicates still accept exactly WindowsNative, and window
identity comparison/selection keys preserve their four original fields.
Windows installation report retains the untagged original wire shape;
new startup capability is Mac cfg-only. Mac-specific dependencies and
workspace module are cfg-gated, with changes to shared Cargo.lock;
Windows dependency declarations/target, package:windows, existing
Windows icon requirements, native DLL sources, bootstrap profile
storage/authentication, DWM and installer config were not changed in
the comparison. The source verification suite includes an additional
offline Mac metadata fixture; this does not activate a Mac game backend.
Source review found no intended Windows behavioral change; build, lock
resolution, regression and native behavior NOT_EXECUTED, not a parity
or compatibility proof. Windows work can continue independently when
requested; all existing test/CI/release holds remain. Mac status PAUSED.

### MAC-13b — current Swift Foundation corroborates fixed-home candidate

Pinned original swiftlang/swift-foundation revision
72996dbdde1147dd142f5b2ce4c7998c689ba9bb. Unlike the earlier historical
CoreFoundation-only evidence, current
[String+Path.swift](https://github.com/swiftlang/swift-foundation/blob/72996dbdde1147dd142f5b2ce4c7998c689ba9bb/Sources/FoundationEssentials/String/String%2BPath.swift#L351-L413)
checks CFFIXED_USER_HOME through Platform.getEnvSecure before resolving
the UID home, then HOME as fallback; the
[Platform.swift](https://github.com/swiftlang/swift-foundation/blob/72996dbdde1147dd142f5b2ce4c7998c689ba9bb/Sources/FoundationEssentials/Platform.swift#L258-L302)
non-Windows/Darwin branch checks issetugid before getenv.
The [Darwin search-path sequence](https://github.com/swiftlang/swift-foundation/blob/72996dbdde1147dd142f5b2ce4c7998c689ba9bb/Sources/FoundationEssentials/FileManager/SearchPaths/FileManager%2BDarwinSearchPaths.swift)
uses sysdir; [search-path expansion](https://github.com/swiftlang/swift-foundation/blob/72996dbdde1147dd142f5b2ce4c7998c689ba9bb/Sources/FoundationEssentials/FileManager/SearchPaths/FileManager%2BSearchPaths.swift#L89-L117)
expands tilde using the path helper. This corroborates a plausible
ordinary Foundation user-domain Application Support route. Counterexample:
the FOUNDATION_FRAMEWORK shared-user branch deliberately resolves real
home through the UID rather than ordinary tilde expansion. No claim
that this upstream revision is the Foundation binary on the target Mac,
that LaunchServices preserves every variable, or that the game chooses
these APIs. Apple NSHomeDirectory public contract still describes actual
user/app home, not a supported per-process isolation guarantee.

Original Education bundle resources previously inspected in MAC-13b
research: edu_worlds_screen.json binds file-storage mode visibility to
external-storage permission; general_section.json offers external/package
radio choices, not arbitrary root. Official
[world location reference](https://edusupport.minecraft.net/hc/en-us/articles/4404785703316-Location-of-World-Files)
documents this choice for Android, not a Mac profile selector.
Do not modify resource UI to invent an unsupported directory option.

Decision: retain existing fixed-home candidate (absolute private HOME
and CFFIXED_USER_HOME, per-slot TMPDIR, original signed app, normal
AppKit multi-instance, official account selection). This is a candidate
recipe for the existing synthetic fixture only; do not mount it as a
production game launcher. Existing --fixed-home-probe already checks
child environment, Foundation home, Application Support, caches, temp,
and independent preferences across reopen. No duplicate fixture needed.
MAC-13a OneAuth evidence still makes shared-user/account and App Group
behavior separate from ordinary home routing. No custom credential store,
new OS user, global environment or preferences manipulation, or re-signing.

Container availability check again failed before reading bytes. Do not
retry the full main binary transfer. No new runtime/fixture changes:
existing compact offline MAC-09 report or documented vendor route can
narrow game-owned file/account behavior; neither substitutes for later
authorized actual native acceptance. Source reviewed, NOT_EXECUTED.
Native launch/presentation admission remains false, not
SOURCE_READY_FOR_LOCAL. All test/CI/release holds remain.

### MAC-13a — actual OneAuth binary narrows the account route

Reused operator-supplied extracted OneAuth 3.5.4 binary (14,069,648 bytes).
Unlike earlier inaccessible container transfers, bounded Drive raw fetch
returned complete bytes; inspected in-memory as data only. Fat Mach-O
magic 0xcafebabe, two 64-bit slices, bounded LC_SYMTAB/nlist64/string
ranges. x86_64: 2228 symbols, 626 external undefined. arm64: 2025
symbols, 631 external undefined. Both slices contain the same relevant
external undefined symbols: NSObject classes NSFileManager, NSProcessInfo,
NSURL, NSUserDefaults; SecItemAdd/CopyMatching/Delete/Update;
kSecUseDataProtectionKeychain, kSecAttrAccessGroup, kSecAttrAccount,
kSecAttrService, kSecAttrAccessible, kSecClassGenericPassword.
Selected kSecUseKeychain, kSecMatchSearchList, SecKeychainSetDefault,
SecKeychainSetSearchList, SecKeychainOpen/Create/CopySearchList/CopyDefault,
NSHomeDirectory, getenv and selected CFPreferences function imports not
found in the examined undefined-symbol lists. Absence does not exclude
other modules, dynamic resolution or indirect implementation. Exact
NUL-delimited lexical indications across the binary additionally include
containerURLForSecurityApplicationGroupIdentifier:, standardUserDefaults,
objectForKey:, setObject:forKey:, login_hint, loginHint, accountIdentifier,
homeAccountId, keychainAccessGroup, cacheConfig. These are not launch
flags, actual settings keys, call sites or observed query attributes.
No raw bytes, credential data, actual group names, private Drive URLs,
or arbitrary strings committed. No signature/digest verification or
native execution; symbol observations are bounded static evidence.

[Apple TN3137](https://developer.apple.com/documentation/technotes/tn3137-on-mac-keychains)
distinguishes user-scoped data-protection Keychain from file-based
Keychain; SecItem may address either, and access groups follow host
signing entitlements. Import evidence makes a custom file-Keychain
strategy particularly weak; do not create/switch/delete Keychains,
alter signing or bypass licensing. Prefer app-owned active account
selection with existing licensed authentication/cache if independence
is established. Official [Manage Accounts](https://edusupport.minecraft.net/hc/en-us/articles/360061382672-Sign-Into-Minecraft-Education)
documents switching accounts and remembering the last player, not
persistent slot-specific selection. Official [world locations](https://edusupport.minecraft.net/hc/en-us/articles/4404785703316-Location-of-World-Files)
remain OS-user-scoped on Mac; [Cloud Update](https://edusupport.minecraft.net/hc/en-us/articles/23323615654164-What-s-New-in-The-Cloud-Update-v1-21-10)
documents account-filtered teacher worlds and new Mac sign-in libraries,
not independent local storage or concurrent account proof.

Plan A unchanged. AppKit multi-instance + per-process data routing +
app-owned licensed account selection is the minimal candidate, not yet
an implementation recipe. No separate credentials store, global
preferences changes, symlink swapping, multi-user or new synthetic
fixtures. Main executable remains unread: container ClientError, no
bounded raw range connector, do not fetch 512 MB inline or repeat full
transfer. The existing compact MAC-09 offline report (or a supported
vendor profile/launch contract) remains decisive for choosing a game
storage mechanism; static imports still cannot prove behavior.
Acceptance remains two concurrent instances with separate mutable data,
correct accounts surviving close/reopen and peer sign-out, then five.
No relaxation of test/CI/release holds, no new runtime, no request to
resend the bundle. Native integration BLOCKED; NOT_EXECUTED.

### MAC-12b — bounded selected-window capture in the existing fixture

Reviewed Apple's [SCWindow](https://developer.apple.com/documentation/screencapturekit/scwindow),
[SCRunningApplication](https://developer.apple.com/documentation/screencapturekit/scrunningapplication)
and [SCScreenshotManager](https://developer.apple.com/documentation/screencapturekit/scscreenshotmanager)
contracts and original lifecycle-fixture.swift. Use the existing macOS 14+
single-frame capture (desktopIndependentWindow, fixed 320x240) rather
than adding a stream, encoder, worker or new binding package before a
managed target is admitted. Apple’s [stream sample](https://developer.apple.com/documentation/screencapturekit/capturing-screen-content-in-macos)
documents a queue-depth/memory tradeoff; continuous active preview is
a later measured requirement, not proven cheaper by a screenshot API.

Correct the existing capturePair path: resolve the entire original app
pair before requesting pixels, require exactly one window per original
PID with expected fixture bundle, title, on-screen normal layer and
distinct nonzero IDs; reject wrong expected-ID cardinality. Recheck
original app identity and existing permission around captures. One fresh
shareable-content enumeration after the pair rejects changed or ambiguous
window targets before returning either image to the original marker/panel
consumer. Keep fixed image bounds, sequential snapshots and no disk/IPC
pixels, polling, synthetic input, permission prompt or external app capture.
Existing marker checks still require the selected A/B window to change
without the peer changing. Observations are non-atomic; this cannot prove
kernel ownership or rule out all transient replacements. Source only,
NOT_EXECUTED; no new product adapter, no Windows change.
Plan A remains exclusive. The product shell still refuses managed
launch/presentation. Next unresolved product-critical work is retained
native launch ownership and game-owned private data/account isolation
(MAC-13); do not keep adding synthetic capture features as a substitute.
All native/test/CI/release holds remain.

### MAC-12a — same-desktop Quartz window candidates

Reuse the existing explicit Mac installation inspection and original
AppKit objects. One Quartz CGWindowListCopyWindowInfo call, on-screen
only, desktop elements excluded; current user GUI session only.
[Apple API](https://developer.apple.com/documentation/coregraphics/cgwindowlistcopywindowinfo(_:_:))
and [required keys](https://developer.apple.com/documentation/coregraphics/required-window-list-keys)
establish owner PID/window number/layer. No continuous enumeration;
bounded parsing, existing CoreFoundation 0.10.1 ownership wrappers, no
package or lock change. Filter matching PIDs before reading other window
fields; count apps with exactly one layer-zero candidate, reject duplicate
or invalid target window IDs. Null list returns unknown. AppKit retained
objects are rechecked after the snapshot; any changed observation refuses
both counts. Simplify AppKit equality to the generated binding's PartialEq
(NSObject isEqual), no explicit superclass conversion.
Report a typed aggregate only in existing diagnostic UI, with count not
greater than observed apps. No raw window/PID/title, screenshot, focus,
permission prompt, process adoption or controller. Layer zero is only a
candidate: not exact main game window, signed loaded image, playable
visibility, or durable ownership. AppKit/Quartz snapshots are non-atomic.
Rust/TS regression source authored, NOT_EXECUTED. Windows behavior,
native admission=false, Plan A and all holds preserved.
Next: review the existing ScreenCaptureKit fixture for selected-window
capture seam; do not activate capture or input without a retained
authorized target. Profile and account isolation remain unresolved.

### MAC-11b — revalidate original AppKit application objects

Apple [processIdentifier](https://developer.apple.com/documentation/appkit/nsrunningapplication/processidentifier)
requires NSRunningApplication isEqual rather than PID equality alone.
The [NSRunningApplication contract](https://developer.apple.com/documentation/appkit/nsrunningapplication?language=objc)
also permits stale time-varying properties until the main run loop and
an object remaining valid after exit. The existing discovery path now
reobserves each original retained array object by PID lookup, object
equality, termination and PID rechecks, with all-or-unknown count for
duplicates, absent or invalid observations. No new report fields,
process table, ownership adoption or termination. The original objects
are released at the end of the explicit inspection; this is not durable
controller ownership, a kernel birth-time pin, or signed mapped-image
verification. No inference of launch permission even on zero count.
Reviewed the objc2 frameworks-0.3.2 tag 7b1abfd750a2cacaea71d6a56ecfb83cb7de560b
manifest and 0.3.2 binding reference; enable its existing libc feature
and matching lock dependency edge (libc already resolved). Regression
covers missing, duplicate and invalid observations; not executed.
Windows contract and runtime admission=false remain unchanged.
Next: same-desktop native window observation using the existing
workspace/fixture boundary. Durable original launch receipt/close
ownership and profile isolation remain prerequisites for managed launch.
No new dependencies, services, CI, testing or release.

### MAC-11a — AppKit discovery in the existing Mac inspection route

Advance the native lifecycle lane without enabling game launch. Reuse
the resolved objc2 0.6.5 and objc2-app-kit/foundation 0.3.2 packages
as direct Mac-only bindings, with narrow features and no new package
version. Update the desktop manifest and existing lock entry together.
Apple NSRunningApplication runningApplications(withBundleIdentifier:)
provides an atomic, run-loop-sensitive list of matching user apps,
not a full system process inventory or signed image attestation.
Reviewed the [Apple API](https://developer.apple.com/documentation/appkit/nsrunningapplication)
and [pinned binding reference](https://docs.rs/objc2-app-kit/0.3.2/objc2_app_kit/struct.NSRunningApplication.html).
Add one read-only aggregate count to the existing explicit installation
inspection, typed bridge and shell notice. A missing count stays unknown;
even zero is not permission to launch. No PID, path, account value,
background polling, new IPC command, transport, process adoption,
or termination of externally opened applications. No change to Windows
wire or lifecycle. Native original-object retention and normal close
remain in the existing unexecuted AppKit synthetic fixture; before a
product launch they must be bound to the actual single controller,
verified loaded image, and private profile admission. Do not manufacture
Mac FILETIME/HWND values. MAC-10 shared status/close policy unchanged.
Source committed; Rust/TypeScript regression source amended, NOT_EXECUTED.
Cargo resolution and native execution not verified. Plan A only;
native launch remains BLOCKED on storage/account isolation.
Next: native window observation and original-object ownership boundary;
no managed launch, capture/input permission changes, CI, test or release.

### MAC-10 — shared native lifecycle policy, independent of profile routing

Operator selected parallel source progress for Plan A (same OS user and
desktop, single original signed installation, one controller). Move the
existing active five-profile status projection and normal-close selection
from desktop session_protocol.rs into runtime-core/src/session_lifecycle.rs.
Share valid_profile_slot with the existing Windows request validation and
launch entry. Existing Windows caller/re-export and status/close regression
fixtures retain their behavior and wire; no changes to IPC schema, process
identity values, native launch/exit authority, profiles, or dependencies.
Do not reuse the separate historical four-slot SessionSlot catalog.

MAC-10 is a small shared pure-policy slice, not a portable native identity
or controller implementation. FILETIME/HWND/session values remain Windows
wire metadata; no cast of Mac birth time or CGWindowID to Windows authority.
Source committed, existing tests NOT_EXECUTED. The native lifecycle/workspace
source lane can proceed independently of game storage evidence; game launch
admission remains blocked and the existing Mac shell continues to refuse it.
Next: reuse the existing AppKit fixture evidence and map original-process
ownership to the retained controller boundary before any product launch.
No new controller, service, fallback topology, CI, localtest or release.

### MAC-09b — inspect remaining accessible bundle resources

Reused the operator-supplied extracted Education bundle; no new upload.
Read the complete OneAuth 3.5.4 Resources/PrivacyInfo.xcprivacy (1022
bytes). It declares UserDefaults reasons 1C8F.1 and AC6B.1, file
timestamp reason C617.1. Apple's
[required reason API reference](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype)
defines these as same-App-Group defaults, managed configuration/feedback,
and metadata of container/group/CloudKit files, respectively.
This is a declaration by the bundled authentication framework, not an
observed call, effective entitlement, active account key or evidence that
all instances share active accounts. Together with MAC-06c provisioning
observations it strengthens the need to distinguish per-process game
storage from group-scoped authentication/settings. Microsoft's
[MSAL Keychain guide](https://learn.microsoft.com/en-us/entra/msal/objc/howto-v2-keychain-objc)
requires app-owned cache configuration and matching entitlements; it
does not expose a supported external per-instance override.

Inspected Resources and Resources/data immediate inventory, including
localization/storyboard directories; data contains shared packs,
definitions, renderer/shaders, templates and small configuration files.
Read all three actual bootstrap JSON files (bootstrap, bootstrap_creator,
bootstrap_legacy): their profile objects are diagnostic verbosity/category
settings, not player storage or account profiles. Do not edit them or
duplicate assets as an isolation mechanism. No profile path configuration
found in this bounded review; nested asset trees were not exhaustively
searched. Main and OneAuth executable bytes remain unread; a fresh
container availability check returned ClientError before reading the
already materialized main file. No further large transfer attempted.

Plan A only. Preserve the single signed app and shared immutable assets;
private mutable game data and independently retained licensed account
selection are the acceptance boundary, not necessarily a separate token
cache implementation. Do not manipulate shared preferences, Keychain,
MDM or signing to force independence. No new runtime, probe or dependency.
The existing MAC-09 compact offline intake remains the next evidence
source; its indicators alone cannot establish native behavior.
Source reviewed, NOT_EXECUTED; native integration BLOCKED. All holds
preserved. No request to resend the existing bundle.

### MAC-09a — operator selects Plan A exclusively

Same OS user, same graphical desktop, one original signed installation,
five independent licensed profiles and the existing controller/workspace
are the selected topology. No distinct-user transport, VM, background
service or fallback topology work. This supersedes the fallback decision
suggestion in MAC-09; historical evidence remains valid.

The [Apple App Sandbox documentation](https://developer.apple.com/documentation/security/accessing-files-from-the-macos-app-sandbox)
associates the filesystem container with the app; [NSHomeDirectory](https://developer.apple.com/documentation/foundation/nshomedirectory())
documents user or app home, not a per-process isolation guarantee.
The historical [Education support thread](https://edusupport.minecraft.net/hc/en-us/community/posts/28795140075540-Technical-issues-1-21-03-Multiplayer?sort_by=votes)
also reports an application-specific support directory and preferences
domain separate from the usual world tree; [support follow-up](https://edusupport.minecraft.net/hc/en-us/community/posts/29066176455956-Minecraft-Edu-1-21-03-Mac-Open-Error)
corroborates Keychain-dependent game functionality. Historical reports
are not current storage maps or instructions to delete/reset data.
Require independent game world/settings, app support/preferences,
and active account/cloud behavior, not merely distinct HOME strings.
Do not manipulate Keychain entries or patch the signed application.

Existing MAC-08a/b synthetic path and preference witnesses and MAC-09
offline collector already cover the next source question. No additional
fixture, analyzer layer or production launch implementation is justified
without the compact actual main executable report. Search of available
conversation/library did not surface that report; current container access
failed before any bytes were read. The existing Windows-side offline
--mach-o intake in Development/tooling/README.md remains the only
necessary artifact handoff; no native execution or localtest requested.
Source reviewed, NOT_EXECUTED. Plan A implementation remains BLOCKED
at evidence, not SOURCE_READY_FOR_LOCAL. Preserve every existing hold.

### MAC-09 — make the existing compact artifact intake decision-oriented

The Drive sample's main executable (512129360 bytes) was materialized
according to the connector receipt, but both subsequent container and
Python access returned ClientError; no binary bytes or digest observed.
Do not retry full transfer in this surface. The existing offline collector
now checks a fixed allowlist of exact NUL-delimited routing-related byte
strings during its existing streaming SHA-256 pass. It emits only
aggregate booleans, no arbitrary strings, paths, credentials, or new
runtime dependencies. Includes documented Foundation/Security APIs,
observed world-path terms, environment names and known *other launcher*
flags for discrimination only. A hit is neither import/call-site proof
nor a supported Education option; absence is not exclusion. Existing
Mac codesign verification and native admission refusal unchanged.
Source regression covers chunk boundaries, exact token matching,
allowlist and unchanged hash; NOT_EXECUTED. Reuse the same documented
--mach-o command from Windows with the original extracted file and
compact JSON, not another large upload or native test request.

New game-specific counterevidence: the Education
[1.21.03 support discussion](https://edusupport.minecraft.net/hc/en-us/community/posts/28795140075540-Technical-issues-1-21-03-Multiplayer?sort_by=votes)
reports a historical Mac Keychain fix, and a
[2021 community report](https://edusupport.minecraft.net/hc/en-us/community/posts/360076715031-iPad-Unable-to-access-library-COMMUNITY-HELP-NEEDED)
describes a separate OS user resolving account-context problems despite
reinstall/world-directory cleanup not signing out the original user.
Neither establishes current concurrent isolation. Apple
[TN3137](https://developer.apple.com/documentation/technotes/tn3137-on-mac-keychains)
distinguishes file-based and data-protection keychains; environment
redirection alone cannot establish their independent use. Apple
[Remote Desktop](https://support.apple.com/en-gw/guide/remote-desktop/apd4f46319e/mac)
documents a distinct-user virtual desktop, but
[High Performance Screen Sharing](https://support.apple.com/en-ie/guide/mac-help/mchl1883115d/27/mac/27)
is limited to one active high-performance session per Mac. Distinct-user
fallback remains an option for operator selection, not an automatic
topology switch or a proven five-slot workspace.

Next: review the compact game-specific artifact report; if no decisive
supported same-user route is found, resolve topology with the operator
rather than add another speculative mechanism. All native/test/CI/release
holds remain. Native integration BLOCKED, not SOURCE_READY_FOR_LOCAL.

### MAC-08c — keep the native profile boundary small

Decision unchanged: same user/desktop, one original signed Education
installation, five private profiles, one existing lifecycle/workspace
authority. No production adapter, additional fixture, transport or
dependency until the game's actual private storage/account route is known.

- Original [Bedrock Launcher UI](https://github.com/minecraft-linux/mcpelauncher-ui-qt/blob/3f955bf59e72311b8709b0b96567bd7c86682190/gamelauncher.cpp)
  passes the profile data directory to its own client; that
  [client](https://github.com/minecraft-linux/mcpelauncher-client/blob/78904982c13d7bcec7af4b572b18cac83d930d31/src/main.cpp)
  separately parses game/assets, data and cache directories and implements
  Android-path translation. [Prism Launcher](https://github.com/PrismLauncher/PrismLauncher/blob/88c670d143435007d1757436d38e393d063bc82b/launcher/minecraft/MinecraftInstance.cpp)
  derives a Java instance game root. Both illustrate explicit data ownership;
  neither supplies an Education native CLI contract. Do not import their
  compatibility layers or forward their arguments to Education by analogy.
- Apple [launch environment](https://developer.apple.com/documentation/appkit/nsworkspace/openconfiguration/environment)
  is ignored for a sandboxed caller; the existing Swift fixture already
  requests a new instance and disables running-app substitution. These
  are launch controls, not file/account isolation. Do not add another
  launch helper or relax sandboxing to make a candidate pass.
- Microsoft [MSAL SSO guidance](https://learn.microsoft.com/en-us/entra/msal/objc/single-sign-on-macos-ios)
  explicitly warns that shared-cache removal operations can affect other
  applications. This is library-level counterevidence, not proof of
  Education behavior. Require peer account survival on sign-out and reopen,
  not merely simultaneous sign-in. [Education cloud saving](https://edusupport.minecraft.net/hc/en-us/articles/19632132361748-Cloud-Upload-and-Download)
  also binds uploads to the user's OneDrive; verify independent cloud
  destinations when available, without accessing tokens or disabling backup
  as a workaround.

Minimal acceptance order remains: actual original image and two retained
native processes → private game storage and independently persistent
licensed identities → exact-window capture/input → five-slot measured
workspace. Keep existing MAC-08a/b fixtures as research only.
No supported Education data-root override established by this review.
Existing small main Mach-O report or documented game-owned route remains
the next decision-changing source input; metadata alone cannot prove
runtime independence. Source review only, NOT_EXECUTED; all holds remain.
Native integration BLOCKED, not SOURCE_READY_FOR_LOCAL.

### MAC-08b — native home-routing candidate, not a game profile recipe

Reviewed current [Minecraft Education world locations](https://edusupport.minecraft.net/hc/en-us/articles/4404785703316-Location-of-World-Files):
macOS world root is under the device user's Library/Application Support/
Minecraftpe/Games/com.mojang, not the Minecraft sign-in account. The
[Microsoft Education deep link reference](https://learn.microsoft.com/en-us/minecraft/creator/reference/content/deep-links?view=minecraft-bedrock-stable)
documents opening/joining worlds and servers, not per-instance storage
roots; its handlers are internal and can change. Java Edition --gameDir
does not establish an Education option. No supported Education-specific
CLI data-root override found in reviewed sources. Main binary remains
unread (container access failed again); no assertion about its internal
argument handling or actual Foundation file-path calls.

New bounded candidate: historical Apple open-source CoreFoundation
[CFPlatform.c](https://github.com/apple-oss-distributions/CF/blob/dc54c6bb1c1e5e0b9486c1d26dd5bef110b20bf3/CFPlatform.c)
prioritizes CFFIXED_USER_HOME for non-setugid home resolution before
the account home and optional HOME fallback. The same pinned
[CFPreferences.c](https://github.com/apple-oss-distributions/CF/blob/dc54c6bb1c1e5e0b9486c1d26dd5bef110b20bf3/CFPreferences.c)
derives a preferences path from the resolved home.
This is historical implementation evidence, not a documented stable
macOS 14+/26 contract. Modern preference daemon, Foundation, caches,
account identity and app-specific storage can still use the actual
user context; [Apple NSHomeDirectory](https://developer.apple.com/documentation/foundation/nshomedirectory())
and [Preferences Utilities](https://developer.apple.com/documentation/corefoundation/preferences-utilities)
describe home/preferences scope, not a supported per-process profile
isolation API.

Decision: extend only the existing opt-in synthetic preference witness
with --fixed-home-probe; same bundle/lifecycle/markers, adds only
process-local CFFIXED_USER_HOME per synthetic child. Keep HOME/TMPDIR
baseline and report actual fixed-home presence and match across write
and reopen. Candidate success requires private paths in all four
observations, matching override and independent markers. Neither
result proves real game data, Keychain, cloud destinations or account
isolation. No preference-daemon override, global environment change,
injection, new helper, runtime dependency or game launch. Synthetic
domain side-effect caveat from MAC-08a remains; no auto-cleanup.
Source NOT_EXECUTED, no compile/test/CI.

Next decisive input remains the existing small main Mach-O metadata
report or documented game-owned storage route; no repeated large
transfer. If no supported route is found, only actual held native
observations can determine whether this candidate is viable. No further
generic launch/profile fixtures without decision-changing evidence.
MAC-07e and multi-user transport remain paused; all native, Windows,
test/CI and release holds preserved. Not SOURCE_READY_FOR_LOCAL.

### MAC-08a — restore same-user native priority; synthetic preference witness

Operator selected the Windows-like approach as investigation priority:
same OS user, same GUI desktop, one official unmodified installation,
independent native processes and data/account contexts, one existing
controller and workspace. Pause MAC-07e and further distinct-user
transport work; MAC-07a–d remain historical fallback research only,
not active product topology or automatic fallback. Supersedes the
investigation priority of MAC-05d/06c/07c, not their observed evidence,
security boundaries or five-instance acceptance requirements.

Re-reviewed actual Windows suspended-process bootstrap and existing
MAC-04c fixture: Windows redirection relies on reviewed compatibility
DLLs and Win32 namespaces, not merely environment variables. Apple
NSWorkspace.OpenConfiguration supports a new instance and process-local
environment; these are candidate mechanisms, not proof of Education
private storage. PluralMacPlus previously reviewed at 458d8e2 shares
Keychain and uses a preferences compatibility shim; no copy, patch,
injection, re-signing or shim adopted. No supported Education-specific
profile-path option established in reviewed public sources. Official
[Minecraft account management](https://edusupport.minecraft.net/hc/en-us/articles/360061382672-Sign-Into-Minecraft-Education)
describes last-player persistence and switching without clearing saved
passwords. Shared cache permission does NOT prove a shared active
account; conversely, concurrent independent active accounts and restart
persistence are not established. Do not read or manipulate credentials,
token caches or MSAL configuration. Static main Mach-O still unread.

Extend existing lifecycle-fixture.swift (no new product owner) with
explicit --preferences-probe: same unique synthetic bundle identifier,
two HOME/TMPDIR roots, current-user/any-host CFPreferences domain,
one synthetic key, ordered A/B writes, exact retained app close/reopen
A and B with peer liveness checks, fresh readbacks. Use documented
[CFPreferencesCopyValue](https://developer.apple.com/documentation/corefoundation/cfpreferencescopyvalue(_:_:_:_:))
and [CFPreferencesSynchronize](https://developer.apple.com/documentation/corefoundation/cfpreferencessynchronize(_:_:_:))
for exact domain reads/flushes, not a global preferences reset. Only
fresh UUID fixture namespace, refuse unexpected values, no real game
preferences or Keychain. Small synthetic preference may remain in real
user Library if HOME is ignored; disclose, no automatic cleanup.
Path observations remain separate from preference comparisons.
Positive markers do not establish general preferences, auth, world
or Minecraft isolation. Previous modes unchanged; source NOT_EXECUTED.

Ordered next gates: (1) one original signed app and two retained native
processes; (2) actual supported game-owned private data/settings route,
including caches and cloud destinations; (3) two independently licensed
manual sign-ins, independent active identities and sign-out/reopen
without cross-account effect or overwrite; (4) native window capture
and human-selected input in same desktop; (5) five profiles and measured
overhead. Do not admit runtime on synthetic results. Source-only review
can pursue existing artifact metadata and a supported game-owned route,
but actual independence requires matching Mac observation. No localtest
request, no compile, CI, dependency, release or Windows runtime change.
Distinct-user path remains paused unless same-user is ruled out and
operator selects the fallback. All previous holds preserved.

### MAC-07d — cooperative cross-session rendezvous witness

Reviewed original Apple [root/login sessions](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPMultipleUsers/Concepts/SystemContexts.html)
documentation: cooperative BSD sockets are an explicit cross-login-session
communication mechanism. [NSXPCListenerEndpoint](https://developer.apple.com/documentation/foundation/nsxpclistenerendpoint)
is passed over existing XPC connections; anonymous endpoints do not
establish the first trusted channel themselves. [NSXPCConnection](https://developer.apple.com/documentation/foundation/nsxpcconnection)
supports audit session identifiers and code-signing requirements after
connection; launch-agent names remain scoped, privileged daemon option
does not solve a non-root design. [getpeereid(3)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/getpeereid.3.html)
returns effective credentials captured at connect/listen, not per-message
or code identity. [SessionGetInfo](https://developer.apple.com/documentation/security/sessiongetinfo(_:_:_:))
can query a known security session ID; it does not bind a peer-reported
session number to a socket. Apple [Fast User Switching](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPMultipleUsers/Concepts/FastUserSwitching.html)
confirms SecuritySessionId differs from POSIX session/user identifiers.

Decision: prepare a smallest cooperative witness rather than a production
transport or extra daemon. Extend the existing synthetic lifecycle fixture
with explicit --rendezvous-probe listen/connect modes, one fresh fixed
AF_UNIX leaf under a manually pre-provisioned canonical /Users/Shared
directory owned by listener, no group/other directory write, explicit
partner UID and prior ACL review. Reject root, wrong peer, missing
graphical context, ambiguous/existing socket leaf, stale/invalid session
or timeout. One nonblocking connection and 120-second total deadline
per invocation, reciprocal getpeereid, exchange fixed eight-byte marker
and reported security session ID only, query both graphical sessions
through SessionGetInfo. No credentials, data access, process/game launch,
input, capture, TCP, account creation, ACL edits, background service,
auto-reconnect or unlink of unknown leaf. Existing single/profile/
workspace modes untouched. Output only scoped booleans/status, no raw
UIDs/session IDs/paths. Fixture NOT_EXECUTED, no compile or test.

Important distinction: even success would show connection-time expected
UID and two reported graphical security sessions, not independent
kernel binding of peer session, code identity, per-message sender,
virtual desktop or game lifetime. Pre-provisioned directory ACLs are
not verified by fixture and need prior operator review; POSIX mode
checks are not a full trust boundary. Source probe is not an admitted
transport and must not carry game frames or commands. XPC bootstrap,
native signature and exact session/window authority remain unproven.
No dependency or product activation; no Windows wire/lifecycle change.

Next: MAC-07e compare a documented OS-provided peer audit identity
mechanism or explicitly authenticated scoped endpoint that binds the
actual login context and signed code, without root, and determine if
native capture remains viable. Do not expand the synthetic channel
into production by assumption or repeat static research that cannot
change the decision. Actual two/five instance identity, isolation,
relative input and render continuity require matched Mac evidence.
All previous native, CI, test and release holds preserved.

### MAC-07c — public native session API review and transport security boundary

Reviewed pinned MAC-07b source and existing AppKit/ScreenCaptureKit lifecycle
fixture. Preserve the Windows controller and wire; no new production route.
The previously selected distinct-OS-user candidate requires explicit consent
in each existing standard graphical login context, not credentials collected
by the manager or a root service.

Native API mapping (source/documentation evidence, not executed):
- Security.SessionGetInfo(callerSecuritySession) returns the caller's
  transient security session identifier and attributes. Require non-root
  graphical context and expected effective UID; graphical access alone
  does not prove a virtual desktop or exact game. Neither POSIX getsid
  nor username is a substitute. Reobserve after login/session change.
  [Apple SessionGetInfo](https://developer.apple.com/documentation/security/sessiongetinfo(_:_:_:))
  and [login session model](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPMultipleUsers/Concepts/SystemContexts.html).
  Original [AuthSession.h](https://github.com/apple-oss-distributions/Security/blob/db15acbe6a7f257a859ad9a3bb86097bfe0679d9/OSX/libsecurity_authorization/lib/AuthSession.h)
  defines callerSecuritySession, noSecuritySession and graphical/root bits.
- Retain original NSRunningApplication in its authorized login context,
  compare isEqual rather than PID alone; launchDate is optional and
  available for LaunchServices launches only. Require exact expected
  bundle/executable observation plus separate native signing/image gate.
  [Apple processIdentifier](https://developer.apple.com/documentation/appkit/nsrunningapplication/processidentifier)
  and [launchDate](https://developer.apple.com/documentation/appkit/nsrunningapplication/launchdate).
  The MAC-07b synthetic process_birth field is not an OS API selection.
- ScreenCaptureKit SCWindow owningApplication and windowID, with
  SCContentFilter(desktopIndependentWindow:), can scope a capture to
  one window. CGWindowListCopyWindowInfo is documented for the current
  user session, not a central cross-user window enumerator. Use the
  session-local retained target; refuse lost/replaced or ambiguous
  window. [Apple SCWindow](https://developer.apple.com/documentation/screencapturekit/scwindow)
  and [SCContentFilter](https://developer.apple.com/documentation/screencapturekit/sccontentfilter).
  Existing screen-capture authorization and background GPU continuity
  still required. No desktop-wide capture or prompt/grant automation.
- CGEvent.postToPid and CGPreflightPostEventAccess are public, but
  postToPid returns no delivery acknowledgement and targets a process,
  not a particular window. Not proof of relative camera motion, key-up,
  focus, independent gameplay or neutral release. No global input fallback,
  no injection, no permission bypass; leave input blocked until matched
  session-local evidence. [Apple CGEvent](https://developer.apple.com/documentation/coregraphics/cgevent).
- XPC supports system-checked peer code requirements on received messages.
  A signed exact Lazy Tester helper identifier plus approved team, UID
  and audit/login context must be independently established. Public
  [XPC peer code requirement](https://developer.apple.com/documentation/xpc/xpc_connection_set_peer_code_signing_requirement(_:_:))
  and [Apple DTS guidance](https://developer.apple.com/forums/thread/681053)
  prefer message-bound checks over PID-based code validation.
  Original [SecCode.h](https://github.com/apple-oss-distributions/Security/blob/db15acbe6a7f257a859ad9a3bb86097bfe0679d9/OSX/libsecurity_codesigning/lib/SecCode.h)
  declares SecCodeCreateWithXPCMessage using the message's associated
  audit token. A connection endpoint can be carried in XPC messages,
  but that does not solve the initial trusted cross-session rendezvous.
  Per-user LaunchAgent registration requires participation of each
  user; default launchd namespaces restrict cross-login lookup.
  [Apple XPC](https://developer.apple.com/documentation/xpc),
  [SMAppService register](https://developer.apple.com/documentation/servicemanagement/smappservice/register())
  and [xpc_endpoint_create](https://developer.apple.com/documentation/xpc/xpc_endpoint_create(_:)).
  No registration, signing key, agent deployment or root broker approved.
- Local BSD socket is a possible bootstrap research path, not an
  admitted channel. getpeereid returns connection-time effective UID/GID,
  not code identity or login identity. Original XNU
  [un.h](https://github.com/apple-oss-distributions/xnu/blob/f6217f891ac0bb64f3d375211650a4c1ff8ca1ea/bsd/sys/un.h)
  defines LOCAL_PEERTOKEN; [uipc_usrreq.c](https://github.com/apple-oss-distributions/xnu/blob/f6217f891ac0bb64f3d375211650a4c1ff8ca1ea/bsd/kern/uipc_usrreq.c)
  obtains it by looking up the peer socket's last_pid and current task
  audit token. Do not equate this with per-message authenticated sender
  identity or assume it solves PID reuse/socket delegation; availability
  and semantics on the minimum supported macOS still require proof.
  [Apple getpeereid](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/getpeereid.3.html).
  No globally writable socket directory, claimed UID or loopback fallback.

Decision: prioritize native session-local capture/observation and
message-bound XPC identity over a custom ARD protocol/crypto stack.
XPC cross-session bootstrap is still unproven and not selected for
production. A manually authorized Apple Standard virtual-session viewer
remains the interim feasibility witness (not final workspace).
The helper may eventually serve exact local game observations and
presentation only; one central controller retains slot, lifecycle and
input authority. No new code, dependencies, public wire, or fixture
duplication in this research step.

Next MAC-07d: establish a minimal consented rendezvous feasibility design
using existing accounts and explicit launch, comparing an OS-mediated
endpoint handoff with a scoped local bootstrap. Before authoring a live
channel, establish how the initial peer is authenticated, how the exact
login/target identity is obtained, and how ownership is revoked without
adoption on reconnect. If no supported boundary meets these conditions,
keep that transport blocked; do not invent a privileged service. A
future synthetic native witness should establish two independent graphical
contexts before any game launch or transport input. No native evidence,
source build/tests or CI executed. All previous image, runtime, test,
Windows and release holds preserved; full workspace parity BLOCKED.

### MAC-07b — synthetic transport admission contract

Reviewed pinned native_clients.rs, native_clients_session.rs and
session_protocol.rs, independent_bootstrap/session_controller.rs,
controller_ai_input_leases.rs, process_identity.rs, workspace_surface.rs
and previews.rs. The existing retained Windows Sessions owner holds
game lifetime and profile state; its original process/window identity
is Windows-specific (FILETIME, session ID, HWND). Existing input leases
are feature-gated Windows-specific, not cross-user authentication.
Workspace preview is presentation-only, max four secondary tiles, with
existing cancellation fencing. Non-Windows session requests and native
previews remain Unsupported. Do not pretend these Windows receipts
represent Mac native facts.

Added a private cfg(test) contract fixture under the existing
workspace_surface owner, no new product module, feature or public wire
schema. It models only the distinct OS-user candidate: exact expected
slot/UID/login-session/PID/birth/window plus controller connection
generation, independently observed authenticated peer, virtual-session
and exact-game-window facts. All IDs/facts are synthetic. Refuse unknown,
console or wrong target, and duplicate user/login/process in two slots.
Frame admission is metadata-only: exact current route, strictly newer
sequence, owner-observed freshness and bounded payload size; no queue,
pixel allocation, codec, fixed production budget or capture loop.
Existing controller remains the sole future owner of generation, one
selected input writer and neutral reconciliation. Model switch admission
only after exact old writer release with a strictly newer lease epoch;
unknown delivery, disconnect, stale receipt, missing focus or wrong
connection refuses. No new writer book, network service or session
lifecycle implementation. Fixture fields are not OS API choices, wire
credentials or authorization evidence. Synthetic refusal tests authored,
NOT_EXECUTED. No Windows code path/wire change.

Next MAC-07c: compare public session-local OS mechanisms for observing
an authorized UID/login context and exact target window, with Apple
native viewer as feasibility witness. Determine a real trust boundary
and transport feasibility before writing any live endpoint or RFB
integration. Need original process lifetime and independently observed
session identity, not peer claims or console fallback. No new controller;
no assumption that a presentation connection implies launch/exit.
Main executable identity, account isolation, background GPU render,
relative gameplay input and actual two/five instance workspace remain
BLOCKED. All native, test/CI and release holds preserved.

### MAC-07a — standard-session transport review and admission boundary

Continue the selected five-profile native Mac goal without repeating
the binary-transfer research. Static main Mach-O receipt is still absent;
work on a transport candidate in parallel, not a relaxation of target
identity, independent authentication or native acceptance.

Original [Apple Remote Desktop session choice documentation](https://support.apple.com/en-gw/guide/remote-desktop/apd4f46319e/mac)
explicitly separates sharing the console from connecting to the authenticated
user's virtual desktop. If nobody is logged in, or the authenticated user
is the console user, the visible screen is the console. Therefore successful
authentication, a connection, a virtual request or a granted result alone
cannot prove an independent session. Apple's [login session model](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPMultipleUsers/Concepts/SystemContexts.html)
describes isolated, transient login contexts and trusted explicit
cross-session connections. [Standard and High Performance settings](https://support.apple.com/en-ie/guide/mac-help/mchl1883115d/27/mac/27)
describe adaptive Standard capture and one active High Performance session
per Mac. High Performance virtual displays are not equivalent to distinct
user login sessions. No official proof of five concurrent same-host
Standard sessions or game rendering was found.
[Apple VNC guidance](https://support.apple.com/ro-ro/guide/remote-desktop/apde0dd523e/mac)
warns about weak protection of input and broad desktop access.

Reviewed original independent source at immutable refs (no imports):
- [madeye/rv](https://github.com/madeye/rv/tree/2f620fb28316f5cb8894d2e9c33c4505a97c059f)
  MIT; Cargo.toml and crates/rv-session/Cargo.toml, src/session.rs and
  src/ard.rs. Authenticates via ARD then uses ordinary RFB connector.
  ARD credentials are encrypted but session traffic is not TLS. Required
  encryption selects VeNCrypt or refuses; prefer/auto can fall back to
  plain. No explicit ARD virtual-session select in reviewed connect path;
  unbounded command queue and OpenSSL vendored dependency are not a
  suitable default for Lazy Tester. Reference for RFB/frame ownership
  only; not an admitted transport.
- [peetinc/noVNC-ARD](https://github.com/peetinc/noVNC-ARD/tree/d699293635083c9e0c30c8f9480993865653772a)
  MPL-2.0; ard/session-select.js and ard/ard-patch.js (Session Select
  state machine). Explicit virtual/console choice and pending/granted
  results before post-init, full-tunnel mode advertised. Auto selection
  can choose console; virtual choice does not itself verify the actual
  login identity. Patched browser client, proxy and license obligations;
  no adoption, copied crypto, code port or claim of official Apple API.
- [vincent-vnc-mcp](https://github.com/cookiemonster0921/vincent-vnc-mcp/tree/1dfe3d849d99128411ee762f2c26d256c7c5b092)
  README and vnc_client.py: author reports a dedicated non-console user
  with localhost VNC on macOS 15.6; single connection with reconnect,
  screenshot polling, additional SSH/Tailscale/FFmpeg components, and
  explicit headless GPU caveat. No proof of five distinct users or
  real-time gameplay; no installer or dependency adopted.

Decision: Standard Screen Sharing is a feasibility candidate only.
Prioritize smallest supported native route before writing a Rust RFB
client. Compare (A) Apple-owned, manually authorized virtual-session
viewer as interim feasibility witness, (B) a scoped session-local
endpoint with exact target window capture and explicitly authorized
input if supported, (C) an embedded Standard ARD client only if
identity, protected transport, relative gameplay input and costs
are demonstrated. A standalone viewer is not final workspace parity.
No new transport dependency selected. No generic VNC password,
unprotected fallback, automatic account creation, SSH/Tailscale,
remote management enablement, root or credential persistence.
Do not reimplement cryptography or treat loopback as authentication.
A dedicated session-local endpoint remains a candidate, not an extra
manager, and requires explicit authenticated, scoped cooperation.

Next architecture gate (MAC-07b): use the same existing controller
as sole owner of lifecycle, slot mapping, normal-close and input lease
authority. Presentation transport may not launch/close/adopt games or
infer identity from username, desktop name, port, pixels or success
status. Require independently observed authorized UID, transient
login session, exact retained app/process/window and connection
generation. Refuse console/shared-desktop result or unknown target;
revoke frames and input on identity loss, logoff, reconnect or focus
change; no stale input replay. Allow only one selected input writer;
explicit neutral release before switching and no mutation retry
after unknown delivery. Bounded latest frame per slot, active viewport
priority, lower-rate previews, no default five full-rate streams,
clipboard disabled. No raw desktop capture of other users. Measure
relative mouse/camera, key press/release, foreground/console separation,
native GPU/render continuity, CPU/RAM/GPU and latency in future matched
native evidence; do not substitute screenshot polling or remote
desktop FPS for game performance. Transport disconnect is not game
exit. Require original two independent sessions before five.
If only desktop-wide capture/control works without exact game identity,
do not admit product automation. Preserve existing Windows wire,
23-tool catalog and all native, CI, test and release holds.
Source research only; no runtime or native/CI/test execution.

### MAC-06d — bounded framework Mach-O observations (static only)

Rechecked the supplied Drive root and Contents, and searched for
minecraft-metadata.json in the accessible Drive and current conversation.
No matching report found. Container access again failed ClientError.
The main executable remains unread; no repeated large transfer.

Read the actual two small framework files from the already supplied
Versions/A paths, as bounded binary data (no execution):
- msalsdk, 115360 bytes: 64-bit Mach-O dylib slices x86_64 and arm64.
  Both primary CodeDirectories match com.microsoft.msalsdk and expected
  team UBF8T346G9; hardened-runtime flag present, ad-hoc flag absent,
  CMS signature blob present. Only one readable symbol per slice;
  no supported profile-isolation route established.
- MinecraftSwift, 345648 bytes: 64-bit Mach-O dylib slices x86_64 and arm64.
  Both primary CodeDirectories match com.mojang.minecraftpe.minecraftswift
  and expected team UBF8T346G9; hardened-runtime flag present, ad-hoc
  flag absent, CMS signature blob present. Recognized Foundation,
  CoreFoundation and libswiftFoundation linkage; bounded symbol
  inspection (345 x86_64, 335 arm64 entries read) found no selected
  direct Keychain, UserDefaults, home or App Group symbol indicators.
  This negative result is not proof those APIs are unused: indirect
  dispatch, stripped symbols, other modules and dynamic loading remain.

Parsed only bounded headers/load commands, symbol names reduced to
allowlisted indicators, and embedded CodeDirectory fields. No
cryptographic validation, effective entitlements, main executable
architecture or main signer proof. No framework binary or raw symbol,
certificate, group value or private Drive URL committed. The larger
OneAuth binary was not inspected. Actual account isolation remains unknown.
The new framework-level observations corroborate prior evidence but
cannot establish a supported same-user profile mechanism. Preserve
distinct existing OS-user feasibility priority, mandatory shared
interactive five-instance workspace, all Windows/native/test/CI/release
holds. No runtime change or new fixture; further framework inspection
without main image and native evidence is not a meaningful unblock.
Next remains the existing small main Mach-O report, not another large
upload or localtest request.

### MAC-06c — extracted bundle provisioning and authentication framework evidence

The operator-provided extracted app directory is accessible through Drive.
Observed actual Contents inventory: main executable 512129360 bytes,
Frameworks, Resources, _CodeSignature, Info.plist (8041 bytes), and
embedded.provisionprofile (12937 bytes). Some framework root entries
are represented as zero-byte files after extraction; actual binaries
and metadata are in Versions/A. Do not regard the Windows/Drive copy
as a complete installable or signature-verifiable macOS bundle.

Read complete framework Info.plists:
- OneAuth.framework/Versions/A/Resources/Info.plist:
  executable OneAuth, com.microsoft.authentication, version/build 3.5.4.
- msalsdk.framework/Versions/A/Resources/Info.plist:
  executable msalsdk, com.microsoft.msalsdk, version/build 1.1.0.
- MinecraftSwift.framework/Versions/A/Resources/Info.plist:
  executable MinecraftSwift, com.mojang.minecraftpe.minecraftswift,
  version 1.0.0, build 1.0.0.0, SDK macosx26.1.
These establish bundled components, not their runtime use.

Retrieved the bounded 12937-byte embedded.provisionprofile as data through
Drive. Parsed the DER CMS SignedData encapsulated XML payload (8460 bytes),
without evaluating instructions, executing game code or validating the
CMS signature. Observed TeamIdentifier matches UBF8T346G9;
com.apple.application-identifier matches
UBF8T346G9.com.microsoft.minecraft-edu; developer team entitlement matches.
The profile has two app-group entries and one keychain-access-groups
entry. That entry has the expected team prefix and wildcard form; it
covers the documented macOS MSAL default group, but is not an exact
match to it. Raw group values, certificate data, private Drive links,
and proprietary bundle bytes are intentionally not committed.
ProvisionsAllDevices=true; no ProvisionedDevices array observed.
App Sandbox, get-task-allow, disable-library-validation and
allow-dyld-environment-variables were absent from this XML entitlement
dictionary, not proven false in executable. No CMS signature verification,
certificate trust evaluation or effective entitlement verification.

Apple [TN3125](https://developer.apple.com/documentation/technotes/tn3125-inside-code-signing-provisioning-profiles)
distinguishes profile authorization from actual claimed code-signature
entitlements, including macOS unrestricted entitlement exceptions.
[Microsoft MSAL macOS keychain documentation](https://learn.microsoft.com/en-us/entra/msal/objc/howto-v2-keychain-objc)
documents the default group and app-owned configuration.
[Apple TN3137](https://developer.apple.com/documentation/technotes/tn3137-on-mac-keychains)
explains that data protection keychain selection uses user login context,
and library access depends on host main executable entitlements.
Thus actual bundle evidence strengthens the shared-authentication risk,
but neither a wildcard nor bundled SDK proves active shared cache use.
Do not substitute framework signing metadata for the main executable.

Decision: retain distinct existing standard macOS users as the strongest
isolation feasibility candidate; same-user route remains preferable only
if supported, with independent sign-in/data proven. No selection of B as
final topology or relaxation of shared interactive five-instance workspace.
No token/credential access, injection, re-signing or private app copies.
The next decisive static evidence is the existing MAC-06b main Mach-O
report (not another binary upload). Container access still fails; large
executable bytes remain unread. No new runtime or parser, no native
execution, no build/test/CI; all holds preserved. This is an evidence
checkpoint, not a new implementation milestone.

### MAC-06b — bounded offline Mach-O evidence intake

The operator supplied minecraft-edu (512129360 bytes) and then seven
7z volumes (six 104857600-byte parts plus 35989534 bytes, total
665135134 bytes). File catalog metadata observed; Python and container
access failed with ClientError before any file bytes were inspected.
Do not claim archive integrity or extraction. Do not request another
large upload or smaller archive parts.

Extend the existing education-target.mjs evidence owner with explicit
`--mach-o` mode, Windows-compatible Node standard library only. No
product change or dependency. Read exact executable basename with
bounded offsets, 64-bit little-endian thin and big-endian FAT/FAT64
headers, slice and load-command bounds, LC_CODE_SIGNATURE SuperBlob,
CodeDirectory primary/alternate flags and expected identifier/team
comparisons, CMS/DER blob presence, selected lexical XML key indications
and selected linked dylib name hints. Hash through 1 MiB buffers; recheck
file metadata. Reject malformed layouts, no arbitrary output of embedded
strings, paths or group values. Source references pinned Apple XNU
[f6217f8 loader.h](https://github.com/apple-oss-distributions/xnu/blob/f6217f891ac0bb64f3d375211650a4c1ff8ca1ea/EXTERNAL_HEADERS/mach-o/loader.h),
[fat.h](https://github.com/apple-oss-distributions/xnu/blob/f6217f891ac0bb64f3d375211650a4c1ff8ca1ea/EXTERNAL_HEADERS/mach-o/fat.h)
and [cs_blobs.h](https://github.com/apple-oss-distributions/xnu/blob/f6217f891ac0bb64f3d375211650a4c1ff8ca1ea/osfmk/kern/cs_blobs.h).
Unsupported byte order/architecture and oversized metadata refuse.
No cryptographic or effective entitlement verification, no runtime
admission, no proof of active auth libraries or profile isolation.
Hash/metadata are not an atomic pin. Original native codesign route
preserved. Authored synthetic format/refusal fixtures NOT_EXECUTED.

Small input is now one JSON report (see tooling README), not another
binary transfer. Explicit offline artifact reading is not localtest
or a request to run game or source tests. No actual report obtained
in this turn; integration remains BLOCKED. All Windows, CI, test,
native activation and release holds preserved.

### MAC-06a — user-supplied 1.26.33.0 bundle metadata

The operator supplied Info.plist after the Drive DMG (810354021 bytes)
was readable as metadata but raw transfer refused HTTP 413 (268435456-byte
limit). Parsed the complete uploaded 8041-byte plist as data with Python
plistlib; SHA-256 of **plist only**:
d97ab0c8b33cc580bdd781996a037d44d97a3292fa32ed469d85b2026ff025c2.
The file is user-supplied extracted metadata, not independent proof of
the original DMG hash, official signature or installed image. No private
Drive link, original plist or proprietary binary is committed.

Observed fields: CFBundleIdentifier=com.microsoft.minecraft-edu,
CFBundleExecutable=minecraft-edu, CFBundleName=minecraft-edu,
CFBundlePackageType=APPL, CFBundleShortVersionString=1.26.33,
CFBundleVersion=1.26.33.0, CFBundleSupportedPlatforms=[MacOSX],
LSMinimumSystemVersion=10.15, DTSDKName=macosx26.1,
NSPrincipalClass=MCEApplication. URL scheme minecraftedu and document
type declarations for mcworld/mcshortcut/mcpack/mctemplate/mcaddon are
present. LSMultipleInstancesProhibited and LSRequiresNativeExecution
are absent; preserve unknown, not a proven absence of internal
singleton or architecture restrictions. No declared per-instance
profile route in these metadata fields. Entitlements, CPU slices,
actual executable signer and account storage are not in this plist.
The product manager's proposed macOS 14 floor is unchanged; a game's
minimum OS does not prove the capture/manager platform floor.

Use the observed exact bundle ID/executable in both existing discovery
and offline artifact collector; require numeric version/build (no
version admission pin), reject different identities before native
signature observation. Add the expected signed code identifier to the
existing one-shot Developer ID team requirement; actual signed identifier
must still be corroborated from the executable. Keep the same unreleased
wire status and existing IPC, update its UI wording. No generic fallback,
new package, new runtime route, profile or launch mutation. Authored
Rust and JS refusal fixtures NOT_EXECUTED; no build, CI or native test.

Next smallest artifact inputs: original DMG SHA-256 and the unchanged
`Contents/MacOS/minecraft-edu` Mach-O if transferable, not a renamed
or patched file. Static inspection can establish declared architecture,
embedded code-signing metadata and entitlement indications, but cannot
substitute Security.framework signature verification or live profile
isolation. If the executable exceeds transfer limits, request only
bounded metadata or agreed smaller evidence chunks instead of repeating
the failed DMG transfer. Mandatory five-client one-workspace parity,
all runtime/CI/test/release holds remain.

### MAC-06 — Windows-compatible artifact evidence route (2026-11-28)

The operator agrees to continue the remaining work without requiring a Mac
for static artifact review. Reconfirmed the official 26.33 download alias
redirects to the MAC-02 DMG; web retrieval is unsupported and an explicit
read-only HTTP HEAD from the available source environment failed DNS.
No bytes obtained. Do not retry indefinitely or substitute unrelated builds.

The [7-Zip official format list](https://www.7-zip.org/) supports read-only
extraction of DMG, HFS and APFS on Windows. Prepare the minimal manual
[offline evidence intake](../Development/tooling/README.md#macos-offline-package-review-from-windows)
for the existing artifact investigation: package SHA-256 and unchanged
Info.plist first; optional extracted main Mach-O only when required.
No Mac test, installer, new parser, runtime dependency or extraction
automation. Actual extraction compatibility of this specific DMG is
unverified. A Windows-extracted file cannot establish macOS signature,
Gatekeeper, loaded-image or profile isolation acceptance.

[Apple Remote Desktop](https://support.apple.com/en-gw/guide/remote-desktop/apd4f46319e/mac)
explicitly documents observing a virtual desktop belonging to a different
authenticated user without interrupting the console user. This supports
further research of topology B, not a supported same-host embedding API
or five concurrent interactive Minecraft sessions. [Apple screen sharing
limits](https://support.apple.com/en-ie/guide/mac-help/mchl1883115d/27/mac/27)
permit only one active High Performance session per Mac. Standard mode
same-host concurrency, gameplay input, performance and per-window
presentation remain unverified. No remote management enablement, user
creation, network listener or new service. Final shared workspace parity
is unchanged.

Decision: no further speculative product or synthetic source until a
decisive artifact or isolation mechanism is available. The first evidence
input is small package metadata, not a request for localtest. All previous
native, CI, test and release holds remain; NOT_EXECUTED. Actual Mac
integration remains BLOCKED.

### MAC-05d — current release evidence and isolation priority (2026-11-28)

OBSERVED_EXTERNAL, not locally verified: [Desktop Insights Minecraft-edu](https://desktopinsights.com/apps/minecraft-edu?platform=macos)
reports a static analysis of release 1.26.33, universal Intel/Apple Silicon,
Microsoft Developer ID team UBF8T346G9, Hardened Runtime and detected
OneAuth/MSAL libraries, with app-group and keychain-access-group entitlements.
Its published analyzed-binary SHA-256 is
1e45a044621d5cefee9221238c6d7e96b565d6ddbb92587d099f68d3be4f2613.
The exact hashed file scope and original archive provenance are not established
from the public page; do not equate this with the DMG SHA or pin an executable
from it. Its [extraction methodology](https://desktopinsights.com/blog/how-we-detect-sdks-in-macos-apps)
combines bundle metadata and strings; detection does not establish active
runtime paths. Corroborates MAC-05c team expectation and universal target
candidate, not an independent cryptographic verification, notarization
receipt, exact entitlements, bundle identifier or admission.

[Official changelog](https://edusupport.minecraft.net/hc/en-us/articles/360047556451-Minecraft-Education-Change-Log/)
confirms 26.33 released Oct 28, 2026. The official alias
https://aka.ms/meeclientmacos redirects to the same 26.33.0 DMG recorded
in MAC-02, but web retrieval of bytes failed; no artifact obtained.
[Official Cloud Update notes](https://edusupport.minecraft.net/hc/en-us/articles/23323615654164-What-s-New-in-the-Cloud-Update-v1-21-10)
document changed Mac sign-in libraries, Keychain permissions and OneDrive
world backup. Add cloud sync to future isolation acceptance (disposable
worlds and separate licensed accounts, verify independent cloud routing and
no cross-account overwrite; no token/credential enumeration).

[Apple TN3137](https://developer.apple.com/documentation/technotes/tn3137-on-mac-keychains)
specifies one data-protection keychain per user in the caller's login
context. [Apple access groups](https://developer.apple.com/documentation/security/sharing-access-to-keychain-items-among-a-collection-of-apps)
and [Microsoft MSAL](https://learn.microsoft.com/en-us/entra/msal/objc/howto-v2-keychain-objc)
describe app-signing-based group access and application-owned configuration,
not an external per-instance account override.

Independent alternative reviewed at exact
[PluralMacPlus 458d8e2](https://github.com/knutwurst/PluralMacPlus/tree/458d8e23f9baeb4bf830381e59f6ba8f94e8c13b):
README explicitly shares Keychain; DirectLauncher.swift retains shared
Keychains under redirected HOME; PreferencesIsolation.swift uses a
compatibility shim subject to hardened-runtime restrictions. Do not import
this solution: it cannot establish Education account isolation; private
app copies, injection, re-signing and device-ID changes are out of scope.
No supported same-user Education isolation switch found.

Decision: prioritize P1/U1–U4 distinct existing standard macOS user
contexts as the strongest current isolation candidate. This is a change
in investigation priority, NOT selection or implementation of topology B.
A remains preferred if a supported game-owned isolation route is found.
For B, P3 inactive-session capture and selected human interaction without
manual console switching are still mandatory before integrating a per-user
endpoint. Do not lower final five-client workspace parity. Explicit separate
consent required for user-session access; no root, account creation,
login automation, credential handling, signature modification or implicit
fallback. The existing fixtures and product discovery remain sufficient
for the current source-only phase.

Next decisive evidence inputs (no localtest request):
1. An unchanged official DMG copy can be provided from any OS, not necessarily
   Mac. Static package hash and metadata review are separate from runtime
   proof; compare the exact package with MAC-02's published checksum and
   inspect an authorized offline bundle. No install, execution or package
   bytes in Git. Existing MAC-03b collector remains the evidence owner.
2. A supported actual Education profile isolation mechanism, or later
   separately authorized evidence for U1–U4 and P3 with existing standard
   users and two licensed sessions. A third-party static report does not
   supply that evidence.

No runtime code changes, additional fixtures, dependencies or admission
gates in this checkpoint. No compiler, tests, CI or native execution.
Do not spend further source iterations on synthetic substitutes or
signing refinements that cannot resolve the missing mechanism. The actual
Mac integration remains BLOCKED; preserve all Windows and release holds.

### MAC-05c — expected signing team and deployment name (P1 partial)

New product-specific evidence: pinned third-party
[Installomator minecraftedu label](https://github.com/Installomator/Installomator/blob/6013fa5e6ef57fc599ba1cb7f727dd8fea963c90/fragments/labels/minecraftedu.sh)
(blob 6a5623594be4430507bc9cb18792236df308a1ee) expects Team ID
UBF8T346G9 for the official aka.ms/meeclientmacos download alias.
[Microsoft Intune](https://learn.microsoft.com/en-us/intune/device-configuration/templates/ref-kernel-extensions-settings-macos)
and [Microsoft Platform SSO](https://learn.microsoft.com/en-us/intune/intune-service/configuration/platform-sso-macos)
corroborate that Team ID for OTHER Microsoft products, not this exact game.
The pinned original [Microsoft Intune Minecraft sample](https://github.com/microsoft/shell-intune-samples/blob/a4f5eeaefe395fb36c70ea41823dabac94fa4db2/macOS/Apps/Minecraft%20Education%20Edition/installMinecraftEE.sh)
(blob eea0693f3d04a26788605145492d81f06e769664) uses the same official
download alias and historically installs as Minecraft EE.app. Add that
third fixed name, with the same ambiguity and symlink refusal; no script
execution, installation or update behavior imported.

Replace generic Apple-anchor observation (MAC-05b) with a stricter single
Security.framework static validity request: Apple anchor, Developer ID
issuer/application certificate OIDs and the expected Team ID in leaf OU
(Apple [TN3127](https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements/)).
This is an informational requirement candidate, NOT the reviewed Education
designated requirement. No exact code identifier assumed, no publisher pin
and no generic fallback or second signature validation. Result becomes
ExpectedTeamMatched or Unconfirmed; old generic AppleAnchored receipt is
rejected by the updated Mac parser. Mac schema remains unreleased; Windows
receipt is unchanged. publisher=NotPinned and runtimeAdmission=false are
mandatory, even for expected-team matches. The exact official DMG,
identifier, architecture-specific signatures and actual isolation remain
unobserved. Historical community [signature failure](https://edusupport.minecraft.net/hc/en-us/community/posts/31685896910228-Error-launching-Minecraft-Education-1-21-06-on-MacOS-Big-Sur)
is not a reason to re-sign or alter an installed game.

Source-only Rust/TS fixture updates cover the additional name, ambiguous
names, synthetic matched team, old generic and forged positive admission.
No new dependency or command, no build, test or native execution.
Unresolved: the third-party expectation could be stale; a legitimate
signer change must remain unconfirmed until reviewed. Static validation
still has no hard deadline and is not atomic. No official signer or Mac
profile acceptance claim. Next: actual artifact evidence and supported
per-instance storage/authentication route, not another speculative gate.
All prior holds stand.

### MAC-05b — static Mac signature and singleton evidence (P1 partial)

Extend the existing explicitly requested installation observation, not
another command, subprocess or controller. Use Apple Security.framework
[static validity](https://developer.apple.com/documentation/security/secstaticcodecheckvalidity%28_%3A_%3A_%3A%29)
with [all architectures, nested and strict validation flags](https://developer.apple.com/documentation/security/static-code-validation-flags),
single-threaded validation and the no-network flag. The generic requirement
`anchor apple generic` ([TN3127](https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements/))
requires an Apple-issued signing identity, not Microsoft; the requirement
is deliberately not a publisher pin. No assumed Team ID, notarization or
loaded-image proof. Use existing locked core-foundation 0.10.1 and
security-framework-sys 2.17.0 direct Mac-only edges; exact
[FFI source](https://docs.rs/security-framework-sys/2.17.0/src/security_framework_sys/code_signing.rs.html)
and [CFURL API](https://docs.rs/core-foundation/0.10.1/core_foundation/url/struct.CFURL.html)
reviewed. No new package version. The blocking native call has no hard
wall-clock deadline; actual cost and SDK/lock resolution NOT_EXECUTED.

Separate NotChecked (no unambiguous candidate), AppleAnchored (generic
static validation succeeds), Unconfirmed (any failure); no raw path,
certificate, signing IDs or error text returned. Recheck metadata after
the native call; this is not atomic against concurrent changes. Preserve
optional LSMultipleInstancesProhibited explicitly as true/false/unknown;
absence or false is not permission or evidence of an internal singleton.
The Mac wire parser and existing notice require publisher=NotPinned and
runtimeAdmission=false. Windows wire and launch behavior unchanged.

Authored synthetic signature outcome, ambiguity, singleton and metadata
replacement cases and TS negative receipt fixtures, NOT_EXECUTED; synthetic
callbacks do not prove platform validity. Actual unsigned/signed official
artifact and native checks remain unobserved. The existing offline
artifact collector remains the deeper provenance owner. No runtime
activation until exact official bundle, signer and isolation mechanism
are established. No CI, localtest, release or version change. Avoid
further speculative admission layers absent that evidence.

### MAC-05a — product-facing Mac installation candidate (P1 partial, source only)

Move the next work to actual product discovery, not further synthetic UI.
Official [Mac installation](https://edusupport.minecraft.net/hc/en-us/articles/360047118792-Mac-Installation-Guide)
uses Applications; [Homebrew cask](https://formulae.brew.sh/cask/minecraft-education)
documents minecraft-edu.app renamed Minecraft Education.app. Names are
discovery hints, never publisher pins. An attempted HTTP HEAD for the
MAC-02 official DMG from the source environment failed DNS resolution;
no DMG content or native evidence was obtained.

Add a Mac-only leaf to native_clients and reuse the existing read-only
installation command and resource admission. Only two fixed system
Applications names; refuse links, ambiguity, malformed metadata and
missing executable. Read at most 256 KiB of Info.plist, parse required
APPL/identifier/executable/version fields, check the ordinary executable's
presence/mode and recheck metadata. No executable bytes, game launch,
signature admission, settings, credentials, home scan or caller paths.
File checks are not atomic against concurrent changes; candidate metadata
can be imitated, and no nonstandard install locations are searched.
Use existing locked plist 1.10.2 and libc 0.2.191 as direct Mac-only
dependencies; no new package version or extra process. The existing lock
root dependency edge is authored, Cargo lock resolution NOT_EXECUTED.
[plist reader](https://docs.rs/plist/latest/plist/enum.Value.html) is the
API reference. Actual Mac compile and candidate behavior remain unknown.

Reuse the existing unsupported shell and one explicit Check installation
button, no startup scan or periodic polling. Mac-only optional capability
and tagged receipt, unchanged Windows report shape. Distinguish Missing,
Candidate, Ambiguous and Unconfirmed; runtimeAdmission always false.
No positive native backend, capture, settings or profile eligibility.
Authored Mac synthetic filesystem and TS wire refusal fixtures NOT_EXECUTED.
No new Tauri command, controller, helper, service or installer. Preserve
Windows build/test/release holds. The offline signature collector remains
the deeper evidence owner. Candidate discovery does not solve independent
storage or authentication. Next meaningful work requires actual artifact
metadata and a defensible profile mechanism; do not claim source-ready
Mac game management or expand speculative runtime plumbing.

### MAC-04g — shared snapshot panel and native focus (P3b partial, source only)

Extend the existing self-only fixture with explicit
`--shared-workspace-probe` and retain the previous three modes.
Reuse exact-window capture and marker validation; one AppKit panel displays
two latest snapshot images and the selected image (no independent encoder,
persistent service or capture loop). The user must select A then B in the
panel, then physically click each original native window. Selection must
originate while the panel is key and active. Request AppKit activation
once for the retained original application; observe the exact frontmost
application before proceeding. [AppKit 14 activation notes](https://developer.apple.com/documentation/macos-release-notes/appkit-release-notes-for-macos-14)
and [NSRunningApplication.activate](https://developer.apple.com/documentation/appkit/nsrunningapplication/activate(from:options:))
establish a request, not guaranteed focus. Refusal, stale target, wrong
selection or mismatched marker blocks further action; no forced activation,
input injection or fallback. Capture markers and independent normal exit
remain required. Correct the marker fill to documented NSBezierPath.fill.

This tests a real shared snapshot/selection consumer at source level, not
embedded interactive gameplay. Sequential snapshots may be stale between
observations; the panel must not be treated as a live player viewport.
No inactive-login capture, cross-user transport, actual Minecraft game
or profile isolation proof. No source compile/test/native execution and
no product changes. The P3b partial receipt explicitly keeps
embeddedGameplay and inactiveLoginSession NOT_VERIFIED, runtimeAdmission=false.
This is the terminal useful synthetic source step absent Mac access;
do not expand synthetic UI, streaming or per-user agents speculatively.
Final acceptance remains five independent interactive Minecraft sessions
in one workspace; P1 actual artifact/isolation and P3 real workspace
feasibility remain unresolved. Existing native/test/CI/release holds stand.

### MAC-04f — two exact native window snapshots (P3a, source only)

Extend the existing self-only Swift/AppKit fixture with explicit
`--workspace-probe`. The normal single and profile probe modes remain.
Requires macOS 14+ and pre-existing screen capture permission; no prompt
or elevated execution. Launch two children from one fixture bundle without
HOME overrides, display two native colored windows, and verify distinct
retained app identities. ScreenCaptureKit enumeration is in memory; filter
to exactly one window per verified original PID, fixture title and stable
CGWindowID. Refuse unknown or changed target. Use Apple's
[SCShareableContent](https://developer.apple.com/documentation/screencapturekit/scshareablecontent),
[window-only filter](https://developer.apple.com/documentation/screencapturekit/sccontentfilter/init(desktopindependentwindow:))
and [SCScreenshotManager](https://developer.apple.com/documentation/screencapturekit/scscreenshotmanager/captureimage(contentfilter:configuration:completionhandler:)).
No display capture, background stream, external image output, recording,
general input injection, Accessibility permission or new runtime dependency.

First bounded screenshot pair must show red A / blue B. Manual physical
click A changes A to green while B stays blue; then click B changes B
to yellow while A stays green. Re-capture both using original window IDs,
sample five interior pixels and reject missing, unexpected or stale markers.
Normal close A must leave B's original app alive; then normal close B.
All outputs are synthetic, runtimeAdmission=false. Failure preserves
the fixture and uncertain processes, never retries mutation, adopts or
force-closes a foreign process.

Proof ceiling: candidate source only, not compiled or executed. Three
sequential screenshot pairs are NOT live simultaneous previews. Manual
clicking actual native windows is NOT selected input through one workspace
and does not prove gameplay, cross-user/inactive-session capture or
interactive control. P3b would require a real shared preview/selection
consumer and scoped interaction evidence. Candidate B remains conditional;
no cross-user endpoint or service implemented. No changes to Windows,
product startup, CI, tests, activation or release holds. Do not repeat
synthetic work beyond its decision value: actual game isolation and
shared workspace evidence remain the decisive gates.

### MAC-04e — full workspace parity, staged design (2026-11-28)

Operator decision: the final Mac product must match the Windows five-instance
single-workspace workflow. Manual fast user switching is a feasibility step
only, not a final Mac-specific reduction in acceptance. No Mac test host is
available during the current source phase. The operator authorizes staged
design, not native execution, installation, privilege changes or release.
This decision supersedes the open workspace-choice question in MAC-04d/U5;
the earlier evidence and alternative feasibility constraints remain valid.

#### Choose the minimum complete topology, not a new provider system

A — Preferred if a supported same-user private profile route exists:
one manager, retained native controller, one verified game installation,
five owned game processes with independent persistent data/authentication,
native presentation and selected human interaction in the existing workspace.
Use AppKit/Accessibility and a window-specific ScreenCaptureKit preview only
if needed; macOS has no established equivalent to the current Win32/DWM
embedding contract. Do not assume window capture supplies gameplay input.

B — Conditional if same-user isolation cannot be established:
one shared game bundle, distinct explicitly consented standard users and
logged-in GUI sessions. Manual switching establishes isolation/persistence
first. A later minimal per-session native endpoint may observe and operate
only the game originally launched by that endpoint in its own login session;
one central manager remains the only five-slot catalog, command router,
selection owner and MCP/SDK authority. Endpoints are OS-bound leaves, not
five full Tauri apps, independent schedulers, provider plugins or alternate
catalogs. Reuse the same endpoint executable; no root daemon, no auto-login,
no installation or persistent service registration at this design stage.
The operator's parity decision does not authorize enabling a cross-user
control service. Candidate only: the endpoint topology must pass inactive
session capture AND selected interactive input feasibility before it can
replace A. If it cannot, B is not an acceptable final product.

Apple [Fast User Switching](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPMultipleUsers/Concepts/FastUserSwitching.html)
states switched-out processes can continue but lose physical input; the
display appears asleep. [Security login sessions](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPMultipleUsers/Concepts/SystemContexts.html)
have distinct identities that change on re-login. [XPC service types](https://developer.apple.com/documentation/XPC)
and [Service Management](https://developer.apple.com/documentation/servicemanagement/)
describe per-user agents and system-context daemons; these do not establish
direct cross-login-session connectivity. A local Unix socket with explicit
peer/user and session binding is a transport candidate, not a permission
grant; design only. No unauthenticated listener, global writable socket,
network exposure, shared credentials or PID-only ownership. A future
connection must bind the consented slot, peer identity, login generation,
reviewed code and exact retained target; reject replay, wrong slot,
disconnection, stale process or re-login. No retry of ambiguous mutation.
Disconnect disables input and marks status unavailable, not STOPPED.

Apple [window filter](https://developer.apple.com/documentation/screencapturekit/sccontentfilter)
supports a selected window; [capture guide](https://developer.apple.com/documentation/screencapturekit/capturing-screen-content-in-macos)
and [Accessibility trust](https://developer.apple.com/documentation/applicationservices/1459186-axisprocesstrustedwithoptions)
require explicit permission and distinct capture/control verification.
Neither proves usable inactive-session Minecraft pixels or gameplay input.
[Apple high-performance screen sharing](https://support.apple.com/en-az/guide/mac-help/mchl1883115d/27/mac/27)
allows only one high-performance session per Mac; do not adopt it as the
five-client workspace. [RustDesk discussion 13869](https://github.com/rustdesk/rustdesk/discussions/13869)
reports fast-user-switch freezes/disconnects; independent counterevidence,
not authoritative proof that all cross-session APIs fail. Do not copy its
root/watchdog workaround or add a general remote desktop stack.

#### Sequential implementation design and evidence gates

| Stage | Scope / smallest acceptance |
| --- | --- |
| P0 — shared source | MAC-03a–d shared UI/status/identity; MAC-04a diagnostic shell and MAC-03b artifact inspector; existing MAC-04b/c fixture. Source committed, not executed. |
| P1 — target / isolation | Exact official Mac bundle metadata and supported A mechanism, or U1–U3 for two explicitly consented existing users. No account/world reads in collector; actual persistence and manual independent licensed sign-in must later be observed. No fake recipe. |
| P2 — native lifecycle | Narrow launch/inspect/normal-close for two owned sessions, reuse five-slot selection and receipt rules. Missing identity, unknown launch/exit or peer loss blocks mutation. B uses only the minimum session-local endpoint after separately approved design; no automatic service. |
| P3 — decisive workspace feasibility | With two disposable synthetic windows first, verify simultaneous fresh, correct window-specific previews while selected interactive input reaches only its intended app, including switched-out sessions for B. Verify permission refusal, locked/logged-out/changed session, no stale frame or input redirection. Then repeat for real licensed game windows; normal switch must not require changing the console user. A preview alone or fast user switching is not a pass. |
| P4 — integrate 2 to 5 | One controller/UI/catalog/agent core, one selected playable viewport, four visible preview tiles, explicit independent launch/close/recovery; same path for MC01–MC05. Preserve Windows contract and source tests. No automatic cross-host data migration. |
| P5 — optimize and distribute | Measure startup, manager and total system memory/CPU/GPU, latency, frame times and background multiplayer with equivalent workload. Only then choose preview/frame/audio policy and Mac package signing/notarization; release separate. |

Efficiency rule: prefer native placement for the selected game wherever it
meets interaction acceptance, avoid encoding if not needed. If capture is
necessary, select the exact owned window, use one active high-quality
view and lower-cost visible preview streams, suspend hidden views, and
reuse bounded latest-frame buffers rather than an unbounded frame backlog.
[SCStreamConfiguration.minimumFrameInterval](https://developer.apple.com/documentation/ScreenCaptureKit/SCStreamConfiguration/minimumFrameInterval)
and [queueDepth](https://developer.apple.com/documentation/ScreenCaptureKit/SCStreamConfiguration/queuedepth)
are capture throttles, NOT Minecraft render/FPS controls; start from the
platform default bounded queue and adjust only after measurement. Audio,
recording, optional AI input and hardware encoding remain separately gated;
no fixed performance claims or five encoders by default.

Stop condition for source design: no speculative production adapter or
duplicated state owner until the decisive isolation and workspace route is
selected on real evidence. Next bounded source task, if useful, is a
synthetic two-window workspace feasibility fixture with explicit user consent
and no Minecraft dependency; design now, execution remains held. Do not
silently implement B or demand localtest. All product runtime, build,
capture/input, CI and release gates remain unchanged. No new runtime source
or dependency in this design checkpoint.

### MAC-04d — native isolation decision and alternate topology (2026-11-28)

New source evidence changes the investigation priority, not product acceptance:
- Minecraft Education [Wild Update 1.19.53 notes](https://edusupport.minecraft.net/hc/en-us/articles/13961617817236-What-s-New-in-The-Wild-Update-v1-19-53)
  document the switch to MSAL. Current [Manage Accounts](https://edusupport.minecraft.net/hc/en-us/articles/360061382672-Sign-Into-Minecraft-Education)
  describes last-player persistence and switching, not concurrent private sessions.
- Microsoft [MSAL macOS cache configuration](https://learn.microsoft.com/en-us/entra/msal/objc/howto-v2-keychain-objc)
  and [MSAL SSO](https://learn.microsoft.com/en-us/entra/msal/objc/single-sign-on-macos-ios)
  document a default shared keychain group and app-owned cache configuration.
  This is evidence against assuming HOME/TMPDIR isolation is sufficient,
  NOT proof of the current Education build's actual token-cache configuration.
  Never read token contents, change keychains, patch/sign the official game,
  or configure MSAL on behalf of the game.
- [Official world locations](https://edusupport.minecraft.net/hc/en-us/articles/4404785703316-Location-of-World-Files)
  are device-user relative. [Apple concurrent login sessions](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPMultipleUsers/Concepts/SystemContexts.html)
  establish native user/session boundaries. [Fast User Switching](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPMultipleUsers/Concepts/FastUserSwitching.html)
  permits background sessions but only the active console receives local
  keyboard/mouse input. [Launch Services restrictions](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/LaunchServicesKeys.html)
  can also prohibit second launches across users.
- Historical [official community reply](https://edusupport.minecraft.net/hc/en-us/community/posts/360072475972-Multi-World-Setup?sort_by=votes)
  (2019-01-25) rejects multi-instance use on one machine in a Windows-host
  question. It is counterevidence, not a current 26.33 macOS determination.
  No current official Education per-instance profile option found in reviewed
  installation, world, account or community sources. DMG direct web retrieval
  unavailable; actual bundle/entitlements/signature still uninspected.

Decision: same-user AppKit/private-profile remains the preferred final
topology if a supported concrete game route is established, but no additional
HOME-only fixture or generic abstraction can solve authentication. Prioritize
evaluating separate existing standard macOS accounts as an alternative
feasibility candidate; do not auto-fallback or silently replace the final
five-client single-workspace requirement.

#### Alternate topology — proposed only

Use one official installed game bundle shared read-only between users,
one normal licensed game session within each explicitly logged-in standard
macOS account, separate native home/preferences/keychain contexts, manual
Minecraft sign-in and normal OS user switching. Do not create users, use
root, automate login, collect credentials, change game registration or
adopt preexisting unrelated sessions. No helper/daemon or cross-user IPC
is required for the minimal manual feasibility phase. A controller launched
in one user session cannot claim ownership of other sessions merely from PID
or bundle ID. The existing Rust/Tauri app and common session contracts remain
the eventual product; a separate user actor, authenticated IPC or screen
sharing integration is a material topology change requiring operator
decision, not an implementation shortcut. No separate product or release.

| Ordered feasibility gate | Required later evidence |
| --- | --- |
| U1 — one official app | Exact admitted game image/signing and two existing consented standard users with distinct homes and GUI login sessions. No account creation or local execution in this phase. |
| U2 — two native clients | Two distinct licensed accounts, simultaneous native processes in different GUI sessions; verify app-wide singleton does not refuse, each remains playable in its own active session. |
| U3 — persistence | Distinct synthetic world/settings changes and normal close/reopen preserve own data; account sign-out/relogin on one does not affect the other. No real world or credential values in receipts. |
| U4 — usability | Verify background multiplayer continues during manual switching; document input, audio, rendering, network and measured resource limits. No claim that switching provides simultaneous embedded previews or direct input. |
| U5 — integration decision | If U1–U4 feasible, explicitly decide whether manual session switching is acceptable as a Mac-specific workspace or require a reviewed per-user bridge to meet single-workspace parity. If not, no admitted Mac multi-instance route. Scale to five only after acceptance. |

The alternative deliberately trades workspace parity for stronger native
isolation. Same-user route cannot be approved solely from MAC-04c positive
synthetic paths; alternate route cannot be approved solely from OS login
semantics. Preserve all prior source/activation/CI/test/release holds.
No new source module or test fixture: decision checkpoint only. Do not
spend further source effort on speculative game-independent abstractions.
Workspace acceptance decision resolved by MAC-04e above: manual switching
is interim only; full shared workspace remains mandatory. Actual artifact
and live validation remain deferred; no localtest request.

### MAC-04c — one bundle, two synthetic profile contexts (source only)

Extend the existing AppKit lifecycle fixture, not the product. The original
no-argument single-app route remains; explicit `--profile-probe` launches
two synthetic children from the same unique bundle/identifier. Two fresh
fixture-only directories receive process-local HOME/TMPDIR values; no global
environment mutation, real Minecraft process, world, preferences or Keychain
content access. Apple [launch environment](https://developer.apple.com/documentation/appkit/nsworkspace/openconfiguration/environment)
can be ignored for sandboxed callers. Apple's [file-system guidance](https://developer.apple.com/documentation/foundation/using-the-file-system-effectively)
and [preference domains](https://developer.apple.com/documentation/corefoundation/preferences-utilities)
show why process-local environment alone is not a profile-isolation contract.

Each child observes only path resolution (environment HOME, Foundation home,
Application Support, Caches, temporary directory); the fixture parent binds
a bounded private receipt to the retained app PID and slot. Require different
PIDs and app objects with the same exact bundle/executable; normal close A,
confirm B remains live, normal close B. No adoption, force close, auto-retry
or cleanup of uncertain outcomes. Public receipt contains per-slot booleans
only. Even a positive synthetic path result leaves preferences, authentication
and Minecraft isolation NOT_OBSERVED/NOT_VERIFIED and runtimeAdmission=false.

Implementation decision: a process-local path recipe is only a candidate
for cooperating applications. If Minecraft has a supported profile route,
bind it to existing five-slot catalog/lifecycle and verify every mutable
namespace before admitting launch. If it does not, the OS-user route remains
a distinct design option with per-login GUI constraints and separate approval;
do not build a second controller or enable an implicit fallback. Do not guess
game switches, intercept credentials or port Windows DLL hooks. No product
dependency, native backend or new fixture file. Compilation and both native
fixture modes NOT_EXECUTED; current Mac artifact and actual game isolation
remain unresolved.

### MAC-04b — native AppKit lifecycle feasibility fixture (source only)

Author one isolated `Development/tooling/acceptance/macos/lifecycle-fixture.swift`
program for a future separately authorized Mac acceptance phase. It uses
Apple AppKit/Swift already available through the developer toolchain, not a
product dependency, second runtime, user profile or service. The fixture
creates a fresh private temporary bundle containing its own executable and a
unique bundle ID; no arbitrary app path, elevated execution, game launch,
credentials or world access. No automatic execution or deletion.

Use [NSWorkspace.openApplication](https://developer.apple.com/documentation/appkit/nsworkspace/openapplication(at:configuration:completionhandler:))
with substitution disabled, new instance requested and no activation/recent
items; retain the original [NSRunningApplication](https://developer.apple.com/documentation/appkit/nsrunningapplication).
Match bundle/executable URL and unique identifier; compare a fresh process
object with the original using isEqual rather than PID alone
([Apple processIdentifier](https://developer.apple.com/documentation/appkit/nsrunningapplication/processidentifier)).
A bounded main-run-loop wait precedes normal terminate; wait separately for
isTerminated, since a quit request is not proof of exit. Unknown launch,
identity or close does not retry, adopt, kill or delete. Receipt always
runtimeAdmission=false, even when the synthetic exit is observed.

This is executable candidate source, not a compiled/observed result.
AppKit launch completion/run-loop delivery, self-copy signing, normal
termination and actual timing remain NOT_EXECUTED. The fixture is not
a game publisher/image proof or an independent profile. No product
module, native command, background worker or change to Windows behavior.

Next source design: build the actual Mac profile mechanism only when
a defensible storage and login route is identified. Same-user isolated
profile is primary; separate existing macOS users is an explicitly
different native topology candidate, not an auto-fallback. The synthetic
fixture makes native lifecycle work concrete without claiming the Mac
game backend is available or requesting Mac tests now.

### MAC-04a — separate source development from native activation

The operator has no current Mac test access and wants staged work to continue.
Do not make local testing a prerequisite for source preparation. Native
activation and release still require matching later evidence.

One explicit `dev:macos` route uses the existing locked Tauri/Svelte/Rust
desktop and native host, not Windows PowerShell, DLL staging or a second app.
The platform-specific Tauri overlay gives a separate dev identifier, disables
bundling and selects Mac icons; existing icon generator also checks ICNS on
Mac. Tauri [platform merge](https://v2.tauri.app/develop/configuration-files/)
and [prerequisites](https://v2.tauri.app/start/prerequisites/) are the source
references. Windows default scripts, native and release gates remain unchanged.
Authored config/route fixture NOT_EXECUTED; actual Mac compile and app startup
NOT_EXECUTED. The diagnostic shell is not a Mac backend or release candidate.

Stage native design/implementation against bounded acceptance, with
activation OFF until verified:

| Stage | Planned native work and acceptance |
| --- | --- |
| A — shell | One explicit host-native development route; ordinary window/close, Unsupported backend, no game or profile access. Source prepared. |
| B — target | Reuse existing artifact inspector; pin actual Microsoft bundle, architecture, signer and launch behavior. Source collector exists; missing artifact remains unknown. |
| C — lifecycle candidate | One explicitly selected, disposable current-user game session as a later opt-in feasibility milestone, not a managed private slot or changed product acceptance. Use AppKit original application object, no adoption/force-close, normal quit and retained exit evidence. Source design can precede Mac runtime validation; no actual launch admitted until provenance and disposable-user boundaries are established. |
| D — independent profiles | Prefer verified same-user per-instance native game profile mechanism. Only accept documented/verified routing of worlds, settings, cache, preferences, login/broker and singleton, with no shared mutable state. No invented environment flags or credentials. |
| E — alternative native isolation | If D is unavailable, evaluate separately consented existing standard macOS users and explicit logged-in GUI sessions, one official app installation. Apple [login sessions](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPMultipleUsers/Concepts/SystemContexts.html) and [Fast User Switching](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPMultipleUsers/Concepts/FastUserSwitching.html) support OS-user boundaries but also restrict cross-session window/IPC access and permit app-wide singleton refusal. A per-user actor or screen-sharing bridge would change topology; not authorized/implemented as fallback. Use no privilege escalation or automated login. Evaluate separately if necessary. |
| F — convergence | Only after verified independent two-client mechanism, connect existing five-slot controller contract, typed native identity, shared UI and MCP; scale 2 to 5 without new providers. Workspace, capture, optimization and packaging follow in scope. |

Mac OS user path is documented in [Minecraft world locations](https://edusupport.minecraft.net/hc/en-us/articles/4404785703316-Location-of-World-Files);
AppKit [new instance](https://developer.apple.com/documentation/appkit/nsworkspace/openconfiguration/createsnewapplicationinstance)
and [environment](https://developer.apple.com/documentation/appkit/nsworkspace/openconfiguration/environment)
are launch tools, not proof of private storage. One-client diagnostic
milestone does not satisfy final five independent playable account acceptance.
No VM, app patch, signature changes, global path swap, credential handling,
new service, release, CI or test execution. The next source unit is a
bounded native lifecycle feasibility design, not more common-code
refactoring. No positive Mac capability until actual admission.

### MAC-03d — native identity boundary (source only)

Centralize frontend exact window observation equality and transient selection
key in existing observedProcess.ts. Scheduling compares responsiveness
separately; DWM invalidation keeps its slot and RUNNING guards; selection
keeps identical key bytes. Rust process_identity.rs already owns exact
Windows wire equality. No new identity store, process scan, data access,
IPC, permission or native mutation. Existing native capture and MCP
receipts have distinct verification semantics and remain unchanged.
Authored fixtures cover identity replacement, responsiveness, absent target,
and rejection of a speculative Mac shape; tests NOT_EXECUTED.

Mac candidate: retain the original NSRunningApplication returned by an
admitted launch and compare application objects, not PID alone (Apple
[processIdentifier](https://developer.apple.com/documentation/appkit/nsrunningapplication/processidentifier)
and [NSRunningApplication](https://developer.apple.com/documentation/appkit/nsrunningapplication)).
The Apple [proc_bsdinfo source](https://github.com/apple/darwin-xnu/blob/main/bsd/sys/proc_info.h)
contains start seconds and microseconds; a native exact-size proc_pidinfo
observation may corroborate process birth and user context, not replace
retained ownership or code-signing evidence. Use a typed platform-specific
receipt when there is a real admitted consumer; do not reinterpret FILETIME,
HWND or Windows session IDs. Window identity is separate from app identity;
Accessibility target and capture window identity must be verified against
the retained app before use, not inferred from caption, bundle ID or PID.
Normal quit is a request, not proof of termination. Missing/changed identity
must refuse selection and mutation, preserving the original session.

Do not introduce a new wire variant or SDK dependencies before the supported
Mac isolation/launch mechanism and actual native consumer exist. This
completes the bounded common identity preparation, not native Mac process
observation. Further speculative refactoring does not resolve MAC-02.
Mac artifact, storage/account separation and launch remain BLOCKED.

### MAC-03c — shared session contract (source only)

Move the existing five-state status projection and five-slot normal-close
selection from the active Windows controller into the existing session
protocol owner. The Windows controller uses both directly; no duplicated
session registry, transport, runtime dependency or native launch path.
Preserve all five wire strings and schema, exact slot semantics, and
native process/window verification, storage checks, reaping, close,
uncertainty and admission. Pure fixtures cover retained/missing window,
unresponsive/issue, absent storage, all five slots and invalid close
targets, independently of OS native APIs; authored NOT_EXECUTED.
Do not reuse the historical four-slot runtime-core identity as a live
five-profile abstraction. Mac identity/storage remain unresolved;
no positive Mac backend or profile claim. A future real Mac owner must
provide verified observations and retain native lifecycle authority.
The design permits comparing OS-user separation as research only
(no user creation, remote desktop fallback or changed acceptance).
No compiler/test/native execution, release or version change.

### MAC-03b — explicit Mac artifact inspection (source only)

Authored `Development/tooling/acceptance/macos/education-target.mjs` using
existing Node and macOS plutil/lipo/codesign. Explicit offline .app only;
no host data discovery, credential read, installation, game launch, new
runtime dependency or controller. Collect bounded bundle metadata, SHA-256
of executable, architecture-specific signature identity/requirement hash and
entitlement presence; check path/metadata stability. A valid signature is
not a publisher pin or official package provenance. Absent singleton key or
entitlement output remains unknown; all receipts deny runtime admission.
No automatic invocation; pure fixtures added to existing native-feasibility
route; source review only, NOT_EXECUTED. No DMG bytes or Mac target available.
MAC-02 isolation gate remains BLOCKED; no alternative provider or fake Mac
launch. Tooling is an evidence collector, not a runtime adapter. Apple refs:
[code identity](https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements/),
[architecture slices](https://developer.apple.com/documentation/apple-silicon/building-a-universal-macos-binary),
[entitlement display](https://developer.apple.com/documentation/technotes/tn3125-inside-code-signing-provisioning-profiles).

### MAC-03a — shared UI session capability (source only)

Extract the existing positive Windows session check into the typed bridge's
`supportsManagedSessions` predicate. Only WindowsNative is accepted today;
Unsupported, null and forged/new backend labels do not enable sessions.
The existing wire enum, Rust authority, feature gates, slot catalog,
DWM/geometry/release guards, timing and one UI dispatcher are retained.
Use the predicate for common session observation, settings/navigation,
feedback, action dispatch and titlebar close wording. Adapt the existing
frontend negative fixtures and repository source witnesses; no new state,
workers, dependencies or native calls. Tests/typecheck NOT_EXECUTED.
This is a bounded shared-source preparation, not native Mac admission.
MAC-02 artifact and isolation blocker remains; no automatic fallback.

### MAC-02 — artifact candidate and isolation findings (2026-11-28)

OBSERVED_EXTERNAL: [official changelog](https://edusupport.minecraft.net/hc/en-us/articles/360047556451-Minecraft-Education-Change-Log)
identifies retail 26.33 (2026-10-28). The [pinned upstream Homebrew cask](https://github.com/Homebrew/homebrew-cask/blob/e4b5deed8c87229fd51ee3a71aff7f8679053419/Casks/m/minecraft-education.rb)
and [cask API](https://formulae.brew.sh/api/cask/minecraft-education.json)
identify the candidate below. The SHA is a third-party published
download checksum, NOT an independently computed hash, a signer pin,
or runtime admission.

- Candidate: `https://downloads.minecrafteduservices.com/retailbuilds/MacOS/Minecraft_Education_1.26.33.0.dmg`
- Expected DMG SHA-256 from cask:
  `80bb151b1db4e46edc503230bc8ece25210fff94b7d73d6774559a054398a91e`
- Bundle entry: `minecraft-edu.app`; Homebrew installs it as
  `Minecraft Education.app`. Names are discovery hints only.
- Cask lists both `com.microsoft.minecraft-edu` and
  `com.microsoft.minecraftpe` preference/cache paths; neither is proof
  of the current bundle ID. Preserve all candidate legacy data; do not
  invoke the cask's zap or uninstall rules.
- Official [Mac update guide](https://edusupport.minecraft.net/hc/en-us/articles/360047705032-Update-to-a-New-Version-of-Minecraft-Education)
  describes manual installation. Do not port Windows updater-task semantics.

| Mechanism | Evidence and decision |
| --- | --- |
| AppKit new instance | [New instance](https://developer.apple.com/documentation/appkit/nsworkspace/openconfiguration/createsnewapplicationinstance) supports a launch request; [LSMultipleInstancesProhibited](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/LaunchServicesKeys.html) can refuse it. Actual plist and internal game singleton unknown. Candidate only. |
| Per-process environment or argument | [Apple environment](https://developer.apple.com/documentation/appkit/nsworkspace/openconfiguration/environment) and [arguments](https://developer.apple.com/documentation/appkit/nsworkspace/openconfiguration/arguments) are ignored for sandboxed callers; [NSHomeDirectory](https://developer.apple.com/documentation/foundation/nshomedirectory%28%29) uses the user's or sandbox's home. No verified Education profile argument. Do not assume HOME redirects all files or identity. |
| Separate Microsoft accounts | [World locations](https://edusupport.minecraft.net/hc/en-us/articles/4404785703316-Location-of-World-Files) explicitly use the device login user, not the game account. [Manage Accounts](https://edusupport.minecraft.net/hc/en-us/articles/360061382672-Sign-Into-Minecraft-Education) can remember the last player or switch accounts; no concurrent isolated login guarantee. |
| App Sandbox / Keychain | [Apple sandbox](https://developer.apple.com/documentation/security/accessing-files-from-the-macos-app-sandbox) binds container to signed app identity on macOS 14+; [Keychain groups](https://developer.apple.com/documentation/security/sharing-access-to-keychain-items-among-a-collection-of-apps) bind protected items to authorized groups. Neither is a supported arbitrary per-instance profile namespace. Do not modify signature, entitlements or credentials. |
| Alternative Bedrock launcher | [mcpelauncher upstream](https://github.com/minecraft-linux/mcpelauncher-manifest) uses a separate launcher/Android compatibility stack; its own [paths](https://mcpelauncher.readthedocs.io/en/latest/faq/index.html) do not establish official Education native compatibility. Reject as replacement. Java launcher gameDir and Education Classroom Mode multi-instance are different products, not evidence. |

Smallest discriminating artifact review for a future authorized evidence
source: read the exact DMG checksum, app Info.plist (CFBundleIdentifier,
CFBundleExecutable, CFBundleVersion, CFBundleShortVersionString,
LSMinimumSystemVersion, LSMultipleInstancesProhibited if present), executable
Mach-O slices, signed bundle integrity, designated requirement and relevant
sandbox/app-group entitlement presence. Use macOS standard metadata
tools (shasum, plutil, lipo, codesign) on the authorized copy; no installer,
launch, credential enumeration, data migration, entitlement changes or
Gatekeeper override. Keep only sanitized metadata, no package bytes in Git.
[Apple designated requirement](https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements/)
is a signature identity, not proof of profile isolation. Missing plist key
is not proof that the game allows multiple instances.

Decision rule: a valid image and supported launch mechanism are necessary,
not sufficient. Before any managed Mac launch, establish a supported
per-instance game data AND account mechanism with no shared mutable roots,
including settings, caches, worlds and authentication broker behavior.
If no supported mechanism exists, keep the Mac route unsupported; a
single-client Mac variant or other-user workflow changes product acceptance
and requires a separate design decision. Do not open a second runtime
or call a normal user game session an isolated profile.

Proof: official documents, exact upstream cask source and existing
Windows source/unsupported gates reviewed. DMG bytes, actual Info.plist,
Mach-O, signature, entitlements and native login/storage behavior NOT
INSPECTED; no host or execution evidence. No new runtime module, dependency,
CLI, test run or version bump. MAC-02 advances provenance and narrows the
blocker, not Mac implementation readiness.

### Minimal architecture — proposed, not implemented

Keep UI -> typed bridge -> allowlisted Tauri commands -> retained Rust
controller -> native game. Bind OS-specific primitives at compile time;
no runtime provider chooser, fallback, second controller, VM, new service,
extra polling, credential store or game patch. Share policy only where
needed by a second concrete consumer; do not mount historical provider
traits as another runtime. Preserve Windows public commands, wire receipts,
slot IDs, private data and existing admission gates.

| Existing owner | Planned boundary |
| --- | --- |
| `engine/native_clients.rs` | Official target admission; Mac bundle/signature/version/architecture, unchanged Windows PE/DLL checks. |
| `independent_bootstrap/session_controller.rs` | One session lifecycle/state owner; extract neutral policy only as needed, retain native handles and uncertainty/recovery. |
| `native_clients_session.rs` + `session_protocol.rs` | Shared validation, framing and admission; Windows pipe, candidate Mac Unix socket with exact peer/image verification, no replay. |
| Existing product profile/storage owners | One five-slot catalog per host; verified private backing per OS; no automatic migration or credential copying. |
| `process_identity.rs` | Preserve Windows receipts; explicitly tagged/versioned Mac identity if needed; no invented FILETIME/HWND. |
| `workspace_surface.rs` | Same presentation contract; DWM or AppKit/Accessibility, optional ScreenCaptureKit after explicit permission and exact target validation. |
| Capture and agent core | Keep capability gates; no automatic capture/input/recording parity; NVENC is not a Mac requirement. |
| `LazyTesterApp.svelte`, `app/bridge/lazyTester.ts`, presentation/titlebar consumers | Replace OS-specific readiness comparisons with shared capability decisions only when real backend is admitted. Keep Unsupported fail-closed. |
| `agent-configuration.mjs`, `native-transport.mjs` | Same MCP/SDK core and same-image CLI; review .exe/path assumptions with real Mac executable. |
| `build.rs`, Tauri config, existing distribution/developer tooling | Preserve NSIS gates; later native .app/DMG and signing/notarization; no second build system. |

Naming: use `windows.rs` / `macos.rs` under the actual owner only when
needed; preserve clear existing names, no blanket rename or empty
scaffolding. Proposed target priority macOS 14+, Apple Silicon first;
Intel not excluded or declared compatible without artifact evidence.

### Isolation feasibility — blocking gate

Official [Mac installation](https://edusupport.minecraft.net/hc/en-us/articles/360047118792-Mac-Installation-Guide)
and [requirements](https://edusupport.minecraft.net/hc/en-us/articles/360047556591-System-Requirements)
confirm Mac 14+ support. Official [world locations](https://edusupport.minecraft.net/hc/en-us/articles/4404785703316-Location-of-World-Files)
show a device-user-relative Library/Application Support/Minecraftpe/Games/com.mojang
root, not a per-instance root. Apple [new instance](https://developer.apple.com/documentation/appkit/nsworkspace/openconfiguration/createsnewapplicationinstance)
and [launch environment](https://developer.apple.com/documentation/appkit/nsworkspace/openconfiguration/environment)
APIs do not establish independent Minecraft storage, singleton or login.
[App Sandbox](https://developer.apple.com/documentation/security/accessing-files-from-the-macos-app-sandbox)
and [Keychain access groups](https://developer.apple.com/documentation/Security/sharing-access-to-keychain-items-among-a-collection-of-apps)
are app/signing-associated, not arbitrary instance isolation.
[Fast user switching](https://support.apple.com/en-euro/guide/mac-help/mchlp2439/27/mac/27)
does not establish simultaneous shared-desktop interaction. Do not use as
fallback. No VM, duplicate game copies, global HOME/root swaps, ad-hoc
re-signing, security bypass, credential inspection/copying or invented flags.

Prefer a supported, verified per-instance profile mechanism. None established
in reviewed references. Exact official Mac bundle artifact, identifier,
signing requirement, Mach-O architectures, storage/authentication routing
and singleton behavior remain unknown. Preserve Unsupported and refuse
managed launch until a safe mechanism is established; two PIDs are not proof.

### Implementation and acceptance sequence

1. MAC-02: candidate package identified; actual artifact metadata and
   verified independent storage/account mechanism remain outstanding.
   Obtain sanitized image/architecture/signature and boundary evidence. No binary or secret in Git; no execution now.
2. MAC-03: implement narrow owner-to-consumer slice, one controller and
   two independent persistent profiles, then reuse slot path for five.
   Author fixtures for shared-root rejection, changed image, stale identity,
   unauthorized IPC peer, uncertain close and unsupported capability.
3. MAC-04: integrate shared UI and agent flow, explicit manual launch and
   sign-in, native presentation. Apple
   [NSRunningApplication](https://developer.apple.com/documentation/appkit/nsrunningapplication)
   and [ScreenCaptureKit](https://developer.apple.com/documentation/ScreenCaptureKit)
   are candidates, not integrated. No unverified permission/features.
4. MAC-05: target-specific packaging after source/admission review;
   [Tauri distribution](https://v2.tauri.app/distribute/) signing and
   notarization for external distribution. Release approval separate.
   Later separately authorized native evidence: two then five independent
   licensed playable accounts/worlds, separate persistent writes, normal
   close/relaunch/reconnect, permission-denied behavior, preservation,
   CPU/RAM/GPU/frame time/startup and Windows regression. No estimated
   performance claim or source-only PASS.

MAC-01 and MAC-02 change planning only; MAC-03a changes common frontend
source and matching fixtures; MAC-03b adds explicit offline artifact tooling;
MAC-03c shares existing pure session policy with its active native caller;
MAC-03d centralizes active frontend window identity comparisons;
MAC-04a adds the explicit non-product Mac development shell route;
MAC-04b authors a synthetic AppKit lifecycle acceptance fixture;
MAC-04c extends it with explicit two-instance path observations;
MAC-04d records authentication counterevidence and alternate native topology;
MAC-04e records operator-confirmed final workspace parity and staged design;
MAC-04f adds same-login synthetic window capture/interaction feasibility source;
MAC-04g adds a bounded synthetic shared snapshot/selection panel;
MAC-05a adds an explicit product-side Mac installation candidate observation;
MAC-05b adds generic static signature and explicit singleton metadata observation;
MAC-05c tightens candidate signature observation with reviewed expected team
and adds the historical Microsoft Intune installation name;
MAC-05d records current-release external binary evidence and prioritizes
OS-user isolation feasibility without changing product acceptance;
MAC-06 records the offline Windows artifact intake and virtual-desktop limits;
MAC-06a records actual user-supplied bundle metadata and exact discovery identity;
MAC-06b adds bounded cross-platform Mach-O evidence intake after transfer failure;
MAC-06c records observed provisioning allowlist and authentication framework metadata;
MAC-06d records bounded Mach-O observations of two actual bundled frameworks;
MAC-07a reviews Standard-session transport and exact-target admission boundaries;
MAC-07b adds a test-only synthetic transport admission contract;
MAC-07c maps public native APIs and identifies the remaining trusted rendezvous gate;
MAC-07d prepares a one-shot cooperative cross-session rendezvous witness;
MAC-08a restores same-user priority and adds an opt-in synthetic preference witness;
MAC-08b reviews a historical native home-routing candidate and adds a comparison variant.
No new native backend, dependencies,
fake adapters, test execution or weakened gates. Builds, native/gameplay/performance,
packaging and tests NOT_EXECUTED. Next: BLOCKED on verified Mac artifact
and defensible isolation mechanism. Preserve all existing holds.

## Previous source closure — zero-legacy-runtime (unchanged)

Branch `Local`; Unreleased. Work: `zero-legacy-runtime`.
Approved goal: one retained native runtime, canonical profile storage, one agent
contract (MCP and SDK adapters), no extra worker/dependency or redundant owner.
Previous MOD/SIM-MOD source closure and exact history:
[15cd0072 handoff](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/15cd0072fcb0e78d74ae661a6092eb4a5eac8851/planning/development.md).

ZLR-01: retired executable pair/five startup and five-profile preparation CLI.
Retired flags fail before dispatch, old runner and legacy profile creation
removed; shared controller journal and test-only historical observer preserved.
Reviewed existing data/replay retained until safe storage cutover. Pair-only
historical data preserved, no automatic expansion. Source fixture NOT_EXECUTED.

ZLR-02: product runtime/binding/location admit verified schema 2 only; non-product
managed launch refused. Explicit existing migration command accepts verified
schema 1 or five-slot replay only, holds original manager/controller gates,
preserves source catalog/data and switches the canonical locator after five
target stores reopen. Schema 1 destination is selected-root/Migrated; old
replay uses unused default Data. Partial target refuses; explicit Locate of
verified complete schema 2 is recovery. Old root/binding presence prevents
fresh setup; no auto-import, fallback or pair expansion. Fixtures NOT_EXECUTED.
ZLR-03: one normal 23-tool catalog, same MCP/SDK tool names and core. Former
surface selectors and Probe prepare/read aliases refuse; trusted new
createAgentClient(core) derives methods from the original catalog, retains
budgets/closure and refuses experimental attachment. Existing explicit MCP
input gates, private checkpoint, full metadata and native authorities remain.
Explicit WebP requests share compact visual receipts; no auto-format fallback.
Source fixtures authored/adapted NOT_EXECUTED. Old host configurations require
operator review; no automatic permission widening.
ZLR-04: compare baseline 15cd0072 to 76cc5d77 (3 commits, 31 paths); review
selected active Rust storage/controller, MCP/SDK/type/catalog, fixtures,
package/verifier and recovery documentation consumers. Canonical logical
AppData validation no longer resolves a legacy replay parent; legacy HKCU
creation fixture-only. Migration no-op receipt reports 0, destination creation
and identity pins tightened, overlapping source/destination refused. One agent
catalog selected per core; Probe schema and private entry reconciled. Fixtures
NOT_EXECUTED. No new dependency/worker. Source phase closed at scoped proof;
external host configurations and exhaustive semantic review not verified.
Next: STOP (SOURCE_READY_FOR_LOCAL at scoped source proof only, not execution approval).
No compiler, test, CI, runtime, data migration or performance execution.
No release or version bump.

## Prior source work and continuity

UPD, RPT-01–07, CAP-01–05, HW-01, FIN-01–08, REV-01–08, BS-01–07,
FLOW-01–09, MCP-01–08, OPT (except OPT-07), E2E-01–06, BS-01–10,
SIM/EFF/DDR/NGE/FMS/DST/Rust-first and ICE-01–04/PLA scopes remain
source-closed, not executed acceptance. OPT-07, ICE-05 and VAL-01–06 held.
Preserve existing decisions and stricter safety constraints; do not reopen
closed work or treat historical Next fields as authority.
[Complete exact pre-MOD-08 handoff and decision history](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/f7fc983ead90964b40d99ec68c06213ab19e8215/planning/development.md).
Use [implementation map](../docs/system/implementation-map.md),
[architecture](../docs/system/architecture.md) and
[development discipline](../docs/system/development-discipline.md).

## Proof ceiling and unresolved gates

- Native presentation ABI v4 digest pair not admitted; standard dev/package/new launch held. Exact DLLs require independent review; never substitute arbitrary bytes.
- No current source test, Windows build, five-client/gameplay, UI visual, capture/recording or performance evidence.
- World/pack disposition, download cache root and seven-day log producer/root held. Existing web-cache admission unchanged; preserve accounts, worlds, private hives, snapshots and legacy roots.
- Client-native F3 `BLOCKED_ENGINE_ABI`: exact Education 26.33 image and input/player/dimension/camera/render interface required. No guessed offsets or fake toggle. [Design](../docs/system/minecraft-education-native-f3-client.md).
- Experimental input/AI/MCP holds unchanged: default OFF, exact writer/neutral/quarantine semantics, no unverified gameplay or Probe dispatch. [Prior decision](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/9b285c2110b7cdf91eda7e26c315a0b850eab82c/planning/development.md#separate-original-native-input-hold--unchanged).
- Candidate export not publication: extracted payload/legal notices, signed bytes, Authenticode chain/timestamp, clean/upgrade/uninstall and active/concurrent native session evidence held. Installer idle check is point-in-time. Repository visibility operator-managed.

## Deferred PC-local validation — not an authorization

Only a separately selected verified `LOCAL_PC_AGENT` with explicit
operator approval may execute the existing sequence. Re-pin actual Local
HEAD after the focused review; the old source-ready baseline is not current proof.
Use [Windows first-run guide](../docs/validation/local-pc-first-run.md),
[progressive smoke](../docs/validation/efficient-local-testing.md) and
the [full prior acceptance sequence](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/f7fc983ead90964b40d99ec68c06213ab19e8215/planning/development.md#source-to-local-handoff--source_ready_for_local).

1. Admission: actual Windows x64 PC/agent, clean checkout and exact SHA;
   read-only DEV.cmd localcheck. Preserve App/Data; bootstrap pinned tools
   only if missing and approved.
2. Source: approved DEV.cmd preflight once, default Cargo features;
   no redundant setup/quick/check or agent suite. Stop at first failure,
   correct canonical owner and re-pin; retain private receipts.
3. Native prerequisite: independently review exact ABI v4 DLL pair and
   admit approved digests in reviewed_bootstrap_payload.rs. Ordinary
   dev/package/new launch held until then; no old or placeholder hashes.
4. Product smoke only after matching native admission: manager then
   MC01/MC02, progressive escalation, preserve slots/data, normal close
   and reconnect. Packaging, capture/recording, AI/input separate gates.
5. RPT and UPD acceptance during approved smoke: follow the exact prior
   acceptance sequence above for disposable roots, checkpoint/receipt
   preservation, ordinary official update observation, generation pins,
   version/ABI refusal, no unreviewed task invocation or forced update.
6. Feedback: exact SHA, first failing stage, sanitized observed versus
   expected, canonical owner. No private logs/data committed, automatic
   repair/replay or repeat of unaffected successful checks.

| Gate | Evidence owner |
| --- | --- |
| VAL-01/02 | [Windows first-run](../docs/validation/local-pc-first-run.md), [native CMake controls](../Development/tooling/acceptance/windows/bootstrap-ordering/README.md) |
| VAL-03 | [Progressive smoke](../docs/validation/efficient-local-testing.md): one then five sessions, identity/layout/close/reconnect/preservation |
| VAL-04 | [Data layout](../docs/system/data-layout.md): exact producer/root, reference stability and exclusive disposition |
| VAL-05 | [Native collectors](../Development/tooling/acceptance/windows/README.md): matched workload, FPS, CPU/RAM/GPU, handles and timings; OPT-07 held |
| VAL-06 | [Distribution](../Development/engine/distribution/README.md): installer bytes, signatures/legal notices, lifecycle |

All execution deferred. No CI/cloud substitution, release or version bump.
