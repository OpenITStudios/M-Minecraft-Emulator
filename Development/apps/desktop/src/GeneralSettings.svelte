<script lang="ts">
  // Presentation only; parent owns state, saved settings, admission and native IPC.
  import type {BackgroundAudio, GraphicsPreset, ProfileSession, ProfileSlot, SettingsReport} from './app/bridge/lazyTester.js';
  import {userMessage} from './app/presentation/sessionFeedback.js';

  export let profiles: ProfileSession[];
  export let selectedSlot: ProfileSlot;
  export let settings: SettingsReport | null;
  export let graphicsPreset: GraphicsPreset;
  export let activeFps: number | undefined;
  export let backgroundFps: number | undefined;
  export let busy: boolean;
  export let available: boolean;
  export let bindingConfigured: boolean;
  export let refreshing: boolean;
  export let resizing: boolean;
  export let presetQueued: boolean;
  export let performanceQueued: boolean;
  export let audioQueued: boolean;
  export let applyingPreset: boolean;
  export let savingPerformance: boolean;
  export let graphicsDirty: boolean;
  export let performanceDirty: boolean;
  export let performanceValid: boolean;
  export let runningCount: number;
  export let settingsNotice: string;
  export let performanceNotice: string;
  export let onChooseSlot: (slot: ProfileSlot) => void;
  export let onRequestPreset: () => void;
  export let onDiscardGraphics: () => void;
  export let onRequestPerformanceSave: () => void;
  export let onSaveAudio: (mode: BackgroundAudio) => void;

  // Preserve a deliberate user's keyboard context when an existing save
  // disables its own control. These headings remain outside the Tab order;
  // saving and the native receipt still belong to LazyTesterApp.
  let graphicsHeading: HTMLHeadingElement | undefined;
  let frameHeading: HTMLHeadingElement | undefined;
  let audioHeading: HTMLHeadingElement | undefined;
  function saveGraphicsWithFocus(event: MouseEvent): void {
    if (document.hasFocus() && document.activeElement === event.currentTarget)
      graphicsHeading?.focus({preventScroll:true});
    onRequestPreset();
  }
  function savePerformanceWithFocus(event: SubmitEvent): void {
    const form = event.currentTarget;
    if (document.hasFocus() && form instanceof HTMLFormElement &&
        document.activeElement instanceof Node && form.contains(document.activeElement))
      frameHeading?.focus({preventScroll:true});
    onRequestPerformanceSave();
  }
  function changeAudioWithFocus(event: Event): void {
    const input = event.currentTarget;
    if (!(input instanceof HTMLInputElement)) return;
    if (document.hasFocus() && document.activeElement === input)
      audioHeading?.focus({preventScroll:true});
    onSaveAudio(input.checked?'Mute':'Keep');
  }

  // Field feedback only; original performanceValid and native admission own Save.
  $: activeFpsFieldInvalid = typeof activeFps !== 'number' ||
    !Number.isInteger(activeFps) || activeFps < 5 || activeFps > 1000;
  $: backgroundFpsFieldInvalid = typeof backgroundFps !== 'number' ||
    !Number.isInteger(backgroundFps) || backgroundFps < 5 ||
    backgroundFps > 1000 ||
    (!activeFpsFieldInvalid && backgroundFps > (activeFps ?? 1000));
