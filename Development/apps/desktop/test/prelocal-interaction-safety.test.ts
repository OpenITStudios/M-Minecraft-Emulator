import {readFileSync} from 'node:fs';
import {describe,it,expect} from 'vitest';

const ui=(file:string)=>readFileSync(new URL('../src/'+file,import.meta.url),'utf8');
const browser=()=>readFileSync(new URL('../../../tooling/acceptance/lazy-tester-ui-regression.mjs',import.meta.url),'utf8');

// Source contracts are AUTHORED_NOT_EXECUTED until approved local execution.
describe('Pre-localtest interaction admission and storage review safety',()=>{
  it('never drops an open CloseOne review through another launcher action',()=>{
    const app=ui('LazyTesterApp.svelte'),rail=ui('LauncherRail.svelte');
    expect(app).toContain('if (closeReviewActive(reviewState()) || navigationReviewOpen || busy ||');
    expect(app).toContain('reviewPending={closeReviewPending}');
    expect(app).toContain('queuedLaunch=null;queued=null;queuedPage=null;');
    expect(rail).toContain('disabled={busy||reviewPending||!observedSlot');
    expect(rail).toContain('disabled={busy||reviewPending||closeInProgress');
    expect(browser()).toContain('Another launcher cannot start and discard the pending CloseOne review');
  });
  it('holds Storage refresh, alternate actions and Advanced recovery disclosure while reviewing',()=>{
    const app=ui('LazyTesterApp.svelte'),storage=ui('DataStorage.svelte');
    expect(app).toContain('manualRefreshRequested||navigationReviewOpen');
    expect(app).toContain("if (navigationReviewOpen) return;");
    expect(storage).toContain('$: storageReviewOpen = clearSlot !== null || recovery !== null || fullReview !== null;');
    expect(storage).toContain('disabled={busy||refreshing||resizing||storageReviewOpen||!available||!selectedStorage?.cache.canClear}');
    expect(storage).toContain('if(recovery!==null)event.preventDefault();');
    expect(storage).toContain('confirmationToken!==fullReview.token');
    expect(browser()).toContain('Refresh cannot invalidate a pending Storage confirmation');
  });
  it('treats a failed audio save as unconfirmed instead of displaying a false toggle',()=>{
    const app=ui('LazyTesterApp.svelte'),settings=ui('GeneralSettings.svelte');
    expect(settings).toContain('await onSaveAudio(intended)');
    expect(settings).toContain("input.checked = settings?.backgroundAudio === 'Mute'");
    expect(settings).toContain('audioHeading?.focus({preventScroll:true})');
    expect(settings).toContain('disabled={busy||audioQueued||refreshing||resizing||!available');
    expect(app).toContain("catch(error){issueKey='audio';}");
    expect(app).toContain("await lazyTester.saveAudio(mode)");
  });
});
