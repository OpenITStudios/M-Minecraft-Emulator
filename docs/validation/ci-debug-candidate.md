---
id: document.validation.ci-debug-candidate
class: DOCUMENT
domain: validation
role: PROCEDURE
authority: WORKING
lifecycle: ACTIVE
---

# Localtest debug executable — Windows x64

**Source of evidence:** GitHub Actions `Emulator Source Verification (manual)`, branch `Local`, automatic `push` run. The downloadable `localtest-debug-<40-character-SHA>` artifact is created **only if** all twelve Full Source stages and the Windows compile-only step actually succeed.

This is a **raw, unsigned development executable**, not a signed installer, Windows release, Sandboxie distribution, driver package or validated portable product. Its existence proves only that a particular debug binary was compiled and archived on a GitHub-hosted Windows runner. It does **not** prove executable startup, screen rendering on your PC, Microsoft login, Minecraft Education, game performance, multiplayer or account isolation.

## Obtain and verify

1. In [GitHub Actions](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/workflows/verify.yml), open a successful automatic run with event `push`, branch `Local`, and inspect its **exact full commit SHA**.
2. Confirm that **Enforce all full-source results**, **Auto compile Windows desktop (no installer)**, **Stage developer executable**, and **Upload developer binary evidence** all succeeded. A manually dispatched `quick`, `full-source` or `windows-build` run does **not** create this artifact.
3. Download its `localtest-debug-<SHA>` GitHub Actions artifact while it is retained (30 days for new runs; earlier artifacts retain their own original expiry). Extract into a new ordinary folder, not an existing VMware application, Minecraft installation or Sandboxie box. It contains only `m-bedrock-virtual-clients.exe`, `BUILD-MANIFEST.txt`, and `LOCALTEST-INSTRUCTIONS.md`.
4. Confirm the manifest declares `pe_machine=AMD64`, `pe_optional_header=PE32_PLUS` and `pe_subsystem=WINDOWS_GUI`. These are structural checks performed by GitHub CI, **not** proof of a trusted signer, successful GUI startup or software safety. Compare the complete `source_commit` in the manifest with the run's `head_sha`. Run this in PowerShell from the extracted folder:

```powershell
$expected = ((Get-Content .\BUILD-MANIFEST.txt | Where-Object { $_ -like 'sha256=*' }) -split '=', 2)[1]
$actual = (Get-FileHash -LiteralPath .\m-bedrock-virtual-clients.exe -Algorithm SHA256).Hash
if ($actual.ToLowerInvariant() -ne $expected.ToLowerInvariant()) { throw 'Debug executable checksum mismatch: stop' }
```

Never use a build with a missing/mismatched manifest, wrong commit, or a failed CI verdict.

**Artifact availability:** the original #15 candidate expires 26 October 2026. The longer 30-day retention applies only to artifacts uploaded by the newer workflow; GitHub retention limits may further restrict availability. Do not mistake a metadata-only artifact listing for a verified local launch.

## First smoke only

Use **Windows 10/11 x64**, a user-controlled test PC, and the officially provided Microsoft Edge **WebView2 Runtime** if required by Tauri. Running the extracted EXE does not install Sandboxie Plus or Minecraft Education and does not install a driver. If Windows warns about an unsigned or unfamiliar executable, stop and verify its provenance before making any OS security exception; this is not a recommendation to bypass SmartScreen.

First verify only that the manager launches, renders the expected UI, shows Client 03/04 unavailable, and can close without changing any other application. Do **not** treat successful startup as game/account acceptance. The binary has not been through MSI/NSIS packaging, installed-resource tests or uninstall/repair tests. Do not copy it into a protected installed application folder or share it as an official release.

For actual **two-client** validation, follow [Native + Sandboxie localtest](./native-pilot-localtest.md), including prerequisite/box settings, authorized distinct accounts, manual process checks, gameplay, persistence and crash observations. If the raw debug executable cannot run independently, record the exact SHA and general error; use the repository's existing `DEV.cmd preflight` → `DEV.cmd dev` developer route when that environment becomes available. Neither path may silently modify existing account-bearing data.

**No sensitive evidence:** never upload Microsoft credentials, Minecraft game binaries, world files, host fingerprints, sandbox profile data, raw process logs or identifiable screenshots to GitHub Actions artifacts. Only redacted test outcomes belong in later reviewed documentation.
