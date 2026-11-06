import {readFileSync} from 'node:fs';
import {describe,expect,it} from 'vitest';

const load=(path:string)=>readFileSync(new URL('../src/'+path,import.meta.url),'utf8');

describe('General / Data Storage navigation UX — source contracts',()=>{
  it('provides an explicitly cancelable, per-profile graphics draft without native calls',()=>{
    const app=load('LazyTesterApp.svelte');
    const settings=load('GeneralSettings.svelte');
    expect(app).toContain('function discardGraphicsDraft(): void');
    expect(app).toContain('graphicsDrafts.delete(selectedSlot)');
    expect(app).toContain("settings?.graphicsPresets?.[selectedSlot - 1] ?? 'Preserve'");
    expect(app).toContain('onDiscardGraphics={discardGraphicsDraft}');
    expect(settings).toContain('on:click={onDiscardGraphics}>Discard draft</button>');
    expect(settings).toContain('{#if graphicsDirty}<button');
    expect(settings).not.toContain('invokeRuntime(');
  });
  it('moves screen-reader focus only to Settings / Storage headings on explicit navigation',()=>{
    const app=load('LazyTesterApp.svelte');
    expect(app).toContain('if (next !== \'workspace\' && !disposed && pageHeading?.isConnected)');
    expect(app).toContain('pageHeading.focus({preventScroll:true})');
    expect(app).toContain("tabindex={page==='workspace'?undefined:-1}");
    expect(app).toContain('loadingPage = next;');
    expect(app).toContain('finally { loadingPage = null; busy = false; }');
    expect(app).toContain('Checking saved profile storage…');
    expect(app).toContain('Checking current settings…');
    expect(app).toContain('role="status" aria-live="polite"');
  });
  it('retains all original data-operation guards while keeping profile review local',()=>{
    const app=load('LazyTesterApp.svelte');
    const storage=load('DataStorage.svelte');
    expect(storage).toContain('disabled={busy||clearSlot!==null||recovery!==null||fullReview!==null}');
    expect(storage).toContain("aria-describedby={clearSlot!==null||recovery!==null||fullReview!==null?'lt-storage-review-guidance':undefined}");
    expect(storage).toContain('Finish or cancel the current review');
    expect(storage).toContain('on:submit|preventDefault={onClearAllData}');
    expect(app).toContain('status.confirmationToken!==pending.token');
    expect(storage).not.toContain('invokeRuntime(');
  });
  it('wraps only settings and storage rows without changing the game viewport',()=>{
    const css=load('lazy-tester.css');
    expect(css).toContain('@media (max-width:1050px)');
    expect(css).toContain('.lt-settings-layout .lt-setting{flex-wrap:wrap;');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
  });
});
