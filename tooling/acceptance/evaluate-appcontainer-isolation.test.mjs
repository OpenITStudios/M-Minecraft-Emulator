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
  assert.equal(result.candidateDisposition, "NO_GO_OBSERVED_ROAMING_BOUNDARY");
  assert.deepEqual(result.mechanismBoundaries, {
    localKnownFolder: "PATH_REPORTED_ONLY",
    roamingEnvironment: "NOT_PRIVATE_IN_SAMPLE",
    roamingKnownFolder: "NOT_PRIVATE_IN_SAMPLE",
    hkcuRegistry: "NOT_OBSERVED",
    microsoftBroker: "NOT_OBSERVED",
    minecraftGameFilesystem: "NOT_EXECUTED"
  });
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
  assert.equal(result.candidateDisposition, "UNVERIFIED_NO_GAME_ACCEPTANCE");
  assert.equal(result.mechanismBoundaries.roamingKnownFolder, "PATH_REPORTED_ONLY");
  assert.equal(result.mechanismBoundaries.hkcuRegistry, "NOT_OBSERVED");
  assert.equal(result.mechanismBoundaries.microsoftBroker, "NOT_OBSERVED");
  assert.equal(result.knownFolderDataAccessProven, false);
  assert.equal(result.isolatedLaunchAllowed, false);
});

test("roaming mechanism blocks candidate even if synthetic private roaming persistence passed", () => {
  const source = fixture();
  // Synthetic private storage persistence is a separate mechanism from
  // SHGetKnownFolderPath(FOLDERID_RoamingAppData) used by normal applications.
  source.syntheticPrivateRoamingPersistsAfterRestart = true;
  source.roamingEnvironmentUnderContainer = true;
  for (const child of source.processPaths) {
    child.roamingEnvironment = "UNDER_EXPECTED_PROFILE";
  }
  const review = evaluateAppContainerIsolation(source);
  assert.equal(review.syntheticPrivateStorageAcrossProcessRestart, "REPORTED_PASS");
  assert.equal(review.candidateDisposition, "NO_GO_OBSERVED_ROAMING_BOUNDARY");
  assert.equal(review.mechanismBoundaries.roamingEnvironment, "PATH_REPORTED_ONLY");
  assert.equal(review.mechanismBoundaries.roamingKnownFolder, "NOT_PRIVATE_IN_SAMPLE");
  assert.equal(review.mechanismBoundaries.hkcuRegistry, "NOT_OBSERVED");
  assert.equal(review.educationAuthenticationVerified, false);
  assert.equal(review.isolatedLaunchAllowed, false);
});

test("unobserved known folder stays unverified if observed paths are private", () => {
  const report = fixture();
  report.roamingKnownFolderQuerySucceeded = false;
  report.processPaths[0].roamingKnownFolder = "UNAVAILABLE";
  report.roamingEnvironmentUnderContainer = true;
  for (const child of report.processPaths) {
    child.roamingEnvironment = "UNDER_EXPECTED_PROFILE";
  }
  const result = evaluateAppContainerIsolation(report);
  assert.equal(result.roamingPathObservation, "INCOMPLETE");
  assert.equal(result.candidateDisposition, "UNVERIFIED_NO_GAME_ACCEPTANCE");
  assert.equal(result.mechanismBoundaries.roamingKnownFolder, "NOT_OBSERVED");
  assert.ok(result.risks.includes("ROAMING_KNOWN_FOLDER_NOT_OBSERVED"));
});

test("missing known-folder cannot hide an escaped roaming environment", () => {
  const report = fixture();
  report.roamingKnownFolderQuerySucceeded = false;
  report.processPaths[0].roamingKnownFolder = "UNAVAILABLE";
  const result = evaluateAppContainerIsolation(report);
  assert.equal(result.candidateDisposition, "NO_GO_OBSERVED_ROAMING_ENV_BOUNDARY");
  assert.equal(result.mechanismBoundaries.roamingKnownFolder, "NOT_OBSERVED");
  assert.equal(result.isolatedLaunchAllowed, false);
});

test("private known-folder cannot hide a roaming environment escape", () => {
  const report = fixture();
  report.roamingKnownFolderUnderContainer = true;
  for (const child of report.processPaths) {
    child.roamingKnownFolder = "UNDER_EXPECTED_PROFILE";
  }
  const result = evaluateAppContainerIsolation(report);
  assert.equal(result.candidateDisposition, "NO_GO_OBSERVED_ROAMING_ENV_BOUNDARY");
  assert.equal(result.mechanismBoundaries.roamingKnownFolder, "PATH_REPORTED_ONLY");
  assert.equal(result.isolatedLaunchAllowed, false);
});

test("LocalAppData known-folder or environment escapes reject the candidate", () => {
  const base = fixture();
  base.roamingEnvironmentUnderContainer = true;
  base.roamingKnownFolderUnderContainer = true;
  for (const child of base.processPaths) {
    child.roamingEnvironment = "UNDER_EXPECTED_PROFILE";
    child.roamingKnownFolder = "UNDER_EXPECTED_PROFILE";
    child.localKnownFolder = "OUTSIDE_EXPECTED_PROFILE";
  }
  base.localAppDataKnownFolderRedirected = false;
  const known = evaluateAppContainerIsolation(base);
  assert.equal(known.candidateDisposition, "NO_GO_OBSERVED_LOCAL_KNOWN_FOLDER_BOUNDARY");
  assert.equal(known.isolatedLaunchAllowed, false);
  base.localAppDataKnownFolderRedirected = true;
  for (const child of base.processPaths) child.localKnownFolder = "UNDER_EXPECTED_PROFILE";
  base.localAppDataEnvironmentRedirected = false;
  const environment = evaluateAppContainerIsolation(base);
  assert.equal(environment.candidateDisposition, "NO_GO_OBSERVED_LOCAL_ENV_BOUNDARY");
  assert.equal(environment.isolatedLaunchAllowed, false);
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
  // Avoid false positives: OUTSIDE_EXPECTED_PROFILE legitimately contains
  // "SID". Test the exact reviewed output shape and its nested path states,
  // not arbitrary substrings that can occur inside public enum labels.
  assert.deepEqual(Object.keys(report).sort(), [
    "schema", "scope", "inputClass", "syntheticIsolation",
    "syntheticPrivateStorageAcrossProcessRestart", "roamingPathObservation",
    "candidateDisposition", "mechanismBoundaries", "processPaths",
    "processIdentityBound", "knownFolderDataAccessProven",
    "educationAuthenticationVerified", "productionProviderSelected",
    "productAcceptance", "isolatedLaunchAllowed", "risks"
  ].sort());
  for (const child of report.processPaths) {
    assert.deepEqual(Object.keys(child).sort(), [
      "fixture", "localKnownFolder", "roamingEnvironment", "roamingKnownFolder"
    ].sort());
    for (const field of ["localKnownFolder", "roamingEnvironment", "roamingKnownFolder"]) {
      assert.ok([
        "UNDER_EXPECTED_PROFILE", "OUTSIDE_EXPECTED_PROFILE", "UNAVAILABLE"
      ].includes(child[field]));
    }
  }
  for (const sensitive of ['"rawSid":', '"sid":', '"profilePath":',
    '"userName":', '"credential":', '"token":', '"accountId":']) {
    assert.equal(JSON.stringify(report).includes(sensitive), false);
  }
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
