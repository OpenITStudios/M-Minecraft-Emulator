import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
const read = (path) => readFileSync(resolve(root, path), "utf8");
const json = (path) => JSON.parse(read(path));
const requireFact = (condition, message) => {
  if (!condition) throw new Error(message);
};
const required = [
  "README.md", "AGENTS.md", "GITHUB_RULES.md", "CONTEXT.md", "CONTRIBUTING.md",
  ".gitmodules",
  "SECURITY.md", "VERSION", "DEV.cmd", ".agents/skills/emulator-development/SKILL.md",
  "planning/development.md", "docs/README.md", "docs/system/implementation-map.md",
  "tooling/local-test/session.mjs", "tooling/local-test/session.test.mjs",
  "tooling/isolation-source/upstream-baseline.json",
  "tooling/isolation-source/build-baseline.ps1",
  "tooling/isolation-source/validate-baseline.mjs",
  "tooling/isolation-source/validate-baseline.test.mjs",
  "tooling/isolation-source/verify-artifact-files.mjs",
  "tooling/isolation-source/verify-artifact-files.test.mjs",
  "tooling/isolation-source/bootstrap-fork.mjs",
  "tooling/isolation-source/bootstrap-fork.test.mjs",
  "tooling/isolation-source/verify-checkout.mjs",
  "tooling/isolation-source/verify-checkout.test.mjs",
  "tooling/isolation-source/assemble-engine.mjs",
  "tooling/isolation-source/assemble-engine.test.mjs",
  "tooling/isolation-source/assemble-private-engine.mjs",
  "tooling/isolation-source/assemble-private-engine.test.mjs",
  "tooling/isolation-source/provision-disposable.mjs",
  "tooling/isolation-source/provision-disposable.test.mjs",
  "tooling/isolation-source/plan-disposable.mjs",
  "tooling/isolation-source/execute-disposable.mjs",
  "tooling/isolation-source/execute-disposable.test.mjs",
  "tooling/isolation-source/simulate-disposable.mjs",
  "tooling/isolation-source/windows/probe-disposable-readiness.ps1",
  "tooling/isolation-source/windows/transport-commands.mjs",
  "tooling/isolation-source/windows/transport-commands.test.mjs",
  "tooling/isolation-source/windows/kmdutil-transport.ps1",
  "tooling/isolation-source/windows/attestation-gate.mjs",
  "tooling/isolation-source/windows/attestation-gate.test.mjs",
  "tooling/isolation-source/windows/service-ownership.mjs",
  "tooling/isolation-source/windows/service-ownership.test.mjs",
  "tooling/isolation-source/windows/probe-owned-service-image.ps1",
  "tooling/isolation-source/windows/ownership-ledger.mjs",
  "tooling/isolation-source/windows/ownership-ledger.test.mjs",
  "tooling/isolation-source/windows/review-ownership.mjs",
  "tooling/isolation-source/windows/reconcile-ownership.mjs",
  "tooling/isolation-source/windows/reconcile-ownership.test.mjs",
  "tooling/isolation-source/windows/reconcile-ownership-cli.mjs",
  "engine/isolation-native/CMakeLists.txt",
  "engine/isolation-native/src/abi_probe.cpp",
  "engine/isolation-native/fork-overlay/apply-private-source.mjs",
  "engine/isolation-native/fork-overlay/apply-private-source.test.mjs",
  "engine/isolation-native/fork-overlay/build-private.ps1",
  "engine/isolation-native/README.md",
  "apps/desktop/package.json", "apps/desktop/package-lock.json",
  "apps/desktop/src-tauri/Cargo.toml", "apps/desktop/src-tauri/tauri.conf.json",
  "engine/runtime-core/Cargo.toml", "engine/distribution/release-channel.json",
  "tooling/acceptance/windows/collect-acceptance.ps1",
  "tooling/acceptance/windows/collect-native-performance.ps1",
  "tooling/acceptance/evaluate-native-performance.mjs",
  "tooling/acceptance/evaluate-native-performance.test.mjs",
  "engine/guest/windows/prepare-base.ps1",
  "docs/validation/client-feasibility.md",
  "docs/validation/pre-local-test-readiness.md",
  "docs/product/four-concurrent-accounts.md",
  "apps/desktop/src-tauri/src/engine/native_clients.rs",
  "apps/desktop/src-tauri/src/engine/owned_isolation.rs",
  "tooling/isolation-source/windows/probe-host-service-names.ps1",
  "apps/desktop/test/owned-isolation.test.ts",
  "apps/desktop/src-tauri/src/commands/native_clients.rs",
  "apps/desktop/src/NativeFeasibilityApp.svelte",
  "apps/desktop/src/CompactManagerApp.svelte",
  "apps/desktop/src/app/presentation/compactManager.ts",
  "docs/product/compact-manager-implementation.md",
  "apps/desktop/src/app/bridge/nativeFeasibilityApi.ts",
  "docs/system/legacy-retirement.md",
  "tooling/acceptance/windows/collect-client-feasibility.ps1",
  "tooling/acceptance/evaluate-client-feasibility.mjs",
  "tooling/acceptance/evaluate-client-feasibility.test.mjs"
];
for (const file of required) {
  requireFact(existsSync(resolve(root, file)), "Missing canonical repository owner: " + file);
}
// The aggregate G0 source route requests an extra reproducibility prerequisite.
// Keep normal repository structure inspections available independently while the
// actual Tauri dependency graph has not yet been generated.
if (process.argv.includes("--g0")) {
  const desktopLock = "apps/desktop/src-tauri/Cargo.lock";
  requireFact(existsSync(resolve(root, desktopLock)),
    "G0 BLOCKED: Tauri Cargo.lock is missing. Resolve it with real Cargo; do not fabricate versions.");
  // A generated but uncommitted file cannot be the reproducible exact-SHA G0
  // dependency graph. Inspect HEAD directly instead of trusting the Git index.
  const committed = spawnSync("git", ["ls-tree", "-r", "--name-only", "HEAD", "--", desktopLock], {
    cwd: root,
    encoding: "utf8",
  });
  requireFact(!committed.error && committed.status === 0 &&
    committed.stdout.trim() === desktopLock,
    "G0 BLOCKED: Tauri Cargo.lock exists locally but is not committed in HEAD. Commit it to Local before source/package verification.");
}

