import {readFileSync} from 'node:fs';
import {describe,expect,it} from 'vitest';

const load=(name:string)=>readFileSync(new URL('../src/'+name,import.meta.url),'utf8');
const browser=()=>readFileSync(
  new URL('../../../tooling/acceptance/lazy-tester-ui-regression.mjs',import.meta.url),'utf8');

describe('Lazy Tester Settings/Storage transient focus source contract',()=>{
  it('anchors Graphics save to the existing section only when the control owned focus',()=>{
    const settings=load('GeneralSettings.svelte');
    const app=load('LazyTesterApp.svelte');
    expect(settings).toContain('let graphicsHeading: HTMLHeadingElement | undefined;');
    expect(settings).toContain('document.activeElement === event.currentTarget');
    expect(settings).toContain('graphicsHeading?.focus({preventScroll:true})');
    expect(settings).toContain('onRequestPreset();');
    expect(settings).toContain('on:click={saveGraphicsWithFocus}');
    expect(settings).toContain('bind:this={graphicsHeading} tabindex="-1"');
    expect(app).toContain('queuedPreset={slot:selectedSlot,preset:graphicsPreset}');
    expect(app).toContain('await lazyTester.applyGraphics(request.slot,request.preset)');
    expect(app).toContain('graphicsDrafts.set(request.slot,request.preset)');
  });

  it('anchors valid frame-limit submission without rewriting the parent save gates',()=>{
    const settings=load('GeneralSettings.svelte');
    const app=load('LazyTesterApp.svelte');
    expect(settings).toContain('function savePerformanceWithFocus(event: SubmitEvent): void');
    expect(settings).toContain('form instanceof HTMLFormElement');
    expect(settings).toContain('form.contains(document.activeElement)');
    expect(settings).toContain('frameHeading?.focus({preventScroll:true})');
    expect(settings).toContain('onRequestPerformanceSave();');
    expect(settings).toContain('on:submit|preventDefault={savePerformanceWithFocus}');
    expect(settings).toContain('bind:this={frameHeading} tabindex="-1"');
    expect(app).toContain('validFpsLimits(activeFps,backgroundFps)');
    expect(app).toContain('await lazyTester.savePerformance(limits.active,limits.background)');
  });

  it('keeps background audio focused context while preserving original native admission',()=>{
    const settings=load('GeneralSettings.svelte');
    const app=load('LazyTesterApp.svelte');
    expect(settings).toContain('async function changeAudioWithFocus(event: Event): Promise<void>');
    expect(settings).toContain('input instanceof HTMLInputElement');
    expect(settings).toContain('document.activeElement === input');
    expect(settings).toContain('audioHeading?.focus({preventScroll:true})');
    expect(settings).toContain('await onSaveAudio(intended)');
    expect(settings).toContain("input.checked = settings?.backgroundAudio === 'Mute'");
    expect(settings).toContain('on:change={changeAudioWithFocus}');
    expect(settings).toContain('bind:this={audioHeading} tabindex="-1"');
    expect(app).toContain("await lazyTester.saveAudio(mode)");
    expect(settings).toContain('disabled={busy||audioQueued||refreshing||resizing||!available||!settings?.backgroundAudio}');
  });

  it('retains Storage cache/settings review semantics and protects full-clear admission',()=>{
    const storage=load('DataStorage.svelte');
    const app=load('LazyTesterApp.svelte');
    expect(storage).toContain('function commitCacheReview(slot: ProfileSlot): void');
    expect(storage).toContain('function commitSettingsReview(): void');
    expect(storage).toContain('cacheReviewElement?.contains(document.activeElement)');
    expect(storage).toContain('settingsReviewElement?.contains(document.activeElement)');
    expect(storage).toContain('storageOverviewHeading?.focus({preventScroll:true})');
    expect(storage).toContain('onClearCache(slot);');
    expect(storage).toContain('onRecoverSettings();');
    expect(storage).toContain('on:click={()=>commitCacheReview(selectedSlot)}');
    expect(storage).toContain('on:click={commitSettingsReview}');
    expect(storage).toContain('bind:this={storageOverviewHeading} tabindex="-1"');
    expect(storage).toContain('cancelReview(onCancelCacheReview)');
    expect(storage).toContain('cancelReview(onCancelSettingsReview)');
    expect(app).toContain('await lazyTester.clearCache(slot)');
    expect(app).toContain('status.confirmationToken!==pending.token');
    expect(storage).toContain('on:submit|preventDefault={onClearAllData}');
    expect(storage).not.toContain('invokeRuntime(');
  });

  it('keeps focus rings and separate native DWM geometry unchanged',()=>{
    const css=load('lazy-tester.css');
    const script=browser();
    expect(css).toContain('#lt-graphics-section,');
    expect(css).toContain('#lt-frame-limits-section,');
    expect(css).toContain('#lt-storage-overview-section,');
    expect(css).toContain('.lt-group h3[tabindex="-1"]');
    expect(css).toContain('outline-color:Highlight;');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
    expect(script).toContain('A completed FPS limits save must retain focus on its persistent Frame Limits heading');
    expect(script).toContain('A completed graphics preset save must retain focus on Graphics');
    expect(script).toContain('Cancel must still return focus to the original cache review trigger');
    expect(script).not.toContain("await fullReview.getByRole('button',{name:'Permanently clear data'}).click()");
  });
});
