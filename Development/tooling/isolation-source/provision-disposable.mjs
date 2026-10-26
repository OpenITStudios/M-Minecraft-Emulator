// Source-first deployment transaction planner for a Sandboxie-derived runtime.
// Pure planning only: never executes SCM, PowerShell, KmdUtil or Minecraft.
export const UPSTREAM_SHA = "abd4c240031cfb46666daa74ca47ed4e48b561e4";
export const DRIVER_SERVICE = "SbieDrv";
export const USER_SERVICE = "SbieSvc";
export const MINIFILTER_ALTITUDE = "86900";

const REQUIRED = ["SbieDrv.sys", "SbieSvc.exe", "SbieDll.dll", "SbieMsg.dll",
  "kmdutil.exe", "32/SbieDll.dll", "32/SbieSvc.exe",
  "LICENSES/Sandboxie/COPYING", "LICENSES/LICENSE.Plus"];
const SHA256 = /^[a-f0-9]{64}$/;
const SAFE_PATH = /^[A-Za-z0-9._/-]+$/;

const fail = message => { throw new Error("Owned engine deployment refused: " + message); };
const object = value => value !== null && typeof value === "object" && !Array.isArray(value);

export function createDisposableProvisionPlan({ payload, host, target }) {
  if (!object(host) || host.schema !== 1 || host.interactiveSession !== true ||
    host.serviceRegistration !== "ABSENT" || host.driverRegistration !== "ABSENT" ||
    host.serviceNamespaceAvailable !== true || host.providerSelected !== false ||
    host.trustedDriverLoaded !== false || host.activationAllowed !== false ||
    host.state !== "BLOCKED_UNVERIFIED_OWNED_DRIVER") {
    fail("host service/driver namespace occupied or not conclusively observed");
  }
  if (!object(payload) || payload.schema !== 1 ||
    payload.state !== "ARTIFACTS_STAGED_NOT_INSTALLABLE" ||
    payload.upstreamRepository !== "sandboxie-plus/Sandboxie" ||
    payload.upstreamTag !== "v1.18.5" || payload.upstreamCommit !== UPSTREAM_SHA ||
    payload.windowsKernelDriverLoaded !== false ||
    payload.minecraftGameplayVerified !== false ||
    payload.sandboxieExternalDependencyRemoved !== false ||
    payload.publicDistributionApproved !== false) {
    fail("unexpected source identity or unproven package claims");
  }
  if (!object(target) || target.kind !== "DEDICATED_DISPOSABLE_WINDOWS_VM" ||
    target.hasAccountBearingBox !== false ||
    target.containsOfficialSandboxie !== false ||
    target.allowProductionInstall !== false) {
    fail("target may contain active accounts or official Sandboxie");
  }
  if (!Array.isArray(payload.artifacts) || payload.artifacts.length < REQUIRED.length ||
    payload.artifacts.length > 48) fail("incomplete artifact manifest");
  const index = new Map();
  for (const item of payload.artifacts) {
    if (!object(item) || typeof item.path !== "string" ||
      !SAFE_PATH.test(item.path) || item.path.startsWith("/") ||
      item.path.split("/").includes("..") || item.path.includes("\\") ||
      index.has(item.path) || !Number.isSafeInteger(item.sizeBytes) ||
      item.sizeBytes < 1 || typeof item.sha256 !== "string" ||
      !SHA256.test(item.sha256)) fail("malformed or duplicated artifact");
    index.set(item.path, item);
  }
  for (const required of REQUIRED) {
    if (!index.has(required)) fail("missing artifact: " + required);
  }
  const step = (id, component, action, details = {}) =>
    Object.freeze({ id, component, action, ...details });
  const install = [
    step("01", DRIVER_SERVICE, "RECHECK_SCM_NAMES_ABSENT"),
    step("02", DRIVER_SERVICE, "VERIFY_SIGNED_DRIVER_AND_BINARY_IDENTITY",
      { path: "SbieDrv.sys", sha256: index.get("SbieDrv.sys").sha256 }),
    step("03", DRIVER_SERVICE, "INSTALL_KERNEL_MINIFILTER_VIA_UPSTREAM_KMDUTIL",
      { helper: "kmdutil.exe", type: "kernel", start: "demand",
        altitude: MINIFILTER_ALTITUDE, messageFile: "SbieMsg.dll" }),
    step("04", USER_SERVICE, "VERIFY_SIGNED_SERVICE_AND_BINARY_IDENTITY",
      { path: "SbieSvc.exe", sha256: index.get("SbieSvc.exe").sha256 }),
    step("05", USER_SERVICE, "INSTALL_USER_SERVICE_VIA_UPSTREAM_KMDUTIL",
      { helper: "kmdutil.exe", type: "own", start: "auto",
        group: "UIGroup", messageFile: "SbieMsg.dll" }),
    step("06", USER_SERVICE, "START_AND_READBACK_SIGNED_SERVICE"),
    step("07", DRIVER_SERVICE, "ATTEST_KERNEL_USERMODE_ABI_AND_BOX_ISOLATION"),
  ];
  const rollback = [
    step("R1", USER_SERVICE, "STOP_ONLY_IF_OWNED_RECEIPT_AND_IMAGE_MATCH"),
    step("R2", USER_SERVICE, "DELETE_ONLY_IF_OWNED_RECEIPT_AND_IMAGE_MATCH"),
    step("R3", DRIVER_SERVICE, "STOP_ONLY_IF_OWNED_RECEIPT_AND_IMAGE_MATCH"),
    step("R4", DRIVER_SERVICE, "DELETE_ONLY_IF_OWNED_RECEIPT_AND_IMAGE_MATCH"),
    step("R5", DRIVER_SERVICE, "PRESERVE_BOXES_WORLDS_AND_RECORD_RESULT"),
  ];
  // The target's declaration is not VM attestation. A staged payload is
  // explicitly NOT installable; no step may be executed from this plan.
  return Object.freeze({
    schema: 1, state: "SOURCE_PLAN_ONLY_DRIVER_INSTALL_BLOCKED",
    upstreamCommit: UPSTREAM_SHA,
    installSteps: Object.freeze(install),
    rollbackSteps: Object.freeze(rollback),
    disposableHostVerified: false,
    codeSignatureVerified: false,
    installationPermitted: false,
    providerSelected: false,
    driverLoaded: false,
    gameplayVerified: false,
    rollbackPermittedWithoutOwnedReceipt: false,
  });
}

