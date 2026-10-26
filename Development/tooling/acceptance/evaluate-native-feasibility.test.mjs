import assert from "node:assert/strict";
import test from "node:test";
import { evaluateNativeFeasibility } from "./evaluate-native-feasibility.mjs";

const sample = () => ({
  schema: 1,
  scope: "NATIVE_TWO_CLIENT_FEASIBILITY",
  stage: "TWO_CLIENT",
  evidenceOrigin: "WINDOWS_HOST",
  automaticVerdict: "NOT_PROVIDED",
  software: {
    desktopVersion: "1.21.120.1",
    storePackagePresent: false,
    uniqueRegisteredStartApp: true
  },
  machine: {
    interactiveSession: true,
    launchRequests: 0,
    processEvidenceClass: "NAME_MATCH_CANDIDATE_ONLY",
    candidateProcesses: [
      { pid: 101, windowHandle: "0x101", workingSetMb: 900 },
      { pid: 202, windowHandle: "0x202", workingSetMb: 950 }
    ]
  },
  operator: {
    attribution: "OPERATOR_DECLARED_NOT_MACHINE_VERIFIED",
    distinctAccounts: "YES",
    bothWorldsPlayable: "YES",
    independentInput: "YES",
    accountCrossingObserved: "NO",
    multiplayerJoined: "YES",
    reopenAndRebootPersistent: "YES",
    observedMinutes: 10,
    evidenceReference: "local-two-client-001"
  }
});

test("operator reported playability never becomes automated gameplay or account proof", () => {
  const r = evaluateNativeFeasibility(sample());
  assert.equal(r.verdict, "OPERATOR_REPORTED_TWO_CLIENTS");
  assert.equal(r.licensedAccountsVerified, false);
  assert.equal(r.automatedGameplayProof, false);
  assert.equal(r.noVmFeasibilityProven, false);
  assert.equal(r.processFeasibility, "TWO_WINDOW_CANDIDATES_OBSERVED");
  assert.equal(r.accountFeasibility, "OPERATOR_REPORTED_PLAYABLE");
  assert.equal(r.productAcceptance, "NOT_EVALUATED");
  assert.ok(!r.risks.includes("MULTIPLAYER_NOT_OBSERVED"));
  assert.ok(!r.risks.includes("SESSION_PERSISTENCE_NOT_OBSERVED"));
});

test("launch requests do not prove independent processes or playable game", () => {
  const r = sample();
  r.stage = "BASELINE";
  r.machine.launchRequests = 2;
  r.machine.candidateProcesses = [];
  for (const key of ["distinctAccounts", "bothWorldsPlayable", "independentInput",
    "accountCrossingObserved", "multiplayerJoined"]) r.operator[key] = "UNKNOWN";
  r.operator.reopenAndRebootPersistent = "UNKNOWN";
  r.operator.observedMinutes = 0;
  r.operator.evidenceReference = null;
  assert.equal(evaluateNativeFeasibility(r).verdict, "INCONCLUSIVE");
});

test("missing independent process, visible window or interactive Windows session stays unknown", () => {
  for (const alter of [
    r => r.machine.candidateProcesses.pop(),
    r => r.machine.candidateProcesses[1].windowHandle = null,
    r => r.machine.interactiveSession = false
  ]) {
    const r = sample(); alter(r);
    assert.equal(evaluateNativeFeasibility(r).verdict, "INCONCLUSIVE");
  }
});

test("Store package or ambiguous Desktop registration is not accepted", () => {
  for (const alter of [
    r => r.software.storePackagePresent = true,
    r => r.software.uniqueRegisteredStartApp = false,
    r => r.software.desktopVersion = null
  ]) {
    const r = sample(); alter(r);
    assert.equal(evaluateNativeFeasibility(r).verdict, "INCONCLUSIVE");
  }
});

test("explicit operator game failures stay failures", () => {
  for (const alter of [
    r => r.operator.distinctAccounts = "NO",
    r => r.operator.bothWorldsPlayable = "NO",
    r => r.operator.independentInput = "NO",
    r => r.operator.accountCrossingObserved = "YES",
    r => r.operator.multiplayerJoined = "NO",
    r => r.operator.reopenAndRebootPersistent = "NO"
  ]) {
    const r = sample(); alter(r);
    assert.equal(evaluateNativeFeasibility(r).verdict, "OPERATOR_REPORTED_FAILURE");
  }
});

