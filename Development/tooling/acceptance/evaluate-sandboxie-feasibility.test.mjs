import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { evaluateSandboxieFeasibility } from "./evaluate-sandboxie-feasibility.mjs";

const sample = () => ({
  schema: 1,
  scope: "SANDBOXIE_TWO_CLIENT_FEASIBILITY",
  stage: "TWO_CLIENT",
  evidenceOrigin: "WINDOWS_HOST",
  automaticVerdict: "NOT_PROVIDED",
  sandboxie: {
    toolsPresent: true,
    toolSignaturesValid: true,
    serviceRunning: true,
    boxConfigured: "YES",
    boxSettingsSource: "BOX_SECTION_ENUMERATED",
    autoDelete: "OFF",
    autoRemove: "OFF",
    credentialHostAccess: "OFF",
    membershipStatus: "OBSERVED"
  },
  machine: {
    interactiveSession: true,
    processEvidenceClass: "NAME_MATCH_CANDIDATE_ONLY",
    candidateProcesses: [
      { pid: 101, windowHandle: "0x100", inSelectedBox: false },
      { pid: 202, windowHandle: "0x200", inSelectedBox: true }
    ]
  },
  operator: {
    attribution: "OPERATOR_DECLARED_NOT_MACHINE_VERIFIED",
    licensedForUse: "YES",
    nativeWorldPlayable: "YES",
    sandboxWorldPlayable: "YES",
    distinctAccounts: "YES",
    independentInput: "YES",
    accountCrossingObserved: "NO",
    reopenAndRebootPersistent: "YES",
    multiplayerJoined: "YES",
    observedMinutes: 10,
    evidenceReference: "sandbox-session-001"
  }
});

test("positive operator claims never become independently verified Microsoft sessions", () => {
  const result = evaluateSandboxieFeasibility(sample());
  assert.equal(result.verdict, "OPERATOR_REPORTED_COMPATIBLE");
  assert.equal(result.nativeVisibleCandidates, 1);
  assert.equal(result.isolatedVisibleCandidates, 1);
  assert.equal(result.automatedAccountProof, false);
  assert.equal(result.automatedGameProof, false);
  assert.equal(result.productionFourClientReady, false);
  assert.ok(!result.risks.includes("MULTIPLAYER_NOT_CONFIRMED"));
});

test("inspecting the box without game observations is inconclusive", () => {
  const r = sample();
  r.stage = "INSPECT";
  for (const k of ["licensedForUse", "nativeWorldPlayable", "sandboxWorldPlayable",
    "distinctAccounts", "independentInput", "accountCrossingObserved",
    "reopenAndRebootPersistent", "multiplayerJoined"]) {
    r.operator[k] = "UNKNOWN";
  }
  r.operator.observedMinutes = 0;
  r.operator.evidenceReference = null;
  assert.equal(evaluateSandboxieFeasibility(r).verdict, "INCONCLUSIVE");
});

test("automatic deletion or removal blocks even when operator claims success", () => {
  for (const change of [
    r => r.sandboxie.autoDelete = "ON",
    r => r.sandboxie.autoRemove = "ON",
    r => r.sandboxie.credentialHostAccess = "ON",
    r => r.sandboxie.boxConfigured = "NO"
  ]) {
    const r = sample();
    change(r);
    // A disabled box cannot have trustworthy observed PID membership.
    if (r.sandboxie.boxConfigured === "NO") {
      r.sandboxie.membershipStatus = "UNKNOWN";
      r.sandboxie.boxSettingsSource = "UNKNOWN";
      for (const p of r.machine.candidateProcesses) p.inSelectedBox = null;
    }
    const result = evaluateSandboxieFeasibility(r);
    assert.equal(result.verdict, "CONFIGURATION_BLOCKED");
  }
});

test("unknown persistence settings never pass as disabled", () => {
  for (const key of ["autoDelete", "autoRemove", "credentialHostAccess"]) {
    const r = sample();
    r.sandboxie[key] = "UNKNOWN";
    const result = evaluateSandboxieFeasibility(r);
    assert.equal(result.verdict, "INCONCLUSIVE");
    assert.ok(result.risks.length > 0);
  }
});

