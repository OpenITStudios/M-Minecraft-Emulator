# Security and Data Protection

Lazy Tester manages independent Minecraft Education processes in the
current Windows user environment. It is **not** a general sandbox and its
in-process compatibility DLL does not establish an OS security boundary.

## Current ownership

- Native game/process lifetime and per-profile storage: desktop
  `independent_bootstrap/` and retained `session_controller.rs`.
- Observed process/window metadata: `process_identity.rs`; PID/HWND values
  are not authorization without original live handle and creation time checks.
- Window recovery and helpers: `workspace_surface.rs`, with exact session
  and helper ownership and terminal completion requirements.
- Controller IPC: `native_clients_session/pipe.rs`, with same-binary,
  logon/session and live peer checks.
- Windows distribution: `Development/engine/distribution/lazy-tester-package.json`,
  Tauri package config and preservation-only installer hooks.

## Non-negotiable boundaries

- Never commit Microsoft passwords, tokens, session cookies, profile
  capabilities, world contents, private Windows hives, signing keys, VM
  snapshots or account-bearing test captures.
- Do not automate authentication/MFA, bypass licensing, copy tokens, patch
  original Minecraft executable bytes, or adopt another process by PID/title.
- Keep independent profile storage and normal-exit semantics. An uncertain
  exit/recovery result must remain uncertain; never force-kill Minecraft or
  erase data as a fallback.
- Sandboxie/VMware source is retired, not a supported launch or security
  provider. Removing source does not uninstall services, power off machines
  or delete existing local account/guest data.
- Installer and update source must not migrate or delete game/user data,
  touch VM disks, or change legacy installed identity without separately
  reviewed migration policy.
- User-controlled macOS capture must respect OS permissions; absence of
  screen-recording authorization is not consent for alternate capture routes.

Source review cannot establish real Windows ACL correctness, authenticated
gameplay, renderer/input stability, installed package integrity, or effective
multi-account isolation on Windows or Mac. These remain separate evidence
requirements.

Report security findings without exposing personal secrets or local profile
data in public GitHub issues.
