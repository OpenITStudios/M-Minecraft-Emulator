import {readFileSync} from 'node:fs';
import {describe, expect, it} from 'vitest';

const source=(name:string)=>readFileSync(new URL('../src/'+name,import.meta.url),'utf8');

describe('Lazy Tester mixed launcher and long-copy source contract',()=>{
  it('uses stable MC01–MC05 visual labels while retaining full slot names for actions',()=>{
    const rail=source('LauncherRail.svelte');
    const css=source('lazy-tester.css');
    expect(rail).toContain("<strong title={'Minecraft - '+profile.slot}>MC0{profile.slot}");
    expect(rail).toContain("const name = 'Minecraft - ' + profile.slot;");
    expect(rail).toContain("aria-label={'Minecraft - '+profile.slot+' launcher controls'}");
    expect(rail).toContain('aria-label={cardActionName(profile, statusLabel)}');
    expect(rail).toContain('onRequestClose(profile.slot,event.currentTarget)');
    expect(rail).toContain('{#each orderedProfiles as profile (profile.slot)}');
    expect(css).toContain('min-height:32px;line-height:32px;padding-right:36px;');
  });

  it('limits global Play warnings without obscuring their text or shrinking the native game owner',()=>{
    const css=source('lazy-tester.css');
    const app=source('LazyTesterApp.svelte');
    expect(css).toContain('main.lt-play-main > :is(.lt-alert,.lt-binding,.lt-exit-recovery)');
    expect(css).toContain('max-height:min(104px,18dvh);');
    expect(css).toContain('max-height:min(88px,16dvh);');
    expect(css).toContain('overflow:auto;\n  overflow-wrap:anywhere;');
    expect(css).toContain('overscroll-behavior:contain;');
    expect(app).toContain('<div class="lt-alert" role="alert" tabindex="0">');
    expect(app).toContain('tabindex="0" aria-label="Window recovery unconfirmed"');
    expect(app).toContain('tabindex="0" aria-label="Launcher recovery"');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
  });

  it('uses one observed status in the card text, accessible name and tooltip',()=>{
    const rail=source('LauncherRail.svelte');
    expect(rail).toContain('{@const statusLabel = !observedSlot || !available || !bindingConfigured');
    expect(rail).toContain('function cardActionName(profile: ProfileSession, statusLabel: string)');
    expect(rail).toContain('aria-label={cardActionName(profile, statusLabel)}');
    expect(rail).toContain('title={statusLabel}>{statusLabel}</small>');
    expect(rail).toContain('use:observePreview={profile.slot}');
    expect(rail).toContain('{#each orderedProfiles as profile (profile.slot)}');
    expect(rail).toContain('canRequestLaunch(profile)&&onLaunch(profile.slot)');
    expect(rail).toContain('disabled={busy||!observedSlot||!available||!bindingConfigured');
    expect(rail).toContain('onRequestClose(profile.slot,event.currentTarget)');
  });

  it('keeps the complete five-session summary in DOM even when it ellipsizes',()=>{
    const app=source('LazyTesterApp.svelte');
    const css=source('lazy-tester.css');
    expect(app).toContain('gamesStatusSummary = available && binding?.configured');
    expect(app).toContain("observedRunningCount + ' of 5 reported running'");
    expect(app).toContain("startingCount > 0 ? ' · ' + startingCount + ' starting'");
    expect(app).toContain("attentionCount > 0 ? ' · ' + attentionCount + ' need attention'");
    expect(app).toContain('class="lt-games-count" title={gamesStatusSummary}');
    expect(css).toContain('.lt-games-heading .lt-games-count{');
    expect(css).toContain('text-overflow:ellipsis;');
  });

  it('makes bounded recovery copy keyboard-scrollable without native IPC',()=>{
    const health=source('SessionHealthNotice.svelte');
    const app=source('LazyTesterApp.svelte');
    const css=source('lazy-tester.css');
    expect(health).toContain("aria-describedby={'lt-health-guidance-'+item.slot}");
    expect(health).toContain("title={item.title+' — '+item.guidance}");
    expect(health).toContain("id={'lt-health-guidance-'+item.slot}");
    expect(health).toContain('tabindex="0"');
    expect(app).toContain('<div class="lt-alert" role="alert" tabindex="0">');
    expect(app).toContain('tabindex="0" aria-label="Window recovery unconfirmed"');
    expect(app).toContain('tabindex="0" aria-label="Launcher recovery"');
    expect(css).toContain('max-height:38px;');
    expect(css).toContain('.lt-app .lt-health-notice [tabindex="0"]:focus-visible');
    expect(css).toContain('max-height:min(160px,28dvh);');
    expect(health).not.toMatch(/\binvokeRuntime\s*\(|\blazyTester\.\w+\(/);
  });

  it('preserves five fixed native preview identities and original close admission',()=>{
    const rail=source('LauncherRail.svelte');
    const app=source('LazyTesterApp.svelte');
    const css=source('lazy-tester.css');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
    expect(app).toContain('onRequestClose={requestCloseOne} onGuidedRecovery={requestGuidedRecovery}');
    expect(app).toContain('await lazyTester.present(slot, viewport.getBoundingClientRect())');
    expect(rail).toContain('aria-pressed={observedSlot&&profile.state===\'RUNNING\'');
  });
});