// Root governance continuity is source-owned; keep it within this verifier.
for (const [file, markers] of Object.entries({
  "AGENTS.md": ["ChatGPT-GitHub Work Continuity", "Short-prompt repository startup", "Cross-chat recovery"],
  "GITHUB_RULES.md": ["Short-prompt repository startup", "Commit-based continuity", "Work:", "State:", "Proof:"],
  "CONTRIBUTING.md": ["GitHub-first", "planning/development.md"]
})) {
  const content = read(file);
  for (const marker of markers) {
    requireFact(content.includes(marker), "Missing governance continuity marker in " + file + ": " + marker);
  }
}
for (const retired of ["virtual-clients", "apps/virtual-clients", "tooling/verify-repository.mjs"]) {
  requireFact(!existsSync(resolve(root, retired)), "Obsolete monorepo path remains: " + retired);
}

const version = read("VERSION").trim();
const rootPackage = json("package.json");
const appPackage = json("apps/desktop/package.json");
const lock = json("apps/desktop/package-lock.json");
const tauri = json("apps/desktop/src-tauri/tauri.conf.json");
const tauriPackage = json("apps/desktop/src-tauri/tauri.package.conf.json");
const release = json("engine/distribution/release-channel.json");
const toolchain = json("toolchain.json");
const cargoVersion = (path) => /^version\s*=\s*"([^"]+)"/m.exec(read(path))?.[1];

requireFact(rootPackage.name === "m-minecraft-emulator", "Root package identity drift");
requireFact(version === rootPackage.version && version === appPackage.version &&
  version === tauri.version && version === cargoVersion("apps/desktop/src-tauri/Cargo.toml") &&
  version === cargoVersion("engine/runtime-core/Cargo.toml"), "Product version mismatch");
requireFact(toolchain.node.version === read(".node-version").trim(), "Node pin mismatch");
requireFact(toolchain.npm.lockfile === "apps/desktop/package-lock.json", "App lockfile path drift");
requireFact(toolchain.policy.ciPolicy === "disabled", "CI must remain disabled per operator request");
// The desktop depends on the already-locked sha2 0.10.8. Its local package
// edge must be recorded or cargo --locked rejects this clean checkout.
const desktopLock = read("apps/desktop/src-tauri/Cargo.lock").replace(/\r\n/g, "\n");
const desktopPackageEntry = desktopLock.split('name = "m-bedrock-virtual-clients"\n')[1]
  ?.split("\n[[package]]")[0] ?? "";
requireFact(read("apps/desktop/src-tauri/Cargo.toml").includes('sha2 = "=0.10.8"') &&
  desktopPackageEntry.includes(' "sha2",') &&
  desktopLock.includes('name = "sha2"\nversion = "0.10.8"'),
  "Desktop Cargo.lock must reflect the pinned direct sha2 dependency");
requireFact(lock.packages[""]?.version === appPackage.version, "Desktop lockfile version mismatch");
for (const [name, range] of Object.entries({ ...appPackage.dependencies, ...appPackage.devDependencies })) {
  requireFact(lock.packages[""]?.dependencies?.[name] === range ||
    lock.packages[""]?.devDependencies?.[name] === range, "Lock missing dependency: " + name);
  requireFact(lock.packages["node_modules/" + name], "Lock missing installed package: " + name);
}
requireFact(rootPackage.scripts["verify:repository"] === "node tooling/repository/verify.mjs",
  "Root static verifier route drift");
requireFact(rootPackage.scripts["check:desktop"].includes("apps/desktop"), "Desktop route drift");
requireFact(rootPackage.scripts["check:runtime"].includes("engine/runtime-core/Cargo.toml"),
  "Runtime check route drift");
requireFact(rootPackage.scripts["verify:source"].includes("npm run verify:repository"),
  "Aggregate source check omits repository contract");
requireFact(rootPackage.scripts["test:feasibility"] ===
  "node --test tooling/acceptance/evaluate-client-feasibility.test.mjs",
  "Single guest feasibility regression route drift");
requireFact(rootPackage.scripts["verify:source"].includes("npm run test:feasibility"),
  "Aggregate source check must include client-feasibility evidence regressions");
requireFact(read("apps/desktop/src/App.svelte").includes(
  "!shouldShowClientControls(snapshot.doctor.nextSetupAction)"),
  "Client-owned setup actions must show real client controls, not the setup wizard");
const windowsToolchain = read("tooling/windows-toolchain/dev.ps1");
requireFact(windowsToolchain.includes("Test-DeveloperToolchain -AllowMissingDesktopLock -RustOnly") &&
  windowsToolchain.includes("Assert-CleanSourceAtHead -ExpectedSha $sha") &&
  windowsToolchain.includes("cargo generate-lockfile --manifest-path $DesktopManifest"),
  "Desktop Cargo.lock must use real Cargo on an exact clean Local checkout");
const guestUninstall = read("engine/guest/windows/uninstall-guest-agent.ps1");
requireFact(guestUninstall.includes("Remove-Item -LiteralPath $agent") &&
  !guestUninstall.includes("Remove-Item -LiteralPath $root -Recurse"),
  "Guest Agent uninstall must preserve unrelated guest runtime data");
