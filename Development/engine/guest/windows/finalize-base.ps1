param(
  [switch]$ConfirmGeneralize
)

$ErrorActionPreference = 'Stop'

if (!$ConfirmGeneralize) {
  throw 'Base finalization is destructive. Re-run with -ConfirmGeneralize.'
}

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($identity)
if (!$principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
  throw 'Base finalization must run as Administrator.'
}

$vmtoolsd = Join-Path $env:ProgramFiles 'VMware\VMware Tools\vmtoolsd.exe'
if (!(Test-Path -LiteralPath $vmtoolsd -PathType Leaf)) {
  throw 'VMware Tools is required for Base finalization.'
}

$stateKey = 'guestinfo.virtualclients.baseState'
$state = (& $vmtoolsd --cmd "info-get $stateKey").Trim()
if ($LASTEXITCODE -ne 0 -or $state -ne 'REGISTERED') {
  throw "Base must be REGISTERED before finalization. Current state: $state"
}

$dsregPath = Join-Path $env:SystemRoot 'System32\dsregcmd.exe'
$dsreg = & $dsregPath /status 2>$null
if ($LASTEXITCODE -ne 0) {
  throw 'Windows device-registration status could not be inspected.'
}
$dsregText = $dsreg -join [Environment]::NewLine
$azureAdJoined = [regex]::IsMatch($dsregText, '(?im)^\s*AzureAdJoined\s*:\s*YES\s*$')
$workplaceJoined = [regex]::IsMatch($dsregText, '(?im)^\s*WorkplaceJoined\s*:\s*YES\s*$')
if ($azureAdJoined -or $workplaceJoined) {
  throw 'Base must not be Microsoft Entra joined or Workplace joined before generalization.'
}

$sysprep = Join-Path $env:SystemRoot 'System32\Sysprep\Sysprep.exe'
if (!(Test-Path -LiteralPath $sysprep -PathType Leaf)) {
  throw 'Windows Sysprep is unavailable.'
}

& $vmtoolsd --cmd "info-set $stateKey FINALIZING" | Out-Null
if ($LASTEXITCODE -ne 0) {
  throw 'Could not mark Base as FINALIZING through VMware guestinfo.'
}

$process = Start-Process -FilePath $sysprep -ArgumentList @(
  '/generalize',
  '/oobe',
  '/mode:vm',
  '/quit',
  '/quiet'
) -Wait -PassThru

if ($process.ExitCode -ne 0) {
  & $vmtoolsd --cmd "info-set $stateKey REGISTERED" | Out-Null
  throw "Sysprep generalization failed with exit code $($process.ExitCode)."
}

& $vmtoolsd --cmd "info-set $stateKey FINALIZED" | Out-Null
if ($LASTEXITCODE -ne 0) {
  throw 'Sysprep succeeded but Base finalization state could not be persisted.'
}

$finalState = (& $vmtoolsd --cmd "info-get $stateKey").Trim()
if ($LASTEXITCODE -ne 0 -or $finalState -ne 'FINALIZED') {
  throw "Base finalization state verification failed: $finalState"
}

shutdown.exe /s /t 0
