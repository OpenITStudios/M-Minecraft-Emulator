import test from "node:test";
import assert from "node:assert/strict";
import { evaluateClientFeasibility } from "./evaluate-client-feasibility.mjs";

const fixture = () => ({
  "schema": 1,
  "scope": "CLIENT_FEASIBILITY",
  "stage": "INTERACTIVE",
  "evidenceOrigin": "WINDOWS_GUEST",
  "automaticVerdict": "NOT_PROVIDED",
  "windows": {
    "interactiveSession": true,
    "architecture": "64-bit",
    "vmwareToolsRunning": true
  },
  "graphics": {
    "openGl": {
      "status": "MEASURED",
      "version": "4.3 VMware"
    }
  },
  "minecraft": {
    "candidateProcessCount": 1,
    "processEvidenceClass": "CANDIDATE_ONLY"
  },
  "operator": {
    "menuRendered": "YES",
    "worldRendered": "YES",
    "inputResponsive": "YES",
    "blockingGraphicsError": "NO",
    "vmDesktopOpened": "YES",
    "educationInstalled": "YES",
    "educationSignedIn": "YES",
    "crashOrFreezeObserved": "NO",
    "observedMinutes": 15,
    "evidenceReference": "proof-01",
    "attribution": "OPERATOR_DECLARED_NOT_MACHINE_VERIFIED"
  }
});
test("working-game evidence never implies official virtual-client support", () => {
  const x = evaluateClientFeasibility(fixture());
  assert.equal(x.verdict, "OPERATOR_REPORTED_PLAYABLE");
  assert.equal(x.officialVmClientSupport, "NOT_CLAIMED");
  assert.ok(x.risks.includes("OPENGL_BELOW_PUBLISHED_4_4_REFERENCE"));
});
test("unsupported or unknown Windows bitness cannot be reported playable", () => {
  for (const architecture of ["32-bit", null, "unknown"]) {
    const report = fixture();
    report.windows.architecture = architecture;
    const result = evaluateClientFeasibility(report);
    assert.equal(result.verdict, "INCONCLUSIVE");
    assert.ok(result.risks.includes("WINDOWS_64_BIT_UNVERIFIED"));
  }
});
test("process alone never proves visible Minecraft", () => {
  const x = fixture(); x.operator.menuRendered = "UNKNOWN";
  assert.equal(evaluateClientFeasibility(x).verdict, "INCONCLUSIVE");
});
test("reported graphics failure and a non-rendered world are explicit failures", () => {
  const x = fixture(); x.operator.blockingGraphicsError = "YES";
  assert.equal(evaluateClientFeasibility(x).verdict, "OPERATOR_REPORTED_FAILURE");
  const y = fixture(); y.operator.worldRendered = "NO";
  assert.equal(evaluateClientFeasibility(y).verdict, "OPERATOR_REPORTED_FAILURE");
});
test("missing evidence, process, or interactive session never produces playability", () => {
  for (const key of ["evidence","process","session"]) {
    const x = fixture();
    if(key==="evidence") x.operator.evidenceReference=null;
    if(key==="process") x.minecraft.candidateProcessCount=0;
    if(key==="session") x.windows.interactiveSession=false;
    assert.equal(evaluateClientFeasibility(x).verdict, "INCONCLUSIVE");
  }
});
test("environment snapshot cannot claim gameplay; unknown GL remains explicitly flagged", () => {
  const x = fixture(); x.stage="ENVIRONMENT";
  for(const k of ["menuRendered","worldRendered","inputResponsive","blockingGraphicsError","vmDesktopOpened","educationInstalled","educationSignedIn","crashOrFreezeObserved"])x.operator[k]="UNKNOWN";
  x.operator.observedMinutes=0;
  x.operator.evidenceReference=null;
  x.graphics.openGl={status:"UNKNOWN",version:null};
  const o=evaluateClientFeasibility(x);
  assert.equal(o.verdict,"INCONCLUSIVE");
  assert.ok(o.risks.includes("OPENGL_UNMEASURED"));
});
test("unknown schema, forged automatic pass and sensitive evidence paths rejected", () => {
  for(const mutation of [x=>x.schema=2,x=>x.automaticVerdict="PASS",x=>x.operator.evidenceReference="C:\\account\\data",x=>x.stage="PASS"]) {
    const x=fixture();mutation(x);
    assert.throws(()=>evaluateClientFeasibility(x),TypeError);
  }
});
test("VM boot, installer and distinct sign-in are mandatory even when a world seems rendered", () => {
  for (const key of ["vmDesktopOpened", "educationInstalled", "educationSignedIn"]) {
    const x=fixture(); x.operator[key]="UNKNOWN";
    assert.equal(evaluateClientFeasibility(x).verdict, "INCONCLUSIVE");
    x.operator[key]="NO";
    assert.equal(evaluateClientFeasibility(x).verdict, "OPERATOR_REPORTED_FAILURE");
  }
});
test("short or crashing game sessions cannot count as responsive", () => {
  const short=fixture(); short.operator.observedMinutes=3;
  assert.equal(evaluateClientFeasibility(short).verdict,"INCONCLUSIVE");
  const crashed=fixture(); crashed.operator.crashOrFreezeObserved="YES";
  assert.equal(evaluateClientFeasibility(crashed).verdict,"OPERATOR_REPORTED_FAILURE");
});
test("conflicting Store and desktop installations cannot satisfy the install gate", () => {
  const x=fixture(); x.minecraft.desktopInstallVersion="26.32.0";
  x.minecraft.storeInstallVersion="26.32.0";
  const out=evaluateClientFeasibility(x);
  assert.equal(out.verdict,"INCONCLUSIVE");
  assert.ok(out.risks.includes("DESKTOP_STORE_INSTALL_CONFLICT"));
});