const prepareBase = read("engine/guest/windows/prepare-base.ps1");
requireFact(prepareBase.includes("[Microsoft.Win32.RegistryView]::Registry32") &&
  prepareBase.includes("GetValue('Version', '')") &&
  read("engine/runtime-core/src/profile.rs").includes('"/reg:32"'),
  "Minecraft Education Base preparation must use the documented 32-bit Desktop registry view");
requireFact(read("docs/product/four-concurrent-accounts.md").includes(
  "four separate Minecraft Education Windows desktop clients concurrently"),
  "Four-client end-user outcome was removed from canonical product contract");
const activeEntry = read("apps/desktop/src/main.ts");
const activeSurface = read("apps/desktop/src/CompactManagerApp.svelte");
const rustCommandBoundary = read("engine/runtime-core/src/command.rs");
const activeTauri = read("apps/desktop/src-tauri/src/app_bootstrap.rs");
const activeCommandModules = read("apps/desktop/src-tauri/src/commands/mod.rs");
const activeEngineModules = read("apps/desktop/src-tauri/src/engine/mod.rs");
requireFact(activeCommandModules.includes("pub mod native_clients;") &&
  !/^\s*pub mod (setup_tools|virtual_clients|window_arrangement);/m.test(activeCommandModules) &&
  activeEngineModules.includes("pub mod native_clients;") &&
  !/^\s*pub mod (setup_tools|window_arrangement|screen_overlay);/m.test(activeEngineModules) &&
  !activeTauri.includes("screen_overlay::shutdown()"),
  "Legacy desktop Rust modules must remain outside the active native compile graph");
requireFact(activeEntry.includes('import CompactManagerApp from "./CompactManagerApp.svelte";') &&
  activeEntry.includes("mount(CompactManagerApp, { target });") &&
  !activeEntry.includes("NativeFeasibilityApp") &&
  !activeEntry.includes('import App from "./App.svelte";'),
  "The sole active desktop entry must mount the Compact Manager");
requireFact(activeSurface.includes("nativeFeasibility.openOfficialMinecraft()") &&
  activeSurface.includes("nativeFeasibility.inspectIsolatedClient02()") &&
  !activeSurface.includes("nativeFeasibility.inspectCandidates()") &&
  !activeSurface.includes("Promise.allSettled") &&
  !activeSurface.includes("setInterval(") &&
  !activeSurface.includes("onMount(") &&
  !activeSurface.includes("<aside") &&
  activeSurface.includes("<h1>Virtual Manager</h1>") &&
  activeSurface.includes("<h2>Minecraft sessions</h2>") &&
  activeSurface.includes("Settings") &&
  !activeSurface.includes("Start All") &&
  !activeSurface.includes("Stop All") &&
  !activeSurface.includes("backend.snapshot()") &&
  !activeSurface.includes("backend.startClient(") &&
  !activeSurface.includes("backend.provision("),
  "Active native/isolated pilot may not call legacy VM lifecycle commands");
requireFact(activeTauri.includes("commands::native_clients::desktop_open_native_minecraft,") &&
  activeTauri.includes("commands::native_clients::desktop_inspect_native_candidates,") &&
  !/commands::virtual_clients::virtual_clients_|commands::setup_tools::setup_|commands::window_arrangement::window_/.test(activeTauri),
  "Legacy VM handlers must not be registered in the active Tauri pilot");
const nativeBridge = read("apps/desktop/src/app/bridge/nativeFeasibilityApi.ts");
const nativeOwner = read("apps/desktop/src-tauri/src/engine/native_clients.rs");
const ownedEngine = read("apps/desktop/src-tauri/src/engine/owned_isolation.rs");
const ownedBridge = read("apps/desktop/src/app/bridge/nativeFeasibilityApi.ts");
requireFact(
  ownedEngine.includes("pub fn inspect_candidate(") &&
  ownedEngine.includes("CANDIDATE_INTEGRITY_VERIFIED_NOT_INSTALLABLE") &&
  ownedEngine.includes("install_permitted: false") &&
  ownedEngine.includes("pub fn inspect_host_service_namespace()") &&
  read("tooling/isolation-source/windows/probe-host-service-names.ps1").includes("Win32_SystemDriver") &&
  ownedBridge.includes('"desktop_inspect_owned_engine_host"') &&
  nativeOwner.includes("const ACTIVE_ISOLATION_BACKEND: OfficialSandboxieBackend") &&
  ownedBridge.includes('"desktop_inspect_owned_engine_candidate"') &&
  read("apps/desktop/src-tauri/src/app_bootstrap.rs").includes(
    "commands::native_clients::desktop_inspect_owned_engine_candidate,"),
  "Owned engine inspection must be candidate-only and never switch live provider");
const nativeProbeSource = read("engine/isolation-native/src/abi_probe.cpp");
const nativeProbeBuild = read("engine/isolation-native/CMakeLists.txt");
requireFact(
  nativeProbeSource.includes('#include "common/my_version.h"') &&
  nativeProbeSource.includes('SbieApi_GetVersionEx') &&
  nativeProbeSource.includes('ServiceImageMatches(L"SbieDrv"') &&
  nativeProbeSource.includes('ServiceImageMatches(L"SbieSvc"') &&
  nativeProbeSource.includes("driverSigningPolicyAccepted") &&
  nativeProbeSource.includes("installerReady") &&
  nativeProbeSource.includes("REJECTED_LOADED_DLL_PATH_MISMATCH") &&
  nativeProbeBuild.includes("abd4c240031cfb46666daa74ca47ed4e48b561e4") &&
  nativeProbeBuild.includes('source_abi_contract_selftest'),
  "The native owned-engine ABI probe must remain pinned and non-installing");
