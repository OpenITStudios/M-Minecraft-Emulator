import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

// Evaluates the existing acceptance evidence; never infers official Minecraft
// Education support or renders a configuration bit as game-playability proof.
export function evaluateClientFeasibility(report) {
  const fail = (message) => { throw new TypeError(message); };
  if (!report || typeof report !== "object" || report.schema !== 1 || report.scope !== "CLIENT_FEASIBILITY" || report.evidenceOrigin !== "WINDOWS_GUEST" || report.automaticVerdict !== "NOT_PROVIDED") fail("Unknown or untrusted feasibility-report contract.");
  if (!["ENVIRONMENT", "INTERACTIVE"].includes(report.stage)) fail("Invalid feasibility stage.");
  // Old schema-1 VMware reports omitted provider; preserve their meaning.
  const provider = report.provider ?? "VMWARE";
  if (!["VMWARE", "APP_SANDBOX_GPU_PV"].includes(provider)) fail("Unknown virtualization provider.");
  const win = report.windows;
  const gl = report.graphics?.openGl;
  const d3d = report.graphics?.direct3D11;
  const mc = report.minecraft;
  const operator = report.operator;
  if (!win || typeof win.interactiveSession !== "boolean" || !gl || !["MEASURED", "UNKNOWN"].includes(gl.status) || !mc || mc.processEvidenceClass !== "CANDIDATE_ONLY" || !operator || operator.attribution !== "OPERATOR_DECLARED_NOT_MACHINE_VERIFIED") fail("Missing or invalid source attribution.");
  const flags = ["menuRendered", "worldRendered", "inputResponsive", "blockingGraphicsError",
    "vmDesktopOpened", "educationInstalled", "educationSignedIn", "crashOrFreezeObserved"];
  if (!flags.every(k => ["YES", "NO", "UNKNOWN"].includes(operator[k]))) fail("Unexpected operator observation value.");
  if (mc.candidateProcessCount !== null && (!Number.isInteger(mc.candidateProcessCount) || mc.candidateProcessCount < 0)) fail("Invalid candidate process count.");
  if (gl.status === "MEASURED" && (typeof gl.version !== "string" || !gl.version.trim())) fail("OpenGL measurement is missing its version.");
  if (provider === "APP_SANDBOX_GPU_PV") {
    // The GPU-PV collector emits an optional diagnostic value only when the
    // probe actually measured it. Reject contradictory report records instead
    // of allowing an UNKNOWN probe to carry fabricated hardware evidence.
    if (gl.status === "UNKNOWN" &&
        (gl.version != null || gl.vendor != null || gl.renderer != null)) {
      fail("Unknown GPU-PV OpenGL result cannot carry measured graphics values.");
    }
    if (gl.status === "MEASURED" && gl.reasonCode != null) {
      fail("Measured GPU-PV OpenGL result cannot carry a failure reason.");
    }
    if (d3d != null) {
      if (typeof d3d !== "object" || !["MEASURED", "UNKNOWN"].includes(d3d.status)) {
        fail("Invalid GPU-PV Direct3D evidence status.");
      }
      if (d3d.status === "MEASURED"
          && (d3d.featureLevel !== "11_0" || d3d.driverType !== "HARDWARE"
              || d3d.reasonCode != null)) {
        fail("Measured GPU-PV Direct3D evidence contradicts the hardware FL11_0 probe.");
      }
      if (d3d.status === "UNKNOWN"
          && (d3d.featureLevel != null || d3d.driverType != null)) {
        fail("Unknown GPU-PV Direct3D result cannot carry measured hardware values.");
      }
    }
  }
  if (!Number.isInteger(operator.observedMinutes) || operator.observedMinutes < 0 || operator.observedMinutes > 1440) fail("Invalid observed session duration.");
  const reference = operator.evidenceReference;
  if (reference !== null && (typeof reference !== "string" || !/^[a-zA-Z0-9_.-]{1,80}$/.test(reference))) fail("Evidence reference must be a non-sensitive file label.");
  if (report.stage === "ENVIRONMENT" && (flags.some(k => operator[k] !== "UNKNOWN") || operator.observedMinutes !== 0 || reference !== null)) fail("Environment probe cannot contain operator gameplay claims.");
  const risks = ["VIRTUAL_MINECRAFT_CLIENT_NOT_OFFICIALLY_SUPPORTED"];
  if (!win.interactiveSession) risks.push("NON_INTERACTIVE_WINDOWS_SESSION");
  // Supported Windows guests must report 64-bit. A 32-bit or unknown OS
  // cannot be promoted by manually entered gameplay observations.
  const supportedWindowsArchitecture = typeof win.architecture === "string" && /\b64[- ]bit\b/i.test(win.architecture);
  if (!supportedWindowsArchitecture) risks.push("WINDOWS_64_BIT_UNVERIFIED");
  if (provider === "VMWARE" && win.vmwareToolsRunning !== true) risks.push("VMWARE_TOOLS_UNVERIFIED");
  // Graphics capability must be observed from the interactive guest itself.
  // An App Sandbox config gpuMode=1, host RTX adapter, or online HCS agent
  // does not establish native NVIDIA OpenGL or D3D11 FL11_0.
  const d3dHardware11 = d3d?.status === "MEASURED"
    && d3d.featureLevel === "11_0" && d3d.driverType === "HARDWARE";
  const nvidiaOpenGl = gl.status === "MEASURED"
    && typeof gl.vendor === "string" && /\bnvidia\b/i.test(gl.vendor)
    && typeof gl.renderer === "string"
    && /\b(?:nvidia|geforce|quadro|tesla|rtx|gtx)\b/i.test(gl.renderer)
    && !/llvmpipe|softpipe|software|gdi generic|swiftshader|warp|basic render/i.test(gl.renderer);
  if (provider === "APP_SANDBOX_GPU_PV") {
    if (!d3dHardware11) risks.push("D3D11_HARDWARE_FL11_0_UNVERIFIED");
    if (!nvidiaOpenGl) risks.push("OPENGL_NVIDIA_HARDWARE_UNVERIFIED");
  }
  let openGl44 = false;
  if (gl.status === "MEASURED") {
    const match = /^\s*(\d+)\.(\d+)(?:\D|$)/.exec(gl.version);
    if (!match) risks.push("OPENGL_VERSION_UNPARSEABLE");
    else if (+match[1] < 4 || (+match[1] === 4 && +match[2] < 4)) risks.push("OPENGL_BELOW_PUBLISHED_4_4_REFERENCE");
    else openGl44 = true;
  } else risks.push("OPENGL_UNMEASURED");
  if (mc.candidateProcessCount === null || mc.candidateProcessCount === 0) risks.push("MINECRAFT_PROCESS_NOT_OBSERVED");
  if (mc.desktopInstallVersion && mc.storeInstallVersion) risks.push("DESKTOP_STORE_INSTALL_CONFLICT");
  if (operator.observedMinutes < 10) risks.push("RESPONSIVE_SESSION_DURATION_UNPROVEN");
  let verdict = "INCONCLUSIVE";
  if (report.stage === "INTERACTIVE") {
    if (["vmDesktopOpened", "educationInstalled", "educationSignedIn", "menuRendered", "worldRendered", "inputResponsive"].some(k => operator[k] === "NO") ||
      operator.blockingGraphicsError === "YES" || operator.crashOrFreezeObserved === "YES") verdict = "OPERATOR_REPORTED_FAILURE";
    else if (["vmDesktopOpened", "educationInstalled", "educationSignedIn", "menuRendered", "worldRendered", "inputResponsive"].every(k => operator[k] === "YES") &&
      operator.blockingGraphicsError === "NO" && operator.crashOrFreezeObserved === "NO" &&
      operator.observedMinutes >= 10 && reference !== null && win.interactiveSession &&
      supportedWindowsArchitecture && mc.candidateProcessCount > 0 && !(mc.desktopInstallVersion && mc.storeInstallVersion)
      && (provider !== "APP_SANDBOX_GPU_PV" || (d3dHardware11 && nvidiaOpenGl && openGl44))) verdict = "OPERATOR_REPORTED_PLAYABLE";
  }
  return {
    schema: 1,
    scope: "CLIENT_FEASIBILITY",
    stage: report.stage,
    provider,
    verdict,
    proofCeiling: "OPERATOR_DECLARED_AND_GUEST_DIAGNOSTICS_ONLY",
    officialVmClientSupport: "NOT_CLAIMED",
    gates: {
      windowsDesktop: operator.vmDesktopOpened === "YES" ? "OPERATOR_REPORTED" : "UNVERIFIED_OR_FAILED",
      educationInstallAndSignIn: operator.educationInstalled === "YES" && operator.educationSignedIn === "YES" ? "OPERATOR_REPORTED" : "UNVERIFIED_OR_FAILED",
      responsiveGameplay: operator.worldRendered === "YES" && operator.inputResponsive === "YES" &&
        operator.crashOrFreezeObserved === "NO" && operator.observedMinutes >= 10 ? "OPERATOR_REPORTED" : "UNVERIFIED_OR_FAILED",
      hardwareGpuGraphics: provider === "APP_SANDBOX_GPU_PV"
        ? (d3dHardware11 && nvidiaOpenGl && openGl44 ? "GUEST_DIAGNOSTICS_OBSERVED" : "UNVERIFIED_OR_FAILED")
        : "NOT_APPLICABLE"
    },
    risks,
    next: verdict === "OPERATOR_REPORTED_PLAYABLE"
      ? (provider === "VMWARE" ? "REVIEW_EVIDENCE_BEFORE_PRODUCTION_BASE" : "REVIEW_GPU_PV_EVIDENCE_BEFORE_THREE_GUESTS") : verdict === "OPERATOR_REPORTED_FAILURE" ? "STOP_AND_DIAGNOSE_RENDER_OR_INPUT" : "COLLECT_MISSING_INTERACTIVE_EVIDENCE"
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const file = process.argv[2];
  if (!file) {
    console.error("Usage: node tooling/acceptance/evaluate-client-feasibility.mjs <guest-report.json>");
    process.exitCode = 2;
  } else {
    try {
      const report = JSON.parse(readFileSync(file, "utf8").replace(/^\uFEFF/, ""));
      console.log(JSON.stringify(evaluateClientFeasibility(report), null, 2));
    } catch (error) {
      console.error(error instanceof Error ? error.message : String(error));
      process.exitCode = 2;
    }
  }
}
