# Read-only local prerequisite inspection; no bootstrap, build, network, game or write.
$ErrorActionPreference = 'Stop'
$Root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../..'))
$script:Blocked = 0
$gitReady = $false
function Report {
    param([string]$Status, [string]$Detail)
    Write-Output "[$Status] $Detail"
    if ($Status -eq 'BLOCKED') { $script:Blocked++ }
}
function Get-ExpectedDigest {
    param([string]$Body, [string]$Field)
    $part = [regex]::Match($Body, ('(?s)\b' + [regex]::Escape($Field) + '\s*:\s*\[([^\]]+)\]'))
    if (-not $part.Success) { throw "Missing native digest: $Field" }
    $contents = $part.Groups[1].Value
    $octets = [regex]::Matches($contents, '0x([0-9a-fA-F]{2})')
    if ($octets.Count -ne 32 -or
        [regex]::Replace($contents, '0x[0-9a-fA-F]{2}|[,\s]', '') -ne '') {
        throw "Invalid native digest: $Field"
    }
    return (@($octets | ForEach-Object { $_.Groups[1].Value.ToLowerInvariant() }) -join '')
}
# The local packager rejects reparse paths for reviewed native DLLs.
function Test-OrdinaryDirectoryChain {
    param([string]$Path)
    $current = [IO.Path]::GetFullPath($Path)
    while ($true) {
        $item = Get-Item -LiteralPath $current -Force -ErrorAction Stop
        if (-not $item.PSIsContainer -or
            ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
            return $false
        }
        $parent = [IO.Directory]::GetParent($current)
        if ($null -eq $parent) { return $true }
        $current = $parent.FullName
    }
}
Write-Output 'Lazy Tester / Local PC prerequisites (READ ONLY)'
Write-Output 'Source and file checks only: not an app, compiler, installer, or Minecraft test.'
if ($env:OS -ceq 'Windows_NT' -and [Environment]::Is64BitOperatingSystem -and
    [Environment]::OSVersion.Version.Major -ge 10) {
    Report 'FOUND' 'Windows 10/11 64-bit host; exact x64 architecture not established by this read-only PowerShell 5.1 check.'
    Report 'CHECK' 'DEV.cmd preflight/dev/package must verify native x64 OS architecture with PowerShell 7.'
} else {
    Report 'BLOCKED' 'Windows 10/11 64-bit OS required; native x64 verification follows in PowerShell 7.'
}
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Report 'BLOCKED' 'Git for Windows not available.'
} else {
    $branch = (& git -C $Root branch --show-current 2>$null | Out-String).Trim()
    $branchExit = $LASTEXITCODE
    $head = (& git -C $Root rev-parse HEAD 2>$null | Out-String).Trim()
    $headExit = $LASTEXITCODE
    $dirty = @(& git -C $Root status --porcelain --untracked-files=normal 2>$null)
    $dirtyExit = $LASTEXITCODE
    if ($branchExit -ne 0 -or $branch -cne 'Local' -or
        $headExit -ne 0 -or $head -cnotmatch '^[0-9a-f]{40}$' -or
        $dirtyExit -ne 0 -or $dirty.Count -gt 0) {
        Report 'BLOCKED' 'Require clean Local branch and valid SHA. Do not reset existing edits.'
    } else {
        $gitReady = $true
        Report 'FOUND' "Clean Local source: $head (remote freshness not checked)."
    }
}
foreach ($lock in @(
    'Development/apps/desktop/package-lock.json',
    'Development/apps/desktop/src-tauri/Cargo.lock',
    'Development/engine/runtime-core/Cargo.lock'
)) {
    if (-not (Test-Path -LiteralPath (Join-Path $Root $lock) -PathType Leaf)) {
        Report 'BLOCKED' "Missing lockfile: $lock."
    } elseif ($gitReady) {
        & git -C $Root ls-files --error-unmatch -- $lock 2>$null | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Report 'FOUND' "Tracked lockfile present: $lock (not compiled)."
        } else {
            Report 'BLOCKED' "Lockfile exists but is not tracked by Git: $lock."
        }
    } else {
        Report 'CHECK' "Lockfile present but Git source authority is not verified: $lock."
    }
}
$requiredTools = @(
    'workspace/toolchains/node/node.exe',
    'workspace/toolchains/node/npm.cmd',
    'workspace/toolchains/powershell/pwsh.exe',
    'workspace/toolchains/cargo/bin/cargo.exe',
    'workspace/toolchains/cargo/bin/rustc.exe',
    'workspace/toolchains/cargo/bin/rustfmt.exe'
)
$missingTools = @($requiredTools | Where-Object {
    -not (Test-Path -LiteralPath (Join-Path $Root $_) -PathType Leaf)
})
if ($missingTools.Count) {
    Report 'BLOCKED' 'Portable developer toolchain incomplete (including rustfmt). Use DEV.cmd bootstrap, then doctor.'
} else {
    $unsafeTools = @($requiredTools | Where-Object {
        $item = Get-Item -LiteralPath (Join-Path $Root $_) -Force -ErrorAction Stop
        ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
    })
    if ($unsafeTools.Count) {
        Report 'BLOCKED' 'Portable tool executable is a reparse point; review workspace toolchains.'
    } else {
        Report 'FOUND' 'Portable tool files present; DEV.cmd doctor must validate versions.'
    }
}
# Same SDK path/versioned x64 resource compiler requirement as build-root-launcher.mjs.
$sdk = 'C:\Program Files (x86)\Windows Kits\10\bin'
$resourceCompilers = @()
if (Test-Path -LiteralPath $sdk -PathType Container) {
    $resourceCompilers = @(Get-ChildItem -LiteralPath $sdk -Directory -ErrorAction Stop |
        Where-Object { Test-Path -LiteralPath (Join-Path $_.FullName 'x64/rc.exe') -PathType Leaf })
}
if ($resourceCompilers.Count) {
    Report 'FOUND' 'Windows SDK x64 rc.exe exists; not executed.'
} else {
    Report 'BLOCKED' 'Windows SDK x64 rc.exe not found in expected Windows Kits 10 folder.'
}
$vswhere = 'C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe'
if (Test-Path -LiteralPath $vswhere -PathType Leaf) {
    try {
        $vs = (& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null | Out-String).Trim()
        $vcRoot = if ($LASTEXITCODE -eq 0 -and $vs) { Join-Path $vs 'VC/Tools/MSVC' } else { '' }
        if ($vcRoot -and (Test-Path -LiteralPath $vcRoot -PathType Container)) {
            $vcTools = @(Get-ChildItem -LiteralPath $vcRoot -Directory -ErrorAction Stop | Where-Object {
                (Test-Path -LiteralPath (Join-Path $_.FullName 'bin/Hostx64/x64/cl.exe') -PathType Leaf) -and
                (Test-Path -LiteralPath (Join-Path $_.FullName 'bin/Hostx64/x64/link.exe') -PathType Leaf)
            })
            if ($vcTools.Count) {
                Report 'FOUND' 'Registered MSVC x64 cl.exe and link.exe found; compilation untested.'
            } else {
                Report 'BLOCKED' 'MSVC x64 cl.exe/link.exe missing under registered Visual Studio tools.'
            }
        } else {
            Report 'BLOCKED' 'Visual Studio MSVC x64 Build Tools not registered.'
        }
    } catch {
        Report 'BLOCKED' "MSVC x64 Build Tools inspection failed: $($_.Exception.Message)"
    }
} else {
    Report 'BLOCKED' 'Visual Studio vswhere.exe missing; install MSVC x64 C++ Build Tools before building.'
}
Report 'CHECK' 'WebView2 and official Minecraft Education require separate Windows observation.'
# Parse only the existing Rust-approved standard DLL digests. This is not an
# alternative trust policy and cannot enable the experimental AI-input DLL.
try {
    $nativeSource = [IO.File]::ReadAllText((Join-Path $Root 'Development/apps/desktop/src-tauri/src/engine/independent_bootstrap.rs'))
    $policy = [regex]::Match($nativeSource,
        '(?s)const\s+REVIEWED_EDUCATION_BOOTSTRAP:\s*Option<PayloadDigests>\s*=\s*Some\(PayloadDigests\s*\{(.*?)\}\);')
    if (-not $policy.Success) { throw 'Cannot find reviewed standard native digests.' }
    $expected = @{
        'bootstrap-compat.dll' = (Get-ExpectedDigest -Body $policy.Groups[1].Value -Field 'compatibility')
        'bootstrap-enroll.dll' = (Get-ExpectedDigest -Body $policy.Groups[1].Value -Field 'enroller')
    }
    $directory = [string]$env:LAZY_TESTER_NATIVE_PAYLOAD_DIR
    $existingApp = $false
    if ([string]::IsNullOrWhiteSpace($directory)) {
        $candidate = Join-Path $Root 'App/independent-bootstrap'
        if (Test-Path -LiteralPath $candidate -PathType Container) {
            $directory = $candidate
            $existingApp = $true
        }
    }
    if ([string]::IsNullOrWhiteSpace($directory)) {
        Report 'BLOCKED' 'Reviewed native DLL pair missing. Provide LAZY_TESTER_NATIVE_PAYLOAD_DIR.'
    } elseif (-not [IO.Path]::IsPathRooted($directory) -or
        -not (Test-Path -LiteralPath $directory -PathType Container)) {
        Report 'BLOCKED' 'Reviewed native directory must be an existing absolute local path.'
    } elseif (-not (Test-OrdinaryDirectoryChain -Path $directory)) {
        Report 'BLOCKED' 'Reviewed native directory or ancestor is not an ordinary directory.'
    } else {
        $matched = $true
        foreach ($name in @('bootstrap-compat.dll', 'bootstrap-enroll.dll')) {
            $file = Join-Path $directory $name
            if (-not (Test-Path -LiteralPath $file -PathType Leaf)) {
                Report 'BLOCKED' "Missing native DLL: $name."
                $matched = $false
                continue
            }
            $item = Get-Item -LiteralPath $file -Force -ErrorAction Stop
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or
                $item.Length -lt 64 -or $item.Length -gt 33554432) {
                Report 'BLOCKED' "Unsafe native DLL type/size: $name."
                $matched = $false
                continue
            }
            $actual = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
            if ($actual -cne $expected[$name]) {
                Report 'BLOCKED' "Source-pinned SHA-256 mismatch: $name."
                $matched = $false
            } else {
                Report 'FOUND' "Source-pinned native DLL hash matches: $name."
            }
        }
        if ($matched -and $existingApp) {
            Report 'NEXT' 'Existing App DLLs match; set LAZY_TESTER_NATIVE_PAYLOAD_DIR to the absolute App\independent-bootstrap directory before dev/package.'
        }
    }
} catch {
    Report 'BLOCKED' "Native DLL verification unavailable: $($_.Exception.Message)"
}
if ($script:Blocked) {
    Write-Output "BLOCKED: $($script:Blocked) prerequisite issue(s); no build/test attempted."
    exit 2
}
Write-Output 'NO KNOWN FILE/SOURCE BLOCKER. No source test, installed app, or Minecraft acceptance was performed.'
exit 0
