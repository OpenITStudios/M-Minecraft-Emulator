# Virtual Clients Desktop Rules

Applies to `Development/apps/desktop/`.

## Boundary

This is the canonical Tauri 2 + Svelte desktop application for the Virtual Clients Rust backend under `Development/engine/runtime-core/`.

## Rules

- Layering is `Svelte surfaces → typed Tauri facade → thin Tauri commands → Rust domain owner`.
- Consume `PUBLIC_CONTRACT_SCHEMA = 1`; never invent a second lifecycle, identity, resource, update, or recovery policy.
- Lifecycle controls consume backend `actions` availability. Presentation code must not infer permission from client state.
- Setup guidance consumes `DoctorReport.nextSetupAction`; health issues are diagnostic facts only.
- `EnginePolicy` is read-only product policy. Do not create desktop-owned configuration for invariant values.
- Desktop-native behavior such as window arrangement belongs under `src-tauri/src/engine/`; VM lifecycle semantics do not.
- Tauri commands are adapters only. They do not duplicate backend business rules.
- No account credentials, Microsoft tokens, Guest Agent tokens, world content, or independent persistent state.
- No fake runtime state in production code. Tauri/backend failures must surface explicitly.
- Presentation-only preferences may use one local owner when persistence materially improves daily use; Window Layout uses `src/app/windowLayoutPreference.ts`. Do not create a general settings framework for presentation state.
- Every backend response must pass the versioned public contract boundary before entering presentation state.
- Do not reintroduce localhost HTTP transport between the desktop frontend and backend, browser-global bridge injection, Inno Setup, or another desktop host. The guest-internal Interactive Launcher is a separate fixed-action Windows boundary and may bind localhost only inside the guest.

- User-facing feature name is **Window Layout**. `window_arrangement` is only an internal Windows adapter/command implementation term.
- Window Layout geometry has one owner: `src-tauri/src/engine/window_arrangement.rs`; Screen Overlay consumes its slots and never calculates a second layout.
- Public Window Layout mutation has one path: `window_apply_layout`. Do not restore legacy arrange/overlay mutation commands.
- Guest Minecraft launch policy belongs to `Development/engine/runtime-core/src/minecraft_runtime.rs`; desktop code does not infer Guest Agent capability.
- Do not add Start Session, account-state detection, auth automation, forced network mode, GPU thresholds, reconnect machinery, or another launcher owner without target-machine evidence recorded in the canonical acceptance/failure matrix.
- Do not retain unused public commands or speculative engines for a future consumer; add them when a proven workflow needs them.
