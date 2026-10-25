import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

// Interpret process-only evidence; never infer licensed accounts, Minecraft
// rendering, frametimes, GPU behavior or four-client readiness from CPU/RAM.
const isObject = (value) => value !== null && typeof value === "object" && !Array.isArray(value);
const finite = (value) => typeof value === "number" && Number.isFinite(value);
const integer = (value) => Number.isSafeInteger(value);
const missing = ["ACCOUNT_ISOLATION", "GAMEPLAY_RENDERING", "FRAME_TIMES",
  "GPU_VRAM", "MULTIPLAYER", "INPUT_RESPONSIVENESS"];

function exactly(object, names) {
  return isObject(object) && Object.keys(object).length === names.length &&
    Object.keys(object).every(key => names.includes(key));
}
function sampleMetrics(value) {
  if (!exactly(value, ["status", "cpuPercentOfHost", "workingSetMb"]) ||
      !["OBSERVED", "UNKNOWN"].includes(value.status)) throw new TypeError("Invalid process sample.");
  const cpu = value.cpuPercentOfHost, ram = value.workingSetMb;
  if (value.status === "UNKNOWN") {
    if (cpu !== null || ram !== null) throw new TypeError("Missing data must be null, not zero.");
  } else if (!finite(cpu) || cpu < 0 || !finite(ram) || ram < 0) {
    throw new TypeError("Observed CPU/RAM must be finite nonnegative values.");
  }
}
// Experimental Windows GPU performance counter hints are NOT verified
// renderer utilization, actual VRAM allocation, FPS or frametimes.
// Windows has documented stale GPU Process Memory counter values.
function validateGpuHint(value) {
  if (!exactly(value, [
    "status", "threeDEngineMaximumInstancePercent",
    "dedicatedMemoryMaximumInstanceMb",
  ]) || !["OBSERVED", "UNKNOWN"].includes(value.status)) {
    throw new TypeError("Invalid optional GPU diagnostic hint.");
  }
  const engine = value.threeDEngineMaximumInstancePercent;
  const dedicated = value.dedicatedMemoryMaximumInstanceMb;
  if (value.status === "UNKNOWN") {
    if (engine !== null || dedicated !== null) {
      throw new TypeError("Unknown GPU hints must use null, not zero.");
    }
  } else if (!finite(engine) || engine < 0 || engine > 100 ||
      !finite(dedicated) || dedicated < 0 || dedicated > 1048576) {
    throw new TypeError("Invalid GPU counter hint value.");
  }
}
function validateGpuDiagnostic(value, keys = ["native", "isolated"]) {
  if (!exactly(value, ["evidenceClass", ...keys]) ||
      value.evidenceClass !== "WINDOWS_GPU_COUNTER_HINT_NOT_FRAME_PROOF") {
    throw new TypeError("Unrecognized optional GPU evidence.");
  }
  for (const key of keys) validateGpuHint(value[key]);
}

function metrics(samples, key) {
  // This is ONE operator-selected process, not its full descendant tree.
  // Missing readings may mean exit, inaccessibility or restart, not a crash.
  const observed = samples.filter(sample => sample[key].status === "OBSERVED");
  const firstMissing = samples.find(sample => sample[key].status === "UNKNOWN");
  const mean = (field) => observed.length
    ? Number((observed.reduce((total, sample) => total + sample[key][field], 0) /
        observed.length).toFixed(2))
    : null;
  return {
    observedSamples: observed.length,
    missingSamples: samples.length - observed.length,
    observationCoveragePercent: Number((100 * observed.length / samples.length).toFixed(2)),
    firstMissingAtSeconds: firstMissing?.elapsedSeconds ?? null,
    meanHostCpuPercent: mean("cpuPercentOfHost"),
    peakHostCpuPercent: observed.length
      ? Math.max(...observed.map(sample => sample[key].cpuPercentOfHost))
      : null,
    meanWorkingSetMb: mean("workingSetMb"),
    maxWorkingSetMb: observed.length
      ? Math.max(...observed.map(sample => sample[key].workingSetMb))
      : null,
  };
}

