param(
    [Parameter(Position=0)]
    [ValidateSet("setup", "doctor", "lock", "g0", "preflight", "check", "test", "dev", "package", "storage", "help")]
    [string]$Command = "help",
    [Parameter(Position=1)]
    [ValidateSet("report", "explain", "plan")]
    [string]$StorageMode = "report"
)

$ErrorActionPreference = "Stop"
$DevelopmentRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$Root = Split-Path -Parent $DevelopmentRoot

# The inventory route must not initialize environment directories or Cargo
# caches. Use the pinned portable Node directly; no global fallback or writes.
if ($Command -eq "storage") {
    $localNode = Join-Path $Root 'workspace/toolchains/node/node.exe'
    if (-not (Test-Path -LiteralPath $localNode -PathType Leaf) -or
        ((Get-Item -LiteralPath $localNode -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "Storage inventory requires the ordinary workspace-local Node executable."
    }
    $manifest = Get-Content -LiteralPath (Join-Path $DevelopmentRoot 'toolchain.json') -Raw | ConvertFrom-Json
    $actualVersion = (& $localNode --version).Trim().TrimStart('v')
    if ($LASTEXITCODE -ne 0 -or $actualVersion -ne [string]$manifest.node.version) {
        throw "Storage inventory Node version does not match the pinned local toolchain."
    }
    if ($StorageMode -eq "plan") {
        & $localNode (Join-Path $DevelopmentRoot 'tooling/storage/retention.mjs')
    } else {
        & $localNode (Join-Path $DevelopmentRoot 'tooling/storage/inventory.mjs') $StorageMode
    }
    if ($LASTEXITCODE -ne 0) {
        throw "Storage inventory was partial or failed (exit $LASTEXITCODE); review the read-only report above."
    }
    return
}
if ($PSBoundParameters.ContainsKey('StorageMode')) {
    throw "Extra arguments are supported only for DEV.cmd storage."
}
. (Join-Path $PSScriptRoot 'local-env.ps1')
Set-LocalDeveloperEnvironment -Root $Root
$LocalNpm = Join-Path $Root 'workspace/toolchains/node/npm.cmd'
$DesktopLock = "apps/desktop/src-tauri/Cargo.lock"
$DesktopManifest = "apps/desktop/src-tauri/Cargo.toml"

function Invoke-RepositoryNpm {
    param([string[]]$NpmArguments)
    & $LocalNpm @NpmArguments
    if ($LASTEXITCODE -ne 0) {
        throw "npm $($NpmArguments -join ' ') failed with exit code $LASTEXITCODE."
    }
}

function Assert-LocalCheckout {
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        throw "Git is required to bind developer results to the committed Local source."
    }
    $branchName = (& git branch --show-current).Trim()
    if ($LASTEXITCODE -ne 0 -or $branchName -ne "Local") {
        throw "Developer verification must run on branch Local; observed '$branchName'. No branch was changed."
    }
    $commit = (& git rev-parse --verify HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $commit -notmatch '^[0-9a-f]{40}$') {
        throw "Git HEAD cannot be verified."
    }
    return $commit
}

function Assert-CleanSourceAtHead {
    param([string]$ExpectedSha)
    $headNow = (& git rev-parse --verify HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $headNow -ne $ExpectedSha) {
        throw "HEAD changed during the local source/package run."
    }
    $changes = @(& git status --porcelain --untracked-files=normal)
    if ($LASTEXITCODE -ne 0) {
        throw "Git status could not be evaluated."
    }
    if ($changes.Count -gt 0 -and ($changes -join "").Trim().Length -gt 0) {
        throw "Source verification needs an exact clean committed Local revision; commit or restore source changes first. VM files were not touched."
    }
}

function Assert-NativePilotHost {
    if (-not $IsWindows) {
        throw "Native Minecraft local test requires Windows 10 or 11 x64."
    }
    if ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne
        [System.Runtime.InteropServices.Architecture]::X64) {
        throw "Native Minecraft local test requires x64 Windows, not ARM64/x86."
    }
    if ([Environment]::OSVersion.Version.Major -lt 10) {
        throw "Native Minecraft local test requires Windows 10 or 11."
    }
}

