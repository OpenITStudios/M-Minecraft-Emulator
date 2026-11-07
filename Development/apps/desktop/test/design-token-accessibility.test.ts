import {readFileSync} from 'node:fs';
import {describe,expect,it} from 'vitest';

const css=readFileSync(new URL('../src/lazy-tester.css',import.meta.url),'utf8');

function token(name:string):string {
  const found=css.match(new RegExp(String.raw`--`+name+String.raw`:\s*(#[0-9a-fA-F]{3,8})\s*;`));
  if(!found)throw new Error('Missing semantic color token: '+name);
  return found[1];
}
function luminance(color:string):number {
  const body=color.slice(1);
  const hex=body.length===3?[...body].map(x=>x+x).join(''):body;
  if(hex.length!==6)throw new Error('Expected RGB token');
  const channels=[0,2,4].map(i=>parseInt(hex.slice(i,i+2),16)/255)
    .map(c=>c<=0.04045?c/12.92:((c+0.055)/1.055)**2.4);
  return 0.2126*channels[0]+0.7152*channels[1]+0.0722*channels[2];
}
function contrast(fg:string,bg:string):number {
  const a=luminance(fg),b=luminance(bg);
  return (Math.max(a,b)+0.05)/(Math.min(a,b)+0.05);
}

describe('Minimal Professional color and control state source contracts',()=>{
  it('maintains readable authored text against known opaque source surfaces',()=>{
    for(const [foreground,background] of [
      ['lt-text','lt-bg'],
      ['lt-muted','lt-panel'],
      ['lt-accent','lt-panel'],
      ['lt-warning','lt-panel'],
      ['lt-danger','lt-alert-surface'],
      ['lt-on-danger','lt-danger-surface'],
      ['lt-on-danger','lt-danger-hover'],
    ]) {
      expect(contrast(token(foreground),token(background))).toBeGreaterThanOrEqual(4.5);
    }
  });
  it('keeps functional input outlines distinct at 3:1 against their surfaces',()=>{
    expect(contrast(token('lt-control-outline'),token('lt-bg')))
      .toBeGreaterThanOrEqual(3);
    expect(contrast(token('lt-control-outline'),token('lt-panel')))
      .toBeGreaterThanOrEqual(3);
    expect(contrast(token('lt-accent'),token('lt-panel')))
      .toBeGreaterThanOrEqual(3);
    expect(contrast(token('lt-advanced-outline'),token('lt-panel')))
      .toBeGreaterThanOrEqual(3);
    expect(contrast(token('lt-advanced-outline'),token('lt-expanded-surface')))
      .toBeGreaterThanOrEqual(3);
    expect(contrast(token('lt-danger-outline'),token('lt-panel')))
      .toBeGreaterThanOrEqual(3);
    expect(contrast(token('lt-alert-outline'),token('lt-alert-surface')))
      .toBeGreaterThanOrEqual(3);
  });
  it('routes special use colors through semantic tokens instead of CSS literals',()=>{
    for(const name of ['lt-advanced-outline','lt-expanded-surface','lt-preview-empty',
      'lt-alert-outline','lt-alert-surface','lt-danger-surface',
      'lt-danger-outline','lt-on-danger','lt-control-hover','lt-accent-hover']) {
      expect(token(name)).toMatch(/^#[0-9a-fA-F]{3,6}$/);
    }
    expect(css).toContain('background:var(--lt-alert-surface)');
    expect(css).toContain('border-color:var(--lt-alert-outline)');
    expect(css).toContain('background:var(--lt-preview-empty)');
    expect(css).toContain('background:var(--lt-danger-surface)');
    expect(css).toContain('color:var(--lt-on-danger)');
    expect(css).toContain('accent-color:var(--lt-accent)');
    expect(css).toContain('box-shadow:0 12px 28px var(--lt-popup-shadow)');
  });
  it('retains visible focus and explicit disabled/hover controls without resizing Play DWM',()=>{
    expect(css).toContain('opacity:.68}');
    expect(css).toContain('.lt-app button.lt-secondary:hover:not(:disabled)');
    expect(css).toContain('.lt-settings-layout .lt-delete-confirm button.lt-danger:hover:not(:disabled)');
    expect(css).toContain('.lt-settings-layout :is(.lt-audio-toggle,.lt-delete-ack){');
    expect(css).toContain('min-height:32px;');
    expect(css).toContain('.lt-settings-layout :is(.lt-target select,.lt-launcher-heading select,#graphics-preset,.lt-fps input){');
    expect(css).toContain('min-height:var(--lt-control-min-height);');
    expect(css).toContain('.lt-app :is(button,a,input,textarea,select,summary):focus-visible{');
    expect(css).not.toContain('outline-offset:-3px}');
    expect(css).toContain('@media (forced-colors:active)');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
  });
});
