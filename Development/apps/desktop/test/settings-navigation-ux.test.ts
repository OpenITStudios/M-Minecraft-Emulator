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
    expect(app).toContain('Preserve Visuals saved. Future launches leave existing graphics unchanged.');
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
  it('retains readable support and settings copy without retired game HUD selectors',()=>{
    const css=load('lazy-tester.css');
    expect(css).toContain('.lt-agent-activity-details p{margin:8px 0;font-size:13px;');
    expect(css).toContain('.lt-settings-layout .lt-setting{flex-wrap:wrap;');
    expect(css).not.toContain('.lt-coordinate-popup');
    expect(css).not.toContain('.lt-f3-popup');
  });
  it('keeps Data Storage navigation only in the Settings sidebar',()=>{
    const settings=load('GeneralSettings.svelte');
    const app=load('LazyTesterApp.svelte');
    expect(settings).not.toContain('lt-install-data-section');
    expect(settings).not.toContain('onOpenDataFolder');
    expect(settings).not.toContain('onShowDataStorage');
    expect(settings).not.toContain('dataLocation?.path');
    expect(app).toContain('on:click={()=>navigate(\'storage\')}>Data Storage</button>');
    expect(app).not.toContain('onShowDataStorage={() => navigate(\'storage\')}');
    expect(settings).toContain("?'Saving…':'Save FPS limits'");
    expect(settings).toContain("?'Saving…':'Save graphics preference'");
  });
  it('does not distract General Settings with a redundant workspace mode selector',()=>{
    const settings=load('GeneralSettings.svelte');
    const app=load('LazyTesterApp.svelte');
    expect(settings).toContain('<div class="lt-settings-content">\n          <section class="lt-group" aria-labelledby="lt-graphics-section">');
    expect(settings).not.toContain('workspace-layout');
    expect(settings).not.toContain('onChooseWorkspaceLayout');
    expect(app).not.toContain('workspaceLayout');
    expect(settings).not.toContain('invokeRuntime(');
  });
  it('explains unfamiliar graphics and FPS options without changing their native values',()=>{
    const general=load('GeneralSettings.svelte');
    const storage=load('DataStorage.svelte');
    expect(general).toContain('<option value="Preserve">Preserve Visuals (recommended)</option>');
    expect(general).toContain('About legacy graphics presets');
    expect(general).toContain('both set 12 chunks and MSAA 1');
    expect(general).toContain('Neither guarantees higher FPS.');
    expect(general).toContain('FPS limits: 5–1000.');
    expect(general).toContain('These are caps, not measured FPS; they apply at the next launch.');
    expect(storage).toContain('<summary>Find existing profile data</summary>');
    expect(storage).toContain('<strong>Advanced recovery</strong>');
    expect(general).toContain('Save graphics preference');
    expect(general).toContain('Save FPS limits');
    expect(general).not.toContain('Use 120 / 5');
  });
  it('wraps only settings and storage rows without changing the game viewport',()=>{
    const css=load('lazy-tester.css');
    expect(css).toContain('@media (max-width:1050px)');
    expect(css).toContain('.lt-settings-layout .lt-setting{flex-wrap:wrap;');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
  });
});
