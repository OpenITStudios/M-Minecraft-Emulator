// Compiles the pinned Sandboxie v1.18.5 KmdUtil transport interface.
// This module returns data, NOT an executable authorization. No OS calls.
import { win32 } from "node:path";
import { UPSTREAM_SHA } from "../provision-disposable.mjs";
import { PRODUCTION_LAB_TRUST_KEY_PEM } from "./attestation-gate.mjs";

const DRIVER = "SbieDrv";
const SERVICE = "SbieSvc";
const ID = [
  ["01", DRIVER, "RECHECK_SCM_NAMES_ABSENT"],
  ["02", DRIVER, "VERIFY_SIGNED_DRIVER_AND_BINARY_IDENTITY"],
  ["03", DRIVER, "INSTALL_KERNEL_MINIFILTER_VIA_UPSTREAM_KMDUTIL"],
  ["04", SERVICE, "VERIFY_SIGNED_SERVICE_AND_BINARY_IDENTITY"],
  ["05", SERVICE, "INSTALL_USER_SERVICE_VIA_UPSTREAM_KMDUTIL"],
  ["06", SERVICE, "START_AND_READBACK_SIGNED_SERVICE"],
  ["07", DRIVER, "ATTEST_KERNEL_USERMODE_ABI_AND_BOX_ISOLATION"],
];
const ROLLBACK = [
  ["R1", SERVICE, "STOP_ONLY_IF_OWNED_RECEIPT_AND_IMAGE_MATCH"],
  ["R2", SERVICE, "DELETE_ONLY_IF_OWNED_RECEIPT_AND_IMAGE_MATCH"],
  ["R3", DRIVER, "STOP_ONLY_IF_OWNED_RECEIPT_AND_IMAGE_MATCH"],
  ["R4", DRIVER, "DELETE_ONLY_IF_OWNED_RECEIPT_AND_IMAGE_MATCH"],
  ["R5", DRIVER, "PRESERVE_BOXES_WORLDS_AND_RECORD_RESULT"],
];
const fail = msg => { throw new Error("KmdUtil transport refused: " + msg); };
const record = value => value !== null && typeof value === "object" && !Array.isArray(value);

function checkedPayloadRoot(path) {
  if (typeof path !== "string" || path.length < 4 || path.length > 240 ||
      !/^[A-Za-z]:\\/.test(path) || !win32.isAbsolute(path) ||
      path.includes('"') || /[\u0000-\u001f]/.test(path) ||
      path.split("\\").some(part => part === ".." || part === ".") ||
      /(^|\\)Sandboxie-Plus(\\|$)/i.test(path) ||
      /\\(?:Users|Windows|Program Files(?: \(x86\))?)\\/i.test(path)) {
    fail("payload root must be a separate ordinary local Windows directory");
  }
  return path.replace(/\\+$/, "");
}
function validatePlan(plan) {
  if (!record(plan) || plan.schema !== 1 ||
    plan.state !== "SOURCE_PLAN_ONLY_DRIVER_INSTALL_BLOCKED" ||
    plan.upstreamCommit !== UPSTREAM_SHA ||
    plan.installationPermitted !== false || plan.providerSelected !== false ||
    plan.codeSignatureVerified !== false ||
    !Array.isArray(plan.installSteps) || plan.installSteps.length !== ID.length ||
    !Array.isArray(plan.rollbackSteps) || plan.rollbackSteps.length !== ROLLBACK.length) {
    fail("unexpected or activated provisioning plan");
  }
  for (const [list, expected] of [
    [plan.installSteps, ID], [plan.rollbackSteps, ROLLBACK],
  ]) {
    for (let i = 0; i < expected.length; i++) {
      if (list[i]?.id !== expected[i][0] || list[i]?.component !== expected[i][1] ||
          list[i]?.action !== expected[i][2]) fail("unrecognized operation sequence");
    }
  }
}
function validateReceipt(receipt, component, relativeImagePath, expectedSha) {
  if (!record(receipt) ||
      !/^[0-9a-f-]{36}$/i.test(receipt.transactionId) ||
      receipt.component !== component ||
      receipt.createdByThisTransaction !== true ||
      receipt.preExistingService !== false ||
      receipt.imageRelativePath !== relativeImagePath ||
      receipt.imageSha256 !== expectedSha ||
      receipt.ownerProofVerified !== true ||
      receipt.serviceImagePathMatches !== true ||
      receipt.signedReceiptAttested !== true) {
    fail("rollback lacks independently verified service ownership");
  }
  // These boolean fields are caller-controlled and therefore NOT evidence.
  // Until a reviewed lab key and independently corroborated SCM/journal proof
  // are wired into the transport, no stop/delete descriptor can be emitted.
  if (!PRODUCTION_LAB_TRUST_KEY_PEM.trim()) {
    fail("rollback blocked: no pinned lab attestation key");
  }
  fail("rollback blocked: independent SCM and journal ownership gate not activated");
}
const SHA256 = /^[a-f0-9]{64}$/;

/**
 * Generate exact KmdUtil argv for a single source-planned step. The caller
 * cannot cause a launch: response.executionAllowed remains false.
 */
export function compileTransportStep({ plan, payloadRoot, stepId, ownershipReceipt = null }) {
  validatePlan(plan);
  const root = checkedPayloadRoot(payloadRoot);
  const step = [...plan.installSteps, ...plan.rollbackSteps].find(v => v.id === stepId);
  if (!step) fail("unknown step ID");
  const image = step.component === DRIVER ? "SbieDrv.sys" : "SbieSvc.exe";
  const expectedStage = plan.installSteps.find(v => v.id === (image === "SbieDrv.sys" ? "02" : "04"));
  if (!expectedStage || !SHA256.test(expectedStage.sha256) ||
      expectedStage.path !== image) fail("missing pinned image hash evidence");
  const serviceImage = win32.join(root, image);
  const msgfile = win32.join(root, "SbieMsg.dll");
  const helper = win32.join(root, "kmdutil.exe");
  const requiresReceipt = ["R1", "R2", "R3", "R4"].includes(step.id);
  if (requiresReceipt) validateReceipt(ownershipReceipt, step.component,
    image, expectedStage.sha256);
  let argv = null;
  switch (step.id) {
    case "03":
      argv = ["install", DRIVER, serviceImage, "type=kernel", "start=demand",
        "msgfile=" + msgfile, "altitude=86900"];
      break;
    case "05":
      argv = ["install", SERVICE, '"' + serviceImage + '"', "type=own", "start=auto",
        "display=MIVUBI Isolation Lab Service", "group=UIGroup",
        "msgfile=" + msgfile];
      break;
    case "06":
      argv = ["start", SERVICE];
      break;
    case "R1": case "R3":
      argv = ["stop", step.component]; break;
    case "R2": case "R4":
      argv = ["delete", step.component]; break;
    default:
      break;
  }
  return {
    schema: 1,
    kind: "PINNED_KMDUTIL_TRANSPORT_SPEC",
    upstreamCommit: UPSTREAM_SHA,
    stepId: step.id,
    component: step.component,
    operation: step.action,
    helperRelativePath: "kmdutil.exe",
    executable: helper,
    argv,
    requiredImagePath: serviceImage,
    requiredImageSha256: expectedStage.sha256,
    expectedScmServiceName: step.component,
    ownershipReceiptRequired: requiresReceipt,
    driverMinifilterAltitude: "86900",
    executionAllowed: false,
    driverSigningVerified: false,
    disposableHostAttested: false,
    rollbackAuthorized: false,
  };
}
