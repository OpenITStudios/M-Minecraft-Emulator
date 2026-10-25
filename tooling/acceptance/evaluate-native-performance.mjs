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

const processMemoryFields = ["privateCommitMb", "privateWorkingSetMb"];
const systemMemoryFields = ["totalPhysicalMb", "availablePhysicalMb", "commitTotalMb", "commitLimitMb"];
const memoryReasons = ["API_UNAVAILABLE", "QUERY_FAILED", "IDENTITY_UNAVAILABLE", "EXTENDED_COUNTER_UNAVAILABLE"];

function memoryMetric(value) {
  if (!exactly(value, ["status", "valueMb", "reason"]) ||
      !["OBSERVED", "UNKNOWN"].includes(value.status)) {
    throw new TypeError("Invalid memory metric.");
  }
  if (value.status === "UNKNOWN") {
    if (value.valueMb !== null || !memoryReasons.includes(value.reason)) {
      throw new TypeError("Unavailable memory requires null and an explicit reason.");
    }
  } else if (!finite(value.valueMb) || value.valueMb < 0 || value.reason !== null) {
    throw new TypeError("Observed memory requires finite nonnegative MiB and no error reason.");
  }
}

function memorySummary(values) {
  const observed = values.filter(value => value.status === "OBSERVED").map(value => value.valueMb);
  const sorted = [...observed].sort((a, b) => a - b);
  const round = value => Number(value.toFixed(3));
  const middle = Math.floor(sorted.length / 2);
  return {
    observedSamples: observed.length,
    missingSamples: values.length - observed.length,
    meanMb: observed.length ? round(observed.reduce((a, b) => a + b, 0) / observed.length) : null,
    medianMb: observed.length ? round(sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2) : null,
    peakMb: observed.length ? sorted.at(-1) : null,
    firstMb: values[0].valueMb,
    lastMb: values.at(-1).valueMb,
    // Never bridge unavailable endpoints to imply a continuous memory trend.
    endpointDifferenceMb: values[0].status === "OBSERVED" && values.at(-1).status === "OBSERVED"
      ? round(values.at(-1).valueMb - values[0].valueMb) : null,
    unavailableReasons: [...new Set(values.filter(value => value.status === "UNKNOWN").map(value => value.reason))],
  };
}

function baseMemoryReport(report) {
  const {baseSchema, memoryComposition, ...base} = report;
  if (![1, 2].includes(baseSchema)) throw new TypeError("Memory composition requires a two-client base schema.");
  return {...base, schema: baseSchema};
}

function evaluateMemoryComposition(report) {
  const result = evaluateNativePerformance(baseMemoryReport(report));
  const data = report.memoryComposition;
  if (!exactly(data, ["evidenceClass", "processTreeAttribution", "managerAttribution", "samples"]) ||
      data.evidenceClass !== "SELECTED_PROCESS_AND_WHOLE_SYSTEM_MEMORY_NOT_ENGINE_OVERHEAD" ||
      data.processTreeAttribution !== "NOT_COLLECTED" || data.managerAttribution !== "NOT_COLLECTED" ||
      !Array.isArray(data.samples) || data.samples.length !== report.sampleCount) {
    throw new TypeError("Invalid memory composition scope or coverage.");
  }
  let previous = -1;
  for (const [index, row] of data.samples.entries()) {
    if (!exactly(row, ["elapsedSeconds", "native", "isolated", "system", "queryDurationMs"]) ||
        !finite(row.elapsedSeconds) || row.elapsedSeconds <= previous ||
        row.elapsedSeconds < report.samples[index].elapsedSeconds ||
        !finite(row.queryDurationMs) || row.queryDurationMs < 0 ||
        row.elapsedSeconds + row.queryDurationMs / 1000 > report.durationObservedSeconds + 0.1) {
      throw new TypeError("Invalid memory sample timing.");
    }
    previous = row.elapsedSeconds;
    for (const key of ["native", "isolated", "system"]) {
      const fields = key === "system" ? systemMemoryFields : processMemoryFields;
      if (!exactly(row[key], fields)) throw new TypeError("Invalid memory sample fields.");
      for (const field of fields) {
        const metric = row[key][field];
        memoryMetric(metric);
        if (key !== "system") {
          const lost = report.samples[index][key].status === "UNKNOWN";
          if ((lost && (metric.status !== "UNKNOWN" || metric.reason !== "IDENTITY_UNAVAILABLE")) ||
              (!lost && metric.reason === "IDENTITY_UNAVAILABLE")) {
            throw new TypeError("Memory attribution contradicts selected process identity.");
          }
        } else if (["IDENTITY_UNAVAILABLE", "EXTENDED_COUNTER_UNAVAILABLE"].includes(metric.reason)) {
          throw new TypeError("Process-only reason cannot describe whole-system memory.");
        }
        if (field === "privateCommitMb" && metric.reason === "EXTENDED_COUNTER_UNAVAILABLE") {
          throw new TypeError("Extended counter availability cannot invalidate base private commit.");
        }
      }
    }
  }
  const summarize = (key, fields) => Object.fromEntries(fields.map(field =>
    [field, memorySummary(data.samples.map(row => row[key][field]))]));
  return {...result, memoryComposition: {
    interpretation: data.evidenceClass,
    processTreeMeasured: false, managerMeasured: false, engineOverheadIsolated: false,
    native: summarize("native", processMemoryFields),
    isolated: summarize("isolated", processMemoryFields),
    system: summarize("system", systemMemoryFields),
    queryMeanDurationMs: Number((data.samples.reduce((sum, row) => sum + row.queryDurationMs, 0) / data.samples.length).toFixed(3)),
    queryPeakDurationMs: Math.max(...data.samples.map(row => row.queryDurationMs)),
    queryTimingIncludes: "MEMORY_QUERIES_ONLY_NOT_ALL_COLLECTOR_CPU_OR_MEMORY",
  }};
}

export function evaluateNativePerformance(report) {
  if (report?.schema === 5) return evaluateMemoryComposition(report);
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
  if (baseline?.schema === 5 || candidate?.schema === 5) {
    if (baseline?.schema !== 5 || candidate?.schema !== 5) {
      throw new TypeError('Memory composition comparisons require two schema-5 captures.');
    }
    const before = evaluateNativePerformance(baseline);
    const after = evaluateNativePerformance(candidate);
    const compared = compareNativePerformance(baseMemoryReport(baseline), baseMemoryReport(candidate));
    const groups = [['native', processMemoryFields], ['isolated', processMemoryFields], ['system', systemMemoryFields]];
    const differences = Object.fromEntries(groups.map(([key, fields]) => [key,
      Object.fromEntries(fields.map(field => {
        const a = before.memoryComposition[key][field], b = after.memoryComposition[key][field];
        const eligible = compared.comparableForNumericDifference && a.missingSamples === 0 && b.missingSamples === 0;
        return [field, eligible ? {
          meanMbDifference: Number((b.meanMb - a.meanMb).toFixed(3)),
          medianMbDifference: Number((b.medianMb - a.medianMb).toFixed(3)),
          peakMbDifference: Number((b.peakMb - a.peakMb).toFixed(3)),
        } : null];
      }))]));
    return {...compared, baseline: before, candidate: after,
      memoryDifferences: differences, engineOverheadIsolated: false};
  }
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

