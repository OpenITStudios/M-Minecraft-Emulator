# Native dependency upgrade gate

The Windows client uses two independently locked Cargo manifests:
`apps/desktop/src-tauri/Cargo.toml` and `engine/runtime-core/Cargo.toml`.

## Preconditions

- Rust toolchain satisfying the workspace MSRV (the desktop declares Rust 1.98).
- A Windows build host and installed Visual Studio C++/Windows SDK prerequisites.
- Online crates.io access for Cargo resolution.
- A clean worktree; disabled CI remains unchanged.

## Scoped upgrade workflow

1. Inspect available releases with `cargo outdated` (if installed) and `cargo audit` (if installed).
2. Update the desktop `tauri` and `tauri-build` **together** to a mutually compatible release, not by transplanting another application's Cargo.lock.
3. Run `cargo update --manifest-path apps/desktop/src-tauri/Cargo.toml` to regenerate the desktop lockfile.
4. Evaluate `ureq` 2 -> 3 separately; its native TLS and JSON APIs require source adaptation. Do not bump it in isolation.
5. Update `engine/runtime-core/Cargo.lock` via `cargo update --manifest-path engine/runtime-core/Cargo.toml` after reviewing HTTP and TLS migrations.
6. Validate:

```sh
npm --prefix apps/desktop run check:rust
npm --prefix apps/desktop run test:rust
cargo check --locked --all-targets --manifest-path engine/runtime-core/Cargo.toml
cargo test --locked --all-targets --manifest-path engine/runtime-core/Cargo.toml
```

7. On Windows, validate foreground observation, process lifecycle, per-client inventory, and packaging using the focused acceptance scripts. No Windows/driver changes should be accepted based only on Linux checks.

No Cargo command was run in the GitHub connector environment. Native dependency versions remain unchanged until compatible Cargo resolution and Windows validation are possible.
