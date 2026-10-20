<script lang="ts">
  import OwnedManagerPanel from "./OwnedManagerPanel.svelte";
  import {ownedProvider,type OwnedSnapshot} from "./app/bridge/ownedProviderApi.js";
  import { tick } from "svelte";
  import {
    nativeFeasibility,
    isSandboxieBoxPrepared,
    canRequestClient02,
    type InstanceSlotDescriptor,
    type InstanceOverview,
    type InstanceFocusObservation,
    type ForegroundEventObserverStatus,
    type IsolatedClientObservation,
    type OwnedCandidateInspection,
    type OwnedHostInspection,
    type VirtualBoxInventory,
  } from "./app/bridge/nativeFeasibilityApi.js";
  import {
    client02Presentation,
    focusCandidate,
    isResourceOperationBusy,
    ownedVirtualCreationReason,
    rowPresentation,
    preserveClient02LaunchFeedback,
    safeErrorDetails,
    setupInstruction,
    shouldClearPickerIssue,
    type InstanceId,
    type UiIssue,
    type UiNotice,
  } from "./app/presentation/compactManager.js";

  type Page = "instances" | "settings";
  type DetailMode = "setup" | "issue" | "info" | "move";
  type MoveSide = "LEFT" | "RIGHT";

  let page: Page = "instances";
  let ownedSession="";
  let ownedSnapshot:OwnedSnapshot|null=null;
  let ownedManagerBusy=false;
  let ownedSelectionError="";
  async function selectOwnedTestSession(){
    if(resourceOperationBusy)return;
    busy="owned-select";ownedSelectionError="";
    try{
      const session=await ownedProvider.selectSession();
      if(session){const snapshot=await ownedProvider.execute(session,"02","Inspect");ownedSession=session;ownedSnapshot=snapshot;}
    }catch(e){ownedSelectionError=String(e);}
    finally{busy="";}
  }
  let expanded: InstanceId | null = null;
  let detailMode: DetailMode = "info";
  let busy = "";
  let observation: IsolatedClientObservation | undefined;
  let lastCheck: Date | null = null;
  let lastCheckIsCurrent = false;
  // Single cheap read-only Rust metadata request when this UI is created.
  // No timer, process inspection, game launch or background helper is started.
  let instanceSlots: InstanceSlotDescriptor[] = [];
  let instanceCatalogError = "";
  void nativeFeasibility.listInstanceSlots().then(
    slots => { instanceSlots = slots; },
    () => { instanceCatalogError = "Four-slot catalog unavailable; no additional client was enabled."; },
  );
  let selectedExecutable = "";
  let licenseConfirmed = false;
  // Development-only candidate inspection is independent of active Client 02.
  let ownedCandidate: OwnedCandidateInspection | null = null;
  let ownedInspectionError = "";
  let ownedInspectionBusy = false;
  let ownedHostInspection: OwnedHostInspection | null = null;
  let ownedHostBusy = false;
  let ownedHostError = "";
  let existingVirtuals: VirtualBoxInventory | null = null;
  let existingVirtualsError = "";
  let instanceOverview: InstanceOverview | null = null;
  let instanceOverviewError = "";
  let instanceFocus: InstanceFocusObservation | null = null;
  let instanceFocusError = "";
  let focusEvents: ForegroundEventObserverStatus | null = null;
  let focusEventsError = "";
  // A failed Start/Stop does not establish the actual Windows hook state.
  // A deliberate successful Read must confirm state before another Start.
  let focusEventsNeedsRecheck = false;
  // Temporary UI feedback, isolated by supported instance identity.
  let issues: Partial<Record<InstanceId, UiIssue>> = {};
  let notices: Partial<Record<InstanceId, UiNotice>> = {};
  let moveSide: MoveSide = "LEFT";
  let checkStatusButton: HTMLButtonElement | undefined;
  let host01OpenButton: HTMLButtonElement | undefined;
  let host01DetailsButton: HTMLButtonElement | undefined;
  let settingsBackButton: HTMLButtonElement | undefined;
  let client02DetailPanel: HTMLDivElement | undefined;
  let disclosureTrigger: HTMLButtonElement | null = null;
  // UI-only revision: IPC completion must not undo later user navigation.
  let navigationRevision = 0;

  // Balanced policy is the current actual behavior, not a game FPS/RAM limit.
  // All expensive helpers are explicitly requested and serialized by the UI.
  $: resourceOperationBusy = isResourceOperationBusy(busy, ownedHostBusy, ownedInspectionBusy) || ownedManagerBusy;
  $: base02 = client02Presentation(observation, selectedExecutable, licenseConfirmed);
  $: issue01 = issues["01"] ?? null;
  $: issue02 = issues["02"] ?? null;
  $: notice01 = notices["01"] ?? null;
  $: notice02 = notices["02"] ?? null;
  $: row01 = rowPresentation("01", base02, busy, issue01, notice01);
  $: row02 = rowPresentation("02", base02, busy, issue02, notice02);
  $: moveEligible = Boolean(observation && focusCandidate(observation) && base02.canMove);

  function closeDetails() {
    navigationRevision += 1;
    const origin = disclosureTrigger;
    expanded = null;
    disclosureTrigger = null;
    // Async operations may resolve after the user navigates to Settings.
    // Never steal focus from Settings or a different newly opened panel.
    void tick().then(() => {
      if (page !== "instances" || expanded !== null) return;
      if (origin?.isConnected && !origin.disabled) origin.focus();
      else if (checkStatusButton?.isConnected && !checkStatusButton.disabled) checkStatusButton.focus();
    });
  }

  function showSettings() {
    navigationRevision += 1;
    page = "settings";
    void tick().then(() => {
      if (page === "settings" && settingsBackButton?.isConnected) settingsBackButton.focus();
    });
  }

  function showInstances() {
    navigationRevision += 1;
    page = "instances";
    void tick().then(() => {
      if (page === "instances" && checkStatusButton?.isConnected && !checkStatusButton.disabled) checkStatusButton.focus();
    });
  }

  function openClient02DetailsFromSettings(mode: "setup" | "move") {
    navigationRevision += 1;
    page = "instances";
    expanded = "02";
    detailMode = mode;
    disclosureTrigger = null;
    // Settings' action is removed when the page changes. Focus the region
    // deliberately so Tab continues into the actual setup/move controls.
    void tick().then(() => {
      if (page === "instances" && expanded === "02" && detailMode === mode && client02DetailPanel?.isConnected) {
        client02DetailPanel.focus();
      }
    });
  }

  function showClient02MoveDetails() {
    if (page !== "instances" || expanded !== "02" || !moveEligible) return;
    // Inline mode changes are navigation too: late IPC results must not
    // override a Move panel selected after their request began.
    navigationRevision += 1;
    detailMode = "move";
  }

  function expandDetails(instance: InstanceId, mode: DetailMode, origin: HTMLButtonElement) {
    navigationRevision += 1;
    disclosureTrigger = origin;
    expanded = instance;
    detailMode = mode;
  }

  function toggleDetails(instance: InstanceId, mode: DetailMode, origin: HTMLButtonElement) {
    if (expanded === instance && detailMode === mode) closeDetails();
    else expandDetails(instance, mode, origin);
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape" || event.defaultPrevented || page !== "instances" || expanded === null) return;
    event.preventDefault();
    closeDetails();
  }

  function invalidateCurrentInspection() {
    instanceFocus = null;
    instanceFocusError = "";
    observation = undefined;
    lastCheckIsCurrent = false;
  }

  function clearFeedback(instance: InstanceId) {
    issues = { ...issues, [instance]: undefined };
    notices = { ...notices, [instance]: undefined };
  }

  // A native Open request has no authoritative Host 01 process check here.
  // A manual acknowledgement clears feedback, never launches or stops Minecraft.
  function dismissHost01Feedback() {
    if (resourceOperationBusy || page !== "instances" || expanded !== "01" ||
        (!issues["01"] && !notices["01"])) return;
    clearFeedback("01");
    navigationRevision += 1;
    expanded = null;
    disclosureTrigger = null;
    void tick().then(() => {
      if (page === "instances" && expanded === null &&
          host01OpenButton?.isConnected && !host01OpenButton.disabled) {
        host01OpenButton.focus();
      }
    });
  }

  // Manual acknowledgement only: never launches, stops or inspects Minecraft.
  function dismissClient02LaunchFeedback() {
    if (resourceOperationBusy || page !== "instances" || expanded !== "02" ||
        notices["02"]?.text !== "Open requested" || issues["02"] ||
        !lastCheckIsCurrent || !canRequestClient02(observation)) return;
    notices = { ...notices, "02": undefined };
    closeDetails();
  }

  function reportIssue(instance: InstanceId, title: string, message: string, error: unknown, requestRevision: number, origin?: UiIssue["origin"]) {
    issues = { ...issues, [instance]: {
      instance, title, message,
      details: error === undefined ? undefined : safeErrorDetails(error),
      ...(origin ? { origin } : {}),
    } };
    notices = { ...notices, [instance]: undefined };
    // Do not replace a detail panel owned by the other client.
    if (page === "instances" && requestRevision === navigationRevision &&
        (expanded === null || expanded === instance)) {
      disclosureTrigger = null;
      expanded = instance;
      detailMode = "issue";
    }
  }

  async function checkStatus() {
    if (isResourceOperationBusy(busy, ownedHostBusy, ownedInspectionBusy)) return;
    const requestRevision = navigationRevision;
    const pendingLaunchNotice = notices["02"]?.text === "Open requested" ? notices["02"] : undefined;
    busy = "check";
    clearFeedback("02");
    // A fast check may run before the newly requested game appears as a PID.
    // Retain the launch guard across an empty snapshot and probe errors.
    if (pendingLaunchNotice) notices = { ...notices, "02": pendingLaunchNotice };
    invalidateCurrentInspection();
    try {
      const result = await nativeFeasibility.inspectIsolatedClient02();
      observation = result;
      lastCheck = new Date();
      lastCheckIsCurrent = true;
      if (pendingLaunchNotice && !preserveClient02LaunchFeedback(pendingLaunchNotice, result)) {
        notices = { ...notices, "02": undefined };
      }
      // Client 02 inspection cannot dismiss Client 01's expanded issue.
      if (page === "instances" && requestRevision === navigationRevision &&
          expanded === "02" && detailMode === "issue") closeDetails();
    } catch (error) {
      reportIssue("02", "Couldn't check status",
        "The app couldn't confirm the instance status. Review Sandboxie Plus, then check again.", error, requestRevision);
      // reportIssue shows the error; keep the separate launch safeguard.
      if (pendingLaunchNotice) notices = { ...notices, "02": pendingLaunchNotice };
    } finally {
      busy = "";
    }
  }

  async function openMinecraft01() {
    if (isResourceOperationBusy(busy, ownedHostBusy, ownedInspectionBusy)) return;
    const requestRevision = navigationRevision;
    busy = "open-01";
    clearFeedback("01");
    invalidateCurrentInspection();
    try {
      await nativeFeasibility.openOfficialMinecraft();
      notices = { ...notices, "01": { instance: "01", text: "Open requested" } };
    } catch (error) {
      reportIssue("01", "Couldn't confirm if Minecraft opened",
        "Check for an existing Minecraft window before trying again.", error, requestRevision);
    } finally {
      busy = "";
      // The native Open control unmounts during the request. Move focus to
      // its resulting review control if the user has not navigated away.
      void tick().then(() => {
        if (page === "instances" && requestRevision === navigationRevision &&
            host01DetailsButton?.isConnected && !host01DetailsButton.disabled) {
          host01DetailsButton.focus();
        }
      });
    }
  }

  async function selectMinecraft() {
    if (isResourceOperationBusy(busy, ownedHostBusy, ownedInspectionBusy)) return;
    const requestRevision = navigationRevision;
    busy = "select";
    try {
      const selected = await nativeFeasibility.selectEducationExe();
      if (selected && selected !== selectedExecutable) {
        selectedExecutable = selected;
        licenseConfirmed = false;
      }
      // Only a prior picker failure is resolved by choosing a file; ambiguous
      // launch/inspection errors remain until a relevant recheck.
      if (shouldClearPickerIssue(issues["02"], selected)) {
        issues = { ...issues, "02": undefined };
      }
      // Cancelled picker is not evidence that a previous issue was resolved.
      // A completed picker must not redirect a different newly opened panel.
      if (selected && page === "instances" && requestRevision === navigationRevision && expanded === "02") {
        detailMode = "setup";
      }
    } catch (error) {
      // An invalid picked file must not retain an earlier launch selection
      // or Sandboxie licence consent. Cancelling the dialog keeps both.
      selectedExecutable = "";
      licenseConfirmed = false;
      reportIssue("02", "Couldn't select Minecraft",
        "That file cannot be used as Minecraft Education. Select an installed Minecraft Education executable; its Microsoft signature is checked when opening.", error, requestRevision, "executable-picker");
    } finally {
      busy = "";
    }
  }

  async function openMinecraft02() {
    if (isResourceOperationBusy(busy, ownedHostBusy, ownedInspectionBusy) || !observation || !selectedExecutable || !licenseConfirmed || base02.action !== "open") return;
    const requestRevision = navigationRevision;
    const requestedPanel = expanded;
    const requestedMode = detailMode;
    busy = "open-02";
    clearFeedback("02");
    invalidateCurrentInspection();
    try {
      await nativeFeasibility.requestSandboxClient02(selectedExecutable, licenseConfirmed);
      notices = { ...notices, "02": { instance: "02", text: "Open requested" } };
      if (page === "instances" && requestRevision === navigationRevision &&
          requestedPanel === "02" && expanded === requestedPanel && detailMode === requestedMode) {
        closeDetails();
      }
    } catch (error) {
      reportIssue("02", "Couldn't confirm if Minecraft opened",
        "The launch request may have started Minecraft despite an error. Check status and Windows before retrying.", error, requestRevision);
      // A failed or timed-out launcher can already have spawned Minecraft.
      // Keep an independent pending-Open safeguard for later status checks.
      // The original issue remains visible until explicit reinspection.
      notices = { ...notices, "02": { instance: "02", text: "Open requested" } };
    } finally {
      busy = "";
    }
  }

  async function switchMinecraft02() {
    const target = focusCandidate(observation);
    if (isResourceOperationBusy(busy, ownedHostBusy, ownedInspectionBusy) || !target || base02.action !== "switch") return;
    const requestRevision = navigationRevision;
    busy = "switch";
    clearFeedback("02");
    try {
      await nativeFeasibility.focusObservedClient02(target);
      // A focus request does not invalidate the observed target or disable further switching.
      // The backend revalidates process and window identity on every attempt.
      // Only Client 02 feedback was reset when Focus started.
    } catch (error) {
      invalidateCurrentInspection();
      reportIssue("02", "Couldn't switch to Minecraft",
        "The window may have changed. Check status or select it with Alt+Tab.", error, requestRevision);
    } finally {
      busy = "";
    }
  }

  async function moveMinecraft02() {
    const target = focusCandidate(observation);
    if (isResourceOperationBusy(busy, ownedHostBusy, ownedInspectionBusy) || !target || !base02.canMove) return;
    const requestRevision = navigationRevision;
    const requestedPanel = expanded;
    const requestedMode = detailMode;
    busy = "move";
    clearFeedback("02");
    try {
      await nativeFeasibility.placeObservedClient02(target, moveSide);
      notices = { ...notices, "02": { instance: "02", text: "Move requested" } };
      if (page === "instances" && requestRevision === navigationRevision &&
          requestedPanel === "02" && expanded === requestedPanel && detailMode === requestedMode) {
        closeDetails();
      }
    } catch (error) {
      invalidateCurrentInspection();
      reportIssue("02", "Couldn't move window",
        "Check status before trying again. If Minecraft is maximized or minimized, restore it to a normal window.", error, requestRevision);
    } finally {
      busy = "";
    }
  }

  function choosePrimary(id: InstanceId, origin: HTMLButtonElement) {
    const action = id === "01" ? row01.action : row02.action;
    if (isResourceOperationBusy(busy, ownedHostBusy, ownedInspectionBusy)) return;
    if (id === "01" && action === "open") { void openMinecraft01(); return; }
    if (id === "02" && action === "check") { void checkStatus(); return; }
    if (id === "02" && action === "open") { void openMinecraft02(); return; }
    if (id === "02" && action === "switch") { void switchMinecraft02(); return; }
    // "View issue" must never close the issue when it is already visible.
    if (action === "issue") { expandDetails(id, "issue", origin); return; }
    if (action === "setup") toggleDetails(id, "setup", origin);
  }


  // Opt-in, process-local Windows event counter. No automatic polling;
  // the user explicitly starts, refreshes and stops the one observer.
  async function changeFocusObserver(enable: boolean) {
    if (isResourceOperationBusy(busy, ownedHostBusy, ownedInspectionBusy) ||
        (focusEventsNeedsRecheck && enable)) return;
    busy = enable ? "focus-events-start" : "focus-events-stop";
    focusEventsError = "";
    try {
      focusEvents = enable
        ? await nativeFeasibility.startFocusObserver()
        : await nativeFeasibility.stopFocusObserver();
      focusEventsNeedsRecheck = false;
    } catch {
      focusEvents = null;
      focusEventsNeedsRecheck = true;
      focusEventsError = "Observer state unverified. Read event count or retry Stop/cleanup; do not start another hook.";
    } finally {
      busy = "";
    }
  }

  async function readFocusObserver() {
    if (isResourceOperationBusy(busy, ownedHostBusy, ownedInspectionBusy)) return;
    busy = "focus-events-read";
    focusEventsError = "";
    try {
      focusEvents = await nativeFeasibility.readFocusObserver();
      focusEventsNeedsRecheck = false;
    } catch {
      focusEvents = null;
      focusEventsNeedsRecheck = true;
      focusEventsError = "Unable to verify foreground observer state. Do not start another hook.";
    } finally {
      busy = "";
    }
  }

  async function inspectInstanceFocus() {
    if (isResourceOperationBusy(busy, ownedHostBusy, ownedInspectionBusy)) return;
    const target = focusCandidate(observation);
    instanceFocus = null;
    instanceFocusError = "";
    if (!target) {
      instanceFocusError = "Client 02 has no unique verified window candidate. Use Check status · 02 first.";
      return;
    }
    busy = "inspect-focus";
    try {
      // Backend independently revalidates box/PID/FILETIME/HWND/signature.
      instanceFocus = await nativeFeasibility.inspectInstanceFocus(target);
    } catch {
      instanceFocusError = "Four-slot focus inspection was inconclusive. No FPS policy was changed.";
    } finally {
      busy = "";
    }
  }

  async function inspectAllSlots() {
    if (isResourceOperationBusy(busy, ownedHostBusy, ownedInspectionBusy)) return;
    busy = "inspect-all";
    instanceOverview = null;
    instanceOverviewError = "";
    try {
      instanceOverview = await nativeFeasibility.inspectInstanceOverview();
    } catch {
      // An incomplete collector response is UNKNOWN, never an empty game.
      instanceOverviewError = "Four-slot inspection was inconclusive. No client was started or changed.";
    } finally {
      busy = "";
    }
  }

  async function discoverExistingVirtuals() {
    if (isResourceOperationBusy(busy, ownedHostBusy, ownedInspectionBusy)) return;
    busy = "discover-virtuals";
    existingVirtuals = null;
    existingVirtualsError = "";
    try {
      existingVirtuals = await nativeFeasibility.discoverExistingVirtuals();
    } catch {
      // No invented empty inventory: a failed query is unknown, not zero boxes.
      existingVirtualsError = "Could not inspect Sandboxie configuration names. No virtuals were changed.";
    } finally {
      busy = "";
    }
  }

  async function inspectOwnedServiceNames() {
    if (isResourceOperationBusy(busy, ownedHostBusy, ownedInspectionBusy)) return;
    ownedHostBusy = true;
    ownedHostInspection = null;
    ownedHostError = "";
    try {
      ownedHostInspection = await nativeFeasibility.inspectOwnedHost();
    } catch {
      ownedHostError = "Windows service name inspection was inconclusive; engine activation remains blocked.";
    } finally {
      ownedHostBusy = false;
    }
  }

  async function inspectStagedOwnedEngine() {
    if (isResourceOperationBusy(busy, ownedHostBusy, ownedInspectionBusy)) return;
    ownedInspectionBusy = true;
    ownedCandidate = null;
    ownedInspectionError = "";
    try {
      const directory = await nativeFeasibility.selectOwnedCandidateDirectory();
      if (!directory) return;
      ownedCandidate = await nativeFeasibility.inspectOwnedCandidate(directory);
    } catch {
      // Keep sensitive local filesystem paths and error details out of UI.
      ownedInspectionError = "Candidate integrity could not be verified. No engine was activated.";
    } finally {
      ownedInspectionBusy = false;
    }
  }

  function primaryLabel(action: string | null) {
    if (action === "check") return "Check status";
    if (action === "open") return "Open";
    if (action === "switch") return "Switch to";
    if (action === "setup") return "Set up";
    if (action === "issue") return "View issue";
    return "";
  }