const privateAssembly = read("tooling/isolation-source/assemble-private-engine.mjs");
const privateAssembler = read("tooling/isolation-source/assemble-engine.mjs");
const ownedCandidate = read("apps/desktop/src-tauri/src/engine/owned_isolation.rs");
requireFact(
  privateAssembly.includes("PRIVATE_CORE_BINARIES_BUILT_NOT_SIGNED_OR_RUN") &&
  privateAssembly.includes("return assembleEngine(") &&
  privateAssembly.includes("checkReport(manifest,report)") &&
  privateAssembler.includes("PRIVATE_CORE_STAGED_NOT_INSTALLABLE") &&
  ownedCandidate.includes("PRIVATE_CORE_STAGED_NOT_INSTALLABLE") &&
  ownedCandidate.includes("can_coexist_with_official_sandboxie == Some(false)") &&
  ownedCandidate.includes("provider_selected: false") &&
  read("engine/isolation-native/fork-overlay/build-private.ps1").includes("--private-report $report"),
  "Private source binaries must stage through the sole assembler, remain exclusively noninstallable and be inspectable by the desktop");
const privateForkPatch = read("engine/isolation-native/fork-overlay/apply-private-source.mjs");
const privateForkBuild = read("engine/isolation-native/fork-overlay/build-private.ps1");
requireFact(
  privateForkPatch.includes('git(root, ["-c", "core.whitespace=cr-at-eol", "apply", "--check", "--index", "--whitespace=error", "-"], patch);') &&
  privateForkPatch.includes('git(root, ["-c", "core.whitespace=cr-at-eol", "apply", "--index", "--whitespace=error", "-"], patch);') &&
  privateForkPatch.includes("canCoexistWithOfficialSandboxie: false") &&
  privateForkPatch.includes("MIVUBI_PRIVATE_ISOLATION_ABI") &&
  privateForkBuild.includes("PRIVATE_CORE_BINARIES_BUILT_NOT_SIGNED_OR_RUN") &&
  privateForkBuild.includes("canCoexistWithOfficialSandboxie = $false") &&
  !privateForkBuild.includes("Start-Service") &&
  !privateForkBuild.includes("New-Service"),
  "Private fork source/build must be explicit, reviewed and non-installing");
const nativeAdapter = read("apps/desktop/src-tauri/src/commands/native_clients.rs");
requireFact(nativeBridge.includes('"desktop_inspect_native_candidates"') &&
  nativeBridge.includes("parseNativeObservation") &&
  !nativeBridge.includes("virtual_clients_"),
  "Native inspection needs a strict typed bridge without legacy backend access");
requireFact(nativeOwner.includes("pub fn inspect_native_candidates()") &&
  nativeOwner.includes("NAME_MATCH_CANDIDATE_ONLY") &&
  nativeAdapter.includes("fn desktop_inspect_native_candidates("),
  "Native inspection must be read-only and explicitly marked candidate-only");
const inspectOwner = nativeOwner.slice(
  nativeOwner.indexOf("pub fn inspect_native_candidates()"),
  nativeOwner.indexOf("pub fn open_native_minecraft()")
);
requireFact(!/Start-Process|Get-Credential|Set-Item|Remove-Item/.test(inspectOwner),
  "Native candidate inspection must not launch, mutate or read credentials");

requireFact(rustCommandBoundary.includes("legacy_operation_paused(command, args)") &&
  rustCommandBoundary.includes('"product-status" =>') &&
  rustCommandBoundary.includes('io::ErrorKind::Unsupported') &&
  rustCommandBoundary.includes('"app-sandbox-pilot"'),
  "Legacy VM command boundary must fail closed and expose read-only mode");
for (const operation of [
  "actions", "stage-update", "register-base", "open-base-finalization",
  "provision", "reprovision", "verify-identities", "start", "start-setup",
  "start-client", "suspend", "stop", "restart", "set-ready", "reset",
  "open", "launch-minecraft",
]) {
  requireFact(rustCommandBoundary.includes('"' + operation + '",'),
    "VM command pause list dropped: " + operation);
}
requireFact(rootPackage.scripts["test:native-performance"] ===
  "node --test tooling/acceptance/evaluate-native-performance.test.mjs" &&
  rootPackage.scripts["native-performance:review"] ===
  "node tooling/acceptance/evaluate-native-performance.mjs",
  "Native performance review and regression command drift");
const nativePerformanceSampler = read("tooling/acceptance/windows/collect-native-performance.ps1");
const nativePerformanceEvaluator = read("tooling/acceptance/evaluate-native-performance.mjs");
requireFact(
  nativePerformanceSampler.includes("automaticVerdict = 'NOT_PROVIDED'") &&
  nativePerformanceSampler.includes("OPERATOR_SELECTED_UNVERIFIED_PIDS") &&
  nativePerformanceEvaluator.includes('automaticGameplayProof: false') &&
  nativePerformanceEvaluator.includes('fourClientReady: false') &&
  !nativePerformanceSampler.includes("Stop-Process") &&
  !nativePerformanceSampler.includes("Start-Process"),
  "Native performance evidence must remain opt-in, non-mutating and never imply gameplay proof");
