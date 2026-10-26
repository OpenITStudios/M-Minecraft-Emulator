import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

// Diagnostic-only review. Synthetic fixture JSON is self-reported data,
// not an attested Windows proof or permission to launch Minecraft.
// Reject extra fields so raw SIDs, paths and other private data cannot be
// accidentally carried into the sanitized diagnostic response.
export function evaluateAppContainerIsolation(report) {
  const invalid = () => { throw new TypeError("Invalid synthetic AppContainer proof; no candidate decision can be made."); };
  if (!report || typeof report !== "object" || Array.isArray(report)) invalid();
  const booleanFields = [
    "syntheticOnly", "distinctAppContainerTokens", "concurrentProcesses",
    "privateMutexNamespaces", "ownStorageReadable", "peerStorageDenied",
    "hostCanaryDenied", "independentNormalStop", "profilesRemoved",
    "localAppDataEnvironmentRedirected", "localAppDataKnownFolderRedirected",
    "roamingEnvironmentUnderContainer", "roamingKnownFolderQuerySucceeded",
    "roamingKnownFolderUnderContainer", "syntheticPrivateRoamingWritten",
    "syntheticPrivateRoamingPeerDenied", "syntheticPrivateRoamingPersistsAfterRestart"
  ];
  const allowed = new Set(["schema", "result", "minecraftRoamingCompatibility", "minecraftAcceptance", "processPaths", ...booleanFields]);
  if (Object.keys(report).length !== allowed.size ||
      Object.keys(report).some(key => !allowed.has(key)) ||
      report.schema !== 2 || report.result !== "PASS" ||
      report.minecraftRoamingCompatibility !== "NOT_EXECUTED" ||
      report.minecraftAcceptance !== "NOT_EXECUTED" ||
      booleanFields.some(key => typeof report[key] !== "boolean") ||
      (report.roamingKnownFolderUnderContainer && !report.roamingKnownFolderQuerySucceeded)) invalid();
  // Validate each child's own path observations separately. Do not infer
  // PID/FILETIME from A/B fixture labels or promote path strings to access proof.
  const pathKeys = ["fixture", "localKnownFolder", "roamingEnvironment", "roamingKnownFolder"];
  const states = new Set(["UNDER_EXPECTED_PROFILE", "OUTSIDE_EXPECTED_PROFILE", "UNAVAILABLE"]);
  if (!Array.isArray(report.processPaths) || report.processPaths.length !== 2 ||
      report.processPaths.some((p, i) => !p || typeof p !== "object" ||
        Array.isArray(p) || Object.keys(p).length !== pathKeys.length ||
        Object.keys(p).some(key => !pathKeys.includes(key)) ||
        p.fixture !== ["A", "B"][i] ||
        p.localKnownFolder === "UNAVAILABLE" ||
        !states.has(p.localKnownFolder) ||
        !states.has(p.roamingEnvironment) ||
        !states.has(p.roamingKnownFolder))) invalid();
  const bothUnder = key => report.processPaths.every(p => p[key] === "UNDER_EXPECTED_PROFILE");
  if (report.localAppDataKnownFolderRedirected !== bothUnder("localKnownFolder") ||
      report.roamingEnvironmentUnderContainer !== bothUnder("roamingEnvironment") ||
      report.roamingKnownFolderQuerySucceeded !==
        report.processPaths.every(p => p.roamingKnownFolder !== "UNAVAILABLE") ||
      report.roamingKnownFolderUnderContainer !== bothUnder("roamingKnownFolder")) invalid();

  const syntheticRequired = [
    "syntheticOnly", "distinctAppContainerTokens", "concurrentProcesses",
    "privateMutexNamespaces", "ownStorageReadable", "peerStorageDenied",
    "hostCanaryDenied", "independentNormalStop", "profilesRemoved",
    "syntheticPrivateRoamingWritten", "syntheticPrivateRoamingPeerDenied",
    "syntheticPrivateRoamingPersistsAfterRestart"
  ];
  if (syntheticRequired.some(key => report[key] !== true)) invalid();

  const risks = [];
  if (!report.localAppDataEnvironmentRedirected) risks.push("LOCAL_APPDATA_ENV_NOT_REDIRECTED");
  if (!report.localAppDataKnownFolderRedirected) risks.push("LOCAL_APPDATA_KNOWN_FOLDER_NOT_REDIRECTED");
  if (!report.roamingEnvironmentUnderContainer) risks.push("ROAMING_ENV_OUTSIDE_SYNTHETIC_PROFILE");
  if (!report.roamingKnownFolderQuerySucceeded) {
    risks.push("ROAMING_KNOWN_FOLDER_NOT_OBSERVED");
  } else if (!report.roamingKnownFolderUnderContainer) {
    risks.push("ROAMING_KNOWN_FOLDER_OUTSIDE_SYNTHETIC_PROFILE");
  }
  const pathObservation = !report.roamingKnownFolderQuerySucceeded
    ? "INCOMPLETE"
    : report.roamingEnvironmentUnderContainer && report.roamingKnownFolderUnderContainer
      ? "PRIVATE_PATHS_REPORTED_ONLY" : "NOT_FULLY_PRIVATE_PATHS_REPORTED";

  // Candidate-only mechanism decision, derived afresh from each disposable
  // synthetic receipt. No durable readiness bit and no replacement for the
  // canonical desktop Launch/Provision gate.
  //
  // Reject every *observed* Local/Roaming boundary escape. A query not
  // performed is unknown, not a successful mechanism. Prefer the Known
  // Folder failure to preserve the earlier Windows pilot diagnosis.
  // This remains an untrusted synthetic candidate diagnostic, never launch.
  const candidateDisposition =
    report.roamingKnownFolderQuerySucceeded && !report.roamingKnownFolderUnderContainer
      ? "NO_GO_OBSERVED_ROAMING_BOUNDARY"
      : !report.roamingEnvironmentUnderContainer
        ? "NO_GO_OBSERVED_ROAMING_ENV_BOUNDARY"
        : !report.localAppDataKnownFolderRedirected
          ? "NO_GO_OBSERVED_LOCAL_KNOWN_FOLDER_BOUNDARY"
          : !report.localAppDataEnvironmentRedirected
            ? "NO_GO_OBSERVED_LOCAL_ENV_BOUNDARY"
            : "UNVERIFIED_NO_GAME_ACCEPTANCE";
  const mechanismBoundaries = {
    localKnownFolder: report.localAppDataKnownFolderRedirected
      ? "PATH_REPORTED_ONLY" : "NOT_PRIVATE_IN_SAMPLE",
    roamingEnvironment: report.roamingEnvironmentUnderContainer
      ? "PATH_REPORTED_ONLY" : "NOT_PRIVATE_IN_SAMPLE",
    roamingKnownFolder: !report.roamingKnownFolderQuerySucceeded
      ? "NOT_OBSERVED"
      : report.roamingKnownFolderUnderContainer
        ? "PATH_REPORTED_ONLY" : "NOT_PRIVATE_IN_SAMPLE",
    hkcuRegistry: "NOT_OBSERVED",
    microsoftBroker: "NOT_OBSERVED",
    minecraftGameFilesystem: "NOT_EXECUTED"
  };
  return {
    schema: 1,
    scope: "INDEPENDENT_APPCONTAINER_SYNTHETIC_REVIEW",
    inputClass: "UNATTESTED_SYNTHETIC_FIXTURE_JSON",
    syntheticIsolation: "REPORTED_PASS",
    syntheticPrivateStorageAcrossProcessRestart: "REPORTED_PASS",
    roamingPathObservation: pathObservation,
    candidateDisposition,
    mechanismBoundaries,
    processPaths: report.processPaths.map(p => ({
      fixture: p.fixture,
      localKnownFolder: p.localKnownFolder,
      roamingEnvironment: p.roamingEnvironment,
      roamingKnownFolder: p.roamingKnownFolder
    })),
    processIdentityBound: false,
    knownFolderDataAccessProven: false,
    educationAuthenticationVerified: false,
    productionProviderSelected: false,
    productAcceptance: "NOT_EXECUTED",
    isolatedLaunchAllowed: false,
    risks
  };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    if (process.argv.length !== 3) {
      throw new TypeError("Usage: node evaluate-appcontainer-isolation.mjs <synthetic-result.json>");
    }
    const report = JSON.parse(readFileSync(resolve(process.argv[2]), "utf8"));
    console.log(JSON.stringify(evaluateAppContainerIsolation(report), null, 2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
