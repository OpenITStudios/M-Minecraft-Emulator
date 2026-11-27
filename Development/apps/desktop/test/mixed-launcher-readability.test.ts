import {readFileSync} from 'node:fs';
import {describe, expect, it} from 'vitest';

const source=(name:string)=>readFileSync(new URL('../src/'+name,import.meta.url),'utf8');

describe('Lazy Tester mixed launcher and long-copy source contract',()=>{
  it('uses stable MC01–MC05 visual labels while retaining full slot names for actions',()=>{
    const rail=source('LauncherRail.svelte');
    const css=source('lazytester.css');
    expect(rail).toContain("<strong title={'Minecraft - '+profile.slot}>MC0{profile.slot}");
    expect(rail).toContain("const name = 'Minecraft - ' + profile.slot;");
    expect(rail).toContain("aria-label={'Minecraft - '+profile.slot+' launcher controls'}");
    expect(rail).toContain('aria-label={cardActionName(profile, statusLabel, launchAllowed, observed)}');
    expect(rail).toContain('onRequestClose(profile.slot,event.currentTarget)');
    expect(rail).toContain('{#each orderedProfiles as profile (profile.slot)}');
    expect(rail).toContain('class="lazytester-launcher-header"');
    expect(css).toContain('padding-right:29px');
    expect(css).toContain('grid-template-rows:24px 18px');
    expect(css).toContain('.lazytester-launcher-entry{container:lazytester-launcher / inline-size}');
    expect(css).toContain('@container lazytester-launcher (max-width:221px)');
    expect(css).toContain('@container lazytester-launcher (max-width:159px)');
    expect(css).toContain('padding-right:27px');
    expect(css).toContain('.lazytester-play-main .lazytester-filmstrip{column-gap:8px}');
    expect(css).toContain('@container lazytester-workspace (min-width:960px) and (min-height:840px)');
    expect(css).toContain('min-height:42px;padding-right:0');
    expect(css).not.toContain('@media(max-width:1199px)');
    expect(css).toContain('.lazytester-workspace > .lazytester-launcher-action-strip:empty{display:none}');
  });

  it('limits global Play warnings without obscuring their text or shrinking the native game owner',()=>{
    const css=source('lazytester.css');
    const app=source('LazyTesterApp.svelte');
    expect(css).toContain('.lazytester-play-feedback:has(> :is(.lazytester-alert,.lazytester-binding,.lazytester-exit-recovery))');
    expect(css).toContain('max-height:min(104px,18dvh);');
    expect(css).toContain('max-height:min(88px,16dvh);');
    expect(css).toContain('overflow:auto;overscroll-behavior:contain;');
    expect(css).toContain('.lazytester-app .lazytester-alert.lazytester-workspace-action-feedback .lazytester-alert-footer{');
    expect(css).toContain('overscroll-behavior:contain;');
    expect(app).toContain('<div class="lazytester-alert" class:lazytester-workspace-action-feedback={compact} role="alert" tabindex="0" aria-label="Action needs attention">');
    expect(app).toContain('tabindex="0" aria-label="Window recovery unconfirmed"');
    expect(app).toContain('tabindex="0" aria-label="Launcher recovery"');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
  });

  it('uses one observed status in the card text, accessible name and tooltip',()=>{
    const rail=source('LauncherRail.svelte');
    expect(rail).toContain('{@const statusLabel = !observedSlot || !available || !bindingConfigured');
    expect(rail).toContain('function cardActionName(profile: ProfileSession, statusLabel: string, launchAllowed: boolean, observed: boolean)');
    expect(rail).toContain('aria-label={cardActionName(profile, statusLabel, launchAllowed, observed)}');
    expect(rail).toContain('title={statusLabel}>{statusLabel}</small>');
    expect(rail).toContain('use:observePreview={profile.slot}');
    expect(rail).toContain('{#each orderedProfiles as profile (profile.slot)}');
    expect(rail).toContain('launchAllowed&&onLaunch(profile.slot)');
    expect(rail).toContain('disabled={busy||reviewPending||!observedSlot||!available||!bindingConfigured');
    expect(rail).toContain('onRequestClose(profile.slot,event.currentTarget)');
  });

  it('keeps the complete five-session summary in DOM even when it ellipsizes',()=>{
    const app=source('LazyTesterApp.svelte');
    const css=source('lazytester.css');
    expect(app).toContain('gamesStatusSummary = available && binding?.configured');
    expect(app).toContain("observedRunningCount + ' of 5 running'");
    expect(app).toContain("startingCount > 0 ? ' · ' + startingCount + ' starting'");
    expect(app).toContain("attentionCount > 0 ? ' · ' + attentionCount + ' need attention'");
    expect(app).toContain('class="lazytester-games-count" title={gamesStatusSummary}');
    expect(css).toContain('.lazytester-play-header .lazytester-games-count{');
    expect(css).toContain('text-overflow:ellipsis;');
  });

  it('makes bounded recovery copy keyboard-scrollable without native IPC',()=>{
    const health=source('SessionHealthNotice.svelte');
    const app=source('LazyTesterApp.svelte');
    const css=source('lazytester.css');
    expect(health).toContain("aria-describedby={'lazytester-health-guidance-'+item.slot}");
    expect(health).toContain("title={item.title+' — '+item.guidance}");
    expect(health).toContain("id={'lazytester-health-guidance-'+item.slot}");
    expect(health).toContain('tabindex="0"');
    expect(app).toContain('<div class="lazytester-alert" class:lazytester-workspace-action-feedback={compact} role="alert" tabindex="0" aria-label="Action needs attention">');
    expect(app).toContain('tabindex="0" aria-label="Window recovery unconfirmed"');
    expect(app).toContain('tabindex="0" aria-label="Launcher recovery"');
    expect(css).toContain('max-height:38px;');
    expect(css).toContain('.lazytester-app .lazytester-health-notice [tabindex="0"]:focus-visible');
    expect(css).toContain('max-height:min(160px,28dvh);');
    expect(health).not.toMatch(/\binvokeRuntime\s*\(|\blazyTester\.\w+\(/);
  });

  it('preserves five fixed native preview identities and original close admission',()=>{
    const rail=source('LauncherRail.svelte');
    const app=source('LazyTesterApp.svelte');
    const css=source('lazytester.css');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
    expect(app).toContain('onRequestClose={requestCloseOne} onGuidedRecovery={requestGuidedRecovery}');
    expect(app).toContain('await presentCurrentViewport(slot)');
    expect(rail).toContain('aria-pressed={observed&&profile.state===\'RUNNING\'');
  });
});


it('reserves close-control space only for an observed close target',()=>{
  const rail=source('LauncherRail.svelte');
  const css=source('lazytester.css');
  expect(rail).toContain('class:has-close={hasClose}');
  expect(rail).toContain('{#if hasClose}');
  expect(css).toContain('.lazytester-launcher-header:not(.has-close){padding-right:0}');
  expect(css).toContain('.lazytester-launcher-header.has-close::after');
});
