import {readFileSync} from 'node:fs';
import {describe,expect,it} from 'vitest';

const ui=(name:string)=>readFileSync(new URL('../src/'+name,import.meta.url),'utf8');
const browser=()=>readFileSync(
  new URL('../../../tooling/acceptance/lazy-tester-ui-regression.mjs',import.meta.url),'utf8');

describe('Lazy Tester transient control focus and original native admission — source only',()=>{
  it('anchors a keyboard-activated header Refresh without taking later Tab focus',()=>{
    const app=ui('LazyTesterApp.svelte');
    const css=ui('lazy-tester.css');
    expect(app).toContain('let headerRefreshButton: HTMLButtonElement | undefined;');
    expect(app).toContain('let headerRefreshFocusPage: Page | null = null;');
    expect(app).toContain('function requestHeaderRefresh(event: MouseEvent): void {');
    expect(app).toContain('event.detail === 0 && document.hasFocus()');
    expect(app).toContain('document.activeElement === event.currentTarget');
    expect(app).toContain('pageHeading.focus({preventScroll:true});');
    expect(app).toContain('const keyboardOrigin = headerRefreshFocusPage;');
    expect(app).toContain('headerRefreshFocusPage = null;');
    expect(app).toContain('page === keyboardOrigin');
    expect(app).toContain('document.activeElement === pageHeading');
    expect(app).toContain('headerRefreshButton?.isConnected && !headerRefreshButton.disabled');
    expect(app).toContain('headerRefreshButton.focus({preventScroll:true})');
    expect(app).toContain('on:click={requestHeaderRefresh}');
    expect(app).toContain('<h1 bind:this={pageHeading} tabindex="-1">');
    expect(css).toContain('.lt-play-header h1:focus,');
  });

  it('reanchors a disappearing CloseOne confirmation to its original MC slot group',()=>{
    const app=ui('LazyTesterApp.svelte');
    const rail=ui('LauncherRail.svelte');
    const block=app.split('async function confirmCloseOne()')[1]
      ?.split('async function release()')[0] ?? '';
    expect(block).toContain("closeOneTrigger?.closest<HTMLElement>('.lt-launcher-entry')");
    expect(block).toContain('closeOneReviewElement?.contains(document.activeElement)');
    expect(block).toContain('document.hasFocus()');
    expect(block).toContain('entry.focus({preventScroll:true})');
    expect(block.indexOf('entry.focus({preventScroll:true})')).toBeLessThan(
      block.indexOf('closeOneSlot=null;closeOneTrigger=null; closingOneSlot=slot;'));
    expect(block).toContain('await lazyTester.closeOne(slot)');
    expect(rail).toContain('role="group" tabindex="-1"');
    expect(rail).toContain('onRequestClose(profile.slot,event.currentTarget)');
  });

  it('keeps deletion admission before focus handoff to a persistent warning heading',()=>{
    const app=ui('LazyTesterApp.svelte');
    const storage=ui('DataStorage.svelte');
    const block=app.split('async function clearAllData()')[1]
      ?.split('onMount(() => {')[0] ?? '';
    expect(block).toContain('fullName!==`Minecraft - ${pending.slot}`');
    expect(block).toContain('status.confirmationToken!==pending.token');
    expect(block).toContain("document.querySelector('.lt-delete-confirm')");
    expect(block).toContain('deletionReview?.contains(document.activeElement)');
    expect(block).toContain("document.getElementById('lt-clear-data-heading')?.focus({preventScroll:true})");
    expect(block).toContain('await lazyTester.clearData(pending.slot,pending.token,fullName)');
    expect(block.indexOf("document.getElementById('lt-clear-data-heading')?.focus")).toBeLessThan(
      block.indexOf('busy=true;clearingData=true;'));
    expect(storage).toContain('id="lt-clear-data-heading" tabindex="-1"');
    expect(storage).toContain('confirmationToken!==fullReview.token');
    expect(storage).toContain('on:submit|preventDefault={onClearAllData}');
  });

  it('authors Tab/Shift+Tab and CloseOne focus journeys without native execution',()=>{
    const journey=browser();
    const css=ui('lazy-tester.css');
    expect(journey).toContain("await page.keyboard.press('Shift+Tab')");
    expect(journey).toContain("Tab progresses through existing controls while Refresh is disabled");
    expect(journey).toContain('The user-selected Shift+Tab destination must not be stolen on completion');
    expect(journey).toContain('Only the existing Refresh button regains keyboard focus');
    expect(journey).toContain('Confirmed CloseOne review keeps its original stable launcher focus group');
    expect(css).toContain('.lt-settings-layout #lt-clear-data-heading:focus{');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
  });
});