test("ten observed minutes, clear account evidence and reference are required", () => {
  for (const alter of [
    r => r.operator.observedMinutes = 9,
    r => r.operator.evidenceReference = null,
    r => r.operator.distinctAccounts = "UNKNOWN",
    r => r.operator.accountCrossingObserved = "UNKNOWN",
    r => r.operator.multiplayerJoined = "UNKNOWN",
    r => r.operator.reopenAndRebootPersistent = "UNKNOWN"
  ]) {
    const r = sample(); alter(r);
    assert.equal(evaluateNativeFeasibility(r).verdict, "INCONCLUSIVE");
  }
});

test("reject fabricated automatic pass, duplicate identities, unsafe references", () => {
  for (const alter of [
    r => r.automaticVerdict = "PASS",
    r => r.machine.candidateProcesses[1].pid = 101,
    r => r.machine.candidateProcesses[1].windowHandle = "0x101",
    r => r.operator.evidenceReference = "C:\\Users\\private\\world",
    r => r.machine.processEvidenceClass = "VERIFIED_GAME",
    r => r.operator.attribution = "AUTOMATED",
    r => r.operator.observedMinutes = 1441,
    r => r.software.desktopVersion = "fake",
    r => r.machine.candidateProcesses[0].windowHandle = "0x0"
  ]) {
    const r = sample(); alter(r);
    assert.throws(() => evaluateNativeFeasibility(r), TypeError);
  }
});

test("baseline claims and other scopes are rejected", () => {
  const r = sample(); r.stage = "BASELINE";
  assert.throws(() => evaluateNativeFeasibility(r), TypeError);
  const s = sample(); s.scope = "CLIENT_FEASIBILITY";
  assert.throws(() => evaluateNativeFeasibility(s), TypeError);
  const t = sample(); t.machine.launchRequests = 2;
  assert.throws(() => evaluateNativeFeasibility(t), TypeError);
});

test("unknown gameplay stays inconclusive and extra processes are only candidates", () => {
  const r = sample(); r.operator.bothWorldsPlayable = "UNKNOWN";
  assert.equal(evaluateNativeFeasibility(r).verdict, "INCONCLUSIVE");
  const s = sample();
  s.machine.candidateProcesses.push({pid:303,windowHandle:null,workingSetMb:null});
  const result = evaluateNativeFeasibility(s);
  assert.equal(result.candidateProcessCount, 3);
  assert.equal(result.candidateWindowCount, 2);
});

test("previous schema-1 reports without reboot evidence stay readable but inconclusive", () => {
  const report = sample();
  delete report.operator.reopenAndRebootPersistent;
  const result = evaluateNativeFeasibility(report);
  assert.equal(result.verdict, "INCONCLUSIVE");
  assert.ok(result.risks.includes("SESSION_PERSISTENCE_NOT_OBSERVED"));
  assert.equal(result.noVmFeasibilityProven, false);
});

test("persistent sessions without multiplayer proof cannot be promoted to reported compatibility", () => {
  const report = sample();
  report.operator.multiplayerJoined = "UNKNOWN";
  const result = evaluateNativeFeasibility(report);
  assert.equal(result.verdict, "INCONCLUSIVE");
  assert.ok(result.risks.includes("MULTIPLAYER_NOT_OBSERVED"));
});

test("numeric PID/HWND identities must be unique and bounded, not just string-distinct", () => {
  for (const mutation of [
    r => r.machine.candidateProcesses[1].windowHandle = "0x0101",
    r => r.machine.candidateProcesses[1].windowHandle = "0x00000101",
    r => r.machine.candidateProcesses[1].windowHandle = "0x000",
    r => r.machine.candidateProcesses[0].windowHandle = "0x10000000000000000",
    r => r.machine.candidateProcesses[0].pid = 0x100000000,
    r => r.machine.candidateProcesses[0].pid = 1.5,
    r => r.machine.candidateProcesses[0].pid = -1,
  ]) {
    const report = sample();
    mutation(report);
    assert.throws(() => evaluateNativeFeasibility(report), TypeError);
  }
});

