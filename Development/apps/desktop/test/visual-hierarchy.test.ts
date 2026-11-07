import {readFileSync} from 'node:fs';
import {describe,expect,it} from 'vitest';

const source=(name:string)=>readFileSync(new URL('../src/'+name,import.meta.url),'utf8');

describe('Lazy Tester Minimal Professional visual hierarchy contract',()=>{
  it('describes launcher actions accurately without claiming gameplay readiness',()=>{
    const rail=source('LauncherRail.svelte');
    expect(rail).toContain('function cardActionName(profile: ProfileSession, statusLabel: string): string');
    expect(rail).toContain("if (!available || !bindingConfigured) return name + ' · ' + statusLabel;");
    expect(rail).toContain("if (profile.state === 'RUNNING') return 'Select ' + name + ' · ' + statusLabel;");
    expect(rail).toContain("if (canRequestLaunch(profile)) return 'Launch ' + name + ' · ' + statusLabel;");
    expect(rail).toContain('return name + \' · \' + statusLabel;');
    expect(rail).toContain('aria-label={cardActionName(profile, statusLabel)}');
    expect(rail).toContain('on:click={()=>profile.state===\'RUNNING\'?onSelect(profile.slot):canRequestLaunch(profile)&&onLaunch(profile.slot)}');
    expect(rail).not.toContain('invokeRuntime(');
  });
  it('keeps the five native preview targets and their status text stable',()=>{
    const rail=source('LauncherRail.svelte');
    const css=source('lazy-tester.css');
    expect(rail).toContain('{#each orderedProfiles as profile (profile.slot)}');
    expect(rail).toContain('use:observePreview={profile.slot}');
    expect(rail).toContain('lt-status-attention');
    expect(rail).toContain('lt-status-selected');
    expect(rail).toContain("'Selected in Play'");
    expect(css).toContain('.lt-filmstrip .lt-card small.lt-status-attention{color:var(--lt-warning)}');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
  });
  it('provides consistent accessible section hierarchy without hiding controls',()=>{
    const settings=source('GeneralSettings.svelte');
    const storage=source('DataStorage.svelte');
    for (const id of ['lt-graphics-section','lt-frame-limits-section',
      'lt-app-behavior-section','lt-install-data-section']) {
      expect(settings).toContain('aria-labelledby="'+id+'"');
      expect(settings).toContain('id="'+id+'"');
    }
    expect(storage).toContain('aria-labelledby="lt-storage-overview-section"');
    expect(storage).toContain('id="lt-storage-overview-section"');
    expect(storage).toContain('aria-labelledby="lt-clear-data-heading"');
    expect(storage).toContain('confirmationToken!==fullReview.token');
    expect(storage).toContain('on:submit|preventDefault={onClearAllData}');
  });
  it('improves dense explanations inside scrollable settings and Play disclosures',()=>{
    const css=source('lazy-tester.css');
    expect(css).toContain('.lt-help{');
    expect(css).toContain('font-size:13px!important}');
    expect(css).toContain('.lt-data-path{');
    expect(css).toContain('.lt-agent-activity-details p{margin:8px 0;font-size:13px;');
    expect(css).toContain('.lt-agent-testing-summary dl{display:grid;');
    expect(css).toContain('.lt-coordinate-popup p{font-size:13px;');
    expect(css).toContain('.lt-coordinate-popup label{display:block;font-size:13px;');
    expect(css).toContain('.lt-f3-data{display:grid;');
    expect(css).toContain('margin:4px 0;font-size:13px;line-height:1.45}');
    expect(css).toContain('.lt-coordinate-saved-text code{font-family:Consolas,"Cascadia Mono",monospace;font-size:13px;');
    expect(css).toContain('max-height:calc(100dvh - 176px)');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
    // Collapsed Play disclosure trigger size stays unchanged: no new topbar row.
    expect(css).toContain('.lt-coordinate-paste>summary{display:block;cursor:pointer;white-space:nowrap;');
    expect(css).toContain('font-size:12px;line-height:normal}');
  });
  it('compacts the default Play view while preserving five native preview targets',()=>{
    const app=source('LazyTesterApp.svelte'),rail=source('LauncherRail.svelte'),css=source('lazy-tester.css');
    expect(app).toContain("<main class={page==='workspace'?'lt-play-main':undefined}>");
    expect(app).toContain('hidden={active!==null&&!detached}');
    expect(app).toContain("aria-label={'Open Minecraft - '+active+' in a window'}>Separate window");
    expect(rail).toContain('aria-hidden="true">×</span></button>');
    expect(rail).toContain('{#each orderedProfiles as profile (profile.slot)}');
    expect(rail).toContain('use:observePreview={profile.slot}');
    expect(rail).toContain('onRequestClose(profile.slot,event.currentTarget)');
    expect(css).toContain('.lt-app main.lt-play-main{padding:10px 16px 12px;overflow-x:hidden}');
    expect(css).toContain('clamp(72px,11vh,108px)');
    expect(css).toContain('min-height:32px;line-height:32px;padding-right:36px;');
    expect(css).toContain('.lt-play-main .lt-health-notice{display:flex;');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
  });
  it('uses one adaptive workspace without mode controls or a second stored state',()=>{
    const app=source('LazyTesterApp.svelte'),settings=source('GeneralSettings.svelte'),css=source('lazy-tester.css');
    expect(app).toContain("<main class={page==='workspace'?'lt-play-main':undefined}>");
    expect(css).toContain('clamp(72px,11vh,108px)');
    expect(css).toContain('aspect-ratio:16/9;');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(app).not.toContain('workspaceLayout');
    expect(app).not.toContain('layoutPersistenceIssue');
    expect(app).not.toContain('window.localStorage');
    expect(app).not.toContain('lt-workspace-monitoring');
    expect(settings).not.toContain('id="workspace-layout"');
    expect(settings).not.toContain('onChooseWorkspaceLayout');
    expect(css).not.toContain('lt-workspace-monitoring');
    expect(css).not.toContain('clamp(96px,15vh,136px)');
    expect(css).not.toContain('clamp(80px,16vh,180px)');
  });
  it('limits new density and status CSS to presentation owners',()=>{
    const css=source('lazy-tester.css');
    expect(css).toContain('--lt-control-min-height:36px');
    expect(css).toContain('.lt-settings-layout :is(.lt-actions > button,.lt-setting > button,.lt-danger-zone > button)');
    expect(css).toContain('.lt-app .lt-preview-slot-issue{color:var(--lt-warning)!important}');
    expect(css).toContain('.lt-settings-layout .lt-delete-confirm{border-left-color:var(--lt-danger)}');
  });
});
