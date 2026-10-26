# Build/provenance ONLY. No driver installation, service changes or Sandboxie box mutations.
# Run from a trusted Windows development machine after cloning the exact upstream revision.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$SourceRoot,
    [Parameter(Mandatory = $true)][string]$ReportPath,
    [switch]$Build,
    [string]$StageOutput
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT') { throw 'This source build requires Windows with VS2022 MSBuild and WDK.' }
if ($StageOutput -and -not $Build) { throw 'StageOutput requires explicit -Build.' }

$specPath = Join-Path $PSScriptRoot 'upstream-baseline.json'
$spec = Get-Content -LiteralPath $specPath -Raw -Encoding UTF8 | ConvertFrom-Json
if ($spec.schema -ne 1 -or $spec.purpose -ne 'SOURCE_ONLY_UNMODIFIED_UPSTREAM_BUILD_PROVENANCE') {
    throw 'Unsupported or untrusted upstream manifest.'
}
$root = (Resolve-Path -LiteralPath $SourceRoot -ErrorAction Stop).ProviderPath.TrimEnd('\')
$reportFull = [IO.Path]::GetFullPath($ReportPath)
$prefix = $root + [IO.Path]::DirectorySeparatorChar
if ($reportFull.Equals($root, [StringComparison]::OrdinalIgnoreCase) -or
    $reportFull.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'ReportPath must be OUTSIDE the upstream source/build tree.'
}
if (Test-Path -LiteralPath $reportFull) {
    throw 'Baseline build report already exists; preserve the previous evidence.'
}
if (-not (Test-Path -LiteralPath (Join-Path $root 'Sandboxie\Sandbox.sln') -PathType Leaf)) {
    throw 'SourceRoot is not a complete Sandboxie checkout.'
}
$gitCmd = Get-Command git -ErrorAction Stop
$actualCommit = (& $gitCmd.Source -C $root rev-parse HEAD | Out-String).Trim()
if ($LASTEXITCODE -ne 0 -or $actualCommit -cne [string]$spec.upstream.commit) {
    throw "Source revision mismatch; expected $($spec.upstream.commit). No build performed."
}
# No changes, even to untracked files, before compiling the pinned baseline.
$dirty = @(& $gitCmd.Source -C $root status --porcelain=v1 --untracked-files=all)
if ($LASTEXITCODE -ne 0 -or $dirty.Count -ne 0) {
    throw 'Upstream checkout is not clean; refusing to label modified source as baseline.'
}
$tagCommit = (& $gitCmd.Source -C $root rev-parse "$($spec.upstream.tag)^{commit}" | Out-String).Trim()
if ($LASTEXITCODE -ne 0 -or $tagCommit -cne $actualCommit) {
    throw 'Upstream tag does not point to the pinned commit.'
}

$parent = Split-Path -Parent $reportFull
[IO.Directory]::CreateDirectory($parent) | Out-Null
# Validate the destination before native compilation, not after the build
# succeeds. The assembler itself will also refuse symlinks/existing paths.
if ($StageOutput) {
    $stageDestination = [IO.Path]::GetFullPath($StageOutput)
    if ($stageDestination.Equals($root, [StringComparison]::OrdinalIgnoreCase) -or
        $stageDestination.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase) -or
        $root.StartsWith($stageDestination.TrimEnd('\') + '\',
            [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Baseline payload cannot overlap upstream source checkout.'
    }
    if ($reportFull.Equals($stageDestination, [StringComparison]::OrdinalIgnoreCase) -or
        $reportFull.StartsWith($stageDestination.TrimEnd('\') + '\',
            [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Baseline build report must not be inside the staged payload directory.'
    }
    if (Test-Path -LiteralPath $stageDestination) {
        throw 'Baseline stage output already exists; refusing overwrite.'
    }
    $stageParent = Split-Path -Parent $stageDestination
    if (-not (Test-Path -LiteralPath $stageParent -PathType Container)) {
        throw 'Baseline stage parent directory must exist before compilation.'
    }
}
if ($Build) {
    foreach ($stage in @($spec.buildStages)) {
        $logPath = Join-Path $parent ("baseline-" + [string]$stage.id + ".log")
        if ($reportFull.Equals($logPath, [StringComparison]::OrdinalIgnoreCase)) {
            throw 'Baseline build report path collides with a reserved stage log.'
        }
        if (Test-Path -LiteralPath $logPath) {
            throw "Baseline build log already exists; preserve evidence: $logPath"
        }
    }
}
$stages = @()
$nodeCmd = Get-Command node.exe -ErrorAction SilentlyContinue
if ($null -eq $nodeCmd) {
    throw 'Node.js is required to independently validate the pinned manifest and artifact report.'
}
& $nodeCmd.Source (Join-Path $PSScriptRoot 'validate-baseline.mjs') $specPath
if ($LASTEXITCODE -ne 0) {
    throw 'Pinned upstream manifest failed independent verification; no build performed.'
}
# Pin provenance of the actual local source checkout, not just HEAD SHA and
# a copied tag. Refuse changed upstream remote, detached branch, shallow
# history, nested worktrees and dirty local source before native compilation.
& $nodeCmd.Source (Join-Path $PSScriptRoot 'verify-checkout.mjs') --source-root $root
if ($LASTEXITCODE -ne 0) {
    throw 'Isolation source checkout provenance failed; no build performed.'
}
$msbuildCmd = Get-Command msbuild.exe -ErrorAction SilentlyContinue
if ($Build -and $null -eq $msbuildCmd) {
    throw 'msbuild.exe unavailable. Use a VS2022 Developer PowerShell with v143, MFC, Windows SDK and WDK.'
}
$msbuildVersion = 'NOT_EXECUTED'
if ($null -ne $msbuildCmd) {
    $msbuildVersion = ((& $msbuildCmd.Source -version -nologo | Out-String).Trim())
}
# A previous ignored build output can survive "git status --clean".
# Do not silently hash stale files or delete someone's prior build directory.
if ($Build) {
    foreach ($rel in @($spec.requiredArtifacts) + @($spec.optionalSymbols)) {
        $artifact = Join-Path $root ([string]$rel -replace '/', '\')
        if (Test-Path -LiteralPath $artifact -PathType Leaf) {
            throw 'Existing build output found. Use a separate clean source checkout for the unmodified baseline; never auto-delete artifacts.'
        }
    }
}
foreach ($stage in @($spec.buildStages)) {
    $stageResult = [ordered]@{
        id = [string]$stage.id
        solution = [string]$stage.solution
        platform = [string]$stage.platform
        state = 'NOT_EXECUTED'
        exitCode = $null
    }
    if ($Build) {
        $solutionPath = Join-Path $root ([string]$stage.solution -replace '/', '\')
        if (-not (Test-Path -LiteralPath $solutionPath -PathType Leaf)) {
            throw "Missing upstream solution: $($stage.solution)"
        }
        # Build logs remain outside source, do not include accounts, license certificates, or world data.
        $logPath = Join-Path $parent ("baseline-" + [string]$stage.id + ".log")
        Push-Location -LiteralPath $root
        try {
            & $msbuildCmd.Source /t:build $solutionPath /p:Configuration=SbieRelease ("/p:Platform=" + [string]$stage.platform) /maxcpucount:8 2>&1 |
                Out-File -LiteralPath $logPath -Encoding UTF8 -NoClobber
            $code = $LASTEXITCODE
        } finally {
            Pop-Location
        }
        $stageResult.exitCode = [int]$code
        $stageResult.state = if ($code -eq 0) { 'COMPILED' } else { 'FAILED' }
        $stages += [pscustomobject]$stageResult
        if ($code -ne 0) {
            throw "Upstream build stage $($stage.id) failed. Inspect local log only: $logPath"
        }
    } else {
        $stages += [pscustomobject]$stageResult
    }
}
# Reject source modifications introduced by build scripts before attesting output.
$dirtyAfter = @(& $gitCmd.Source -C $root status --porcelain=v1 --untracked-files=no)
if ($LASTEXITCODE -ne 0 -or $dirtyAfter.Count -ne 0) {
    throw 'Tracked upstream source changed during build; provenance cannot be claimed.'
}
# Files are collected in place and hashed. This script does NOT copy, sign, load
# or deploy .sys/.dll/.exe files into Windows.
function Get-ArtifactRecord([string]$rel, [string]$kind) {
    if ($rel -notmatch '^[A-Za-z0-9_.\/-]+$' -or $rel.Contains('..') -or $rel.StartsWith('/')) {
        throw 'Unsafe path in upstream artifact manifest.'
    }
    $file = Join-Path $root ($rel -replace '/', '\')
    $found = Test-Path -LiteralPath $file -PathType Leaf
    $hash = $null
    $bytes = $null
    $sign = 'NOT_EXECUTED'
    if ($found) {
        $bytes = (Get-Item -LiteralPath $file).Length
        $hash = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($kind -eq 'required') {
            $sign = (Get-AuthenticodeSignature -LiteralPath $file).Status.ToString()
        }
    }
    [pscustomobject][ordered]@{
        path = $rel
        kind = $kind
        state = if ($found) { 'PRESENT' } else { 'MISSING' }
        sizeBytes = $bytes
        sha256 = $hash
        authenticode = $sign
    }
}
$artifacts = @()
foreach ($rel in @($spec.requiredArtifacts)) { $artifacts += Get-ArtifactRecord ([string]$rel) 'required' }
foreach ($rel in @($spec.optionalSymbols)) { $artifacts += Get-ArtifactRecord ([string]$rel) 'optional-symbol' }
foreach ($rel in @($spec.sourceResources)) { $artifacts += Get-ArtifactRecord ([string]$rel) 'source-resource' }

$missingRequired = @($artifacts | Where-Object { $_.kind -eq 'required' -and $_.state -ne 'PRESENT' })
$missingResources = @($artifacts | Where-Object { $_.kind -eq 'source-resource' -and $_.state -ne 'PRESENT' })
$status = if ($Build -and $missingRequired.Count -eq 0 -and $missingResources.Count -eq 0) {
    'BUILD_ARTIFACTS_COLLECTED'
} elseif ($Build) { 'BUILD_OUTPUT_INCOMPLETE' } else { 'COLLECTION_ONLY_NOT_BUILD_PROOF' }
$report = [pscustomobject][ordered]@{
    schema = 1
    sourceRepository = [string]$spec.upstream.repository
    sourceTag = [string]$spec.upstream.tag
    sourceCommit = $actualCommit
    sourceCleanAtStart = $true
    compiledInThisInvocation = [bool]$Build
    msbuildVersion = $msbuildVersion
    stageResults = @($stages)
    artifactResult = $status
    artifacts = @($artifacts)
    signingLoadAcceptance = 'NOT_EXECUTED'
    minecraftLoginGameplayAcceptance = 'NOT_EXECUTED'
    distributionApproval = 'NOT_APPROVED'
}
# Never write success claims about driver loading, game sign-in or resource savings.
$report | ConvertTo-Json -Depth 10 | Out-File -LiteralPath $reportFull -Encoding UTF8 -NoClobber
# The report is not accepted merely because the PowerShell collector produced JSON.
# The standalone validator checks ordered stage evidence, hashes, and disallows
# any claim of signed/loaded kernel driver or successful Minecraft gameplay.
& $nodeCmd.Source (Join-Path $PSScriptRoot 'validate-baseline.mjs') $specPath $reportFull
if ($LASTEXITCODE -ne 0) {
    throw 'Isolation artifact report is inconsistent. Do not use these binaries.'
}
# Independent check against the ACTUAL produced files, not only reported
# SHA-256 strings. Read-only and rejects missing, altered or symlinked artifacts.
& $nodeCmd.Source (Join-Path $PSScriptRoot 'verify-artifact-files.mjs') $specPath $reportFull $root
if ($LASTEXITCODE -ne 0) {
    throw 'Artifact bytes do not match report metadata. Do not use these binaries.'
}
Write-Output ("Artifact inventory written: " + $reportFull)
Write-Output ("Result: " + $status + " / required missing: " + $missingRequired.Count)
if ($Build -and ($missingRequired.Count -gt 0 -or $missingResources.Count -gt 0)) {
    throw 'Upstream build output is incomplete; do not deploy or claim engine readiness.'
}
if ($Build -and $StageOutput) {
    # A private noninstallable payload, never driver/service installation.
    & $nodeCmd.Source (Join-Path $PSScriptRoot 'assemble-engine.mjs') --assemble --source-root $root --manifest $specPath --report $reportFull --output $StageOutput
    if ($LASTEXITCODE -ne 0) {
        throw 'Owned engine assembly failed. Never install an unverified payload.'
    }
}
