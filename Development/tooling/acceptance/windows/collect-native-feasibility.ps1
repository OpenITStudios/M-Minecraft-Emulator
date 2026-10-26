[CmdletBinding()]
param(
  [ValidateSet('BASELINE', 'TWO_CLIENT')]
  [string]$Stage = 'BASELINE',
  [ValidateRange(0, 2)]
  [int]$LaunchRequests = 0,
  [switch]$ConfirmLaunch,
  [ValidateSet('YES', 'NO', 'UNKNOWN')]
  [string]$DistinctAccounts = 'UNKNOWN',
  [ValidateSet('YES', 'NO', 'UNKNOWN')]
  [string]$BothWorldsPlayable = 'UNKNOWN',
  [ValidateSet('YES', 'NO', 'UNKNOWN')]
  [string]$IndependentInput = 'UNKNOWN',
  [ValidateSet('YES', 'NO', 'UNKNOWN')]
  [string]$AccountCrossingObserved = 'UNKNOWN',
  [ValidateSet('YES', 'NO', 'UNKNOWN')]
  [string]$MultiplayerJoined = 'UNKNOWN',
  [ValidateSet('YES', 'NO', 'UNKNOWN')]
  [string]$ReopenAndRebootPersistent = 'UNKNOWN',
  [ValidateRange(0, 1440)]
  [int]$ObservedMinutes = 0,
  [ValidatePattern('^(?:$|[A-Za-z0-9_.-]{1,80})$')]
  [string]$EvidenceReference = '',
  [string]$OutputDirectory = ''
)

$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT') {
  throw 'Native feasibility requires a Windows host.'
}

$observations = @($DistinctAccounts, $BothWorldsPlayable, $IndependentInput,
  $AccountCrossingObserved, $MultiplayerJoined, $ReopenAndRebootPersistent)
if ($Stage -eq 'BASELINE' -and (
  @($observations | Where-Object { $_ -ne 'UNKNOWN' }).Count -gt 0 -or
  $ObservedMinutes -ne 0 -or $EvidenceReference
)) {
  throw 'Operator gameplay observations belong only to TWO_CLIENT.'
}
if ($Stage -eq 'TWO_CLIENT' -and $LaunchRequests -gt 0) {
  throw 'Launch requests and post-gameplay observations must be separate runs.'
}
if ($LaunchRequests -gt 0 -and -not $ConfirmLaunch) {
  throw 'Pass -ConfirmLaunch to explicitly request opening the official application.'
}
if ($LaunchRequests -eq 0 -and $ConfirmLaunch) {
  throw '-ConfirmLaunch requires -LaunchRequests 1 or 2.'
}

$session = [System.Diagnostics.Process]::GetCurrentProcess().SessionId
if ($LaunchRequests -gt 0 -and $session -eq 0) {
  throw 'The official game must be launched from an interactive user session.'
}

# Read only the documented Windows Desktop product version from its 32-bit
# registry view. The Store package is not a second independent game install.
$desktopVersion = $null
$registry = [Microsoft.Win32.RegistryKey]::OpenBaseKey(
  [Microsoft.Win32.RegistryHive]::LocalMachine,
  [Microsoft.Win32.RegistryView]::Registry32
)
try {
  $entry = $registry.OpenSubKey('SOFTWARE\Microsoft\Microsoft Studios\Minecraft Education Edition')
  if ($null -ne $entry) {
    try { $desktopVersion = [string]$entry.GetValue('Version', '') }
    finally { $entry.Dispose() }
  }
}
finally { $registry.Dispose() }
if ($desktopVersion -notmatch '^\d+(?:\.\d+){1,4}$') { $desktopVersion = $null }

$storePresent = @(Get-AppxPackage -Name 'Microsoft.MinecraftEducationEdition' -ErrorAction SilentlyContinue).Count -gt 0
$registered = @(Get-StartApps -ErrorAction Stop | Where-Object {
  $_.Name -eq 'Minecraft Education' -or
  $_.Name -eq 'Minecraft: Education Edition' -or
  $_.Name -eq 'Minecraft Education Edition'
})
$appId = $null
if ($registered.Count -eq 1) {
  $candidate = [string]$registered[0].AppID
  if ($candidate.Length -le 256 -and $candidate -match '^[A-Za-z0-9_.!+{}-]+$') {
    $appId = $candidate
  }
}
$uniqueRegisteredApp = $null -ne $appId

