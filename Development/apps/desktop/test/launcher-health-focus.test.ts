import {readFileSync} from 'node:fs';
import {describe,expect,it} from 'vitest';

const read=(name:string)=>readFileSync(new URL('../src/'+name,import.meta.url),'utf8');
const browser=()=>readFileSync(
  new URL('../../../tooling/acceptance/lazy-tester-ui-regression.mjs',import.meta.url),'utf8');

describe('Per-slot health notice keyboard focus lifecycle — source contract',()=>{
  it('keeps focused Refresh inside its authored message before original admission',()=>{
    const health=read('SessionHealthNotice.svelte');
    expect(health).toContain("import {onDestroy,tick} from 'svelte';");
    expect(health).toContain('function requestAction(): void {');
    expect(health).toContain("item.action === 'refresh' && document.activeElement === actionButton");
    expect(health).not.toContain("actionButton?.matches(':focus-visible')");
    expect(health).toContain('statusElement?.focus({preventScroll:true})');
    expect(health).toContain('onAction(item.slot,item.action);');
    expect(health).toContain('on:click={requestAction}');
    expect(health).toContain('disabled={disabled}');
    expect(health).not.toMatch(/\binvokeRuntime\s*\(|\blazyTester\s*\.\w+\s*\(/);
  });

  it('restores only owned WebView focus when the original same-slot notice disappears',()=>{
    const health=read('SessionHealthNotice.svelte');
    expect(health).toContain("if (item.action !== 'refresh') return;");
    expect(health).toContain("!noticeElement?.contains(previousFocus)");
    expect(health).toContain("const entry = noticeElement.closest('.lt-launcher-entry');");
    expect(health).toContain('void tick().then(() => {');
    expect(health).toContain('!entry.isConnected || !document.hasFocus()');
    expect(health).toContain('current !== document.body && current !== previousFocus');
    expect(health).toContain("entry.querySelector<HTMLButtonElement>(':scope > .lt-card')");
    expect(health).toContain('card?.isConnected && !card.disabled');
    expect(health).toContain('card.focus({preventScroll:true})');
    expect(health).toContain('entry.focus({preventScroll:true})');
  });

  it('keeps a non-Tabbable same-slot fallback visibly focusable without moving native previews',()=>{
    const rail=read('LauncherRail.svelte');
    const css=read('lazy-tester.css');
    expect(rail).toContain('role="group" tabindex="-1"');
    expect(rail).toContain("aria-label={'Minecraft - '+profile.slot+' launcher controls'}");
    expect(rail).toContain('{#each orderedProfiles as profile (profile.slot)}');
    expect(rail).toContain('use:observePreview={profile.slot}');
    expect(rail).toContain('onRequestClose(profile.slot,event.currentTarget)');
    expect(css).toContain('.lt-filmstrip .lt-launcher-entry:focus{');
    expect(css).toContain('.lt-filmstrip .lt-launcher-entry:focus{outline-color:Highlight}');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
  });

  it('authors success, disconnected and superseded focus checks without new gameplay actions',()=>{
    const journey=browser();
    expect(journey).toContain('Keyboard Refresh should keep focus in its read-only notice');
    expect(journey).toContain('A verified STOPPED observation returns focus to the same newly enabled launcher card');
    expect(journey).toContain('Connection uncertainty must not create a new Launch admission');
    expect(journey).toContain('Removing an old notice cannot steal focus after the operator chose another control');
    expect(journey).toContain('The focus handoff must not trigger native game operations');
    expect(journey).toContain('window.__holdInspect=true;window.__offline=true;');
    expect(journey).not.toContain('new ChromiumRunner(');
  });
});
