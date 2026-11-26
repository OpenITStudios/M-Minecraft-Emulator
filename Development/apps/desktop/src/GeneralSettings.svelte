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

        <div class="lazytester-settings-content">
          <section class="lazytester-group" aria-labelledby="lazytester-frame-limits-section"><h2 id="lazytester-frame-limits-section" bind:this={frameHeading} tabindex="-1">Frame Rate</h2>{#if settings}<form class="lazytester-settings-panel" on:submit|preventDefault={savePerformanceWithFocus}>
            <div class="lazytester-setting"><div><h3><label for="active-fps">Active FPS</label></h3><p>Minecraft window in use.</p></div><div class="lazytester-fps"><input id="active-fps" type="number" min="5" max="1000" required bind:value={activeFps} disabled={busy||performanceQueued} aria-invalid={activeFpsFieldInvalid} aria-describedby={activeFpsFieldInvalid?'lazytester-fps-guidance lazytester-active-fps-error':'lazytester-fps-guidance'}/> FPS</div></div>
            <div class="lazytester-setting"><div><h3><label for="background-fps">Background FPS</label></h3><p>Other running Minecraft windows.</p></div><div class="lazytester-fps"><input id="background-fps" type="number" min="5" max={activeFps??1000} required bind:value={backgroundFps} disabled={busy||performanceQueued} aria-invalid={backgroundFpsFieldInvalid} aria-describedby={backgroundFpsFieldInvalid?'lazytester-fps-guidance lazytester-background-fps-error':'lazytester-fps-guidance'}/> FPS</div></div>
            <p id="lazytester-fps-guidance" class="lazytester-visually-hidden">FPS limits: 5–1000. Background FPS cannot exceed active FPS. These are limits, not measured FPS.</p>
            {#if !performanceValid}<div class="lazytester-form-errors" role="status" aria-live="polite">
              {#if activeFpsFieldInvalid}<p id="lazytester-active-fps-error" class="lazytester-field-error">Enter a whole number from 5 to 1000.</p>{/if}
              {#if backgroundFpsFieldInvalid}<p id="lazytester-background-fps-error" class="lazytester-field-error">Enter 5–1000, no higher than Active FPS.</p>{/if}
            </div>{/if}
            {#if performanceDirty && !available}<p class="lazytester-help" role="status">Session status unavailable. Refresh before saving.</p>{/if}
            <div class="lazytester-actions lazytester-settings-panel-footer"><span class="lazytester-settings-note">Applies on next launch.</span><button type="submit" disabled={busy||performanceQueued||!available||!performanceDirty||!performanceValid}>{savingPerformance||performanceQueued?'Saving…':'Save Changes'}</button>{#if performanceDirty}<button type="button" class="lazytester-secondary" disabled={busy||performanceQueued} on:click={()=>{activeFps=settings!.activeFps;backgroundFps=settings!.backgroundFps;}}>Discard</button>{/if}</div>{#if performanceNotice}<p role="status">{performanceNotice}</p>{/if}
          </form>{:else}<p>Settings unavailable. Refresh to try again.</p>{/if}</section>
          <section class="lazytester-group" aria-labelledby="lazytester-app-behavior-section"><h2 id="lazytester-app-behavior-section">Audio</h2><div class="lazytester-settings-panel"><div class="lazytester-setting"><div><h3 bind:this={audioHeading} tabindex="-1">Mute Background Games</h3><p>Mute audio from background games.</p>{#if settings?.audioIssue}<p class="lazytester-error" role="status">{userMessage('audio')}</p>{/if}{#if !settings?.backgroundAudio}<p>Audio control is unavailable in this controller.</p>{/if}</div><label class="lazytester-audio-toggle"><input type="checkbox" role="switch" checked={settings?.backgroundAudio==='Mute'} disabled={busy||audioQueued||refreshing||resizing||!available||!settings?.backgroundAudio} aria-label="Mute background games" on:change={changeAudioWithFocus}/></label></div></div></section>
          {#if settingsNotice}<p role="status">{settingsNotice}</p>{/if}
        </div>
