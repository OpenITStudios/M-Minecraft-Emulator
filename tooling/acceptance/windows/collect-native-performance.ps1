[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)]
  [ValidateRange(1, 2147483647)]
  [int]$NativePid,
  [Parameter(Mandatory = $true)]
  [ValidateRange(1, 2147483647)]
  [int]$IsolatedPid,
  [ValidateRange(30, 7200)]
  [int]$DurationSeconds = 3600,
  [ValidateRange(1, 60)]
  [int]$SampleIntervalSeconds = 5,
  [switch]$GpuSnapshot,
  [string]$OutputDirectory = '',
  [ValidateRange(0, 2147483647)]
  [int]$Client03Pid = 0,
  [ValidateRange(0, 2147483647)]
  [int]$Client04Pid = 0
)

$ErrorActionPreference = 'Stop'
# Diagnostic-only sampler. It never starts/stops Minecraft, changes Sandboxie,
# reads sign-in/profile/world content or infers gameplay from PID existence.
if ($NativePid -eq $IsolatedPid) {
  throw 'Native and isolated process IDs must be distinct.'
}
$fourClients = $Client03Pid -ne 0 -or $Client04Pid -ne 0
if ($fourClients) {
  if ($Client03Pid -eq 0 -or $Client04Pid -eq 0) {
    throw 'Four-client sampling requires both Client03Pid and Client04Pid.'
  }
  if (@(@($NativePid, $IsolatedPid, $Client03Pid, $Client04Pid) | Select-Object -Unique).Count -ne 4) {
    throw 'All four manually selected process IDs must be distinct.'
  }
  if ($GpuSnapshot) {
    throw 'Four-client CPU/RAM sampling does not support the two-client GPU snapshot. Collect GPU evidence separately.'
  }
}
if ($SampleIntervalSeconds -gt $DurationSeconds) {
  throw 'The sample interval cannot exceed the duration.'
}

function Read-Target {
  param([int]$ProcessId, [long]$ExpectedStartTicks = 0)
  try {
    $process = Get-Process -Id $ProcessId -ErrorAction Stop
    $started = [long]$process.StartTime.ToUniversalTime().Ticks
    if ($started -le 0 -or ($ExpectedStartTicks -ne 0 -and $started -ne $ExpectedStartTicks)) {
      return $null
    }
    $cpu = [double]$process.TotalProcessorTime.TotalSeconds
    $workingSet = [double]$process.WorkingSet64 / 1MB
    if ([double]::IsNaN($cpu) -or [double]::IsInfinity($cpu) -or
        [double]::IsNaN($workingSet) -or [double]::IsInfinity($workingSet)) {
      return $null
    }
    return [pscustomobject]@{
      startTicks = $started
      cpuSeconds = $cpu
      workingSetMb = $workingSet
    }
  } catch {
    # Missing/inaccessible/restarted process is UNKNOWN, not zero CPU/RAM.
    return $null
  }
}

function Process-Sample {
  param($Previous, $Current, [double]$ElapsedSeconds, [int]$LogicalCpus)
  if ($null -eq $Current -or $null -eq $Previous -or $ElapsedSeconds -le 0) {
    return [ordered]@{ status = 'UNKNOWN'; cpuPercentOfHost = $null; workingSetMb = $null }
  }
  $cpuDelta = $Current.cpuSeconds - $Previous.cpuSeconds
  if ($cpuDelta -lt 0) {
    return [ordered]@{ status = 'UNKNOWN'; cpuPercentOfHost = $null; workingSetMb = $null }
  }
  # Normalize process CPU time over all host logical processors. This is not FPS.
  $cpuPercent = [math]::Round(100.0 * $cpuDelta / ($ElapsedSeconds * $LogicalCpus), 2)
  return [ordered]@{
    status = 'OBSERVED'
    cpuPercentOfHost = $cpuPercent
    workingSetMb = [math]::Round($Current.workingSetMb, 2)
  }
}

