<script lang="ts">
  // Presentation only; parent owns state, saved settings, admission and native IPC.
  import type {BackgroundAudio, DataLocation, GraphicsPreset, ProfileSession, ProfileSlot, SettingsReport} from './app/bridge/lazyTester.js';
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
  export let dataLocation: DataLocation | null;
  export let locationIssue: string;
  export let onChooseSlot: (slot: ProfileSlot) => void;
  export let onRequestPreset: () => void;
  export let onRequestPerformanceSave: () => void;
  export let onSaveAudio: (mode: BackgroundAudio) => void;
  export let onOpenDataFolder: () => void;
  export let onShowDataStorage: () => void;
</script>

        <div class="lt-settings-content">
          <section class="lt-group"><h2>Graphics</h2><label class="lt-target">Applies to <select value={selectedSlot} disabled={busy||presetQueued} on:change={event=>onChooseSlot(Number(event.currentTarget.value) as ProfileSlot)}>{#each profiles as p}<option value={p.slot}>Minecraft - {p.slot}</option>{/each}</select></label>
            <div class="lt-setting"><div><h3><label for="graphics-preset">Graphics preset</label></h3><p>Preserve Visuals does not modify Minecraft's graphics options or create new options.txt. High/Low are legacy visual presets that apply 12 chunks and MSAA 1. Simulation distance is unchanged.</p></div><select id="graphics-preset" bind:value={graphicsPreset} disabled={busy||presetQueued||!settings?.graphicsPresets}><option value="Preserve">Preserve Visuals</option><option value="High">High (legacy)</option><option value="Low">Low (legacy)</option></select></div>
            <div class="lt-actions"><button disabled={busy||presetQueued||!available||!bindingConfigured||!settings?.canApplyPreset||!graphicsDirty||runningCount>0} on:click={onRequestPreset}>{applyingPreset||presetQueued?'Applying…':graphicsPreset==='Preserve'?'Save preference':'Apply preset'}</button></div>
            <p class="lt-help">Close all Minecraft sessions before saving. Preserve Visuals stops future automatic overrides but does not restore graphics values previously changed by Low/High. Existing explicit High/Low preferences are retained.</p>{#if settings&&!settings.graphicsPresets}<p class="lt-error">Update the running controller before changing graphics presets. Existing games are preserved.</p>{/if}{#if settingsNotice}<p role="status">{settingsNotice}</p>{/if}
          </section>
          <section class="lt-group"><h2>Frame Limits <small>All launchers</small></h2>{#if settings}<form on:submit|preventDefault={onRequestPerformanceSave}>
            <div class="lt-setting"><div><h3><label for="active-fps">While playing</label></h3><p>The game receiving keyboard and mouse input.</p></div><div class="lt-fps"><input id="active-fps" type="number" min="5" max="1000" required bind:value={activeFps} disabled={busy||performanceQueued}/> FPS</div></div>
            <div class="lt-setting"><div><h3><label for="background-fps">In the background</label></h3><p>Other games keep running at this limit.</p></div><div class="lt-fps"><input id="background-fps" type="number" min="5" max={activeFps??1000} required bind:value={backgroundFps} disabled={busy||performanceQueued}/> FPS</div></div>
            <p class="lt-help">Saved limits apply at the next launch. Running games keep their current limits.</p>{#if !performanceValid}<p class="lt-error" role="alert">Use whole numbers from 5 to 1000. Background FPS cannot exceed active FPS.</p>{/if}
            <div class="lt-actions"><button type="submit" disabled={busy||performanceQueued||!available||!performanceDirty||!performanceValid}>{savingPerformance||performanceQueued?'Saving…':'Save changes'}</button>{#if performanceDirty}<button type="button" class="lt-secondary" disabled={busy||performanceQueued} on:click={()=>{activeFps=settings!.activeFps;backgroundFps=settings!.backgroundFps;}}>Discard</button>{/if}<button type="button" class="lt-secondary" disabled={busy||performanceQueued} on:click={()=>{activeFps=120;backgroundFps=5;}}>Use 120 / 5</button></div>{#if performanceNotice}<p role="status">{performanceNotice}</p>{/if}
          </form>{:else}<p>Settings unavailable. Refresh to try again.</p>{/if}</section>
          <section class="lt-group"><h2>App behavior</h2><div class="lt-setting"><div><h3>When Lazy Tester closes</h3><p>All Minecraft sessions started by Lazy Tester close normally first. Lazy Tester waits for them to finish before exiting.</p></div></div><p class="lt-help">If Minecraft is waiting for a dialog or has not finished closing, Lazy Tester stays open. Other Minecraft applications are left alone.</p><div class="lt-setting"><div><h3>Background game audio</h3><p>Mute games that are not receiving keyboard and mouse input. Existing user mute and volume settings are kept.</p>{#if settings?.audioIssue}<p class="lt-error" role="status">{userMessage('audio')}</p>{/if}{#if !settings?.backgroundAudio}<p>Update the running controller to enable audio control.</p>{/if}</div><label class="lt-audio-toggle"><input type="checkbox" checked={settings?.backgroundAudio==='Mute'} disabled={busy||audioQueued||!settings?.backgroundAudio} aria-label="Mute background games" on:change={event=>onSaveAudio(event.currentTarget.checked?'Mute':'Keep')}/>Mute background games</label></div></section>
          <section class="lt-group"><h2>Installation & data</h2><h3>Minecraft Education</h3><p>The controller verifies the registered game executable before launch. Moving data does not move Minecraft.</p><h3>Launcher data location</h3><p class="lt-data-path">{dataLocation?.path??'No data folder connected.'}</p>{#if locationIssue}<p class="lt-error">{locationIssue}</p>{/if}<div class="lt-actions"><button class="lt-secondary" disabled={busy||refreshing||resizing||!dataLocation?.path} on:click={onOpenDataFolder}>Open data folder</button><button class="lt-secondary" on:click={onShowDataStorage}>Data Storage →</button></div></section>
        </div>