</script>

<svelte:window on:keydown={handleWindowKeydown} />
<div class="cm-shell">
  <main class="cm-content">
    {#if page === "instances"}
      <header class="cm-toolbar">
        <h1>Virtual Manager</h1>
        <div class="cm-toolbar-actions">
          {#if !ownedSession}
          <button class="cm-button cm-secondary" type="button" title="Inspects Virtual 02 only; Host 01 is not checked" bind:this={checkStatusButton} disabled={resourceOperationBusy} on:click={checkStatus}>
            <span class="cm-refresh-icon" aria-hidden="true">↻</span>
            {busy === "check" ? "Checking…" : "Check status · 02"}
          </button>
          {/if}
          <button class="cm-button cm-quiet" type="button" on:click={showSettings}>Settings</button>
        </div>
      </header>

      {#if ownedSession && ownedSnapshot}
        <OwnedManagerPanel sessionDirectory={ownedSession} bind:initialSnapshot={ownedSnapshot} bind:operationBusy={ownedManagerBusy} externalBusy={!!busy || ownedHostBusy || ownedInspectionBusy}/>
      {:else}
      <section class="cm-virtual-overview" aria-label="Create virtual availability">
        <div>
          <h2>Minecraft sessions</h2>
          <p>Supported today: Host 01 can request a Windows launch. Virtual 02 can be checked and opened using the existing Sandboxie Plus pilot after setup. Virtuals 03 and 04 are not available.</p>
        </div>
        <div class="cm-virtual-actions">
          <button type="button" class="cm-button cm-secondary" disabled
            aria-describedby="cm-create-reason"
            title="Internal isolation engine is not yet installed and verified">Create virtual</button>
        </div>
      </section>
      <section class="cm-create-readiness" aria-label="Virtual creation availability">
        <p id="cm-create-reason" aria-live="polite">Create Virtual unavailable. The private MIVUBI engine has not completed Windows validation.</p>
        <p>Advanced checks are available in Settings → Diagnostics.</p>
      </section>
      <section class="cm-list" aria-label="Virtual sessions">
        <div class="cm-instance">
          <div class="cm-row">
            <span class="cm-monitor" aria-hidden="true"></span>
            <div class="cm-row-copy">
              <h2>Host · Minecraft 01</h2>
              <p class={row01.tone === "warning" ? "cm-warning" : ""} aria-live="polite">{row01.status}</p>
            </div>
            <div class="cm-row-actions">
              {#if row01.action}
                <button class="cm-button cm-primary" type="button" bind:this={host01OpenButton} disabled={resourceOperationBusy}
                  on:click={(event) => choosePrimary("01", event.currentTarget)} aria-expanded={row01.action === "issue" ? expanded === "01" : undefined} aria-controls={row01.action === "issue" ? "cm-details-01" : undefined}>{primaryLabel(row01.action)}</button>
              {/if}
              {#if issue01 || notice01}
                <button class="cm-link" type="button" bind:this={host01DetailsButton}
                  aria-label={issue01 ? "Minecraft 01 error details" : "Minecraft 01 launch request details"}
                  aria-expanded={expanded === "01"} aria-controls="cm-details-01"
                  on:click={(event) => toggleDetails("01", issue01 ? "issue" : "info", event.currentTarget)}>Details</button>
              {/if}
            </div>
          </div>
          {#if expanded === "01"}
            <div id="cm-details-01" class="cm-detail" role="region" aria-label="Minecraft 01 details">
              {#if issue01}
                <h3>{issue01.title}</h3>
                <p>{issue01.message}</p>
                {#if issue01.details}<details><summary>Advanced details</summary><p class="cm-technical">{issue01.details}</p></details>{/if}
              {:else if notice01}
                <h3>Open requested</h3>
                <p>Windows accepted a request to open Minecraft 01. This app cannot confirm whether the game is still open.</p>
              {:else}
                <h3>About Minecraft 01</h3>
                <p>Opens the official Minecraft Education app on this PC. The manager does not check account sign-in or gameplay.</p>
              {/if}
              {#if issue01 || notice01}
                <p class="cm-assist">First check Windows for an existing Minecraft Education window. Clearing this message does not stop Minecraft or verify it is closed.</p>
                <button type="button" class="cm-button cm-secondary" disabled={resourceOperationBusy}
                  on:click={dismissHost01Feedback}>I checked Windows · Clear message</button>
              {/if}
              <button class="cm-link" type="button" on:click={closeDetails}>Close details</button>
            </div>
          {/if}
        </div>

        <div class="cm-instance">
          <div class="cm-row">
            <span class="cm-monitor" aria-hidden="true"></span>
            <div class="cm-row-copy">
              <h2>Virtual 02 · Minecraft Education</h2>
              <p class={row02.tone === "warning" ? "cm-warning" : ""} aria-live="polite">{row02.status}</p>
            </div>
            <div class="cm-row-actions">
              {#if row02.action}
                <button type="button" class="cm-button cm-primary" disabled={resourceOperationBusy}
                  on:click={(event) => choosePrimary("02", event.currentTarget)} aria-label={row02.action === "check" ? "Check status for Minecraft 02" : undefined} aria-expanded={row02.action === "issue" || row02.action === "setup" ? expanded === "02" : undefined} aria-controls={row02.action === "issue" || row02.action === "setup" ? "cm-details-02" : undefined}>{primaryLabel(row02.action)}</button>
              {/if}
              {#if row02.action === "switch" || expanded === "02" || notice02}
                <button class="cm-link" type="button" aria-label="Minecraft 02 details" aria-expanded={expanded === "02"}
                  aria-controls="cm-details-02" on:click={(event) => toggleDetails("02", row02.action === "switch" ? "move" : row02.action === "issue" ? "issue" : "info", event.currentTarget)}>Details</button>
              {/if}
            </div>
          </div>

          {#if expanded === "02"}
            <div id="cm-details-02" class="cm-detail" role="region" aria-label="Minecraft 02 details" tabindex="-1" bind:this={client02DetailPanel}>
              {#if detailMode === "issue"}
                <h3>{issue02 ? issue02.title : base02.status}</h3>
                <p>{issue02 ? issue02.message : base02.details}</p>
                <p class="cm-assist">Check Virtual 02 again before retrying any action. A failed launch request may have started the game.</p>
                <button type="button" class="cm-button cm-secondary" disabled={resourceOperationBusy}
                  on:click={checkStatus}>Check status · 02</button>
                {#if issue02?.details}
                  <details><summary>Advanced details</summary><p class="cm-technical">{issue02.details}</p></details>
                {:else if observation}
                  <details><summary>Advanced details</summary>
                    <p class="cm-technical">Box: {observation.boxConfigured}; membership: {observation.membershipStatus}; settings: {observation.boxSettingsSource}.</p>
                  </details>
                {/if}
              {:else if notice02?.text === "Open requested"}
                <h3>Open requested</h3>
                <p>Sandboxie accepted a launch request, but this app cannot confirm whether Minecraft started or stopped.</p>
                <button type="button" class="cm-button cm-secondary" disabled={resourceOperationBusy}
                  on:click={checkStatus}>Check status · 02</button>
                {#if lastCheckIsCurrent && canRequestClient02(observation)}
                  <p class="cm-assist">The latest check found no matching game process, but a late launch is still possible. Inspect Windows and Sandboxie before clearing this notice. Clearing does not stop Minecraft or verify it is closed.</p>
                  <button type="button" class="cm-button cm-secondary" disabled={resourceOperationBusy}
                    on:click={dismissClient02LaunchFeedback}>I checked Windows · Clear message</button>
                {:else}
                  <p class="cm-assist">A fresh safely observed empty box and manual Windows review are required before clearing this launch notice. Do not retry an uncertain launch.</p>
                {/if}
              {:else if detailMode === "setup"}
                <h3>{setupInstruction(observation).title}</h3>
                <p>{setupInstruction(observation).text}</p>
                {#if !canRequestClient02(observation)}
                  <p class="cm-assist">Executable selection and license confirmation are available only when the last check found the sandbox prepared and no isolated process candidates. Status can change; every launch is checked again. The check does not modify accounts or sandbox settings.</p>
                  {#if !observation || !isSandboxieBoxPrepared(observation)}
                    <details><summary>Existing Sandboxie Plus setup</summary>
                      <p>The current pilot requires a separate persistent box named MinecraftClient02 in Sandboxie Plus. Create it manually only if it does not already exist. Keep AutoDelete, AutoRemove and credential-bypass/protected-storage settings off. Never copy another Minecraft account profile. MIVUBI does not modify the box here.</p>
                    </details>
                  {/if}
                {:else}
                  <div class="cm-field">
                    <span class="cm-field-label">Installed Minecraft Education</span>
                    <div class="cm-field-row">
                      <span class="cm-file-state">{selectedExecutable ? "File selected · Publisher checked when opening" : "No installation selected"}</span>
                      <button class="cm-button cm-secondary" type="button" disabled={resourceOperationBusy} on:click={selectMinecraft}>Browse…</button>
                    </div>
                  </div>
                  <label class="cm-checkbox">
                    <input type="checkbox" bind:checked={licenseConfirmed} disabled={resourceOperationBusy}/>
                    I confirm Sandboxie Plus is licensed for this use and MinecraftClient02 is a persistent box.
                  </label>
                  <p class="cm-assist">Sign in separately in each Minecraft window. This app never requests Microsoft credentials.</p>
                {/if}
                <button type="button" class="cm-button cm-secondary" disabled={resourceOperationBusy}
                  on:click={checkStatus}>Check status · 02</button>
              {:else if detailMode === "move" && moveEligible}
                <h3>Move window</h3>
                <p>Choose where Minecraft 02 should appear on its current monitor.</p>
                <div class="cm-side-choices" role="group" aria-label="Window position">
                  <button type="button" class:cm-selected={moveSide === "LEFT"} aria-pressed={moveSide === "LEFT"} disabled={resourceOperationBusy} on:click={() => (moveSide = "LEFT")}>Left</button>
                  <button type="button" class:cm-selected={moveSide === "RIGHT"} aria-pressed={moveSide === "RIGHT"} disabled={resourceOperationBusy} on:click={() => (moveSide = "RIGHT")}>Right</button>
                </div>
                <button class="cm-button cm-primary" type="button" disabled={resourceOperationBusy} on:click={moveMinecraft02}>Apply</button>
                <p class="cm-assist">Only Minecraft 02 is moved. Restore a maximized or minimized window before applying.</p>
              {:else}
                <h3>About Minecraft 02</h3>
                <p>{base02.details}</p>
                <p class="cm-assist">Runtime: existing Sandboxie Plus pilot. MIVUBI-owned engine is not active.</p>
                {#if moveEligible}<button class="cm-button cm-secondary" type="button" on:click={showClient02MoveDetails}>Move window</button>{/if}
              {/if}
              <div class="cm-detail-footer">
                <button class="cm-link" type="button" on:click={closeDetails}>Close details</button>
              </div>
            </div>
          {/if}
        </div>
      </section>
      <section class="cm-future-virtuals" aria-label="Planned virtual clients">
        <h2>Planned virtuals</h2>
        <p>Clients 03 and 04 cannot be created or launched yet. A built, signed and verified MIVUBI isolation engine is required.</p>
        {#if instanceCatalogError}
          <p role="status">{instanceCatalogError}</p>
        {:else if instanceSlots.length === 0}
          <p role="status">Reading the four-slot catalog…</p>
        {/if}
        {#each instanceSlots.filter(slot => slot.id === "03" || slot.id === "04") as slot (slot.id)}
          <div class="cm-instance cm-planned-instance">
            <div class="cm-row">
              <span class="cm-monitor" aria-hidden="true"></span>
              <div class="cm-row-copy">
                <h3>Virtual {slot.id} · Minecraft Education</h3>
                <p>{slot.availability === "NOT_AVAILABLE" ? "Unavailable in MIVUBI · No managed session" : "State unknown · Do not launch"}</p>
              </div>
            </div>
          </div>
        {/each}
      </section>
      <footer class="cm-footer">
        {#if lastCheck}
          {lastCheckIsCurrent ? "Last checked:" : "Previous check:"}
          {lastCheck.toLocaleTimeString("en-US", { hour: "numeric", minute: "2-digit" })}
          {#if !lastCheckIsCurrent} · Status may have changed{/if}
        {:else}
          Status not checked this session
        {/if}
      </footer>
      {/if}
    {:else}
      <header class="cm-toolbar">
        <h1>Settings</h1>
        <button class="cm-button cm-secondary" type="button" bind:this={settingsBackButton} on:click={showInstances}>Back to Virtuals</button>
      </header>
      <section class="cm-settings" aria-label="Application settings">
        <div class="cm-setting-group">
          <h2>Engine</h2>
          <p>{ownedSession?"Local engine test selected · Four-client manager operations use the verified owned provider.":"Existing official pilot selected · Owned test engine has not been selected in this app session."}</p>
          <button type="button" class="cm-button cm-secondary" disabled={resourceOperationBusy} on:click={selectOwnedTestSession}>Select local engine test</button>
          <p>Select the activated session directory containing session.json and payload. This does not install or activate a driver.</p>
          {#if ownedSelectionError}<p role="alert" class="cm-warning">{ownedSelectionError}</p>{/if}
        </div>
        <div class="cm-setting-group">
          <h2>Minecraft setup</h2>
          <p>Choose the official Minecraft Education Desktop installation for Minecraft 02.</p>
          <button type="button" class="cm-button cm-secondary" on:click={() => openClient02DetailsFromSettings("setup")}>Set up Minecraft 02</button>
        </div>
        <div class="cm-setting-group">
          <h2>Performance</h2>
          <p><strong>Balanced · Active</strong> — resource-intensive manager operations are handled one at a time. Status checks and engine inspection run only when requested.</p>
          <p>This policy avoids overlapping PowerShell probes and file hashing. It does not restrict Minecraft CPU, RAM or GPU usage or change game graphics settings.</p>
          <details>
            <summary>Eco and Performance modes</summary>
            <p>Not available yet. These modes require Windows and Minecraft Education frame-time, GPU and memory testing before any game resource policy can be enabled.</p>
          </details>
        </div>
        <div class="cm-setting-group">
          <h2>Window preferences</h2>
          <p>Move a detected Minecraft 02 window using its instance details. Other layouts are not available yet.</p>
          {#if moveEligible}<button type="button" class="cm-button cm-secondary" on:click={() => openClient02DetailsFromSettings("move")}>Move window</button>{/if}
        </div>
        <div class="cm-setting-group">
          <h2>Help</h2>
          <p>Use Check status to inspect your setup. Minecraft sign-in remains inside the official game.</p>
          <details>
            <summary>Advanced details</summary>
            <p>Host 01: Windows installation. Virtual 02: existing persistent Sandboxie Plus box MinecraftClient02. New virtual creation is not yet active.</p>
            <p>Process observations do not confirm accounts or playable worlds. The manager never copies credentials or game worlds.</p>
          </details>
        </div>
        <div class="cm-setting-group">
          <details class="cm-diagnostics">
            <summary>Diagnostics</summary>
            <div class="cm-diagnostics-body">
              <p class="cm-assist">Advanced, read-only checks for the current Sandboxie Plus pilot and the not-yet-installed MIVUBI engine. These checks do not create, install, migrate or sign in to any virtual.</p>
              <h3>Four-slot process discovery</h3>
              <p>Inspect only on request. Host 01 results are name matches from the Windows session, not verified native ownership; Client 02 results reflect only process candidates in its official Sandboxie box. An observed zero is not proof that Minecraft stopped.</p>
              <button type="button" class="cm-button cm-secondary"
                disabled={resourceOperationBusy} on:click={inspectAllSlots}>
                {busy === "inspect-all" ? "Inspecting slots…" : "Inspect all four slots"}
              </button>
              {#if instanceOverview}
                <section aria-label="Four-slot candidate evidence" class="cm-existing-virtuals">
                  <h3>Read-only instance overview</h3>
                  <ul>
                    {#each instanceOverview.slots as slot (slot.id)}
                      <li>Client {slot.id} ·
                        {slot.evidence === "NAME_MATCH_UNATTRIBUTED" ? "Windows name matches, not attributed to Client 01"
                          : slot.evidence === "BOX_PROCESS_CANDIDATES_ONLY" ? "Sandboxie box process candidates only"
                          : slot.evidence === "PROVIDER_NOT_AVAILABLE" ? "MIVUBI engine not available"
                          : "Observation unknown"}
                        {#if slot.candidateCount !== null} · {slot.candidateCount} candidate(s){/if}
                      </li>
                    {/each}
                  </ul>
                  <p>These observations never confirm Minecraft sign-in, gameplay, or permission to launch another instance.</p>
                </section>
              {/if}
              {#if instanceOverviewError}
                <p class="cm-warning" role="alert">{instanceOverviewError}</p>
              {/if}
              <h3>Four-slot focus evidence</h3>
              <p>Read-only snapshot after fresh verification of Client 02. Because clicking this control usually focuses MIVUBI itself, the result may be UNKNOWN. Client 01 cannot yet be attributed; Client 03/04 have no verified provider. No FPS limiter is enabled.</p>
              <button type="button" class="cm-button cm-secondary"
                disabled={resourceOperationBusy || !focusCandidate(observation)} on:click={inspectInstanceFocus}>
                {busy === "inspect-focus" ? "Reading focus…" : "Inspect four-slot focus"}
              </button>
              {#if instanceFocus}
                <section class="cm-existing-virtuals" aria-label="Four-slot focus evidence" aria-live="polite">
                  <p>Mode: observation only. Focus on the MIVUBI manager or an Alt+Tab transition reports UNKNOWN, not background-game eligibility.</p>
                  <ul>
                    {#each instanceFocus.slots as slot (slot.id)}
                      <li>Client {slot.id} · {slot.evidence === "FOREGROUND" ? "Verified foreground at check"
                        : slot.evidence === "NOT_FOREGROUND" ? "Verified window not foreground"
                        : slot.evidence === "UNATTRIBUTED" ? "Not attributed to a native client"
                        : slot.evidence === "PROVIDER_NOT_AVAILABLE" ? "Provider not available"
                        : "Unknown foreground state"}</li>
                    {/each}
                  </ul>
                  <p>No FPS changes were requested. This does not confirm sign-in or playability.</p>
                </section>
              {/if}
              {#if instanceFocusError}
                <p class="cm-warning" role="alert">{instanceFocusError}</p>
              {/if}
              <h3>Optional Windows focus event observer</h3>
              <p>Observe the number of foreground changes across Windows. This is opt-in and uses one low-overhead event hook, not constant polling. Events are counted only; no window title, account, or PID is retained. It does not identify Minecraft clients or modify FPS.</p>
              <div class="cm-field-row">
                {#if focusEventsNeedsRecheck}
                  <button type="button" class="cm-button cm-secondary"
                    disabled={resourceOperationBusy} on:click={() => changeFocusObserver(false)}>
                    {busy === "focus-events-stop" ? "Cleaning up…" : "Retry Stop/cleanup"}
                  </button>
                {:else if focusEvents?.enabled}
                  <button type="button" class="cm-button cm-secondary"
                    disabled={resourceOperationBusy} on:click={() => changeFocusObserver(false)}>
                    {busy === "focus-events-stop" ? "Stopping…" : "Stop observer"}
                  </button>
                {:else}
                  <button type="button" class="cm-button cm-secondary"
                    disabled={resourceOperationBusy || focusEventsNeedsRecheck} on:click={() => changeFocusObserver(true)}>
                    {busy === "focus-events-start" ? "Starting…" : "Start observer"}
                  </button>
                {/if}
                <button type="button" class="cm-button cm-secondary"
                  disabled={resourceOperationBusy} on:click={readFocusObserver}>
                  {busy === "focus-events-read" ? "Reading…" : "Read event count"}
                </button>
              </div>
              {#if focusEvents}
                <p aria-live="polite">Observer {focusEvents.enabled ? "running" : "stopped"} · {focusEvents.eventCount} foreground-change event(s). An event count is not evidence of game FPS or process ownership.</p>
              {/if}
              {#if focusEventsNeedsRecheck}
                <p class="cm-warning" aria-live="polite">Observer state unknown. Start stays disabled until Read confirms a safe state or Stop/cleanup completes.</p>
              {/if}
              {#if focusEventsError}<p class="cm-warning" role="alert">{focusEventsError}</p>{/if}
              <h3>Sandboxie configuration</h3>
              <p>Inspect configuration section names from the installed pilot. Results are candidates, not verified running Minecraft sessions.</p>
              <button type="button" class="cm-button cm-secondary"
                disabled={resourceOperationBusy} on:click={discoverExistingVirtuals}>
                {busy === "discover-virtuals" ? "Checking…" : "Inspect Sandboxie names"}
              </button>
              {#if existingVirtuals}
                <section class="cm-existing-virtuals" aria-label="Sandboxie configuration name candidates">
                  <h3>Sandboxie name candidates (read-only)</h3>
                  <p>These names come from existing Sandboxie configuration sections, including disabled boxes. Known user and template sections are omitted, but names alone cannot prove a box exists or a game is running. This is not the MIVUBI-owned engine.</p>
                  {#if existingVirtuals.boxes.length === 0}
                    <p>No Sandboxie box-name candidates were reported for the current Windows user.</p>
                  {:else}
                    <ul>
                      {#each existingVirtuals.boxes as boxName (boxName)}
                        <li>{boxName}</li>
                      {/each}
                    </ul>
                  {/if}
                </section>
              {/if}
              {#if existingVirtualsError}
                <p class="cm-warning cm-existing-error" role="alert">{existingVirtualsError}</p>
              {/if}
              <h3>M-Isolation-Engine · Internal runtime</h3>
              <p>The owned C/C++ source is not an installed or proven provider. Package and service-name inspections only gather diagnostic evidence; they do not authorize Create Virtual.</p>
              <p class="cm-assist" aria-live="polite">Engine readiness: {ownedHostError || ownedVirtualCreationReason(ownedHostInspection)}</p>
              <button type="button" class="cm-button cm-secondary"
                disabled={resourceOperationBusy}
                on:click={inspectStagedOwnedEngine}>
                {ownedInspectionBusy ? "Inspecting candidate…" : "Inspect engine candidate"}
              </button>
              {#if ownedCandidate}
                <p aria-live="polite">Engine package {ownedCandidate.upstreamCommit === "cdba7096240b93d79c5373aa81a75a1c5c71b883" ? "v1.18.6" : "v1.18.5"}: checksums match {ownedCandidate.verifiedArtifactCount} declared files and source paths, with no additional entries observed. Files could change after inspection. This does not authenticate the build or prove driver compatibility. Not installable; the official Sandboxie provider remains active.</p>
              {/if}
              {#if ownedInspectionError}
                <p class="cm-warning" role="alert">{ownedInspectionError}</p>
              {/if}
              <button type="button" class="cm-button cm-secondary"
                disabled={resourceOperationBusy}
                on:click={inspectOwnedServiceNames}>
                {ownedHostBusy ? "Inspecting Windows…" : "Check engine readiness"}
              </button>
              {#if ownedHostInspection}
                <p aria-live="polite">Host: {ownedHostInspection.state.replaceAll("_", " ")}. Engine activation remains blocked.</p>
              {/if}
              {#if ownedHostError}<p class="cm-warning" role="alert">{ownedHostError}</p>{/if}
              <div class="cm-diagnostic-presets">
                <h3>Planned native profiles</h3>
                <details>
                  <summary>Supported owned virtuals</summary>
                  <p>The current internal Minecraft preset supports three separate persistent boxes: MivubiEdu02, MivubiEdu03 and MivubiEdu04. General custom names are not yet supported by its kernel policy. MIVUBI has not created or verified these virtuals.</p>
                  <p>Minecraft login compatibility uses reduced process/COM isolation and requires explicit approval once the engine is operational.</p>
                </details>
              </div>
            </div>
          </details>
        </div>
      </section>
    {/if}
  </main>
</div>
