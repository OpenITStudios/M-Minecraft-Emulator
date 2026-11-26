import {readFileSync} from 'node:fs';
import {describe,expect,it} from 'vitest';

const source=(file:string)=>readFileSync(new URL('../src/'+file,import.meta.url),'utf8');
const browser=()=>readFileSync(new URL('../../../tooling/acceptance/lazy-tester-ui-regression.mjs',import.meta.url),'utf8');

describe('Lazy Tester optional activity status stays inside fixed-height topbar',()=>{
  it('preserves the five keyed native previews and original aperture geometry',()=>{
    const css=source('lazytester.css');
    const app=source('LazyTesterApp.svelte');
    expect(css).toContain('height:56px;');
    expect(css).toContain('@media(max-height:700px)');
    expect(css).toContain('height:54px}');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
    expect(app).toContain('<LauncherRail {profiles} {labels} {healthIssues}');
    expect(app).toContain('onRequestClose={requestCloseOne} onGuidedRecovery={requestGuidedRecovery}');
  });
  it('gives the short-lived AI summary flow space rather than an overlapping badge',()=>{
    const css=source('lazytester.css');
    const app=source('LazyTesterApp.svelte');
    const notice=source('AgentActivityNotice.svelte');
    expect(css).toContain('.lazytester-agent-activity{position:relative;right:auto;top:auto;z-index:12;flex:0 1 192px;');
    expect(css).not.toContain('right:230px;top:9px');
    expect(css).toContain('.lazytester-topbar .lazytester-brand-name{white-space:nowrap}');
    expect(css).toContain('.lazytester-topbar .lazytester-connection{');
    expect(app).toContain('aria-label="LazyTester"');
    expect(app).toContain('<span class="lazytester-brand-name">LazyTester</span>');
    expect(app).toContain('role="status" title={sessionConnectionLabel(');
    expect(app).toContain("{#if initializing || nativeBackend !== 'WindowsNative' || !binding?.configured || !available}");
    expect(notice).toContain('Summary takes a bounded slot in the fixed-height topbar');
    expect(notice).toContain('name="lazytester-play-disclosures"');
  });
  it('keeps details viewport-bounded and the original activity information intact',()=>{
    const css=source('lazytester.css');
    const notice=source('AgentActivityNotice.svelte');
    expect(css).toContain('max-height:min(260px,calc(100dvh - 102px))');
    expect(css).toContain('@media (max-width:860px)');
    expect(css).toContain('@media (max-width:650px)');
    expect(css).toContain('@media (max-width:490px)');
    expect(css).toContain('.lazytester-topbar .lazytester-brand-name{display:none}');
    expect(css).toContain('top:90px;left:12px;right:12px;');
    expect(notice).toContain('aria-label={view.summary} title={view.summary}');
    expect(notice).toContain('Reported by the local agent.');
    expect(notice).not.toContain('invokeRuntime(');
  });
  it('authors CSS viewport checks separately from real Windows DPI verification',()=>{
    const journey=browser();
    expect(journey).toContain('Routine AI activity must not occupy the topbar');
    expect(journey).toContain('for(const width of [920,720,620,480])');
    expect(journey).toContain('Optional AI summary must not overlap navigation');
    expect(journey).toContain('Optional AI summary must not change the fixed-height topbar');
    expect(journey).toContain('Narrow agent details must stay in the WebView bounds');
    expect(journey).toContain('NOT evidence of actual Windows device DPI');
  });
});
