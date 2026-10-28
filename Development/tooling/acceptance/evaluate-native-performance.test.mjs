import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";
import test from "node:test";
import { evaluateNativePerformance, compareNativePerformance } from "./evaluate-native-performance.mjs";

const sample = () => ({
  schema: 1,
  scope: "NATIVE_SANDBOXIE_PROCESS_PERFORMANCE",
  evidenceOrigin: "WINDOWS_HOST",
  attribution: "OPERATOR_SELECTED_UNVERIFIED_PIDS",
  automaticVerdict: "NOT_PROVIDED",
  capturedAtUtc: "2026-10-13T12:00:00Z",
  durationRequestedSeconds: 60,
  durationObservedSeconds: 60,
  sampleIntervalSeconds: 5,
  hostLogicalCpus: 8,
  sampleCount: 2,
  nativeProcessIdentityLost: false,
  isolatedProcessIdentityLost: false,
  samples: [
    { elapsedSeconds: 5, native: {status: "OBSERVED", cpuPercentOfHost: 5, workingSetMb: 100},
      isolated: {status: "OBSERVED", cpuPercentOfHost: 10, workingSetMb: 220} },
    { elapsedSeconds: 10, native: {status: "OBSERVED", cpuPercentOfHost: 7, workingSetMb: 120},
      isolated: {status: "OBSERVED", cpuPercentOfHost: 20, workingSetMb: 240} },
  ],
  missingEvidence: ["ACCOUNT_ISOLATION", "GAMEPLAY_RENDERING", "FRAME_TIMES",
    "GPU_VRAM", "MULTIPLAYER", "INPUT_RESPONSIVENESS"],
});

const memoryMetric = (valueMb = 0, reason = null) => ({
  status: reason === null ? 'OBSERVED' : 'UNKNOWN',
  valueMb: reason === null ? valueMb : null, reason,
});
const memorySample = () => {
  const report = sample();
  report.baseSchema = report.schema;
  report.schema = 5;
  report.memoryComposition = {
    evidenceClass: 'SELECTED_PROCESS_AND_WHOLE_SYSTEM_MEMORY_NOT_ENGINE_OVERHEAD',
    processTreeAttribution: 'NOT_COLLECTED', managerAttribution: 'NOT_COLLECTED',
    samples: report.samples.map((row, i) => ({
      elapsedSeconds: row.elapsedSeconds, queryDurationMs: 1,
      native: {privateCommitMb: memoryMetric(200 + i * 20), privateWorkingSetMb: memoryMetric(80 + i * 10)},
      isolated: {privateCommitMb: memoryMetric(300 + i * 20), privateWorkingSetMb: memoryMetric(null, 'EXTENDED_COUNTER_UNAVAILABLE')},
      system: {totalPhysicalMb: memoryMetric(32768), availablePhysicalMb: memoryMetric(20000 - i * 100),
        commitTotalMb: memoryMetric(15000 + i * 100), commitLimitMb: memoryMetric(40000)},
    })),
  };
  return report;
};

test('memory composition distinguishes commitment, private residency, system pressure and unknown coverage', () => {
  const report = memorySample();
  const result = evaluateNativePerformance(report);
  assert.equal(result.native.meanWorkingSetMb, 110);
  assert.deepEqual(result.memoryComposition.native.privateCommitMb, {
    observedSamples: 2, missingSamples: 0, meanMb: 210, medianMb: 210, peakMb: 220,
    firstMb: 200, lastMb: 220, endpointDifferenceMb: 20, unavailableReasons: [],
  });
  assert.equal(result.memoryComposition.native.privateWorkingSetMb.meanMb, 85);
  assert.equal(result.memoryComposition.isolated.privateWorkingSetMb.meanMb, null);
  assert.equal(result.memoryComposition.isolated.privateWorkingSetMb.missingSamples, 2);
  assert.deepEqual(result.memoryComposition.isolated.privateWorkingSetMb.unavailableReasons, ['EXTENDED_COUNTER_UNAVAILABLE']);
  assert.equal(result.memoryComposition.system.availablePhysicalMb.endpointDifferenceMb, -100);
  assert.equal(result.memoryComposition.queryMeanDurationMs, 1);
  assert.equal(result.memoryComposition.engineOverheadIsolated, false);
  assert.equal(result.memoryComposition.processTreeMeasured, false);
  assert.equal(result.memoryComposition.managerMeasured, false);
  assert.equal(result.automaticGameplayProof, false);
  assert.equal(result.automaticPerformanceVerdict, 'NOT_PROVIDED');
  assert.equal(result.fourClientReady, false);
});

test('memory availability is independent per metric and identity loss invalidates both process metrics', () => {
  const report = memorySample();
  report.memoryComposition.samples[0].native.privateCommitMb = memoryMetric(null, 'QUERY_FAILED');
  const partial = evaluateNativePerformance(report).memoryComposition.native.privateCommitMb;
  assert.equal(partial.meanMb, 220);
  assert.equal(partial.firstMb, null);
  assert.equal(partial.endpointDifferenceMb, null);
  report.nativeProcessIdentityLost = true;
  report.samples[1].native = {status:'UNKNOWN', cpuPercentOfHost:null, workingSetMb:null};
  for (const field of ['privateCommitMb', 'privateWorkingSetMb']) {
    report.memoryComposition.samples[1].native[field] = memoryMetric(null, 'IDENTITY_UNAVAILABLE');
  }
  assert.equal(evaluateNativePerformance(report).memoryComposition.native.privateCommitMb.meanMb, null);
  report.memoryComposition.samples[1].native.privateCommitMb = memoryMetric(0);
  assert.throws(() => evaluateNativePerformance(report), /identity/);
});

test('memory comparison emits only fully observed matched-metric differences without engine claims', () => {
  const baseline = memorySample(), candidate = memorySample();
  candidate.memoryComposition.samples.forEach(row => row.native.privateCommitMb.valueMb -= 20);
  const result = compareNativePerformance(baseline, candidate);
  assert.equal(result.memoryDifferences.native.privateCommitMb.meanMbDifference, -20);
  assert.equal(result.memoryDifferences.native.privateWorkingSetMb.meanMbDifference, 0);
  assert.equal(result.memoryDifferences.isolated.privateWorkingSetMb, null);
  assert.equal(result.engineOverheadIsolated, false);
  assert.equal(result.improvementVerified, false);
  candidate.memoryComposition.samples[0].native.privateCommitMb = memoryMetric(null, 'API_UNAVAILABLE');
  assert.equal(compareNativePerformance(baseline, candidate).memoryDifferences.native.privateCommitMb, null);
  candidate.sampleIntervalSeconds++;
  const mismatched = compareNativePerformance(baseline, candidate);
  assert.equal(mismatched.differences, null);
  assert.ok(Object.values(mismatched.memoryDifferences).every(group => Object.values(group).every(value => value === null)));
  assert.throws(() => compareNativePerformance(baseline, sample()), /two schema-5/);
});

test('memory schema refuses injected identity, invented availability, incomplete timing and overclaims', () => {
  const invalid = [
    r => r.baseSchema = 3,
    r => r.baseSchema = 2,
    r => r.memoryComposition.pid = 123,
    r => r.memoryComposition.managerAttribution = 'VERIFIED',
    r => r.memoryComposition.samples.pop(),
    r => r.memoryComposition.samples[1].elapsedSeconds = 5,
    r => r.memoryComposition.samples[0].elapsedSeconds = 4,
    r => r.memoryComposition.samples[0].queryDurationMs = -1,
    r => r.memoryComposition.samples[1].queryDurationMs = 60000,
    r => r.memoryComposition.samples[0].native.privateCommitMb.valueMb = Infinity,
    r => r.memoryComposition.samples[0].native.privateCommitMb.reason = 'QUERY_FAILED',
    r => r.memoryComposition.samples[0].isolated.privateWorkingSetMb.valueMb = 0,
    r => r.memoryComposition.samples[0].isolated.privateWorkingSetMb.reason = null,
    r => r.memoryComposition.samples[0].native.privateCommitMb = memoryMetric(null, 'EXTENDED_COUNTER_UNAVAILABLE'),
    r => r.memoryComposition.samples[0].native.privateCommitMb = memoryMetric(null, 'IDENTITY_UNAVAILABLE'),
    r => r.memoryComposition.samples[0].system.commitTotalMb = memoryMetric(null, 'IDENTITY_UNAVAILABLE'),
    r => r.memoryComposition.samples[0].native.privateCommitMb.pid = 42,
    r => r.samples[0].native.pid = 42,
    r => r.providerReady = true,
  ];
  for (const mutate of invalid) {
    const report = memorySample(); mutate(report);
    assert.throws(() => evaluateNativePerformance(report), TypeError);
  }
  const report = memorySample();
  report.baseSchema = 2;
  report.gpuDiagnostic = withGpu().gpuDiagnostic;
  assert.equal(evaluateNativePerformance(report).gpuPerformanceVerified, false);
});

