import assert from "node:assert/strict";
import test from "node:test";
import { evaluateAppContainerIsolation } from "./evaluate-appcontainer-isolation.mjs";

function fixture() {
  const names = [
    "syntheticOnly", "distinctAppContainerTokens", "concurrentProcesses",
    "privateMutexNamespaces", "ownStorageReadable", "peerStorageDenied",
    "hostCanaryDenied", "independentNormalStop", "profilesRemoved",
    "localAppDataEnvironmentRedirected", "localAppDataKnownFolderRedirected",
    "roamingEnvironmentUnderContainer", "roamingKnownFolderQuerySucceeded",
    "roamingKnownFolderUnderContainer", "syntheticPrivateRoamingWritten",
    "syntheticPrivateRoamingPeerDenied", "syntheticPrivateRoamingPersistsAfterRestart"
  ];
  const report = {
    schema: 2,
    result: "PASS",
    processPaths: [
      {fixture:"A",localKnownFolder:"UNDER_EXPECTED_PROFILE",roamingEnvironment:"OUTSIDE_EXPECTED_PROFILE",roamingKnownFolder:"OUTSIDE_EXPECTED_PROFILE"},
      {fixture:"B",localKnownFolder:"UNDER_EXPECTED_PROFILE",roamingEnvironment:"OUTSIDE_EXPECTED_PROFILE",roamingKnownFolder:"OUTSIDE_EXPECTED_PROFILE"}
    ],
    minecraftRoamingCompatibility: "NOT_EXECUTED",
    minecraftAcceptance: "NOT_EXECUTED"
  };
  for (const name of names) report[name] = true;
  report.roamingEnvironmentUnderContainer = false;
  report.roamingKnownFolderUnderContainer = false;
  return report;
}

test("prior host observation never makes game launch available", () => {
  const result = evaluateAppContainerIsolation(fixture());
  assert.equal(result.syntheticIsolation, "REPORTED_PASS");
  assert.equal(result.syntheticPrivateStorageAcrossProcessRestart, "REPORTED_PASS");
  assert.equal(result.roamingPathObservation, "NOT_FULLY_PRIVATE_PATHS_REPORTED");
  assert.ok(result.risks.includes("ROAMING_KNOWN_FOLDER_OUTSIDE_SYNTHETIC_PROFILE"));
  assert.equal(result.isolatedLaunchAllowed, false);
  assert.equal(result.productionProviderSelected, false);
  assert.equal(result.productAcceptance, "NOT_EXECUTED");
  assert.equal(result.educationAuthenticationVerified, false);
});

test("all paths reported under profile are still not proof of auth or real file access", () => {
  const report = fixture();
  report.roamingEnvironmentUnderContainer = true;
  report.roamingKnownFolderUnderContainer = true;
  for (const p of report.processPaths) {
    p.roamingEnvironment = "UNDER_EXPECTED_PROFILE";
    p.roamingKnownFolder = "UNDER_EXPECTED_PROFILE";
  }
  const result = evaluateAppContainerIsolation(report);
  assert.equal(result.roamingPathObservation, "PRIVATE_PATHS_REPORTED_ONLY");
  assert.equal(result.knownFolderDataAccessProven, false);
  assert.equal(result.isolatedLaunchAllowed, false);
});

test("unknown known-folder query never becomes private", () => {
  const report = fixture();
  report.roamingKnownFolderQuerySucceeded = false;
  report.processPaths[0].roamingKnownFolder = "UNAVAILABLE";
  const result = evaluateAppContainerIsolation(report);
  assert.equal(result.roamingPathObservation, "INCOMPLETE");
  assert.ok(result.risks.includes("ROAMING_KNOWN_FOLDER_NOT_OBSERVED"));
});

test("positive synthetic flags and evidence tiers must all be consistent", () => {
  const updates = [
    r => r.result = "FAIL",
    r => r.syntheticOnly = false,
    r => r.peerStorageDenied = false,
    r => r.syntheticPrivateRoamingPersistsAfterRestart = false,
    r => r.profilesRemoved = false,
    r => r.minecraftAcceptance = "PASS",
    r => r.minecraftRoamingCompatibility = "PASS",
    r => { r.roamingKnownFolderQuerySucceeded = false; r.roamingKnownFolderUnderContainer = true; },
    r => r.schema = 1,
    r => r.ownStorageReadable = "true",
    r => delete r.hostCanaryDenied,
    r => r.profilePath = "C:\\Users\\private\\world"
  ];
  for (const change of updates) {
    const report = fixture();
    change(report);
    assert.throws(() => evaluateAppContainerIsolation(report), TypeError);
  }
});

test("reject malformed inputs without leaking private fields", () => {
  for (const report of [null, [], "", false, {schema:1}]) {
    assert.throws(() => evaluateAppContainerIsolation(report), TypeError);
  }
  const report = evaluateAppContainerIsolation(fixture());
  assert.equal(JSON.stringify(report).includes("Users"), false);
  assert.equal(JSON.stringify(report).includes("SID"), false);
});

test("per-process paths remain separate and never prove launch", () => {
  const report = fixture();
  report.processPaths[1].roamingEnvironment = "UNDER_EXPECTED_PROFILE";
  const outcome = evaluateAppContainerIsolation(report);
  assert.equal(outcome.processPaths[0].roamingEnvironment, "OUTSIDE_EXPECTED_PROFILE");
  assert.equal(outcome.processPaths[1].roamingEnvironment, "UNDER_EXPECTED_PROFILE");
  assert.equal(outcome.processIdentityBound, false);
  assert.equal(outcome.isolatedLaunchAllowed, false);
});

test("contradictory child values and leaked paths fail closed", () => {
  for (const mutate of [
    r => {r.processPaths[0].fixture = "B";},
    r => {r.processPaths[1].localKnownFolder = "UNAVAILABLE";},
    r => {r.processPaths[0].rawPath = "private";},
    r => {r.processPaths.pop();},
    r => {r.processPaths[0].roamingKnownFolder = "UNAVAILABLE";},
    r => {r.processPaths[1].roamingEnvironment = "UNDER_EXPECTED_PROFILE";r.roamingEnvironmentUnderContainer = true;}
  ]) {
    const report = fixture();
    mutate(report);
    assert.throws(() => evaluateAppContainerIsolation(report), TypeError);
  }
});