// Pure, bounded receipt analyzer. On a failed or unknown side-effect, stop.
// An untrusted receipt NEVER proves an installed driver or safe deletion.
export function evaluateProvisionReceipt(plan, receipt) {
  if (!object(plan) || plan.state !== "SOURCE_PLAN_ONLY_DRIVER_INSTALL_BLOCKED" ||
    !object(receipt) || receipt.schema !== 1 || !Array.isArray(receipt.steps) ||
    receipt.steps.length > plan.installSteps.length) fail("invalid plan or receipt");
  let hasUncertain = false;
  for (let i = 0; i < receipt.steps.length; i++) {
    const observed = receipt.steps[i];
    const expected = plan.installSteps[i];
    if (!object(observed) || observed.id !== expected.id ||
      observed.component !== expected.component ||
      !["CONFIRMED", "FAILED", "UNKNOWN"].includes(observed.outcome)) {
      fail("unknown or out-of-order receipt step");
    }
    if (i > 0 && receipt.steps[i - 1].outcome !== "CONFIRMED") {
      fail("steps continued after failed or ambiguous operation");
    }
    if (observed.outcome !== "CONFIRMED") hasUncertain = true;
  }
  return {
    schema: 1,
    state: receipt.steps.length === 0 ? "NO_MUTATION_RECORDED" :
      hasUncertain ? "STOP_MANUAL_RECONCILIATION_REQUIRED" :
        "AWAITING_SIGNED_RUNTIME_ACCEPTANCE",
    observedStepCount: receipt.steps.length,
    canRetryAutomatically: false,
    canDeleteServiceAutomatically: false,
    installationComplete: false,
    providerSelected: false,
  };
}
