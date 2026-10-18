import { describe, expect, it } from "vitest";
import { parse } from "svelte/compiler";
import { readFileSync } from "node:fs";
import {
  client02Presentation, focusCandidate, rowPresentation,
  safeErrorDetails, setupInstruction, shouldClearPickerIssue,
  preserveClient02LaunchFeedback,
  ownedVirtualCreationReason,
  isResourceOperationBusy,
} from "../src/app/presentation/compactManager.js";
import type { IsolatedClientObservation, OwnedHostInspection } from "../src/app/bridge/nativeFeasibilityApi.js";

function inspected(): IsolatedClientObservation {
  return {
    schema: 6, boxName: "MinecraftClient02", stage: "INSPECT",
    boxConfigured: "YES", boxSettingsSource: "BOX_SECTION_ENUMERATED",
    autoDelete: "OFF", autoRemove: "OFF", credentialHostAccess: "OFF",
    membershipStatus: "OBSERVED", interactiveSession: true,
    processEvidenceClass: "NAME_MATCH_CANDIDATE_ONLY",
    candidateProcessCount: 0, processCandidates: [],
    officialImageOutsideSelectedBoxCount: 0,
    directPidQuery: "NOT_REQUESTED", signedImageOtherBoxCount: 0,
    signedImageUnknownBoxCount: 0,
    isolatedCandidateProcesses: 0, nativeVisibleCandidates: 0,
    isolatedVisibleCandidates: 0,
    interpretation: "CONFIG_AND_PROCESS_CANDIDATES_ONLY_NO_AUTHENTICATION_PROOF",
  };
}

describe("MIVUBI Create Virtual readiness", () => {
  const app = readFileSync(new URL("../src/CompactManagerApp.svelte", import.meta.url), "utf8");
  const probe = (state: OwnedHostInspection["state"]): OwnedHostInspection => ({
    schema: 1, state, interactiveSession: true,
    serviceRegistration: state === "BLOCKED_SERVICE_NAMESPACE_IN_USE" ? "PRESENT" : "ABSENT",
    driverRegistration: "ABSENT", serviceNamespaceAvailable: false,
    providerSelected: false, trustedDriverLoaded: false,
    gameplayVerified: false, activationAllowed: false,
  });

  it("explains that source and host-name presence cannot authorize creation", () => {
    expect(ownedVirtualCreationReason(null)).toContain("not yet installed or validated");
    expect(ownedVirtualCreationReason(probe("BLOCKED_SERVICE_NAMESPACE_IN_USE")))
      .toContain("already registered");
    expect(ownedVirtualCreationReason(probe("BLOCKED_UNVERIFIED_OWNED_DRIVER")))
      .toContain("no signed and verified MIVUBI driver");
    expect(ownedVirtualCreationReason(probe("BLOCKED_SERVICE_PROBE_UNKNOWN")))
      .toContain("could not verify");
    expect(ownedVirtualCreationReason(probe("BLOCKED_NON_INTERACTIVE_SESSION")))
      .toContain("interactive Windows desktop");
  });

  it("uses one existing Rust engine probe in the UI and keeps Create disabled", () => {
    expect(app).toContain("ownedVirtualCreationReason(ownedHostInspection)");
    expect(app).toContain('on:click={inspectOwnedServiceNames}');
    expect(app).toContain("Check engine readiness");
    expect(app).toContain("MivubiEdu02, MivubiEdu03 and MivubiEdu04");
    expect(app).toContain("MIVUBI has not created or verified these virtuals.");
    expect(app).toContain('title="Internal isolation engine is not yet installed and verified">Create virtual</button>');
    expect(app).not.toContain("desktop_create_virtual");
  });
});

describe("Native Windows readiness probe optimization", () => {
  it("uses direct read-only SCM calls instead of starting PowerShell/WMI", () => {
    const source = readFileSync(new URL("../src-tauri/src/engine/owned_isolation.rs", import.meta.url), "utf8");
    const win = source.split("pub fn inspect_host_service_namespace()")[1]
      ?.split("#[cfg(not(windows))]")[0] ?? "";
    expect(win).toContain("OpenSCManagerW");
    expect(source).toContain("OpenServiceW");
    expect(win).toContain('read_native_service_registration(manager, "SbieSvc")');
    expect(win).toContain("SC_MANAGER_CONNECT");
    expect(win).toContain("CloseServiceHandle");
    expect(win).toContain("interpret_host_probe(OwnedHostProbe");
    expect(win).not.toContain("bounded_command_output");
    expect(win).not.toContain("probe-host-service-names.ps1");
    expect(win).not.toContain("CreateServiceW");
    expect(win).not.toContain("StartServiceW");
    expect(source).toContain('if error == SERVICE_DOES_NOT_EXIST { "ABSENT" } else { "UNKNOWN" }');
    expect(source).toContain("activation_allowed: false");
  });
});

describe("Balanced resource admission", () => {
  it("permits a single explicit request when the manager is idle", () => {
    expect(isResourceOperationBusy("", false, false)).toBe(false);
  });

  it("blocks overlapping Windows helpers and package hashing independently", () => {
    for (const state of [
      ["check", false, false],
      ["open-01", false, false],
      ["open-02", false, false],
      ["move", false, false],
      ["discover-virtuals", false, false],
      ["", true, false],
      ["", false, true],
      ["check", true, true],
    ] as const) {
      expect(isResourceOperationBusy(...state)).toBe(true);
    }
  });

  it("guards every runtime action and disables action buttons during costly inspection", () => {
    const application = readFileSync(new URL("../src/CompactManagerApp.svelte", import.meta.url), "utf8");
    expect(application).toContain(
      '$: resourceOperationBusy = isResourceOperationBusy(busy, ownedHostBusy, ownedInspectionBusy);');
    expect(application).not.toContain("disabled={Boolean(busy)}");
    expect(application).not.toContain("if (busy) return;");
    expect(application).not.toContain("if (busy ||");
    expect(application).not.toContain("setInterval(");
    expect(application).not.toContain("setTimeout(");
    expect(application).not.toContain("onMount(");
    for (const action of [
      "checkStatus", "openMinecraft01", "selectMinecraft", "openMinecraft02",
      "switchMinecraft02", "moveMinecraft02", "discoverExistingVirtuals",
      "inspectOwnedServiceNames", "inspectStagedOwnedEngine",
    ]) {
      const body = application.split(`async function ${action}()`)[1]
        ?.split("  async function ")[0] ?? "";
      expect(body).toContain("isResourceOperationBusy(busy, ownedHostBusy, ownedInspectionBusy)");
    }
    expect(application).toContain("disabled={resourceOperationBusy}");
    // This UI admission protects from overlapping helpers, not against
    // a compromised direct IPC caller; backend launch guards remain separate.
  });
});