test('Windows memory reader uses pinned query-only handles and preserves unavailable metrics', {
  skip: process.platform !== 'win32',
}, () => {
  const folder = mkdtempSync(join(tmpdir(), 'mivubi-memory-reader-'));
  try {
    const script = join(folder, 'verify.ps1');
    writeFileSync(script, `param([string]$Collector)
$ErrorActionPreference = 'Stop'
$tokens = $null; $errors = $null
$ast = [System.Management.Automation.Language.Parser]::ParseFile($Collector, [ref]$tokens, [ref]$errors)
if ($errors.Count -ne 0) { throw 'Collector does not parse' }
$MemoryComposition = $true
$memoryApiAvailable = $false
$setup = $ast.EndBlock.Statements | Where-Object {
  $_ -is [System.Management.Automation.Language.IfStatementAst] -and
  $_.Clauses[0].Item1.Extent.Text -eq '$MemoryComposition'
} | Select-Object -First 1
if ($null -eq $setup) { throw 'Missing opt-in setup' }
Invoke-Expression $setup.Extent.Text
if (-not $memoryApiAvailable) { throw 'Native memory API setup failed' }
$names = @('Read-Target', 'Memory-Metric', 'Read-ProcessMemory', 'Read-SystemMemory')
$ast.FindAll({ param($node)
  $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -in $names
}, $false) | ForEach-Object { Invoke-Expression $_.Extent.Text }
$self = Read-Target -ProcessId $PID
if ($null -eq $self) { throw 'Own process unavailable' }
$observed = Read-ProcessMemory -ProcessId $PID -ExpectedStartTicks $self.startTicks -IdentityLost $false
if ($observed.privateCommitMb.status -ne 'OBSERVED' -or $observed.privateCommitMb.valueMb -le 0) {
  throw 'Private commit was not observed'
}
if ($null -ne [Mivubi.PerformanceMemory]::Read($PID, ($self.startTicks + 1))) {
  throw 'Mismatched creation identity accepted'
}
$lost = Read-ProcessMemory -ProcessId $PID -ExpectedStartTicks $self.startTicks -IdentityLost $true
if ($lost.privateCommitMb.reason -ne 'IDENTITY_UNAVAILABLE' -or $null -ne $lost.privateCommitMb.valueMb) {
  throw 'Lost identity produced a value'
}
$system = Read-SystemMemory
if ($system.totalPhysicalMb.status -ne 'OBSERVED' -or
    $system.availablePhysicalMb.valueMb -gt $system.totalPhysicalMb.valueMb -or
    $system.commitTotalMb.valueMb -gt $system.commitLimitMb.valueMb) { throw 'Invalid system counters' }
# No retained native or System.Diagnostics.Process handle per sample.
$process = [System.Diagnostics.Process]::GetCurrentProcess()
$before = $process.HandleCount
foreach ($i in 1..200) {
  $null = Read-Target -ProcessId $PID -ExpectedStartTicks $self.startTicks
  $null = [Mivubi.PerformanceMemory]::Read($PID, $self.startTicks)
}
$process.Refresh()
$growth = $process.HandleCount - $before
$process.Dispose()
if ($growth -gt 20) { throw 'Query handles retained per sample' }
$memoryApiAvailable = $false
$unavailable = Read-ProcessMemory -ProcessId $PID -ExpectedStartTicks $self.startTicks -IdentityLost $false
if ($unavailable.privateCommitMb.reason -ne 'API_UNAVAILABLE' -or
    $null -ne $unavailable.privateWorkingSetMb.valueMb) { throw 'Unavailable API fabricated zero' }
if (@((Read-SystemMemory).Values | Where-Object { $_.reason -ne 'API_UNAVAILABLE' }).Count -ne 0) {
  throw 'Unavailable system metrics fabricated values'
}
@{ readerPass=$true; privateWorkingSet=$observed.privateWorkingSetMb.status;
  handleGrowth=$growth; minecraft='NOT_EXECUTED' } | ConvertTo-Json -Compress
`, 'utf8');
    const collector = fileURLToPath(new URL('./windows/collect-native-performance.ps1', import.meta.url));
    for (const shell of ['powershell.exe', 'pwsh.exe']) {
      const run = spawnSync(shell, ['-NoProfile', '-File', script, '-Collector', collector],
        {encoding:'utf8', timeout:20000, maxBuffer:1024 * 1024, windowsHide:true});
      assert.equal(run.status, 0, run.stderr || shell);
      const result = JSON.parse(run.stdout.replace(/^\uFEFF/, ''));
      assert.equal(result.readerPass, true);
      assert.equal(result.minecraft, 'NOT_EXECUTED');
    }
  } finally { rmSync(folder, {recursive:true, force:true}); }
});

const childMetrics = (count = 1) => ({
  status:'OBSERVED', reason:null, verifiedCount:count, unresolvedCount:0,
  privateCommitMb:memoryMetric(count * 30), privateWorkingSetMb:memoryMetric(count * 10),
});
const relatedSample = () => {
  const report = memorySample();
  report.schema = 6;
  Object.assign(report.memoryComposition, {managerAttribution:'OPERATOR_SELECTED_IDENTITY_PINNED',
    managerProcessIdentityLost:false, directChildrenRequested:true});
  for (const row of report.memoryComposition.samples) {
    row.manager = {process:{status:'OBSERVED', cpuPercentOfHost:1, workingSetMb:50},
      memory:{privateCommitMb:memoryMetric(60), privateWorkingSetMb:memoryMetric(30)}};
    row.directChildren = {native:childMetrics(), isolated:childMetrics(0), manager:childMetrics(2)};
  }
  return report;
};

test('related-process memory keeps selected manager and direct relations separate without ownership claims', () => {
  const result = evaluateNativePerformance(relatedSample());
  const memory = result.memoryComposition;
  assert.equal(memory.managerMeasured, true);
  assert.equal(memory.managerRoleVerified, false);
  assert.equal(memory.manager.process.meanHostCpuPercent, 1);
  assert.equal(memory.manager.privateCommitMb.meanMb, 60);
  assert.equal(memory.directChildren.native.privateCommitMb.meanMb, 30);
  assert.equal(memory.directChildren.isolated.privateCommitMb.meanMb, 0);
  assert.deepEqual(memory.directChildren.manager.verifiedCounts, [2, 2]);
  assert.equal(memory.sharedBrokerAttribution, 'UNKNOWN');
  assert.equal(memory.processTreeMeasured, false);
  assert.equal(memory.engineOverheadIsolated, false);
  assert.equal(result.automaticPerformanceVerdict, 'NOT_PROVIDED');
  const report = relatedSample();
  report.memoryComposition.managerAttribution = 'NOT_REQUESTED';
  report.memoryComposition.managerProcessIdentityLost = null;
  for (const row of report.memoryComposition.samples) { row.manager = null; delete row.directChildren.manager; }
  assert.equal(evaluateNativePerformance(report).memoryComposition.manager, null);
});

test('related-process loss and partial child relationships retain null totals', () => {
  const report = relatedSample(), row = report.memoryComposition.samples[1];
  report.memoryComposition.managerProcessIdentityLost = true;
  row.manager.process = {status:'UNKNOWN', cpuPercentOfHost:null, workingSetMb:null};
  row.manager.memory = {privateCommitMb:memoryMetric(null, 'IDENTITY_UNAVAILABLE'),
    privateWorkingSetMb:memoryMetric(null, 'IDENTITY_UNAVAILABLE')};
  row.directChildren.manager = {status:'UNKNOWN', reason:'ROOT_IDENTITY_UNAVAILABLE',
    verifiedCount:null, unresolvedCount:null, privateCommitMb:memoryMetric(null, 'QUERY_FAILED'),
    privateWorkingSetMb:memoryMetric(null, 'QUERY_FAILED')};
  row.directChildren.native = {...row.directChildren.manager, reason:'PARTIAL_RELATION',
    verifiedCount:1, unresolvedCount:1};
  const result = evaluateNativePerformance(report).memoryComposition;
  assert.equal(result.manager.process.missingSamples, 1);
  assert.equal(result.manager.privateCommitMb.lastMb, null);
  assert.equal(result.directChildren.native.privateCommitMb.lastMb, null);
  assert.equal(result.directChildren.native.observedInventorySamples, 1);
  assert.equal(result.directChildren.manager.unresolvedInventorySamples, 1);
  assert.equal(compareNativePerformance(report, report).comparableForNumericDifference, false);
});

test('schema-6 comparisons preserve descriptive component differences and reject mismatched collection modes', () => {
  const before = relatedSample(), after = relatedSample();
  for (const row of after.memoryComposition.samples) {
    row.manager.memory.privateCommitMb.valueMb -= 5;
    row.directChildren.native.privateCommitMb.valueMb -= 10;
  }
  const result = compareNativePerformance(before, after);
  assert.equal(result.managerMemoryDifferences.privateCommitMb.meanMbDifference, -5);
  assert.equal(result.directChildMemoryDifferences.native.privateCommitMb.meanMbDifference, -10);
  assert.equal(result.directChildMemoryDifferences.isolated.privateCommitMb.meanMbDifference, 0);
  assert.equal(result.exclusiveHelperOwnershipVerified, false);
  assert.equal(result.improvementVerified, false);
  after.memoryComposition.directChildrenRequested = false;
  for (const row of after.memoryComposition.samples) row.directChildren = null;
  const mismatch = compareNativePerformance(before, after);
  assert.equal(mismatch.conditions.sameRelatedSelection, false);
  assert.equal(mismatch.differences, null);
  assert.equal(mismatch.directChildMemoryDifferences.native.privateCommitMb, null);
  assert.throws(() => compareNativePerformance(before, memorySample()), /two schema-5 or two schema-6/);
});

test('related evidence rejects identity resurrection, unrequested metrics, partial totals and overclaimed scope', () => {
  const invalid = [
    r => r.memoryComposition.processTreeAttribution = 'COMPLETE',
    r => r.memoryComposition.managerProcessIdentityLost = null,
    r => r.memoryComposition.managerProcessIdentityLost = true,
    r => r.memoryComposition.samples[0].manager.pid = 123,
    r => r.memoryComposition.samples[0].manager.memory.privateCommitMb.pid = 123,
    r => r.memoryComposition.samples[0].manager.process.status = 'UNKNOWN',
    r => r.memoryComposition.samples[0].directChildren.native.reason = 'PARTIAL_RELATION',
    r => r.memoryComposition.samples[0].directChildren.native.unresolvedCount = 1,
    r => r.memoryComposition.samples[0].directChildren.native.verifiedCount = 65,
    r => r.memoryComposition.samples[0].directChildren.native.pid = 123,
    r => r.memoryComposition.samples[0].directChildren.isolated.privateCommitMb.valueMb = 1,
    r => delete r.memoryComposition.samples[0].directChildren.manager,
    r => r.memoryComposition.directChildrenRequested = false,
    r => r.memoryComposition.managerAttribution = 'NOT_REQUESTED',
  ];
  for (const mutate of invalid) {
    const report = relatedSample(); mutate(report);
    assert.throws(() => evaluateNativePerformance(report), TypeError);
  }
});

