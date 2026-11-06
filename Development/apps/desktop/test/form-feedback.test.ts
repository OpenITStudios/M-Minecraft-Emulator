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
    expect(storage).toContain('The sizes below are from the last storage inspection');
    expect(storage).toContain('Last measured at');
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