# Optional GPU counters are hints, not physical VRAM or FPS proof.
# This costs no counter queries unless -GpuSnapshot was explicitly selected.
function Unknown-GpuHint {
  return [ordered]@{ status = 'UNKNOWN'; threeDEngineMaximumInstancePercent = $null;
    dedicatedMemoryMaximumInstanceMb = $null }
}
function Read-GpuSnapshot {
  param([int]$FirstPid, [int]$SecondPid,
        [long]$FirstStartTicks, [long]$SecondStartTicks)
  $result = [ordered]@{
    evidenceClass = 'WINDOWS_GPU_COUNTER_HINT_NOT_FRAME_PROOF'
    native = (Unknown-GpuHint)
    isolated = (Unknown-GpuHint)
  }
  try {
    # Only one counter query for the entire manual run; two samples are
    # necessary for rate-based utilization values.
    $sets = @(Get-Counter -Counter @(
      '\GPU Engine(*)\Utilization Percentage',
      '\GPU Process Memory(*)\Dedicated Usage'
    ) -SampleInterval 1 -MaxSamples 2 -ErrorAction Stop)
    if ($sets.Count -ne 2) { return $result }
    $observed = @{}
    foreach ($candidate in @($FirstPid, $SecondPid)) {
      $observed[[string]$candidate] = [ordered]@{ engine = @(); dedicated = @() }
    }
    foreach ($counter in @($sets[-1].CounterSamples)) {
      $path = [string]$counter.Path
      if ($path -notmatch '(?i)\(pid_([0-9]+)_') { continue }
      [long]$counterPid = 0
      if (-not [long]::TryParse($Matches[1], [ref]$counterPid)) { continue }
      $key = [string]$counterPid
      if (-not $observed.ContainsKey($key)) { continue }
      $value = [double]$counter.CookedValue
      if ([double]::IsNaN($value) -or [double]::IsInfinity($value) -or $value -lt 0) {
        continue
      }
      if ($path -match '(?i)\\gpu engine\(' -and
          $path -match '(?i)engtype_3d' -and $value -le 100) {
        $observed[$key].engine += $value
      } elseif ($path -match '(?i)\\gpu process memory\(' -and
                $path -match '(?i)\\dedicated usage$') {
        $observed[$key].dedicated += ($value / 1MB)
      }
    }
    foreach ($item in @(
      @{ role='native'; pid=$FirstPid; started=$FirstStartTicks },
      @{ role='isolated'; pid=$SecondPid; started=$SecondStartTicks }
    )) {
      # Refuse counter values if PID was recycled during the optional query.
      if ($null -eq (Read-Target -ProcessId $item.pid -ExpectedStartTicks $item.started)) {
        continue
      }
      $counterData = $observed[[string]$item.pid]
      if ($counterData.engine.Count -eq 0 -or $counterData.dedicated.Count -eq 0) {
        continue
      }
      $result[$item.role] = [ordered]@{
        status = 'OBSERVED'
        # Not total GPU use: largest 3D engine instance in the last sample.
        threeDEngineMaximumInstancePercent = [math]::Round(
          [double]($counterData.engine | Measure-Object -Maximum).Maximum, 2)
        # Not actual aggregate VRAM: this Windows counter can be stale.
        dedicatedMemoryMaximumInstanceMb = [math]::Round(
          [double]($counterData.dedicated | Measure-Object -Maximum).Maximum, 2)
      }
    }
  } catch {
    # Missing/localized PDH counters and GPU driver limitations are UNKNOWN.
    $result.native = Unknown-GpuHint
    $result.isolated = Unknown-GpuHint
  }
  return $result
}