test("GPU-PV requires actual guest D3D11 hardware and NVIDIA OpenGL >=4.4", () => {
  const ready = fixture();
  ready.provider = "APP_SANDBOX_GPU_PV";
  ready.windows.vmwareToolsRunning = null;
  ready.graphics.openGl = {
    status: "MEASURED", version: "4.6.0 NVIDIA", vendor: "NVIDIA Corporation",
    renderer: "NVIDIA RTX 3070"
  };
  ready.graphics.direct3D11 = {
    status: "MEASURED", featureLevel: "11_0", driverType: "HARDWARE", reasonCode: null
  };
  const result = evaluateClientFeasibility(ready);
  assert.equal(result.provider, "APP_SANDBOX_GPU_PV");
  assert.equal(result.verdict, "OPERATOR_REPORTED_PLAYABLE");
  assert.equal(result.gates.hardwareGpuGraphics, "GUEST_DIAGNOSTICS_OBSERVED");
  assert.ok(!result.risks.includes("VMWARE_TOOLS_UNVERIFIED"));
  assert.equal(result.officialVmClientSupport, "NOT_CLAIMED");
  const withoutD3D = structuredClone(ready);
  withoutD3D.graphics.direct3D11 = { status: "UNKNOWN" };
  const d3d = evaluateClientFeasibility(withoutD3D);
  assert.equal(d3d.verdict, "INCONCLUSIVE");
  assert.ok(d3d.risks.includes("D3D11_HARDWARE_FL11_0_UNVERIFIED"));
  const wrongRenderer = structuredClone(ready);
  wrongRenderer.graphics.openGl.renderer = "GDI Generic";
  wrongRenderer.graphics.openGl.vendor = "Microsoft Corporation";
  const gl = evaluateClientFeasibility(wrongRenderer);
  assert.equal(gl.verdict, "INCONCLUSIVE");
  assert.ok(gl.risks.includes("OPENGL_NVIDIA_HARDWARE_UNVERIFIED"));
  const oldVersion = structuredClone(ready);
  oldVersion.graphics.openGl.version = "4.3 NVIDIA";
  assert.equal(evaluateClientFeasibility(oldVersion).verdict, "INCONCLUSIVE");
});

test("unknown provider and unmeasured GPU-PV cannot be declared playable", () => {
  const invalid = fixture();
  invalid.provider = "HCS_100_PERCENT";
  assert.throws(() => evaluateClientFeasibility(invalid), TypeError);
  const unknown = fixture();
  unknown.provider = "APP_SANDBOX_GPU_PV";
  unknown.windows.vmwareToolsRunning = null;
  unknown.graphics.openGl = { status: "UNKNOWN", version: null };
  assert.equal(evaluateClientFeasibility(unknown).verdict, "INCONCLUSIVE");
  const old = fixture();
  delete old.provider;
  assert.equal(evaluateClientFeasibility(old).provider, "VMWARE");
  assert.equal(evaluateClientFeasibility(old).verdict, "OPERATOR_REPORTED_PLAYABLE");
});