function Test-DeveloperToolchain {
    param([switch]$AllowMissingDesktopLock, [switch]$RustOnly)
    $toolchain = Get-Content (Join-Path $DevelopmentRoot "toolchain.json") -Raw | ConvertFrom-Json
    $errors = [System.Collections.Generic.List[string]]::new()

    if ($PSVersionTable.PSVersion.Major -lt [int]$toolchain.powershell.minimumMajor) {
        $errors.Add("PowerShell 7+ is required for DEV.cmd.")
    }
    $requiredTools = @("rustc", "cargo")
    if (-not $RustOnly) {
        $requiredTools += @("node", "npm.cmd", "rustfmt")
    }
    $localBins = @{
        "node" = (Join-Path $Root 'workspace/toolchains/node/node.exe')
        "npm.cmd" = $LocalNpm
        "rustc" = (Join-Path $Root 'workspace/toolchains/cargo/bin/rustc.exe')
        "cargo" = (Join-Path $Root 'workspace/toolchains/cargo/bin/cargo.exe')
        "rustfmt" = (Join-Path $Root 'workspace/toolchains/cargo/bin/rustfmt.exe')
    }
    foreach ($tool in $requiredTools) {
        $found = Get-Command $tool -ErrorAction SilentlyContinue
        if (-not $found) {
            $errors.Add("$tool is unavailable in the workspace-local toolchain.")
        } elseif ([IO.Path]::GetFullPath($found.Source) -ine
                  [IO.Path]::GetFullPath($localBins[$tool])) {
            $errors.Add("$tool resolves outside workspace/toolchains: $($found.Source).")
        }
    }
    if (-not $RustOnly -and (Get-Command node -ErrorAction SilentlyContinue)) {
        $actual = (& node --version).Trim().TrimStart("v")
        if ($actual -ne [string]$toolchain.node.version) {
            $errors.Add("Node v$actual does not match pinned v$($toolchain.node.version).")
        }
    }
    if (-not $RustOnly -and (Get-Command npm.cmd -ErrorAction SilentlyContinue)) {
        $actual = (& $LocalNpm --version).Trim()
        if ($actual -ne [string]$toolchain.npm.version) {
            $errors.Add("npm $actual does not match pinned $($toolchain.npm.version).")
        }
    }
    if (Get-Command rustc -ErrorAction SilentlyContinue) {
        $rustcVersion = & rustc --version
        if ($LASTEXITCODE -ne 0 -or $rustcVersion -notmatch '^rustc (\d+\.\d+\.\d+)') {
            $errors.Add("Rust compiler version could not be identified.")
        } elseif ($Matches[1] -ne [string]$toolchain.rust.version) {
            $errors.Add("Rust $($Matches[1]) does not match pinned $($toolchain.rust.version).")
        }
    }
    if (Get-Command rustc -ErrorAction SilentlyContinue) {
        $hostReport = (& rustc -vV) -join "`n"
        if ($LASTEXITCODE -ne 0) {
            $errors.Add("Rust host target cannot be inspected; the unsigned NSIS build requires x86_64-pc-windows-msvc.")
        } elseif ($hostReport -match '(?m)^host:\s*([^\s]+)') {
            $hostTriple = $Matches[1]
            if ($hostTriple -ne "x86_64-pc-windows-msvc") {
                $errors.Add("Rust host target $hostTriple is unsupported; use the pinned x86_64-pc-windows-msvc toolchain.")
            }
        } else {
            $errors.Add("Rust host target cannot be identified; the unsigned NSIS build requires x86_64-pc-windows-msvc.")
        }
    }
    if (-not $RustOnly -and -not (Test-Path "apps/desktop/package-lock.json")) {
        $errors.Add("Canonical desktop npm lockfile is missing.")
    }
    if (-not $RustOnly -and -not (Test-Path "engine/runtime-core/Cargo.lock")) {
        $errors.Add("Canonical backend Cargo.lock is missing.")
    }
    if (-not $RustOnly -and -not $AllowMissingDesktopLock -and -not (Test-Path $DesktopLock)) {
        $errors.Add("Desktop Cargo.lock is missing. DEV.cmd lock can generate it using real Cargo.")
    }

    if ($errors.Count -gt 0) {
        foreach ($errorMessage in $errors) { Write-Host "FAIL $errorMessage" }
        throw "Developer toolchain doctor found $($errors.Count) issue(s)."
    }
    Write-Host "Toolchain inputs present. No G0 or VM/game acceptance implied."
    Write-Host "Project root: $Root"
    Write-Host "Developer tools: $(Join-Path $Root 'workspace/toolchains')"
    Write-Host "Project downloads: $(Join-Path $Root 'workspace/downloads')"
    Write-Host "Project cache: $(Join-Path $Root 'workspace/cache')"
    Write-Host "Project temporary files: $(Join-Path $Root 'workspace/tmp')"
}