export function evaluateNativePerformance(report) {
  if ([3, 4].includes(report?.schema)) return evaluateMultiClientPerformance(report);
  const fields = ["schema", "scope", "evidenceOrigin", "attribution",
    "automaticVerdict", "capturedAtUtc", "durationRequestedSeconds",
    "durationObservedSeconds", "sampleIntervalSeconds", "hostLogicalCpus",
    "sampleCount", "nativeProcessIdentityLost", "isolatedProcessIdentityLost",
    "samples", "missingEvidence",
    ...(report?.schema === 2 ? ["gpuDiagnostic"] : [])];
  if (!exactly(report, fields) || ![1, 2].includes(report.schema) ||
      report.scope !== "NATIVE_SANDBOXIE_PROCESS_PERFORMANCE" ||
      report.evidenceOrigin !== "WINDOWS_HOST" ||
      report.attribution !== "OPERATOR_SELECTED_UNVERIFIED_PIDS" ||
      report.automaticVerdict !== "NOT_PROVIDED" ||
      typeof report.capturedAtUtc !== "string" ||
      !Number.isFinite(Date.parse(report.capturedAtUtc)) ||
      !integer(report.durationRequestedSeconds) ||
      report.durationRequestedSeconds < 30 || report.durationRequestedSeconds > 7200 ||
      !finite(report.durationObservedSeconds) || report.durationObservedSeconds < 0 ||
      !integer(report.sampleIntervalSeconds) || report.sampleIntervalSeconds < 1 ||
      report.sampleIntervalSeconds > 60 ||
      !integer(report.hostLogicalCpus) || report.hostLogicalCpus < 1 ||
      report.hostLogicalCpus > 1024 ||
      typeof report.nativeProcessIdentityLost !== "boolean" ||
      typeof report.isolatedProcessIdentityLost !== "boolean" ||
      !integer(report.sampleCount) || report.sampleCount < 1 ||
      report.sampleCount > 7200 ||
      !Array.isArray(report.samples) || report.samples.length !== report.sampleCount ||
      !Array.isArray(report.missingEvidence) ||
      report.missingEvidence.length !== missing.length ||
      !missing.every((flag, index) => report.missingEvidence[index] === flag)) {
    throw new TypeError("Unrecognized or inconsistent native performance report.");
  }
  if (report.schema === 2) validateGpuDiagnostic(report.gpuDiagnostic);
  let lastTime = -1;
  let nativeLostInSamples = false;
  let isolatedLostInSamples = false;
  for (const sample of report.samples) {
    if (!exactly(sample, ["elapsedSeconds", "native", "isolated"]) ||
        !finite(sample.elapsedSeconds) || sample.elapsedSeconds <= lastTime ||
        sample.elapsedSeconds < 0 ||
        sample.elapsedSeconds > report.durationObservedSeconds + 0.1) {
      throw new TypeError("Invalid or unordered sample timestamp.");
    }
    lastTime = sample.elapsedSeconds;
    sampleMetrics(sample.native);
    sampleMetrics(sample.isolated);
    // The process collector pins creation time and never follows a reused PID.
    // Once identity is lost, a later OBSERVED sample would be contradictory.
    if ((nativeLostInSamples && sample.native.status === "OBSERVED") ||
        (isolatedLostInSamples && sample.isolated.status === "OBSERVED")) {
      throw new TypeError("A lost process identity cannot resume within one capture.");
    }
    nativeLostInSamples ||= sample.native.status === "UNKNOWN";
    isolatedLostInSamples ||= sample.isolated.status === "UNKNOWN";
  }
  if (nativeLostInSamples !== report.nativeProcessIdentityLost ||
      isolatedLostInSamples !== report.isolatedProcessIdentityLost) {
    throw new TypeError("Process identity-loss flags contradict the observed samples.");
  }
  const native = metrics(report.samples, "native");
  const isolated = metrics(report.samples, "isolated");
  return {
    interpretation: "PROCESS_TELEMETRY_ONLY",
    automaticGameplayProof: false,
    automaticPerformanceVerdict: "NOT_PROVIDED",
    fourClientReady: false,
    durationRequestedSeconds: report.durationRequestedSeconds,
    durationObservedSeconds: report.durationObservedSeconds,
    sampleCount: report.sampleCount,
    native,
    isolated,
    identityDiscontinuityObserved: report.nativeProcessIdentityLost || report.isolatedProcessIdentityLost,
    identityDiscontinuityInterpretation: "PROCESS_MISSING_OR_INACCESSIBLE_NOT_PROOF_OF_CRASH",
    metricScope: "OPERATOR_SELECTED_PROCESSES_ONLY_NOT_PROCESS_TREES",
    ...(report.schema === 2 ? {
      gpuDiagnostic: {
        interpretation: "ONE_TIME_WINDOWS_COUNTER_HINT_UNVERIFIED",
        native: { ...report.gpuDiagnostic.native },
        isolated: { ...report.gpuDiagnostic.isolated },
      },
      gpuPerformanceVerified: false,
      gpuCounterMemoryMayBeStale: true,
      frameTimingVerified: false,
    } : {}),
    missingEvidence: [...missing],
  };
}