for (const command of ["test:native-feasibility", "test:sandboxie-feasibility", "test:native-performance"]) {
  requireFact(rootPackage.scripts["verify:source"].includes("npm run " + command),
    "Native-first source checks omit " + command);
}
const nativeEvidenceCollector = read("tooling/acceptance/windows/collect-native-feasibility.ps1");
const nativeEvidenceEvaluator = read("tooling/acceptance/evaluate-native-feasibility.mjs");
requireFact(nativeEvidenceCollector.includes("ReopenAndRebootPersistent") &&
  nativeEvidenceCollector.includes("reopenAndRebootPersistent") &&
  nativeEvidenceEvaluator.includes("reopenAndRebootPersistent") &&
  nativeEvidenceEvaluator.includes("SESSION_PERSISTENCE_NOT_OBSERVED") &&
  nativeEvidenceEvaluator.includes('operator.multiplayerJoined === "YES"'),
  "Native two-account evidence may not ignore persistence or multiplayer");
requireFact(nativeEvidenceEvaluator.includes("processFeasibility") &&
  nativeEvidenceEvaluator.includes("accountFeasibility") &&
  nativeEvidenceEvaluator.includes('productAcceptance: "NOT_EVALUATED"'),
  "Native process/account observations must stay distinct from final four-client acceptance");

const sandboxCollector = read("tooling/acceptance/windows/collect-sandboxie-feasibility.ps1");
requireFact(sandboxCollector.includes("[switch]$SummaryOnly") &&
  sandboxCollector.includes("if (-not $SummaryOnly)") &&
  sandboxCollector.includes("if ($null -ne $path) { $summary['reportPath'] = $path }") &&
  (sandboxCollector.split("$boxConfigured = if ($enabled").length - 1) === 1 &&
  sandboxCollector.includes("$membershipStatus = 'UNKNOWN'") &&
  sandboxCollector.includes("$allProcesses = @(Get-Process -ErrorAction Stop)") &&
  sandboxCollector.includes("foreach ($boxedPid in $boxedPids)") &&
  sandboxCollector.includes("if (-not $observedByPid.ContainsKey($key))") &&
  sandboxCollector.includes("if ($null -eq $rawSession -or $rawSession -isnot [int])") &&
  sandboxCollector.includes("$processSession -lt 0 -or $processSession -ne $session") &&
  sandboxCollector.includes("$boxedIdentityByPid.ContainsKey($candidateKey)") &&
  sandboxCollector.includes("$boxedIdentityByPid[$candidateKey].Name -match") &&
  sandboxCollector.includes("$allProcesses = @()") &&
  sandboxCollector.includes("$handle = $null") &&
  !sandboxCollector.includes("Get-Process -ErrorAction SilentlyContinue"),
  "One valid Sandboxie evidence owner; desktop summary refresh writes no report");
const nativeProcessPattern = "^(Minecraft\\.Windows|Minecraft.*Education.*|Education.*Minecraft.*)$";
requireFact(nativeOwner.includes(nativeProcessPattern) &&
  nativeEvidenceCollector.includes(nativeProcessPattern) &&
  sandboxCollector.includes(nativeProcessPattern),
  "Native and Sandboxie collectors must count the same Minecraft.Windows candidates");
const sandboxEvaluator = read("tooling/acceptance/evaluate-sandboxie-feasibility.mjs");
requireFact(sandboxCollector.includes("'query ' + $BoxName + ' *'") &&
  sandboxCollector.includes("'BOX_SECTION_ENUMERATED'") &&
  sandboxCollector.includes("if ($null -ne $rawNames)") &&
  sandboxCollector.includes("if ($names -contains 'AutoDelete')") &&
  sandboxCollector.includes("if ($names -contains 'AutoRemove')") &&
  !sandboxCollector.includes("Read-BoxSetting 'GlobalSettings' 'AutoDelete'") &&
  !sandboxCollector.includes("Read-BoxSetting 'GlobalSettings' 'AutoRemove'") &&
  sandboxEvaluator.includes("BOX_LOCAL_SETTINGS_NOT_VERIFIED") &&
  sandboxEvaluator.includes('boxSettingsSource === "BOX_SECTION_ENUMERATED"'),
  "Sandboxie lifecycle settings must derive only from a successful box-local enumeration");
requireFact(sandboxEvaluator.includes("processFeasibility") &&
  sandboxEvaluator.includes("accountFeasibility") &&
  sandboxEvaluator.includes('productAcceptance: "NOT_EVALUATED"') &&
  sandboxEvaluator.includes('operator.multiplayerJoined === "YES"') &&
  sandboxEvaluator.includes("BigInt(candidate.windowHandle)"),
  "Sandboxie candidate/account evidence must not masquerade as four-account acceptance");
// The selected-PID verifier may prove a signed boxed image, but the original
// outside-only counter must remain zero for that image. Rust rejects drift.
requireFact(
  sandboxCollector.includes("$signedOutside = @($gameCandidates | Where-Object {") &&
  sandboxCollector.includes("$_.imageEvidence -eq 'SIGNED_EDUCATION_IMAGE_CANDIDATE' -and") &&
  sandboxCollector.includes("$_.inSelectedBox -eq $false") &&
  read("apps/desktop/src-tauri/src/engine/native_clients.rs").includes(
    "if candidate.in_selected_box == Some(false) { official_outside += 1; }"),
  "Boxed signed Education images must never inflate outside-only counters");

// The current UI cannot be shipped as the old VMware installer while
// still bundling a Guest Agent and account-bearing installed identity.
const bundleBuilder = read("apps/desktop/scripts/prepare-package-binaries.mjs");
requireFact(bundleBuilder.includes("PACKAGE_BLOCKED_NATIVE_FEASIBILITY") &&
  bundleBuilder.includes("const LEGACY_PACKAGE_RELEASE_APPROVED = false;") &&
  bundleBuilder.includes("if (!LEGACY_PACKAGE_RELEASE_APPROVED)") &&
  bundleBuilder.indexOf("PACKAGE_BLOCKED_NATIVE_FEASIBILITY") < bundleBuilder.indexOf("const binaries ="),
  "Native-feasibility desktop must be blocked from old VMware NSIS packaging");
