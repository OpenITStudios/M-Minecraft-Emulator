import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { verifyManifest, verifyReport } from "./validate-baseline.mjs";

// Contract fixtures are derived from the pinned complete manifest, so that
// test cases prove rejection of reduced or reordered real inventories.
const baseline = JSON.parse(readFileSync(
  new URL("./upstream-baseline.json", import.meta.url), "utf8"));
function fixture() { return structuredClone(baseline); }
function report(m, compiled = true) {
  return {
    schema: 1, sourceRepository: m.upstream.repository, sourceTag: m.upstream.tag,
    sourceCommit: m.upstream.commit, sourceCleanAtStart: true,
    compiledInThisInvocation: compiled,
    stageResults: m.buildStages.map(s => ({
      ...s, state: compiled ? "COMPILED" : "NOT_EXECUTED",
      exitCode: compiled ? 0 : null,
    })),
    artifacts: [
      ...m.requiredArtifacts.map(path => ({ path, kind: "required" })),
      ...m.optionalSymbols.map(path => ({ path, kind: "optional-symbol" })),
      ...m.sourceResources.map(path => ({ path, kind: "source-resource" })),
    ].map(a => ({
      ...a, state: "PRESENT", sizeBytes: 4096,
      sha256: "a".repeat(64), authenticode: "NotSigned",
    })),
    artifactResult: compiled ? "BUILD_ARTIFACTS_COLLECTED" : "COLLECTION_ONLY_NOT_BUILD_PROOF",
    signingLoadAcceptance: "NOT_EXECUTED",
    minecraftLoginGameplayAcceptance: "NOT_EXECUTED", distributionApproval: "NOT_APPROVED",
  };
}
const clone = value => structuredClone(value);
test("pins upstream version, stage order and critical x86/x64 artifacts", () => {
  assert.equal(verifyManifest(fixture()).status, "MANIFEST_VALID");
  const bad = fixture(); bad.buildStages.reverse();
  assert.throws(() => verifyManifest(bad), /build stages/);
  const wrong = fixture(); wrong.upstream.commit = "0".repeat(40);
  assert.throws(() => verifyManifest(wrong), /upstream pin/);
  const missing = fixture(); missing.requiredArtifacts.pop();
  assert.throws(() => verifyManifest(missing), /complete requiredArtifacts inventory/);
});
test("rejects path traversal, duplicates and removal of core GPL notice", () => {
  const bad = fixture(); bad.optionalSymbols.push("../malicious.pdb");
  assert.throws(() => verifyManifest(bad), /invalid artifact path/);
  const dup = fixture(); dup.optionalSymbols.push(dup.requiredArtifacts[0]);
  assert.throws(() => verifyManifest(dup), /invalid artifact path/);
  const license = fixture(); license.upstream.sourceLicenseFiles = [];
  assert.throws(() => verifyManifest(license), /complete license inventory/);
});
test("collection-only and build reports never establish runnable driver or gameplay", () => {
  const m = fixture();
  assert.equal(verifyReport(m, report(m, false)).artifactResult, "COLLECTION_ONLY_NOT_BUILD_PROOF");
  const built = verifyReport(m, report(m));
  assert.equal(built.artifactResult, "BUILD_ARTIFACTS_COLLECTED");
  assert.equal(built.driverLoadVerified, false);
  assert.equal(built.gameplayVerified, false);
  assert.equal(built.cutoverApproved, false);
});
test("rejects forged compile stages or report source revision", () => {
  const m = fixture(); const r = report(m, false);
  r.stageResults[0].state = "COMPILED";
  assert.throws(() => verifyReport(m, r), /stage evidence/);
  const forged = report(m); forged.sourceCommit = "0".repeat(40);
  assert.throws(() => verifyReport(m, forged), /provenance/);
});
test("rejects missing rows, corrupted hashes and false runtime PASS", () => {
  const m = fixture(); let r = report(m);
  r.artifacts[0].sha256 = "bad"; assert.throws(() => verifyReport(m, r), /hash/);
  r = report(m); r.artifacts.pop(); assert.throws(() => verifyReport(m, r), /missing or extra/);
  r = report(m); r.signingLoadAcceptance = "PASS";
  assert.throws(() => verifyReport(m, r), /fabricated runtime/);
});
test("incomplete mandatory output cannot be reported as successful; missing symbols are optional", () => {
  const m = fixture(); const r = report(m);
  r.artifacts[0] = {
    ...r.artifacts[0], state: "MISSING", sizeBytes: null, sha256: null,
    authenticode: "NOT_EXECUTED",
  };
  assert.throws(() => verifyReport(m, r), /false result/);
  r.artifactResult = "BUILD_OUTPUT_INCOMPLETE";
  assert.equal(verifyReport(m, r).missingRequired, 1);
  const symbols = report(m); const i = m.requiredArtifacts.length;
  symbols.artifacts[i] = {
    ...symbols.artifacts[i], state: "MISSING", sizeBytes: null,
    sha256: null, authenticode: "NOT_EXECUTED",
  };
  assert.equal(verifyReport(m, symbols).artifactResult, "BUILD_ARTIFACTS_COLLECTED");
});

