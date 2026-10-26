import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

// Config and process membership are machine observations; Microsoft account
// isolation, Minecraft rendering and long-term persistence remain human claims.
export function evaluateSandboxieFeasibility(report) {
  const invalid = (why) => { throw new TypeError(why); };
  if (!report || typeof report !== "object" || Array.isArray(report) ||
      report.schema !== 1 || report.scope !== "SANDBOXIE_TWO_CLIENT_FEASIBILITY" ||
      report.evidenceOrigin !== "WINDOWS_HOST" ||
      report.automaticVerdict !== "NOT_PROVIDED" ||
      !["INSPECT", "TWO_CLIENT"].includes(report.stage)) {
    invalid("Unrecognized Sandboxie feasibility report.");
  }
  const box = report.sandboxie;
  // Old schema-1 reports do not identify where box-local settings came from.
  // They stay readable but cannot prove persistent configuration.
  const boxSettingsSource = box?.boxSettingsSource ?? "UNKNOWN";
  const credentialHostAccess = box?.credentialHostAccess ?? "UNKNOWN";
  const machine = report.machine;
  const operator = report.operator;
  if (!box || !machine || !operator ||
      !["toolsPresent", "toolSignaturesValid", "serviceRunning"].every(
        k => typeof box[k] === "boolean") ||
      !["YES", "NO", "UNKNOWN"].includes(box.boxConfigured) ||
      !["BOX_SECTION_ENUMERATED", "UNKNOWN"].includes(boxSettingsSource) ||
      !["ON", "OFF", "UNKNOWN"].includes(box.autoDelete) ||
      !["ON", "OFF", "UNKNOWN"].includes(box.autoRemove) ||
       !["ON", "OFF", "UNKNOWN"].includes(credentialHostAccess) ||
      !["OBSERVED", "UNKNOWN"].includes(box.membershipStatus) ||
      typeof machine.interactiveSession !== "boolean" ||
      machine.processEvidenceClass !== "NAME_MATCH_CANDIDATE_ONLY" ||
      !Array.isArray(machine.candidateProcesses) ||
      machine.candidateProcesses.length > 16 ||
      operator.attribution !== "OPERATOR_DECLARED_NOT_MACHINE_VERIFIED") {
    invalid("Unrecognized machine or operator evidence fields.");
  }

  if (box.membershipStatus === "OBSERVED" &&
      (!box.toolsPresent || !box.toolSignaturesValid ||
       !box.serviceRunning || box.boxConfigured !== "YES")) {
    invalid("Sandbox membership cannot be observed with an unavailable provider.");
  }
  if (boxSettingsSource === "BOX_SECTION_ENUMERATED" &&
      (!box.toolsPresent || !box.toolSignaturesValid ||
       !box.serviceRunning || box.boxConfigured !== "YES")) {
    invalid("Box-local settings cannot be verified without an enabled box and trusted tools.");
  }
  const seenPids = new Set();
  const seenHandles = new Set();
  let sandboxVisible = 0;
  let nativeVisible = 0;
  for (const candidate of machine.candidateProcesses) {
    if (!candidate || !Number.isSafeInteger(candidate.pid) ||
        candidate.pid < 1 || candidate.pid > 0xffffffff ||
         seenPids.has(candidate.pid) ||
        (candidate.inSelectedBox !== null &&
          typeof candidate.inSelectedBox !== "boolean") ||
        (box.membershipStatus === "UNKNOWN" && candidate.inSelectedBox !== null)) {
      invalid("Invalid, duplicate or unattributed process candidate.");
    }
    seenPids.add(candidate.pid);
    if (candidate.windowHandle !== null) {
      if (typeof candidate.windowHandle !== "string" ||
          !/^0x[0-9a-f]{1,16}$/i.test(candidate.windowHandle)) {
        invalid("Invalid candidate HWND.");
      }
      const id = BigInt(candidate.windowHandle);
      if (id === 0n || seenHandles.has(id)) invalid("Two processes cannot own one visible HWND.");
      seenHandles.add(id);
      if (candidate.inSelectedBox === true) sandboxVisible++;
      if (candidate.inSelectedBox === false) nativeVisible++;
    }
  }

  const flags = ["licensedForUse", "nativeWorldPlayable", "sandboxWorldPlayable",
    "distinctAccounts", "independentInput", "accountCrossingObserved",
    "reopenAndRebootPersistent", "multiplayerJoined"];
  if (!flags.every(k => ["YES", "NO", "UNKNOWN"].includes(operator[k])) ||
      !Number.isInteger(operator.observedMinutes) ||
      operator.observedMinutes < 0 || operator.observedMinutes > 1440 ||
      (operator.evidenceReference !== null &&
        (typeof operator.evidenceReference !== "string" ||
          !/^[A-Za-z0-9_.-]{1,80}$/.test(operator.evidenceReference)))) {
    invalid("Invalid operator statements or reference.");
  }
  if (report.stage === "INSPECT" &&
      (flags.some(k => operator[k] !== "UNKNOWN") ||
       operator.observedMinutes !== 0 || operator.evidenceReference !== null)) {
    invalid("Inspection cannot claim account or gameplay evidence.");
  }

  const risks = [];
  if (!box.toolsPresent || !box.toolSignaturesValid) {
    risks.push("SANDBOXIE_TOOL_TRUST_NOT_PROVEN");
  }
  if (!box.serviceRunning || box.boxConfigured !== "YES") {
    risks.push("SANDBOX_NOT_READY");
  }
  if (boxSettingsSource !== "BOX_SECTION_ENUMERATED") {
    risks.push("BOX_LOCAL_SETTINGS_NOT_VERIFIED");
  }
  if (box.autoDelete !== "OFF") risks.push("AUTODELETE_NOT_PROVEN_OFF");
  if (box.autoRemove !== "OFF") risks.push("AUTOREMOVE_NOT_PROVEN_OFF");
  if (credentialHostAccess !== "OFF") risks.push(
    credentialHostAccess === "ON" ? "HOST_CREDENTIAL_STORE_OPEN" : "HOST_CREDENTIAL_ISOLATION_UNKNOWN");
  if (box.membershipStatus !== "OBSERVED") risks.push("BOX_MEMBERSHIP_UNKNOWN");
  if (!machine.interactiveSession) risks.push("NON_INTERACTIVE_SESSION");
  if (sandboxVisible < 1) risks.push("SANDBOXED_GAME_WINDOW_NOT_OBSERVED");
  if (nativeVisible < 1) risks.push("NATIVE_GAME_WINDOW_NOT_OBSERVED");
  if (operator.licensedForUse !== "YES") risks.push("LICENSE_NOT_ATTESTED");
  if (operator.multiplayerJoined !== "YES") risks.push("MULTIPLAYER_NOT_CONFIRMED");
  if (operator.reopenAndRebootPersistent !== "YES") risks.push("SESSION_PERSISTENCE_NOT_CONFIRMED");

  // Separate early process/interactive-account observations from the final
  // persistence condition. All account assertions remain operator-attributed.
  const processFeasibility = machine.interactiveSession &&
    box.membershipStatus === "OBSERVED" &&
    nativeVisible >= 1 && sandboxVisible >= 1
    ? "TWO_WINDOW_CANDIDATES_OBSERVED" : "INCONCLUSIVE";
  const accountFailure = report.stage === "TWO_CLIENT" &&
    (operator.nativeWorldPlayable === "NO" ||
     operator.sandboxWorldPlayable === "NO" ||
     operator.distinctAccounts === "NO" ||
     operator.independentInput === "NO" ||
     operator.accountCrossingObserved === "YES");
  const accountFeasibility = accountFailure ? "OPERATOR_REPORTED_FAILURE" :
    report.stage === "TWO_CLIENT" &&
    processFeasibility === "TWO_WINDOW_CANDIDATES_OBSERVED" &&
    operator.licensedForUse === "YES" &&
    operator.nativeWorldPlayable === "YES" &&
    operator.sandboxWorldPlayable === "YES" &&
    operator.distinctAccounts === "YES" &&
    operator.independentInput === "YES" &&
    operator.accountCrossingObserved === "NO" &&
    operator.observedMinutes >= 10 &&
    operator.evidenceReference !== null
      ? "OPERATOR_REPORTED_PLAYABLE" : "INCONCLUSIVE";

  const configBlocked = box.autoDelete === "ON" || box.autoRemove === "ON" ||
    credentialHostAccess === "ON" ||
    box.boxConfigured === "NO";
  const operatorFailed = report.stage === "TWO_CLIENT" &&
    (operator.licensedForUse === "NO" ||
     operator.nativeWorldPlayable === "NO" ||
     operator.sandboxWorldPlayable === "NO" ||
     operator.distinctAccounts === "NO" ||
     operator.independentInput === "NO" ||
     operator.accountCrossingObserved === "YES" ||
     operator.reopenAndRebootPersistent === "NO" ||
      operator.multiplayerJoined === "NO");
  const observedCandidate = report.stage === "TWO_CLIENT" &&
    boxSettingsSource === "BOX_SECTION_ENUMERATED" &&
    box.toolsPresent && box.toolSignaturesValid && box.serviceRunning &&
    box.boxConfigured === "YES" && box.autoDelete === "OFF" &&
    credentialHostAccess === "OFF" &&
    box.autoRemove === "OFF" && box.membershipStatus === "OBSERVED" &&
    machine.interactiveSession && sandboxVisible >= 1 && nativeVisible >= 1 &&
    operator.licensedForUse === "YES" &&
    operator.nativeWorldPlayable === "YES" &&
    operator.sandboxWorldPlayable === "YES" &&
    operator.distinctAccounts === "YES" &&
    operator.independentInput === "YES" &&
    operator.accountCrossingObserved === "NO" &&
    operator.reopenAndRebootPersistent === "YES" &&
    operator.multiplayerJoined === "YES" &&
    operator.observedMinutes >= 10 && operator.evidenceReference !== null;
  const verdict = configBlocked ? "CONFIGURATION_BLOCKED" :
    operatorFailed ? "OPERATOR_REPORTED_FAILURE" :
    observedCandidate ? "OPERATOR_REPORTED_COMPATIBLE" : "INCONCLUSIVE";

  return {
    schema: 1,
    scope: report.scope,
    sourceStage: report.stage,
    verdict,
    nativeVisibleCandidates: nativeVisible,
    isolatedVisibleCandidates: sandboxVisible,
    sandboxProcessMembershipObserved: box.membershipStatus === "OBSERVED",
    boxLocalSettingsVerified: boxSettingsSource === "BOX_SECTION_ENUMERATED",
    credentialHostAccess,
    processFeasibility,
    accountFeasibility,
    productAcceptance: "NOT_EVALUATED",
    automatedAccountProof: false,
    automatedGameProof: false,
    productionFourClientReady: false,
    risks
  };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    if (process.argv.length !== 3) {
      throw new Error("Usage: node evaluate-sandboxie-feasibility.mjs <report.json>");
    }
    const report = JSON.parse(readFileSync(resolve(process.argv[2]), "utf8"));
    console.log(JSON.stringify(evaluateSandboxieFeasibility(report), null, 2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
