param(
  [Parameter(Mandatory = $true)] [string]$PreviousInstaller,
  [Parameter(Mandatory = $true)] [string]$CurrentInstaller,
  [Parameter(Mandatory = $true)] [string]$InstallDirectory
)

$ErrorActionPreference = 'Stop'

foreach ($installer in @($PreviousInstaller, $CurrentInstaller)) {
  if (!(Test-Path -LiteralPath $installer -PathType Leaf)) {
    throw "Upgrade fixture installer does not exist: $installer"
  }
}

$previousVersion = [Diagnostics.FileVersionInfo]::GetVersionInfo((Resolve-Path $PreviousInstaller)).ProductVersion
$currentVersion = [Diagnostics.FileVersionInfo]::GetVersionInfo((Resolve-Path $CurrentInstaller)).ProductVersion
if ([string]::IsNullOrWhiteSpace($previousVersion) -or [string]::IsNullOrWhiteSpace($currentVersion)) {
  throw 'Upgrade fixture requires installers with product version metadata.'
}
try {
  $previousSemver = [version]$previousVersion
  $currentSemver = [version]$currentVersion
} catch {
  throw 'Upgrade fixture installer versions are not comparable.'
}
if ($currentSemver -le $previousSemver) {
  throw "Upgrade fixture requires CurrentInstaller newer than PreviousInstaller. previous=$previousVersion current=$currentVersion"
}

$policyPath = Join-Path $PSScriptRoot 'install-lifecycle-policy.json'
$policy = Get-Content -LiteralPath $policyPath -Raw | ConvertFrom-Json
if ($policy.schema -ne 1 -or $policy.installer.allowUpgrade -ne $true -or $policy.installer.allowDowngrade -ne $false -or $policy.runtimeData.owner -ne 'runtime-core' -or $policy.runtimeData.installerMayMigrate -ne $false -or $policy.runtimeData.installerMayDelete -ne $false) {
  throw 'Installer lifecycle policy is invalid or unsupported.'
}

$runtimeRoot = Join-Path $env:LOCALAPPDATA 'M-Bedrock\VirtualClients'
New-Item -ItemType Directory -Path $runtimeRoot -Force | Out-Null
$sentinel = Join-Path $runtimeRoot 'upgrade-fixture-preserve.txt'
'preserve-runtime-data' | Set-Content -LiteralPath $sentinel -Encoding utf8

if (Test-Path -LiteralPath $InstallDirectory) {
  Remove-Item -LiteralPath $InstallDirectory -Recurse -Force
}

$installPrevious = Start-Process -FilePath $PreviousInstaller -ArgumentList @('/S', "/D=$InstallDirectory") -Wait -PassThru
if ($installPrevious.ExitCode -ne 0) { throw "Previous installer failed: $($installPrevious.ExitCode)" }

$upgrade = Start-Process -FilePath $CurrentInstaller -ArgumentList @('/S', "/D=$InstallDirectory") -Wait -PassThru
if ($upgrade.ExitCode -ne 0) { throw "Current installer upgrade failed: $($upgrade.ExitCode)" }
if (!(Test-Path -LiteralPath $sentinel -PathType Leaf)) { throw 'Upgrade removed runtime data.' }

$contractPath = Join-Path $InstallDirectory 'distribution\package-contract.json'
if (!(Test-Path -LiteralPath $contractPath -PathType Leaf)) { throw 'Upgraded installation is missing package contract.' }
$contract = Get-Content -LiteralPath $contractPath -Raw | ConvertFrom-Json
foreach ($relative in @($contract.requiredFiles)) {
  $path = Join-Path $InstallDirectory ($relative -replace '/', '\')
  if (!(Test-Path -LiteralPath $path -PathType Leaf)) { throw "Upgrade is missing required file: $relative" }
}

[ordered]@{
  schema = 1
  previousVersion = $previousVersion
  currentVersion = $currentVersion
  runtimeDataPreserved = $true
  packageContractSatisfied = $true
} | ConvertTo-Json -Depth 4
