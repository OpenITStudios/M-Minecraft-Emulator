<script lang="ts">
  // Presentation only; parent owns state, saved settings, admission and native IPC.
  import {tick} from 'svelte';
  import type {BackgroundAudio, SettingsReport} from './app/bridge/lazyTester.js';
  import {userMessage} from './app/presentation/sessionFeedback.js';

  export let settings: SettingsReport | null;
  export let activeFps: number | undefined;
  export let backgroundFps: number | undefined;
  export let busy: boolean;
  export let available: boolean;
  export let refreshing: boolean;
  export let resizing: boolean;
  export let performanceQueued: boolean;
  export let audioQueued: boolean;
  export let savingPerformance: boolean;
  export let performanceDirty: boolean;
  export let performanceValid: boolean;
  export let settingsNotice: string;
  export let performanceNotice: string;
  export let onRequestPerformanceSave: () => void;
  export let onSaveAudio: (mode: BackgroundAudio) => Promise<void>;

  // Preserve a deliberate user's keyboard context when an existing save
  // disables its own control. These headings remain outside the Tab order;
  // saving and the native receipt still belong to LazyTesterApp.
  let frameHeading: HTMLHeadingElement | undefined;
  let audioHeading: HTMLHeadingElement | undefined;
  function savePerformanceWithFocus(event: SubmitEvent): void {
    const form = event.currentTarget;
    if (document.hasFocus() && form instanceof HTMLFormElement &&
        document.activeElement instanceof Node && form.contains(document.activeElement))
      frameHeading?.focus({preventScroll:true});
    onRequestPerformanceSave();
  }
  async function changeAudioWithFocus(event: Event): Promise<void> {
    const input = event.currentTarget;
    if (!(input instanceof HTMLInputElement)) return;
    // A native failure or lost receipt cannot leave a visually toggled
    // checkbox claiming that the new audio preference was applied.
    const intended = input.checked?'Mute':'Keep';
    if (document.hasFocus() && document.activeElement === input)
      audioHeading?.focus({preventScroll:true});
    try {
      await onSaveAudio(intended);
    } finally {
      await tick();
      if (input.isConnected) input.checked = settings?.backgroundAudio === 'Mute';
    }
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
          <section class="lt-group" aria-labelledby="lt-app-behavior-section"><h2 id="lt-app-behavior-section">Audio</h2><div class="lt-setting"><div><h3 bind:this={audioHeading} tabindex="-1">Background game audio</h3><p>Applies immediately after saving. Existing mute and volume settings stay.</p>{#if settings?.audioIssue}<p class="lt-error" role="status">{userMessage('audio')}</p>{/if}{#if !settings?.backgroundAudio}<p>Audio control is unavailable in this controller.</p>{/if}</div><label class="lt-audio-toggle"><input type="checkbox" checked={settings?.backgroundAudio==='Mute'} disabled={busy||audioQueued||refreshing||resizing||!available||!settings?.backgroundAudio} aria-label="Mute background games" on:change={changeAudioWithFocus}/>Mute background games</label></div></section>
          <section class="lt-group" aria-labelledby="lt-graphics-policy"><h2 id="lt-graphics-policy">Graphics <small>All launchers</small></h2><p class="lt-help">All five launchers use the optimized graphics preset at their next launch. No individual graphics preferences are needed.</p></section>
          {#if settingsNotice}<p role="status">{settingsNotice}</p>{/if}
        </div>