const nativeDesktop = read("apps/desktop/src-tauri/src/engine/native_clients.rs");
const isolatedScreen = read("apps/desktop/src/CompactManagerApp.svelte");
const isolatedBridge = read("apps/desktop/src/app/bridge/nativeFeasibilityApi.ts");
requireFact(nativeDesktop.includes("pub fn request_sandbox_client_02(") &&
  nativeDesktop.includes('const CLIENT_02_BOX_NAME: &str = "MinecraftClient02";') &&
  nativeDesktop.includes("$box = '{box_name}'") &&
  nativeDesktop.includes("('/box:' + $box)") &&
  nativeDesktop.includes("AutoDelete") &&
  nativeDesktop.includes("AutoRemove") &&
  nativeDesktop.includes("Get-AuthenticodeSignature") &&
  nativeDesktop.includes("O=Microsoft Corporation") &&
  nativeDesktop.includes("$signature.Status -ne 'Valid'") &&
  !nativeDesktop.includes("delete_sandbox") &&
  !nativeDesktop.includes("/terminate_all") &&
  read("apps/desktop/src-tauri/src/app_bootstrap.rs").includes(
    "commands::native_clients::desktop_request_sandbox_client_02,") &&
  isolatedScreen.includes("nativeFeasibility.requestSandboxClient02(") &&
  isolatedScreen.includes("nativeFeasibility.inspectIsolatedClient02()") &&
  isolatedScreen.includes("const result = await nativeFeasibility.inspectIsolatedClient02()") &&
  nativeDesktop.includes('-SummaryOnly"].concat()') &&
  nativeDesktop.includes("include_str!(concat!(") &&
  nativeDesktop.includes('observation.credential_host_access != "OFF"') &&
  nativeDesktop.includes("WaitForExit(8000)") &&
  nativeDesktop.includes("$launcher.ExitCode") &&
  nativeDesktop.includes("-WorkingDirectory $gameFolder") &&
  nativeDesktop.includes("observation.isolated_candidate_processes > 0") &&
  nativeDesktop.includes("GetEnvironmentVariable('ProgramW6432')") &&
  read("apps/desktop/src-tauri/src/app_bootstrap.rs").includes(
    "commands::native_clients::desktop_inspect_sandbox_client_02,") &&
  isolatedBridge.includes('"desktop_request_sandbox_client_02"') &&
  isolatedBridge.includes('"desktop_inspect_sandbox_client_02"'),
  "One non-destructive MinecraftClient02 launch must be registered end to end");
const sandboxBoxCollector = read("tooling/acceptance/windows/collect-sandboxie-feasibility.ps1");
const desktopPilotLease = read("apps/desktop/src-tauri/src/engine/app_instance.rs");
// The installed legacy Windows product identity is stable even while the
// active manager UI names MIVUBI's not-yet-installed isolation engine.
requireFact(tauri.app.windows[0].title === "Minecraft Instance Manager" &&
  tauri.productName === "M-Bedrock Virtual Clients" &&
  tauri.identifier === "com.mivubi.mbedrock.virtualclients",
  "Installed Windows product identity changed without a migration review");
requireFact(activeEntry.includes("Minecraft Instance Manager") &&
  isolatedScreen.includes("MinecraftClient02") &&
  isolatedScreen.includes("Runtime: existing Sandboxie Plus pilot. MIVUBI-owned engine is not active.") &&
  isolatedScreen.includes("MIVUBI has not created or verified these virtuals.") &&
  sandboxBoxCollector.includes("BoxName = 'MinecraftClient02'") &&
  sandboxBoxCollector.includes("MinecraftInstanceManager\\SandboxieFeasibility") &&
  read("tooling/acceptance/windows/collect-native-feasibility.ps1").includes(
    "MinecraftInstanceManager\\NativeFeasibility") &&
  desktopPilotLease.includes('.join("MinecraftInstanceManager")') &&
  desktopPilotLease.includes('.join("NativePilot")'),
  "Active pilot naming, data paths or unavailable owned-engine labels drifted");
const desktopFacade = read("apps/desktop/src/app/bridge/virtualClientsApi.ts");
const desktopBootstrap = read("apps/desktop/src-tauri/src/app_bootstrap.rs");
requireFact(read("apps/desktop/src-tauri/src/commands/native_clients.rs").includes("fn desktop_open_native_minecraft(") &&
  desktopFacade.includes('"desktop_open_native_minecraft"') &&
  desktopBootstrap.includes("commands::native_clients::desktop_open_native_minecraft,"),
  "Active desktop must retain only the official native game launcher");
requireFact(!nativeDesktop.includes("BlueStacks") && !nativeDesktop.includes("open_android") &&
  !nativeDesktop.includes("Get-Credential") && !nativeDesktop.includes("vmrun.exe"),
  "Desktop native launcher must not own another emulator, credential or VM lifecycle");
for (const file of ["apps/desktop/src/App.svelte", "apps/desktop/src/app/surfaces/SetupSurface.svelte",
  "apps/desktop/src/app/surfaces/ClientsSurface.svelte", "apps/desktop/src/app/bridge/virtualClientsApi.ts"]) {
  requireFact(!/BlueStacks|onOpenAndroid|openAndroidInstances|desktop_open_android_instances/.test(read(file)),
    "Retired alternative client backend remains in desktop source: " + file);
}
requireFact(!existsSync(resolve(root,"tooling/acceptance/windows/probe-android-instances.ps1")) &&
  !existsSync(resolve(root,"tooling/acceptance/windows/probe-native-multi-instance.ps1")),
  "Competing multi-instance acceptance probes must be retired");

