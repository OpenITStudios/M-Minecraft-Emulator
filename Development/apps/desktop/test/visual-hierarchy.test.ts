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
  it('limits new density and status CSS to presentation owners',()=>{
    const css=source('lazy-tester.css');
    expect(css).toContain('--lt-control-min-height:36px');
    expect(css).toContain('.lt-settings-layout :is(.lt-actions > button,.lt-setting > button,.lt-danger-zone > button)');
    expect(css).toContain('.lt-app .lt-preview-slot-issue{color:var(--lt-warning)!important}');
    expect(css).toContain('.lt-settings-layout .lt-delete-confirm{border-left-color:var(--lt-danger)}');
  });
});
