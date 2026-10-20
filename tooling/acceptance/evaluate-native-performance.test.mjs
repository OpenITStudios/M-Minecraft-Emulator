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
  assert.match(collector, /if \(\$nativeLost -or \$isolatedLost\)/);
  assert.match(collector, /if \(\$GpuSnapshot\) \{[\s\S]*?if \(\$nativeLost -or \$isolatedLost\)/);
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