test('Windows child attribution rejects reuse, churn, ambiguous coverage and selected-root double counting', {
  skip: process.platform !== 'win32',
}, () => {
  const folder = mkdtempSync(join(tmpdir(), 'mivubi-child-attribution-'));
  try {
    const script = join(folder, 'verify.ps1');
    writeFileSync(script, `param([string]$Collector)
$ErrorActionPreference = 'Stop'
$tokens = $null; $errors = $null
$ast = [System.Management.Automation.Language.Parser]::ParseFile($Collector, [ref]$tokens, [ref]$errors)
if ($errors.Count -ne 0) { throw 'Collector parse failed' }
$names = @('Memory-Metric', 'Unknown-Children', 'Read-DirectChildren')
$ast.FindAll({ param($node)
  $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -in $names
}, $false) | ForEach-Object { Invoke-Expression $_.Extent.Text }
$roots = @(@{key='native'; processId=10; started=100; lost=$false},
  @{key='isolated'; processId=20; started=200; lost=$false},
  @{key='manager'; processId=30; started=50; lost=$false})
function Reset-Case {
  $script:births = @{10=100; 20=200; 30=50; 11=300; 21=400; 31=450; 41=500; 99=600}
  $script:first = @(
    [pscustomobject]@{ProcessId=11; ParentId=10}, [pscustomobject]@{ProcessId=21; ParentId=20},
    [pscustomobject]@{ProcessId=31; ParentId=30}, [pscustomobject]@{ProcessId=10; ParentId=30},
    [pscustomobject]@{ProcessId=20; ParentId=30}, [pscustomobject]@{ProcessId=41; ParentId=11},
    [pscustomobject]@{ProcessId=99; ParentId=10})
  $script:second = @($script:first)
  $script:calls = 0; $script:missing = 0; $script:restart = 0; $script:failSnapshot = $false
}
function Read-Target {
  param([int]$ProcessId, [long]$ExpectedStartTicks=0)
  if ($ProcessId -eq $script:missing -or
      ($ExpectedStartTicks -gt 0 -and $ProcessId -eq $script:restart)) { return $null }
  if (-not $script:births.ContainsKey($ProcessId)) { return $null }
  $birth = $script:births[$ProcessId]
  if ($ExpectedStartTicks -gt 0 -and $ExpectedStartTicks -ne $birth) { return $null }
  return [pscustomobject]@{startTicks=$birth}
}
function Read-ChildSnapshot {
  param($Parents)
  $script:calls++
  if ($script:failSnapshot) { return $null }
  return [pscustomobject]@{Entries=$(if ($script:calls % 2 -eq 1) { $script:first } else { $script:second })}
}
function Read-ProcessMemory {
  param([int]$ProcessId, [long]$ExpectedStartTicks, [bool]$IdentityLost)
  return [ordered]@{
    privateCommitMb=(Memory-Metric -Value ($ProcessId * 10) -Reason 'QUERY_FAILED')
    privateWorkingSetMb=$(if ($ProcessId -eq 21) { Memory-Metric -Value $null -Reason 'EXTENDED_COUNTER_UNAVAILABLE' }
      else { Memory-Metric -Value $ProcessId -Reason 'QUERY_FAILED' })
  }
}
function Run-Case { Read-DirectChildren -Roots $roots -Excluded @(10,20,30,99) }
Reset-Case
$result = Run-Case
if ($result.native.verifiedCount -ne 1 -or $result.native.privateCommitMb.valueMb -ne 110 -or
    $result.manager.verifiedCount -ne 1 -or $result.manager.privateCommitMb.valueMb -ne 310) {
  throw 'Wrong root or selected root/collector/grandchild double counting'
}
if ($result.isolated.privateCommitMb.valueMb -ne 210 -or
    $result.isolated.privateWorkingSetMb.status -ne 'UNKNOWN') { throw 'Per-metric availability lost' }
Reset-Case; $script:births[11] = 90
if ((Run-Case).native.reason -ne 'PARTIAL_RELATION') { throw 'Reused parent accepted an older child' }
Reset-Case; $script:restart = 11
if ((Run-Case).native.reason -ne 'PARTIAL_RELATION') { throw 'Reused child identity accepted' }
Reset-Case
$script:second += [pscustomobject]@{ProcessId=12; ParentId=10}
$result = Run-Case
if ($result.native.reason -ne 'PARTIAL_RELATION' -or
    $result.native.unresolvedCount -ne 1 -or $null -ne $result.native.privateCommitMb.valueMb) {
  throw 'New unpinned child hidden in an observed total'
}
Reset-Case; $script:missing = 10
if ((Run-Case).native.reason -ne 'ROOT_IDENTITY_UNAVAILABLE') { throw 'Missing root accepted' }
Reset-Case; $script:failSnapshot = $true
if ((Run-Case).native.reason -ne 'SNAPSHOT_UNAVAILABLE') { throw 'Snapshot failure fabricated empty inventory' }
Reset-Case; $script:first = @(); $script:second = @()
$result = Run-Case
if ($result.native.status -ne 'OBSERVED' -or $result.native.verifiedCount -ne 0 -or
    $result.native.privateCommitMb.valueMb -ne 0) { throw 'Verified empty inventory unavailable' }
Reset-Case
$script:first = @(1000..1064 | ForEach-Object { [pscustomobject]@{ProcessId=$_; ParentId=10} })
$script:second = $script:first
if ((Run-Case).native.reason -ne 'CANDIDATE_LIMIT') { throw 'Unbounded helper pinning' }
@{attributionPass=$true; cases=8; minecraft='NOT_EXECUTED'} | ConvertTo-Json -Compress
`, 'utf8');
    const collector = fileURLToPath(new URL('./windows/collect-native-performance.ps1', import.meta.url));
    for (const shell of ['powershell.exe', 'pwsh.exe']) {
      const run = spawnSync(shell, ['-NoProfile','-File',script,'-Collector',collector],
        {encoding:'utf8', timeout:20000, maxBuffer:1024 * 1024, windowsHide:true});
      assert.equal(run.status, 0, run.stderr || shell);
      assert.equal(JSON.parse(run.stdout).attributionPass, true);
    }
  } finally { rmSync(folder, {recursive:true, force:true}); }
});

test('related-process options reject incomplete or overlapping selection before producing artifacts', {
  skip: process.platform !== 'win32',
}, () => {
  const folder = mkdtempSync(join(tmpdir(), 'mivubi-related-options-'));
  try {
    const collector = fileURLToPath(new URL('./windows/collect-native-performance.ps1', import.meta.url));
    const cases = [
      [['-ManagerPid','30'], /require -MemoryComposition/],
      [['-DirectChildren'], /require -MemoryComposition/],
      [['-ManagerPid','10','-MemoryComposition'], /manager must be distinct/],
    ];
    for (const [i, [args, message]] of cases.entries()) {
      const output = join(folder, 'unexpected-' + i);
      const run = spawnSync('powershell.exe', ['-NoProfile','-File',collector,
        '-NativePid','10','-IsolatedPid','20','-OutputDirectory',output,...args],
        {encoding:'utf8', timeout:15000, maxBuffer:1024 * 1024, windowsHide:true});
      assert.notEqual(run.status, 0);
      assert.match(run.stderr, message);
      assert.equal(existsSync(output), false);
    }
  } finally { rmSync(folder, {recursive:true, force:true}); }
});