# Create ONLY the missing desktop lockfile. Never synthesize dependency versions,
# modify the user's VM storage, commit automatically or promote Local to main.
function New-DesktopCargoLock {
    $sha = Assert-LocalCheckout
    Assert-CleanSourceAtHead -ExpectedSha $sha
    # Rust dependency resolution does not require Node/npm or frontend packages.
    Test-DeveloperToolchain -AllowMissingDesktopLock -RustOnly
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw "Cargo is required for genuine desktop dependency resolution."
    }
    if (Test-Path $DesktopLock) {
        throw "$DesktopLock already exists; it was not overwritten. Review and commit the existing file if it is valid."
    }
    & cargo generate-lockfile --manifest-path $DesktopManifest
    if ($LASTEXITCODE -ne 0 -or -not (Test-Path $DesktopLock)) {
        throw "Cargo dependency resolution failed. No lockfile completeness is claimed."
    }
    $after = (& git rev-parse --verify HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $after -ne $sha) {
        throw "Git HEAD changed during Cargo resolution. The generated lockfile is unverified; do not commit it blindly."
    }
    Write-Host "Cargo generated $DesktopLock from the actual registry graph for HEAD $sha."
    Write-Host "Review it, then commit and push it to Local. G0 cannot pass on an uncommitted lockfile."
    Write-Host "Example: git add -- $DesktopLock ; git commit -m 'build(desktop): lock resolved Cargo dependencies'"
}

