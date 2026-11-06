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
    expect(svg).toContain('Focused Workspace');
    expect(css).toContain('url("../src-tauri/app-icon.svg")');
    expect(play).toContain('class="lt-brand-mark"');
    expect(bar).toContain('class="lt-window-mark"');
    expect(play).not.toContain('<span>LT</span>');
  });
  it('keeps occupied-session gates apart from observed running labels',()=>{
    const play=asset('src/LazyTesterApp.svelte');
    expect(play).toContain("['STARTING', 'RUNNING', 'NOT_RESPONDING']");
    expect(play).toContain("observedRunningCount = profiles.filter(profile => profile.state === 'RUNNING').length");
    expect(play).toContain('of 5 reported running');
    expect(play).toContain('startingCount');
    expect(play).toContain('{runningCount}');
  });
  it('preserves only ephemeral user drafts while maintaining native admission',()=>{
    const play=asset('src/LazyTesterApp.svelte');
    const coordinates=asset('src/CoordinatePastePanel.svelte');
    const settings=asset('src/GeneralSettings.svelte');
    expect(play).toContain('graphicsDrafts = new Map<ProfileSlot, GraphicsPreset>()');
    expect(play).toContain("if (page==='settings') rememberGraphicsDraft()");
    expect(play).toContain('graphicsPreset=graphicsChoice(selectedSlot)');
    expect(play).toContain('graphicsDrafts.delete(request.slot)');
    expect(settings).toContain('Unsaved graphics choice.');
    const close=coordinates.split('function resetOnClose(')[1]?.split('</script>')[0]??'';
    expect(close).not.toContain("raw = ''");
    expect(close).not.toContain("bookmarkName = ''");
    expect(coordinates).toContain('Closing this panel keeps your unsaved text');
    expect(coordinates).not.toContain('navigator.clipboard');
  });
  it('keeps functional focus and contrast tokens within the existing UI owner',()=>{
    const css=asset('src/lazy-tester.css');
    expect(css).toContain('--lt-control-outline:#667387');
    expect(css).toContain('border:1px solid var(--lt-control-outline)');
    expect(css).toContain('@media (forced-colors:active)');
    expect(css).toContain('@media (prefers-reduced-motion:reduce)');
  });
});