// Reuse the strict two-process validator; all rows belong to one capture.
function evaluateMultiClientPerformance(report) {
  const five = report.schema === 4;
  const keys = ['native', 'isolated', 'client03', 'client04', ...(five ? ['client05'] : [])];
  const extraFlags = ['client03ProcessIdentityLost', 'client04ProcessIdentityLost',
    ...(five ? ['client05ProcessIdentityLost'] : [])];
  if (report.scope !== (five ? 'FIVE_CLIENT_PROCESS_PERFORMANCE' : 'FOUR_CLIENT_PROCESS_PERFORMANCE') ||
      extraFlags.some(key => typeof report[key] !== 'boolean') ||
      !Array.isArray(report.samples) || report.samples.length > 7200) {
    throw new TypeError('Invalid multi-client process capture.');
  }
  for (const row of report.samples) {
    if (!exactly(row, ['elapsedSeconds', ...keys])) {
      throw new TypeError('Multi-client sample must contain exactly its selected process readings.');
    }
  }
  const base = {...report};
  for (const flag of extraFlags) delete base[flag];
  if (five) {
    if (!Object.hasOwn(report, 'gpuDiagnostic')) {
      throw new TypeError('Five-client capture must record whether GPU hints were requested.');
    }
    if (report.gpuDiagnostic !== null) validateGpuDiagnostic(report.gpuDiagnostic, keys);
    delete base.gpuDiagnostic;
  }
  const lost = key => report[key + 'ProcessIdentityLost'];
  const pair = (first, second) => evaluateNativePerformance({
    ...base, schema: 1, scope: 'NATIVE_SANDBOXIE_PROCESS_PERFORMANCE',
    nativeProcessIdentityLost: lost(first), isolatedProcessIdentityLost: lost(second),
    samples: report.samples.map(row => ({elapsedSeconds: row.elapsedSeconds,
      native: row[first], isolated: row[second]})),
  });
  const first = pair('native', 'isolated');
  const second = pair('client03', 'client04');
  const fifth = five ? pair('client05', 'native') : null;
  if (five && keys.some(lost) && report.gpuDiagnostic !== null &&
      keys.some(key => report.gpuDiagnostic[key].status !== 'UNKNOWN')) {
    throw new TypeError('A capture with identity loss must skip all GPU hints.');
  }
  return {
    interpretation: 'PROCESS_TELEMETRY_ONLY', automaticGameplayProof: false,
    automaticPerformanceVerdict: 'NOT_PROVIDED', fourClientReady: false,
    ...(five ? {fiveClientReady: false} : {}),
    durationRequestedSeconds: first.durationRequestedSeconds,
    durationObservedSeconds: first.durationObservedSeconds, sampleCount: first.sampleCount,
    clients: { '01': first.native, '02': first.isolated,
      '03': second.native, '04': second.isolated, ...(five ? {'05': fifth.native} : {}) },
    identityDiscontinuityObserved: first.identityDiscontinuityObserved ||
      second.identityDiscontinuityObserved || (fifth?.identityDiscontinuityObserved ?? false),
    identityDiscontinuityInterpretation: first.identityDiscontinuityInterpretation,
    metricScope: first.metricScope,
    ...(five ? {
      gpuDiagnostic: report.gpuDiagnostic === null ? null : {
        interpretation: 'ONE_TIME_WINDOWS_COUNTER_HINT_UNVERIFIED',
        ...Object.fromEntries(keys.map((key, i) => [String(i + 1).padStart(2, '0'), {...report.gpuDiagnostic[key]}])),
      },
      gpuPerformanceVerified: false, gpuCounterMemoryMayBeStale: true,
      frameTimingVerified: false,
    } : {}),
    missingEvidence: [...missing],
  };
}