$baselineNative = Read-Target -ProcessId $NativePid
$baselineIsolated = Read-Target -ProcessId $IsolatedPid
if ($null -eq $baselineNative -or $null -eq $baselineIsolated) {
  throw 'Both manually selected processes must be accessible before measurement. No report produced.'
}
$extraTargets = @()
if ($fourClients) {
  foreach ($entry in @(@{ key='client03'; processId=$Client03Pid },
                       @{ key='client04'; processId=$Client04Pid })) {
    $baseline = Read-Target -ProcessId $entry.processId
    if ($null -eq $baseline) { throw 'All four manually selected processes must be accessible before measurement. No report produced.' }
    $extraTargets += @{ key=$entry.key; processId=$entry.processId;
      baseline=$baseline; previous=$baseline; lost=$false }
  }
}
$logicalCpus = [Environment]::ProcessorCount
if ($logicalCpus -lt 1) { throw 'Host logical CPU count is invalid.' }
if (-not $OutputDirectory) {
  $OutputDirectory = Join-Path $env:TEMP 'MinecraftInstanceManager\Performance'
}
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null

$beginUtc = [DateTime]::UtcNow
$timer = [System.Diagnostics.Stopwatch]::StartNew()
$previousSecond = 0.0
$previousNative = $baselineNative
$previousIsolated = $baselineIsolated
$nativeLost = $false
$isolatedLost = $false
$samples = [System.Collections.Generic.List[object]]::new()

while ($timer.Elapsed.TotalSeconds -lt $DurationSeconds) {
  $remainingMs = [math]::Max(1, [math]::Ceiling(1000 * ($DurationSeconds - $timer.Elapsed.TotalSeconds)))
  $pauseMs = [int][math]::Min(1000 * $SampleIntervalSeconds, $remainingMs)
  Start-Sleep -Milliseconds $pauseMs
  $elapsed = [double]$timer.Elapsed.TotalSeconds
  $delta = $elapsed - $previousSecond

  $nativeCurrent = if ($nativeLost) { $null } else {
    Read-Target -ProcessId $NativePid -ExpectedStartTicks $baselineNative.startTicks
  }
  $isolatedCurrent = if ($isolatedLost) { $null } else {
    Read-Target -ProcessId $IsolatedPid -ExpectedStartTicks $baselineIsolated.startTicks
  }
  # Once a candidate disappears or becomes inaccessible, do not silently
  # follow a new process with the same PID. A new capture must be deliberate.
  if ($null -eq $nativeCurrent) { $nativeLost = $true }
  if ($null -eq $isolatedCurrent) { $isolatedLost = $true }

  $sample = [ordered]@{
    elapsedSeconds = [math]::Round($elapsed, 2)
    native = (Process-Sample -Previous $previousNative -Current $nativeCurrent -ElapsedSeconds $delta -LogicalCpus $logicalCpus)
    isolated = (Process-Sample -Previous $previousIsolated -Current $isolatedCurrent -ElapsedSeconds $delta -LogicalCpus $logicalCpus)
  }
  foreach ($entry in $extraTargets) {
    $current = if ($entry.lost) { $null } else {
      Read-Target -ProcessId $entry.processId -ExpectedStartTicks $entry.baseline.startTicks
    }
    if ($null -eq $current) { $entry.lost = $true }
    $sample[$entry.key] = Process-Sample -Previous $entry.previous -Current $current -ElapsedSeconds $delta -LogicalCpus $logicalCpus
    $entry.previous = $current
  }
  $samples.Add($sample)
  $previousNative = $nativeCurrent
  $previousIsolated = $isolatedCurrent
  $previousSecond = $elapsed
}

