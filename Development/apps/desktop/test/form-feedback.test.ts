import {readFileSync} from 'node:fs';
import {describe, expect, it} from 'vitest';

const source=(name:string)=>readFileSync(new URL('../src/'+name,import.meta.url),'utf8');

describe('Lazy Tester Settings / Storage field feedback contracts',()=>{
  it('describes each invalid FPS field but retains the original Save admission',()=>{
    const settings=source('GeneralSettings.svelte');
    const app=source('LazyTesterApp.svelte');
    expect(settings).toContain('activeFpsFieldInvalid');
    expect(settings).toContain('backgroundFpsFieldInvalid');
    expect(settings).toContain('aria-invalid={activeFpsFieldInvalid}');
    expect(settings).toContain('aria-invalid={backgroundFpsFieldInvalid}');
    expect(settings).toContain('lt-active-fps-error');
    expect(settings).toContain('lt-background-fps-error');
    expect(settings).toContain('role="status" aria-live="polite"');
    expect(settings).toContain('!performanceDirty||!performanceValid');
    expect(app).toContain('performanceValid=validFpsLimits(activeFps,backgroundFps)');
    expect(app).toContain('await lazyTester.savePerformance(limits.active,limits.background)');
    expect(settings).not.toMatch(/\binvokeRuntime\s*\(|\blazyTester\s*\.\w+\(/);
  });
  it('preserves native confirmation name, acknowledgement and token gates',()=>{
    const storage=source('DataStorage.svelte');
    const app=source('LazyTesterApp.svelte');
    expect(storage).toContain('typedFullClearNameIncorrect');
    expect(storage).toContain('aria-invalid={typedFullClearNameIncorrect}');
    expect(storage).toContain('lt-full-clear-hint lt-full-clear-error');
    expect(storage).toContain('No deletion has been requested.');
    expect(storage).toContain('lt-full-clear-ack-hint');
    expect(storage).toContain('!fullAck||fullName!==`Minecraft - ${fullReview.slot}`');
    expect(storage).toContain('confirmationToken!==fullReview.token');
    expect(app).toContain('fullName!==`Minecraft - ${pending.slot}`');
    expect(app).toContain('status.confirmationToken!==pending.token');
    expect(storage).toContain('on:submit|preventDefault={onClearAllData}');
    expect(storage).not.toMatch(/\binvokeRuntime\s*\(|\blazyTester\s*\.\w+\(/);
  });
  it('distinguishes unavailable measurements from an empty profile',()=>{
    const storage=source('DataStorage.svelte');
    expect(storage).toContain('storage===null');
    expect(storage).toContain('!selectedStorage');
    expect(storage).toContain('an unmeasured size does not mean a profile is empty');
    expect(storage).toContain('These sizes are from the last storage report, not a live measurement.');
    expect(storage).toContain('Last storage report received:');
    expect(storage).toContain('formatStorageExtent(selectedStorage.bytes,selectedStorage.partial)');
    expect(storage).toContain('formatStorageExtent(selectedStorage.cache.bytes,selectedStorage.cache.partial)');
    expect(storage).toContain('formatStorageBytes(selectedStorage.disk.availableBytes)');
    expect(storage).not.toContain('(selectedStorage.bytes/1048576).toFixed(1)');
    expect(storage).not.toContain('Last measured at');
  });
  it('keeps Settings save scopes clear and local data management in one destination',()=>{
    const app=source('LazyTesterApp.svelte');
    const settings=source('GeneralSettings.svelte');
    const storage=source('DataStorage.svelte');
    expect(settings).toContain('Save graphics preference');
    expect(settings).toContain('Save FPS limits');
    expect(settings).toContain('Applies to');
    expect(settings).toContain('All launchers');
    expect(settings).toContain('Save graphics preference');
    expect(settings).not.toContain('Use 120 / 5');
    expect(settings).not.toContain('onOpenDataFolder');
    expect(settings).not.toContain('dataLocation?.path');
    expect(settings).toContain('onShowDataStorage}>Open Data Storage</button>');
    expect(app).toContain('storageReportReceivedAt=storageReportReceivedTime()');
    expect(storage).toContain('storageReportReceivedAt');
    expect(storage).not.toContain('storageCheckedAt');
    expect(settings).not.toContain('onChooseWorkspaceLayout');
    expect(app).toContain('onRequestClose={requestCloseOne} onGuidedRecovery={requestGuidedRecovery}');
  });
  it('keeps full data deletion after connect/locate and recovery without bypassing its confirmation',()=>{
    const storage=source('DataStorage.svelte');
    const location=storage.indexOf('<details class="lt-location-options">');
    const packs=storage.indexOf('<details class="lt-storage-packs">');
    const advanced=storage.indexOf('<details class="lt-advanced"');
    const danger=storage.indexOf('class="lt-danger-zone"');
    expect(location).toBeGreaterThan(-1);
    expect(packs).toBeGreaterThan(location);
    expect(advanced).toBeGreaterThan(packs);
    expect(danger).toBeGreaterThan(advanced);
    expect(storage).toContain('confirmationToken!==fullReview.token');
    expect(storage).toContain('on:submit|preventDefault={onClearAllData}');
    expect(storage).toContain('!fullAck||fullName!==`Minecraft - ${fullReview.slot}`');
  });
  it('keeps responsive rules scoped away from the five native game surfaces',()=>{
    const css=source('lazy-tester.css');
    const app=source('LazyTesterApp.svelte');
    expect(css).toContain('.lt-settings-layout .lt-danger-zone{flex-wrap:wrap;');
    expect(css).toContain('.lt-settings-layout .lt-field-error');
    expect(css).toContain('.lt-settings-layout input[aria-invalid="true"]');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
    expect(app).toContain('onSelect={select} onLaunch={launch}');
  });
});