test("GPU-PV rejects contradictory diagnostics rather than promoting them", () => {
  const sample = fixture();
  sample.provider = "APP_SANDBOX_GPU_PV";
  sample.graphics.openGl = {
    status: "MEASURED", version: "4.6 NVIDIA", vendor: "NVIDIA Corporation",
    renderer: "NVIDIA RTX 3070", reasonCode: null
  };
  sample.graphics.direct3D11 = {
    status: "MEASURED", featureLevel: "11_0", driverType: "HARDWARE", reasonCode: null
  };
  assert.equal(evaluateClientFeasibility(sample).verdict, "OPERATOR_REPORTED_PLAYABLE");
  for (const mutate of [
    x => { x.graphics.openGl.reasonCode = "OPENGL_CONTEXT_PROBE_FAILED"; },
    x => { x.graphics.openGl.status = "UNKNOWN"; },
    x => { x.graphics.direct3D11.driverType = "WARP"; },
    x => { x.graphics.direct3D11.featureLevel = "12_1"; },
    x => { x.graphics.direct3D11.reasonCode = "D3D11_HARDWARE_PROBE_FAILED"; },
    x => { x.graphics.direct3D11.status = "UNKNOWN"; },
    x => { x.graphics.direct3D11.status = "ERROR"; }
  ]) {
    const corrupted = structuredClone(sample);
    mutate(corrupted);
    assert.throws(() => evaluateClientFeasibility(corrupted), TypeError);
  }
});

test("GPU-PV cannot treat a generic renderer or partially measured OpenGL as NVIDIA hardware", () => {
  const ready = fixture();
  ready.provider = "APP_SANDBOX_GPU_PV";
  ready.windows.vmwareToolsRunning = null;
  ready.graphics.openGl = {
    status: "MEASURED", version: "4.6 NVIDIA", vendor: "NVIDIA Corporation",
    renderer: "NVIDIA GeForce RTX 3070", reasonCode: null
  };
  ready.graphics.direct3D11 = {
    status: "MEASURED", featureLevel: "11_0", driverType: "HARDWARE", reasonCode: null
  };
  const validGeForce = structuredClone(ready);
  validGeForce.graphics.openGl.renderer = "GeForce RTX 3070/PCIe/SSE2";
  assert.equal(evaluateClientFeasibility(validGeForce).verdict, "OPERATOR_REPORTED_PLAYABLE");
  for (const renderer of [
    "Microsoft Basic Render Driver", "GDI Generic", "llvmpipe NVIDIA", "Generic Renderer"
  ]) {
    const observation = structuredClone(ready);
    observation.graphics.openGl.renderer = renderer;
    const result = evaluateClientFeasibility(observation);
    assert.equal(result.verdict, "INCONCLUSIVE");
    assert.ok(result.risks.includes("OPENGL_NVIDIA_HARDWARE_UNVERIFIED"));
  }
  const absent = structuredClone(ready);
  absent.graphics.openGl.status = "UNKNOWN";
  absent.graphics.openGl.version = null;
  absent.graphics.openGl.vendor = null;
  absent.graphics.openGl.renderer = null;
  absent.graphics.openGl.reasonCode = "OPENGL_CONTEXT_PROBE_FAILED";
  assert.equal(evaluateClientFeasibility(absent).verdict, "INCONCLUSIVE");
  assert.ok(evaluateClientFeasibility(absent).risks.includes("OPENGL_UNMEASURED"));
});

test("legacy VMware reports remain compatible without new GPU-PV fields", () => {
  const legacy = fixture();
  delete legacy.provider;
  delete legacy.graphics.direct3D11;
  legacy.graphics.openGl.vendor = undefined;
  legacy.graphics.openGl.renderer = undefined;
  assert.equal(evaluateClientFeasibility(legacy).verdict, "OPERATOR_REPORTED_PLAYABLE");
  assert.equal(evaluateClientFeasibility(legacy).gates.hardwareGpuGraphics, "NOT_APPLICABLE");
});
