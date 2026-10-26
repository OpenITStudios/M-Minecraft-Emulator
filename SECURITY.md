# Security and Data Protection

The product controls host virtualization, per-guest Windows services and Microsoft-account-bearing application sessions. Treat both host and guest as explicit trust boundaries.

## Canonical owners

- Guest authenticated status/launch transport and per-VM tokens: `Development/engine/runtime-core/src/guest.rs` and `Development/engine/runtime-core/src/bin/virtual-guest-agent.rs`.
- VM provider state, clone/recovery safeguards and disk paths: `Development/engine/runtime-core/`.
- Windows guest agent installation, Scheduled Task and firewall scopes: `Development/engine/guest/windows/`.
- Update trust and installer data-preservation policy: `Development/engine/distribution/` and desktop Tauri package config.
- Bounded collection of Windows acceptance evidence: `Development/tooling/acceptance/windows/`.

## Hard boundaries

- No Microsoft passwords, access/refresh tokens, session cookies, Guest Agent tokens, signer private keys, private Minecraft worlds, user VM images or raw host machine-identity values in source or repository evidence.
- No automated Microsoft sign-in/MFA, unauthenticated remote-control transport or arbitrary guest command execution endpoints.
- Each virtual client must own independent account/session state. Routine lifecycle operations preserve guest disks. Reprovision/snapshot reset require the original safeguards and explicit user authorization where destructive.
- Provider mutation results that cannot be reconciled must fail closed; do not blindly retry or silently report success.
- Update staging requires trusted artifact identity/integrity; self-apply remains disabled. Release signing/publishing is a separate explicitly approved operation.
- Installer upgrade and uninstall do not implicitly delete Base/Virtual runtime data.

Source review cannot verify effective Windows ACL, VMware guest isolation, network/firewall policy, Authenticode signing or Microsoft/Minecraft account behavior. Those require matching Windows/VMware/Minecraft acceptance evidence; use `docs/validation/README.md`.

Report a potential security issue without attaching real secrets or private guest data to public GitHub issues.