test('atomic writer preserves existing artifacts and refuses unusable output before timed sampling', {
  skip: process.platform !== 'win32',
}, () => {
  const folder = mkdtempSync(join(tmpdir(), 'mivubi-output-preflight-'));
  try {
    const script = join(folder, 'verify.ps1');
    writeFileSync(script, `param([string]$Collector, [string]$Folder, [int]$OtherPid)
$ErrorActionPreference = 'Stop'
$tokens=$null; $errors=$null
$ast=[System.Management.Automation.Language.Parser]::ParseFile($Collector,[ref]$tokens,[ref]$errors)
if ($errors.Count -ne 0) { throw 'Collector parse failed' }
$function=$ast.Find({param($node) $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
  $node.Name -eq 'Write-ReportArtifact'}, $false)
Invoke-Expression $function.Extent.Text
$bytes=[System.Text.UTF8Encoding]::new($false).GetBytes("{}\n")
$normal=Write-ReportArtifact -Directory $Folder -Name 'existing.json' -Bytes $bytes
$original=[System.IO.File]::ReadAllText($normal)
$collisionRefused=$false
try { $null=Write-ReportArtifact -Directory $Folder -Name 'existing.json' -Bytes ([byte[]]@(1)) }
catch { $collisionRefused=$true }
if (-not $collisionRefused -or [System.IO.File]::ReadAllText($normal) -ne $original) {
  throw 'An existing report was overwritten'
}
if (@(Get-ChildItem -LiteralPath $Folder -Filter '*.tmp' -Force).Count -ne 0) { throw 'Owned staging leaked after failed move' }
$longRoot=Join-Path $Folder ('x' * (215 - $Folder.Length - 1))
New-Item -ItemType Directory -Path $longRoot | Out-Null
$longWritable=$false
$owned=$null
try {
  $owned=Write-ReportArtifact -Directory $longRoot -Name 'probe.json' -Bytes $bytes
  $longWritable=$true
} catch { }
finally { if ($null -ne $owned) { [System.IO.File]::Delete($owned) } }
$earlyRefusal=$false
if (-not $longWritable) {
  $timer=[System.Diagnostics.Stopwatch]::StartNew()
  try {
    & $Collector -NativePid $OtherPid -IsolatedPid $PID -DurationSeconds 30 -SampleIntervalSeconds 5 -OutputDirectory $longRoot | Out-Null
  } catch {
    if ($_.Exception.Message -notmatch 'Cannot create and move a report') { throw }
    $earlyRefusal=$true
  }
  if (-not $earlyRefusal -or $timer.Elapsed.TotalSeconds -gt 10) { throw 'Output failure was not detected before timed capture' }
  if (@(Get-ChildItem -LiteralPath $longRoot -File -Force).Count -ne 0) { throw 'Early refusal left a partial report' }
}
@{writerPass=$true; longWritable=$longWritable; earlyRefusal=$earlyRefusal; minecraft='NOT_EXECUTED'} | ConvertTo-Json -Compress
`, 'utf8');
    const collector = fileURLToPath(new URL('./windows/collect-native-performance.ps1', import.meta.url));
    for (const shell of ['powershell.exe', 'pwsh.exe']) {
      const run = spawnSync(shell, ['-NoProfile','-File',script,'-Collector',collector,
        '-Folder',folder,'-OtherPid',String(process.pid)],
        {encoding:'utf8',timeout:20000,maxBuffer:1024 * 1024,windowsHide:true});
      assert.equal(run.status, 0, run.stderr || shell);
      const result = JSON.parse(run.stdout);
      assert.equal(result.writerPass, true);
      assert.equal(result.earlyRefusal, !result.longWritable);
      // Only the test owns these deliberately created files.
      rmSync(join(folder, 'existing.json'));
      const longRoot = join(folder, 'x'.repeat(215 - folder.length - 1));
      rmSync(longRoot, {recursive:true,force:true});
    }
  } finally { rmSync(folder, {recursive:true,force:true}); }
});

const fourSample = () => {
  const report = sample();
  report.schema = 3;
  report.scope = 'FOUR_CLIENT_PROCESS_PERFORMANCE';
  report.client03ProcessIdentityLost = false;
  report.client04ProcessIdentityLost = false;
  for (const row of report.samples) {
    row.client03 = {...row.native};
    row.client04 = {...row.isolated};
  }
  return report;
};

const fiveSample = () => {
  const report = fourSample();
  report.schema = 4;
  report.scope = 'FIVE_CLIENT_PROCESS_PERFORMANCE';
  report.client05ProcessIdentityLost = false;
  report.gpuDiagnostic = null;
  report.samples.forEach((row, i) => {
    row.client05 = {status: 'OBSERVED', cpuPercentOfHost: i + 1, workingSetMb: 500 + i * 100};
  });
  return report;
};

test('five-client capture retains separate fifth metrics, loss and unavailable GPU evidence', () => {
  const report = fiveSample();
  const result = evaluateNativePerformance(report);
  assert.deepEqual(Object.keys(result.clients), ['01', '02', '03', '04', '05']);
  assert.equal(result.clients['05'].meanHostCpuPercent, 1.5);
  assert.equal(result.clients['05'].meanWorkingSetMb, 550);
  assert.equal(result.clients['05'].maxWorkingSetMb, 600);
  assert.equal(result.gpuDiagnostic, null);
  assert.equal(result.fiveClientReady, false);
  assert.equal(result.automaticGameplayProof, false);
  assert.equal(result.gpuPerformanceVerified, false);
  assert.equal(result.frameTimingVerified, false);
  assert.equal(result.automaticPerformanceVerdict, 'NOT_PROVIDED');
  report.client05ProcessIdentityLost = true;
  report.samples[1].client05 = {status: 'UNKNOWN', cpuPercentOfHost: null, workingSetMb: null};
  const lost = evaluateNativePerformance(report);
  assert.equal(lost.clients['05'].observationCoveragePercent, 50);
  assert.equal(lost.clients['05'].firstMissingAtSeconds, 10);
  assert.equal(lost.clients['04'].observationCoveragePercent, 100);
  assert.equal(lost.identityDiscontinuityObserved, true);
  assert.throws(() => compareNativePerformance(report, report), /individual review only/);
  assert.throws(() => compareNativePerformance(sample(), report), /individual review only/);
});

test('four-client review preserves separate metrics without account or FPS verdicts', () => {
  const result = evaluateNativePerformance(fourSample());
  assert.deepEqual(Object.keys(result.clients), ['01', '02', '03', '04']);
  assert.equal(result.clients['03'].meanHostCpuPercent, 6);
  assert.equal(result.clients['04'].meanWorkingSetMb, 230);
  assert.equal(result.fourClientReady, false);
  assert.equal(result.automaticGameplayProof, false);
  assert.equal(result.automaticPerformanceVerdict, 'NOT_PROVIDED');
});

test('four-client review refuses partial, resumed, injected and contradictory evidence', () => {
  const invalid = [
    r => delete r.samples[0].client03,
    r => r.samples[0].pid = 42,
    r => r.client04ProcessIdentityLost = true,
    r => r.samples[0].client03 = {status:'UNKNOWN', cpuPercentOfHost:null, workingSetMb:null},
    r => r.samples[0].client04.cpuPercentOfHost = Infinity,
    r => r.scope = 'NATIVE_SANDBOXIE_PROCESS_PERFORMANCE',
    r => r.gpuDiagnostic = {},
    r => r.providerVerified = true,
  ];
  for (const mutate of invalid) {
    const report = fourSample();
    mutate(report);
    assert.throws(() => evaluateNativePerformance(report), TypeError);
  }
  const lost = fourSample();
  lost.client03ProcessIdentityLost = true;
  lost.samples[1].client03 = {status:'UNKNOWN', cpuPercentOfHost:null, workingSetMb:null};
  const result = evaluateNativePerformance(lost);
  assert.equal(result.clients['03'].missingSamples, 1);
  assert.equal(result.identityDiscontinuityObserved, true);
  assert.equal(result.clients['04'].missingSamples, 0);
  assert.throws(() => compareNativePerformance(lost, lost), /individual review only/);
});

test("complete CPU/RAM values still never create gameplay or performance PASS", () => {
  const result = evaluateNativePerformance(sample());
  assert.equal(result.interpretation, "PROCESS_TELEMETRY_ONLY");
  assert.equal(result.automaticGameplayProof, false);
  assert.equal(result.automaticPerformanceVerdict, "NOT_PROVIDED");
  assert.equal(result.fourClientReady, false);
  assert.deepEqual(result.native, {
    observedSamples: 2, missingSamples: 0,
    observationCoveragePercent: 100, firstMissingAtSeconds: null,
    meanHostCpuPercent: 6, peakHostCpuPercent: 7,
    meanWorkingSetMb: 110, maxWorkingSetMb: 120,
  });
  assert.equal(result.isolated.peakHostCpuPercent, 20);
  assert.equal(result.isolated.meanWorkingSetMb, 230);
  assert.equal(result.isolated.maxWorkingSetMb, 240);
  assert.equal(result.metricScope, "OPERATOR_SELECTED_PROCESSES_ONLY_NOT_PROCESS_TREES");
  assert.equal(result.identityDiscontinuityInterpretation,
    "PROCESS_MISSING_OR_INACCESSIBLE_NOT_PROOF_OF_CRASH");
});

test("inaccessible/restarted process stays unknown instead of fabricating 0 samples", () => {
  const r = sample();
  r.isolatedProcessIdentityLost = true;
  r.samples[1].isolated = {status: "UNKNOWN", cpuPercentOfHost: null, workingSetMb: null};
  const result = evaluateNativePerformance(r);
  assert.equal(result.identityDiscontinuityObserved, true);
  assert.equal(result.isolated.observedSamples, 1);
  assert.equal(result.isolated.missingSamples, 1);
  assert.equal(result.isolated.meanHostCpuPercent, 10);
  assert.equal(result.isolated.peakHostCpuPercent, 10);
  assert.equal(result.isolated.meanWorkingSetMb, 220);
  assert.equal(result.isolated.observationCoveragePercent, 50);
  assert.equal(result.isolated.firstMissingAtSeconds, 10);
  // A missing reading is not automatically classified as a Minecraft crash.
  assert.equal(result.automaticPerformanceVerdict, "NOT_PROVIDED");
});

test("no process observations preserve null metrics, not artificial zeros", () => {
  const r = sample();
  r.nativeProcessIdentityLost = true;
  r.samples.forEach(s => s.native = {status: "UNKNOWN", cpuPercentOfHost: null, workingSetMb: null});
  const result = evaluateNativePerformance(r);
  assert.equal(result.native.meanHostCpuPercent, null);
  assert.equal(result.native.peakHostCpuPercent, null);
  assert.equal(result.native.meanWorkingSetMb, null);
  assert.equal(result.native.maxWorkingSetMb, null);
  assert.equal(result.native.observationCoveragePercent, 0);
  assert.equal(result.native.firstMissingAtSeconds, 5);
});

test("reject impossible process reappearance and mismatched identity flags", () => {
  const falseFlag = sample();
  falseFlag.samples[1].native = {status: "UNKNOWN", cpuPercentOfHost: null, workingSetMb: null};
  assert.throws(() => evaluateNativePerformance(falseFlag), TypeError);
  const impossibleResurrection = sample();
  impossibleResurrection.nativeProcessIdentityLost = true;
  impossibleResurrection.samples[0].native = {status: "UNKNOWN", cpuPercentOfHost: null, workingSetMb: null};
  assert.throws(() => evaluateNativePerformance(impossibleResurrection), TypeError);
});

