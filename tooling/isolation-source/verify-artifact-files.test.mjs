import test from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync, symlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";
import { verifyArtifactFiles } from "./verify-artifact-files.mjs";

function setup() {
  const root = mkdtempSync(join(tmpdir(), "isolation-hash-"));
  // Test the complete version-locked closure, not an obsolete subset.
  const manifest = JSON.parse(readFileSync(
    new URL("./upstream-baseline.json", import.meta.url), "utf8"));
  const artifacts = [
    ...manifest.requiredArtifacts.map(path => ({ path, kind: "required" })),
    ...manifest.optionalSymbols.map(path => ({ path, kind: "optional-symbol" })),
    ...manifest.sourceResources.map(path => ({ path, kind: "source-resource" })),
  ].map(a => {
    const bytes = Buffer.from("artifact:" + a.path);
    const filename = join(root, ...a.path.split("/"));
    mkdirSync(dirname(filename), { recursive: true });
    writeFileSync(filename, bytes);
    return { ...a, state: "PRESENT", sizeBytes: bytes.length,
      sha256: createHash("sha256").update(bytes).digest("hex"), authenticode: "NotSigned" };
  });
  const report = {
    schema: 1, sourceRepository: manifest.upstream.repository,
    sourceTag: manifest.upstream.tag, sourceCommit: manifest.upstream.commit,
    sourceCleanAtStart: true, compiledInThisInvocation: true,
    stageResults: manifest.buildStages.map(s => ({ ...s, state: "COMPILED", exitCode: 0 })),
    artifacts, artifactResult: "BUILD_ARTIFACTS_COLLECTED",
    signingLoadAcceptance: "NOT_EXECUTED", minecraftLoginGameplayAcceptance: "NOT_EXECUTED",
    distributionApproval: "NOT_APPROVED",
  };
  return { root, manifest, report, cleanup: () => rmSync(root, { recursive: true, force: true }) };
}

test("hash verifier checks actual file bytes without promoting driver/game acceptance", async () => {
  const f = setup(); try {
    const result = await verifyArtifactFiles(f.manifest, f.report, f.root);
    assert.equal(result.checkedPresent, f.report.artifacts.length);
    assert.equal(result.driverLoadVerified, false);
    assert.equal(result.gameplayVerified, false);
  } finally { f.cleanup(); }
});
test("hash verifier rejects same-size modifications absent from report", async () => {
  const f = setup(); try {
    const item = f.report.artifacts[0];
    writeFileSync(join(f.root, ...item.path.split("/")), Buffer.alloc(item.sizeBytes, 0x41));
    await assert.rejects(verifyArtifactFiles(f.manifest, f.report, f.root), /SHA-256 differs/);
  } finally { f.cleanup(); }
});
test("hash verifier rejects missing required files that report claimed present", async () => {
  const f = setup(); try {
    rmSync(join(f.root, ...f.report.artifacts[0].path.split("/")));
    await assert.rejects(verifyArtifactFiles(f.manifest, f.report, f.root), /reported present artifact missing/);
  } finally { f.cleanup(); }
});
test("hash verifier refuses symlinked artifact outside source root", async t => {
  const f = setup(); const outside = mkdtempSync(join(tmpdir(), "isolation-outside-"));
  try {
    const file = join(f.root, ...f.report.artifacts[0].path.split("/"));
    rmSync(file);
    const external = join(outside, "payload");
    writeFileSync(external, "external");
    try { symlinkSync(external, file); } catch (error) {
      if (process.platform === "win32" && ["EPERM", "EACCES"].includes(error.code)) {
        t.skip("Windows symlink privilege unavailable");
        return;
      }
      throw error;
    }
    await assert.rejects(verifyArtifactFiles(f.manifest, f.report, f.root), /symlinked component/);
  } finally { f.cleanup(); rmSync(outside, {recursive:true, force:true}); }
});
test("hash verifier rejects a false missing symbol and permits genuinely absent optional symbols", async () => {
  const f = setup(); try {
    const item = f.report.artifacts.find(x => x.kind === "optional-symbol");
    Object.assign(item, { state: "MISSING", sizeBytes: null, sha256: null, authenticode: "NOT_EXECUTED" });
    await assert.rejects(verifyArtifactFiles(f.manifest, f.report, f.root), /reported missing artifact exists/);
    rmSync(join(f.root, ...item.path.split("/")));
    const result = await verifyArtifactFiles(f.manifest, f.report, f.root);
    assert.equal(result.checkedAbsent, 1);
  } finally { f.cleanup(); }
});

test("refuses dangling symlink disguised as a missing optional artifact", async t => {
  const f = setup(); const outside = mkdtempSync(join(tmpdir(), "isolation-dangling-"));
  try {
    const item = f.report.artifacts.find(x => x.kind === "optional-symbol");
    const file = join(f.root, ...item.path.split("/"));
    rmSync(file);
    try { symlinkSync(join(outside, "nonexistent-payload"), file); } catch (error) {
      if (process.platform === "win32" && ["EPERM", "EACCES"].includes(error.code)) {
        t.skip("Windows symlink privilege unavailable"); return;
      }
      throw error;
    }
    Object.assign(item, {state:"MISSING", sizeBytes:null, sha256:null, authenticode:"NOT_EXECUTED"});
    await assert.rejects(verifyArtifactFiles(f.manifest, f.report, f.root), /symlinked component/);
  } finally { f.cleanup(); rmSync(outside, {recursive:true, force:true}); }
});
