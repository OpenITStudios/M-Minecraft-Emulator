param(
  [Parameter(Mandatory = $true)] [string]$MinecraftInstaller,
  [string]$GuestAgentSource = ''
)

$ErrorActionPreference = 'Stop'

# Education removed Windows 32-bit support in February 2026.
# Reject an unsupported Base before touching its software or account state.
if (-not [Environment]::Is64BitOperatingSystem) {
  throw 'Minecraft Education Base preparation requires Windows 10/11 64-bit; 32-bit Windows is unsupported.'
}

if ([string]::IsNullOrWhiteSpace($GuestAgentSource)) {
  $GuestAgentSource = Join-Path $PSScriptRoot 'virtual-guest-agent.exe'
}

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($identity)
if (!$principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
  throw 'Base preparation must run as Administrator.'
}

if (!(Test-Path -LiteralPath $MinecraftInstaller -PathType Leaf)) {
  throw "Minecraft Education installer not found: $MinecraftInstaller"
}
if ([IO.Path]::GetExtension($MinecraftInstaller) -ine '.exe') {
  throw 'Managed Base preparation requires the official Minecraft Education Windows Desktop .exe installer, not an MSI, Store AppX, or other launcher.'
}

if (!(Test-Path -LiteralPath $GuestAgentSource -PathType Leaf)) {
  throw "Virtual Guest Agent not found: $GuestAgentSource"
}

$dsregPath = Join-Path $env:SystemRoot 'System32\dsregcmd.exe'
if (!(Test-Path -LiteralPath $dsregPath -PathType Leaf)) {
  throw 'Windows device-registration inspector is unavailable.'
}

$dsreg = & $dsregPath /status 2>$null
if ($LASTEXITCODE -ne 0) {
  throw 'Windows device-registration status could not be inspected.'
}
$dsregText = $dsreg -join [Environment]::NewLine
$azureAdJoined = [regex]::IsMatch($dsregText, '(?im)^\s*AzureAdJoined\s*:\s*YES\s*$')
$workplaceJoined = [regex]::IsMatch($dsregText, '(?im)^\s*WorkplaceJoined\s*:\s*YES\s*$')
if ($azureAdJoined -or $workplaceJoined) {
  throw 'Base must not be Microsoft Entra joined or Workplace joined before cloning.'
}

$store = @(Get-AppxPackage -AllUsers *MinecraftEducation* -ErrorAction SilentlyContinue)
if ($store.Count -gt 0) {
  throw 'Microsoft Store Minecraft Education is installed. Remove it before preparing a managed Desktop Base.'
}

$arguments = @('/qn', 'INSTALL_UPDATER="NONE"')
$process = Start-Process -FilePath $MinecraftInstaller -ArgumentList $arguments -Wait -PassThru
if ($process.ExitCode -ne 0) {
  throw "Minecraft Education installer failed with exit code $($process.ExitCode)."
}

$updaterTask = Get-ScheduledTask -TaskName 'Minecraft Education Automatic Updater' -ErrorAction SilentlyContinue
if ($updaterTask) {
  Disable-ScheduledTask -InputObject $updaterTask | Out-Null
}

# The official Minecraft Education EXE installer registers the Desktop
# version as a 32-bit application on 64-bit Windows. Use the same registry
# view as runtime-core profile.rs (/reg:32) regardless of PowerShell bitness.
$registry = [Microsoft.Win32.RegistryKey]::OpenBaseKey(
  [Microsoft.Win32.RegistryHive]::LocalMachine,
  [Microsoft.Win32.RegistryView]::Registry32
)
try {
  $desktopKey = $registry.OpenSubKey('SOFTWARE\Microsoft\Microsoft Studios\Minecraft Education Edition')
  if ($null -eq $desktopKey) {
    throw 'Minecraft Education Desktop is missing from the 32-bit machine registry after installation.'
  }
  try {
    $version = [string]$desktopKey.GetValue('Version', '')
  }
  finally {
    $desktopKey.Dispose()
  }
}
finally {
  $registry.Dispose()
}
if (!$version -or $version -notmatch '^\d+(\.\d+)+$') {
  throw 'Minecraft Education desktop version could not be verified after installation.'
}

$agentInstaller = Join-Path $PSScriptRoot 'install-guest-agent.ps1'
if (!(Test-Path -LiteralPath $agentInstaller -PathType Leaf)) {
  throw "Guest Agent installer script is missing: $agentInstaller"
}
& $agentInstaller -AgentSource $GuestAgentSource

$finalizeSource = Join-Path $PSScriptRoot 'finalize-base.ps1'
if (!(Test-Path -LiteralPath $finalizeSource -PathType Leaf)) {
  throw "Base finalization script is missing: $finalizeSource"
}
$guestRoot = Join-Path $env:ProgramData 'M-Bedrock\VirtualClients'
New-Item -ItemType Directory -Path $guestRoot -Force | Out-Null
$finalizeDestination = Join-Path $guestRoot 'finalize-base.ps1'
Copy-Item -LiteralPath $finalizeSource -Destination $finalizeDestination -Force

[ordered]@{
  minecraftVersion = [string]$version
  installType = 'DESKTOP'
  independentUpdaterEnabled = $false
  guestAgentInstalled = $true
  finalizeBasePath = $finalizeDestination
  microsoftDeviceRegistrationClean = $true
} | ConvertTo-Json
