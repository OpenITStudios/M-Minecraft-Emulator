import {readFileSync} from 'node:fs';
import {describe,expect,it} from 'vitest';

const source=(name:string)=>readFileSync(new URL('../src/'+name,import.meta.url),'utf8');

describe('Lazy Tester Minimal Professional visual hierarchy contract',()=>{
  it('describes launcher actions accurately without claiming gameplay readiness',()=>{
    const rail=source('LauncherRail.svelte');
    expect(rail).toContain('function cardActionName(profile: ProfileSession, statusLabel: string, launchAllowed: boolean, observed: boolean): string');
    expect(rail).toContain("if (!observed) return name + ' · ' + statusLabel;");
    expect(rail).toContain("if (profile.state === 'RUNNING') return 'Select ' + name + ' · ' + statusLabel;");
    expect(rail).toContain("if (launchAllowed) return 'Launch ' + name + ' · ' + statusLabel;");
    expect(rail).toContain('return name + \' · \' + statusLabel;');
    expect(rail).toContain('aria-label={cardActionName(profile, statusLabel, launchAllowed, observed)}');
    expect(rail).toContain('on:click={()=>profile.state===\'RUNNING\'?onSelect(profile.slot):launchAllowed&&onLaunch(profile.slot)}');
    expect(rail).not.toContain('invokeRuntime(');
  });
  it('keeps the five native preview targets and their status text stable',()=>{
    const rail=source('LauncherRail.svelte');
    const css=source('lazytester.css');
    expect(rail).toContain('{#each orderedProfiles as profile (profile.slot)}');
    expect(rail).toContain('use:observePreview={profile.slot}');
    expect(rail).toContain('<div class="lazytester-launcher-health-slot">');
    expect(css).toContain('.lazytester-launcher-health-slot{flex:0 0 42px;height:42px;');
    expect(rail).toContain('lazytester-status-attention');
    expect(rail).toContain('lazytester-status-routine');
    expect(rail).not.toContain(' ✓');
    expect(css).toContain('.lazytester-filmstrip .lazytester-launcher-card small.lazytester-status-routine{color:var(--lazytester-muted)}');
    expect(rail).toContain("<strong title={'Minecraft - '+profile.slot}>MC0{profile.slot}");
    expect(rail).toContain("'Selected'");
    expect(css).toContain('.lazytester-filmstrip .lazytester-launcher-card small.lazytester-status-attention{color:var(--lazytester-warning)}');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
  });
  it('keeps storage inventory in its disclosure without repeating summary counts',()=>{
    const storage=source('DataStorage.svelte'),css=source('lazytester.css');
    expect(storage).toContain('<summary><strong>Resource pack details</strong></summary>');
    expect(storage).toContain('Counts pack folders with manifests, not packs loaded in the game.');
    expect(storage).toContain('Active packs and memory usage are unknown.');
    expect(storage).not.toContain('Texture Streaming — off');
    expect(storage).not.toContain('Improved Input Response — on');
    expect(css).not.toContain('.lazytester-storage-packs summary small');
  });
  it('provides accessible section hierarchy and uses global settings',()=>{
    const settings=source('GeneralSettings.svelte');
    const storage=source('DataStorage.svelte');
    for (const id of ['lazytester-frame-limits-section',
      'lazytester-app-behavior-section']) {
      expect(settings).toContain('aria-labelledby="'+id+'"');
      expect(settings).toContain('id="'+id+'"');
    }
    expect(storage).toContain('aria-labelledby="lazytester-storage-overview-section"');
    expect(storage).toContain('id="lazytester-storage-overview-section"');
    expect(storage).toContain('aria-labelledby="lazytester-clear-data-heading"');
    expect(storage).toContain('confirmationToken!==fullReview.token');
    expect(storage).toContain('on:submit|preventDefault={onClearAllData}');
  });
  it('keeps dense Settings/agent copy readable without desktop game-HUD controls',()=>{
    const css=source('lazytester.css');
    expect(css).toContain('.lazytester-help{');
    expect(css).toContain('font-size:13px!important}');
    expect(css).toContain('.lazytester-data-path{');
    expect(css).toContain('.lazytester-agent-activity-details p{margin:8px 0;font-size:13px;');
    expect(css).toContain('.lazytester-agent-testing-summary dl{display:grid;');
    expect(css).not.toContain('.lazytester-coordinate-popup');
    expect(css).not.toContain('.lazytester-f3-popup');
    expect(css).not.toContain('.lazytester-play-utilities');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
  });
  it('compacts the default Play view while preserving five native preview targets',()=>{
    const app=source('LazyTesterApp.svelte'),rail=source('LauncherRail.svelte'),css=source('lazytester.css');
    expect(app).toContain("<main class={page==='workspace'?'lazytester-play-main':undefined}>");
    expect(app).toContain('hidden={active!==null&&!detached}');
    expect(app).not.toContain("aria-label={'Open Minecraft - '+active+' in a window'}");
    expect(app).not.toContain('class="lazytester-play-utilities"');
    expect(app).not.toContain('F3InspectorPanel');
    expect(app).not.toContain('CoordinatePastePanel');
    expect(app).not.toContain('on:click={release}>Separate window</button>');
    expect(rail).toContain('aria-hidden="true">×</span></button>');
    expect(rail).toContain('{#each orderedProfiles as profile (profile.slot)}');
    expect(rail).toContain('use:observePreview={profile.slot}');
    expect(rail).toContain('onRequestClose(profile.slot,event.currentTarget)');
    expect(css).toContain('.lazytester-app main.lazytester-play-main{padding:10px 16px 12px;overflow-x:hidden}');
    expect(css).toContain('clamp(72px,11vh,108px)');
    expect(css).toContain('min-height:32px;line-height:32px;padding-right:36px;');
    expect(css).toContain('.lazytester-play-main .lazytester-health-notice{display:flex;');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
  });
  it('keeps selected close review inside the fixed launcher action strip',()=>{
    const app=source('LazyTesterApp.svelte'),css=source('lazytester.css');
    const strip=app.indexOf('class="lazytester-games-heading lazytester-launcher-action-strip"');
    const review=app.indexOf('class="lazytester-launcher-close-review"');
    const rail=app.indexOf('<LauncherRail');
    expect(strip).toBeGreaterThan(-1);
    expect(review).toBeGreaterThan(strip);
    expect(rail).toBeGreaterThan(review);
    expect(css).toContain('.lazytester-launcher-action-strip{height:56px;flex:0 0 56px;');
    expect(app).not.toContain('lazytester-single-close-confirm');
    expect(app).toContain('class="lazytester-workspace-preview-feedback"');
    expect(app).toContain('{@render actionFeedback(true)}');
    expect(app).toContain('{@render actionFeedback(false)}');
    expect(css).toContain('.lazytester-app .lazytester-launcher-action-strip > .lazytester-workspace-action-feedback');
    expect(app).not.toContain('lazytester-preview-details');
    expect(css).not.toContain('.lazytester-preview-details{');
  });
  it('uses one adaptive workspace without mode controls or a second stored state',()=>{
    const app=source('LazyTesterApp.svelte'),settings=source('GeneralSettings.svelte'),css=source('lazytester.css');
    expect(app).toContain("<main class={page==='workspace'?'lazytester-play-main':undefined}>");
    expect(css).toContain('clamp(72px,11vh,108px)');
    expect(css).toContain('aspect-ratio:16/9;');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(app).not.toContain('workspaceLayout');
    expect(app).not.toContain('layoutPersistenceIssue');
    expect(app).not.toContain('window.localStorage');
    expect(app).not.toContain('lazytester-workspace-monitoring');
    expect(settings).not.toContain('id="workspace-layout"');
    expect(settings).not.toContain('onChooseWorkspaceLayout');
    expect(css).not.toContain('lazytester-workspace-monitoring');
    expect(css).not.toContain('clamp(96px,15vh,136px)');
    expect(css).not.toContain('clamp(80px,16vh,180px)');
  });
  it('keeps manual recovery Refresh but no detach action in the Play header',()=>{
    const app=source('LazyTesterApp.svelte');
    const css=source('lazytester.css');
    const refresh=app.indexOf('bind:this={headerRefreshButton}');
    expect(app).toContain('{#if showHeaderRefresh}');
    expect(app).toContain("page!=='workspace' || manualRefreshing || manualRefreshRequested");
    const separate=app.indexOf('on:click={release}>Separate window</button>');
    expect(refresh).toBeGreaterThan(-1);
    expect(separate).toBe(-1);
    expect(app).not.toContain('async function release()');
    expect(app).not.toContain("aria-label={'Open Minecraft - '+active+' in a window'}");
    expect(app).not.toContain('F3InspectorPanel');
    expect(app).not.toContain('CoordinatePastePanel');
    expect(app).not.toContain('playToolsElement');
    expect(app).not.toContain('projectF3Inspector');
    expect(app).not.toContain('on:blur={closePlayToolsOnBlur}');
    expect(css).not.toContain('.lazytester-play-tools');
    expect(css).not.toContain('.lazytester-play-utilities');
    expect(css).not.toContain('.lazytester-coordinate-paste');
    expect(css).not.toContain('.lazytester-f3-popup');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
  });
  it('limits new density and status CSS to presentation owners',()=>{
    const css=source('lazytester.css');
    expect(css).toContain('--lazytester-control-min-height:36px');
    expect(css).toContain('.lazytester-settings-layout :is(.lazytester-actions > button,.lazytester-setting > button,.lazytester-danger-zone > button)');
    expect(css).toContain('.lazytester-app .lazytester-preview-slot-issue{color:var(--lazytester-warning)!important}');
    expect(css).toContain('.lazytester-settings-layout .lazytester-delete-confirm{border-left-color:var(--lazytester-danger)}');
  });
});