# Explicitly request Windows' registered official Start app; do not copy,
# patch, inject, bypass any instance restriction, or automate sign-in.
if ($LaunchRequests -gt 0) {
  if ($null -eq $desktopVersion -or $storePresent -or -not $uniqueRegisteredApp) {
    throw 'A unique Minecraft Education Desktop installation is required to request launch.'
  }
  for ($i = 0; $i -lt $LaunchRequests; $i++) {
    Start-Process -FilePath 'explorer.exe' -ArgumentList ('shell:AppsFolder\' + $appId) -ErrorAction Stop | Out-Null
    if ($i -lt ($LaunchRequests - 1)) { Start-Sleep -Seconds 2 }
  }
}

# Windows process names are candidates, not verified game identities.
# Do not record process paths, command lines, window titles, accounts, or worlds.
$candidates = @(
  Get-Process -ErrorAction SilentlyContinue |
    Where-Object {
      $_.SessionId -eq $session -and
      $_.ProcessName -match '^(Minecraft\.Windows|Minecraft.*Education.*|Education.*Minecraft.*)$'
    } |
    Sort-Object Id |
    ForEach-Object {
      $window = [string]::Empty
      if ($_.MainWindowHandle -ne [IntPtr]::Zero) {
        $window = '0x' + $_.MainWindowHandle.ToInt64().ToString('x')
      }
      [ordered]@{
        pid = [int]$_.Id
        windowHandle = $(if ($window) { $window } else { $null })
        workingSetMb = [math]::Round(($_.WorkingSet64 / 1MB), 1)
      }
    }
)
if ($candidates.Count -gt 16) { throw 'Too many name-matched processes; inspect interactively.' }
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
  $OutputDirectory = Join-Path $env:TEMP 'MinecraftInstanceManager\NativeFeasibility'
}
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$reference = if ($EvidenceReference) { $EvidenceReference } else { $null }

$report = [ordered]@{
  schema = 1
  scope = 'NATIVE_TWO_CLIENT_FEASIBILITY'
  stage = $Stage
  evidenceOrigin = 'WINDOWS_HOST'
  automaticVerdict = 'NOT_PROVIDED'
  capturedUtc = (Get-Date).ToUniversalTime().ToString('o')
  software = [ordered]@{
    desktopVersion = $desktopVersion
    storePackagePresent = [bool]$storePresent
    uniqueRegisteredStartApp = [bool]$uniqueRegisteredApp
  }
  machine = [ordered]@{
    interactiveSession = ($session -ne 0)
    launchRequests = $LaunchRequests
    processEvidenceClass = 'NAME_MATCH_CANDIDATE_ONLY'
    candidateProcesses = @($candidates)
  }
  operator = [ordered]@{
    attribution = 'OPERATOR_DECLARED_NOT_MACHINE_VERIFIED'
    distinctAccounts = $DistinctAccounts
    bothWorldsPlayable = $BothWorldsPlayable
    independentInput = $IndependentInput
    accountCrossingObserved = $AccountCrossingObserved
    multiplayerJoined = $MultiplayerJoined
    reopenAndRebootPersistent = $ReopenAndRebootPersistent
    observedMinutes = $ObservedMinutes
    evidenceReference = $reference
  }
}
$filename = 'native-feasibility-' + (Get-Date -Format 'yyyyMMdd-HHmmss') +
  '-' + [guid]::NewGuid().ToString('N').Substring(0, 8) + '.json'
$reportPath = Join-Path $OutputDirectory $filename
$json = ConvertTo-Json -InputObject $report -Depth 8
[System.IO.File]::WriteAllText($reportPath, $json, [System.Text.UTF8Encoding]::new($false))
[pscustomobject]@{
  reportPath = $reportPath
  stage = $Stage
  candidateProcessCount = $candidates.Count
  candidateWindowCount = @($candidates | Where-Object { $null -ne $_.windowHandle }).Count
  interpretation = 'PROCESS_CANDIDATES_ONLY_NO_AUTOMATIC_GAMEPLAY_OR_ACCOUNT_PROOF'
} | ConvertTo-Json -Compress
