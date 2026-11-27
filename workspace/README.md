# Local workspace

Machine-local developer output. Only this README is tracked. The
[artifact storage policy](../docs/system/artifact-storage.md) owns paths,
retention and evidence rules; the [product data layout](../docs/system/data-layout.md)
owns App/Data, launcher locations and private profiles.

| Folder (on demand) | Purpose |
| --- | --- |
| `builds/desktop/target/` | Reusable Cargo output and intermediates |
| `cache/`, `tmp/` | Developer caches and temporary files |
| `toolchains/`, `downloads/` | Pinned portable tools and verified downloads |
| `packages/` | Immutable installer candidates |
| `runs/` | Explicit local-test receipts |
| `scratch/`, `archive/`, `private/` | Investigation, retained history, protected rollback material |

No second Data root, automatic profile migration, arbitrary cleanup,
or release. Existing local files and separate installed copies are preserved.

## Read-only developer storage inventory

`DEV.cmd storage report` and `DEV.cmd storage explain` inspect the
[bounded developer output inventory](../docs/system/artifact-storage.md#read-only-developer-disk-inventory).
`DEV.cmd storage plan` is a [retention review](../docs/system/artifact-storage.md#retention-review-and-verified-post-build-export-turnover),
not a deletion command. Logical bytes are not physical savings.

## Setup on Windows x64 (workspace-only)

See [one-root developer setup](../docs/system/development-operations.md#before-localtest-one-root-command-protection)
and [pinned toolchain](../Development/toolchain.json). Execution requires the
approved local phase; source review does not run builds or tests.

## Approved retirement of one archived compiler cache

Only the [separately approved historical target operation](../docs/system/artifact-storage.md#historical-compiler-target-retirement-separate-approval)
may retire that exact archive. No broad cleanup.

## Cargo target breakdown (read-only)

`DEV.cmd storage cargo`: [canonical and historical target attribution](../docs/system/artifact-storage.md#cargo-build-cache-attribution-read-only).

## Bounded per-pass compiler cache evidence

`DEV.cmd bench:rust`: [bounded logical-byte snapshots](../docs/system/artifact-storage.md#build-evidence-with-bounded-per-pass-cargo-snapshots).

## Scoped checks and optional timing

Use the existing [development operations](../docs/system/development-operations.md)
for checks and benchmarks. Both DEV.cmd and the supported package entrypoint
follow the [canonical output contract](../docs/system/artifact-storage.md#lazy-tester-development-candidate-and-installed-data);
arbitrary direct tool invocations are outside that guard.

## Preserved local history

See [historical storage boundaries](../docs/system/artifact-storage.md#preserved-local-history).
Never infer that old local files are disposable.
