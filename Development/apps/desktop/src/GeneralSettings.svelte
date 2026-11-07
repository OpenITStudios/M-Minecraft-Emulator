<script lang="ts">
  // Presentation only; parent owns state, saved settings, admission and native IPC.
  import type {BackgroundAudio, DataLocation, GraphicsPreset, ProfileSession, ProfileSlot, SettingsReport} from './app/bridge/lazyTester.js';
  import {userMessage} from './app/presentation/sessionFeedback.js';

  export let profiles: ProfileSession[];
  export let selectedSlot: ProfileSlot;
  export let settings: SettingsReport | null;
  export let graphicsPreset: GraphicsPreset;
  export let workspaceLayout: 'focus' | 'monitor';
  export let layoutPersistenceIssue: boolean;
  export let onChooseWorkspaceLayout: (mode: 'focus' | 'monitor') => void;
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
  export let dataLocation: DataLocation | null;
  export let locationIssue: string;
  export let onChooseSlot: (slot: ProfileSlot) => void;
  export let onRequestPreset: () => void;
  export let onDiscardGraphics: () => void;
  export let onRequestPerformanceSave: () => void;
  export let onSaveAudio: (mode: BackgroundAudio) => void;
  export let onOpenDataFolder: () => void;
  export let onShowDataStorage: () => void;

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
          <section class="lt-group" aria-labelledby="lt-workspace-layout-section">
            <h2 id="lt-workspace-layout-section">Workspace layout</h2>
            <div class="lt-setting">
              <div><h3><label for="workspace-layout">Workspace layout</label></h3>
                <p>Choose the balance between the game you're playing and the background previews.</p>
              </div>
              <select id="workspace-layout" value={workspaceLayout}
                aria-describedby="lt-workspace-layout-help"
                on:change={event=>onChooseWorkspaceLayout(event.currentTarget.value as 'focus' | 'monitor')}>
                <option value="focus">Gameplay Focus (recommended)</option>
                <option value="monitor">Monitoring (larger previews)</option>
              </select>
            </div>
            <p id="lt-workspace-layout-help" class="lt-help">Changes only Lazy Tester layout, not Minecraft graphics or FPS. The preference is kept on this device when storage is available.</p>
            {#if layoutPersistenceIssue}<p class="lt-field-error" role="status">Layout changed for this session, but could not be saved on this device.</p>{/if}
          </section>
          <section class="lt-group" aria-labelledby="lt-graphics-section"><h2 id="lt-graphics-section" bind:this={graphicsHeading} tabindex="-1">Graphics</h2><label class="lt-target">Applies to <select value={selectedSlot} disabled={busy||presetQueued} on:change={event=>onChooseSlot(Number(event.currentTarget.value) as ProfileSlot)}>{#each profiles as p}<option value={p.slot}>Minecraft - {p.slot}</option>{/each}</select></label>
            <div class="lt-setting"><div><h3><label for="graphics-preset">Graphics preset</label></h3><p>Choose whether to keep Minecraft's own visuals or apply a legacy preset for this launcher.</p></div><select id="graphics-preset" bind:value={graphicsPreset} disabled={busy||presetQueued||!settings?.graphicsPresets}><option value="Preserve">Preserve Visuals</option><option value="High">High (legacy)</option><option value="Low">Low (legacy)</option></select></div>
            <div class="lt-actions"><button disabled={busy||presetQueued||!available||!bindingConfigured||!settings?.canApplyPreset||!graphicsDirty||runningCount>0} on:click={saveGraphicsWithFocus}>{applyingPreset||presetQueued?'Applying…':graphicsPreset==='Preserve'?'Save preference':'Apply preset'}</button>{#if graphicsDirty}<button type="button" class="lt-secondary" disabled={busy||presetQueued} on:click={onDiscardGraphics}>Discard draft</button>{/if}</div>
            <p class="lt-help">{runningCount>0?'Close all Minecraft sessions before saving a graphics preference.':'Saved graphics preferences apply the next time this Minecraft starts.'}</p>
            {#if graphicsDirty}<p class="lt-help" role="status">Unsaved graphics choice. You can switch launchers and return to this draft before closing Lazy Tester; no setting has been written yet.</p>{/if}
            {#if settings?.graphicsPresets && !settings.canApplyPreset && runningCount===0}<p class="lt-help">Graphics changes are unavailable in the latest controller report. Choose Refresh to check again.</p>{/if}
            <details class="lt-settings-explanation"><summary>What do these presets change?</summary>
              <p>Preserve Visuals leaves Minecraft's graphics options unchanged on future launches and does not create options.txt. It does not undo values previously changed by Low/High. The legacy High/Low presets each apply 12 chunks and MSAA 1; simulation distance stays unchanged. Existing explicit High/Low preferences are kept.</p>
            </details>
            {#if settings&&!settings.graphicsPresets}<p class="lt-error">Update the running controller before changing graphics presets. Existing games are preserved.</p>{/if}{#if settingsNotice}<p role="status">{settingsNotice}</p>{/if}
          </section>
          <section class="lt-group" aria-labelledby="lt-frame-limits-section"><h2 id="lt-frame-limits-section" bind:this={frameHeading} tabindex="-1">Frame Limits <small>All launchers</small></h2>{#if settings}<form on:submit|preventDefault={savePerformanceWithFocus}>
            <div class="lt-setting"><div><h3><label for="active-fps">While playing</label></h3><p>The game receiving keyboard and mouse input.</p></div><div class="lt-fps"><input id="active-fps" type="number" min="5" max="1000" required bind:value={activeFps} disabled={busy||performanceQueued} aria-invalid={activeFpsFieldInvalid} aria-describedby={activeFpsFieldInvalid?'lt-fps-guidance lt-active-fps-error':'lt-fps-guidance'}/> FPS</div></div>
            <div class="lt-setting"><div><h3><label for="background-fps">In the background</label></h3><p>Other games keep running at this limit.</p></div><div class="lt-fps"><input id="background-fps" type="number" min="5" max={activeFps??1000} required bind:value={backgroundFps} disabled={busy||performanceQueued} aria-invalid={backgroundFpsFieldInvalid} aria-describedby={backgroundFpsFieldInvalid?'lt-fps-guidance lt-background-fps-error':'lt-fps-guidance'}/> FPS</div></div>
            <p id="lt-fps-guidance" class="lt-help">Use whole numbers from 5 to 1000 FPS; background cannot exceed active FPS. Saved limits apply at the next launch; running games keep their current limits.</p>
            {#if !performanceValid}<div class="lt-form-errors" role="status" aria-live="polite">
              {#if activeFpsFieldInvalid}<p id="lt-active-fps-error" class="lt-field-error">While playing: enter a whole number from 5 to 1000.</p>{/if}
              {#if backgroundFpsFieldInvalid}<p id="lt-background-fps-error" class="lt-field-error">In the background: enter a whole number from 5 to 1000, no higher than While playing.</p>{/if}
            </div>{/if}
            {#if performanceDirty && !available}<p class="lt-help" role="status">Session status is unavailable. Choose Refresh before saving these limits.</p>{/if}
            <div class="lt-actions"><button type="submit" disabled={busy||performanceQueued||!available||!performanceDirty||!performanceValid}>{savingPerformance||performanceQueued?'Saving…':'Save changes'}</button>{#if performanceDirty}<button type="button" class="lt-secondary" disabled={busy||performanceQueued} on:click={()=>{activeFps=settings!.activeFps;backgroundFps=settings!.backgroundFps;}}>Discard</button>{/if}<button type="button" class="lt-secondary" disabled={busy||performanceQueued} on:click={()=>{activeFps=120;backgroundFps=5;}}>Use 120 / 5</button></div>{#if performanceNotice}<p role="status">{performanceNotice}</p>{/if}
          </form>{:else}<p>Settings unavailable. Refresh to try again.</p>{/if}</section>
          <section class="lt-group" aria-labelledby="lt-app-behavior-section"><h2 id="lt-app-behavior-section">App behavior</h2><div class="lt-setting"><div><h3>When Lazy Tester closes</h3><p>All Minecraft sessions started by Lazy Tester close normally first. Lazy Tester waits for them to finish before exiting.</p></div></div><p class="lt-help">If Minecraft is waiting for a dialog or has not finished closing, Lazy Tester stays open. Other Minecraft applications are left alone.</p><div class="lt-setting"><div><h3 bind:this={audioHeading} tabindex="-1">Background game audio</h3><p>Mute games that are not receiving keyboard and mouse input. Existing user mute and volume settings are kept.</p>{#if settings?.audioIssue}<p class="lt-error" role="status">{userMessage('audio')}</p>{/if}{#if !settings?.backgroundAudio}<p>Update the running controller to enable audio control.</p>{/if}</div><label class="lt-audio-toggle"><input type="checkbox" checked={settings?.backgroundAudio==='Mute'} disabled={busy||audioQueued||!settings?.backgroundAudio} aria-label="Mute background games" on:change={changeAudioWithFocus}/>Mute background games</label></div></section>
          <section class="lt-group" aria-labelledby="lt-install-data-section"><h2 id="lt-install-data-section">Installation & data</h2><h3>Minecraft Education</h3><p>The controller verifies the registered game executable before launch. Moving data does not move Minecraft.</p><h3>Launcher data location</h3><p class="lt-data-path">{dataLocation?.path??'No data folder connected.'}</p>{#if locationIssue}<p class="lt-error">{locationIssue}</p>{/if}<div class="lt-actions"><button class="lt-secondary" disabled={busy||refreshing||resizing||!dataLocation?.path} on:click={onOpenDataFolder}>Open data folder</button><button class="lt-secondary" on:click={onShowDataStorage}>Data Storage →</button></div></section>
        </div>
