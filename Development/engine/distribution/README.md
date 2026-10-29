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

The native bundler consumes
`Development/apps/desktop/src-tauri/tauri.lazy-tester.conf.json`, the
`lazy-tester-package` Cargo feature and staged, reviewed native DLLs. The
source packaging wrapper exports candidate installer/checksum/manifest receipts
without publishing, installing, opening games or removing account/world data.

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