// Catch stale relative source references that survive a monorepo extraction.
// GitHub search indexing alone cannot certify that current tests have no dead imports.
for (const name of readdirSync(resolve(root, "apps/desktop/test"))) {
  if (!name.endsWith(".test.ts")) continue;
  const source = read("apps/desktop/test/" + name);
  requireFact(!source.includes("../../../virtual-clients/runtime-core/") &&
    !source.includes("apps/virtual-clients/"),
    "Desktop test still references the retired monorepo source: " + name);
}
requireFact(read("tooling/acceptance/windows/new-stage-result.ps1").includes("'CLIENT_FEASIBILITY'"),
  "Client feasibility evidence stage not recognized by acceptance recorder");
const hostAcceptance = read("tooling/acceptance/windows/collect-acceptance.ps1");
requireFact(hostAcceptance.includes("if ($provisionedCount -eq 3)") &&
  hostAcceptance.includes("$runningCount -ne 3") &&
  hostAcceptance.includes("invalid CLI success envelope") &&
  hostAcceptance.includes("$resources = $null") &&
  hostAcceptance.includes("resourcesCollected = ($null -ne $resources)") &&
  !hostAcceptance.includes("readyForProvisioning = [bool]$doctor.readyForProvisioning") &&
  !hostAcceptance.includes("multiplayerVerified = $_.multiplayerVerified"),
  "Host-stage acceptance must collect before VM provisioning without inventing readiness");
const guestProofSource = read("tooling/acceptance/windows/collect-client-feasibility.ps1");
const evaluatorSource = read("tooling/acceptance/evaluate-client-feasibility.mjs");
for (const fact of ["vmDesktopOpened", "educationInstalled", "educationSignedIn",
  "crashOrFreezeObserved", "observedMinutes"]) {
  requireFact(guestProofSource.includes(fact) && evaluatorSource.includes(fact),
    "Single Windows guest boot, install/login and gameplay proof owner drift: " + fact);
}

requireFact(read("apps/desktop/src-tauri/Cargo.toml").includes('path = "../../../engine/runtime-core"'),
  "Tauri backend dependency points outside canonical engine");
requireFact(release.repository === "MIVUBI-STD/M-Minecraft-Emulator", "Update repository identity drift");
requireFact(release.selfUpdateRuntimeEnabled === false, "Runtime updater must not self-apply");
requireFact(read("engine/runtime-core/src/update.rs").includes(
  'policy.repository != "MIVUBI-STD/M-Minecraft-Emulator"'), "Backend update policy pin drift");

const resourceRoots = [
  "../../../engine/guest/windows/",
  "../../../tooling/acceptance/windows/",
  "../../../engine/distribution/release-channel.json",
  "../../../engine/distribution/package-contract.json"
];
const baseResources = tauri.bundle?.resources ?? {};
const packageResources = tauriPackage.bundle?.resources ?? {};
const packageContract = json("engine/distribution/package-contract.json");
requireFact(packageContract.schema === 1 && Array.isArray(packageContract.requiredFiles),
  "Unsupported or invalid installed package contract");
const resourceDestinations = new Set([...Object.values(baseResources), ...Object.values(packageResources)]);
const generatedExecutables = new Set(["m-bedrock-virtual-clients.exe", "virtual-clients.exe"]);
for (const requiredFile of packageContract.requiredFiles) {
  if (generatedExecutables.has(requiredFile)) continue;
  requireFact([...resourceDestinations].some((destination) =>
    destination === requiredFile || (destination.endsWith("/") && requiredFile.startsWith(destination))),
    "Tauri resources do not cover required installed file: " + requiredFile);
}
for (const origin of resourceRoots) {
  requireFact(origin in baseResources && origin in packageResources,
    "Installer resource route is missing: " + origin);
  requireFact(existsSync(resolve(root, "apps/desktop/src-tauri", origin)),
    "Installer source resource does not exist: " + origin);
}
requireFact(tauri.bundle?.windows?.nsis?.installMode === "currentUser", "Windows install scope changed");
requireFact(tauri.bundle?.windows?.allowDowngrades === false, "Downgrade restriction changed");
const debugCandidateGuide = read("docs/validation/ci-debug-candidate.md");
requireFact(debugCandidateGuide.includes("raw, unsigned development executable") &&
  debugCandidateGuide.includes("sha256=*") &&
  debugCandidateGuide.includes("DEV.cmd preflight") &&
  debugCandidateGuide.includes("does not install a driver"),
  "CI debug candidate must retain explicit non-installer and checksum instructions");
// CI is deliberately disabled. Retain the existing `npm run verify:source`
// and its twelve constituent scripts for opt-in, non-Actions testing only.
// Do not add GitHub workflows, silent runners, deployment, or scheduled jobs.
const canonicalStages = rootPackage.scripts["verify:source"].split(" && ");
requireFact(canonicalStages.length === 12,
  "Keep existing source-verification scripts independently available");
requireFact(!existsSync(resolve(root, ".github/workflows/verify.yml")) &&
  !existsSync(resolve(root, ".github/workflows/package-smoke.yml")) &&
  (!existsSync(resolve(root, ".github/workflows")) ||
    readdirSync(resolve(root, ".github/workflows")).every(name =>
      name.startsWith(".") && name !== "." && name !== "..")),
  "GitHub Actions workflows must stay removed from Local");

