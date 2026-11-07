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
  it('keeps a user graphics draft if its native save receipt is unconfirmed',()=>{
    const app=load('LazyTesterApp.svelte');
    const apply=app.split('async function applyPreset(')[1]
      ?.split('  function requestPerformanceSave()')[0] ?? '';
    expect(apply).toContain('graphicsDrafts.delete(request.slot)');
    expect(apply).toContain('graphicsDrafts.set(request.slot,request.preset)');
    expect(apply).toContain("issueKey='settings';settings=null;");
    expect(apply).toContain("settings.changedProfiles===null");
    expect(app).toContain("if (page==='settings') rememberGraphicsDraft()");
    expect(app).toContain("graphicsPreset=graphicsChoice(selectedSlot)");
    expect(app).toContain('Lazy Tester will leave Minecraft graphics options unchanged');
  });
  it('moves screen-reader focus only to Settings / Storage headings on explicit navigation',()=>{
    const app=load('LazyTesterApp.svelte');
    expect(app).toContain('if (next !== \'workspace\' && !disposed && pageHeading?.isConnected)');
    expect(app).toContain('pageHeading.focus({preventScroll:true})');
    expect(app).toContain('<h1 bind:this={pageHeading} tabindex="-1">');
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
  it('retains readable Play tool copy at the original component rules',()=>{
    const css=load('lazy-tester.css');
    expect(css).toContain('.lt-coordinate-popup .lt-coordinate-help{font-size:13px;');
    expect(css).toContain('.lt-f3-popup .lt-f3-disclaimer{font-size:13px;');
    expect(css).toContain('.lt-coordinate-saved-header button{font-size:12px;');
    expect(css).toContain('.lt-coordinate-saved-buttons button{font-size:12px;');
    expect(css).not.toContain('.lt-coordinate-popup .lt-coordinate-help,\n.lt-f3-popup .lt-f3-disclaimer,');
  });
  it('offers one existing General configuration selector for the workspace, not gameplay settings',()=>{
    const settings=load('GeneralSettings.svelte');
    const app=load('LazyTesterApp.svelte');
    expect(settings).toContain('aria-labelledby="lt-workspace-layout-section"');
    expect(settings).toContain('for="workspace-layout"');
    expect(settings).toContain('aria-describedby="lt-workspace-layout-help"');
    expect(settings).toContain("onChooseWorkspaceLayout(event.currentTarget.value as 'focus' | 'monitor')");
    expect(app).toContain('{workspaceLayout} {layoutPersistenceIssue} onChooseWorkspaceLayout={chooseWorkspaceLayout}');
    expect(settings).not.toContain('invokeRuntime(');
  });
  it('wraps only settings and storage rows without changing the game viewport',()=>{
    const css=load('lazy-tester.css');
    expect(css).toContain('@media (max-width:1050px)');
    expect(css).toContain('.lt-settings-layout .lt-setting{flex-wrap:wrap;');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
  });
});
