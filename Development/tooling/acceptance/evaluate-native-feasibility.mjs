import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

// This evaluator cannot certify licensed game sessions from process inventory.
export function evaluateNativeFeasibility(report) {
  const invalid = (why) => { throw new TypeError(why); };
  if (!report || typeof report !== "object" || Array.isArray(report) ||
      report.schema !== 1 || report.scope !== "NATIVE_TWO_CLIENT_FEASIBILITY" ||
      report.evidenceOrigin !== "WINDOWS_HOST" ||
      report.automaticVerdict !== "NOT_PROVIDED" ||
      !["BASELINE", "TWO_CLIENT"].includes(report.stage)) {
    invalid("Unrecognized or untrusted native feasibility report.");
  }

  const software = report.software;
  const machine = report.machine;
  const operator = report.operator;
  if (!software || !machine || !operator ||
      (software.desktopVersion !== null &&
        (typeof software.desktopVersion !== "string" ||
          !/^\d+(?:\.\d+){1,4}$/.test(software.desktopVersion))) ||
      typeof software.storePackagePresent !== "boolean" ||
      typeof software.uniqueRegisteredStartApp !== "boolean" ||
      typeof machine.interactiveSession !== "boolean" ||
      !Number.isInteger(machine.launchRequests) ||
      machine.launchRequests < 0 || machine.launchRequests > 2 ||
      machine.processEvidenceClass !== "NAME_MATCH_CANDIDATE_ONLY" ||
      !Array.isArray(machine.candidateProcesses) ||
      machine.candidateProcesses.length > 16 ||
      operator.attribution !== "OPERATOR_DECLARED_NOT_MACHINE_VERIFIED") {
    invalid("Invalid evidence provenance or machine inventory.");
  }

  const pids = new Set();
  const handles = new Set();
  let visible = 0;
  for (const p of machine.candidateProcesses) {
    if (!p || !Number.isSafeInteger(p.pid) || p.pid < 1 ||
        p.pid > 0xffffffff || pids.has(p.pid) ||
        (p.workingSetMb !== null &&
          (typeof p.workingSetMb !== "number" ||
           !Number.isFinite(p.workingSetMb) || p.workingSetMb < 0))) {
      invalid("Invalid or duplicate process candidate.");
    }
    pids.add(p.pid);
    if (p.windowHandle !== null) {
      if (typeof p.windowHandle !== "string" ||
          !/^0x[0-9a-f]{1,16}$/i.test(p.windowHandle)) {
        invalid("Invalid visible window candidate.");
      }
      const key = BigInt(p.windowHandle);
      if (key === 0n || handles.has(key)) {
        invalid("Zero or duplicate visible window candidate.");
      }
      handles.add(key);
      visible++;
    }
  }

  // Schema-1 reports captured before persistence was introduced remain
  // readable, but can no longer reach a positive compatibility verdict.
  const persistence = operator.reopenAndRebootPersistent ?? "UNKNOWN";
  const flags = ["distinctAccounts", "bothWorldsPlayable", "independentInput",
    "accountCrossingObserved", "multiplayerJoined"];
  if (!flags.every(k => ["YES", "NO", "UNKNOWN"].includes(operator[k])) ||
      !["YES", "NO", "UNKNOWN"].includes(persistence) ||
      !Number.isInteger(operator.observedMinutes) ||
      operator.observedMinutes < 0 || operator.observedMinutes > 1440 ||
      (operator.evidenceReference !== null &&
        (typeof operator.evidenceReference !== "string" ||
          !/^[A-Za-z0-9_.-]{1,80}$/.test(operator.evidenceReference)))) {
    invalid("Invalid operator declarations or evidence reference.");
  }
  if ((report.stage === "BASELINE" &&
        (flags.some(k => operator[k] !== "UNKNOWN") ||
         persistence !== "UNKNOWN" ||
         operator.observedMinutes !== 0 || operator.evidenceReference !== null)) ||
      (report.stage === "TWO_CLIENT" && machine.launchRequests !== 0)) {
    invalid("Launch requests and gameplay evidence must be separate.");
  }

  const risks = [];
  const readyDesktop = Boolean(software.desktopVersion) &&
    !software.storePackagePresent && software.uniqueRegisteredStartApp;
  if (!readyDesktop) risks.push("OFFICIAL_DESKTOP_INSTALL_NOT_UNAMBIGUOUS");
  if (!machine.interactiveSession) risks.push("NON_INTERACTIVE_SESSION");
  if (machine.candidateProcesses.length < 2) risks.push("TWO_PROCESSES_NOT_OBSERVED");
  if (visible < 2) risks.push("TWO_WINDOWS_NOT_OBSERVED");
  if (report.stage === "TWO_CLIENT" && operator.multiplayerJoined !== "YES") {
    risks.push(operator.multiplayerJoined === "NO"
      ? "MULTIPLAYER_FAILED" : "MULTIPLAYER_NOT_OBSERVED");
  }
  if (report.stage === "TWO_CLIENT" && persistence !== "YES") {
    risks.push(persistence === "NO"
      ? "SESSION_PERSISTENCE_FAILED" : "SESSION_PERSISTENCE_NOT_OBSERVED");
  }
  if (report.stage === "TWO_CLIENT" && operator.evidenceReference === null) {
    risks.push("OPERATOR_REFERENCE_MISSING");
  }

  // Three evidence levels, not three separate runtimes or lifecycle managers.
  // A candidate HWND/PID never certifies game identity; account evidence is
  // strictly operator-attributed. Four-client acceptance is outside this report.
  const processFeasibility = readyDesktop && machine.interactiveSession &&
    machine.candidateProcesses.length >= 2 && visible >= 2
    ? "TWO_WINDOW_CANDIDATES_OBSERVED" : "INCONCLUSIVE";
  const accountFailure = report.stage === "TWO_CLIENT" &&
    (operator.distinctAccounts === "NO" ||
     operator.bothWorldsPlayable === "NO" ||
     operator.independentInput === "NO" ||
     operator.accountCrossingObserved === "YES");
  const accountFeasibility = accountFailure ? "OPERATOR_REPORTED_FAILURE" :
    report.stage === "TWO_CLIENT" &&
    processFeasibility === "TWO_WINDOW_CANDIDATES_OBSERVED" &&
    operator.distinctAccounts === "YES" &&
    operator.bothWorldsPlayable === "YES" &&
    operator.independentInput === "YES" &&
    operator.accountCrossingObserved === "NO" &&
    operator.observedMinutes >= 10 &&
    operator.evidenceReference !== null
      ? "OPERATOR_REPORTED_PLAYABLE" : "INCONCLUSIVE";

  // Preserve the pre-existing verdict for completed TWO_CLIENT observations;
  // multiplayer/reboot are final acceptance criteria, not prerequisites for
  // studying account feasibility.
  const explicitFailure = accountFailure ||
    (report.stage === "TWO_CLIENT" &&
      (operator.multiplayerJoined === "NO" || persistence === "NO"));
  const observedPlayable = accountFeasibility === "OPERATOR_REPORTED_PLAYABLE" &&
    operator.multiplayerJoined === "YES" && persistence === "YES";

  return {
    schema: 1,
    scope: report.scope,
    sourceStage: report.stage,
    verdict: explicitFailure ? "OPERATOR_REPORTED_FAILURE" :
      observedPlayable ? "OPERATOR_REPORTED_TWO_CLIENTS" : "INCONCLUSIVE",
    candidateProcessCount: machine.candidateProcesses.length,
    candidateWindowCount: visible,
    processFeasibility,
    accountFeasibility,
    productAcceptance: "NOT_EVALUATED",
    licensedAccountsVerified: false,
    automatedGameplayProof: false,
    noVmFeasibilityProven: false,
    risks
  };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    if (process.argv.length !== 3) {
      throw new Error("Usage: node evaluate-native-feasibility.mjs <report.json>");
    }
    const report = JSON.parse(readFileSync(resolve(process.argv[2]), "utf8"));
    console.log(JSON.stringify(evaluateNativeFeasibility(report), null, 2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