test("reject PID/account injection, fabricated verdict, and invalid schema", () => {
  for (const mutate of [
    r => r.PID = 100,
    r => r.samples[0].native.pid = 100,
    r => r.accountEmail = "hidden@example.com",
    r => r.automaticVerdict = "PASS",
    r => r.scope = "FOUR_CLIENT_READY",
    r => r.samples[0].native.cpuPercentOfHost = Number.NaN,
    r => r.samples[0].native.workingSetMb = -3,
    r => r.samples[0].isolated.status = "GAMEPLAY_VERIFIED",
    r => r.samples[0].isolated.workingSetMb = null,
    r => r.samples[1].elapsedSeconds = 5,
    r => r.sampleCount = 20,
    r => r.missingEvidence = [],
    r => r.nativeProcessIdentityLost = true
  ]) {
    const r = sample();
    mutate(r);
    assert.throws(() => evaluateNativePerformance(r), TypeError);
  }
});

test("zero CPU samples remain legitimate observations, not missing evidence", () => {
  const r = sample();
  r.samples[0].native.cpuPercentOfHost = 0;
  r.samples[1].native.cpuPercentOfHost = 0;
  r.samples[0].native.workingSetMb = 0;
  const result = evaluateNativePerformance(r);
  assert.equal(result.native.meanHostCpuPercent, 0);
  assert.equal(result.native.peakHostCpuPercent, 0);
  assert.equal(result.native.meanWorkingSetMb, 60);
  assert.equal(result.native.observationCoveragePercent, 100);
  assert.equal(result.native.firstMissingAtSeconds, null);
  assert.equal(result.fourClientReady, false);
});

test("fractional observation coverage and first loss locate incomplete samples", () => {
  const r = sample();
  r.sampleCount = 3;
  r.samples.push({ elapsedSeconds: 15,
    native: {status: "OBSERVED", cpuPercentOfHost: 3, workingSetMb: 90},
    isolated: {status: "UNKNOWN", cpuPercentOfHost: null, workingSetMb: null},
  });
  r.isolatedProcessIdentityLost = true;
  assert.equal(evaluateNativePerformance(r).isolated.observationCoveragePercent, 66.67);
  assert.equal(evaluateNativePerformance(r).isolated.firstMissingAtSeconds, 15);
});

test("paired complete reports produce differences but never an optimization verdict", () => {
  const baseline = sample();
  const candidate = sample();
  candidate.samples[0].native.cpuPercentOfHost = 3;
  candidate.samples[1].native.cpuPercentOfHost = 5;
  candidate.samples[0].native.workingSetMb = 90;
  candidate.samples[1].native.workingSetMb = 100;
  candidate.samples[0].isolated.cpuPercentOfHost = 8;
  candidate.samples[1].isolated.cpuPercentOfHost = 14;
  const result = compareNativePerformance(baseline, candidate);
  assert.equal(result.comparableForNumericDifference, true);
  assert.equal(result.performanceVerdict, "NOT_PROVIDED");
  assert.equal(result.improvementVerified, false);
  assert.equal(result.sameHostVerified, false);
  assert.equal(result.sameWorkloadVerified, false);
  assert.equal(result.fullProcessTreeMeasured, false);
  assert.deepEqual(result.differences.native, {
    meanHostCpuPercentagePointDifference: -2,
    peakHostCpuPercentagePointDifference: -2,
    meanWorkingSetMbDifference: -15,
    maxWorkingSetMbDifference: -20,
  });
  assert.equal(result.differences.isolated.meanHostCpuPercentagePointDifference, -4);
  assert.equal(result.baseline.fourClientReady, false);
  assert.equal(result.candidate.fourClientReady, false);
});

test("mismatched host, duration, interval, count or identity loss suppress differences", () => {
  const variants = [
    r => r.hostLogicalCpus = 16,
    r => r.durationRequestedSeconds = 120,
    r => r.sampleIntervalSeconds = 6,
    r => { r.sampleCount = 1; r.samples = r.samples.slice(0, 1); },
    r => r.durationObservedSeconds = 70,
    r => { r.nativeProcessIdentityLost = true;
      r.samples[1].native = {status: "UNKNOWN", cpuPercentOfHost: null, workingSetMb: null}; },
  ];
  for (const mutate of variants) {
    const candidate = sample();
    mutate(candidate);
    const result = compareNativePerformance(sample(), candidate);
    assert.equal(result.comparableForNumericDifference, false);
    assert.equal(result.differences, null);
    assert.equal(result.improvementVerified, false);
  }
});

test("invalid evidence in either comparison input fails closed", () => {
  const invalid = sample();
  invalid.accountEmail = "must-never-enter-report@example.invalid";
  assert.throws(() => compareNativePerformance(sample(), invalid), TypeError);
  assert.throws(() => compareNativePerformance(invalid, sample()), TypeError);
  const impossible = sample();
  impossible.samples[1].isolated.status = "UNKNOWN";
  assert.throws(() => compareNativePerformance(sample(), impossible), TypeError);
});

const gpuHint = (status, engine, dedicated) => ({
  status,
  threeDEngineMaximumInstancePercent: engine,
  dedicatedMemoryMaximumInstanceMb: dedicated,
});
const withGpu = () => ({
  ...sample(),
  schema: 2,
  gpuDiagnostic: {
    evidenceClass: "WINDOWS_GPU_COUNTER_HINT_NOT_FRAME_PROOF",
    native: gpuHint("OBSERVED", 4.5, 256),
    isolated: gpuHint("OBSERVED", 12, 300),
  },
});

test("optional GPU hints do not change old CPU/RAM report structure", () => {
  const legacy = evaluateNativePerformance(sample());
  assert.equal("gpuDiagnostic" in legacy, false);
  assert.equal("gpuPerformanceVerified" in legacy, false);
  assert.equal(legacy.interpretation, "PROCESS_TELEMETRY_ONLY");
  assert.equal(legacy.native.meanHostCpuPercent, 6);
});

test("GPU snapshot stays an unverified Windows counter hint, never gameplay proof", () => {
  const result = evaluateNativePerformance(withGpu());
  assert.equal(result.gpuDiagnostic.interpretation,
    "ONE_TIME_WINDOWS_COUNTER_HINT_UNVERIFIED");
  assert.equal(result.gpuDiagnostic.native.threeDEngineMaximumInstancePercent, 4.5);
  assert.equal(result.gpuDiagnostic.isolated.dedicatedMemoryMaximumInstanceMb, 300);
  assert.equal(result.gpuPerformanceVerified, false);
  assert.equal(result.gpuCounterMemoryMayBeStale, true);
  assert.equal(result.frameTimingVerified, false);
  assert.equal(result.fourClientReady, false);
  assert.equal(result.automaticPerformanceVerdict, "NOT_PROVIDED");
  assert.ok(result.missingEvidence.includes("GPU_VRAM"));
  assert.ok(result.missingEvidence.includes("FRAME_TIMES"));
});

test("missing GPU counters remain null UNKNOWN while CPU/RAM evidence survives", () => {
  const report = withGpu();
  report.gpuDiagnostic.native = gpuHint("UNKNOWN", null, null);
  report.gpuDiagnostic.isolated = gpuHint("UNKNOWN", null, null);
  const result = evaluateNativePerformance(report);
  assert.equal(result.gpuDiagnostic.native.status, "UNKNOWN");
  assert.equal(result.gpuDiagnostic.native.threeDEngineMaximumInstancePercent, null);
  assert.equal(result.native.meanHostCpuPercent, 6);
  assert.equal(result.gpuPerformanceVerified, false);
});

test("schema-2 rejects malformed, unsafe, overclaimed and unexpected GPU evidence", () => {
  const changes = [
    r => r.gpuDiagnostic.native.status = "READY",
    r => r.gpuDiagnostic.native.threeDEngineMaximumInstancePercent = 150,
    r => r.gpuDiagnostic.native.dedicatedMemoryMaximumInstanceMb = -1,
    r => r.gpuDiagnostic.native.dedicatedMemoryMaximumInstanceMb = 1e12,
    r => r.gpuDiagnostic.native.dedicatedMemoryMaximumInstanceMb = Number.NaN,
    r => r.gpuDiagnostic.native.frameTimes = [16],
    r => r.gpuDiagnostic.native.status = "UNKNOWN",
    r => r.gpuDiagnostic.evidenceClass = "GPU_FPS_VERIFIED",
    r => r.gpuDiagnostic.native = {},
    r => r.gpuDiagnostic.capturedPid = 123,
    r => r.gpuDiagnostic.isolated.threeDEngineMaximumInstancePercent = null,
    r => r.schema = 1,
    r => r.missingEvidence = r.missingEvidence.filter(x => x !== "GPU_VRAM"),
  ];
  for (const mutate of changes) {
    const r = withGpu();
    mutate(r);
    assert.throws(() => evaluateNativePerformance(r), TypeError);
  }
});

test("cross-schema CPU/RAM reports do not emit numeric improvement comparisons", () => {
  const result = compareNativePerformance(sample(), withGpu());
  assert.equal(result.conditions.sameReportSchema, false);
  assert.equal(result.comparableForNumericDifference, false);
  assert.equal(result.differences, null);
  assert.equal(result.improvementVerified, false);
  const pair = compareNativePerformance(withGpu(), withGpu());
  assert.equal(pair.conditions.sameReportSchema, true);
  assert.equal(pair.comparableForNumericDifference, true);
  assert.equal(pair.improvementVerified, false);
  assert.equal(pair.differences.native.meanHostCpuPercentagePointDifference, 0);
});

