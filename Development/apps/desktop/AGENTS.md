# Lazy Tester Desktop Rules

Scope: `Development/apps/desktop/`. This is the mounted Tauri 2 / Svelte
front end for the native Minecraft Education five-profile controller.

## Boundaries

- `LazyTesterApp.svelte` projects typed native state; the UI does not own
  process or authentication lifetime. Mounted Tauri IPC is allowlisted in
  `src-tauri/src/app_bootstrap.rs`.
- `engine/independent_bootstrap/session_controller.rs` owns original game
  process handles, profile storage and normal exit. Do not add another actor,
  daemon, power-management provider, process registry or credentials store.
- `engine/process_identity.rs` supplies serialization-only identity values.
  Exact OS handle/birth/window checks remain in native owners.
- `engine/workspace_surface.rs` owns presentation, DWM preview, parking,
  native recovery helpers and restoration. UI never claims a successful
  restore from a posted command, PID or HWND alone.
- Immutable private profile locations/data, settings, worlds, authentication
  and Windows hives are never deleted/recreated by packaging or repository
  cleanup. Profile clear still requires its own explicit admission/confirmation.
- `engine/native_clients_session.rs` owns checked reconnect and the retained
  controller IPC. A timed-out operation cannot start a replacement worker.
- Deprecated VMware/Sandboxie/owned-fork Tauri handlers, provider helpers
  and box/Guest Agent tooling are not active fallback routes and must not
  be reintroduced by tests or previews.

## Verification boundary

Source-level review and authored fixtures are **not** executable verification.
Never claim Windows, Minecraft gameplay, five account persistence or Mac
compatibility from a GitHub-only commit. Native DLLs in
`tooling/acceptance/windows/bootstrap-ordering/` are active source and
must not be deleted as historical fixture code. Keep CI disabled and use
`Local` exclusively for development. Do not initiate user-PC testing until
explicitly authorized.
