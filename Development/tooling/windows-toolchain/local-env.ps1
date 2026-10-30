# Process-scoped developer environment. No machine/user PATH or profile writes.
function Set-LocalDeveloperEnvironment {
  param([Parameter(Mandatory=$true)][string]$Root)
  $workspace = Join-Path $Root 'workspace'
  $toolchains = Join-Path $workspace 'toolchains'
  $nodeDir = Join-Path $toolchains 'node'
  $cargoHome = Join-Path $toolchains 'cargo'
  $rustupHome = Join-Path $toolchains 'rustup'
  $cacheDir = Join-Path $workspace 'cache'
  $tempDir = Join-Path $workspace 'tmp'
  foreach ($path in @($workspace,$toolchains,$nodeDir,$cargoHome,$rustupHome,$cacheDir,$tempDir)) {
    if ((Test-Path -LiteralPath $path) -and
        ((Get-Item -LiteralPath $path -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) {
      throw "Local developer storage may not be a reparse point: $path"
    }
  }
  if (-not (Test-Path -LiteralPath (Join-Path $nodeDir 'node.exe') -PathType Leaf) -or
      -not (Test-Path -LiteralPath (Join-Path $nodeDir 'npm.cmd') -PathType Leaf) -or
      -not (Test-Path -LiteralPath (Join-Path $cargoHome 'bin/cargo.exe') -PathType Leaf) -or
      -not (Test-Path -LiteralPath (Join-Path $cargoHome 'bin/rustc.exe') -PathType Leaf)) {
    throw 'Workspace-local Node/npm/Rust is missing. Run DEV.cmd bootstrap first; no global fallback.'
  }
  # One canonical compiler target for DEV.cmd. An ambient override would
  # silently create another complete Rust artifact cache and exhaust disk.
  $canonicalCargoTarget = [IO.Path]::GetFullPath((Join-Path $workspace 'builds/desktop/target'))
  if (-not [string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR)) {
    if (-not [IO.Path]::IsPathRooted($env:CARGO_TARGET_DIR) -or
        [IO.Path]::GetFullPath($env:CARGO_TARGET_DIR) -ine $canonicalCargoTarget) {
      throw 'DEV.cmd requires the canonical workspace/builds/desktop/target; external CARGO_TARGET_DIR overrides are refused.'
    }
  }
  foreach ($path in @((Join-Path $workspace 'builds'),
      (Join-Path $workspace 'builds/desktop'), $canonicalCargoTarget)) {
    if (Test-Path -LiteralPath $path) {
      $item = Get-Item -LiteralPath $path -Force
      if (-not $item.PSIsContainer -or
          ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "Cargo target must be an ordinary project-local directory: $path"
      }
    }
  }
  $env:CARGO_TARGET_DIR = $canonicalCargoTarget
  foreach ($directory in @($cacheDir,$tempDir,(Join-Path $cacheDir 'npm'),
      (Join-Path $cacheDir 'nuget'))) {
    if (-not (Test-Path -LiteralPath $directory)) {
      New-Item -ItemType Directory -Path $directory -ErrorAction Stop | Out-Null
    }
    $item = Get-Item -LiteralPath $directory -Force
    if (-not $item.PSIsContainer -or
        ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
      throw "Developer cache/temp must be a real local directory: $directory"
    }
  }
  $env:CARGO_HOME = $cargoHome
  $env:RUSTUP_HOME = $rustupHome
  $env:NPM_CONFIG_CACHE = Join-Path $cacheDir 'npm'
  $env:NPM_CONFIG_PREFIX = Join-Path $toolchains 'npm-global'
  $env:NPM_CONFIG_USERCONFIG = Join-Path $cacheDir 'npm-user.npmrc'
  $env:NPM_CONFIG_UPDATE_NOTIFIER = 'false'
  $env:NUGET_PACKAGES = Join-Path $cacheDir 'nuget'
  $env:TEMP = $tempDir
  $env:TMP = $tempDir
  $env:PATH = "$nodeDir;$(Join-Path $cargoHome 'bin');$env:PATH"
  # Do not override USERPROFILE/APPDATA/LOCALAPPDATA: those are app/account state.
}