# Capture duration excludes the optional diagnostic query.
$durationWithoutGpu = [math]::Round($timer.Elapsed.TotalSeconds, 2)
$gpuDiagnostic = $null
if ($GpuSnapshot) {
  if ($nativeLost -or $isolatedLost) {
    # Identity was lost during the timed CPU capture. Avoid the extra GPU
    # counter query: a PID might now belong to an unrelated process.
    $gpuDiagnostic = [ordered]@{
      evidenceClass = 'WINDOWS_GPU_COUNTER_HINT_NOT_FRAME_PROOF'
      native = (Unknown-GpuHint)
      isolated = (Unknown-GpuHint)
    }
  } else {
    $gpuDiagnostic = Read-GpuSnapshot -FirstPid $NativePid -SecondPid $IsolatedPid -FirstStartTicks $baselineNative.startTicks -SecondStartTicks $baselineIsolated.startTicks
  }
}
# Hints are not account, FPS, frametime or actual VRAM proof.
$report = [ordered]@{
  schema = 1
  scope = 'NATIVE_SANDBOXIE_PROCESS_PERFORMANCE'
  evidenceOrigin = 'WINDOWS_HOST'
  attribution = 'OPERATOR_SELECTED_UNVERIFIED_PIDS'
  automaticVerdict = 'NOT_PROVIDED'
  capturedAtUtc = $beginUtc.ToString('o')
  durationRequestedSeconds = $DurationSeconds
  durationObservedSeconds = $durationWithoutGpu
  sampleIntervalSeconds = $SampleIntervalSeconds
  hostLogicalCpus = $logicalCpus
  sampleCount = $samples.Count
  nativeProcessIdentityLost = $nativeLost
  isolatedProcessIdentityLost = $isolatedLost
  samples = @($samples.ToArray())
  missingEvidence = @('ACCOUNT_ISOLATION', 'GAMEPLAY_RENDERING', 'FRAME_TIMES', 'GPU_VRAM', 'MULTIPLAYER', 'INPUT_RESPONSIVENESS')
}
if ($GpuSnapshot) {
  $report['schema'] = 2
  $report['gpuDiagnostic'] = $gpuDiagnostic
}
if ($fourClients) {
  $report['schema'] = 3
  $report['scope'] = 'FOUR_CLIENT_PROCESS_PERFORMANCE'
  $report['client03ProcessIdentityLost'] = [bool]$extraTargets[0].lost
  $report['client04ProcessIdentityLost'] = [bool]$extraTargets[1].lost
}
$stamp = $beginUtc.ToString('yyyyMMddTHHmmssfffffffZ')
$file = Join-Path $OutputDirectory ('native-process-performance-' + $stamp + '.json')
# Windows PowerShell 5.1's Set-Content -Encoding UTF8 emits a BOM
# that strict Node JSON.parse rejects. Commit only a COMPLETE UTF8-no-BOM
# file: create an exclusive temporary file beside the final destination,
# flush/close it, then move it into place WITHOUT replacing existing reports.
# The same-directory move prevents an interrupted write leaving a fake final.
$json = $report | ConvertTo-Json -Depth 7
$utf8NoBom = [System.Text.UTF8Encoding]::new($false)
$bytes = $utf8NoBom.GetBytes($json + [Environment]::NewLine)
$temporary = Join-Path $OutputDirectory ('.mivubi-native-performance-' +
  [Guid]::NewGuid().ToString('N') + '.tmp')
$stream = $null
try {
  $stream = [System.IO.File]::Open($temporary, [System.IO.FileMode]::CreateNew,
    [System.IO.FileAccess]::Write, [System.IO.FileShare]::None)
  $stream.Write($bytes, 0, $bytes.Length)
  $stream.Flush()
  $stream.Dispose()
  $stream = $null
  # File.Move (two arguments) refuses an existing destination; do not use
  # Move-Item -Force or File.Copy here. The original report is never replaced.
  [System.IO.File]::Move($temporary, $file)
} finally {
  if ($null -ne $stream) { $stream.Dispose() }
  # This invocation owns ONLY the GUID-named staging file. A failed move
  # never removes or mutates an existing final report.
  if ([System.IO.File]::Exists($temporary)) {
    try { [System.IO.File]::Delete($temporary) } catch { }
  }
}
[ordered]@{
  report = $file
  sampleCount = $samples.Count
  automaticVerdict = 'NOT_PROVIDED'
  interpretation = 'PROCESS_TELEMETRY_ONLY'
} | ConvertTo-Json -Depth 3
