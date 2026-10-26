import test from "node:test";
import assert from "node:assert/strict";
import { createDisposableProvisionPlan, evaluateProvisionReceipt } from "./provision-disposable.mjs";

const hash = "a".repeat(64);
const makePayload = () => ({
  schema: 1, state: "ARTIFACTS_STAGED_NOT_INSTALLABLE",
  upstreamRepository: "sandboxie-plus/Sandboxie",
  upstreamTag: "v1.18.5", upstreamCommit: "abd4c240031cfb46666daa74ca47ed4e48b561e4",
  windowsKernelDriverLoaded: false, minecraftGameplayVerified: false,
  sandboxieExternalDependencyRemoved: false, publicDistributionApproved: false,
  artifacts: ["SbieDrv.sys", "SbieSvc.exe", "SbieDll.dll", "SbieMsg.dll",
    "kmdutil.exe", "32/SbieDll.dll", "32/SbieSvc.exe",
    "LICENSES/Sandboxie/COPYING", "LICENSES/LICENSE.Plus"]
    .map(path => ({ path, sizeBytes: 4096, sha256: hash, source: "Sandboxie" })),
});
const host = () => ({
  schema: 1, state: "BLOCKED_UNVERIFIED_OWNED_DRIVER",
  interactiveSession: true, serviceRegistration: "ABSENT", driverRegistration: "ABSENT",
  serviceNamespaceAvailable: true, providerSelected: false, trustedDriverLoaded: false,
  activationAllowed: false,
});
const target = () => ({
  kind: "DEDICATED_DISPOSABLE_WINDOWS_VM", hasAccountBearingBox: false,
  containsOfficialSandboxie: false, allowProductionInstall: false,
});
const plan = (p = makePayload(), h = host(), t = target()) =>
  createDisposableProvisionPlan({ payload: p, host: h, target: t });

test("upstream KmdUtil kernel/service sequencing and reverse-order rollback", () => {
  const result = plan();
  assert.equal(result.state, "SOURCE_PLAN_ONLY_DRIVER_INSTALL_BLOCKED");
  assert.equal(result.installSteps.length, 7);
  assert.equal(result.rollbackSteps.length, 5);
  assert.equal(result.installSteps[2].component, "SbieDrv");
  assert.equal(result.installSteps[2].altitude, "86900");
  assert.equal(result.installSteps[2].helper, "kmdutil.exe");
  assert.equal(result.installSteps[4].component, "SbieSvc");
  assert.equal(result.rollbackSteps[0].component, "SbieSvc");
  assert.equal(result.rollbackSteps[2].component, "SbieDrv");
  for (const key of ["installationPermitted", "codeSignatureVerified",
    "disposableHostVerified", "providerSelected", "driverLoaded", "gameplayVerified"]) {
    assert.equal(result[key], false);
  }
});

test("refuses present or unknown service/driver names", () => {
  for (const changed of [
    { serviceRegistration: "PRESENT", state: "BLOCKED_SERVICE_NAMESPACE_IN_USE" },
    { driverRegistration: "PRESENT", state: "BLOCKED_SERVICE_NAMESPACE_IN_USE" },
    { driverRegistration: "UNKNOWN", state: "BLOCKED_SERVICE_PROBE_UNKNOWN" },
    { interactiveSession: false, state: "BLOCKED_NON_INTERACTIVE_SESSION" },
  ]) assert.throws(() => plan(makePayload(), { ...host(), ...changed }),
    /host service\/driver namespace/);
});

test("disposable declaration is required and cannot include existing box", () => {
  for (const changed of [
    { kind: "NORMAL_WINDOWS_HOST" }, { hasAccountBearingBox: true },
    { containsOfficialSandboxie: true }, { allowProductionInstall: true },
  ]) assert.throws(() => plan(makePayload(), host(), { ...target(), ...changed }),
    /target may contain active/);
});

test("rejects missing signed-driver prerequisite and upstream provenance drift", () => {
  const missing = makePayload(); missing.artifacts.shift();
  assert.throws(() => plan(missing), /incomplete artifact manifest|missing artifact/);
  const changed = makePayload(); changed.upstreamCommit = "0".repeat(40);
  assert.throws(() => plan(changed), /unexpected source identity/);
  const overclaim = makePayload(); overclaim.windowsKernelDriverLoaded = true;
  assert.throws(() => plan(overclaim), /unexpected source identity/);
});

test("rejects duplicate, traversal and malformed artifact SHA", () => {
  const duplicate = makePayload(); duplicate.artifacts.push({...duplicate.artifacts[0]});
  assert.throws(() => plan(duplicate), /malformed or duplicated/);
  const path = makePayload(); path.artifacts[0].path = "../escape.sys";
  assert.throws(() => plan(path), /malformed or duplicated/);
  const sha = makePayload(); sha.artifacts[0].sha256 = "unknown";
  assert.throws(() => plan(sha), /malformed or duplicated/);
});

test("stops after UNKNOWN without automatic retry or service deletion", () => {
  const p = plan();
  const receipt = { schema: 1, steps: [
    { id: "01", component: "SbieDrv", outcome: "CONFIRMED" },
    { id: "02", component: "SbieDrv", outcome: "UNKNOWN" },
  ]};
  const result = evaluateProvisionReceipt(p, receipt);
  assert.equal(result.state, "STOP_MANUAL_RECONCILIATION_REQUIRED");
  assert.equal(result.canRetryAutomatically, false);
  assert.equal(result.canDeleteServiceAutomatically, false);
  assert.equal(result.installationComplete, false);
});

test("rejects operations after ambiguous state or out-of-order records", () => {
  const p = plan();
  const report = { schema: 1, steps: [
    { id: "01", component: "SbieDrv", outcome: "UNKNOWN" },
    { id: "02", component: "SbieDrv", outcome: "CONFIRMED" },
  ]};
  assert.throws(() => evaluateProvisionReceipt(p, report), /continued after/);
  const wrong = { schema: 1, steps: [{id:"R1",component:"SbieSvc",outcome:"CONFIRMED"}] };
  assert.throws(() => evaluateProvisionReceipt(p, wrong), /out-of-order/);
});

test("empty receipt is non-authoritative and cannot authorize retry", () => {
  const p = plan();
  const result = evaluateProvisionReceipt(p, { schema: 1, steps: [] });
  assert.equal(result.state, "NO_MUTATION_RECORDED");
  assert.equal(result.installationComplete, false);
  assert.equal(result.canRetryAutomatically, false);
  assert.equal(result.canDeleteServiceAutomatically, false);
});

test("a fully self-reported receipt still cannot prove native acceptance", () => {
  const p = plan();
  const receipt = { schema: 1,
    steps: p.installSteps.map(({ id, component }) => ({id,component,outcome:"CONFIRMED"})) };
  const result = evaluateProvisionReceipt(p, receipt);
  assert.equal(result.state, "AWAITING_SIGNED_RUNTIME_ACCEPTANCE");
  assert.equal(result.installationComplete, false);
  assert.equal(result.providerSelected, false);
});