test("unknown PID membership and incomplete visible game inventory are inconclusive", () => {
  for (const change of [
    r => {
      r.sandboxie.membershipStatus = "UNKNOWN";
      r.machine.candidateProcesses.forEach(p => p.inSelectedBox = null);
    },
    r => r.machine.candidateProcesses[1].windowHandle = null,
    r => r.machine.candidateProcesses[0].windowHandle = null,
    r => r.machine.candidateProcesses[1].inSelectedBox = false,
    r => r.machine.interactiveSession = false
  ]) {
    const r = sample();
    change(r);
    assert.equal(evaluateSandboxieFeasibility(r).verdict, "INCONCLUSIVE");
  }
});

test("unknown account isolation, license, persistence or observation reference stays inconclusive", () => {
  for (const change of [
    r => r.operator.distinctAccounts = "UNKNOWN",
    r => r.operator.licensedForUse = "UNKNOWN",
    r => r.operator.reopenAndRebootPersistent = "UNKNOWN",
    r => r.operator.multiplayerJoined = "UNKNOWN",
    r => r.operator.observedMinutes = 9,
    r => r.operator.evidenceReference = null
  ]) {
    const r = sample();
    change(r);
    assert.equal(evaluateSandboxieFeasibility(r).verdict, "INCONCLUSIVE");
  }
});

test("explicit cross-account, license, gameplay or persistence failure remains visible", () => {
  for (const change of [
    r => r.operator.accountCrossingObserved = "YES",
    r => r.operator.licensedForUse = "NO",
    r => r.operator.nativeWorldPlayable = "NO",
    r => r.operator.sandboxWorldPlayable = "NO",
    r => r.operator.reopenAndRebootPersistent = "NO",
    r => r.operator.multiplayerJoined = "NO"
  ]) {
    const r = sample();
    change(r);
    assert.equal(evaluateSandboxieFeasibility(r).verdict, "OPERATOR_REPORTED_FAILURE");
  }
});

test("untrusted process attribution or invented automatic PASS is rejected", () => {
  for (const change of [
    r => r.scope = "NATIVE_TWO_CLIENT_FEASIBILITY",
    r => r.automaticVerdict = "PASS",
    r => r.machine.processEvidenceClass = "VERIFIED_GAME",
    r => r.operator.attribution = "AUTOMATED",
    r => r.machine.candidateProcesses[1].pid = 101,
    r => r.machine.candidateProcesses[1].windowHandle = "0x100",
    r => r.machine.candidateProcesses[1].windowHandle = "0x0100",
    r => r.machine.candidateProcesses[1].windowHandle = "0x00000100",
    r => r.machine.candidateProcesses[1].pid = 0x100000000,
    r => r.machine.candidateProcesses[1].windowHandle = "0x10000000000000000",
    r => r.machine.candidateProcesses[0].windowHandle = "0x0",
    r => r.operator.evidenceReference = "C:\\Accounts\\secret",
    r => r.operator.observedMinutes = 1441,
    r => r.sandboxie.membershipStatus = "UNKNOWN",
    r => r.sandboxie.toolSignaturesValid = false
  ]) {
    const r = sample();
    change(r);
    assert.throws(() => evaluateSandboxieFeasibility(r), TypeError);
  }
});

test("no box membership can be reported with missing tools, service or enabled box", () => {
  for (const change of [
    r => r.sandboxie.toolsPresent = false,
    r => r.sandboxie.serviceRunning = false,
    r => r.sandboxie.boxConfigured = "UNKNOWN"
  ]) {
    const r = sample();
    change(r);
    assert.throws(() => evaluateSandboxieFeasibility(r), TypeError);
  }
});

test("inspection mode rejects user-injected game or licensing assertions", () => {
  const r = sample(); r.stage = "INSPECT";
  assert.throws(() => evaluateSandboxieFeasibility(r), TypeError);
});


