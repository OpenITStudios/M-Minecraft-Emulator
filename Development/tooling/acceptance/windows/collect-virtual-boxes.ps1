# Read-only on-demand discovery of section-name candidates in the EXISTING
# official Sandboxie pilot. Never create, configure, stop or delete any box.
[CmdletBinding()]
param()
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ($env:OS -cne 'Windows_NT') { throw 'Windows required.' }

$roots = @(
  [Environment]::GetEnvironmentVariable('ProgramW6432'),
  [Environment]::GetEnvironmentVariable('ProgramFiles'),
  [Environment]::GetEnvironmentVariable('ProgramFiles(x86)')
) | Where-Object { $_ } | Select-Object -Unique
$install = @($roots | ForEach-Object { Join-Path $_ 'Sandboxie-Plus' } |
  Where-Object { Test-Path -LiteralPath (Join-Path $_ 'SbieIni.exe') -PathType Leaf })
if ($install.Count -ne 1) { throw 'Official Sandboxie installation unavailable or ambiguous.' }

$ini = Join-Path $install[0] 'SbieIni.exe'
$signature = Get-AuthenticodeSignature -LiteralPath $ini -ErrorAction Stop
if ($signature.Status -ne 'Valid') { throw 'Installed Sandboxie query tool has untrusted signature.' }

# Verified upstream query.c: "query /boxes *" excludes disabled boxes;
# "query *" lists all configuration sections with GlobalSettings first.
# Known template/user sections are not boxes. Unknown custom sections remain
# name candidates only, never engine ownership or Create Virtual authority.
$sections = @(& $ini query '*' 2>$null)
if ($LASTEXITCODE -ne 0 -or $sections.Count -lt 1 -or
    $sections.Count -gt 512 -or
    ([string]$sections[0]) -cne 'GlobalSettings') {
  throw 'Cannot reliably enumerate Sandboxie configuration sections.'
}

$seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
$boxes = @()
foreach ($item in @($sections | Select-Object -Skip 1)) {
  $name = [string]$item
  # Never normalize padded/unsafe names into a different apparent identity.
  if ($name -imatch '^(Template_|UserSettings_)' -or
      $name -ieq 'TemplateSettings') { continue }
  if ($name -cnotmatch '^[A-Za-z0-9_]{1,32}$' -or -not $seen.Add($name)) {
    throw 'Unexpected or duplicate Sandboxie section-name candidate.'
  }
  $boxes += $name
  if ($boxes.Count -gt 128) {
    throw 'Sandboxie section-name candidates exceed the bounded inventory.'
  }
}
[ordered]@{
  schema = 1
  source = 'EXISTING_OFFICIAL_SANDBOXIE_PILOT'
  evidence = 'SECTION_NAME_CANDIDATES'
  boxes = @($boxes | Sort-Object -CaseSensitive)
  ownedProviderSelected = $false
  gameplayVerified = $false
} | ConvertTo-Json -Depth 3 -Compress
