import {readFileSync} from 'node:fs';
import {describe, expect, it} from 'vitest';

const ui=(name:string)=>readFileSync(new URL('../src/'+name,import.meta.url),'utf8');
const browser=()=>readFileSync(new URL('../../../tooling/acceptance/lazy-tester-ui-regression.mjs',import.meta.url),'utf8');

describe('Lazy Tester scoped keyboard and transient report focus — source contract',()=>{
  it('limits Escape to the active CloseOne review and retains Cancel-first focus',()=>{
    const app=ui('LazyTesterApp.svelte');
    expect(app).toContain('let closeOneReviewElement: HTMLElement | undefined;');
    expect(app).toContain('bind:this={closeOneReviewElement}');
    expect(app).toContain("event.key!=='Escape'");
    expect(app).toContain('event.target instanceof Node');
    expect(app).toContain('!closeOneReviewElement?.contains(event.target)');
    expect(app).toContain('closeCancelButton.focus()');
    expect(app).toContain('trigger.focus({preventScroll:true})');
    expect(app).toContain('event.stopPropagation();');
    expect(app).toContain('cancelCloseOne();');
    expect(app).toContain('await lazyTester.closeOne(slot)');
  });

  it('restores keyboard context when a transient local activity report unmounts',()=>{
    const notice=ui('AgentActivityNotice.svelte');
    const app=ui('LazyTesterApp.svelte');
    expect(notice).toContain("import {onDestroy} from 'svelte';");
    expect(notice).toContain('onDestroy(() => {');
    expect(notice).toContain('detailsElement?.contains(focused)');
    expect(notice).toContain("'.lt-topbar nav button[aria-current=\"page\"]'");
    expect(notice).toContain('playNav?.isConnected && !playNav.disabled');
    expect(notice).toContain('playNav.focus({preventScroll: true})');
    expect(app).toContain('{#if page===\'workspace\'&&available&&activityView}');
    for(const forbidden of ['setInterval(', 'invokeRuntime(', 'navigator.clipboard']) {
      expect(notice).not.toContain(forbidden);
    }
  });

  it('retains positive accessible action selectors and checks blocked native states by slot',()=>{
    const test=browser();
    expect(test).toContain("name:/^Launch Minecraft - 1 · /");
    expect(test).toContain("name:/^Select Minecraft - 2 · /");
    expect(test).not.toContain("name:'Launch Minecraft - 1',exact:true");
    expect(test).toContain(".lt-launcher-entry[data-slot=\"2\"] > .lt-card");
    expect(test).toContain('A blocked native state must not be announced as Launch');
    expect(test).toContain('Escape outside the focused CloseOne review must not intercept other UI');
    expect(test).toContain('lazy_tester_close_one');
  });

  it('preserves the original five keyed native preview and data-clear contracts',()=>{
    const app=ui('LazyTesterApp.svelte');
    const rail=ui('LauncherRail.svelte');
    const storage=ui('DataStorage.svelte');
    const css=ui('lazy-tester.css');
    expect(rail).toContain('{#each orderedProfiles as profile (profile.slot)}');
    expect(rail).toContain('use:observePreview={profile.slot}');
    expect(app).toContain('onRequestClose={requestCloseOne} onGuidedRecovery={requestGuidedRecovery}');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
    expect(storage).toContain('confirmationToken!==fullReview.token');
    expect(storage).toContain('on:submit|preventDefault={onClearAllData}');
  });
});