test("box-local absent-by-default settings can be OFF only after successful box enumeration", () => {
  const actual = sample();
  assert.equal(evaluateSandboxieFeasibility(actual).verdict, "OPERATOR_REPORTED_COMPATIBLE");
  assert.equal(evaluateSandboxieFeasibility(actual).boxLocalSettingsVerified, true);

  // Backwards-compatible older schema-1 reports without provenance are
  // readable, but can no longer assert a persistent Sandboxie box.
  const previous = sample();
  delete previous.sandboxie.boxSettingsSource;
  const old = evaluateSandboxieFeasibility(previous);
  assert.equal(old.verdict, "INCONCLUSIVE");
  assert.equal(old.boxLocalSettingsVerified, false);
  assert.ok(old.risks.includes("BOX_LOCAL_SETTINGS_NOT_VERIFIED"));

  const failedEnumeration = sample();
  failedEnumeration.sandboxie.boxSettingsSource = "UNKNOWN";
  const unknown = evaluateSandboxieFeasibility(failedEnumeration);
  assert.equal(unknown.verdict, "INCONCLUSIVE");
  assert.ok(unknown.risks.includes("BOX_LOCAL_SETTINGS_NOT_VERIFIED"));
});

test("rejects a forged local-settings result when box tools, service or box are unavailable", () => {
  for (const alter of [
    r => r.sandboxie.toolsPresent = false,
    r => r.sandboxie.toolSignaturesValid = false,
    r => r.sandboxie.serviceRunning = false,
    r => r.sandboxie.boxConfigured = "UNKNOWN"
  ]) {
    const r = sample();
    alter(r);
    assert.throws(() => evaluateSandboxieFeasibility(r), TypeError);
  }
  const r = sample();
  r.sandboxie.boxSettingsSource = "GLOBAL_SETTINGS_INHERITED";
  assert.throws(() => evaluateSandboxieFeasibility(r), TypeError);
});


test("process and operator-account evidence are distinct from session persistence and final acceptance", () => {
  const processOnly = sample();
  processOnly.stage = "INSPECT";
  for (const key of ["licensedForUse", "nativeWorldPlayable", "sandboxWorldPlayable",
    "distinctAccounts", "independentInput", "accountCrossingObserved",
    "reopenAndRebootPersistent", "multiplayerJoined"]) {
    processOnly.operator[key] = "UNKNOWN";
  }
  processOnly.operator.observedMinutes = 0;
  processOnly.operator.evidenceReference = null;
  const early = evaluateSandboxieFeasibility(processOnly);
  assert.equal(early.processFeasibility, "TWO_WINDOW_CANDIDATES_OBSERVED");
  assert.equal(early.accountFeasibility, "INCONCLUSIVE");
  assert.equal(early.productAcceptance, "NOT_EVALUATED");

  const accountOnly = sample();
  accountOnly.operator.reopenAndRebootPersistent = "UNKNOWN";
  const reported = evaluateSandboxieFeasibility(accountOnly);
  assert.equal(reported.accountFeasibility, "OPERATOR_REPORTED_PLAYABLE");
  assert.equal(reported.verdict, "INCONCLUSIVE");
  assert.equal(reported.productAcceptance, "NOT_EVALUATED");

  const unknownMultiplayer = sample();
  unknownMultiplayer.operator.multiplayerJoined = "UNKNOWN";
  assert.equal(evaluateSandboxieFeasibility(unknownMultiplayer).accountFeasibility, "OPERATOR_REPORTED_PLAYABLE");
  assert.equal(evaluateSandboxieFeasibility(unknownMultiplayer).verdict, "INCONCLUSIVE");

  const failedMultiplayer = sample();
  failedMultiplayer.operator.multiplayerJoined = "NO";
  assert.equal(evaluateSandboxieFeasibility(failedMultiplayer).verdict, "OPERATOR_REPORTED_FAILURE");

  const failedPersistence = sample();
  failedPersistence.operator.reopenAndRebootPersistent = "NO";
  const failure = evaluateSandboxieFeasibility(failedPersistence);
  assert.equal(failure.accountFeasibility, "OPERATOR_REPORTED_PLAYABLE");
  assert.equal(failure.verdict, "OPERATOR_REPORTED_FAILURE");
});

