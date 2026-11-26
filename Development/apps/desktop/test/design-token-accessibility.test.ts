import {readFileSync} from 'node:fs';
import {describe,expect,it} from 'vitest';

const css=readFileSync(new URL('../src/lazytester.css',import.meta.url),'utf8');

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
      ['lazytester-text','lazytester-bg'],
      ['lazytester-muted','lazytester-panel'],
      ['lazytester-accent','lazytester-panel'],
      ['lazytester-warning','lazytester-panel'],
      ['lazytester-danger','lazytester-alert-surface'],
      ['lazytester-on-danger','lazytester-danger-surface'],
      ['lazytester-on-danger','lazytester-danger-hover'],
    ]) {
      expect(contrast(token(foreground),token(background))).toBeGreaterThanOrEqual(4.5);
    }
  });
  it('keeps functional input outlines distinct at 3:1 against their surfaces',()=>{
    expect(contrast(token('lazytester-control-outline'),token('lazytester-bg')))
      .toBeGreaterThanOrEqual(3);
    expect(contrast(token('lazytester-control-outline'),token('lazytester-panel')))
      .toBeGreaterThanOrEqual(3);
    expect(contrast(token('lazytester-accent'),token('lazytester-panel')))
      .toBeGreaterThanOrEqual(3);
    expect(contrast(token('lazytester-advanced-outline'),token('lazytester-panel')))
      .toBeGreaterThanOrEqual(3);
    expect(contrast(token('lazytester-advanced-outline'),token('lazytester-expanded-surface')))
      .toBeGreaterThanOrEqual(3);
    expect(contrast(token('lazytester-danger-outline'),token('lazytester-panel')))
      .toBeGreaterThanOrEqual(3);
    expect(contrast(token('lazytester-alert-outline'),token('lazytester-alert-surface')))
      .toBeGreaterThanOrEqual(3);
  });
  it('routes special use colors through semantic tokens instead of CSS literals',()=>{
    for(const name of ['lazytester-advanced-outline','lazytester-expanded-surface','lazytester-preview-empty',
      'lazytester-alert-outline','lazytester-alert-surface','lazytester-danger-surface',
      'lazytester-danger-outline','lazytester-on-danger','lazytester-control-hover','lazytester-accent-hover']) {
      expect(token(name)).toMatch(/^#[0-9a-fA-F]{3,6}$/);
    }
    expect(css).toContain('background:var(--lazytester-alert-surface)');
    expect(css).toContain('border-color:var(--lazytester-alert-outline)');
    expect(css).toContain('background:var(--lazytester-preview-empty)');
    expect(css).toContain('background:var(--lazytester-danger-surface)');
    expect(css).toContain('color:var(--lazytester-on-danger)');
    expect(css).toContain('accent-color:var(--lazytester-accent)');
    expect(css).toContain('box-shadow:0 6px 24px var(--lazytester-overlay-shadow)');
    expect(css).toContain('box-shadow:0 6px 16px var(--lazytester-overlay-shadow)');
  });
  it('retains visible focus and explicit disabled/hover controls without resizing Play DWM',()=>{
    expect(css).toContain('opacity:.68}');
    expect(css).toContain('.lazytester-app button.lazytester-secondary:hover:not(:disabled)');
    expect(css).toContain('.lazytester-settings-layout .lazytester-delete-confirm button.lazytester-danger:hover:not(:disabled)');
    expect(css).toContain('.lazytester-settings-layout :is(.lazytester-audio-toggle,.lazytester-delete-ack){');
    expect(css).toContain('min-height:32px;');
    expect(css).toContain('.lazytester-settings-layout :is(.lazytester-launcher-heading select,.lazytester-fps input){');
    expect(css).toContain('min-height:var(--lazytester-control-min-height);');
    expect(css).toContain('.lazytester-app :is(button,a,input,textarea,select,summary):focus-visible{');
    expect(css).not.toContain('outline-offset:-3px}');
    expect(css).toContain('@media (forced-colors:active)');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(css).toContain('width:min(100cqw,calc(100cqh * 16 / 9))');
  });
});