/**
 * Compare two process-only captures without claiming same host/workload,
 * full sandbox process-tree cost, FPS or successful Minecraft optimization.
 * Results are withheld for mismatched captures or any lost process identity.
 */
export function compareNativePerformance(baseline, candidate) {
  if ([3, 4].includes(baseline?.schema) || [3, 4].includes(candidate?.schema)) {
    throw new TypeError('Multi-client captures support individual review only; paired comparison requires the same two-client schema.');
  }
  const before = evaluateNativePerformance(baseline);
  const after = evaluateNativePerformance(candidate);
  const conditions = {
    sameReportSchema: baseline.schema === candidate.schema,
    equalRequestedDuration: baseline.durationRequestedSeconds === candidate.durationRequestedSeconds,
    equalSampleInterval: baseline.sampleIntervalSeconds === candidate.sampleIntervalSeconds,
    equalLogicalCpus: baseline.hostLogicalCpus === candidate.hostLogicalCpus,
    equalSampleCount: baseline.sampleCount === candidate.sampleCount,
    similarObservedDuration: Math.abs(
      baseline.durationObservedSeconds - candidate.durationObservedSeconds
    ) <= baseline.sampleIntervalSeconds,
    noIdentityLoss: !before.identityDiscontinuityObserved && !after.identityDiscontinuityObserved,
    completeProcessReadings: [before.native, before.isolated, after.native, after.isolated]
      .every(item => item.missingSamples === 0),
  };
  const eligibleForNumericDifference = Object.values(conditions).every(Boolean);
  const difference = (a, b) => Number((b - a).toFixed(2));
  const perProcess = (first, second) => eligibleForNumericDifference ? {
    meanHostCpuPercentagePointDifference: difference(first.meanHostCpuPercent, second.meanHostCpuPercent),
    peakHostCpuPercentagePointDifference: difference(first.peakHostCpuPercent, second.peakHostCpuPercent),
    meanWorkingSetMbDifference: difference(first.meanWorkingSetMb, second.meanWorkingSetMb),
    maxWorkingSetMbDifference: difference(first.maxWorkingSetMb, second.maxWorkingSetMb),
  } : null;
  return {
    interpretation: "NUMERIC_PROCESS_COMPARISON_ONLY",
    performanceVerdict: "NOT_PROVIDED",
    improvementVerified: false,
    sameHostVerified: false,
    sameWorkloadVerified: false,
    fullProcessTreeMeasured: false,
    conditions,
    comparableForNumericDifference: eligibleForNumericDifference,
    differences: eligibleForNumericDifference ? {
      native: perProcess(before.native, after.native),
      isolated: perProcess(before.isolated, after.isolated),
    } : null,
    baseline: before,
    candidate: after,
  };
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  try {
    if (![3, 4].includes(process.argv.length)) {
      throw new TypeError("Usage: node evaluate-native-performance.mjs <report.json> [candidate-report.json]");
    }
    // Historical Windows PowerShell 5.1 Set-Content -Encoding UTF8
    // emitted a BOM. Normalize only that encoding marker; all JSON
    // schema/evidence validation remains unchanged.
    const readReport = path => JSON.parse(readFileSync(path, "utf8").replace(/^\uFEFF/, ""));
    const baseline = readReport(process.argv[2]);
    const result = process.argv.length === 4
      ? compareNativePerformance(baseline, readReport(process.argv[3]))
      : evaluateNativePerformance(baseline);
    process.stdout.write(JSON.stringify(result, null, 2) + "\n");
  } catch (error) {
    process.stderr.write("Performance evidence invalid: " + error.message + "\n");
    process.exitCode = 1;
  }
}
