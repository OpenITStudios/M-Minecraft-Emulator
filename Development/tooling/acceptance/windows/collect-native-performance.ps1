[CmdletBinding()]
param(
  [ValidateRange(0, 2147483647)]
  [int]$NativePid = 0,
  [ValidateRange(0, 2147483647)]
  [int]$IsolatedPid = 0,
  [ValidateRange(30, 7200)]
  [int]$DurationSeconds = 3600,
  [ValidateRange(1, 60)]
  [int]$SampleIntervalSeconds = 5,
  [switch]$GpuSnapshot,
  [switch]$MemoryComposition,
  [string]$OutputDirectory = '',
  [ValidateRange(0, 2147483647)]
  [int]$Client03Pid = 0,
  [ValidateRange(0, 2147483647)]
  [int]$Client04Pid = 0,
  [ValidateRange(0, 2147483647)]
  [int]$Client05Pid = 0,
  [ValidateRange(0, 2147483647)]
  [int]$ManagerPid = 0,
  [switch]$DirectChildren,
  # Current native profile-only baseline. Legacy PID flags remain supported separately.
  [switch]$NativeProfileBaseline,
  [int[]]$ProfilePids = @(),
  [int[]]$ProfileSlots = @()
)

$ErrorActionPreference = 'Stop'
# Diagnostic-only sampler. It never starts/stops Minecraft, changes Sandboxie,
# reads sign-in/profile/world content or infers gameplay from PID existence.
if ($NativeProfileBaseline) {
  # A native observation never launches a game. Slot assignment is caller supplied.
  if ($NativePid -ne 0 -or $IsolatedPid -ne 0 -or
      $Client03Pid -ne 0 -or $Client04Pid -ne 0 -or $Client05Pid -ne 0) {
    throw 'Use -ProfilePids for native profiles, not historical Native/Isolated flags.'
  }
  if (-not $MemoryComposition -or $GpuSnapshot) {
    throw 'Native profile baseline requires -MemoryComposition and does not accept -GpuSnapshot.'
  }
  if ($ProfilePids.Count -gt 5 -or @($ProfilePids | Where-Object { $_ -lt 1 }).Count -gt 0) {
    throw 'Native baseline accepts 0 to 5 positive existing game process IDs.'
  }
  if (@($ProfilePids | Select-Object -Unique).Count -ne $ProfilePids.Count) {
    throw 'Each native profile must have a distinct process ID.'
  }
  if ($ProfileSlots.Count -ne 0) {
    if ($ProfileSlots.Count -ne $ProfilePids.Count) {
      throw 'Each explicit slot requires exactly one matching existing process ID.'
    }
    for ($index = 0; $index -lt $ProfileSlots.Count; $index++) {
      if ($ProfileSlots[$index] -lt 1 -or $ProfileSlots[$index] -gt 5 -or
          ($index -gt 0 -and $ProfileSlots[$index] -le $ProfileSlots[$index - 1])) {
        throw 'Explicit native profile slots must be unique, 1..5, and strictly increasing.'
      }
    }
  }
  if ($ManagerPid -ne 0 -and $ManagerPid -in $ProfilePids) {
    throw 'The selected manager must not be one of the game processes.'
  }
  if ($DirectChildren -and $ProfilePids.Count -eq 0 -and $ManagerPid -eq 0) {
    throw 'Direct-child attribution needs at least one selected process.'
  }
} else {
  if ($ProfilePids.Count -gt 0 -or $ProfileSlots.Count -gt 0) {
    throw 'Use -NativeProfileBaseline -MemoryComposition with -ProfilePids/-ProfileSlots.'
  }
  if ($NativePid -eq 0 -or $IsolatedPid -eq 0) {
    throw 'Historical capture requires both -NativePid and -IsolatedPid.'
  }
if ($NativePid -eq $IsolatedPid) {
  throw 'Native and isolated process IDs must be distinct.'
}
$fiveClients = $Client05Pid -ne 0
$fourClients = $Client03Pid -ne 0 -or $Client04Pid -ne 0 -or $fiveClients
if ($fourClients) {
  if ($Client03Pid -eq 0 -or $Client04Pid -eq 0) {
    throw 'Multi-client sampling requires both Client03Pid and Client04Pid.'
  }
  $selectedPids = @($NativePid, $IsolatedPid, $Client03Pid, $Client04Pid)
  if ($fiveClients) { $selectedPids += $Client05Pid }
  if (@($selectedPids | Select-Object -Unique).Count -ne $selectedPids.Count) {
    throw 'All manually selected process IDs must be distinct.'
  }
  if ($GpuSnapshot -and -not $fiveClients) {
    throw 'Four-client CPU/RAM sampling does not support the GPU snapshot. Use a two- or five-client capture.'
  }
}
if ($SampleIntervalSeconds -gt $DurationSeconds) {
  throw 'The sample interval cannot exceed the duration.'
}
if ($MemoryComposition -and $fourClients) {
  throw 'Memory composition currently supports the two-client comparison only.'
}
if (($ManagerPid -ne 0 -or $DirectChildren) -and -not $MemoryComposition) {
  throw 'Manager/direct-child diagnostics require -MemoryComposition.'
}
if ($ManagerPid -ne 0 -and $ManagerPid -in @($NativePid, $IsolatedPid)) {
  throw 'The selected manager must be distinct from both selected games.'
}

}