test("older reports without credential isolation evidence stay inconclusive", () => {
  const old = sample();
  delete old.sandboxie.credentialHostAccess;
  const observed = evaluateSandboxieFeasibility(old);
  assert.equal(observed.verdict, "INCONCLUSIVE");
  assert.ok(observed.risks.includes("HOST_CREDENTIAL_ISOLATION_UNKNOWN"));
});

test("source-side collector must distinguish missing box keys from failed queries", () => {
  // Protect the Windows implementation contract even when PowerShell cannot
  // execute in the current ChatGPT/GitHub-only development context.
  const source = readFileSync(new URL("./windows/collect-sandboxie-feasibility.ps1", import.meta.url), "utf8");
  assert.ok(source.includes("'BOX_SECTION_ENUMERATED'"));
  assert.ok(source.includes("'query ' + $BoxName + ' *'"));
  assert.ok(source.includes("if ($null -ne $rawNames)"));
  assert.ok(source.includes("if ($names -contains 'AutoDelete')"));
  assert.ok(source.includes("if ($names -contains 'AutoRemove')"));
  assert.ok(source.includes("else { 'n' }"));
  assert.ok(!source.includes("Read-BoxSetting 'GlobalSettings' 'AutoDelete'"));
  assert.ok(!source.includes("Read-BoxSetting 'GlobalSettings' 'AutoRemove'"));
  assert.ok(source.includes("Read-BoxSetting 'GlobalSettings' 'OpenCredentials'"));
  assert.ok(source.includes("credentialHostAccess = $credentialHostAccess"));
  assert.ok(source.includes("foreach ($section in @($BoxName, 'GlobalSettings'))"));
  assert.ok(source.includes("$rawTemplates = Invoke-BoundedReadOnlyTool"));
  assert.ok(source.includes("$sectionNames -contains 'OpenProtectedStorage'"));
  assert.ok(source.includes("if ($null -eq $rawTemplates) { $credentialValue = $null; break }"));
  assert.equal(
    source.split("$rawGlobal = Invoke-BoundedReadOnlyTool $iniExe 'query GlobalSettings *'").length - 1,
    1
  );
  assert.equal(source.split("credentialHostAccess = $credentialHostAccess").length - 1, 2);
  assert.ok(source.includes("[switch]$SummaryOnly"));
  assert.ok(source.includes("$reader.ReadAsync($buffer, 0, $buffer.Length)"));
  assert.ok(source.includes("$remainingMs = 5000 - [int]$clock.ElapsedMilliseconds"));
  assert.ok(source.includes("$readTask = $reader.ReadAsync($buffer, 0, $buffer.Length)"));
  assert.ok(source.indexOf("$readTask.Wait($remainingMs)") < source.indexOf("$process.WaitForExit($remainingMs)"));
  assert.ok(source.includes("$first = Read-BoxPidList"));
  assert.ok(source.includes("$second = Read-BoxPidList"));
  // A failed first query cannot produce a verified two-snapshot inventory.
  // Never launch the second Start.exe helper just to discard its result.
  const firstRead = source.indexOf("$first = Read-BoxPidList");
  const secondGuard = source.indexOf("if ($null -ne $first) {", firstRead);
  const secondRead = source.indexOf("$second = Read-BoxPidList", secondGuard);
  assert.ok(firstRead >= 0 && secondGuard > firstRead && secondRead > secondGuard);
  assert.ok(source.includes("$second = $null"));
  // Valid snapshots must still agree; the verified DLL fallback is preserved.
  assert.ok(source.includes("(($first.Pids -join ',') -ceq ($second.Pids -join ','))"));
  assert.ok(source.includes("if ($membershipStatus -ne 'OBSERVED' -and"));
  assert.ok(source.includes("if ($output.Length + $received -gt 4096) { return $null }"));
  assert.ok(!source.includes("$process.StandardOutput.ReadToEndAsync()"));
  assert.ok(source.includes("$process.StandardOutput.Dispose()"));
  assert.ok(source.includes("schema = 1")); // durable full CLI report
  assert.ok(source.includes("schema = 6")); // on-demand desktop summary only
  assert.ok(source.includes("isolatedCandidateProcesses = @($gameCandidates | Where-Object {"));
  assert.ok(source.includes("createdFileTime = $createdFileTime"));
  assert.ok(source.includes("imageEvidence = $imageEvidence"));
  assert.ok(source.includes("officialImageOutsideSelectedBoxCount = $signedOutside.Count"));
  assert.ok(source.includes("$signedOutside = @($gameCandidates | Where-Object {"));
  assert.ok(source.includes("$_.imageEvidence -eq 'SIGNED_EDUCATION_IMAGE_CANDIDATE' -and"));
  assert.ok(source.includes("$_.inSelectedBox -eq $false"));
  assert.ok(source.includes("$educationImageProof = @{}"));
  assert.ok(source.includes("Get-AuthenticodeSignature -LiteralPath $image"));
  assert.ok(!source.includes("query /boxes *")); // replaced by per-PID API
  assert.ok(source.includes("SbieApi_QueryProcess"));
  assert.ok(source.includes("GetProcAddress(module, \"SbieApi_QueryProcess\")"));
  assert.ok(source.includes("GetProcessTimes(process,"));
  assert.ok(source.includes("SameProcess(process, expected)"));
  assert.ok(source.includes("LoadLibraryExW(dll, IntPtr.Zero"));
  assert.ok(source.includes("Get-AuthenticodeSignature -LiteralPath $dll"));
  assert.ok(source.includes("$dllSig.SignerCertificate.Thumbprint -eq"));
  assert.ok(source.includes("if ($ImageProof -and $signedOutside.Count -gt 0)"));
  assert.ok(source.includes("$directPidQuery = 'UNAVAILABLE'"));
  assert.ok(source.includes("$directPidQuery = 'QUERIED'"));
  assert.ok(source.includes("$signedOtherBoxCount"));
  assert.ok(source.includes("$signedUnknownBoxCount"));
  assert.ok(source.includes("directPidQuery = $directPidQuery"));
  assert.ok(!source.includes("imagePath = $image"));
  assert.ok(source.includes("processCandidates = @($gameCandidates)"));
  assert.ok(source.includes("candidateProcesses = @($gameCandidates | ForEach-Object {"));
  assert.ok(source.includes("if (-not $SummaryOnly)"));
  assert.ok(source.includes("if ($null -ne $path) { $summary['reportPath'] = $path }"));
  assert.equal(source.split("$boxConfigured = if ($enabled").length - 1, 1);
  assert.ok(source.includes("$membershipStatus = 'UNKNOWN'"));
  // A successful Sandboxie box scan cannot authorize a launch if the Windows
  // process enumeration fails or omits any PID reported inside the box.
  assert.ok(source.includes("$allProcesses = @(Get-Process -ErrorAction Stop)"));
  assert.ok(source.includes("if (-not $processInventoryComplete)"));
  assert.ok(source.includes("foreach ($boxedPid in $boxedPids)"));
  assert.ok(source.includes("if (-not $observedByPid.ContainsKey($key))"));
  assert.ok(source.includes("if ($null -eq $rawSession -or $rawSession -isnot [int])"));
  assert.ok(source.includes("$processSession -lt 0 -or $processSession -ne $session"));
  assert.ok(source.includes("$boxedIdentityByPid[$key] = [pscustomobject]@{"));
  assert.ok(source.includes("$allProcesses = @()"));
  assert.ok(source.includes("$handle = $null"));
  // A boxed PID with an inaccessible/missing session or process name
  // may be a Minecraft client. It cannot be silently discarded as a helper.
  assert.ok(source.includes("$rawSession = $process.SessionId"));
  assert.ok(source.includes("if ($null -eq $rawSession -or $rawSession -isnot [int])"));
  assert.ok(source.includes("$processName = [string]$process.ProcessName"));
  assert.ok(source.includes("$boxedIdentityByPid.ContainsKey($candidateKey)"));
  assert.ok(source.includes("$boxedIdentityByPid[$candidateKey].Name -match"));
  assert.ok(source.includes("$allProcesses = @()"));
  assert.ok(source.includes("if ($_.MainWindowHandle -ne [IntPtr]::Zero)"));
  assert.ok(source.includes("Window handles may be unavailable"));
  assert.ok(!source.includes("Get-Process -ErrorAction SilentlyContinue"));
});
