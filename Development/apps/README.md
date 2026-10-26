# Applications

The only user-facing desktop application is [`desktop/`](desktop). Its Svelte presentation and Tauri adapters consume the canonical Rust backend in `../engine/runtime-core/`; they do not own provider/VM lifecycle state.

See [`desktop/README.md`](desktop/README.md) for local UI/build commands. Do not create a second app shell for the same VM state.
