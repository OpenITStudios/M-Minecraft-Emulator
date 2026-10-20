<script lang="ts">
  import {
    nativeFeasibility,
    isSandboxieBoxPrepared,
    canRequestClient02,
    type IsolatedClientObservation,
    type ProcessIdentityCandidate,
  } from "./app/bridge/nativeFeasibilityApi.js";

  // One short-lived request at a time; zero timers or background observers.
  let busy = false;
  let activeOperation = "";
  let selectedExecutable = "";
  let licenseConfirmed = false;
  let observation: IsolatedClientObservation | undefined;
  let notice = "";
  let error = "";

  $: membershipObserved = observation?.interactiveSession === true &&
    observation?.membershipStatus === "OBSERVED";
  $: nativeWindows = membershipObserved ? (observation?.nativeVisibleCandidates ?? null) : null;
  $: isolatedWindows = membershipObserved ? (observation?.isolatedVisibleCandidates ?? null) : null;
  // The backend still refreshes and verifies identity on every Focus request.
  $: focusCandidates = (observation?.processCandidates ?? []).filter((candidate) =>
    candidate.inSelectedBox === true && candidate.createdFileTime !== null &&
    candidate.windowHandle !== null);
  $: focusTarget = focusCandidates.length === 1 ? focusCandidates[0] : null;

  // Presentation-only guidance from the existing observation. The backend
  // remains the sole launch authority and always performs a fresh preflight.
  $: boxPrepared = isSandboxieBoxPrepared(observation);
  $: canLaunchIsolated = canRequestClient02(observation);
  $: nextAction = !observation
    ? "Choose Refresh to inspect the Windows session and Sandboxie configuration."
    : !observation.interactiveSession
    ? "Open the manager in an interactive Windows desktop session."
    : observation.boxConfigured !== "YES"
    ? "Create and enable the persistent MinecraftClient02 box in Sandboxie Plus."
    : observation.boxSettingsSource !== "BOX_SECTION_ENUMERATED"
    ? "Box settings could not be read reliably. Inspect Sandboxie Plus before launching."
    : observation.autoDelete !== "OFF" || observation.autoRemove !== "OFF"
    ? "Disable AutoDelete and AutoRemove so Client 02 does not lose its sandbox data."
    : observation.credentialHostAccess !== "OFF"
    ? "Review OpenCredentials and protected-storage options in Sandboxie Plus. This is not Microsoft account isolation proof."
    : observation.membershipStatus !== "OBSERVED"
    ? "Sandboxie membership or Windows process identity is unknown. Check Sandboxie Plus, process-inspection permissions and whether the box has processes in another Windows session, then Refresh. Client 02 launch remains blocked."
    : observation.isolatedCandidateProcesses > 0
    ? "A Client 02 process candidate exists. Use Focus if its window identity is available; do not request another launch."
    : "The inspected box settings meet the launch preflight checks. Select the official Desktop EXE and confirm licensing; authentication remains unverified.";

  function errorMessage(reason: unknown): string {
    if (typeof reason === "string" && reason.trim()) return reason.trim();
    if (reason instanceof Error) return reason.message;
    if (reason !== null && typeof reason === "object" && "message" in reason &&
        typeof reason.message === "string") return reason.message;
    return "Windows operation failed without a usable diagnostic message.";
  }

  function invalidateObservation() {
    observation = undefined;
    error = "";
    notice = "";
  }

  async function chooseExecutable() {
    if (busy) return;
    busy = true;
    activeOperation = "Selecting the Minecraft Education executable…";
    error = "";
    try {
      const chosen = await nativeFeasibility.selectEducationExe();
      if (chosen && chosen !== selectedExecutable) {
        selectedExecutable = chosen;
        licenseConfirmed = false;
      }
    } catch (reason) {
      error = errorMessage(reason);
    } finally {
      busy = false;
      activeOperation = "";
    }
  }

  async function launchNative() {
    if (busy) return;
    busy = true;
    activeOperation = "Requesting Client 01 launch…";
    invalidateObservation();
    try {
      await nativeFeasibility.openOfficialMinecraft();
      notice = "Client 01 launch requested. Use Refresh to observe the Windows process.";
    } catch (reason) {
      error = errorMessage(reason);
    } finally {
      busy = false;
      activeOperation = "";
    }
  }

  async function launchIsolated() {
    if (busy || !licenseConfirmed || !selectedExecutable || !canLaunchIsolated) return;
    busy = true;
    activeOperation = "Rechecking Sandboxie and requesting Client 02 launch…";
    invalidateObservation();
    try {
      await nativeFeasibility.requestSandboxClient02(selectedExecutable, licenseConfirmed);
      notice = "Client 02 launch requested in MinecraftClient02. Use Refresh to verify sandbox PID membership. Login and gameplay remain unverified.";
    } catch (reason) {
      error = errorMessage(reason);
    } finally {
      busy = false;
      activeOperation = "";
    }
  }

  async function focusClient02() {
    if (busy || !focusTarget) return;
    const target: ProcessIdentityCandidate = focusTarget;
    busy = true;
    activeOperation = "Verifying and focusing Client 02…";
    error = "";
    notice = "";
    try {
      await nativeFeasibility.focusObservedClient02(target);
      notice = "Windows accepted the Client 02 focus request. Game and account status remain unverified.";
    } catch (reason) {
      observation = undefined;
      error = errorMessage(reason);
    } finally {
      busy = false;
      activeOperation = "";
    }
  }

  async function placeClient02(side: "LEFT" | "RIGHT") {
    if (busy || !focusTarget) return;
    const target: ProcessIdentityCandidate = focusTarget;
    busy = true;
    activeOperation = "Verifying and arranging Client 02…";
    error = "";
    notice = "";
    try {
      await nativeFeasibility.placeObservedClient02(target, side);
      notice = `Windows accepted the Client 02 ${side === "LEFT" ? "left" : "right"} placement request. Client 01 was not moved.`;
    } catch (reason) {
      observation = undefined;
      error = errorMessage(reason);
    } finally {
      busy = false;
      activeOperation = "";
    }
  }

  async function refreshInstances() {
    if (busy) return;
    busy = true;
    activeOperation = "Inspecting Windows and Sandboxie process membership…";
    error = "";
    notice = "";
    observation = undefined;
    try {
      // A single canonical Sandboxie collector call covers BOTH clients.
      // SummaryOnly: no per-refresh evidence file or background process.
      observation = await nativeFeasibility.inspectIsolatedClient02();
      notice = "Process observations refreshed. No accounts or worlds were inspected.";
    } catch (reason) {
      error = errorMessage(reason);
    } finally {
      busy = false;
      activeOperation = "";
    }
  }
