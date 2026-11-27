import {readFileSync} from 'node:fs';
import {describe,expect,it} from 'vitest';

const load=(path:string)=>readFileSync(new URL('../src/'+path,import.meta.url),'utf8');
const chooseGuard=(app:string)=>app.split('function chooseSlot(slot:ProfileSlot){')[1]?.split('selectedSlot=slot;')[0]??'';

describe('General / Data Storage navigation UX — source contracts',()=>{
  it('keeps graphics automatic without exposing a setting',()=>{
    const app=load('LazyTesterApp.svelte'), settings=load('GeneralSettings.svelte');
    expect(app).not.toContain('graphicsDrafts');
    expect(app).not.toContain('queuedPreset');
    expect(settings).not.toContain('graphics-preset');
    expect(settings).not.toContain('lazytester-graphics-policy');
  });
  it('keeps profile-specific Storage receipts and pending reviews scoped to the original slot',()=>{
    const app=load('LazyTesterApp.svelte');
    const choose=app.split('function chooseSlot(slot:ProfileSlot){')[1]
      ?.split('  let savingPerformance=')[0] ?? '';
    // The parent rejects unexpected selector events even if the child
    // controls were disabled before the pending review began.
    expect(choose).toContain("closeReviewActive(reviewState()) || activeOperation === 'clearData' ||");
    expect(choose).toContain("activeOperation === 'clearData' || storageReview!==null) return;");
    expect(choose).not.toMatch(/storageReview\s*=\s*null/);
    expect(choose).toContain("settingsNotice='';storageNotice='';");
  });
  it('moves screen-reader focus only to Settings / Storage headings on explicit navigation',()=>{
    const app=load('LazyTesterApp.svelte');
    expect(app).toContain("if (next !== 'workspace' && pageHeading?.isConnected)");
    expect(app).toContain('page = next; await tick();\n      if (!current()) return;');
    expect(app).toContain('pageHeading.focus({preventScroll:true})');
    expect(app).toContain('<h1 bind:this={pageHeading} tabindex="-1">');
    expect(app).toContain('loadingPage = next;');
    expect(app).toContain('finally { loadingPage = null; activeOperation = null; }');
    expect(app).toContain('Checking saved profile storage…');
    expect(app).toContain('Checking current settings…');
    expect(app).toContain('role="status" aria-live="polite"');
  });
  it('retains all original data-operation guards while keeping profile review local',()=>{
    const app=load('LazyTesterApp.svelte');
    const storage=load('DataStorage.svelte');
    expect(storage).toContain('disabled={busy||storageReviewOpen}');
    expect(chooseGuard(app)).toContain('selectiveOpen');
    expect(storage).toContain("aria-describedby={storageReviewOpen?'lazytester-storage-review-guidance':undefined}");
    expect(storage).toContain('Finish or cancel this review before switching profiles.');
    expect(storage).toContain('on:submit|preventDefault={onClearAllData}');
    expect(app).toContain('status.confirmationToken!==pending.token');
    expect(storage).not.toContain('invokeRuntime(');
  });
  it('retains readable support and settings copy without retired game HUD selectors',()=>{
    const css=load('lazytester.css');
    expect(css).toContain('.lazytester-agent-activity-details p{margin:8px 0;font-size:13px;');
    expect(css).toContain('.lazytester-settings-layout .lazytester-setting{flex-wrap:wrap;');
    expect(css).not.toContain('.lazytester-coordinate-popup');
    expect(css).not.toContain('.lazytester-f3-popup');
  });
  it('keeps Data Storage navigation only in the Settings sidebar',()=>{
    const settings=load('GeneralSettings.svelte');
    const app=load('LazyTesterApp.svelte');
    expect(settings).not.toContain('lazytester-install-data-section');
    expect(settings).not.toContain('onOpenDataFolder');
    expect(settings).not.toContain('onShowDataStorage');
    expect(settings).not.toContain('dataLocation?.path');
    expect(app).toContain('on:click={()=>navigate(\'storage\')}>Data Storage</button>');
    expect(app).not.toContain('onShowDataStorage={() => navigate(\'storage\')}');
    expect(settings).toContain("?'Saving…':'Save Changes'");
  });
  it('places the approved workspace choice before frame rate',()=>{
    const settings=load('GeneralSettings.svelte');
    const app=load('LazyTesterApp.svelte');
    expect(settings.indexOf('id="workspace-layout"')).toBeLessThan(settings.indexOf('id="lazytester-frame-limits-section"'));
    expect(settings).not.toContain('Legacy graphics preferences');
    expect(settings).not.toContain('When LazyTester closes');
    expect(settings).toContain('workspace-layout');
    expect(settings).toContain('onChooseWorkspaceLayout');
    expect(app).toContain('workspaceLayout');
    expect(settings).not.toContain('invokeRuntime(');
  });
  it('explains FPS limits without a graphics setting',()=>{
    const general=load('GeneralSettings.svelte');
    const storage=load('DataStorage.svelte');
    expect(general).toContain('FPS limits: 5–1000.');
    expect(general).toContain('These are limits, not measured FPS.');
    expect(storage).toContain('<summary>Find existing profile data</summary>');
    expect(storage).toContain('<strong>Advanced recovery</strong>');
    expect(general).toContain('Save Changes');
    expect(general).not.toContain('Use 120 / 5');
  });
  it('wraps only settings and storage rows without changing the game viewport',()=>{
    const css=load('lazytester.css');
    expect(css).toContain('@media (max-width:1050px)');
    expect(css).toContain('.lazytester-settings-layout .lazytester-setting{flex-wrap:wrap;');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
  });
});