test("opt-in GPU counter capture is one-shot and leaves default collector overhead unchanged", () => {
  const collector = readFileSync(new URL("./windows/collect-native-performance.ps1", import.meta.url), "utf8");
  assert.match(collector, /\[switch\]\$GpuSnapshot/);
  assert.match(collector, /if \(\$GpuSnapshot\) \{[\s\S]*?Read-GpuSnapshot/);
  assert.match(collector, /\$report\['gpuDiagnostic'\] = \$gpuDiagnostic/);
  assert.match(collector, /schema = 1/);
  assert.match(collector, /\$report\['schema'\] = 2/);
  assert.match(collector, /Get-Counter[\s\S]*?-MaxSamples 2/);
  assert.match(collector, /ExpectedStartTicks \$item\.started/);
  assert.match(collector, /status = 'UNKNOWN'/);
  assert.match(collector, /durationObservedSeconds = \$durationWithoutGpu/);
  assert.doesNotMatch(collector, /Start-Process|Start-Service|Stop-Process|Stop-Service|Set-Process/);
  assert.doesNotMatch(collector, /Set-ItemProperty|Set-GpuPreference|taskkill/);
  // No GPU counter collection happens inside the CPU/RAM sampling loop.
  const loopBody = collector.split("while ($timer.Elapsed.TotalSeconds -lt $DurationSeconds) {")[1]
    ?.split("# Capture duration excludes the optional diagnostic query.")[0] ?? "";
  assert.ok(loopBody.length > 0);
  assert.doesNotMatch(loopBody, /Get-Counter|Read-GpuSnapshot/);
});

test("historical Windows PowerShell UTF8 BOM and no-BOM captures work through the real Node CLI", () => {
  const folder = mkdtempSync(join(tmpdir(), "mivubi-native-performance-"));
  try {
    const old = join(folder, "windows-powershell-5.1-bom.json");
    const current = join(folder, "native-no-bom.json");
    writeFileSync(old, "\uFEFF" + JSON.stringify(sample()), "utf8");
    writeFileSync(current, JSON.stringify(sample()), "utf8");
    const executable = fileURLToPath(new URL("./evaluate-native-performance.mjs", import.meta.url));
    const single = spawnSync(process.execPath, [executable, old], {
      encoding: "utf8", timeout: 10000, maxBuffer: 1024 * 1024,
    });
    assert.equal(single.status, 0, single.stderr || "BOM single capture failed");
    assert.equal(JSON.parse(single.stdout).native.meanHostCpuPercent, 6);
    const pair = spawnSync(process.execPath, [executable, old, current], {
      encoding: "utf8", timeout: 10000, maxBuffer: 1024 * 1024,
    });
    assert.equal(pair.status, 0, pair.stderr || "mixed BOM/non-BOM pair failed");
    const compared = JSON.parse(pair.stdout);
    assert.equal(compared.comparableForNumericDifference, true);
    assert.equal(compared.improvementVerified, false);
    assert.equal(compared.differences.native.meanWorkingSetMbDifference, 0);
  } finally {
    rmSync(folder, { recursive: true, force: true });
  }
});

test("Windows GPU diagnostics skip identity-loss captures and write BOM-less reports exclusively", () => {
  const collector = readFileSync(new URL("./windows/collect-native-performance.ps1", import.meta.url), "utf8");
  assert.match(collector, /if \(\$nativeLost -or \$isolatedLost -or \$extraIdentityLost\)/);
  assert.match(collector, /if \(\$GpuSnapshot\) \{[\s\S]*?if \(\$nativeLost -or \$isolatedLost -or \$extraIdentityLost\)/);
  assert.match(collector, /\[System\.Text\.UTF8Encoding\]::new\(\$false\)/);
  assert.match(collector, /\[System\.IO\.FileMode\]::CreateNew/);
  assert.match(collector, /\[System\.IO\.FileShare\]::None/);
  assert.match(collector, /\[System\.IO\.File\]::Move\(\$temporary, \$file\)/);
  assert.match(collector, /\[Guid\]::NewGuid\(\)/);
  assert.match(collector, /\[System\.IO\.File\]::Delete\(\$temporary\)/);
  assert.doesNotMatch(collector, /\[System\.IO\.File\]::Delete\(\$file\)/);
  assert.doesNotMatch(collector, /Set-Content -LiteralPath \$file/);
  assert.doesNotMatch(collector, /FileMode\]::Create\b/);
});

const fiveWithGpu = () => {
  const report = fiveSample();
  report.gpuDiagnostic = {evidenceClass: 'WINDOWS_GPU_COUNTER_HINT_NOT_FRAME_PROOF'};
  ['native', 'isolated', 'client03', 'client04', 'client05'].forEach((key, i) => {
    report.gpuDiagnostic[key] = gpuHint('OBSERVED', i + 1, (i + 1) * 100);
  });
  return report;
};

test('five-client GPU hints retain slot attribution and never certify VRAM or rendering', () => {
  const report = fiveWithGpu();
  const result = evaluateNativePerformance(report);
  assert.equal(result.gpuDiagnostic['01'].threeDEngineMaximumInstancePercent, 1);
  assert.equal(result.gpuDiagnostic['05'].dedicatedMemoryMaximumInstanceMb, 500);
  assert.equal(result.gpuCounterMemoryMayBeStale, true);
  assert.equal(result.gpuPerformanceVerified, false);
  assert.equal(result.fiveClientReady, false);
  assert.ok(result.missingEvidence.includes('GPU_VRAM'));
  assert.ok(result.missingEvidence.includes('MULTIPLAYER'));
  report.client05ProcessIdentityLost = true;
  report.samples[1].client05 = {status: 'UNKNOWN', cpuPercentOfHost: null, workingSetMb: null};
  for (const key of ['native', 'isolated', 'client03', 'client04', 'client05']) {
    report.gpuDiagnostic[key] = gpuHint('UNKNOWN', null, null);
  }
  assert.equal(evaluateNativePerformance(report).gpuDiagnostic['05'].status, 'UNKNOWN');
});

test('five-client review rejects missing, revived, misattributed and contradictory evidence', () => {
  for (const mutate of [
    r => delete r.client05ProcessIdentityLost,
    r => delete r.gpuDiagnostic,
    r => delete r.samples[0].client05,
    r => r.samples[0].client06 = {...r.samples[0].client05},
    r => r.client05ProcessIdentityLost = true,
    r => { r.client05ProcessIdentityLost = true;
      r.samples[0].client05 = {status:'UNKNOWN', cpuPercentOfHost:null, workingSetMb:null}; },
    r => r.samples[1].client05.cpuPercentOfHost = Infinity,
    r => r.samples[0].client05.pid = 123,
    r => r.scope = 'FOUR_CLIENT_PROCESS_PERFORMANCE',
    r => r.schema = 3,
    r => r.providerVerified = true,
    r => delete r.gpuDiagnostic.client05,
    r => r.gpuDiagnostic.client06 = gpuHint('OBSERVED', 1, 100),
    r => r.gpuDiagnostic.client05.pid = 123,
    r => r.gpuDiagnostic.client05.threeDEngineMaximumInstancePercent = 101,
    r => r.gpuDiagnostic.client05.status = 'UNKNOWN',
    r => { r.client05ProcessIdentityLost = true;
      r.samples[1].client05 = {status:'UNKNOWN', cpuPercentOfHost:null, workingSetMb:null}; },
  ]) {
    const report = fiveWithGpu();
    mutate(report);
    assert.throws(() => evaluateNativePerformance(report), TypeError);
  }
});


test('Windows GPU reader attributes all five synthetic slots and refuses recycled identities',
  {skip: process.platform !== 'win32'}, () => {
    const folder = mkdtempSync(join(tmpdir(), 'mivubi-five-gpu-reader-'));
    try {
      const script = join(folder, 'reader-control.ps1');
      writeFileSync(script, String.raw`param([string]$Collector)
$ErrorActionPreference = 'Stop'
$tokens = $null; $errors = $null
$ast = [System.Management.Automation.Language.Parser]::ParseFile($Collector, [ref]$tokens, [ref]$errors)
if ($errors.Count -ne 0) { throw 'Collector parse failed' }
$definitions = @($ast.FindAll({ param($node)
  $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
  $node.Name -in @('Unknown-GpuHint', 'Read-GpuSnapshot')
}, $true))
if ($definitions.Count -ne 2) { throw 'Missing canonical GPU reader functions' }
foreach ($definition in $definitions) { . ([scriptblock]::Create($definition.Extent.Text)) }
$script:counterCalls = 0
$script:missing = 0
$script:failCounters = $false
function Read-Target {
  param([int]$ProcessId, [long]$ExpectedStartTicks)
  if ($ProcessId -eq $script:missing) { return $null }
  return @{ startTicks = $ExpectedStartTicks }
}
function Get-Counter {
  param($Counter, $SampleInterval, $MaxSamples, $ErrorAction)
  $script:counterCalls++
  if ($MaxSamples -ne 2 -or $SampleInterval -ne 1) { throw 'Unexpected query frequency' }
  if ($script:failCounters) { throw 'Unavailable counter control' }
  $samples = @()
  foreach ($i in 0..4) {
    $fakePid = 100 + $i
    $samples += [pscustomobject]@{ Path=('\GPU Engine(pid_' + $fakePid + '_engtype_3D)\Utilization Percentage'); CookedValue=($i + 1) }
    $samples += [pscustomobject]@{ Path=('\GPU Process Memory(pid_' + $fakePid + '_one)\Dedicated Usage'); CookedValue=((10 + $i) * 1MB) }
    $samples += [pscustomobject]@{ Path=('\GPU Process Memory(pid_' + $fakePid + '_two)\Dedicated Usage'); CookedValue=((20 + $i) * 1MB) }
  }
  return @([pscustomobject]@{CounterSamples=$samples}, [pscustomobject]@{CounterSamples=$samples})
}
$keys = @('native', 'isolated', 'client03', 'client04', 'client05')
$targets = @(foreach ($i in 0..4) { @{key=$keys[$i]; pid=(100 + $i); started=123} })
$result = Read-GpuSnapshot -Targets $targets
foreach ($i in 0..4) {
  $hint = $result[$keys[$i]]
  if ($hint.status -ne 'OBSERVED' -or $hint.threeDEngineMaximumInstancePercent -ne ($i + 1) -or
      $hint.dedicatedMemoryMaximumInstanceMb -ne (20 + $i)) { throw 'Cross-slot counter attribution' }
}
$legacy = Read-GpuSnapshot -Targets $targets[0..1]
if ($legacy.Count -ne 3 -or $legacy.native.status -ne 'OBSERVED' -or
    $legacy.isolated.dedicatedMemoryMaximumInstanceMb -ne 21) { throw 'Legacy two-client shape changed' }
$script:missing = 104
$result = Read-GpuSnapshot -Targets $targets
if ($result.client05.status -ne 'UNKNOWN' -or $null -ne $result.client05.dedicatedMemoryMaximumInstanceMb -or
    $result.client04.status -ne 'OBSERVED') { throw 'Recycled identity accepted or wrong slot lost' }
$script:failCounters = $true
$result = Read-GpuSnapshot -Targets $targets
if (@($keys | Where-Object { $result[$_].status -ne 'UNKNOWN' }).Count -ne 0) { throw 'Counter failure fabricated data' }
if ($script:counterCalls -ne 4) { throw 'Unexpected counter query count' }
@{ syntheticReaderPass=$true; slots=5; queryCalls=$script:counterCalls; minecraft='NOT_EXECUTED' } | ConvertTo-Json -Compress
`, 'utf8');
      const collector = fileURLToPath(new URL('./windows/collect-native-performance.ps1', import.meta.url));
      const run = spawnSync('powershell.exe', ['-NoProfile', '-File', script, '-Collector', collector],
        {encoding: 'utf8', timeout: 15000, maxBuffer: 1024 * 1024, windowsHide: true});
      assert.equal(run.status, 0, run.stderr);
      const result = JSON.parse(run.stdout.replace(/^\uFEFF/, ''));
      assert.equal(result.syntheticReaderPass, true);
      assert.equal(result.slots, 5);
      assert.equal(result.queryCalls, 4);
      assert.equal(result.minecraft, 'NOT_EXECUTED');
    } finally {
      rmSync(folder, {recursive: true, force: true});
    }
  });





const nativeBaselineFixture = (slots = ['01', '04'], manager = false, children = false) => {
  const system = i => ({
    totalPhysicalMb: memoryMetric(32768),
    availablePhysicalMb: memoryMetric(20000 - i * 30),
    commitTotalMb: memoryMetric(16000 + i * 30),
    commitLimitMb: memoryMetric(45000),
  });
  const process = i => ({
    process: {status:'OBSERVED', cpuPercentOfHost:2+i, workingSetMb:300+i*10},
    memory: {privateCommitMb: memoryMetric(500+i*20), privateWorkingSetMb: memoryMetric(250+i*10)},
  });
  const emptyChild = () => ({
    status:'OBSERVED', reason:null, verifiedCount:0, unresolvedCount:0,
    privateCommitMb:memoryMetric(0), privateWorkingSetMb:memoryMetric(0),
  });
  return {
    schema: 7, scope:'NATIVE_PROFILE_MEMORY_BASELINE',
    evidenceOrigin:'WINDOWS_HOST', attribution:'OPERATOR_SELECTED_UNVERIFIED_PIDS',
    automaticVerdict:'NOT_PROVIDED', capturedAtUtc:'2026-10-28T12:00:00Z',
    durationRequestedSeconds:30, durationObservedSeconds:10.5, sampleIntervalSeconds:5,
    hostLogicalCpus:16, sampleCount:2, selectedSlots:[...slots],
    profileIdentityLost:Object.fromEntries(slots.map(slot=>[slot,false])),
    managerSelected:manager, managerProcessIdentityLost:manager ? false : null,
    directChildrenRequested:children,
    samples:[0,1].map(i => ({
      elapsedSeconds:(i+1)*5,
      clients:Object.fromEntries(slots.map(slot => [slot,process(i)])),
      manager:manager ? process(i) : null,
      system:system(i),
      directChildren:children ? Object.fromEntries([...slots,...(manager?['manager']:[])]
        .map(key => [key,emptyChild()])) : null,
      queryDurationMs:1.25,
    })),
    missingEvidence:['ACCOUNT_ISOLATION','GAMEPLAY_RENDERING','FRAME_TIMES',
      'GPU_VRAM','MULTIPLAYER','INPUT_RESPONSIVENESS'],
  };
};

test('native RAM schema-7 handles zero, sparse, and five selected slots without inventing workload proof', () => {
  for (const slots of [[],['03'],['01','04'],['01','02','03','04','05']]) {
    const report = nativeBaselineFixture(slots);
    const review = evaluateNativePerformance(report);
    assert.deepEqual(review.selectedSlots, slots);
    assert.equal(Object.keys(review.clients).length, slots.length);
    assert.equal(review.fiveClientReady, false);
    assert.equal(review.improvementVerified, false);
    assert.equal(review.system.availablePhysicalMb.medianMb, 19985);
  }
  const withManager = evaluateNativePerformance(nativeBaselineFixture(['02'], true, true));
  assert.equal(withManager.manager.privateCommitMb.medianMb, 510);
  assert.equal(withManager.directChildren['02'].verifiedCounts.length, 2);
  assert.equal(withManager.processTreeMeasured, false);
});

test('native per-profile RAM goal distance is numeric only with full residency and identity coverage', () => {
  assert.deepEqual(evaluateNativePerformance(nativeBaselineFixture([])).aspirationalRamGoal, {});
  const baseline = nativeBaselineFixture(['01', '05']);
  for (const row of baseline.samples) {
    row.clients['01'].memory.privateWorkingSetMb = memoryMetric(1000);
    row.clients['05'].memory.privateWorkingSetMb = memoryMetric(1400);
  }
  const report = evaluateNativePerformance(baseline);
  assert.equal(report.aspirationalRamGoal['01'].medianMinusTargetMb, -24);
  assert.equal(report.aspirationalRamGoal['01'].reductionToTargetPercent, 0);
  assert.equal(report.aspirationalRamGoal['05'].medianPrivateWorkingSetMb, 1400);
  assert.equal(report.aspirationalRamGoal['05'].medianMinusTargetMb, 376);
  assert.equal(report.aspirationalRamGoal['05'].reductionToTargetPercent, 26.86);
  assert.equal(report.aspirationalRamGoal['05'].fullSampleCoverage, true);
  assert.equal(report.aspirationalRamGoal['05'].verifiedGoalAchievement, false);
  assert.equal(report.aspirationalRamGoal['05'].verifiedGameWorkload, false);
  assert.equal(report.fiveClientReady, false);

  const partial = structuredClone(baseline);
  partial.samples[1].clients['05'].memory.privateWorkingSetMb =
    memoryMetric(null, 'QUERY_FAILED');
  const unknown = evaluateNativePerformance(partial).aspirationalRamGoal['05'];
  assert.equal(unknown.fullSampleCoverage, false);
  assert.equal(unknown.medianPrivateWorkingSetMb, null);
  assert.equal(unknown.medianMinusTargetMb, null);
  assert.equal(unknown.reductionToTargetPercent, null);

  const lost = structuredClone(baseline);
  lost.samples[1].clients['05'].process = {
    status:'UNKNOWN', cpuPercentOfHost:null, workingSetMb:null,
  };
  lost.samples[1].clients['05'].memory = {
    privateCommitMb:memoryMetric(null, 'IDENTITY_UNAVAILABLE'),
    privateWorkingSetMb:memoryMetric(null, 'IDENTITY_UNAVAILABLE'),
  };
  lost.profileIdentityLost['05'] = true;
  assert.equal(evaluateNativePerformance(lost).aspirationalRamGoal['05'].medianMinusTargetMb, null);

  const exactlyAtThreshold = structuredClone(baseline);
  for (const row of exactlyAtThreshold.samples) {
    row.clients['05'].memory.privateWorkingSetMb = memoryMetric(1024);
  }
  const exact = evaluateNativePerformance(exactlyAtThreshold).aspirationalRamGoal['05'];
  assert.equal(exact.medianMinusTargetMb, 0);
  assert.equal(exact.verifiedGoalAchievement, false, 'Strict <1 GiB target is never auto-certified');
});

test('schema-7 numeric RAM comparisons are descriptive and require matching complete identities', () => {
  const before = nativeBaselineFixture(['01','05']);
  const after = structuredClone(before);
  for (const row of after.samples) row.clients['05'].memory.privateCommitMb.valueMb -= 40;
  const compared = compareNativePerformance(before,after);
  assert.equal(compared.comparableForNumericDifference,true);
  assert.equal(compared.memoryDifferences.clients['05'].privateCommitMb.meanMbDifference,-40);
  assert.equal(compared.improvementVerified,false);
  assert.equal(compared.sameHostVerified,false);
  after.samples[1].clients['05'].process = {status:'UNKNOWN',cpuPercentOfHost:null,workingSetMb:null};
  after.samples[1].clients['05'].memory = {
    privateCommitMb:memoryMetric(null,'IDENTITY_UNAVAILABLE'),
    privateWorkingSetMb:memoryMetric(null,'IDENTITY_UNAVAILABLE'),
  };
  after.profileIdentityLost['05']=true;
  assert.equal(compareNativePerformance(before,after).comparableForNumericDifference,false);
  after.profileIdentityLost['05']=false;
  assert.throws(()=>evaluateNativePerformance(after),/identity-loss flags/);
});

test('schema-7 rejects misleading slot names, unrequested descendants, invented RAM and malformed timestamps', () => {
  for (const mutate of [
    r=>r.selectedSlots=['05','01'],
    r=>r.selectedSlots=['01','01'],
    r=>r.selectedSlots=['06'],
    r=>r.profileIdentityLost['03']=false,
    r=>r.managerProcessIdentityLost=false,
    r=>r.samples[0].directChildren={},
    r=>r.samples[0].clients['04'].memory.privateCommitMb=memoryMetric(null,'IDENTITY_UNAVAILABLE'),
    r=>r.samples[1].system.availablePhysicalMb.valueMb=-20,
    r=>r.samples[1].queryDurationMs=6000,
    r=>r.samples[1].elapsedSeconds=5,
    r=>r.samples[0].clients['01'].process.status='UNKNOWN',
  ]) {
    const report=nativeBaselineFixture();
    mutate(report);
    assert.throws(()=>evaluateNativePerformance(report));
  }
});


const faultCount = (count = 0, reason = null) => ({
  status: reason === null ? 'OBSERVED' : 'UNKNOWN',
  count: reason === null ? count : null, reason,
});

test('schema-7 page-fault counts are optional, cumulative and not disk/page-in proof', () => {
  const previous = nativeBaselineFixture(['01','04'], true);
  assert.equal(evaluateNativePerformance(previous).pageFaultCountCollected, false);
  const current = structuredClone(previous);
  for (const [i,row] of current.samples.entries()) {
    for (const slot of ['01','04']) row.clients[slot].memory.pageFaultCount =
      faultCount(100 + 20 * i);
    row.manager.memory.pageFaultCount = faultCount(300 + 5 * i);
  }
  const result = evaluateNativePerformance(current);
  assert.equal(result.pageFaultCountCollected, true);
  assert.equal(result.clients['04'].pageFaultCount.endpointDeltaCount, 20);
  assert.equal(result.manager.pageFaultCount.endpointDeltaCount, 5);
  assert.equal(result.clients['01'].privateWorkingSetMb.medianMb, 255);
  assert.equal(compareNativePerformance(previous,current).comparableForNumericDifference,false);
  assert.equal(compareNativePerformance(current,structuredClone(current)).comparableForNumericDifference,true);
  current.samples[1].clients['01'].memory.pageFaultCount.count = 50;
  const wrap = evaluateNativePerformance(current);
  assert.equal(wrap.clients['01'].pageFaultCount.endpointDeltaCount, null);
  assert.equal(wrap.clients['01'].pageFaultCount.counterDecreaseObserved, true);
});

test('schema-7 rejects forged, inconsistent or identity-lost page-fault counters', () => {
  const full = () => {
    const report = nativeBaselineFixture(['01','03']);
    for (const row of report.samples) {
      row.clients['01'].memory.pageFaultCount = faultCount(100);
      row.clients['03'].memory.pageFaultCount = faultCount(100);
    }
    return report;
  };
  for (const mutate of [
    r=>r.samples[1].clients['01'].memory.pageFaultCount.count = -1,
    r=>r.samples[1].clients['01'].memory.pageFaultCount.count = 4294967296,
    r=>r.samples[1].clients['01'].memory.pageFaultCount.count = 3.5,
    r=>r.samples[1].clients['03'].memory.pageFaultCount.reason = 'QUERY_FAILED',
    r=>r.samples[0].clients['01'].memory.pageFaultCount = faultCount(null,'IDENTITY_UNAVAILABLE'),
    r=>r.samples[0].clients['01'].memory.pageFaultCount = faultCount(null,'EXTENDED_COUNTER_UNAVAILABLE'),
    r=>delete r.samples[1].clients['03'].memory.pageFaultCount,
    r=>r.samples[0].clients['01'].memory.pageFaultCount.pid = 42,
  ]) {
    const report = full();
    mutate(report);
    assert.throws(()=>evaluateNativePerformance(report));
  }
  const lost=full();
  lost.samples[1].clients['01'].process = {
    status:'UNKNOWN',cpuPercentOfHost:null,workingSetMb:null,
  };
  lost.samples[1].clients['01'].memory = {
    privateCommitMb:memoryMetric(null,'IDENTITY_UNAVAILABLE'),
    privateWorkingSetMb:memoryMetric(null,'IDENTITY_UNAVAILABLE'),
    pageFaultCount:faultCount(null,'IDENTITY_UNAVAILABLE'),
  };
  lost.profileIdentityLost['01']=true;
  const status=evaluateNativePerformance(lost);
  assert.equal(status.clients['01'].pageFaultCount.endpointDeltaCount,null);
  assert.equal(status.clients['01'].pageFaultCount.missingSamples,1);
});


const nativeBuildFixture = (slots = ['01', '04']) => {
  const report = nativeBaselineFixture(slots);
  report.schema = 8;
  report.imageEvidence = {
    evidenceClass: 'PROCESS_IMAGE_PATH_FILE_SHA256_NOT_LOADED_IMAGE_PROOF',
    slots: Object.fromEntries(slots.map(slot => [slot, {
      status: 'OBSERVED', sha256: 'a'.repeat(64), reason: null,
    }])),
  };
  return report;
};

test('schema-8 compares only matching complete path-file fingerprints; no loaded-image claim', () => {
  const baseline = nativeBuildFixture();
  const candidate = structuredClone(baseline);
  const report = evaluateNativePerformance(baseline);
  assert.equal(report.imageEvidence.complete, true);
  assert.equal(report.imageEvidence.provesLoadedGameImage, false);
  assert.equal(compareNativePerformance(baseline, candidate).comparableForNumericDifference, true);
  candidate.imageEvidence.slots['04'].sha256 = 'b'.repeat(64);
  const changed = compareNativePerformance(baseline, candidate);
  assert.equal(changed.conditions.sameGameImageEvidence, false);
  assert.equal(changed.comparableForNumericDifference, false);
  assert.equal(changed.memoryDifferences, null);
  candidate.imageEvidence.slots['04'] = {
    status: 'UNKNOWN', sha256: null, reason: 'IMAGE_FILE_UNAVAILABLE',
  };
  assert.equal(evaluateNativePerformance(candidate).imageEvidence.complete, false);
  assert.equal(compareNativePerformance(baseline, candidate).comparableForNumericDifference, false);
  assert.throws(() => compareNativePerformance(nativeBaselineFixture(), baseline), /matching schema-7 or schema-8/);
  assert.equal(evaluateNativePerformance(nativeBaselineFixture()).imageEvidence, undefined);
  assert.equal(compareNativePerformance(nativeBaselineFixture(), nativeBaselineFixture()).comparableForNumericDifference, true);
  assert.equal(compareNativePerformance(nativeBuildFixture([]), nativeBuildFixture([])).comparableForNumericDifference, false);
});


test('schema-8 rejects mixed file hashes inside one capture even when A/B matches per-slot', () => {
  const baseline = nativeBuildFixture(['01','04','05']);
  baseline.imageEvidence.slots['04'].sha256 = 'b'.repeat(64);
  const candidate = structuredClone(baseline);
  const result = compareNativePerformance(baseline,candidate);
  assert.equal(result.conditions.sameGameImageEvidence, true,
    'The old per-slot A/B check alone would have accepted this mixed-build roster');
  assert.equal(result.conditions.uniformSelectedGameBuilds, false);
  assert.equal(result.comparableForNumericDifference, false);
  assert.equal(result.memoryDifferences, null);
  const inspected = evaluateNativePerformance(baseline);
  assert.equal(inspected.imageEvidence.complete, true);
  assert.equal(inspected.imageEvidence.uniformSelectedImageHashes, false);
  assert.equal(inspected.imageEvidence.provesLoadedGameImage, false);
  assert.equal(evaluateNativePerformance(nativeBuildFixture(['03']))
    .imageEvidence.uniformSelectedImageHashes, true);
  assert.equal(evaluateNativePerformance(nativeBuildFixture([]))
    .imageEvidence.uniformSelectedImageHashes, false);
  assert.equal(compareNativePerformance(nativeBaselineFixture(['01','04']),
    nativeBaselineFixture(['01','04'])).comparableForNumericDifference, true);
});

test('schema-8 rejects invented, malformed, or incorrectly attributed image identities', () => {
  for (const mutate of [
    r => delete r.imageEvidence,
    r => r.imageEvidence.evidenceClass = 'VERIFIED_MINECRAFT_EDUCATION_BINARY',
    r => r.imageEvidence.slots['03'] = {status:'OBSERVED',sha256:'a'.repeat(64),reason:null},
    r => r.imageEvidence.slots['01'].sha256 = 'A'.repeat(64),
    r => r.imageEvidence.slots['01'].sha256 = 'a'.repeat(63),
    r => r.imageEvidence.slots['01'].reason = 'verified',
    r => r.imageEvidence.slots['04'] = {status:'UNKNOWN',sha256:'a'.repeat(64),reason:'QUERY_FAILED'},
    r => r.imageEvidence.slots['04'] = {status:'UNKNOWN',sha256:null,reason:'NOT_FOUND'},
    r => r.imageEvidence.slots['01'].filePath = 'C:/Users/secret',
  ]) {
    const report = nativeBuildFixture();
    mutate(report);
    assert.throws(() => evaluateNativePerformance(report));
  }
});
