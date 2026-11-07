import {readFileSync} from 'node:fs';
import {describe, expect, it} from 'vitest';
import {projectF3Inspector} from '../src/app/presentation/f3Inspector.js';
import type {ProfileSession, ProfileSlot, SettingsReport} from '../src/app/bridge/lazyTester.js';

const profile = (slot: ProfileSlot = 2): ProfileSession => ({
  slot, state: 'RUNNING', issue: null,
  process: {pid: 232, createdFileTime: '134000000000000232',
    sessionId: 1, windowHandle: '0x123', responding: true},
});
const settings: SettingsReport = {
  activeFps: 144, backgroundFps: 10, renderDistanceChunks: 12,
  canApplyPreset: false, changedProfiles: null,
};

describe('F3 Inspector native-only projection (no Minecraft source)', () => {
  it('never borrows a background game when Play has no selected client', () => {
    const v = projectF3Inspector({
      selectedSlot: null, profile: profile(), currentSelection: true, actionPending: false, settings,
    });
    expect(v.sourceStatus).toBe('NO_SELECTION');
    expect(v.clientLabel).toBe('No selected client');
    expect(v.processId).toBe('—');
    expect(v.configuredFps).toBe('Not available');
    expect(v.coordinates).toBe('Not connected');
  });

  it('removes PID and other native observations if selection is stale or slot mismatched', () => {
    for (const record of [
      {selectedSlot: 2 as ProfileSlot, profile: profile(), currentSelection: false, actionPending: false, settings},
      {selectedSlot: 3 as ProfileSlot, profile: profile(), currentSelection: true, actionPending: false, settings},
      {selectedSlot: 2 as ProfileSlot, profile: undefined, currentSelection: true, actionPending: false, settings},
    ]) {
      const v = projectF3Inspector(record);
      expect(v.sourceStatus).toBe('NOT_CURRENT');
      expect(v.processId).toBe('—');
      expect(v.nativeWindow).toBe('Unavailable');
      expect(v.configuredFps).toBe('Not available');
    }
  });

  it('withholds the former game on queued selection/launch/close even if its PID still matches', () => {
    const pending = projectF3Inspector({
      selectedSlot: 2, profile: profile(), currentSelection: true,
      actionPending: true, settings,
    });
    expect(pending.sourceStatus).toBe('ACTION_PENDING');
    expect(pending.processId).toBe('—');
    expect(pending.windowsSession).toBe('—');
    expect(pending.nativeWindow).toBe('Unavailable');
    expect(pending.configuredFps).toBe('Not available');
    expect(pending.summary).toContain('Launcher action pending');
    const noSelection = projectF3Inspector({
      selectedSlot: null, profile: profile(), currentSelection: false,
      actionPending: true, settings,
    });
    expect(noSelection.sourceStatus).toBe('ACTION_PENDING');
    expect(noSelection.processId).toBe('—');
  });

  it('labels last saved FPS targets separately from observed per-client FPS', () => {
    const p = projectF3Inspector({
      selectedSlot: 2, profile: profile(), currentSelection: true,
      actionPending: false, settings,
    });
    expect(p.configuredFps).toContain('Last reported targets');
    expect(p.configuredFps).toContain('not measured or confirmed for this game');
    expect(p.measuredFps).toBe('Not exposed to desktop');
    const noSettings = projectF3Inspector({
      selectedSlot: 2, profile: profile(), currentSelection: true,
      actionPending: false, settings: null,
    });
    expect(noSettings.configuredFps).toBe('Not available from current settings');
  });

  it('shows exact selected client native process with configured (not measured) FPS', () => {
    const v = projectF3Inspector({
      selectedSlot: 2, profile: profile(), currentSelection: true, actionPending: false, settings,
    });
    expect(v).toMatchObject({
      sourceStatus: 'NATIVE_PROCESS_OBSERVED', clientLabel: 'MC02',
      processId: '232', windowsSession: '1',
      configuredFps: 'Last reported targets: active 144 / background 10 FPS (not measured or confirmed for this game)',
      measuredFps: 'Not exposed to desktop', coordinates: 'Not connected',
      rotation: 'Not connected', dimension: 'Not connected',
    });
    expect(v.summary).toContain('NOT verified');
  });

  it('does not call a nonresponding game ready or read a missing game process', () => {
    const down = profile();
    down.state = 'NOT_RESPONDING';
    down.process = {...down.process!, responding: false};
    const v = projectF3Inspector({selectedSlot: 2, profile: down, currentSelection: true, actionPending: false, settings: null});
    expect(v.nativeState).toBe('Not responding');
    expect(v.nativeWindow).toBe('Not responding (Windows observation)');
    expect(v.measuredFps).toBe('Not exposed to desktop');
    const dead = profile();
    dead.state = 'STOPPED';
    dead.process = null;
    expect(projectF3Inspector({selectedSlot: 2, profile: dead, currentSelection: true, actionPending: false, settings})
      .processId).toBe('—');
  });
});

const app = readFileSync(new URL('../src/LazyTesterApp.svelte', import.meta.url), 'utf8');
const ui = readFileSync(new URL('../src/F3InspectorPanel.svelte', import.meta.url), 'utf8');
const css = readFileSync(new URL('../src/lazy-tester.css', import.meta.url), 'utf8');

describe('F3 Inspector presentation-only integration', () => {
  it('uses the original selected Play process identity and existing session mirror', () => {
    expect(app).toContain("import F3InspectorPanel from './F3InspectorPanel.svelte';");
    expect(app).toContain("import {projectF3Inspector} from './app/presentation/f3Inspector.js';");
    expect(app).toContain('selectedIdentity === identity(profiles.find(profile => profile.slot === active))');
    expect(app).toContain('!uncertainWindowSlots.has(active)');
    expect(app).toContain('actionPending: f3ActionPending,');
    expect(app).toContain('!f3ActionPending && !closeInProgress');
    expect(app).toContain('busy || closeReviewPending || queued !== null');
    expect(app).toContain('queuedPage !== null || queuedLaunch !== null || requested !== null');
    expect(app).toContain('manualRefreshRequested || manualRefreshing;');
    expect(app).toContain("<F3InspectorPanel view={f3View} />");
    expect(app).toContain("<CoordinatePastePanel selectedSlot={active} />");
    expect(app).toContain('class="lt-play-tools"');
    expect(app).toContain('<LauncherRail {profiles} {labels} {healthIssues}');
    expect(css).toContain('.lt-f3-data{');
  });

  it('does not pretend to capture player XYZ, execute F3, read clipboard or create a collector', () => {
    for (const forbidden of ['navigator.clipboard', 'invokeRuntime(', 'runCommand(',
      'setInterval(', 'addEventListener(', 'fetch(', 'localStorage', 'writeText(']) {
      expect(ui).not.toContain(forbidden);
    }
    expect(ui).toContain('{view.coordinates}');
    expect(ui).toContain('Show Coordinates');
    expect(ui).toContain('FPS targets (last report)');
    expect(ui).toContain('F3 Inspector');
    expect(ui).toContain('<svelte:window on:keydown={closeOnEscape} on:blur={closeOnWindowBlur} on:resize={repositionPopup} on:scroll|capture={repositionPopup} />');
    expect(ui).toContain('name="lt-play-disclosures"');
    expect(ui).toContain('on:focusout={closeWhenFocusLeaves}');
    expect(ui).toContain('style={popupStyle}');
  });
});