# Inspect the just-built output without running NSIS or accessing persisted
# Virtual Clients data. This is a G0 construction check, not installed proof.
function Assert-BuiltWindowsPackage {
    param([string]$ExpectedSha, [datetime]$BuildStartedUtc)

    $bundleDir = Join-Path $env:CARGO_TARGET_DIR "x86_64-pc-windows-msvc/release/bundle/nsis"
    if (-not (Test-Path -LiteralPath $bundleDir -PathType Container)) {
        throw "Tauri reported success, but the NSIS bundle directory does not exist: $bundleDir"
    }
    $version = (Get-Content (Join-Path $DevelopmentRoot "apps/desktop/package.json") -Raw | ConvertFrom-Json).version
    $installers = @(Get-ChildItem -LiteralPath $bundleDir -Filter "*.exe" -File |
        Where-Object { $_.Name.Contains([string]$version) })
    if ($installers.Count -ne 1) {
        throw "Expected one NSIS installer for version $version, found $($installers.Count) in $bundleDir."
    }
    $installer = $installers[0]
    if ($installer.Length -le 0 -or $installer.LastWriteTimeUtc -lt $BuildStartedUtc.AddSeconds(-5)) {
        throw "NSIS output is empty or predates this G0 package command; do not trust a stale installer."
    }

    $sidecar = Join-Path $DevelopmentRoot "apps/desktop/src-tauri/binaries/virtual-clients-x86_64-pc-windows-msvc.exe"
    $agent = Join-Path $DevelopmentRoot "apps/desktop/src-tauri/package-resources/guest/windows/virtual-guest-agent.exe"
    foreach ($required in @($sidecar, $agent)) {
        if (-not (Test-Path -LiteralPath $required -PathType Leaf) -or
            (Get-Item -LiteralPath $required).Length -le 0) {
            throw "Required staged package binary is absent or empty: $required"
        }
    }

    $hash = (Get-FileHash -LiteralPath $installer.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    Write-Host "G0 package source SHA: $ExpectedSha"
    Write-Host "G0 unsigned NSIS output: $($installer.FullName)"
    Write-Host "G0 installer SHA-256: $hash"
    Write-Host "Installed payload, repair/uninstall preservation, signatures, VMware and Minecraft remain UNVERIFIED."
}


# Active native + Sandboxie pilot: one read-only source checkpoint, not the
# intentionally blocked legacy VMware G0 packaging route. Installs only locked
# developer dependencies and produces ordinary ignored build artifacts.
function Invoke-NativeSourcePreflight {
    Write-Host "[1/4] Checking Windows x64 and clean committed Local HEAD..."
    Assert-NativePilotHost
    $sha = Assert-LocalCheckout
    Assert-CleanSourceAtHead -ExpectedSha $sha

    Write-Host "[2/4] Checking exact toolchain versions and tracked Cargo lockfile..."
    Test-DeveloperToolchain
    & git ls-files --error-unmatch -- $DesktopLock *> $null
    if ($LASTEXITCODE -ne 0) {
        throw "Native source preflight requires a tracked desktop Cargo.lock at the committed Local HEAD."
    }
    Write-Host "Native source preflight SHA: $sha"
    # Begin metadata-only recording after pinned Node and clean commit checks.
    # Never capture raw console output, credentials or Minecraft game data.
    $recordDir = (& node "tooling/local-test/session.mjs" start preflight $sha)
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($recordDir)) {
        throw "Cannot create a safe local-test record. Source checks were not started."
    }
    Write-Host "Local-test record: $(Join-Path $recordDir 'manifest.json')"
    $stage = "setup"
    try {
        Write-Host "[3/4] Installing locked developer npm dependencies..."
        Invoke-RepositoryNpm -NpmArguments @("run", "setup")
        $stage = "verify-source"
        Write-Host "[4/4] Executing all source checks, without packaging or launching games..."
        Invoke-RepositoryNpm -NpmArguments @("run", "verify:source")
        $stage = "source-integrity"
        Assert-CleanSourceAtHead -ExpectedSha $sha
        & node "tooling/local-test/session.mjs" finish $recordDir PASS complete
        if ($LASTEXITCODE -ne 0) {
            throw "Source checks finished but the local-test manifest could not be finalized."
        }
    }
    catch {
        # Preserve the original source error even if recording fails separately.
        try {
            & node "tooling/local-test/session.mjs" finish $recordDir FAIL $stage
            if ($LASTEXITCODE -ne 0) {
                Write-Warning "Local-test failure record could not be finalized."
            }
        } catch {
            Write-Warning "Local-test recorder failed; preserve the original console error."
        }
        Write-Host "Local-test record: $(Join-Path $recordDir 'manifest.json')"
        throw
    }

    Write-Host "SOURCE PREFLIGHT COMPLETE for $sha (not Minecraft acceptance)."
    Write-Host "NEXT: .\DEV.cmd dev  (manual UI test; no legacy installer)"
    Write-Host "No installer, VM, Sandboxie process, licensed sign-in or Minecraft session was tested."
}

