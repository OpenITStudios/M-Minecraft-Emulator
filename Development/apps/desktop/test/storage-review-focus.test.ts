import {readFileSync} from 'node:fs';
import {describe, expect, it} from 'vitest';

const source = (name:string)=>readFileSync(new URL('../src/'+name, import.meta.url), 'utf8');

describe('Data Storage keyboard review source contract', ()=>{
  it('opens existing cache, settings and full-clear reviews without new mutation owners', ()=>{
    const storage=source('DataStorage.svelte');
    const app=source('LazyTesterApp.svelte');
    expect(storage).toContain('function showCacheReview(trigger: HTMLButtonElement)');
    expect(storage).toContain('function showSettingsReview(action: StorageSettingsAction, trigger: HTMLButtonElement)');
    expect(storage).toContain('function showFullReview(trigger: HTMLButtonElement)');
    expect(storage).toContain('onReviewCache();');
    expect(storage).toContain('onReviewSettings(action);');
    expect(storage).toContain('onReviewFullClear();');
    expect(storage).not.toContain('invokeRuntime(');
    expect(app).toContain('async function clearCache(slot: ProfileSlot)');
    expect(app).toContain('async function recoverSettings()');
    expect(app).toContain('async function clearAllData()');
    expect(app).toContain('confirmationToken!==pending.token');
  });

  it('defaults focus to safe cache/settings Cancel and preserves full-clear identity entry', ()=>{
    const storage=source('DataStorage.svelte');
    const app=source('LazyTesterApp.svelte');
    expect(storage).toContain('cacheCancelButton.focus({preventScroll: true})');
    expect(storage).toContain('settingsCancelButton.focus({preventScroll: true})');
    expect(storage).toContain('bind:this={cacheReviewElement}');
    expect(storage).toContain('bind:this={settingsReviewElement}');
    expect(storage).toContain('bind:this={fullReviewElement}');
    expect(app).toContain("document.getElementById('full-clear-name')?.focus()");
    expect(storage).toContain('bind:this={cacheCancelButton}');
    expect(storage).toContain('bind:this={settingsCancelButton}');
  });

  it('only consumes Escape from inside an active storage review and restores safe keyboard context', ()=>{
    const storage=source('DataStorage.svelte');
    expect(storage).toContain('<svelte:window on:keydown={onReviewEscape}/>');
    expect(storage).toContain("event.key !== 'Escape' || busy || refreshing || resizing");
    expect(storage).toContain('fullReviewElement?.contains(target)');
    expect(storage).toContain('settingsReviewElement?.contains(target)');
    expect(storage).toContain('cacheReviewElement?.contains(target)');
    expect(storage).toContain('if (!cancel) return;');
    expect(storage).toContain('if (trigger?.isConnected && !trigger.disabled) trigger.focus({preventScroll: true})');
    expect(storage).toContain('cancelReview(onCancelCacheReview)');
    expect(storage).toContain('cancelReview(onCancelSettingsReview)');
    expect(storage).toContain('cancelReview(onCancelFullReview)');
  });

  it('uses contextual explanations for unavailable settings without weakening gates', ()=>{
    const storage=source('DataStorage.svelte');
    const settings=source('GeneralSettings.svelte');
    expect(storage).toContain('!selectedStorage.gameSettings.canReset && !selectedStorage.gameSettings.canRestore');
    expect(settings).toContain('!settings.canApplyPreset && runningCount===0');
    expect(settings).toContain('performanceDirty && !available');
    expect(storage).toContain('disabled={busy||refreshing||resizing||storageReviewOpen||!available||!selectedStorage?.cache.canClear}');
  });
});
