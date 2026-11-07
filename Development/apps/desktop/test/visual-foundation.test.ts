import {readFileSync} from 'node:fs';
import {describe,it,expect} from 'vitest';

const asset=(name:string)=>readFileSync(new URL('../'+name,import.meta.url),'utf8');

describe('Lazy Tester minimal professional source contracts',()=>{
  it('reuses one standalone branded vector in the Tauri and WebView surfaces',()=>{
    const svg=asset('src-tauri/app-icon.svg');
    const css=asset('src/lazy-tester.css');
    const play=asset('src/LazyTesterApp.svelte');
    const bar=asset('src/WindowTitlebar.svelte');
    expect(svg).toContain('viewBox="0 0 256 256"');
    expect(svg).toContain('aria-label="LazyTester"');
    expect(svg).toContain('Approved LazyTester icon');
    expect(svg).toContain('fill="#343DF3"');
    expect(svg).toContain('<g fill="#FFFFFF">');
    expect((svg.match(/<path /g)??[])).toHaveLength(8);
    expect(svg).not.toContain('<image');
    expect(svg).not.toContain('data:image/');
    expect(css).toContain('url("../src-tauri/app-icon.svg")');
    expect(play).toContain('class="lt-brand-mark"');
    expect(bar).toContain('class="lt-window-mark"');
    expect(play).not.toContain('<span>LT</span>');
  });
  it('does not expose the pointer-only native titlebar drag surface as a keyboard button',()=>{
    const bar=asset('src/WindowTitlebar.svelte');
    expect(bar).toContain('<div class="lt-window-drag" data-tauri-drag-region>');
    expect(bar).toContain('<span class="lt-window-mark" aria-hidden="true" data-tauri-drag-region></span>');
    expect(bar).toContain('<span data-tauri-drag-region>LazyTester</span>');
    expect(bar).not.toContain('<button class="lt-window-drag"');
    expect(bar).not.toContain('Move Lazy Tester window');
    expect(bar).not.toContain('on:dblclick=');
    expect(bar).toContain('aria-label="Minimize"');
    expect(bar).toContain("aria-label={maximized?'Restore down':'Maximize'}");
    expect(bar).toContain("on:click={()=>action('toggleMaximize')}");
    expect(bar).toContain("on:click={()=>action('close')}");
  });
  it('keeps overflowing no-game status text keyboard-scrollable without changing native placement',()=>{
    const play=asset('src/LazyTesterApp.svelte');
    const css=asset('src/lazy-tester.css');
    const browser=readFileSync(new URL('../../../tooling/acceptance/lazy-tester-ui-regression.mjs',import.meta.url),'utf8');
    expect((play.match(/class="lt-empty" role="region" aria-label="Minecraft workspace status" tabindex="0"/g)??[]))
      .toHaveLength(2);
    expect(css).toContain('justify-content:safe center;');
    expect(css).toContain('overflow-y:auto;');
    expect(css).toContain('overflow-wrap:anywhere;');
    expect(css).toContain('.lt-app .lt-viewport .lt-empty:focus-visible{');
    expect(css).toContain('.lt-app .lt-viewport .lt-empty:focus-visible{\n    outline-color:Highlight;');
    expect(browser).toContain('const emptyStatus=page.getByRole(\'region\',{name:\'Minecraft workspace status\'});');
    expect(browser).toContain('emptyStatusScrollable.scrollOffset>0');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
  });
  it('keeps occupied-session gates apart from observed running labels',()=>{
    const play=asset('src/LazyTesterApp.svelte');
    expect(play).toContain("['STARTING', 'RUNNING', 'NOT_RESPONDING']");
    expect(play).toContain("observedRunningCount = profiles.filter(profile => profile.state === 'RUNNING').length");
    expect(play).toContain('of 5 reported running');
    expect(play).toContain('startingCount');
    expect(play).toContain('{runningCount}');
  });
  it('retains graphics drafts without any in-app F3 or coordinate feature',()=>{
    const play=asset('src/LazyTesterApp.svelte');
    const settings=asset('src/GeneralSettings.svelte');
    expect(play).toContain('graphicsDrafts = new Map<ProfileSlot, GraphicsPreset>()');
    expect(play).toContain("if (page==='settings') rememberGraphicsDraft()");
    expect(play).toContain('graphicsPreset=graphicsChoice(selectedSlot)');
    expect(play).toContain('graphicsDrafts.delete(request.slot)');
    expect(settings).toContain('Unsaved choice.');
    expect(play).not.toContain('CoordinatePastePanel');
    expect(play).not.toContain('F3InspectorPanel');
    expect(play).not.toContain('projectF3Inspector');
    expect(settings).not.toContain('Coordinates & Saves');
  });
  it('keeps functional focus and contrast tokens within the existing UI owner',()=>{
    const css=asset('src/lazy-tester.css');
    expect(css).toContain('--lt-control-outline:#667387');
    expect(css).toContain('border:1px solid var(--lt-control-outline)');
    expect(css).toContain('@media (forced-colors:active)');
    expect(css).toContain('@media (prefers-reduced-motion:reduce)');
  });
});
