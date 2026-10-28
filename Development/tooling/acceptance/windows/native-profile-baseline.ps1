# Current Minecraft Education native 0-5 profile RAM baseline.
# Dot-sourced by collect-native-performance.ps1 after its existing read-only
# process, private memory, whole-system memory and atomic-report helpers.
# No launching, game manipulation, resident trimming, credentials or new worker.
function Invoke-NativeProfileBaseline {
  param(
    [int[]]$ProfilePids,
    [int[]]$ProfileSlots,
    [int]$ManagerPid,
    [int]$DurationSeconds,
    [int]$SampleIntervalSeconds,
    [string]$OutputDirectory,
    [switch]$DirectChildren
  )

  $selected = [ordered]@{}
  for ($index = 0; $index -lt $ProfilePids.Count; $index++) {
    $slotNumber = if ($ProfileSlots.Count -eq 0) { $index + 1 } else { $ProfileSlots[$index] }
    $slot = '{0:D2}' -f $slotNumber
    $processId = [int]$ProfilePids[$index]
    $baseline = Read-Target -ProcessId $processId
    if ($null -eq $baseline) {
      throw "Minecraft - $slot is not accessible. No RAM report was created."
    }
    $selected[$slot] = @{ processId=$processId; baseline=$baseline;
      previous=$baseline; lost=$false }
  }
  $manager = $null
  if ($ManagerPid -ne 0) {
    $baseline = Read-Target -ProcessId $ManagerPid
    if ($null -eq $baseline) {
      throw 'Selected manager is not accessible. No RAM report was created.'
    }
    $manager = @{ processId=$ManagerPid; baseline=$baseline;
      previous=$baseline; lost=$false }
  }
  $logicalCpus = [Environment]::ProcessorCount
  if ($logicalCpus -lt 1) { throw 'Host logical CPU count is invalid.' }

  if (-not $OutputDirectory) {
    $OutputDirectory = Join-Path $env:TEMP 'MinecraftInstanceManager\Performance'
  }
  New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
  $OutputDirectory = (Get-Item -LiteralPath $OutputDirectory -ErrorAction Stop).FullName

  $begin = [DateTime]::UtcNow
  $timer = [Diagnostics.Stopwatch]::StartNew()
  $lastSampleSeconds = 0.0
  $rows = [Collections.Generic.List[object]]::new()

  while ($timer.Elapsed.TotalSeconds -lt $DurationSeconds) {
    $remainingMs = [math]::Max(1, [math]::Ceiling(
      1000 * ($DurationSeconds - $timer.Elapsed.TotalSeconds)))
    $pauseMs = [int][math]::Min(1000 * $SampleIntervalSeconds, $remainingMs)
    Start-Sleep -Milliseconds $pauseMs
    $now = [double]$timer.Elapsed.TotalSeconds
    $delta = $now - $lastSampleSeconds
    $row = [ordered]@{
      elapsedSeconds = [math]::Round($now, 2)
      clients = [ordered]@{}
      manager = $null
      system = $null
      directChildren = $null
      queryDurationMs = 0.0
    }
    foreach ($slot in $selected.Keys) {
      $target = $selected[$slot]
      $current = if ($target.lost) { $null } else {
        Read-Target -ProcessId $target.processId -ExpectedStartTicks $target.baseline.startTicks
      }
      $process = Process-Sample -Previous $target.previous -Current $current -ElapsedSeconds $delta -LogicalCpus $logicalCpus
      if ($process.status -eq 'UNKNOWN') { $target.lost = $true }
      $row.clients[$slot] = [ordered]@{
        process = $process
        memory = (Read-ProcessMemory -ProcessId $target.processId -ExpectedStartTicks $target.baseline.startTicks -IdentityLost $target.lost)
      }
      $target.previous = $current
    }
    if ($null -ne $manager) {
      $current = if ($manager.lost) { $null } else {
        Read-Target -ProcessId $manager.processId -ExpectedStartTicks $manager.baseline.startTicks
      }
      $process = Process-Sample -Previous $manager.previous -Current $current -ElapsedSeconds $delta -LogicalCpus $logicalCpus
      if ($process.status -eq 'UNKNOWN') { $manager.lost = $true }
      $row.manager = [ordered]@{
        process = $process
        memory = (Read-ProcessMemory -ProcessId $manager.processId -ExpectedStartTicks $manager.baseline.startTicks -IdentityLost $manager.lost)
      }
      $manager.previous = $current
    }
    $row.system = Read-SystemMemory
    if ($DirectChildren) {
      $roots = @()
      foreach ($slot in $selected.Keys) {
        $target = $selected[$slot]
        $roots += @{ key=$slot; processId=$target.processId;
          started=$target.baseline.startTicks; lost=$target.lost }
      }
      if ($null -ne $manager) {
        $roots += @{ key='manager'; processId=$manager.processId;
          started=$manager.baseline.startTicks; lost=$manager.lost }
      }
      $excluded = @($PID) + @($ProfilePids)
      if ($null -ne $manager) { $excluded += $manager.processId }
      $row.directChildren = Read-DirectChildren -Roots $roots -Excluded $excluded
    }
    $row.queryDurationMs = [math]::Round(1000 * ($timer.Elapsed.TotalSeconds - $now), 3)
    $rows.Add($row)
    $lastSampleSeconds = $now
  }
  $duration = [double]$timer.Elapsed.TotalSeconds
  $timer.Stop()

  $slots = @()
  $lost = [ordered]@{}
  foreach ($slot in $selected.Keys) {
    $slots += [string]$slot
    $lost[$slot] = [bool]$selected[$slot].lost
  }
  $report = [ordered]@{
    schema = 7
    scope = 'NATIVE_PROFILE_MEMORY_BASELINE'
    evidenceOrigin = 'WINDOWS_HOST'
    attribution = 'OPERATOR_SELECTED_UNVERIFIED_PIDS'
    automaticVerdict = 'NOT_PROVIDED'
    capturedAtUtc = $begin.ToString('o')
    durationRequestedSeconds = $DurationSeconds
    durationObservedSeconds = [math]::Round($duration, 3)
    sampleIntervalSeconds = $SampleIntervalSeconds
    hostLogicalCpus = $logicalCpus
    sampleCount = $rows.Count
    selectedSlots = $slots
    profileIdentityLost = $lost
    managerSelected = ($null -ne $manager)
    managerProcessIdentityLost = $(if ($null -ne $manager) { [bool]$manager.lost } else { $null })
    directChildrenRequested = [bool]$DirectChildren
    samples = @($rows.ToArray())
    missingEvidence = @('ACCOUNT_ISOLATION', 'GAMEPLAY_RENDERING', 'FRAME_TIMES',
      'GPU_VRAM', 'MULTIPLAYER', 'INPUT_RESPONSIVENESS')
  }

  # The original writer owns an exclusive temporary file and never overwrites
  # an existing report. Preserve that behavior for baseline A/B evidence.
  $json = $report | ConvertTo-Json -Depth 12
  $bytes = [Text.UTF8Encoding]::new($false).GetBytes($json + [Environment]::NewLine)
  $filename = 'native-memory-baseline-' + $begin.ToString('yyyyMMddTHHmmssfffffffZ') + '.json'
  $file = Write-ReportArtifact -Directory $OutputDirectory -Name $filename -Bytes $bytes
  [ordered]@{
    report = $file
    selectedProfileCount = $selected.Count
    sampleCount = $rows.Count
    automaticVerdict = 'NOT_PROVIDED'
    interpretation = 'NATIVE_PROFILE_MEMORY_OBSERVATION_ONLY'
  } | ConvertTo-Json -Depth 3
}
