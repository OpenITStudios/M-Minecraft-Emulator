import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import {
  parseNativeObservation, parseIsolatedClientObservation,
  isSandboxieBoxPrepared, canRequestClient02,
} from "../src/app/bridge/nativeFeasibilityApi.js";

const fixture = () => ({
  schema: 1,
  evidenceClass: "NAME_MATCH_CANDIDATE_ONLY",
  interactiveSession: true,
  candidates: [
    { pid: 123, windowHandle: "0xAA" },
    { pid: 456, windowHandle: null },
  ],
});

describe("Native candidate IPC evidence", () => {
  it("accepts bounded candidate data without promoting it to Minecraft gameplay", () => {
    expect(parseNativeObservation(fixture())).toEqual(fixture());
    const empty = fixture();
    empty.candidates = [];
    expect(parseNativeObservation(empty).candidates).toHaveLength(0);
    expect(parseNativeObservation({ ...empty, interactiveSession: false }).interactiveSession).toBe(false);
  });

  it("fails closed for malformed schemas, PIDs, handles and excess processes", () => {
    const bad = [
      { ...fixture(), schema: 2 },
      { ...fixture(), evidenceClass: "GAMEPLAY_VERIFIED" },
      { ...fixture(), interactiveSession: "yes" },
      { ...fixture(), candidates: Array.from({ length: 17 }, (_, i) => ({pid: i + 1, windowHandle: null})) },
      { ...fixture(), candidates: [{ pid: 0, windowHandle: null }] },
      { ...fixture(), candidates: [{ pid: 0x100000000, windowHandle: null }] },
      { ...fixture(), candidates: [{ pid: NaN, windowHandle: null }] },
      { ...fixture(), candidates: [{ pid: 123, windowHandle: "0x0" }] },
      { ...fixture(), candidates: [{ pid: 123, windowHandle: "0xXYZ" }] },
      { ...fixture(), candidates: [{ pid: 123, windowHandle: "0x" }] },
      { ...fixture(), candidates: [
        { pid: 123, windowHandle: "0xAA" },
        { pid: 123, windowHandle: null },
      ] },
      { ...fixture(), candidates: [
        { pid: 123, windowHandle: "0xAA" },
        { pid: 456, windowHandle: "0xaa" },
      ] },
      { ...fixture(), candidates: [
        { pid: 123, windowHandle: "0xAA" },
        { pid: 456, windowHandle: "0x000aa" },
      ] },
    ];
    for (const report of bad) {
      expect(() => parseNativeObservation(report)).toThrow(TypeError);
    }
  });

  it("rejects injected account identifiers, window titles and process command lines", () => {
    expect(() => parseNativeObservation({
      ...fixture(),
      accountEmail: "hidden@example.com",
    })).toThrow(TypeError);
    expect(() => parseNativeObservation({
      ...fixture(),
      candidates: [{ pid: 123, windowHandle: null, commandLine: "secret" }],
    })).toThrow(TypeError);
    expect(() => parseNativeObservation({
      ...fixture(),
      candidates: [{ pid: 123, windowHandle: null, windowTitle: "private world" }],
    })).toThrow(TypeError);
  });

  it("registers native observation and exactly one isolated Client 02 launch experiment", () => {
    const bootstrap = readFileSync(new URL("../src-tauri/src/app_bootstrap.rs", import.meta.url), "utf8");
    const commands = readFileSync(new URL("../src-tauri/src/commands/native_clients.rs", import.meta.url), "utf8");
    const engine = readFileSync(new URL("../src-tauri/src/engine/native_clients.rs", import.meta.url), "utf8");
    const active = readFileSync(new URL("../src/NativeFeasibilityApp.svelte", import.meta.url), "utf8");
    const bridge = readFileSync(new URL("../src/app/bridge/nativeFeasibilityApi.ts", import.meta.url), "utf8");

    expect(bootstrap).toContain("commands::native_clients::desktop_open_native_minecraft,");
    expect(bootstrap).toContain("commands::native_clients::desktop_inspect_native_candidates,");
    expect(bootstrap).toContain("commands::native_clients::desktop_select_education_exe,");
    expect(bootstrap).toContain("commands::native_clients::desktop_request_sandbox_client_02,");
    expect(bootstrap).toContain("commands::native_clients::desktop_inspect_sandbox_client_02,");
    expect(bootstrap).toContain("commands::native_clients::desktop_inspect_owned_engine_candidate,");
    expect(commands).toContain("fn desktop_inspect_owned_engine_candidate(");
    expect(bridge).toContain('"desktop_inspect_owned_engine_candidate"');
    expect(bootstrap).toContain("commands::native_clients::desktop_focus_observed_client_02,");
    expect(bootstrap).toContain("commands::native_clients::desktop_place_observed_client_02,");
    expect(commands).toContain("fn desktop_inspect_sandbox_client_02(");
    expect(commands).toContain("fn desktop_focus_observed_client_02(");
    expect(commands).toContain("fn desktop_place_observed_client_02(");
    expect(engine).toContain("pub fn inspect_sandbox_client_02()");
    expect(engine).toContain("pub fn focus_observed_client_02(");
    expect(engine).toContain("pub fn place_observed_client_02(");
    expect(engine).toContain("fn with_verified_client_02_window(");
    expect(engine).toContain("GetMonitorInfoW(monitor, &mut info)");
    expect(engine).toContain("MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST)");
    expect(engine).toContain("SetWindowPos(");
    expect(engine).toContain("SWP_NOACTIVATE");
    expect(engine).toContain("SWP_NOZORDER");
    expect(engine).toContain("SWP_ASYNCWINDOWPOS");
    expect(engine).toContain("fn placement_slot_keeps_all_pieces_inside_monitor_work_area()");
    expect(engine).toContain("verify_fresh_focus_target(&target, &fresh)");
    expect(engine).toContain("fresh.isolated_candidate_processes != 1");
    expect(engine).toContain("fresh.isolated_visible_candidates != 1");
    expect(engine).toContain("extra_boxed.process_candidates.push(newcomer.clone());");
    expect(engine).toContain("native_outside.process_candidates.push(newcomer);");
    expect(engine).toContain("GetProcessTimes(process, &mut created");
    expect(engine).toContain("GetWindowThreadProcessId(hwnd, &mut actual_pid)");
    expect(engine).toContain("fn verify_live_client_02_window(");
    expect(engine).toContain("GetWindow(hwnd, GW_OWNER)");
    expect(engine).toContain("GetAncestor(hwnd, GA_ROOT)");
    expect(engine).toContain("usize::try_from(hwnd_number)");
    expect(engine).toContain("verify_live_client_02_window(hwnd, target.pid)?;");
    expect(engine).toContain("verify_live_client_02_window(hwnd, process_id)?;");
    expect(engine).toContain("MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) } != monitor");
    expect(engine).toContain("Client 02 window state changed before placement");
    expect(engine).not.toContain("hwnd_number as usize as *mut");
    expect(active).toContain("not an owned popup");
    expect(engine).toContain("SetForegroundWindow(hwnd)");
    expect(engine).not.toContain("AllowSetForegroundWindow");
    expect(engine).not.toContain("SendInput(");
    expect(engine).toContain("include_str!(concat!(");
    // Structural guard only; the Windows Rust test executes the resolved path.
    expect(engine).toContain("GetSystemDirectoryW");
    expect(engine).not.toContain('Path::new("powershell.exe")');
    expect(engine).toContain("$allProcesses = @(Get-Process -ErrorAction Stop)");
    expect(engine).not.toContain("Get-Process -ErrorAction SilentlyContinue");
    expect(engine).toContain("Start-Process -FilePath $explorer");
    expect(engine).toContain("WaitForExit(8000)");
    expect(engine).toContain("$launcher.ExitCode");
    expect(engine).toContain("-WorkingDirectory $gameFolder");
    expect(engine).toContain("observation.isolated_candidate_processes > 0");
    expect(engine).toContain("let mut launch_guard = acquire_client_02_launch()?;");
    expect(engine).toContain("*launch_guard = Some(Instant::now());");
    expect(engine).toContain("GetEnvironmentVariable('ProgramW6432')");
    // Both launch routes refuse unverified package inventory and require
    // a uniquely registered current-user Minecraft Education Start app.
    expect(engine.match(/Get-AppxPackage -Name 'Microsoft.MinecraftEducationEdition' -ErrorAction Stop/g)).toHaveLength(1);
    expect(engine.match(/Get-StartApps -ErrorAction Stop/g)).toHaveLength(1);
    expect(engine).toContain("Assert-ManagerRegisteredAppId $appId $selectedTarget");
    expect(engine).toContain("let desktop_preflight = EDUCATION_DESKTOP_PREFLIGHT;");
    expect(engine).toContain("{desktop_preflight}");
    expect(engine).toContain("Minecraft Education must have exactly one registered Start menu app.");
    expect(engine).toContain("A registered AppID is NOT an authoritative on-disk EXE path");
    expect(engine).not.toContain("Get-AppxPackage -Name 'Microsoft.MinecraftEducationEdition' -ErrorAction SilentlyContinue");
    expect(engine).toContain('observation.credential_host_access != "OFF"');
    expect(bridge).toContain('"desktop_inspect_sandbox_client_02"');
    expect(bridge).toContain('"desktop_focus_observed_client_02"');
    expect(bridge).toContain('"desktop_place_observed_client_02"');
    expect(active).toContain('nativeFeasibility.placeObservedClient02(target, side)');
    expect(active).toContain('placeClient02("LEFT")');
    expect(active).toContain('placeClient02("RIGHT")');
    expect(active).toContain("Client 01 was not moved.");
    expect(active).toContain("officialImageOutsideSelectedBoxCount");
    expect(active).toContain("signedImageOtherBoxCount");
    expect(active).toContain("signedImageUnknownBoxCount");
    expect(active).toContain("Neither result proves an unsandboxed native process.");
    expect(bridge).toContain("officialImageOutsideSelectedBoxCount");
    expect(engine).toContain("official_image_outside_selected_box_count");
    expect(active).toContain("Direct Sandboxie PID lookup unavailable");
    expect(active).toContain("nativeFeasibility.focusObservedClient02(target)");
    expect(active).toContain("disabled={busy || !focusTarget}");
    expect(active).not.toContain("SetForegroundWindow");
    expect(active).toContain("nativeFeasibility.inspectIsolatedClient02()");
    expect(active).not.toContain("Promise.allSettled");
    expect(active).toContain("observation = await nativeFeasibility.inspectIsolatedClient02()");
    expect(active).not.toContain("nativeFeasibility.inspectCandidates()");
    expect(engine).toContain('["& {\\n", INSPECT, "\\n} -SummaryOnly"].concat()');
    // Only the oversized embedded collector must use stdin; launch is separate.
    const inspection = engine.match(/fn inspect_sandbox_client_02_mode\([\s\S]*?\n\}/)?.[0];
    expect(inspection).toBeDefined();
    expect(inspection).toContain("bounded_command_output_with_stdin(");
    expect(inspection).toContain("script.as_bytes()");
    expect(inspection).toContain("OpenStandardInput()");
    expect(inspection).toContain("System.IO.StreamReader]::new");
    expect(inspection).toContain("ScriptBlock]::Create");
    expect(inspection).not.toContain('"-Command", script.as_str()');
    expect(engine).toContain('["& {\\n", INSPECT, "\\n} -SummaryOnly -ImageProof"].concat()');
    expect(engine).toContain("inspect_sandbox_client_02_mode(true, None)");
    expect(engine).toContain("inspect_sandbox_client_02_mode(false, None)");
    expect(engine).toContain("IsolationInspection::TargetedImageProof(target.pid)");
    expect(engine).toContain("trait IsolationBackend {");
    expect(engine).toContain("struct OfficialSandboxieBackend;");
    expect(engine).toContain("fn request_launch(&self, executable: String, licensed: bool) -> io::Result<()>;");
    expect(engine).toContain("request_with_backend(&ACTIVE_ISOLATION_BACKEND, executable, licensed)");
    expect(engine).toContain("launch_with_official_sandboxie(executable, licensed)");
    expect(engine).toContain("fn adapter_launch_dispatches_once_without_retry_on_timeout()");
    expect(engine).toMatch(/inspect_with_backend\(\s*&ACTIVE_ISOLATION_BACKEND,\s*IsolationInspection::FastLaunchPreflight,?\s*\)/);
    expect(engine).toContain('candidate.image_evidence == "SIGNED_EDUCATION_IMAGE_CANDIDATE"');
    expect(engine).toContain('candidate.created_file_time == target.created_file_time');
    expect(active).toContain("UNKNOWN or unsigned files are refused");
    expect(engine).toContain("let observation = inspect_sandbox_client_02_fast()?;");
    expect(engine).toContain("admit_client_02_launch_snapshot(&observation)?;");
    expect(engine).toContain('if observation.membership_status != "OBSERVED"');
    expect(engine).toContain("fn client_02_launch_requires_observed_empty_box()");
    expect(active).toContain('observation.membershipStatus !== "OBSERVED"');
    const sandboxList = readFileSync(new URL("../../../tooling/acceptance/windows/collect-sandboxie-feasibility.ps1", import.meta.url), "utf8");
    expect(sandboxList).toContain("$BoxedImageProofPid = 0");
    expect(sandboxList).toContain("MivubiReadOnlyImageLease");
    expect(sandboxList).toContain("QueryFullProcessImageNameW");
    expect(sandboxList).toContain("GetFileInformationByHandleEx");
    expect(sandboxList).toContain("FileShare.Read");
    expect(sandboxList).toContain("FileAttributes.ReparsePoint");
    expect(sandboxList).toContain("MivubiReadOnlyImageLease]::Acquire(");
    expect(sandboxList).toContain("Get-EducationImageCandidate -ImagePath $lease.ImagePath");
    expect(sandboxList).toContain("if ($null -ne $lease) { $lease.Dispose() }");
    expect(sandboxList).not.toContain("Get-Process -Id $_.Id -ErrorAction Stop");
    expect(sandboxList).toContain("Get-AuthenticodeSignature -LiteralPath $image");
    expect(sandboxList).toContain("$count -lt 0 -or $count -gt 256");
    expect(sandboxList).toContain("function Read-BoxPidList");
    expect(sandboxList).toContain("$first = Read-BoxPidList");
    expect(sandboxList).toContain("$second = Read-BoxPidList");
    expect(sandboxList).toContain("(($first.Pids -join ',') -ceq ($second.Pids -join ','))");
    expect(sandboxList).toContain("if ($gameCandidates.Count -gt 16)");
    expect(sandboxList).toContain("$rawSession = $process.SessionId");
    expect(sandboxList).toContain("$processSession -lt 0 -or $processSession -ne $session");
    expect(sandboxList).toContain("$boxedIdentityByPid[$key] = [pscustomobject]@{");
    expect(sandboxList).toContain("$boxedIdentityByPid.ContainsKey($candidateKey)");
    expect(sandboxList).toContain("$boxedIdentityByPid[$candidateKey].Name -match");
    expect(sandboxList).toContain("if ($null -eq $rawSession -or $rawSession -isnot [int])");
    expect(sandboxList).toContain("if (-not $observedByPid.ContainsKey($key))");
    expect(sandboxList).toContain("$handle = $null");
    expect(engine).toContain("Sandboxie PID membership or Windows process identity is UNKNOWN");
    expect(active).toContain("Sandboxie membership or Windows process identity is unknown");
    expect(active).toContain('typeof reason === "string"');
    expect(commands).toContain("fn desktop_request_sandbox_client_02(");
    expect(engine).toContain("pub fn request_sandbox_client_02(");
    expect(engine).toContain('const CLIENT_02_BOX_NAME: &str = "MinecraftClient02";');
    expect(engine).toContain("$box = '{box_name}'");
    expect(engine).toContain("('/box:' + $box)");
    expect(engine).toContain("AutoDelete");
    expect(engine).toContain("AutoRemove");
    expect(engine).toContain("Get-AuthenticodeSignature");
    expect(engine).toContain("O=Microsoft Corporation");
    expect(engine).toContain("$signature.Status -ne 'Valid'");
    expect(engine).toContain("Start-Process -FilePath $start");
    expect(engine).not.toContain("delete_sandbox");
    expect(engine).not.toContain("/terminate_all");
    expect(bridge).toContain('"desktop_request_sandbox_client_02"');
    expect(active).toContain("nativeFeasibility.requestSandboxClient02(");
    expect(active).toContain("canRequestClient02(observation)");
    expect(active).toContain("if (busy || !licenseConfirmed || !selectedExecutable || !canLaunchIsolated) return;");
    expect(active).toContain("disabled={busy || !licenseConfirmed || !selectedExecutable || !canLaunchIsolated}");
    expect(active).toContain('activeOperation = "Rechecking Sandboxie and requesting Client 02 launch…";');
    expect(bootstrap).not.toContain("commands::virtual_clients::virtual_clients_");
    expect(bootstrap).not.toContain("commands::setup_tools::setup_");
    expect(bootstrap).not.toContain("commands::window_arrangement::window_");

    expect(commands).toContain("fn desktop_inspect_native_candidates(");
    expect(engine).toContain("pub fn inspect_native_candidates()");
    expect(bridge).toContain('"desktop_inspect_native_candidates"');
    expect(active).toContain("nativeFeasibility.inspectIsolatedClient02()");
    expect(active).toContain("nativeFeasibility.openOfficialMinecraft()");
    expect(active).not.toContain("backend.");
  });

  it("accepts only non-authoritative isolated process and box observations", () => {
    const valid = {
      schema: 6, boxName: "MinecraftClient02", stage: "INSPECT",
      boxConfigured: "YES", boxSettingsSource: "BOX_SECTION_ENUMERATED",
      autoDelete: "OFF", autoRemove: "OFF", credentialHostAccess: "OFF",
      membershipStatus: "OBSERVED", interactiveSession: true,
      processEvidenceClass: "NAME_MATCH_CANDIDATE_ONLY",
      candidateProcessCount: 2,
      processCandidates: [
        { pid: 101, createdFileTime: "133400000000000000", windowHandle: "0x100", inSelectedBox: false, imageEvidence: "SIGNED_EDUCATION_IMAGE_CANDIDATE" },
        { pid: 202, createdFileTime: "133400000000000001", windowHandle: "0x200", inSelectedBox: true, imageEvidence: "UNKNOWN" },
      ],
      officialImageOutsideSelectedBoxCount: 1,
      directPidQuery: "QUERIED",
      signedImageOtherBoxCount: 0,
      signedImageUnknownBoxCount: 1,
      isolatedCandidateProcesses: 1,
      nativeVisibleCandidates: 1, isolatedVisibleCandidates: 1,
      interpretation: "CONFIG_AND_PROCESS_CANDIDATES_ONLY_NO_AUTHENTICATION_PROOF",
    };
    expect(parseIsolatedClientObservation(valid)).toEqual(valid);
    // Behavior-level regression: launch is only suggested after an observed,
    // complete, empty persistent box; the Rust backend still re-inspects.
    expect(isSandboxieBoxPrepared(parseIsolatedClientObservation(valid))).toBe(true);
    expect(canRequestClient02(parseIsolatedClientObservation(valid))).toBe(false);
    const emptyBox = {
      ...valid, candidateProcessCount: 0, processCandidates: [],
      isolatedCandidateProcesses: 0, nativeVisibleCandidates: 0,
      isolatedVisibleCandidates: 0, officialImageOutsideSelectedBoxCount: 0,
      directPidQuery: "NOT_REQUESTED", signedImageOtherBoxCount: 0,
      signedImageUnknownBoxCount: 0,
    };
    expect(canRequestClient02(parseIsolatedClientObservation(emptyBox))).toBe(true);
    expect(canRequestClient02(undefined)).toBe(false);
    for (const ineligible of [
      { ...emptyBox, membershipStatus: "UNKNOWN" },
      { ...emptyBox, autoDelete: "ON" },
      { ...emptyBox, autoRemove: "ON" },
      { ...emptyBox, credentialHostAccess: "ON" },
      { ...emptyBox, interactiveSession: false },
      { ...emptyBox, boxSettingsSource: "UNKNOWN", credentialHostAccess: "UNKNOWN" },
    ]) {
      expect(canRequestClient02(parseIsolatedClientObservation(ineligible))).toBe(false);
    }
    const signedBoxed = {
      ...valid, processCandidates: [
        valid.processCandidates[0],
        { ...valid.processCandidates[1], imageEvidence: "SIGNED_EDUCATION_IMAGE_CANDIDATE" },
      ],
    };
    expect(parseIsolatedClientObservation(signedBoxed)).toEqual(signedBoxed);
    // The outside-only counter must not change when a boxed PID is signed.
    expect(signedBoxed.officialImageOutsideSelectedBoxCount).toBe(1);
    // Targeted boxed executable proof must not be mistaken for an image
    // observed outside the box. Its existing schema reports zero outside.
    const onlyBoxedSigned = {
      ...signedBoxed,
      candidateProcessCount: 1,
      processCandidates: [signedBoxed.processCandidates[1]],
      officialImageOutsideSelectedBoxCount: 0,
      directPidQuery: "NOT_REQUESTED",
      signedImageOtherBoxCount: 0, signedImageUnknownBoxCount: 0,
      isolatedCandidateProcesses: 1,
      isolatedVisibleCandidates: 1, nativeVisibleCandidates: 0,
    };
    expect(parseIsolatedClientObservation(onlyBoxedSigned)).toEqual(onlyBoxedSigned);
    // Make counter metadata internally plausible, then verify that the
    // per-PID reconciliation (not just metadata parsing) rejects the lie.
    const incorrectlyCounted = {
      ...onlyBoxedSigned,
      officialImageOutsideSelectedBoxCount: 1,
      directPidQuery: "UNAVAILABLE",
      signedImageUnknownBoxCount: 1,
    };
    expect(() => parseIsolatedClientObservation(incorrectlyCounted)).toThrow(TypeError);
    expect(parseIsolatedClientObservation({
      ...valid, signedImageOtherBoxCount: 1, signedImageUnknownBoxCount: 0,
    }).directPidQuery).toBe("QUERIED");
    expect(parseIsolatedClientObservation({
      ...valid, directPidQuery: "UNAVAILABLE",
    }).directPidQuery).toBe("UNAVAILABLE");
    const invalid = [
      { ...valid, boxName: "AnotherBox" },
      { ...valid, boxConfigured: "UNKNOWN" },
      { ...valid, boxSettingsSource: "UNKNOWN" },
      { ...valid, credentialHostAccess: "GAMEPLAY_VERIFIED" },
      { ...valid, candidateProcessCount: 1 },
      { ...valid, isolatedCandidateProcesses: 0 },
      { ...valid, isolatedCandidateProcesses: 3 },
      { ...valid, schema: 5 },
      { ...valid, directPidQuery: "NATIVE_VERIFIED" },
      { ...valid, directPidQuery: "NOT_REQUESTED" },
      { ...valid, signedImageOtherBoxCount: 2 },
      { ...valid, signedImageUnknownBoxCount: 0 },
      { ...valid, directPidQuery: "UNAVAILABLE", signedImageOtherBoxCount: 1, signedImageUnknownBoxCount: 0 },
      { ...valid, officialImageOutsideSelectedBoxCount: 0 },
      { ...valid, processCandidates: [
        valid.processCandidates[0],
        { ...valid.processCandidates[1], createdFileTime: null,
          imageEvidence: "SIGNED_EDUCATION_IMAGE_CANDIDATE" },
      ] },
      { ...valid, processCandidates: [
        { ...valid.processCandidates[0], imageEvidence: "NATIVE_AUTHENTICATED" },
        valid.processCandidates[1],
      ] },
      { ...valid, processCandidates: [] },
      { ...valid, processCandidates: [
        { ...valid.processCandidates[0], pid: 202 }, valid.processCandidates[1],
      ] },
      { ...valid, processCandidates: [
        valid.processCandidates[0], { ...valid.processCandidates[1], createdFileTime: "0" },
      ] },
      { ...valid, processCandidates: [
        valid.processCandidates[0], { ...valid.processCandidates[1], createdFileTime: 133400000000000000 },
      ] },
      { ...valid, processCandidates: [
        valid.processCandidates[0], { ...valid.processCandidates[1], inSelectedBox: false },
      ] },
      { ...valid, processCandidates: [
        valid.processCandidates[0], { ...valid.processCandidates[1], windowHandle: "0x100" },
      ] },
      { ...valid, nativeVisibleCandidates: 17 },
      { ...valid, membershipStatus: "UNKNOWN" },
      { ...valid, accountEmail: "forbidden@example.com" },
    ];
    for (const item of invalid) expect(() => parseIsolatedClientObservation(item)).toThrow(TypeError);
    expect(parseIsolatedClientObservation({
      ...valid, membershipStatus: "UNKNOWN",
      nativeVisibleCandidates: 0, isolatedVisibleCandidates: 0, isolatedCandidateProcesses: 0,
      officialImageOutsideSelectedBoxCount: 0,
      directPidQuery: "NOT_REQUESTED",
      signedImageOtherBoxCount: 0,
      signedImageUnknownBoxCount: 0,
      processCandidates: valid.processCandidates.map(p => ({ ...p, inSelectedBox: null, imageEvidence: "UNKNOWN" })),
    }).membershipStatus).toBe("UNKNOWN");
    expect(parseIsolatedClientObservation({
      ...valid, processCandidates: [
        valid.processCandidates[0],
        { ...valid.processCandidates[1], createdFileTime: null },
      ],
    }).processCandidates[1].createdFileTime).toBeNull();
  });

  it("uses only generic live names without changing legacy installed identity", () => {
    const screen = readFileSync(new URL("../src/NativeFeasibilityApp.svelte", import.meta.url), "utf8");
    const backend = readFileSync(new URL("../src-tauri/src/engine/native_clients.rs", import.meta.url), "utf8");
    const pilotLease = readFileSync(new URL("../src-tauri/src/engine/app_instance.rs", import.meta.url), "utf8");
    const sandboxCollector = readFileSync(new URL("../../../tooling/acceptance/windows/collect-sandboxie-feasibility.ps1", import.meta.url), "utf8");
    const desktopConfig = JSON.parse(readFileSync(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"));
    expect(screen).toContain("Minecraft Instance Manager");
    expect(screen).toContain("MinecraftClient02");
    expect(screen).toContain("native unverified");
    expect(screen).toContain("Known credential bypass settings");
    expect(screen).not.toContain("MIVUBI");
    expect(backend).toContain('const CLIENT_02_BOX_NAME: &str = "MinecraftClient02";');
    expect(sandboxCollector).toContain("BoxName = 'MinecraftClient02'");
    // Vendor fallback is bounded, signed, read-only, and cannot infer login.
    expect(sandboxCollector).toContain("function Get-TrustedSbieDll");
    expect(sandboxCollector).toContain("SbieApi_EnumProcessEx");
    expect(sandboxCollector).toContain("if ($membershipStatus -ne 'OBSERVED' -and");
    expect(sandboxCollector).toContain("char[] name = new char[34];");
    expect(sandboxCollector).toContain("uint[] second = Read(query, name, capacity);");
    expect(sandboxCollector).toContain("return new Inventory(first);");
    expect(sandboxCollector).toContain("finalCount != expected");
    expect(sandboxCollector).toContain("$membershipStatus = 'UNKNOWN'");
    expect(sandboxCollector).toContain("if ($gameCandidates.Count -gt 16)");
    expect(pilotLease).toContain('.join("MinecraftInstanceManager")');
    expect(pilotLease).toContain('.join("NativePilot")');
    expect(desktopConfig.app.windows[0].title).toBe("Minecraft Instance Manager");
    expect(desktopConfig.productName).toBe("M-Bedrock Virtual Clients");
    expect(desktopConfig.identifier).toBe("com.mivubi.mbedrock.virtualclients");
  });

  it("keeps the new inspection read-only and no implicit polling", () => {
    const engine = readFileSync(new URL("../src-tauri/src/engine/native_clients.rs", import.meta.url), "utf8");
    const section = engine.slice(
      engine.indexOf("pub fn inspect_native_candidates()"),
      engine.indexOf("pub fn open_native_minecraft()")
    );
    const active = readFileSync(new URL("../src/NativeFeasibilityApp.svelte", import.meta.url), "utf8");
    expect(section).toContain("Get-Process");
    const pattern = "^(Minecraft\\.Windows|Minecraft.*Education.*|Education.*Minecraft.*)$";
    const collector = readFileSync(new URL("../../../tooling/acceptance/windows/collect-native-feasibility.ps1", import.meta.url), "utf8");
    const isolated = readFileSync(new URL("../../../tooling/acceptance/windows/collect-sandboxie-feasibility.ps1", import.meta.url), "utf8");
    for (const source of [engine, collector, isolated]) expect(source).toContain(pattern);
    expect(new RegExp(pattern, "i").test("Minecraft.Windows")).toBe(true);
    expect(new RegExp(pattern, "i").test("EducationHelper")).toBe(false);
    expect(new RegExp(pattern, "i").test("MinecraftJava")).toBe(false);
    expect(engine).toContain("RegistryView]::Registry32");
    expect(engine).toContain("Get-AppxPackage -Name 'Microsoft.MinecraftEducationEdition'");
    expect(engine).toContain("Get-StartApps");
    expect(section).not.toContain("Start-Process");
    expect(section).not.toContain("Get-Content");
    expect(section).not.toContain("Set-Item");
    expect(active).not.toContain("onMount(");
    expect(active).not.toContain("setInterval");
    expect(active).not.toContain("localStorage");
  });
  it("admits modern Desktop registration without accepting command text or ambiguous installs", () => {
    const engine = readFileSync(new URL("../src-tauri/src/engine/native_clients.rs", import.meta.url), "utf8");
    const preflight = engine.match(/const EDUCATION_DESKTOP_PREFLIGHT: &str = r#"([\s\S]*?)"#;/)?.[1] ?? "";
    expect(preflight).toContain("RegistryView]::Registry32");
    expect(preflight).toContain("RegistryView]::Registry64");
    expect(preflight).toContain("$desktopVersions.Count -ne 1 -or $storePresent");
    const pattern = preflight.match(/\$desktopAppId = \$appId -match '([^']+)'/)?.[1];
    expect(pattern).toBeDefined();
    const matcher = new RegExp(pattern!);
    const appId = "{6D809377-6AF0-444B-8957-A3773F02200E}\\Microsoft Studios\\Minecraft Education\\Minecraft.Windows.exe";
    expect(matcher.test(appId)).toBe(true);
    for (const invalid of [appId + '\"', appId + ";calc", appId.replace("Microsoft Studios", ".."), "C:\\Minecraft.Windows.exe", "\\\\server\\game.exe"]) {
      expect(matcher.test(invalid)).toBe(false);
    }
    expect(engine).toContain(String.raw`('"shell:AppsFolder\' + $appId + '"')`);
  });
});

describe("Client 02 retry safety — bounded official launch", () => {
  it("starts the anti-duplicate gap after the helper outcome, not before", () => {
    const source = readFileSync(new URL("../src-tauri/src/engine/native_clients.rs", import.meta.url), "utf8");
    const launch = source.slice(source.indexOf("fn launch_with_official_sandboxie(executable: String, licensed: bool) -> io::Result<()>"),
      source.indexOf("#[cfg(not(windows))]\nfn launch_with_official_sandboxie"));
    const start = launch.indexOf("let launch_result = virtual_clients_core::bounded_command_output(");
    const mark = launch.indexOf("*launch_guard = Some(Instant::now());");
    const complete = launch.indexOf("launch_result?;");
    expect(start).toBeGreaterThan(-1);
    expect(mark).toBeGreaterThan(start);
    expect(complete).toBeGreaterThan(mark);
    expect(source).toContain("client_02_launch_in_retry_gap(*guard, Instant::now())");
    expect(source).toContain("fn client_02_retry_gap_uses_helper_completion_timestamp()");
    expect(launch).toContain("let mut launch_guard = acquire_client_02_launch()?;");
    expect(launch).toContain("admit_client_02_launch_snapshot(&observation)?;");
    expect(launch).not.toContain("desktop_activate_owned_engine");
  });
});


import {parseNativeWorkspaceObservation} from '../src/app/bridge/nativeFeasibilityApi.js';

describe('Official Minecraft 01 direct workspace integration',()=>{
  const process=()=>({pid:123,createdFileTime:'134000000000000001',sessionId:1,windowHandle:'0xabc',responding:true});
  const report=()=>({schema:1,evidenceClass:'REGISTERED_IMAGE_PATH_CANDIDATE_ONLY',processes:[process()]});
  it('accepts one exact candidate or no running process, without sign-in claims',()=>{
    expect(parseNativeWorkspaceObservation(report()).processes[0].pid).toBe(123);
    expect(parseNativeWorkspaceObservation({...report(),processes:[]}).processes).toEqual([]);
    expect(parseNativeWorkspaceObservation(report())).not.toHaveProperty('signedIn');
  });
  it('refuses ambiguous process identity and forged account evidence',()=>{
    for(const bad of [
      {...report(),schema:2},
      {...report(),evidenceClass:'FOUR_CLIENT_READY'},
      {...report(),processes:[process(),process()]},
      {...report(),accountEmail:'hidden'},
      {...report(),processes:[{...process(),sessionId:0}]},
      {...report(),processes:[{...process(),createdFileTime:'0'}]},
      {...report(),processes:[{...process(),windowHandle:'bad'}]},
    ])expect(()=>parseNativeWorkspaceObservation(bad)).toThrow(TypeError);
  });
  it('rechecks native presentation before legacy provider access',()=>{
    const commands=readFileSync(new URL('../src-tauri/src/commands/native_clients.rs',import.meta.url),'utf8');
    const start=commands.indexOf('if client_id == "01" {',commands.indexOf('pub async fn desktop_present_workspace'));
    const legacy=commands.indexOf('let request = OwnedRequest',start);
    expect(start).toBeGreaterThan(0);
    expect(legacy).toBeGreaterThan(start);
    expect(commands.slice(start,legacy)).toContain('native_clients::inspect_native_workspace()?');
    expect(commands.slice(start,legacy)).toContain('matches_native_workspace_target');
    const bootstrap=readFileSync(new URL('../src-tauri/src/app_bootstrap.rs',import.meta.url),'utf8');
    expect(bootstrap).toContain('commands::native_clients::desktop_inspect_native_workspace,');
  });
});