</script>

<div class="app-shell">
  <aside class="rail">
    <div class="brand">
      <div class="brand-mark" aria-hidden="true"><span></span><span></span><span></span></div>
      <div><strong>Minecraft Instance Manager</strong><span>Lightweight Windows pilot</span></div>
    </div>
    <div class="pilot-sidebar">
      <strong>Two clients · one controller</strong>
      <p>Manual launch and inspection. No VM guest, frame capture, background polling or account automation.</p>
    </div>
  </aside>

  <main>
    <header class="topbar">
      <h1>Minecraft Education · Two Windows clients</h1>
      <span class="pilot-label">GAMEPLAY NOT VERIFIED</span>
    </header>
    <div class="pilot-main">
      <section class="pilot-intro">
        <span class="eyebrow">ON-DEMAND CONTROL</span>
        <h2>Run Windows Minecraft directly, with isolated Client 02</h2>
        <p>Client 01 runs on the Windows host. Client 02 requests the same official Desktop game inside the persistent Sandboxie Plus box MinecraftClient02. The manager never captures game frames or creates virtual machines.</p>
        <div class="pilot-actions">
          <button class="pilot-primary" disabled={busy} on:click={launchNative}>
            Launch Client 01
          </button>
          <button class="pilot-secondary" disabled={busy} on:click={refreshInstances}>
            {busy ? "Checking…" : "Refresh both clients"}
          </button>
        </div>
        {#if activeOperation}<p class="pilot-info" role="status">{activeOperation}</p>{/if}
        {#if notice}<p class="pilot-info" role="status">{notice}</p>{/if}
        {#if error}<p class="pilot-error" role="alert">{error}</p>{/if}
        <p class="pilot-caution">Refresh is a single explicit Sandboxie/Windows inspection. It does not poll in the background or save a diagnostic report.</p>
      </section>

      <section class="pilot-section pilot-readiness" aria-label="Setup guidance">
        <span class="eyebrow">SETUP GUIDANCE</span>
        <h3>What needs attention?</h3>
        <p>These are configuration observations, not a Minecraft readiness or sign-in verdict. Launch always rechecks the current Windows state.</p>
        <ul class="pilot-readiness-list">
          <li>
            <strong>Windows desktop</strong>
            <span>{!observation ? "Not inspected" : observation.interactiveSession ? "Interactive session observed" : "Interactive session unavailable"}</span>
          </li>
          <li>
            <strong>Persistent Sandboxie box</strong>
            <span>{!observation ? "Not inspected" : boxPrepared ? "Configuration checks passed" : "Configuration incomplete or unknown"}</span>
          </li>
          <li>
            <strong>Client 02 process inventory</strong>
            <span>{!observation ? "Not inspected" : observation.membershipStatus !== "OBSERVED" ? "Unknown — not an empty-box result" : observation.isolatedCandidateProcesses > 0 ? `${observation.isolatedCandidateProcesses} candidate(s) inside box` : "No candidate observed at inspection time"}</span>
          </li>
          <li>
            <strong>Account sign-in and gameplay</strong>
            <span>Not verified by this manager</span>
          </li>
        </ul>
        <p class="pilot-next-action" role="status"><strong>Next step:</strong> {nextAction}</p>
      </section>

      <section class="pilot-section" aria-label="Client status">
        <span class="eyebrow">WINDOWS PROCESS CANDIDATES</span>
        <div class="pilot-grid">
          <article class="pilot-slot">
            <strong>Client 01 · Native</strong>
            <p>Minecraft Education Windows host</p>
            <span>{nativeWindows === null ? "Membership UNKNOWN" : nativeWindows > 0 ? `${nativeWindows} visible candidate(s) outside selected box — native unverified` : "No candidate outside selected box"}</span>
            {#if membershipObserved}
              <p class="pilot-caution">{observation?.officialImageOutsideSelectedBoxCount ?? 0} Microsoft-signed Education image candidate(s) outside the selected box. Native identity and account login remain unverified.</p>
              {#if observation?.directPidQuery === "QUERIED"}
                <p class="pilot-caution">Sandboxie PID lookup: {observation.signedImageOtherBoxCount} signed candidate(s) confirmed in another box; {observation.signedImageUnknownBoxCount} have UNKNOWN sandbox attribution. Neither result proves an unsandboxed native process.</p>
              {:else if observation?.directPidQuery === "UNAVAILABLE"}
                <p class="pilot-caution">Direct Sandboxie PID lookup unavailable. All {observation.signedImageUnknownBoxCount} outside-box signed candidate(s) retain UNKNOWN attribution.</p>
              {/if}
            {/if}
          </article>
          <article class="pilot-slot">
            <strong>Client 02 · Isolated</strong>
            <p>Sandboxie box MinecraftClient02</p>
            <span>{isolatedWindows === null ? "Membership UNKNOWN" : observation?.isolatedCandidateProcesses ? `${observation.isolatedCandidateProcesses} process candidate(s) in box · ${isolatedWindows} with window` : "No Minecraft-name process candidate observed in box"}</span>
            <div class="pilot-actions">
              <button class="pilot-secondary" disabled={busy || !focusTarget} on:click={focusClient02}>
                Focus Client 02
              </button>
              <button class="pilot-secondary" disabled={busy || !focusTarget} on:click={() => placeClient02("LEFT")}>
                Place Left
              </button>
              <button class="pilot-secondary" disabled={busy || !focusTarget} on:click={() => placeClient02("RIGHT")}>
                Place Right
              </button>
            </div>
            <p class="pilot-caution">Focus and placement require one observed boxed game-window candidate, not an owned popup. Before acting, the backend checks that PID's Microsoft-signed Education executable metadata and rechecks the live process/window; UNKNOWN or unsigned files are refused. This does not verify login or gameplay. Place Left/Right never moves Client 01; restore and Refresh if necessary.</p>
            {#if isolatedWindows !== null && !focusTarget}
              <p class="pilot-caution">Focus and placement require exactly one boxed window with a known process start time. Refresh or select the game window manually.</p>
            {/if}
          </article>
        </div>
        {#if observation}
          <p class="pilot-caution">
            An outside-box process is not necessarily Client 01; another sandbox or Minecraft edition may own it. Neither process counts nor OpenCredentials OFF establish independent Microsoft accounts.
          </p>
          <details class="pilot-diagnostics">
            <summary>Advanced Sandboxie diagnostics</summary>
            <p>Box configured: {observation.boxConfigured} · Settings source: {observation.boxSettingsSource} · PID membership: {observation.membershipStatus}.</p>
            <p>Known credential bypass settings: {observation.credentialHostAccess} · AutoDelete: {observation.autoDelete} · AutoRemove: {observation.autoRemove}.</p>
            <p>This is a bounded, on-demand observation. It never reads Microsoft credentials, account tokens or Minecraft worlds.</p>
          </details>
        {:else}
          <p class="pilot-caution">No current inspection. Click Refresh both clients before interpreting the client status.</p>
        {/if}
      </section>

      <section class="pilot-section" aria-label="Launch isolated Client 02">
        <span class="eyebrow">CLIENT 02 · EXPLICIT ISOLATED LAUNCH</span>
        <h3>Launch through Sandboxie Plus</h3>
        <p>Create a clean, persistent box named MinecraftClient02 in Sandboxie Plus. Use its appropriate licence; keep AutoDelete, AutoRemove and OpenCredentials disabled. Select the installed Microsoft-signed Minecraft Education Desktop executable.</p>
        <div class="pilot-actions">
          <button class="pilot-secondary" disabled={busy} on:click={chooseExecutable}>
            Select installed Education EXE
          </button>
        </div>
        {#if selectedExecutable}
          <p class="pilot-caution">Education EXE selected for this session. Backend will verify file metadata and signer before requesting launch.</p>
        {/if}
        <label class="pilot-caution">
          <input type="checkbox" bind:checked={licenseConfirmed} />
          I confirm Sandboxie Plus is licensed for this use, and MinecraftClient02 is a persistent box.
        </label>
        <div class="pilot-actions">
          <button class="pilot-primary" disabled={busy || !licenseConfirmed || !selectedExecutable || !canLaunchIsolated} on:click={launchIsolated}>
            Launch Client 02
          </button>
        </div>
        {#if !canLaunchIsolated}
          <p class="pilot-caution">Use Refresh to check setup first. Incomplete Sandboxie settings, an unknown process inventory, or an existing Client 02 candidate prevent another launch.</p>
        {/if}
        <p class="pilot-caution">Launch performs a fresh backend inspection. Unreadable Sandboxie process membership blocks launch rather than assuming the box is empty. A previous Refresh cannot waive this check.</p>
      </section>

      <section class="pilot-section pilot-boundary">
        <span class="eyebrow">VALIDATION BOUNDARY</span>
        <h3>Two independent licensed game accounts are not yet proven</h3>
        <p>OpenCredentials OFF is only a WinCred hook setting and may still permit host credential reads. OpenProtectedStorage can bypass the hooks. Microsoft WAM/ADAL helpers may behave differently. Independently verify login, input, world playability, multiplayer and persistence before declaring Client 02 compatible.</p>
        <p>The former VMware runtime remains disabled and its account data untouched. The old VMware NSIS installer is blocked.</p>
      </section>
    </div>
  </main>
</div>

<style>
  .pilot-sidebar { border-top: 1px solid rgba(255,255,255,.09); padding: 20px 10px; font-size: 12px; color: #b4becb; }
  .pilot-sidebar p { font-size: 11px; color: #828c9d; line-height: 1.65; }
  .pilot-label { border:1px solid #605035; background:#2e2519; color:#e5c48d; padding:5px 10px; border-radius:20px; font-size:10px; font-weight:700; }
  .pilot-main { display:grid; gap:24px; padding:34px; max-width:1120px; }
  .pilot-intro,.pilot-section { border:1px solid rgba(255,255,255,.08); border-radius:16px; padding:26px; background:#10141c; }
  .pilot-intro h2 { font-size:26px; margin:12px 0; }
  .pilot-intro p,.pilot-boundary p { color:#9ca9ba; max-width:760px; line-height:1.75; }
  .pilot-actions { display:flex; flex-wrap:wrap; gap:12px; align-items:center; margin-top:16px; }
  .pilot-primary,.pilot-secondary { display:inline-block; padding:12px 18px; border-radius:9px; font-weight:750; cursor:pointer; }
  .pilot-primary { border:none; background:#949eff; color:#10121b; }
  .pilot-secondary { border:1px solid rgba(255,255,255,.25); background:#212938; color:#f3f5f9; }
  .pilot-primary:disabled,.pilot-secondary:disabled { opacity:.5; cursor:not-allowed; }
  .pilot-primary:focus-visible,.pilot-secondary:focus-visible { outline:2px solid white; outline-offset:2px; }
  .pilot-info { color:#a4dabc !important; }
  .pilot-error { color:#f3aaa5 !important; }
  .pilot-caution { color:#b7a37e; font-size:12px; line-height:1.7; margin-top:16px; }
  .pilot-grid { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:12px; margin-top:15px; }
  .pilot-slot { padding:18px; border-radius:12px; background:#191e28; border:1px solid rgba(255,255,255,.06); }
  .pilot-slot strong { font-size:14px; }
  .pilot-slot p { color:#a0acbb; font-size:12px; }
  .pilot-slot span { color:#7f8c9a; font-size:11px; }
  .pilot-boundary h3 { margin:12px 0; }
  .pilot-readiness h3 { margin:12px 0; }
  .pilot-readiness > p { color:#a0acbb; line-height:1.65; }
  .pilot-readiness-list { margin:18px 0; padding:0; display:grid; gap:8px; list-style:none; }
  .pilot-readiness-list li { display:flex; align-items:baseline; justify-content:space-between; gap:10px; background:#191e28; border-radius:8px; padding:12px 14px; }
  .pilot-readiness-list strong { font-size:13px; font-weight:700; }
  .pilot-readiness-list span { color:#b6c2d3; font-size:12px; text-align:right; }
  .pilot-next-action { background:#202837; border-left:3px solid #949eff; border-radius:6px; padding:12px 14px; font-size:13px; }
  .pilot-diagnostics { padding:12px 14px; border:1px solid rgba(255,255,255,.12); border-radius:8px; margin-top:14px; font-size:12px; color:#a0acbb; }
  .pilot-diagnostics summary { cursor:pointer; color:#d9e1ed; font-weight:650; }
  .pilot-diagnostics summary:focus-visible { outline:2px solid white; outline-offset:3px; }
  .pilot-diagnostics p { line-height:1.6; margin:10px 0 0; }
  @media(max-width:740px) { .pilot-grid { grid-template-columns:1fr; } .pilot-main { padding:16px; } .pilot-readiness-list li { flex-direction:column; gap:4px; } .pilot-readiness-list span { text-align:left; } }
</style>
