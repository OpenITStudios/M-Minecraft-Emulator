param(
  [Parameter(Mandatory = $true)] [string]$Version,
  [Parameter(Mandatory = $true)] [string]$InstallerName,
  [Parameter(Mandatory = $true)] [string]$AuthenticodeThumbprint,
  [Parameter(Mandatory = $true)] [string]$InstallerSha256,
  [Parameter(Mandatory = $true)] [string]$OutputPath
)

$ErrorActionPreference = 'Stop'

if ($Version -notmatch '^\d+\.\d+\.\d+$') {
  throw 'Virtual Clients update version must be MAJOR.MINOR.PATCH.'
}

$expectedInstallerName = "M-Bedrock-Virtual-Clients-$Version-windows-x86_64.exe"
if ([IO.Path]::GetFileName($InstallerName) -ne $InstallerName -or $InstallerName -ne $expectedInstallerName) {
  throw "Virtual Clients installer name must be exactly $expectedInstallerName."
}

if ($InstallerSha256 -notmatch '^[A-Fa-f0-9]{64}$') {
  throw 'Installer SHA-256 is invalid.'
}

$thumbprint = $AuthenticodeThumbprint.Replace(' ', '').ToUpperInvariant()
if ($thumbprint -notmatch '^[A-F0-9]{40}$') {
  throw 'Authenticode certificate thumbprint is invalid.'
}

$policyPath = Join-Path $PSScriptRoot 'release-channel.json'
$policy = Get-Content -LiteralPath $policyPath -Raw | ConvertFrom-Json

if ($policy.schema -ne 1 -or $policy.channel -ne 'stable') {
  throw 'Virtual Clients release-channel policy is invalid.'
}

$expectedManifestEndpoint = "https://github.com/$($policy.repository)/releases/latest/download/$($policy.manifestAsset)"

if (
  $policy.repository -ne 'MIVUBI-STD/M-Bedrock-Dev' -or
  $policy.releaseTagPrefix -ne 'virtual-clients-v' -or
  $policy.platform -ne 'windows-x86_64' -or
  $policy.manifestAsset -ne 'latest.json' -or
  $policy.manifestEndpoint -ne $expectedManifestEndpoint -or
  $policy.checkMode -ne 'startup-once' -or
  $policy.applyGate -ne 'all-virtuals-stopped'
) {
  throw 'Virtual Clients release-channel identity is invalid.'
}

if ($policy.selfUpdateRuntimeEnabled -ne $false) {
  throw 'Runtime self-update activation requires a separate reviewed change.'
}

$tag = "$($policy.releaseTagPrefix)$Version"
$assetUrl = "https://github.com/$($policy.repository)/releases/download/$tag/$([Uri]::EscapeDataString($InstallerName))"

$manifest = [ordered]@{
  version = $Version
  platforms = [ordered]@{
    $policy.platform = [ordered]@{
      authenticodeThumbprint = $thumbprint
      url = $assetUrl
      sha256 = $InstallerSha256.ToLowerInvariant()
    }
  }
}

$parent = Split-Path -Parent $OutputPath
if ($parent) {
  New-Item -ItemType Directory -Path $parent -Force | Out-Null
}

$manifest | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $OutputPath -Encoding utf8
