import {readFileSync} from 'node:fs';
import {describe, expect, it} from 'vitest';
import {playToolPopupStyle} from '../src/app/presentation/playToolPopup.js';

const source=(file:string)=>readFileSync(new URL('../src/'+file,import.meta.url),'utf8');

describe('nonmodal Play disclosure positioning and keyboard source contract',()=>{
  it('fits a 420px F3 popup inside a 920×620 viewport',()=>{
    const css=playToolPopupStyle({top:118,bottom:155,right:790},{width:920,height:620},420);
    expect(css).toContain('left:370px;');
    expect(css).toContain('width:420px;');
    expect(css).toContain('top:163px;bottom:auto;');
    expect(css).toContain('max-height:441px;');
    const small=playToolPopupStyle({top:100,bottom:136,right:80},{width:320,height:620},390);
    expect(small).toContain('left:16px;');
    expect(small).toContain('width:288px;');
  });
  it('places the panel above a near-bottom tool instead of clipping it',()=>{
    const css=playToolPopupStyle({top:555,bottom:595,right:900},{width:920,height:620},420);
    expect(css).toContain('top:auto;bottom:73px;');
    expect(css).toContain('max-height:531px;');
  });
  it('collapses a popup if its opening summary is scrolled outside the viewport',()=>{
    for(const file of [source('F3InspectorPanel.svelte'),source('CoordinatePastePanel.svelte')]){
      expect(file).toContain('const anchor = summaryElement.getBoundingClientRect();');
      expect(file).toContain('anchor.bottom <= 0 || anchor.top >= window.innerHeight');
      expect(file).toContain('anchor.right <= 0 || anchor.left >= window.innerWidth');
      expect(file).toContain('detailsElement.open = false;');
      expect(file).toContain('playToolPopupStyle(anchor,');
      expect(file).toContain('on:scroll|capture={repositionPopup}');
      expect(file).not.toContain('setInterval(');
    }
  });
  it('shares exclusive native disclosures without creating a second native owner',()=>{
    const f3=source('F3InspectorPanel.svelte');
    const coords=source('CoordinatePastePanel.svelte');
    const agent=source('AgentActivityNotice.svelte');
    const app=source('LazyTesterApp.svelte');
    const css=source('lazy-tester.css');
    for (const file of [f3,coords,agent]) {
      expect(file).toContain('name="lt-play-disclosures"');
      expect(file).toContain('on:focusout={closeWhenFocusLeaves}');
      expect(file).toContain('summaryElement?.focus({preventScroll: true});');
    }
    for (const file of [f3,coords]) {
      expect(file).toContain('playToolPopupStyle(');
      expect(file).toContain('visibility:hidden;');
      expect(file).toContain('style={popupStyle}');
      expect(file).toContain('on:resize={repositionPopup}');
      expect(file).toContain('on:scroll|capture={repositionPopup}');
      expect(file).not.toContain('setInterval(');
      expect(file).not.toContain('ResizeObserver');
      expect(file).not.toContain('invokeRuntime(');
    }
    expect(css).toContain('.lt-coordinate-popup{position:fixed;');
    expect(css).toContain('grid-template-columns:repeat(5,minmax(0,1fr))');
    expect(app).toContain('onSelect={select} onLaunch={launch}');
    expect(app).toContain('<div class="lt-stage-space" bind:this={stageSpace}>');
  });
});
