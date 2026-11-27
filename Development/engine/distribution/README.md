# Lazy Tester distribution contract

The only current product package is the separate **Lazy Tester** Windows
per-user installer. Its machine-readable contract is
[lazy-tester-package.json](lazy-tester-package.json).

Current schema 2 locations in the package contract are
`Data/Launchers/Minecraft-1..5`, `Data/profiles.json`, and the canonical
pointer `Data/Settings/data-location.json`. The explicit legacy fields
describe `Data/Profiles/Minecraft-1..5` for schema 1 and
`App/data-location.json` for old flat installations. The native owner checks
the old pointer only if the canonical pointer is absent; installer/refresh
preserve the old file. No field authorizes automatic data migration.

## Install, upgrade and uninstall

Fresh installations keep the standard per-user App/Data layout. An existing
registered location is authoritative even if the old executable has already
been removed by the upstream reinstall page; a changed destination is refused
before the custom hook permits payload replacement. No App suffix is added
to a registered legacy flat install. Existing unregistered flat executable
or locator markers still preserve their location. This is not a migration
mechanism.

The existing controller-idle check, exact same-image role-link handling and
uninstall data-checkbox override remain. The pinned Tauri NSIS template can
invoke Restart Manager to close the desktop process after the custom idle
hook; the hook is a point-in-time check, not a whole-install lock. Native
installation/upgrade/uninstall and concurrent startup scenarios must be
verified before distribution. Do not run installation against live game
sessions or production data to demonstrate the source contract.

## Binary-only distribution scope

The approved design retains the existing Rust/Tauri/NSIS build, not a new
installer engine. Ship compiled application, reviewed native payload, required
runtime metadata/resources and required legal notices only. Original source,
development tools, repository metadata, private symbols, source maps and user
data are not intended payload. Source-only configuration checks do not certify
the extracted installer. Publication remains disabled; code signing and final
payload verification require separate native evidence and approval. No runtime
packer, decryption service or activation/DRM is introduced. Repository visibility
is the operator's responsibility.

The supported frontend build runs the existing package checker in
`--frontend` mode immediately after Vite and before Tauri embeds dist.
Vite does not copy a public directory. The generated layout is limited to
index.html, hashed JavaScript/CSS and the optional hashed application icon;
unexpected files, links, nested asset directories, source-map directives and
missing JS/CSS refuse the build. This is a bounded build-output check, not
proof that permitted compiled assets contain no sensitive content. Intentional
new asset classes and required legal notices must be reviewed explicitly.
Tauri copies bundle resources during native compilation, before
beforeBundleCommand generates MUI files. The locale directory mapping is
therefore retained; the same exact three-file validator runs before native
compilation (missing/partial allowed) and immediately before bundling
(complete required). Extra entries, links, directories and invalid files
are refused. The original role-generation and DLL digest gates are preserved.
No placeholder MUI or second compilation is necessary.

Production Vite explicitly disables source maps and uses the existing Oxc
minifier. The desktop Cargo release profile disables debug information and
uses debuginfo stripping; full symbol stripping is not required. Native
C++/Rust DLL build flags and developer/debug profiles are unchanged.
PDBs, MAP files and other private symbols are not admitted to distribution.
This is not an anti-reverse-engineering guarantee or a measured size gain. Direct Cargo or altered build tooling
is not a certified release route; final extracted-payload verification remains held.

## Integrity and signing boundary

The existing package command requires the native digest review before building
and rechecks both staged DLL inputs and local release DLL copies after
bundling, before the candidate receipt or App refresh. A changed byte,
including a signature added after digest approval, blocks export. The
installer must be an ordinary fresh build output (mtime not older than the
attempt); this timestamp check is not proof of its internal payload.
The existing final SHA-256 and immutable three-file receipt remain the
candidate identity. Export-only skips the unrelated checkout root-launcher build.

For a separately approved signed distribution, use the existing Tauri
Windows signing mechanism and Windows SDK SignTool, not a new signing service.
Sign native DLLs first, verify the final signatures and independently review
the exact final DLL digests in the native owner before admitting them. After
role-resource generation and binary modifications, sign the application and
installer using the authorized publisher certificate and timestamping. Verify
actual extracted payload hashes, Authenticode chain/publisher/timestamp,
and install/upgrade/uninstall evidence for that exact final installer.
Do not sign reviewed DLLs in place, accept newly computed digests automatically,
or rewrite a previous candidate receipt after signing. Keep signing keys
outside source and packages. Signing authenticates publisher/integrity;
it does not prevent copying or reverse engineering.

The source policy explicitly refuses publication-enabled candidates or removal
of the Authenticode-before-publication requirement. No certificate, signing
command, key storage or release authorization is configured by this work.
Actual signing and release validation remain deferred; the candidate manifest
continues to report signing NOT_VERIFIED and publication DISABLED.

The native bundler consumes
`Development/apps/desktop/src-tauri/tauri.lazy-tester.conf.json`, the
`lazy-tester-package` Cargo feature and staged, reviewed native DLLs. The
source packaging wrapper exports candidate installer/checksum/manifest receipts
without publishing, installing the exported installer, opening games or
removing account/world data. Normal checkout-local staging retains the existing
desktop startup check; export-only does not run the app.

The shared Tauri base configuration carries **no legacy VM/Guest Agent assets or
installer hooks**. Lazy Tester and the optional historical native preview overlay
explicitly select their own identity and payload policy; the retired legacy
Windows distribution entrypoint always refuses package construction. Direct
release builds without an approved native feature remain blocked in `build.rs`.

The VMX/VMware, Guest Agent, driver/service, legacy update manifest and old NSIS
installer definitions were retired from `Local`, but archived Git revisions
remain available for historical compatibility review. Existing installed
products and saved files remain untouched. No signed/installed Windows package
or gameplay acceptance was executed in this source-only refactor.

See [data layout](../../../docs/system/data-layout.md),
[artifact storage](../../../docs/system/artifact-storage.md), and
[retirement register](../../../docs/system/legacy-retirement.md).