describe("Compact Manager projections", () => {
  it("does not treat a missing observation as stopped or ready", () => {
    expect(client02Presentation(undefined, "", false)).toMatchObject({ status: "Not checked", action: "check", canMove: false });
  });
  it("offers a single explicit read-only first-use check without mistaking unknown state for readiness", () => {
    const missing = client02Presentation(undefined, "a previously selected EXE", true);
    expect(missing).toMatchObject({
      status: "Not checked", action: "check", canMove: false,
    });
    expect(missing.action).not.toBe("open");
    expect(rowPresentation("02", missing, "check", null, null))
      .toMatchObject({ status: "Checking…", action: null, canMove: false });
    expect(rowPresentation("02", missing, "", {
      instance: "02", title: "Couldn't check status", message: "Inspect again.",
    }, null)).toMatchObject({ action: "issue", canMove: false });
    const source = readFileSync(new URL("../src/CompactManagerApp.svelte", import.meta.url), "utf8");
    expect(source).toContain('if (action === "check") return "Check status"');
    expect(source).toContain('if (id === "02" && action === "check") { void checkStatus(); return; }');
    expect(source).toContain('row02.action === "check" ? "Check status for Minecraft 02"');
    expect(source).not.toContain("desktop_create_virtual");
  });
  it("keeps native Open separate from any isolated setup or inspection blocker", () => {
    const isolation = client02Presentation({ ...inspected(), boxConfigured: "NO", boxSettingsSource: "UNKNOWN", credentialHostAccess: "UNKNOWN" }, "", false);
    expect(isolation.action).toBe("setup");
    expect(rowPresentation("01", isolation, "", null, null).action).toBe("open");
    expect(rowPresentation("02", isolation, "", null, null).action).toBe("setup");
  });
  it("requires the entire proven-empty-box and UI setup before enabling Open", () => {
    expect(client02Presentation(inspected(), "", false).action).toBe("setup");
    expect(client02Presentation(inspected(), "some-selected-path", false).action).toBe("setup");
    expect(client02Presentation(inspected(), "some-selected-path", true).action).toBe("open");
    for (const change of [
      { membershipStatus: "UNKNOWN" as const },
      { autoDelete: "UNKNOWN" as const },
      { autoRemove: "ON" as const },
      { credentialHostAccess: "ON" as const },
      { interactiveSession: false },
    ]) expect(client02Presentation({ ...inspected(), ...change }, "some-selected-path", true).action).not.toBe("open");
  });
  it("never treats a boxed process without one eligible window as an empty box", () => {
    const ambiguous = { ...inspected(), isolatedCandidateProcesses: 1, candidateProcessCount: 1,
      processCandidates: [{ pid: 123, windowHandle: null, createdFileTime: null, inSelectedBox: true, imageEvidence: "UNKNOWN" as const }] };
    expect(client02Presentation(ambiguous, "path", true)).toMatchObject({ action: "issue", canMove: false });
    expect(focusCandidate(ambiguous)).toBeNull();
  });
  it("focuses only a unique last-observed boxed window and leaves signing proof to Rust", () => {
    const candidate = { pid: 123, windowHandle: "0x100", createdFileTime: "133400000000000000", inSelectedBox: true, imageEvidence: "UNKNOWN" as const };
    const found = { ...inspected(), isolatedCandidateProcesses: 1, isolatedVisibleCandidates: 1, candidateProcessCount: 1,
      processCandidates: [candidate] };
    expect(focusCandidate(found)).toEqual(candidate);
    expect(client02Presentation(found, "", false)).toMatchObject({ status: "Window detected", action: "switch", canMove: true });
    expect(focusCandidate({ ...found, membershipStatus: "UNKNOWN" })).toBeNull();
    expect(focusCandidate({ ...found, isolatedCandidateProcesses: 2 })).toBeNull();
  });
  it("does not mark Client 01 as being checked by Client 02 inspection", () => {
    const isolated = client02Presentation(inspected(), "path", true);
    expect(rowPresentation("01", isolated, "check", null, null).status).toBe("Not checked");
    expect(rowPresentation("01", isolated, "", null, null).status).toBe("Not checked");
    expect(rowPresentation("02", isolated, "check", null, null))
      .toMatchObject({ status: "Checking…", action: null });
  });
  it("never interprets a Virtual 02 inspection as Host 01 status", () => {
    const observation02 = client02Presentation(inspected(), "some-selected-path", true);
    const native = rowPresentation("01", observation02, "", null, null);
    expect(native).toMatchObject({ status: "Not checked", action: "open" });
    const nativeFeedback = rowPresentation("01", observation02, "", null,
      { instance: "01", text: "Open requested" });
    expect(nativeFeedback).toMatchObject({ status: "Open requested", action: null });
    const app = readFileSync(new URL("../src/CompactManagerApp.svelte", import.meta.url), "utf8");
    expect(app).toContain('rowPresentation("01", base02, busy, issue01, notice01)');
    expect(app).toContain('rowPresentation("02", base02, busy, issue02, notice02)');
    expect(app).not.toContain("rowPresentation(\"01\", observation !== undefined");
    expect(app).not.toContain("Status unconfirmed");
  });
  it("lets users review and dismiss stale Host 01 feedback without launching or claiming stopped gameplay", () => {
    const native = rowPresentation("01", client02Presentation(undefined, "", false), "", null,
      { instance: "01", text: "Open requested" });
    expect(native).toMatchObject({ status: "Open requested", action: null });
    const blocked = rowPresentation("01", client02Presentation(undefined, "", false), "",
      { instance: "01", title: "Couldn't confirm if Minecraft opened", message: "Check Windows" }, null);
    expect(blocked).toMatchObject({ action: "issue", canMove: false });
    const src = readFileSync(new URL("../src/CompactManagerApp.svelte", import.meta.url), "utf8");
    const dismiss = src.split("function dismissHost01Feedback()")[1]
      ?.split("function reportIssue(")[0] ?? "";
    expect(dismiss).toContain('(!issues["01"] && !notices["01"])');
    expect(dismiss).toContain('if (resourceOperationBusy || page !== "instances" || expanded !== "01"');
    expect(dismiss).toContain('clearFeedback("01");');
    expect(dismiss).toContain("navigationRevision += 1;");
    expect(dismiss).toContain('host01OpenButton?.isConnected && !host01OpenButton.disabled');
    expect(dismiss).not.toContain("nativeFeasibility.");
    const hostDetails = src.split('<div id="cm-details-01"')[1]
      ?.split('<h2>Virtual 02 · Minecraft Education</h2>')[0] ?? "";
    expect(hostDetails).toContain('{:else if notice01}');
    expect(hostDetails).toContain("This app cannot confirm whether the game is still open.");
    expect(hostDetails).toContain("Clearing this message does not stop Minecraft or verify it is closed.");
    expect(hostDetails).toContain("I checked Windows · Clear message");
    expect(hostDetails).toContain("on:click={dismissHost01Feedback}");
    expect(src).toContain("bind:this={host01DetailsButton}");
    expect(src).toContain("bind:this={host01OpenButton}");
    const launch = src.split("async function openMinecraft01()")[1]
      ?.split("async function selectMinecraft()")[0] ?? "";
    expect(launch).toContain('requestRevision === navigationRevision');
    expect(launch).toContain("host01DetailsButton.focus()");
    expect(src).not.toContain("setTimeout(");
  });
  it("retains native errors while displaying isolated request outcomes", () => {
    const isolated = client02Presentation(inspected(), "path", true);
    const nativeIssue = { instance: "01" as const, title: "Native launch uncertain", message: "Look for existing window." };
    const boxedNotice = { instance: "02" as const, text: "Open requested" };
    expect(rowPresentation("01", isolated, "", nativeIssue, null))
      .toMatchObject({ status: "Native launch uncertain", action: "issue" });
    expect(rowPresentation("02", isolated, "", null, boxedNotice))
      .toMatchObject({ status: "Open requested", action: null });
  });
  it("does not present Client 02 as ready while executable selection is pending", () => {
    const safeBox = inspected();
    const initial = client02Presentation(safeBox, "C:/old.exe", true);
    expect(initial).toMatchObject({ status: "Ready to open", action: "open" });
    expect(rowPresentation("02", initial, "select", null, null))
      .toMatchObject({ status: "Selecting…", action: null, canMove: false });
    expect(rowPresentation("01", initial, "select", null, null).status).toBe("Not checked");
    const afterChangedFile = client02Presentation(safeBox, "C:/new.exe", false);
    expect(afterChangedFile).toMatchObject({ status: "Setup required", action: "setup" });
    expect(client02Presentation(safeBox, "C:/new.exe", true))
      .toMatchObject({ status: "Ready to open", action: "open" });
    const uncertain = { instance: "02" as const, title: "Couldn't confirm if Minecraft opened",
      message: "Check status before opening another copy." };
    expect(rowPresentation("02", afterChangedFile, "", uncertain, null))
      .toMatchObject({ status: uncertain.title, action: "issue", canMove: false });
    expect(client02Presentation(undefined, "C:/new.exe", true))
      .toMatchObject({ status: "Not checked", action: "check", canMove: false });
  });
  it("does not show an available action during busy and uncertain operations", () => {
    const base = client02Presentation(inspected(), "path", true);
    expect(rowPresentation("02", base, "open-02", null, null)).toMatchObject({ status: "Opening…", action: null });
    const issue = { instance: "02" as const, title: "Couldn't confirm if Minecraft opened", message: "Check status." };
    expect(rowPresentation("02", base, "", issue, null)).toMatchObject({ action: "issue", canMove: false });
    expect(rowPresentation("02", base, "", null, { instance: "02", text: "Open requested" }))
      .toMatchObject({ status: "Open requested", action: null });
  });
  it("keeps safe Switch access after a move request, not after a launch request", () => {
    const observed = {
      status: "Window detected", tone: "neutral" as const,
      action: "switch" as const, canMove: true,
      details: "Unique name-matched window; backend will revalidate.",
    };
    const moved = { instance: "02" as const, text: "Move requested" };
    expect(rowPresentation("02", observed, "", null, moved))
      .toMatchObject({ status: "Move requested", action: "switch", canMove: true });
    expect(rowPresentation("02", observed, "move", null, moved))
      .toMatchObject({ status: "Moving…", action: null, canMove: false });
    expect(rowPresentation("02", observed, "", {
      instance: "02" as const, title: "Couldn't move", message: "Check status.",
    }, moved)).toMatchObject({ action: "issue", canMove: false });
    const notObserved = { ...observed, action: "setup" as const, canMove: false };
    expect(rowPresentation("02", notObserved, "", null, moved))
      .toMatchObject({ action: null, canMove: false });
    expect(rowPresentation("02", observed, "", null, { instance: "02", text: "Open requested" }))
      .toMatchObject({ status: "Open requested", action: null, canMove: false });
    expect(rowPresentation("01", observed, "", null, { instance: "01", text: "Move requested" }))
      .toMatchObject({ action: null, canMove: false });
    const app = readFileSync(new URL("../src/CompactManagerApp.svelte", import.meta.url), "utf8");
    const move = app.split("async function moveMinecraft02()")[1]?.split("function choosePrimary(")[0] ?? "";
    const focus = app.split("async function switchMinecraft02()")[1]?.split("async function moveMinecraft02()")[0] ?? "";
    expect(move).toContain("await nativeFeasibility.placeObservedClient02(target, moveSide);");
    expect(move).toContain('text: "Move requested"');
    expect(focus).toContain("await nativeFeasibility.focusObservedClient02(target);");
  });
  it("keeps a failed or timed-out Client 02 Open request guarded through recheck", () => {
    const source = readFileSync(new URL("../src/CompactManagerApp.svelte", import.meta.url), "utf8");
    const launch = source.split("async function openMinecraft02()")[1]
      ?.split("async function switchMinecraft02()")[0] ?? "";
    const failure = launch.split("} catch (error) {")[1]?.split("} finally {")[0] ?? "";
    expect(launch).toContain("await nativeFeasibility.requestSandboxClient02(selectedExecutable, licenseConfirmed);");
    expect(failure).toContain("reportIssue(\"02\", \"Couldn't confirm if Minecraft opened\"");
    expect(failure).toContain("The launch request may have started Minecraft despite an error.");
    expect(failure).toContain('notices = { ...notices, "02": { instance: "02", text: "Open requested" } };');
    expect(failure.indexOf("reportIssue(")).toBeLessThan(failure.indexOf("notices = {"));
    const check = source.split("async function checkStatus()")[1]
      ?.split("async function openMinecraft01()")[0] ?? "";
    expect(check).toContain('const pendingLaunchNotice = notices["02"]?.text === "Open requested"');
    expect(check).toContain("!preserveClient02LaunchFeedback(pendingLaunchNotice, result)");
    expect(check.match(/notices = \{ \.\.\.notices, "02": pendingLaunchNotice \};/g)).toHaveLength(2);
    const pending = { instance: "02" as const, text: "Open requested" };
    const empty = client02Presentation(inspected(), "selected.exe", true);
    expect(rowPresentation("02", empty, "", null, pending))
      .toMatchObject({ status: "Open requested", action: null, canMove: false });
    expect(rowPresentation("02", empty, "", {
      instance: "02", title: "Couldn't confirm if Minecraft opened", message: "Review.",
    }, pending)).toMatchObject({ action: "issue", canMove: false });
    expect(preserveClient02LaunchFeedback(pending, inspected())).toBe(true);
    expect(preserveClient02LaunchFeedback(pending, {
      ...inspected(), membershipStatus: "UNKNOWN",
    })).toBe(true);
    expect(preserveClient02LaunchFeedback(pending, {
      ...inspected(), isolatedCandidateProcesses: 1,
    })).toBe(false);
    expect(failure).not.toContain("requestSandboxClient02(");
    expect(check).not.toContain("requestSandboxClient02(");
  });
  it("retains an unresolved Client 02 launch across quick or failed checks", () => {
    const pending = { instance: "02" as const, text: "Open requested" };
    expect(preserveClient02LaunchFeedback(pending, undefined)).toBe(true);
    expect(preserveClient02LaunchFeedback(pending, inspected())).toBe(true);
    expect(preserveClient02LaunchFeedback(pending,
      { ...inspected(), membershipStatus: "UNKNOWN" })).toBe(true);
    expect(preserveClient02LaunchFeedback(pending,
      { ...inspected(), isolatedCandidateProcesses: 1 })).toBe(false);
    expect(preserveClient02LaunchFeedback({ instance: "02", text: "Move requested" }, inspected())).toBe(false);
    expect(preserveClient02LaunchFeedback({ instance: "01", text: "Open requested" }, inspected())).toBe(false);
    const unknown = client02Presentation({ ...inspected(), membershipStatus: "UNKNOWN" }, "selected.exe", true);
    expect(rowPresentation("02", unknown, "", null, pending))
      .toMatchObject({ action: "issue", canMove: false });
    const ready = client02Presentation(inspected(), "selected.exe", true);
    expect(rowPresentation("02", ready, "", null, pending))
      .toMatchObject({ status: "Open requested", action: null });
    const src = readFileSync(new URL("../src/CompactManagerApp.svelte", import.meta.url), "utf8");
    const check = src.split("async function checkStatus()")[1]?.split("async function openMinecraft01()")[0] ?? "";
    expect(check).toContain('const pendingLaunchNotice = notices["02"]?.text === "Open requested"');
    expect(check.match(/notices = \{ \.\.\.notices, "02": pendingLaunchNotice \};/g)).toHaveLength(2);
    expect(check).toContain('!preserveClient02LaunchFeedback(pendingLaunchNotice, result)');
    const dismiss = src.split("function dismissClient02LaunchFeedback()")[1]?.split("function reportIssue(")[0] ?? "";
    expect(dismiss).toContain('!lastCheckIsCurrent || !canRequestClient02(observation)');
    expect(dismiss).toContain('notices["02"]?.text !== "Open requested" || issues["02"]');
    expect(dismiss).toContain('notices = { ...notices, "02": undefined };');
    expect(dismiss).not.toContain("nativeFeasibility.");
    const panel = src.split('<div id="cm-details-02"')[1]?.split('<section class="cm-future-virtuals"')[0] ?? "";
    expect(panel).toContain('{:else if notice02?.text === "Open requested"}');
    expect(panel).toContain('{#if lastCheckIsCurrent && canRequestClient02(observation)}');
    expect(panel).toContain('on:click={dismissClient02LaunchFeedback}');
    expect(src).toContain('{#if row02.action === "switch" || expanded === "02" || notice02}');
    expect(src).not.toContain("setTimeout(");
    expect(src).not.toContain("setInterval(");
  });
  it("the open-request notice cannot be confused with a verified window", () => {
    const base = client02Presentation(inspected(), "path", true);
    expect(rowPresentation("02", base, "", null, { instance: "02", text: "Open requested" }))
      .toMatchObject({ status: "Open requested", action: null });
  });
  it("hides launch setup until Virtual 02 has a safely observed empty pilot box", () => {
    const unknownMembership = { ...inspected(), membershipStatus: "UNKNOWN" as const };
    expect(setupInstruction(unknownMembership).title).toBe("Check instance status");
    expect(client02Presentation(unknownMembership, "selected.exe", true).action).not.toBe("open");
    const occupied = { ...inspected(), isolatedCandidateProcesses: 1 };
    expect(setupInstruction(occupied).title).toBe("Minecraft may already be open");
    expect(setupInstruction(occupied).text).toContain("Do not start another copy");
    expect(client02Presentation(occupied, "selected.exe", true).action).not.toBe("open");
    expect(setupInstruction(inspected()).title).toBe("Select Minecraft Education");
    const app = readFileSync(new URL("../src/CompactManagerApp.svelte", import.meta.url), "utf8");
    const setup = app.split('{:else if detailMode === "setup"}')[1]
      ?.split('{:else if detailMode === "move"')[0] ?? "";
    expect(setup).toContain("{#if !canRequestClient02(observation)}");
    const [blocked, eligible] = setup.split("{:else}");
    expect(blocked).not.toContain("on:click={selectMinecraft}");
    expect(blocked).not.toContain("bind:checked={licenseConfirmed}");
    expect(blocked).not.toContain("on:click={selectMinecraft}");
    expect(setup).toContain('on:click={checkStatus}>Check status · 02</button>');
    expect(eligible).toContain("on:click={selectMinecraft}");
    expect(eligible).toContain("bind:checked={licenseConfirmed}");
    expect(app).toContain("Create it manually only if it does not already exist.");
    const issue = app.split('{#if detailMode === "issue"}')[1]
      ?.split('{:else if detailMode === "setup"}')[0] ?? "";
    expect(issue).toContain('on:click={checkStatus}>Check status · 02</button>');
    expect(issue).not.toContain("on:click={openMinecraft02}");
    const inspector = app.split("async function checkStatus()")[1]
      ?.split("async function openMinecraft01()")[0] ?? "";
    expect(inspector).toContain('expanded === "02" && detailMode === "issue") closeDetails();');
    expect(eligible).toContain('on:click={checkStatus}>Check status · 02</button>');
    expect(app).not.toContain("desktop_create_virtual");
  });
  it("gives user-owned setup steps without claiming configuration automation", () => {
    expect(setupInstruction({ ...inspected(), boxConfigured: "NO", boxSettingsSource: "UNKNOWN", credentialHostAccess: "UNKNOWN" }).text).toContain("Sandboxie Plus");
    expect(setupInstruction({ ...inspected(), autoDelete: "ON" }).title).toContain("sandbox data");
  });
  it("resolves only picker-origin errors when an executable is selected", () => {
    const pickerIssue = {
      instance: "02" as const, title: "Couldn't select Minecraft",
      message: "Choose the official executable.",
      origin: "executable-picker" as const,
    };
    const launchIssue = {
      instance: "02" as const, title: "Couldn't confirm if Minecraft opened",
      message: "Inspect the window before retrying.",
    };
    const path = "C:/Program Files/Minecraft Education/Minecraft.Windows.exe";
    expect(shouldClearPickerIssue(pickerIssue, path)).toBe(true);
    expect(shouldClearPickerIssue(pickerIssue, null)).toBe(false);
    expect(shouldClearPickerIssue(pickerIssue, "")).toBe(false);
    expect(shouldClearPickerIssue(launchIssue, path)).toBe(false);
    expect(shouldClearPickerIssue(undefined, path)).toBe(false);
  });
  it("redacts Windows, forward-slash and macOS-style user directories", () => {
    for (const path of [
      "C:\\Users\\PersonalName\\Documents\\file.exe failed",
      "C:/Users/PersonalName/Documents/file.exe failed",
      "/Users/PersonalName/Library/file.exe failed",
    ]) {
      const result = safeErrorDetails(path);
      expect(result).not.toContain("PersonalName");
      expect(result).toContain("[user directory]");
      expect(result).toContain("file.exe failed");
    }
  });
  it("redacts common authentication details before truncating errors", () => {
    const sensitive = [
      "Authorization: Bearer topSecretAccessValue",
      'api_key="credential with spaces"',
      "access_token=highlySensitive",
      "refresh-token=keepprivate",
      "password=123",
      "token=abc",
      "secret=hidden",
      "alex@example.com",
    ];
    for (const value of sensitive) {
      const result = safeErrorDetails(value);
      expect(result).not.toContain("topSecretAccessValue");
      expect(result).not.toContain("credential with spaces");
      expect(result).not.toContain("highlySensitive");
      expect(result).not.toContain("keepprivate");
      expect(result).not.toContain("password=123");
      expect(result).not.toContain("token=abc");
      expect(result).not.toContain("secret=hidden");
      expect(result).not.toContain("alex@example.com");
    }
    expect(safeErrorDetails("a".repeat(700)).length).toBe(450);
  });
});