</script>

        <div class="lt-settings-content">
          <section class="lt-group" aria-labelledby="lt-graphics-section"><h2 id="lt-graphics-section" bind:this={graphicsHeading} tabindex="-1">Graphics <small>Selected launcher</small></h2><label class="lt-target">Applies to <select value={selectedSlot} disabled={busy||presetQueued} on:change={event=>onChooseSlot(Number(event.currentTarget.value) as ProfileSlot)}>{#each profiles as p}<option value={p.slot}>MC0{p.slot}</option>{/each}</select></label>
            <div class="lt-setting"><div><h3><label for="graphics-preset">Graphics preset</label></h3><p>Choose this launcher's visuals for its next start.</p></div><select id="graphics-preset" bind:value={graphicsPreset} disabled={busy||presetQueued||!settings?.graphicsPresets}><option value="Preserve">Preserve Visuals (recommended)</option><option value="High">High (legacy)</option><option value="Low">Low (legacy)</option></select></div>
            <div class="lt-actions"><button disabled={busy||presetQueued||!available||!bindingConfigured||!settings?.canApplyPreset||!graphicsDirty||runningCount>0} on:click={saveGraphicsWithFocus}>{applyingPreset||presetQueued?'Saving…':'Save graphics preference'}</button>{#if graphicsDirty}<button type="button" class="lt-secondary" disabled={busy||presetQueued} on:click={onDiscardGraphics}>Discard draft</button>{/if}</div>
            <p class="lt-help">{runningCount>0?'Close all games before saving graphics.':'Applies to this launcher at its next start.'}</p>
            {#if graphicsDirty}<p class="lt-help" role="status">Unsaved choice. You can return to it until Lazy Tester closes.</p>{/if}
            {#if settings?.graphicsPresets && !settings.canApplyPreset && runningCount===0}<p class="lt-help">Graphics changes unavailable. Refresh to check again.</p>{/if}
            <details class="lt-settings-explanation"><summary>About legacy graphics presets</summary>
              <p>Preserve Visuals leaves Minecraft graphics unchanged and does not undo earlier edits. Legacy High/Low both set 12 chunks and MSAA 1; simulation distance is unchanged. Neither guarantees higher FPS.</p>
            </details>
            {#if settings&&!settings.graphicsPresets}<p class="lt-error">Update the running controller before changing graphics presets. Existing games are preserved.</p>{/if}{#if settingsNotice}<p role="status">{settingsNotice}</p>{/if}
          </section>
          <section class="lt-group" aria-labelledby="lt-frame-limits-section"><h2 id="lt-frame-limits-section" bind:this={frameHeading} tabindex="-1">Frame Limits <small>All launchers</small></h2>{#if settings}<form on:submit|preventDefault={savePerformanceWithFocus}>
            <div class="lt-setting"><div><h3><label for="active-fps">While playing</label></h3><p>Game in focus.</p></div><div class="lt-fps"><input id="active-fps" type="number" min="5" max="1000" required bind:value={activeFps} disabled={busy||performanceQueued} aria-invalid={activeFpsFieldInvalid} aria-describedby={activeFpsFieldInvalid?'lt-fps-guidance lt-active-fps-error':'lt-fps-guidance'}/> FPS</div></div>
            <div class="lt-setting"><div><h3><label for="background-fps">In the background</label></h3><p>Other running games.</p></div><div class="lt-fps"><input id="background-fps" type="number" min="5" max={activeFps??1000} required bind:value={backgroundFps} disabled={busy||performanceQueued} aria-invalid={backgroundFpsFieldInvalid} aria-describedby={backgroundFpsFieldInvalid?'lt-fps-guidance lt-background-fps-error':'lt-fps-guidance'}/> FPS</div></div>
            <p id="lt-fps-guidance" class="lt-help">FPS limits: 5–1000. Background cannot exceed active. These are caps, not measured FPS; they apply at the next launch.</p>
            {#if !performanceValid}<div class="lt-form-errors" role="status" aria-live="polite">
              {#if activeFpsFieldInvalid}<p id="lt-active-fps-error" class="lt-field-error">Enter a whole number from 5 to 1000.</p>{/if}
              {#if backgroundFpsFieldInvalid}<p id="lt-background-fps-error" class="lt-field-error">Enter 5–1000, no higher than While playing.</p>{/if}
            </div>{/if}
            {#if performanceDirty && !available}<p class="lt-help" role="status">Session status unavailable. Refresh before saving.</p>{/if}
            <div class="lt-actions"><button type="submit" disabled={busy||performanceQueued||!available||!performanceDirty||!performanceValid}>{savingPerformance||performanceQueued?'Saving…':'Save FPS limits'}</button>{#if performanceDirty}<button type="button" class="lt-secondary" disabled={busy||performanceQueued} on:click={()=>{activeFps=settings!.activeFps;backgroundFps=settings!.backgroundFps;}}>Discard</button>{/if}</div>{#if performanceNotice}<p role="status">{performanceNotice}</p>{/if}
          </form>{:else}<p>Settings unavailable. Refresh to try again.</p>{/if}</section>
          <section class="lt-group" aria-labelledby="lt-app-behavior-section"><h2 id="lt-app-behavior-section">App behavior</h2><div class="lt-setting"><div><h3>When Lazy Tester closes</h3><p>On exit, Lazy Tester closes managed games normally and waits.</p></div></div><p class="lt-help">If a game needs attention, Lazy Tester stays open. Other Minecraft apps are unaffected.</p><div class="lt-setting"><div><h3 bind:this={audioHeading} tabindex="-1">Background game audio</h3><p>Applies immediately after saving. Existing mute and volume settings stay.</p>{#if settings?.audioIssue}<p class="lt-error" role="status">{userMessage('audio')}</p>{/if}{#if !settings?.backgroundAudio}<p>Audio control is unavailable in this controller.</p>{/if}</div><label class="lt-audio-toggle"><input type="checkbox" checked={settings?.backgroundAudio==='Mute'} disabled={busy||audioQueued||!settings?.backgroundAudio} aria-label="Mute background games" on:change={changeAudioWithFocus}/>Mute background games</label></div></section>
        </div>
