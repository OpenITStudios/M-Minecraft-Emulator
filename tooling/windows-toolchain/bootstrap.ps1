# Opt-in, checkout-local prerequisite installer. Windows PowerShell 5.1 entrypoint.
# Downloads and extracts ONLY into workspace; never installs drivers or the app.
$ErrorActionPreference = 'Stop'
$Root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$manifest = Get-Content -LiteralPath (Join-Path $Root 'toolchain.json') -Raw | ConvertFrom-Json
if ($env:OS -cne 'Windows_NT' -or -not [Environment]::Is64BitOperatingSystem) {
  throw 'Portable bootstrap supports Windows x64 only.'
}
if ([string]$manifest.node.version -cne '24.21.0' -or
    [string]$manifest.npm.version -cne '11.19.0' -or
    [string]$manifest.rust.version -cne '1.98.1') {
  throw 'Unreviewed toolchain version: update authenticated artifact pins before bootstrap.'
}
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$workspace = Join-Path $Root 'workspace'
$downloads = Join-Path $workspace 'downloads'
$tools = Join-Path $workspace 'toolchains'
$temp = Join-Path $workspace 'tmp'
$cache = Join-Path $workspace 'cache'
function Assert-RealDirectory {
  param([string]$Path)
  if (-not (Test-Path -LiteralPath $Path)) {
    [IO.Directory]::CreateDirectory($Path) | Out-Null
  }
  $item = Get-Item -LiteralPath $Path -Force -ErrorAction Stop
  if (-not $item.PSIsContainer -or
      ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
    throw "Refuse non-directory or reparse-point workspace path: $Path"
  }
}
foreach ($dir in @($workspace,$downloads,$tools,$temp,$cache,(Join-Path $tools 'cargo'),(Join-Path $tools 'rustup'))) {
  Assert-RealDirectory $dir
}
function Get-VerifiedDownload {
  param([string]$Url,[string]$FileName,[string]$Sha256)
  $parsed = $null
  if (-not [Uri]::TryCreate($Url,[UriKind]::Absolute,[ref]$parsed) -or
      $parsed.Scheme -cne 'https' -or
      $parsed.Host -notin @('nodejs.org','github.com','static.rust-lang.org') -or
      $FileName -notmatch '^[A-Za-z0-9._-]+$' -or
      $Sha256 -notmatch '^[a-fA-F0-9]{64}$') {
    throw 'Unapproved bootstrap download metadata.'
  }
  $target = Join-Path $downloads $FileName
  if (-not (Test-Path -LiteralPath $target)) {
    $pending = "$target.partial"
    if (Test-Path -LiteralPath $pending) {
      throw "Previous partial download exists. Review it manually: $pending"
    }
    Write-Host "Downloading $FileName into workspace/downloads..."
    Invoke-WebRequest -Uri $Url -OutFile $pending -UseBasicParsing
    $digest = (Get-FileHash -LiteralPath $pending -Algorithm SHA256).Hash
    if ($digest -ine $Sha256) {
      throw "SHA-256 mismatch for $FileName; untrusted partial was preserved."
    }
    Move-Item -LiteralPath $pending -Destination $target -ErrorAction Stop
  }
  $info = Get-Item -LiteralPath $target -Force
  if ($info.PSIsContainer -or
      ($info.Attributes -band [IO.FileAttributes]::ReparsePoint) -or
      (Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash -ine $Sha256) {
    throw "Untrusted or modified cached download: $target"
  }
  return $target
}
function Expand-PinnedZip {
  param([string]$Archive,[string]$Destination,[string]$NestedDirectory)
  if (Test-Path -LiteralPath $Destination) {
    if ((Get-Item -LiteralPath $Destination -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) {
      throw "Refuse junction toolchain destination: $Destination"
    }
    Write-Host "Preserving existing portable runtime: $Destination"
    return
  }
  $staging = Join-Path $temp ('bootstrap-' + [guid]::NewGuid().ToString('N'))
  [IO.Directory]::CreateDirectory($staging) | Out-Null
  try {
    Expand-Archive -LiteralPath $Archive -DestinationPath $staging -ErrorAction Stop
    $content = if ($NestedDirectory) { Join-Path $staging $NestedDirectory } else { $staging }
    if (-not (Test-Path -LiteralPath $content -PathType Container)) {
      throw 'Downloaded runtime archive has an unexpected directory layout.'
    }
    Move-Item -LiteralPath $content -Destination $Destination -ErrorAction Stop
  } finally {
    # This unique scratch location was created solely by this bootstrap.
    if (Test-Path -LiteralPath $staging) {
      Remove-Item -LiteralPath $staging -Recurse -Force -ErrorAction Stop
    }
  }
}
$portable = $manifest.portableWindowsX64
if ([string]$manifest.powershell.version -cne '7.5.4' -or
    [string]$portable.node.name -cne 'node-v24.21.0-win-x64.zip' -or
    [string]$portable.powershell.name -cne 'PowerShell-7.5.4-win-x64.zip' -or
    [string]$portable.rustup.url -cne 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe') {
  throw 'Unreviewed portable archive identities in toolchain.json.'
}
$nodeZip = Get-VerifiedDownload $portable.node.url $portable.node.name $portable.node.sha256
$psZip = Get-VerifiedDownload $portable.powershell.url $portable.powershell.name $portable.powershell.sha256
Expand-PinnedZip -Archive $nodeZip -Destination (Join-Path $tools 'node') -NestedDirectory $portable.node.directory
Expand-PinnedZip -Archive $psZip -Destination (Join-Path $tools 'powershell') -NestedDirectory $portable.powershell.directory
$node = Join-Path $tools 'node/node.exe'
$npm = Join-Path $tools 'node/npm.cmd'
$pwsh = Join-Path $tools 'powershell/pwsh.exe'
if (-not (Test-Path -LiteralPath $node -PathType Leaf) -or
    -not (Test-Path -LiteralPath $npm -PathType Leaf) -or
    -not (Test-Path -LiteralPath $pwsh -PathType Leaf) -or
    ((& $node --version).Trim() -cne 'v24.21.0') -or
    ((& $npm --version).Trim() -cne '11.19.0') -or
    ((& $pwsh --version).Trim() -cne 'PowerShell 7.5.4')) {
  throw 'Portable Node/npm/PowerShell runtime identity mismatch.'
}
$rustupUrl = [string]$portable.rustup.url
$checksumName = 'rustup-init.exe.sha256'
$checksumFile = Join-Path $downloads $checksumName
if (-not (Test-Path -LiteralPath $checksumFile)) {
  $partial = "$checksumFile.partial"
  if (Test-Path -LiteralPath $partial) { throw 'Stale rustup checksum download; inspect it first.' }
  Invoke-WebRequest -Uri $portable.rustup.sha256Url -OutFile $partial -UseBasicParsing
  Move-Item -LiteralPath $partial -Destination $checksumFile
}
$checksumText = (Get-Content -LiteralPath $checksumFile -Raw).Trim()
if ($checksumText -match '^([a-fA-F0-9]{64})(?:\s+.*)?$') {
  $rustupHash = $Matches[1]
} else {
  throw 'Official rustup checksum file is malformed.'
}
$rustupExe = Get-VerifiedDownload $rustupUrl 'rustup-init.exe' $rustupHash
$env:CARGO_HOME = Join-Path $tools 'cargo'
$env:RUSTUP_HOME = Join-Path $tools 'rustup'
$env:TEMP = $temp
$env:TMP = $temp
$env:PATH = "$(Join-Path $tools 'node');$(Join-Path $env:CARGO_HOME 'bin');$env:PATH"
$cargo = Join-Path $env:CARGO_HOME 'bin/cargo.exe'
$rustc = Join-Path $env:CARGO_HOME 'bin/rustc.exe'
$rustup = Join-Path $env:CARGO_HOME 'bin/rustup.exe'
if (-not (Test-Path -LiteralPath $cargo)) {
  Write-Host 'Installing pinned Rust into workspace/toolchains (no global PATH changes)...'
  & $rustupExe -y --no-modify-path --default-host 'x86_64-pc-windows-msvc' --default-toolchain $manifest.rust.version --profile minimal
  if ($LASTEXITCODE -ne 0) { throw 'Checkout-local rustup setup failed.' }
}
if (-not (Test-Path -LiteralPath $cargo) -or
    -not (Test-Path -LiteralPath $rustc) -or
    -not (Test-Path -LiteralPath $rustup)) {
  throw 'Rustup did not provision the required local tools.'
}
& $rustup component add rustfmt --toolchain $manifest.rust.version
if ($LASTEXITCODE -ne 0) { throw 'Pinned local rustfmt installation failed.' }
$rustVersion = (& $rustc --version | Out-String).Trim()
if ($LASTEXITCODE -ne 0 -or $rustVersion -notmatch '^rustc 1\.98\.1\b') {
  throw "Incorrect workspace-local Rust version: $rustVersion"
}
Write-Host 'Local toolchains ready. Use DEV.cmd doctor, then DEV.cmd preflight.'
Write-Host 'Git and Visual Studio/WDK remain OS-level prerequisites; no drivers/games were installed.'
