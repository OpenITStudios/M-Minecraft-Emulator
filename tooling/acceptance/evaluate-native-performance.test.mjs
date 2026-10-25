import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
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