# One opt-in source/package checkpoint. This is NOT a game/VM test and does not
# install/uninstall the product or access account-bearing guest disks.
function Invoke-G0SourcePackage {
    $sha = Assert-LocalCheckout
    Test-DeveloperToolchain
    & git ls-files --error-unmatch -- $DesktopLock *> $null
    if ($LASTEXITCODE -ne 0) {
        throw "Desktop Cargo.lock must be tracked in committed Local before G0."
    }
    Assert-CleanSourceAtHead -ExpectedSha $sha

    Write-Host "G0 source/package revision: $sha"
    Invoke-RepositoryNpm -NpmArguments @("run", "setup")
    Invoke-RepositoryNpm -NpmArguments @("run", "verify:source")
    $packageStartedUtc = (Get-Date).ToUniversalTime()
    Invoke-RepositoryNpm -NpmArguments @("run", "package:windows")
    Assert-BuiltWindowsPackage -ExpectedSha $sha -BuildStartedUtc $packageStartedUtc

    # Builds may write ignored artifacts. Verify tracked sources did not change.
    $trackedChanges = @(& git diff --name-only HEAD --)
    if ($LASTEXITCODE -ne 0 -or ($trackedChanges -join "").Trim().Length -gt 0) {
        throw "Tracked source changed during G0; resulting evidence is not revision-stable."
    }
    $after = (& git rev-parse --verify HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $after -ne $sha) {
        throw "Git revision changed during G0; do not combine evidence from different commits."
    }
    Write-Host "Source checks and unsigned Windows package command completed for $sha."
    Write-Host "G0 NOT YET PASSED: installed payload and disposable install/repair/uninstall safety require independent review."
    Write-Host "Do not use account-bearing VM disks for the first guest feasibility test."
}

Push-Location $DevelopmentRoot
try {
    # Any command that may install dependencies, build or launch the desktop
    # must prove pinned local binaries first; "doctor" is not an optional guard.
    if ($Command -in @("setup", "check", "test", "dev")) {
        Test-DeveloperToolchain
    }
    switch ($Command) {
        "setup"   { Invoke-RepositoryNpm -NpmArguments @("run", "setup") }
        "doctor"  { Test-DeveloperToolchain }
        "lock"    { New-DesktopCargoLock }
        "g0"      { Invoke-G0SourcePackage }
        "preflight" { Invoke-NativeSourcePreflight }
        "check"   { Invoke-RepositoryNpm -NpmArguments @("run", "verify:source") }
        "test"    {
            Invoke-RepositoryNpm -NpmArguments @("run", "test")
            Invoke-RepositoryNpm -NpmArguments @("run", "test:runtime")
        }
        "dev"     { Invoke-RepositoryNpm -NpmArguments @("run", "dev") }
        "package" { Invoke-RepositoryNpm -NpmArguments @("run", "package:windows") }
        default {
            Write-Host "M-Minecraft-Emulator (workspace-local Windows developer commands)"
            Write-Host "  DEV.cmd bootstrap # verified, portable Node / PowerShell / Rust under workspace"
            Write-Host "  DEV.cmd lock     # resolve missing Tauri Cargo.lock (no automatic commit)"
            Write-Host "  DEV.cmd doctor   # pinned toolchain checks"
            Write-Host "  DEV.cmd preflight # native pilot exact-SHA setup + full source verification (NO package/game)"
            Write-Host "  DEV.cmd g0       # LEGACY VMware packaging, BLOCKED for native pilot"
            Write-Host "  DEV.cmd setup    # install pinned npm dependencies"
            Write-Host "  DEV.cmd check    # full source verification"
            Write-Host "  DEV.cmd test     # desktop and runtime tests"
            Write-Host "  DEV.cmd dev      # Tauri desktop development"
            Write-Host "  DEV.cmd package  # build and stage a Lazy Tester development candidate"
            Write-Host "  DEV.cmd storage report  # read-only developer file-size inventory"
            Write-Host "  DEV.cmd storage explain # top measured categories; no deletion"
            Write-Host "  DEV.cmd storage plan    # dry-run retention review; no deletion"
        }
    }
}
finally {
    Pop-Location
}