test("pins COMPLETE binary, helper, optional symbol, resource and license closure", () => {
  for (const group of ["requiredArtifacts", "optionalSymbols", "sourceResources"]) {
    const missing = fixture(); missing[group].splice(Math.floor(missing[group].length/2), 1);
    assert.throws(() => verifyManifest(missing), /complete .* inventory/);
    const reordered = fixture(); reordered[group].reverse();
    assert.throws(() => verifyManifest(reordered), /complete .* inventory/);
    const extra = fixture(); extra[group].push("Sandboxie/extra.txt");
    assert.throws(() => verifyManifest(extra), /complete .* inventory/);
  }
  for (const license of [
    "LICENSE.Plus", "Sandboxie/COPYING", "SandboxiePlus/QSbieAPI/LICENSE"
  ]) {
    const missing = fixture();
    missing.upstream.sourceLicenseFiles =
      missing.upstream.sourceLicenseFiles.filter(path => path !== license);
    assert.throws(() => verifyManifest(missing), /complete license inventory/);
  }
});

test("baseline Windows build preserves reports, logs and existing staging paths", () => {
  const script = readFileSync(new URL("./build-baseline.ps1", import.meta.url), "utf8");
  assert.match(script, /Set-StrictMode -Version Latest/);
  assert.match(script, /Baseline build report already exists/);
  assert.match(script, /Baseline stage output already exists/);
  assert.match(script, /Baseline build log already exists/);
  assert.match(script, /Baseline payload cannot overlap upstream source checkout/);
  assert.match(script, /Baseline stage parent directory must exist before compilation/);
  assert.match(script, /Baseline build report must not be inside the staged payload directory/);
  assert.match(script, /Baseline build report path collides with a reserved stage log/);
  assert.match(script, /@\(\$spec\.requiredArtifacts\) \+ @\(\$spec\.optionalSymbols\)/);
  assert.match(script, /Out-File -LiteralPath \$logPath -Encoding UTF8 -NoClobber/);
  assert.match(script, /Out-File -LiteralPath \$reportFull -Encoding UTF8 -NoClobber/);
  // Source checkout can be CRLF on Windows; preserve ordering/guard assertion.
  const scriptLines = script.replace(/\r\n/g, "\n");
  assert.ok(scriptLines.indexOf("Baseline build log already exists") <
    scriptLines.indexOf("foreach ($stage in @($spec.buildStages)) {\n    $stageResult"),
    "prior logs must be rejected before any MSBuild invocation");
  assert.doesNotMatch(script, /\b(?:Start-Service|Stop-Service|sc\.exe)\b/i);
});

test("archived v1.18.5 baseline handoff retains distinct report examples without becoming current", () => {
  const guide = readFileSync(new URL("../../planning/history/isolation-v1185-source-notes.md", import.meta.url), "utf8");
  const current = readFileSync(new URL("./README.md", import.meta.url), "utf8");
  assert.match(guide, /^# Archived Sandboxie v1\.18\.5 source investigations/);
  assert.match(guide, /HISTORICAL \/ NON-EXECUTABLE GUIDE/);
  assert.doesNotMatch(current, /build-baseline\.ps1\s+-SourceRoot/);
  assert.match(current, /build-owned-v1186\.ps1/);
  const lines = guide.split("\n").filter(line => line.includes("build-baseline.ps1") &&
    line.includes("-ReportPath") && !line.trim().startsWith("#"));
  assert.equal(lines.length, 3, "baseline handoff must show the collection and two alternative build commands");
  const paths = lines.map(line => line.match(/-ReportPath '([^']+)'/)?.[1]);
  assert.ok(paths.every(Boolean), "every build example must name an output report");
  assert.notEqual(paths[0], paths[1], "collection-only must not block the later build report");
  assert.equal(paths[1], paths[2], "build and build+stage are alternatives, not sequential runs");
  assert.ok(guide.includes("Do not run both `-Build` examples"));
});