describe("Compact Manager app source and command boundary", () => {
  const entry = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
  const app = readFileSync(new URL("../src/CompactManagerApp.svelte", import.meta.url), "utf8");
  const bootstrap = readFileSync(new URL("../src-tauri/src/app_bootstrap.rs", import.meta.url), "utf8");
  it("keeps the active Sandboxie localtest instructions aligned with mounted controls", () => {
    const guide = readFileSync(new URL("../../../docs/validation/sandboxie-two-client-feasibility.md", import.meta.url), "utf8");
    expect(app).toContain('{busy === "check" ? "Checking…" : "Check status · 02"}');
    expect(app).toContain(">Settings</button>");
    expect(app).toContain('on:click={showClient02MoveDetails}>Move window</button>');
    expect(guide).toContain(".\\DEV.cmd preflight");
    expect(guide).toContain(".\\DEV.cmd dev");
    expect(guide).toContain("**Check status**");
    expect(guide).toContain("**Set up**");
    expect(guide).toContain("**Open**");
    expect(guide).toContain("**Switch to**");
    expect(guide).toContain("**Move window**");
    expect(guide).toContain("not the historical");
    expect(guide).not.toContain("Click **Refresh both clients**");
    expect(guide).not.toContain("click **Launch isolated Client 02**");
    expect(guide).not.toContain("manual **Focus Client 02** button");
  });
  it("keeps the Windows operator procedure and architecture owner aligned with mounted Compact Manager", () => {
    const localGuide = readFileSync(new URL("../../../docs/validation/native-pilot-localtest.md", import.meta.url), "utf8");
    const map = readFileSync(new URL("../../../docs/system/implementation-map.md", import.meta.url), "utf8");
    const acceptanceSteps = localGuide.split("## B. Two-client manual acceptance sequence")[1]
      ?.split("## C. Read-only collector")[0] ?? "";
    expect(entry).toContain('mount(CompactManagerApp, { target });');
    expect(app).toContain('{busy === "check" ? "Checking…" : "Check status · 02"}');
    expect(app).toContain('on:click={showClient02MoveDetails}>Move window</button>');
    expect(localGuide).toContain(".\\DEV.cmd preflight");
    expect(localGuide).toContain(".\\DEV.cmd dev");
    for (const control of ["**Check status**", "**Set up**", "**Browse…**", "**Open**", "**Switch to**", "**Move window**"]) {
      expect(acceptanceSteps).toContain(control);
    }
    expect(acceptanceSteps).toContain("**Check status** inspects Client 02 Sandboxie/process candidates;");
    expect(acceptanceSteps).toContain("it does not establish Client 01 identity, account authentication or game playability.");
    expect(acceptanceSteps).toContain("do not auto-retry");
    expect(acceptanceSteps).not.toMatch(/Refresh both clients|Launch Client 0[12]|Launch isolated Client 02|Focus Client 02|Place Left/);
    expect(localGuide).toContain("CHECK STATUS (CLIENT 02):");
    expect(localGuide).not.toContain("REFRESH STATUS:");
    expect(localGuide).toContain("**NOT_EXECUTED** for that revision");
    expect(map).toContain("Active desktop navigation / transient visual state | `apps/desktop/src/CompactManagerApp.svelte`");
    expect(map).toContain("Legacy VMware client-owned setup wizard (inactive)");
    expect(map).toContain("legacy VMware layout/overlay (inactive)");
    expect(map).toContain("Explicit **Check status** (`-ImageProof`)");
  });

  it("parses the mounted Svelte component with separate Host 01 click and ARIA attributes", () => {
    // Catch invalid markup before the first Windows localtest.
    expect(parse(app, { filename: "CompactManagerApp.svelte" })).toBeDefined();
    expect(app).toContain(
      'on:click={(event) => choosePrimary("01", event.currentTarget)} aria-expanded={row01.action === "issue" ? expanded === "01" : undefined}'
    );
    expect(app).not.toContain(
      'on:click={(event) => choosePrimary("01", event.currentTarget) aria-expanded='
    );
    expect(app).toContain('aria-controls={row01.action === "issue" ? "cm-details-01" : undefined}');
    expect(app).toContain('on:click={(event) => choosePrimary("02", event.currentTarget)} aria-label=');
  });
  it("mounts exactly one compact UI with no new native providers", () => {
    expect(entry).toContain('mount(CompactManagerApp, { target });');
    expect(entry).not.toContain("NativeFeasibilityApp");
    expect(app).not.toContain("<aside");
    expect(app).toContain("Virtual Manager");
    expect(app).toContain("Settings");
    expect(bootstrap).not.toContain("commands::virtual_clients::virtual_clients_");
  });
  it("treats Sandboxie as an internal MIVUBI capability without pretending new virtuals work", () => {
    expect(app).toContain('aria-label="Create virtual availability"');
    expect(app).toContain('<h1>Virtual Manager</h1>');
    expect(app).toContain('<h2>Host · Minecraft 01</h2>');
    expect(app).toContain('<h2>Virtual 02 · Minecraft Education</h2>');
    expect(app).toMatch(/<button type="button" class="cm-button cm-secondary" disabled[\s\S]*?>Create virtual<\/button>/);
    expect(app).toContain("M-Isolation-Engine · Internal runtime");
    expect(app).toContain("A built, signed and verified MIVUBI isolation engine is required.");
    expect(app).toContain('ownedCandidate.upstreamCommit === "cdba7096240b93d79c5373aa81a75a1c5c71b883"');
    expect(app).not.toContain("Create virtual requested");
    expect(bootstrap).not.toContain("desktop_create_virtual");
  });
  it("shows all four client slots without pretending future clients can launch", () => {
    expect(app).toContain('<h2>Host · Minecraft 01</h2>');
    expect(app).toContain('<h2>Virtual 02 · Minecraft Education</h2>');
    expect(app).toContain('aria-label="Planned virtual clients"');
    expect(app).toContain('<h2>Planned virtuals</h2>');
    expect(app).toContain("Supported today: Host 01 can request a Windows launch.");
    expect(app).toContain("Minecraft sessions");
    expect(app).toContain('{#each ["03", "04"] as client (client)}');
    expect(app).toContain('<h3>Virtual {client} · Minecraft Education</h3>');
    expect(app).toContain('Unavailable in MIVUBI · No managed session');
    expect(app).toContain('Clients 03 and 04 cannot be created or launched yet.');
    expect(app).not.toContain("openMinecraft03");
    expect(app).not.toContain("openMinecraft04");
    expect(bootstrap).not.toContain("desktop_create_virtual");
  });
  it("explains disabled Create Virtual for keyboard and screen-reader users", () => {
    expect(app).toContain('aria-describedby="cm-create-reason"');
    expect(app).toContain('id="cm-create-reason" aria-live="polite"');
    expect(app).toContain("Create Virtual unavailable.");
    expect(app).toContain("has not completed Windows validation.");
    expect(app).toContain('aria-label={issue01 ? "Minecraft 01 error details" : "Minecraft 01 launch request details"}');
    expect(app).toContain('aria-label="Minecraft 02 details"');
    expect(app).toContain("Check status · 02");
    expect(app).toContain("Inspects Virtual 02 only; Host 01 is not checked");
    expect(app).toContain('aria-pressed={moveSide === "LEFT"} disabled={resourceOperationBusy}');
    expect(app).toContain('aria-pressed={moveSide === "RIGHT"} disabled={resourceOperationBusy}');
    expect(app).toContain('tabindex="-1" bind:this={client02DetailPanel}');
  });
  it("keeps the four Minecraft slots primary and puts technical Sandboxie controls only in Settings Diagnostics", () => {
    const overview = app.split('<section class="cm-virtual-overview"')[1]
      ?.split('<section class="cm-list"')[0] ?? "";
    expect(overview).toContain("Minecraft sessions");
    expect(overview).toContain("Create Virtual unavailable.");
    expect(overview).toContain("Settings → Diagnostics");
    expect(overview).not.toContain("Inspect Sandboxie names");
    expect(overview).not.toContain("Check engine readiness");
    expect(overview).not.toContain("MivubiEdu02");
    const diagnostics = app.indexOf('<details class="cm-diagnostics">');
    const help = app.indexOf("<h2>Help</h2>");
    expect(diagnostics).toBeGreaterThan(help);
    for (const marker of [
      '<summary>Diagnostics</summary>',
      'on:click={discoverExistingVirtuals}',
      'on:click={inspectStagedOwnedEngine}',
      'on:click={inspectOwnedServiceNames}',
      '{#if existingVirtuals}',
      '{#if existingVirtualsError}',
      'ownedVirtualCreationReason(ownedHostInspection)',
      'Supported owned virtuals',
    ]) {
      expect(app.indexOf(marker)).toBeGreaterThan(diagnostics);
    }
    for (const action of [
      'on:click={discoverExistingVirtuals}',
      'on:click={inspectStagedOwnedEngine}',
      'on:click={inspectOwnedServiceNames}',
    ]) {
      expect(app.split(action)).toHaveLength(2);
    }
    expect(app).toContain("These checks do not create, install, migrate or sign in to any virtual.");
    expect(bootstrap).not.toContain("desktop_create_virtual");
  });
  it("lists read-only Sandboxie section candidates without authorizing virtual creation", () => {
    expect(app).toContain("async function discoverExistingVirtuals()");
    expect(app).toContain("nativeFeasibility.discoverExistingVirtuals()");
    expect(app).toContain('on:click={discoverExistingVirtuals}');
    expect(app).toContain('existingVirtuals = null;');
    expect(app).toContain("No virtuals were changed.");
    expect(app).toContain("Sandboxie name candidates (read-only)");
    expect(app).toContain('aria-label="Sandboxie configuration name candidates"');
    expect(app).toContain("Inspect Sandboxie names");
    expect(app).not.toContain("No enabled boxes were reported");
    expect(app).not.toContain("Find existing virtuals");
    expect(app).toContain("Known user and template sections are omitted");
    expect(app).toContain("names alone cannot prove a box exists");
    expect(app).toContain('{#each existingVirtuals.boxes as boxName (boxName)}');
    expect(app).toContain('title="Internal isolation engine is not yet installed and verified">Create virtual</button>');
    expect(app).not.toMatch(/onMount\s*\(|setInterval\s*\(/);
  });
  it("keeps checking explicitly user-triggered and launch calls guarded", () => {
    expect(app).toContain("nativeFeasibility.inspectIsolatedClient02()");
    expect(app).toContain("nativeFeasibility.requestSandboxClient02(");
    expect(app).toContain("nativeFeasibility.openOfficialMinecraft()");
    expect(app).toContain("nativeFeasibility.focusObservedClient02(");
    expect(app).toContain("nativeFeasibility.placeObservedClient02(");
    expect(app).not.toMatch(/onMount\s*\(|setInterval\s*\(|setTimeout\s*\(/);
    expect(app).not.toContain("localStorage");
    expect(app).not.toContain("Stop All");
    expect(app).not.toContain("Start All");
  });
  it("restores keyboard focus when a panel closes and allows Escape to close it", () => {
    expect(app).toContain('import { tick } from "svelte"');
    expect(app).toContain("origin?.isConnected && !origin.disabled");
    const close = app.split("function closeDetails()")[1]?.split("function showSettings()")[0] ?? "";
    expect(close).toContain("else if (checkStatusButton?.isConnected && !checkStatusButton.disabled) checkStatusButton.focus();");
    expect(app).toContain('<svelte:window on:keydown={handleWindowKeydown} />');
    expect(app).toContain('event.key !== "Escape"');
    expect(app.match(/on:click={closeDetails}>Close details/g)).toHaveLength(2);
  });
  it("does not close or replace the other client's open issue details", () => {
    const inspector = app.split("async function checkStatus()")[1]?.split("async function openMinecraft01()")[0] ?? "";
    expect(inspector).toContain('expanded === "02" && detailMode === "issue") closeDetails();');
    const issueReport = app.split("function reportIssue(")[1]?.split("async function checkStatus()")[0] ?? "";
    expect(issueReport).toContain('(expanded === null || expanded === instance)');
    expect(issueReport).toContain("issues = { ...issues, [instance]");
  });
  it("tracks instance-local issue/notice feedback, not a shared global message", () => {
    expect(app).toContain("let issues: Partial<Record<InstanceId, UiIssue>> = {}");
    expect(app).toContain("let notices: Partial<Record<InstanceId, UiNotice>> = {}");
    expect(app).toContain('clearFeedback("01");');
    expect(app.match(/clearFeedback\("02"\);/g)).toHaveLength(4);
    expect(app).toMatch(/issues = \{ \.\.\.issues, \[instance\]: \{\s+instance, title, message,/);
    expect(app).toContain('notices = { ...notices, [instance]: undefined }');
    expect(app).toContain('rowPresentation("01", base02, busy, issue01, notice01)');
    expect(app).toContain('rowPresentation("02", base02, busy, issue02, notice02)');
    expect(app).not.toContain("let issue: UiIssue | null");
    expect(app).not.toContain("let notice: UiNotice | null");
    expect(app).toContain('{#if issue01}');
    expect(app).toContain('{#if issue02?.details}');
  });
  it("counts the inline Move window action as user navigation for pending IPC results", () => {
    const action = app.split("function showClient02MoveDetails()")[1]?.split("function expandDetails(")[0] ?? "";
    expect(action).toContain('page !== "instances" || expanded !== "02" || !moveEligible');
    expect(action).toContain("navigationRevision += 1;");
    expect(action).toContain('detailMode = "move";');
    expect(app).toContain('on:click={showClient02MoveDetails}>Move window</button>');
    expect(app).not.toContain('on:click={() => (detailMode = "move")}');
  });
  it("fails closed on Focus/Move errors without discarding the error or native feedback", () => {
    const focus = app.split("async function switchMinecraft02()")[1]?.split("async function moveMinecraft02()")[0] ?? "";
    const move = app.split("async function moveMinecraft02()")[1]?.split("function choosePrimary(")[0] ?? "";
    for (const operation of [focus, move]) {
      const failure = operation.split("} catch (error) {")[1]?.split("} finally {")[0] ?? "";
      // Never leave PID/HWND eligibility active when a backend revalidation
      // or asynchronous Win32 outcome fails.
      expect(failure).toContain("invalidateCurrentInspection();");
      expect(failure).toContain('reportIssue("02",');
      expect(failure.indexOf("invalidateCurrentInspection();"))
        .toBeLessThan(failure.indexOf('reportIssue("02",'));
      expect(operation).toContain('busy = "";');
      expect(operation).toContain("const requestRevision = navigationRevision;");
      expect(operation).toContain('clearFeedback("02");');
    }
    expect(focus).toContain("focusObservedClient02(target)");
    expect(move).toContain("placeObservedClient02(target, moveSide)");
    expect(move).toContain("Check status before trying again.");
    expect(move).toContain("If Minecraft is maximized or minimized");
    expect(move).not.toContain("Restore Minecraft to a normal window, then check status");
    expect(app).toContain("lastCheckIsCurrent ?");
  });
  it("keeps IPC outcome effects subordinate to later UI navigation", () => {
    expect(app).toContain("let navigationRevision = 0;");
    for (const operation of [
      "checkStatus", "openMinecraft01", "selectMinecraft",
      "openMinecraft02", "switchMinecraft02", "moveMinecraft02",
    ]) {
      const body = app.split(`async function ${operation}()`)[1]?.split("async function ")[0] ?? "";
      expect(body).toContain("const requestRevision = navigationRevision;");
      expect(body).not.toMatch(/reportIssue\([\s\S]*?, error\);/);
    }
    expect(app).toContain('if (page === "instances" && requestRevision === navigationRevision &&');
    expect(app).toContain('(expanded === null || expanded === instance)');
    expect(app).toContain('expanded === "02" && detailMode === "issue") closeDetails();');
    expect(app).toContain('requestedPanel === "02" && expanded === requestedPanel && detailMode === requestedMode');
    expect(app).toContain('if (selected && page === "instances" && requestRevision === navigationRevision && expanded === "02")');
    expect(app).not.toContain("      issue = null;\n      detailMode = \"setup\";");
  });
  it("rejects unrelated picked EXEs before UI claims Minecraft and clears stale consent on errors", () => {
    const rust = readFileSync(new URL("../src-tauri/src/engine/native_clients.rs", import.meta.url), "utf8");
    const picker = rust.split("#[cfg(windows)]\npub fn select_education_exe()")[1]
      ?.split("#[cfg(not(windows))]")[0] ?? "";
    expect(picker).toContain("validate_education_path(executable)?;");
    expect(picker).toContain("if !path.is_file()");
    expect(picker).toContain(".transpose()");
    const launch = rust.split("fn launch_with_official_sandboxie(executable: String, licensed: bool)")[1]
      ?.split("#[cfg(not(windows))]")[0] ?? "";
    expect(launch).toContain("validate_education_path(&executable)?;");
    expect(launch).toContain("Get-AuthenticodeSignature -LiteralPath $game");
    const ui = app.split("async function selectMinecraft()")[1]
      ?.split("async function openMinecraft02()")[0] ?? "";
    const failure = ui.split("} catch (error) {")[1]
      ?.split("} finally {")[0] ?? "";
    expect(failure).toContain('selectedExecutable = "";');
    expect(failure).toContain("licenseConfirmed = false;");
    expect(failure).toContain('reportIssue("02", "Couldn\'t select Minecraft"');
    expect(failure).toContain('"executable-picker"');
    expect(ui.split("} catch (error) {")[0]).not.toContain('selectedExecutable = "";');
    expect(ui.split("} catch (error) {")[0]).toContain('if (selected && selected !== selectedExecutable)');
    expect(app).toContain("File selected · Publisher checked when opening");
    expect(app).not.toContain('"Minecraft selected"');
    expect(bootstrap).not.toContain("desktop_create_virtual");
  });
  it("reconciles only picker-origin feedback and preserves ambiguous outcomes", () => {
    const picker = app.split("async function selectMinecraft()")[1]?.split("async function openMinecraft02()")[0] ?? "";
    expect(picker).toContain('shouldClearPickerIssue(issues["02"], selected)');
    expect(picker).toContain('issues = { ...issues, "02": undefined };');
    expect(picker).toContain('requestRevision, "executable-picker");');
    expect(picker).not.toContain('clearFeedback("02")');
    const report = app.split("function reportIssue(")[1]?.split("async function checkStatus()")[0] ?? "";
    expect(report).toContain('origin?: UiIssue["origin"]');
    expect(report).toContain('...(origin ? { origin } : {})');
  });
  it("retains canceled file-picker errors until actual reinspection", () => {
    const picker = app.split("async function selectMinecraft()")[1]?.split("async function openMinecraft02()")[0] ?? "";
    expect(picker).not.toContain("issue = null;");
    expect(picker).toContain("if (selected && selected !== selectedExecutable)");
    expect(picker).toContain('if (selected && page === "instances"');
  });

  it("keeps keyboard focus inside the mounted page after Settings navigation", () => {
    expect(app).toContain("function showSettings()");
    expect(app).toContain("function showInstances()");
    expect(app).toContain('page === "settings" && settingsBackButton?.isConnected');
    expect(app).toContain('page === "instances" && checkStatusButton?.isConnected && !checkStatusButton.disabled');
    expect(app).toContain('on:click={showSettings}>Settings</button>');
    expect(app).toContain('on:click={showInstances}>Back to Virtuals</button>');
    expect(app).toContain('openClient02DetailsFromSettings("setup")');
    expect(app).toContain('openClient02DetailsFromSettings("move")');
    expect(app).toContain('page === "instances" && expanded === "02" && detailMode === mode && client02DetailPanel?.isConnected');
    expect(app).toContain("client02DetailPanel.focus()");
    expect(app).toContain('tabindex="-1" bind:this={client02DetailPanel}');
  });
  it("does not steal focus from Settings or newly expanded details after an async close", () => {
    expect(app).toContain('if (page !== "instances" || expanded !== null) return;');
    expect(app).toContain("checkStatusButton?.isConnected && !checkStatusButton.disabled");
    expect(app).toContain("disclosureTrigger = null;");
    expect(app).not.toMatch(/on:click=\{\(\) => \(page =/);
  });

  it("uses the selected Client 02 detail mode even if an earlier issue persists", () => {
    // Issue status may remain available through View issue, but selecting
    // Set up in Settings must not be trapped in the previous issue panel.
    expect(app).toContain('{#if detailMode === "issue"}');
    expect(app).not.toContain('{#if detailMode === "issue" || issue?.instance === "02"}');
    expect(app).toContain('{:else if detailMode === "setup"}');
    expect(app).toContain('{:else if detailMode === "move" && moveEligible}');
    expect(app).toContain('if (action === "issue") { expandDetails(id, "issue", origin); return; }');
    expect(app).toContain('on:click={() => openClient02DetailsFromSettings("setup")}');
  });
  it("never toggles an already visible issue closed from View issue", () => {
    expect(app).toContain('if (action === "issue") { expandDetails(id, "issue", origin); return; }');
    expect(app).toContain('if (action === "setup") toggleDetails(id, "setup", origin);');
    expect(app).toContain('id="cm-details-01"');
    expect(app).toContain('id="cm-details-02"');
    expect(app).toContain('aria-controls="cm-details-02"');
  });
});
