import test from "node:test";
import assert from "node:assert/strict";
import { createDisposableProvisionPlan } from "../provision-disposable.mjs";
import { compileTransportStep } from "./transport-commands.mjs";

const digest = "a".repeat(64);
function fixture() {
  const paths = ["SbieDrv.sys", "SbieSvc.exe", "SbieDll.dll", "SbieMsg.dll",
    "kmdutil.exe", "32/SbieDll.dll", "32/SbieSvc.exe",
    "LICENSES/Sandboxie/COPYING", "LICENSES/LICENSE.Plus"];
  const payload = {
    schema: 1, state: "ARTIFACTS_STAGED_NOT_INSTALLABLE",
    upstreamRepository: "sandboxie-plus/Sandboxie", upstreamTag: "v1.18.5",
    upstreamCommit: "abd4c240031cfb46666daa74ca47ed4e48b561e4",
    windowsKernelDriverLoaded: false, minecraftGameplayVerified: false,
    sandboxieExternalDependencyRemoved: false, publicDistributionApproved: false,
    artifacts: paths.map(path => ({ path, sizeBytes: 4096,
      sha256: digest, source: "Sandboxie" })),
  };
  const host = {
    schema: 1, state: "BLOCKED_UNVERIFIED_OWNED_DRIVER",
    interactiveSession: true, serviceRegistration: "ABSENT",
    driverRegistration: "ABSENT", serviceNamespaceAvailable: true,
    providerSelected: false, trustedDriverLoaded: false, activationAllowed: false,
  };
  const target = {
    kind: "DEDICATED_DISPOSABLE_WINDOWS_VM", hasAccountBearingBox: false,
    containsOfficialSandboxie: false, allowProductionInstall: false,
  };
  return createDisposableProvisionPlan({ payload, host, target });
}
const compile = (stepId, extras={}) =>
  compileTransportStep({plan: fixture(), payloadRoot: "D:\\MIVUBI\\Isolation-Payload",
    stepId, ...extras});
const owner = (component, image) => ({
  transactionId: "01234567-89ab-cdef-0123-456789abcdef",
  component, createdByThisTransaction: true, preExistingService: false,
  imageRelativePath: image, imageSha256: digest,
  ownerProofVerified: true, serviceImagePathMatches: true,
  signedReceiptAttested: true,
});

test("kernel install uses the pinned Sandboxie minifilter altitude and helper argv", () => {
  const spec = compile("03");
  assert.deepEqual(spec.argv, [
    "install", "SbieDrv", "D:\\MIVUBI\\Isolation-Payload\\SbieDrv.sys",
    "type=kernel", "start=demand",
    "msgfile=D:\\MIVUBI\\Isolation-Payload\\SbieMsg.dll", "altitude=86900",
  ]);
  assert.equal(spec.executionAllowed, false);
  assert.equal(spec.disposableHostAttested, false);
});

test("user-mode install preserves upstream service type and group", () => {
  const spec = compile("05");
  assert.equal(spec.argv[0], "install");
  assert.equal(spec.argv[1], "SbieSvc");
  assert.equal(spec.argv[2], '"D:\\MIVUBI\\Isolation-Payload\\SbieSvc.exe"');
  assert.ok(spec.argv.includes("type=own"));
  assert.ok(spec.argv.includes("group=UIGroup"));
  assert.ok(spec.argv.includes("start=auto"));
});

test("start, audit-only and rollback command semantics stay explicit", () => {
  assert.deepEqual(compile("06").argv, ["start", "SbieSvc"]);
  assert.equal(compile("01").argv, null);
  assert.equal(compile("02").argv, null);
  assert.equal(compile("04").argv, null);
  assert.equal(compile("07").argv, null);
  assert.equal(compile("R5").argv, null);
  // Rollback transports cannot be generated merely from self-reported
  // ownership booleans. A trusted lab key is not yet pinned.
  for (const [id, component, image] of [
    ["R1", "SbieSvc", "SbieSvc.exe"],
    ["R2", "SbieSvc", "SbieSvc.exe"],
    ["R3", "SbieDrv", "SbieDrv.sys"],
    ["R4", "SbieDrv", "SbieDrv.sys"],
  ]) {
    assert.throws(() => compile(id, { ownershipReceipt: owner(component, image) }),
      /rollback blocked: no pinned lab attestation key/);
  }
});

test("rollback cannot be compiled without an ownership proof receipt", () => {
  assert.throws(() => compile("R1"), /rollback lacks independently verified/);
  assert.throws(() => compile("R4", {ownershipReceipt: owner("SbieSvc","SbieSvc.exe")}),
    /rollback lacks independently verified/);
  assert.throws(() => compile("R3", {ownershipReceipt: {
    ...owner("SbieDrv","SbieDrv.sys"), signedReceiptAttested: false,
  }}), /rollback lacks independently verified/);
});

test("wrong pinned source, altered hash and reordered planner steps are blocked", () => {
  const invalid = { ...fixture(), upstreamCommit: "0".repeat(40) };
  assert.throws(() => compileTransportStep({plan: invalid, stepId: "03",
    payloadRoot:"D:\\MIVUBI\\Isolation-Payload"}), /unexpected or activated/);
  const missing = JSON.parse(JSON.stringify(fixture())); delete missing.installSteps[1].sha256;
  assert.throws(() => compileTransportStep({plan: missing, stepId: "03",
    payloadRoot:"D:\\MIVUBI\\Isolation-Payload"}), /missing pinned image hash/);
  const reorder = JSON.parse(JSON.stringify(fixture())); reorder.installSteps.reverse();
  assert.throws(() => compileTransportStep({plan: reorder, stepId: "03",
    payloadRoot:"D:\\MIVUBI\\Isolation-Payload"}), /unrecognized operation/);
});

test("paths outside isolated local Windows source locations are refused", () => {
  for (const path of [
    "C:\\Program Files\\Sandboxie-Plus", "C:\\Windows\\System32",
    "\\\\server\\share\\payload", "D:\\X\\..\\Owned", "D:\\Users\\Public\\MyData",
    "D:\\bad\"path", "/tmp/payload",
  ]) assert.throws(() => compileTransportStep({
    plan: fixture(), stepId: "03", payloadRoot: path,
  }), /payload root/);
});

test("all command descriptors remain blocked even if planner succeeded", () => {
  for (const id of ["01","02","03","04","05","06","07","R5"]) {
    const spec = compile(id);
    assert.equal(spec.executionAllowed, false);
    assert.equal(spec.driverSigningVerified, false);
    assert.equal(spec.rollbackAuthorized, false);
    assert.equal(spec.upstreamCommit, "abd4c240031cfb46666daa74ca47ed4e48b561e4");
  }
});
