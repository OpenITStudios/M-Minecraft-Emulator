param(
  [Parameter(Mandatory = $true)] [string]$Root,
  [Parameter(Mandatory = $true)] [string]$OutputPath
)

$ErrorActionPreference = 'Stop'
$rootPath = (Resolve-Path -LiteralPath $Root).Path
$contractPath = Join-Path $rootPath 'distribution\package-contract.json'
if (!(Test-Path -LiteralPath $contractPath -PathType Leaf)) {
  throw 'Package contract is missing from payload root.'
}
$contract = Get-Content -LiteralPath $contractPath -Raw | ConvertFrom-Json
if ($contract.schema -ne 1) { throw 'Unsupported package contract schema.' }

$files = @()
foreach ($relative in @($contract.requiredFiles)) {
  $normalized = $relative -replace '/', '\'
  $path = Join-Path $rootPath $normalized
  if (!(Test-Path -LiteralPath $path -PathType Leaf)) {
    throw "Required payload file is missing: $relative"
  }
  $item = Get-Item -LiteralPath $path
  $files += [ordered]@{
    path = $relative
    size = $item.Length
    sha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
  }
}

$manifest = [ordered]@{
  schema = 1
  packageContractSchema = [int]$contract.schema
  files = $files
}
$parent = Split-Path -Parent $OutputPath
if ($parent) { New-Item -ItemType Directory -Path $parent -Force | Out-Null }
$manifest | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $OutputPath -Encoding utf8