function Read-Target {
  param([int]$ProcessId, [long]$ExpectedStartTicks = 0)
  $process = $null
  try {
    $process = Get-Process -Id $ProcessId -ErrorAction Stop
    if ($process.HasExited) { return $null }
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
  } finally {
    if ($null -ne $process) { $process.Dispose() }
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
  param([object[]]$Targets)
  $result = [ordered]@{
    evidenceClass = 'WINDOWS_GPU_COUNTER_HINT_NOT_FRAME_PROOF'
  }
  foreach ($item in $Targets) { $result[$item.key] = Unknown-GpuHint }
  try {
    # Only one counter query for the entire manual run; two samples are
    # necessary for rate-based utilization values.
    $sets = @(Get-Counter -Counter @(
      '\GPU Engine(*)\Utilization Percentage',
      '\GPU Process Memory(*)\Dedicated Usage'
    ) -SampleInterval 1 -MaxSamples 2 -ErrorAction Stop)
    if ($sets.Count -ne 2) { return $result }
    $observed = @{}
    foreach ($item in $Targets) {
      $observed[[string]$item.pid] = [ordered]@{ engine = @(); dedicated = @() }
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
    foreach ($item in $Targets) {
      # Refuse counter values if PID was recycled during the optional query.
      if ($null -eq (Read-Target -ProcessId $item.pid -ExpectedStartTicks $item.started)) {
        continue
      }
      $counterData = $observed[[string]$item.pid]
      if ($counterData.engine.Count -eq 0 -or $counterData.dedicated.Count -eq 0) {
        continue
      }
      $result[$item.key] = [ordered]@{
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
    foreach ($item in $Targets) { $result[$item.key] = Unknown-GpuHint }
  }
  return $result
}

# Loaded only for the explicit composition capture; no VM_READ or target mutation.
$memoryApiAvailable = $false
if ($MemoryComposition) {
  try {
    if (-not ('Mivubi.PerformanceMemory' -as [type])) {
      Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
namespace Mivubi {
  public static class PerformanceMemory {
    [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)]
    struct ProcessEntry {
      public uint Size, Usage, ProcessId;
      public UIntPtr DefaultHeapId;
      public uint ModuleId, Threads, ParentId;
      public int Priority;
      public uint Flags;
      [MarshalAs(UnmanagedType.ByValTStr, SizeConst=260)] public string ImageName;
    }
    [DllImport("kernel32.dll", SetLastError=true)]
    static extern IntPtr CreateToolhelp32Snapshot(uint flags, uint pid);
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    static extern bool Process32FirstW(IntPtr snapshot, ref ProcessEntry entry);
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    static extern bool Process32NextW(IntPtr snapshot, ref ProcessEntry entry);
    public sealed class ChildEntry { public int ProcessId, ParentId; }
    public sealed class ChildSnapshot { public ChildEntry[] Entries; }
    public static ChildSnapshot Children(int[] parents) {
      IntPtr snapshot = CreateToolhelp32Snapshot(2, 0);
      if (snapshot == new IntPtr(-1)) return null;
      try {
        var entry = new ProcessEntry();
        entry.Size = (uint)Marshal.SizeOf(typeof(ProcessEntry));
        var entries = new List<ChildEntry>();
        bool more = Process32FirstW(snapshot, ref entry);
        while (more) {
          if (entry.ProcessId <= int.MaxValue && Array.IndexOf(parents, (int)entry.ParentId) >= 0)
            entries.Add(new ChildEntry { ProcessId=(int)entry.ProcessId, ParentId=(int)entry.ParentId });
          more = Process32NextW(snapshot, ref entry);
        }
        if (Marshal.GetLastWin32Error() != 18) return null;
        return new ChildSnapshot { Entries=entries.ToArray() };
      } finally { CloseHandle(snapshot); }
    }
    [StructLayout(LayoutKind.Sequential)]
    struct Counters {
      public uint cb, PageFaultCount;
      public UIntPtr PeakWorkingSetSize, WorkingSetSize, QuotaPeakPagedPoolUsage,
        QuotaPagedPoolUsage, QuotaPeakNonPagedPoolUsage, QuotaNonPagedPoolUsage,
        PagefileUsage, PeakPagefileUsage, PrivateUsage;
    }
    [StructLayout(LayoutKind.Sequential)]
    struct Counters2 {
      public Counters Base;
      public UIntPtr PrivateWorkingSetSize;
      public ulong SharedCommitUsage;
    }
    [StructLayout(LayoutKind.Sequential)]
    struct PerformanceInfo {
      public uint cb;
      public UIntPtr CommitTotal, CommitLimit, CommitPeak, PhysicalTotal,
        PhysicalAvailable, SystemCache, KernelTotal, KernelPaged, KernelNonpaged,
        PageSize;
      public uint HandleCount, ProcessCount, ThreadCount;
    }
    [DllImport("kernel32.dll", SetLastError=true)]
    static extern IntPtr OpenProcess(uint access, bool inherit, int pid);
    [DllImport("kernel32.dll")]
    static extern bool CloseHandle(IntPtr handle);
    [DllImport("kernel32.dll", SetLastError=true)]
    static extern bool GetProcessTimes(IntPtr handle, out long creation,
      out long exit, out long kernel, out long user);
    [DllImport("psapi.dll", EntryPoint="GetProcessMemoryInfo", SetLastError=true)]
    static extern bool GetMemory(IntPtr handle, ref Counters data, uint size);
    [DllImport("psapi.dll", EntryPoint="GetProcessMemoryInfo", SetLastError=true)]
    static extern bool GetMemory2(IntPtr handle, ref Counters2 data, uint size);
    [DllImport("psapi.dll", SetLastError=true)]
    static extern bool GetPerformanceInfo(ref PerformanceInfo data, uint size);
    public sealed class ProcessValues {
      public double? PrivateCommitMb, PrivateWorkingSetMb;
    }
    static bool SameProcess(IntPtr handle, long expectedTicks) {
      long birth, exit, kernel, user;
      return GetProcessTimes(handle, out birth, out exit, out kernel, out user)
        && DateTime.FromFileTimeUtc(birth).Ticks == expectedTicks && exit == 0;
    }
    public static ProcessValues Read(int pid, long expectedTicks) {
      IntPtr handle = OpenProcess(0x1000, false, pid);
      if (handle == IntPtr.Zero) return null;
      try {
        if (!SameProcess(handle, expectedTicks)) return null;
        var result = new ProcessValues();
        var extended = new Counters2();
        extended.Base.cb = (uint)Marshal.SizeOf(typeof(Counters2));
        // An older implementation must not turn an untouched field into zero.
        ulong sentinel = UIntPtr.Size == 8 ? ulong.MaxValue : uint.MaxValue;
        extended.PrivateWorkingSetSize = new UIntPtr(sentinel);
        if (GetMemory2(handle, ref extended, extended.Base.cb)) {
          result.PrivateCommitMb = extended.Base.PrivateUsage.ToUInt64() / 1048576.0;
          if (extended.PrivateWorkingSetSize.ToUInt64() != sentinel)
            result.PrivateWorkingSetMb = extended.PrivateWorkingSetSize.ToUInt64() / 1048576.0;
        } else {
          var basic = new Counters();
          basic.cb = (uint)Marshal.SizeOf(typeof(Counters));
          if (GetMemory(handle, ref basic, basic.cb))
            result.PrivateCommitMb = basic.PrivateUsage.ToUInt64() / 1048576.0;
        }
        return SameProcess(handle, expectedTicks) ? result : null;
      } finally { CloseHandle(handle); }
    }
    public static double[] ReadSystem() {
      var data = new PerformanceInfo();
      data.cb = (uint)Marshal.SizeOf(typeof(PerformanceInfo));
      if (!GetPerformanceInfo(ref data, data.cb)) return null;
      double pageMb = data.PageSize.ToUInt64() / 1048576.0;
      return new double[] { data.PhysicalTotal.ToUInt64() * pageMb,
        data.PhysicalAvailable.ToUInt64() * pageMb,
        data.CommitTotal.ToUInt64() * pageMb, data.CommitLimit.ToUInt64() * pageMb };
    }
  }
}
'@ -ErrorAction Stop
    }
    $memoryApiAvailable = $true
  } catch {
    # Extended metrics fail independently of the established CPU/working-set route.
  }
}

function Memory-Metric {
  param($Value, [string]$Reason)
  if ($null -eq $Value -or [double]::IsNaN([double]$Value) -or
      [double]::IsInfinity([double]$Value) -or [double]$Value -lt 0) {
    return [ordered]@{ status='UNKNOWN'; valueMb=$null; reason=$Reason }
  }
  return [ordered]@{ status='OBSERVED'; valueMb=[math]::Round([double]$Value, 3); reason=$null }
}
function Read-ProcessMemory {
  param([int]$ProcessId, [long]$ExpectedStartTicks, [bool]$IdentityLost)
  $values = $null
  $reason = 'API_UNAVAILABLE'
  if ($IdentityLost) { $reason = 'IDENTITY_UNAVAILABLE' }
  elseif ($memoryApiAvailable) {
    $reason = 'QUERY_FAILED'
    try { $values = [Mivubi.PerformanceMemory]::Read($ProcessId, $ExpectedStartTicks) } catch { }
  }
  return [ordered]@{
    privateCommitMb = (Memory-Metric -Value $values.PrivateCommitMb -Reason $reason)
    privateWorkingSetMb = (Memory-Metric -Value $values.PrivateWorkingSetMb -Reason $(if ($null -ne $values) { 'EXTENDED_COUNTER_UNAVAILABLE' } else { $reason }))
  }
}
function Read-SystemMemory {
  $values = $null
  $reason = 'API_UNAVAILABLE'
  if ($memoryApiAvailable) {
    $reason = 'QUERY_FAILED'
    try { $values = [Mivubi.PerformanceMemory]::ReadSystem() } catch { }
  }
  $result = [ordered]@{}
  $names = @('totalPhysicalMb', 'availablePhysicalMb', 'commitTotalMb', 'commitLimitMb')
  for ($i = 0; $i -lt $names.Count; $i++) {
    $value = if ($null -ne $values) { $values[$i] } else { $null }
    $result[$names[$i]] = Memory-Metric -Value $value -Reason $reason
  }
  return $result
}


function Read-ChildSnapshot {
  param([int[]]$Parents)
  if ($memoryApiAvailable) {
    try { return [Mivubi.PerformanceMemory]::Children($Parents) } catch { }
  }
  return $null
}
function Unknown-Children {
  param([string]$Reason, $Verified = $null, $Unresolved = $null)
  return [ordered]@{
    status='UNKNOWN'; reason=$Reason; verifiedCount=$Verified; unresolvedCount=$Unresolved
    privateCommitMb=(Memory-Metric -Value $null -Reason 'QUERY_FAILED')
    privateWorkingSetMb=(Memory-Metric -Value $null -Reason 'QUERY_FAILED')
  }
}
function Read-DirectChildren {
  param([object[]]$Roots, [int[]]$Excluded)
  $result = [ordered]@{}
  $before = @{}
  foreach ($root in $Roots) {
    $before[$root.key] = if ($root.lost) { $null } else {
      Read-Target -ProcessId $root.processId -ExpectedStartTicks $root.started
    }
  }
  $parents = @($Roots | ForEach-Object { $_.processId })
  $first = Read-ChildSnapshot -Parents $parents
  $pins = @{}
  if ($null -ne $first) {
    $candidates = @($first.Entries | Where-Object { $_.ProcessId -notin $Excluded })
    if ($candidates.Count -le 64) {
      foreach ($entry in $candidates) {
        $pins[[string]$entry.ProcessId] = Read-Target -ProcessId $entry.ProcessId
      }
    }
  }
  # Pin candidate birth BEFORE this relation snapshot, then recheck after stats.
  $second = Read-ChildSnapshot -Parents $parents
  foreach ($root in $Roots) {
    if ($null -eq $before[$root.key]) {
      $result[$root.key] = Unknown-Children -Reason 'ROOT_IDENTITY_UNAVAILABLE'
      continue
    }
    if ($null -eq $first -or $null -eq $second) {
      $result[$root.key] = Unknown-Children -Reason 'SNAPSHOT_UNAVAILABLE'
      continue
    }
    $allIds = @(@($first.Entries) + @($second.Entries) |
      Where-Object { $_.ParentId -eq $root.processId -and $_.ProcessId -notin $Excluded } |
      ForEach-Object { $_.ProcessId } | Select-Object -Unique)
    if ($candidates.Count -gt 64 -or $allIds.Count -gt 64) {
      $result[$root.key] = Unknown-Children -Reason 'CANDIDATE_LIMIT'
      continue
    }
    $verified = 0; $unresolved = 0
    $memories = @()
    foreach ($childId in $allIds) {
      $pin = $pins[[string]$childId]
      $relation = @($second.Entries | Where-Object {
        $_.ProcessId -eq $childId -and $_.ParentId -eq $root.processId
      })
      if ($null -eq $pin -or $pin.startTicks -lt $root.started -or $relation.Count -ne 1) {
        $unresolved++
        continue
      }
      $memory = Read-ProcessMemory -ProcessId $childId -ExpectedStartTicks $pin.startTicks -IdentityLost $false
      if ($null -eq (Read-Target -ProcessId $childId -ExpectedStartTicks $pin.startTicks)) {
        $unresolved++
        continue
      }
      $verified++
      $memories += $memory
    }
    if ($null -eq (Read-Target -ProcessId $root.processId -ExpectedStartTicks $root.started)) {
      $result[$root.key] = Unknown-Children -Reason 'ROOT_IDENTITY_UNAVAILABLE'
    } elseif ($unresolved -ne 0) {
      $result[$root.key] = Unknown-Children -Reason 'PARTIAL_RELATION' -Verified $verified -Unresolved $unresolved
    } else {
      $group = [ordered]@{ status='OBSERVED'; reason=$null; verifiedCount=$verified; unresolvedCount=0 }
      foreach ($field in @('privateCommitMb', 'privateWorkingSetMb')) {
        $sum = 0.0
        $missing = @($memories | Where-Object { $_[$field].status -ne 'OBSERVED' }).Count -ne 0
        if ($missing) { $group[$field] = Memory-Metric -Value $null -Reason 'QUERY_FAILED' }
        else {
          foreach ($memory in $memories) { $sum += $memory[$field].valueMb }
          $group[$field] = Memory-Metric -Value $sum -Reason 'QUERY_FAILED'
        }
      }
      $result[$root.key] = $group
    }
  }
  return $result
}

function Write-ReportArtifact {
  param([string]$Directory, [string]$Name, [byte[]]$bytes)
  $file = Join-Path $Directory $Name
  $temporary = Join-Path $Directory ('.mivubi-native-performance-' +
    [Guid]::NewGuid().ToString('N') + '.tmp')
  $stream = $null
  $temporaryOwned = $false
  try {
    $stream = [System.IO.File]::Open($temporary, [System.IO.FileMode]::CreateNew,
      [System.IO.FileAccess]::Write, [System.IO.FileShare]::None)
    $temporaryOwned = $true
    $stream.Write($bytes, 0, $bytes.Length)
    $stream.Flush()
    $stream.Dispose()
    $stream = $null
    # File.Move (two arguments) refuses an existing destination; do not use
    # Move-Item -Force or File.Copy here. The original report is never replaced.
    [System.IO.File]::Move($temporary, $file)
    $temporaryOwned = $false
  } finally {
    if ($null -ne $stream) { $stream.Dispose() }
    # This invocation owns ONLY the GUID-named staging file. A failed move
    # never removes or mutates an existing final report.
    if ($temporaryOwned -and [System.IO.File]::Exists($temporary)) {
      try { [System.IO.File]::Delete($temporary) } catch { }
    }
  }
  
  return $file
}

if ($NativeProfileBaseline) {
  # Reuse the same guarded Windows process/memory readers; no second monitor.
  . (Join-Path $PSScriptRoot 'native-profile-baseline.ps1')
  Invoke-NativeProfileBaseline -ProfilePids $ProfilePids -ProfileSlots $ProfileSlots -ManagerPid $ManagerPid -DurationSeconds $DurationSeconds -SampleIntervalSeconds $SampleIntervalSeconds -OutputDirectory $OutputDirectory -DirectChildren:$DirectChildren
  return
}

$baselineNative = Read-Target -ProcessId $NativePid
$baselineIsolated = Read-Target -ProcessId $IsolatedPid
if ($null -eq $baselineNative -or $null -eq $baselineIsolated) {
  throw 'Both manually selected processes must be accessible before measurement. No report produced.'
}
$extraTargets = @()
if ($fourClients) {
  $selectedExtras = @(@{ key='client03'; processId=$Client03Pid },
                      @{ key='client04'; processId=$Client04Pid })
  if ($fiveClients) { $selectedExtras += @{ key='client05'; processId=$Client05Pid } }
  foreach ($entry in $selectedExtras) {
    $baseline = Read-Target -ProcessId $entry.processId
    if ($null -eq $baseline) { throw 'All manually selected processes must be accessible before measurement. No report produced.' }
    $extraTargets += @{ key=$entry.key; processId=$entry.processId;
      baseline=$baseline; previous=$baseline; lost=$false }
  }
}
$logicalCpus = [Environment]::ProcessorCount
$managerBaseline = $null
$managerPrevious = $null
$managerLost = $null
$managerPreviousSecond = 0.0
if ($ManagerPid -ne 0) {
  $managerBaseline = Read-Target -ProcessId $ManagerPid
  if ($null -eq $managerBaseline) { throw 'Selected manager is inaccessible. No report produced.' }
  $managerPrevious = $managerBaseline
  $managerLost = $false
}
if ($logicalCpus -lt 1) { throw 'Host logical CPU count is invalid.' }
if (-not $OutputDirectory) {
  $OutputDirectory = Join-Path $env:TEMP 'MinecraftInstanceManager\Performance'
}
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null

# Exercise the SAME atomic writer before a long capture, including legacy path limits.
$OutputDirectory = (Get-Item -LiteralPath $OutputDirectory -ErrorAction Stop).FullName
$probe = $null
try {
  $probe = Write-ReportArtifact -Directory $OutputDirectory -Name (
    '.mivubi-native-performance-' + [Guid]::NewGuid().ToString('N') + '.probe') -Bytes ([byte[]]@(0))
} catch {
  throw [System.IO.IOException]::new(
    'Cannot create and move a report in the output directory. Choose a writable, shorter filesystem path.',
    $_.Exception)
} finally {
  # The writer returns this name only after it owns the completed probe.
  if ($null -ne $probe) { [System.IO.File]::Delete($probe) }
}

$beginUtc = [DateTime]::UtcNow
$timer = [System.Diagnostics.Stopwatch]::StartNew()
$previousSecond = 0.0
$previousNative = $baselineNative
$previousIsolated = $baselineIsolated
$nativeLost = $false
$isolatedLost = $false
$samples = [System.Collections.Generic.List[object]]::new()
$memorySamples = [System.Collections.Generic.List[object]]::new()

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
  if ($MemoryComposition) {
    $memoryStarted = $timer.Elapsed.TotalSeconds
    $memoryRow = [ordered]@{
      elapsedSeconds = [math]::Round($memoryStarted, 2)
      native = (Read-ProcessMemory -ProcessId $NativePid -ExpectedStartTicks $baselineNative.startTicks -IdentityLost $nativeLost)
      isolated = (Read-ProcessMemory -ProcessId $IsolatedPid -ExpectedStartTicks $baselineIsolated.startTicks -IdentityLost $isolatedLost)
      system = (Read-SystemMemory)
      queryDurationMs = [math]::Round(1000 * ($timer.Elapsed.TotalSeconds - $memoryStarted), 3)
    }
    if ($ManagerPid -ne 0 -or $DirectChildren) {
      $memoryRow['manager'] = $null
      if ($ManagerPid -ne 0) {
        $managerCurrent = if ($managerLost) { $null } else {
          Read-Target -ProcessId $ManagerPid -ExpectedStartTicks $managerBaseline.startTicks
        }
        if ($null -eq $managerCurrent) { $managerLost = $true }
        $managerSecond = $timer.Elapsed.TotalSeconds
        $memoryRow['manager'] = [ordered]@{
          process=(Process-Sample -Previous $managerPrevious -Current $managerCurrent -ElapsedSeconds ($managerSecond - $managerPreviousSecond) -LogicalCpus $logicalCpus)
          memory=(Read-ProcessMemory -ProcessId $ManagerPid -ExpectedStartTicks $managerBaseline.startTicks -IdentityLost $managerLost)
        }
        $managerPrevious = $managerCurrent
        $managerPreviousSecond = $managerSecond
      }
      $memoryRow['directChildren'] = $null
      if ($DirectChildren) {
        $roots = @(
          @{key='native'; processId=$NativePid; started=$baselineNative.startTicks; lost=$nativeLost},
          @{key='isolated'; processId=$IsolatedPid; started=$baselineIsolated.startTicks; lost=$isolatedLost}
        )
        $excluded = @($NativePid, $IsolatedPid, $PID)
        if ($ManagerPid -ne 0) {
          $roots += @{key='manager'; processId=$ManagerPid; started=$managerBaseline.startTicks; lost=$managerLost}
          $excluded += $ManagerPid
        }
        $memoryRow['directChildren'] = Read-DirectChildren -Roots $roots -Excluded $excluded
      }
      $memoryRow['queryDurationMs'] = [math]::Round(1000 * ($timer.Elapsed.TotalSeconds - $memoryStarted), 3)
    }
    $memorySamples.Add($memoryRow)
  }
  $previousNative = $nativeCurrent
  $previousIsolated = $isolatedCurrent
  $previousSecond = $elapsed
}

# Capture duration excludes the optional diagnostic query.
$durationWithoutGpu = [math]::Round($timer.Elapsed.TotalSeconds, 2)
$gpuDiagnostic = $null
if ($GpuSnapshot) {
  $gpuTargets = @(
    @{ key='native'; pid=$NativePid; started=$baselineNative.startTicks },
    @{ key='isolated'; pid=$IsolatedPid; started=$baselineIsolated.startTicks }
  )
  foreach ($entry in $extraTargets) {
    $gpuTargets += @{ key=$entry.key; pid=$entry.processId; started=$entry.baseline.startTicks }
  }
  $extraIdentityLost = @($extraTargets | Where-Object { $_.lost }).Count -gt 0
  if ($nativeLost -or $isolatedLost -or $extraIdentityLost) {
    # Never query recycled/inaccessible selected identities, including slot 05.
    $gpuDiagnostic = [ordered]@{
      evidenceClass = 'WINDOWS_GPU_COUNTER_HINT_NOT_FRAME_PROOF'
    }
    foreach ($item in $gpuTargets) { $gpuDiagnostic[$item.key] = Unknown-GpuHint }
  } else {
    $gpuDiagnostic = Read-GpuSnapshot -Targets $gpuTargets
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
if ($fiveClients) {
  $report['schema'] = 4
  $report['scope'] = 'FIVE_CLIENT_PROCESS_PERFORMANCE'
  $report['client05ProcessIdentityLost'] = [bool]$extraTargets[2].lost
  # A null field explicitly records that the optional one-shot query was not requested.
  $report['gpuDiagnostic'] = $gpuDiagnostic
}
$stamp = $beginUtc.ToString('yyyyMMddTHHmmssfffffffZ')
if ($MemoryComposition) {
  $report['baseSchema'] = $report.schema
  $report['schema'] = 5
  $report['memoryComposition'] = [ordered]@{
    evidenceClass = 'SELECTED_PROCESS_AND_WHOLE_SYSTEM_MEMORY_NOT_ENGINE_OVERHEAD'
    processTreeAttribution = 'NOT_COLLECTED'
    managerAttribution = 'NOT_COLLECTED'
    samples = @($memorySamples.ToArray())
  }
  if ($ManagerPid -ne 0 -or $DirectChildren) {
    $report['schema'] = 6
    $report.memoryComposition['managerAttribution'] = if ($ManagerPid -ne 0) { 'OPERATOR_SELECTED_IDENTITY_PINNED' } else { 'NOT_REQUESTED' }
    $report.memoryComposition['managerProcessIdentityLost'] = $managerLost
    $report.memoryComposition['directChildrenRequested'] = [bool]$DirectChildren
  }
}
# Windows PowerShell 5.1's Set-Content -Encoding UTF8 emits a BOM
# that strict Node JSON.parse rejects. Commit only a COMPLETE UTF8-no-BOM
# file: create an exclusive temporary file beside the final destination,
# flush/close it, then move it into place WITHOUT replacing existing reports.
# The same-directory move prevents an interrupted write leaving a fake final.
$json = $report | ConvertTo-Json -Depth 9
$utf8NoBom = [System.Text.UTF8Encoding]::new($false)
$bytes = $utf8NoBom.GetBytes($json + [Environment]::NewLine)
$file = Write-ReportArtifact -Directory $OutputDirectory -Name ('native-process-performance-' + $stamp + '.json') -Bytes $bytes
[ordered]@{
  report = $file
  sampleCount = $samples.Count
  automaticVerdict = 'NOT_PROVIDED'
  interpretation = 'PROCESS_TELEMETRY_ONLY'
} | ConvertTo-Json -Depth 3