function markdownFiles(dir) {
  return readdirSync(resolve(root, dir), { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    return entry.isDirectory() ? markdownFiles(path) : entry.name.endsWith(".md") ? [path] : [];
  });
}
for (const path of markdownFiles("docs")) {
  const content = read(path);
  requireFact(/^---\s*\nid: document\./.test(content), "Documentation identity missing: " + path);
  for (const link of content.matchAll(/\]\(([^)]+)\)/g)) {
    const target = link[1].split("#", 1)[0];
    if (!target || target.startsWith("#") || /^[a-z]+:/i.test(target)) continue;
    requireFact(existsSync(resolve(root, dirname(path), decodeURIComponent(target))),
      "Broken documentation link in " + path + " -> " + target);
  }
}

// One active engine/one current handoff: historical recipes must not be
// mistaken for the current v1.18.6 source/build flow.
const currentHandoff = read("planning/development.md");
const currentIsolationGuide = read("tooling/isolation-source/README.md");
const engineRouter = read("engine/README.md");
const toolRouter = read("tooling/README.md");
const validationRouter = read("docs/validation/README.md");
requireFact(currentHandoff.startsWith("# Current Development Handoff") &&
  currentHandoff.length < 8000 &&
  currentHandoff.includes("planning/history") &&
  read("planning/history/development-through-2026-10-19.md").includes("ARCHIVE ONLY"),
  "Current handoff must be concise and separate from historical progress");
requireFact(currentIsolationGuide.includes("build-owned-v1186.ps1") &&
  currentIsolationGuide.includes("assemble-owned-v1186.mjs") &&
  currentIsolationGuide.includes("workspace/builds/") &&
  currentIsolationGuide.includes("workspace/packages/") &&
  !currentIsolationGuide.includes("D:\\MIVUBIReports") &&
  !currentIsolationGuide.includes("D:\\Source\\M-Isolation-Engine") &&
  currentIsolationGuide.includes("historical"),
  "Active native README must select one v1.18.6 owner and workspace outputs");
requireFact(
  engineRouter.includes("isolation-owned/") &&
  engineRouter.includes("isolation-candidates/sandboxie-v1.18.6/") &&
  engineRouter.includes("Isolation Source Tooling README") &&
  engineRouter.includes("official Sandboxie Plus"),
  "Engine router must distinguish editable owned v1.18.6, immutable reference and official runtime");
requireFact(
  toolRouter.includes("Sole current v1.18.6") &&
  toolRouter.includes("v1.18.5 scripts are historical"),
  "Tooling router must reference only the current v1.18.6 native build owner");
requireFact(
  validationRouter.includes("GitHub Actions CI disabled") &&
  !validationRouter.includes("Current `Local` pushes run automatic"),
  "Validation router must reflect CI disabled policy");
for (const historicalPath of [
  "planning/history/isolation-v1185-source-notes.md",
  "planning/history/isolation-native-development-notes.md",
  "planning/history/owned-source-import-notes.md"
]) {
  const note = read(historicalPath);
  requireFact(note.includes("HISTORICAL") || note.includes("ARCHIVED"),
    "Missing explicitly historical source archive: " + historicalPath);
}
requireFact(read("docs/system/owned-source-import.md").length < 5500 &&
  read("docs/system/owned-isolation-design.md").includes("lifecycle: HISTORICAL") &&
  read("docs/validation/owned-isolation-upstream-inventory.md").includes("lifecycle: HISTORICAL"),
  "Active source-import procedure still mixes historical v1.18.5 authority");

const scripts = json("package.json").scripts;
requireFact(scripts["test:localtest"] === "node --test tooling/local-test/session.test.mjs tooling/windows-toolchain/portable.test.mjs" &&
  scripts["verify:source"].includes("npm run test:localtest"),
  "Source verification must execute local-test storage regressions");
const nativePreflight = read("tooling/windows-toolchain/dev.ps1");
for (const marker of [
  'session.mjs" start preflight $sha',
  'session.mjs" finish $recordDir PASS complete',
  'session.mjs" finish $recordDir FAIL $stage',
  'Assert-CleanSourceAtHead -ExpectedSha $sha'
]) {
  requireFact(nativePreflight.includes(marker),
    "Native preflight lost run recording or exact-source binding: " + marker);
}
const storagePolicy = read("docs/system/artifact-storage.md");
requireFact(storagePolicy.startsWith("---\nid: document.system.artifact-storage\n"),
  "Canonical artifact storage policy is missing");
for (const localPath of [
  "workspace/downloads/vendor-setup.exe",
  "workspace/packages/installer-copy.exe",
  "workspace/runs/test-session/logs/output.log",
  "workspace/private/profile.json",
  "artifacts/misplaced.zip",
  "downloads/misplaced.exe",
  "installers/misplaced.msi",
  "test-results/misplaced.json"
]) {
  const result = spawnSync("git", ["check-ignore", "--no-index", "-q", "--", localPath], {
    cwd: root, encoding: "utf8"
  });
  requireFact(!result.error && result.status === 0,
    "Machine-local output is not Git-ignored: " + localPath);
}
const readmeIgnoreCheck = spawnSync("git",
  ["check-ignore", "--no-index", "-q", "--", "workspace/README.md"], {
    cwd: root, encoding: "utf8"
  });
requireFact(!readmeIgnoreCheck.error && readmeIgnoreCheck.status === 1,
  "Canonical workspace README must remain trackable");
const trackedLocal = spawnSync("git", ["ls-files", "-z", "--",
  "workspace", "artifacts", "downloads", "installers", "test-results"], {
  cwd: root, encoding: "utf8"
});
requireFact(!trackedLocal.error && trackedLocal.status === 0,
  "Cannot verify tracked machine-output ownership");
const trackedStrays = trackedLocal.stdout.split("\0").filter(Boolean)
  .filter((path) => path !== "workspace/README.md");
requireFact(trackedStrays.length === 0,
  "Machine-local output is already tracked: " + trackedStrays.join(", "));
console.log("Emulator repository contract: PASS (source/layout inspection only).");