test("baseline rejects persistence claims and explicit multiplayer/persistence failures", () => {
  const baseline = sample();
  baseline.stage = "BASELINE";
  for (const key of ["distinctAccounts", "bothWorldsPlayable", "independentInput",
    "accountCrossingObserved", "multiplayerJoined"]) baseline.operator[key] = "UNKNOWN";
  baseline.operator.observedMinutes = 0;
  baseline.operator.evidenceReference = null;
  baseline.operator.reopenAndRebootPersistent = "YES";
  assert.throws(() => evaluateNativeFeasibility(baseline), TypeError);

  const report = sample();
  report.operator.reopenAndRebootPersistent = "NO";
  assert.equal(evaluateNativeFeasibility(report).verdict, "OPERATOR_REPORTED_FAILURE");
  assert.ok(evaluateNativeFeasibility(report).risks.includes("SESSION_PERSISTENCE_FAILED"));

  const multiplayer = sample();
  multiplayer.operator.multiplayerJoined = "NO";
  assert.equal(evaluateNativeFeasibility(multiplayer).verdict, "OPERATOR_REPORTED_FAILURE");
  assert.ok(evaluateNativeFeasibility(multiplayer).risks.includes("MULTIPLAYER_FAILED"));
});


test("native process evidence can advance without account, multiplayer or reboot claims", () => {
  const r = sample();
  r.stage = "BASELINE";
  for (const key of ["distinctAccounts", "bothWorldsPlayable", "independentInput",
    "accountCrossingObserved", "multiplayerJoined"]) r.operator[key] = "UNKNOWN";
  r.operator.reopenAndRebootPersistent = "UNKNOWN";
  r.operator.observedMinutes = 0;
  r.operator.evidenceReference = null;
  const result = evaluateNativeFeasibility(r);
  assert.equal(result.processFeasibility, "TWO_WINDOW_CANDIDATES_OBSERVED");
  assert.equal(result.accountFeasibility, "INCONCLUSIVE");
  assert.equal(result.productAcceptance, "NOT_EVALUATED");
  assert.equal(result.verdict, "INCONCLUSIVE");
  assert.equal(result.noVmFeasibilityProven, false);
});

test("account feasibility is independent of multiplayer and reboot evidence", () => {
  const r = sample();
  r.operator.multiplayerJoined = "UNKNOWN";
  r.operator.reopenAndRebootPersistent = "UNKNOWN";
  const result = evaluateNativeFeasibility(r);
  assert.equal(result.processFeasibility, "TWO_WINDOW_CANDIDATES_OBSERVED");
  assert.equal(result.accountFeasibility, "OPERATOR_REPORTED_PLAYABLE");
  assert.equal(result.verdict, "INCONCLUSIVE");
  assert.equal(result.productAcceptance, "NOT_EVALUATED");
  assert.equal(result.licensedAccountsVerified, false);
});

test("failure of networking/persistence must not erase separately reported playable accounts", () => {
  for (const key of ["multiplayerJoined", "reopenAndRebootPersistent"]) {
    const r = sample();
    r.operator[key] = "NO";
    const result = evaluateNativeFeasibility(r);
    assert.equal(result.accountFeasibility, "OPERATOR_REPORTED_PLAYABLE");
    assert.equal(result.verdict, "OPERATOR_REPORTED_FAILURE");
    assert.equal(result.productAcceptance, "NOT_EVALUATED");
  }
});

test("game/account failure prevents positive account feasibility independently", () => {
  const r = sample();
  r.operator.distinctAccounts = "NO";
  const result = evaluateNativeFeasibility(r);
  assert.equal(result.processFeasibility, "TWO_WINDOW_CANDIDATES_OBSERVED");
  assert.equal(result.accountFeasibility, "OPERATOR_REPORTED_FAILURE");
  assert.equal(result.verdict, "OPERATOR_REPORTED_FAILURE");
});

test("two Windows candidates are necessary even when operator flags claim success", () => {
  const r = sample();
  r.machine.candidateProcesses.pop();
  const result = evaluateNativeFeasibility(r);
  assert.equal(result.processFeasibility, "INCONCLUSIVE");
  assert.equal(result.accountFeasibility, "INCONCLUSIVE");
  assert.equal(result.verdict, "INCONCLUSIVE");
});
