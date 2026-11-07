import {readFileSync} from 'node:fs';
import {describe,expect,it} from 'vitest';

const load=(name:string)=>readFileSync(new URL('../src/'+name,import.meta.url),'utf8');
const browser=()=>readFileSync(
  new URL('../../../tooling/acceptance/lazy-tester-ui-regression.mjs',import.meta.url),'utf8');

describe('Launcher recovery action target and accessibility source contracts',()=>{
  it('exposes authored issue guidance on both the scrollable notice and its action',()=>{
    const health=load('SessionHealthNotice.svelte');
    expect((health.match(/aria-describedby=\{'lt-health-guidance-'\+item.slot\}/g)??[]))
      .toHaveLength(2);
    expect(health).toContain("id={'lt-health-guidance-'+item.slot}");
    expect(health).toContain("aria-label={actionName + ' for Minecraft - ' + item.slot}");
    expect(health).toContain('tabindex="0"');
    expect(health).toContain("item.action === 'open' ? 'Select game' : 'Refresh'");
  });

  it('guarantees a 28px recovery target without expanding the default rail footprint',()=>{
    const health=load('SessionHealthNotice.svelte');
    const rail=load('LauncherRail.svelte');
    const css=load('lazy-tester.css');
    expect(health).toContain('box-sizing: border-box; min-height: 28px;');
    expect(health).toContain('margin-top: 0; padding: 3px 6px;');
    expect(health).toContain('.lt-health-notice { min-width: 0; font-size: 13px;');
    expect(css).toContain('max-height:38px;');
    expect(css).toContain('font-size:13px;\n  line-height:1.35;');
    expect(health).toContain('disabled={disabled}');
    expect(health).toContain('on:click={requestAction}');
    expect(health).toContain('onAction(item.slot,item.action);');
    expect(health).toContain("item.action === 'refresh' && document.activeElement === actionButton");
    expect(rail).toContain('disabled={!guidedRecoveryAllowed(healthItem, profile, profile.slot,');
    expect(css).toContain('.lt-app .lt-close-launcher{width:100%;min-height:32px;');
    expect(rail).toContain('onRequestClose(profile.slot,event.currentTarget)');
    expect(health).not.toMatch(/\binvokeRuntime\s*\(|\blazyTester\s*\.\w+\s*\(/);
  });

  it('uses system feedback colors for high-contrast and retains keyboard focus',()=>{
    const health=load('SessionHealthNotice.svelte');
    expect(health).toContain('.lt-health-action:hover:not(:disabled) {');
    expect(health).toContain('background: var(--lt-selected-bg)');
    expect(health).toContain('.lt-health-action:focus-visible { outline: 2px solid var(--lt-accent);');
    expect(health).toContain('@media (forced-colors: active) {');
    expect(health).toContain('color: LinkText; border: 1px solid ButtonText;');
    expect(health).toContain('.lt-health-action:disabled { color: GrayText; }');
    expect(health).toContain('.lt-health-action:focus-visible { outline-color: Highlight; }');
  });

  it('authors CSS target-size checks without claiming native Windows acceptance',()=>{
    const journey=browser();
    const css=load('lazy-tester.css');
    expect(journey).toContain("await checkHealth.getAttribute('aria-describedby')");
    expect(journey).toContain('recoveryTargets.refresh.width>=24');
    expect(journey).toContain('recoveryTargets.refresh.height>=28');
    expect(journey).toContain('recoveryTargets.refreshMargin===0');
    expect(journey).toContain('recoveryTargets.close.height>=32');
    expect(journey).toContain('Native DWM preview target must remain allocated independently');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
  });
});
