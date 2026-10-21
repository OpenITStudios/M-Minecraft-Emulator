import {expect,it} from 'vitest';
import {workspaceViewport} from '../src/app/bridge/workspaceSurface.js';
import {readFileSync} from 'node:fs';
it('keeps focus authority in the foreground manager and geometry in the recovery helper',()=>{
  const source=readFileSync(new URL('../src-tauri/src/engine/workspace_surface.rs',import.meta.url),'utf8');
  const helperPlacement=source.slice(source.indexOf('pub(super) fn place('),source.indexOf('pub(super) fn confirm_foreground('));
  expect(helperPlacement).toContain('SWP_NOZORDER');
  expect(helperPlacement).not.toContain('SetForegroundWindow');
  const foreground=source.slice(source.indexOf('pub(super) fn confirm_foreground('),source.indexOf('pub(super) fn hide('));
  expect(foreground).toContain('foreground != manager && foreground != window');
  expect(foreground).toMatch(/if activate\s*&& foreground == manager\s*&& GetWindowLongPtrW\(window, GWL_STYLE\) & WS_DISABLED as isize == 0/);
  expect(foreground.indexOf('return Ok(())')).toBeLessThan(foreground.indexOf('WS_DISABLED'));
  expect(foreground).toContain('SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE');
  expect(foreground).not.toContain('Workspace foreground activation refused');
  const passive=source.slice(source.indexOf('pub fn update(manager:'),source.indexOf('#[cfg(all(test, windows))]'));
  expect(passive).toContain('lease.place(rect, false)');
  expect(source).toContain('native::confirm_foreground(');
  expect(source).not.toMatch(/AttachThreadInput|AllowSetForegroundWindow|SPI_SETFOREGROUNDLOCKTIMEOUT/);
});
it('keeps fractional CSS coordinates and explicit display scale for native conversion',()=>{
  expect(workspaceViewport({x:40.25,y:80.5,width:960.5,height:540.25},1.5)).toEqual({x:40.25,y:80.5,width:960.5,height:540.25,pixelRatio:1.5});
});
it('retries layout on the next event after a transient error instead of permanently disabling resize',async()=>{
  const source=readFileSync(new URL('../src/NativeWorkspace.svelte',import.meta.url),'utf8');
  const start=source.indexOf('async function update()'),end=source.indexOf('async function returnToWindows()',start);
  const fn=source.slice(start,end);
  let attempts=0;
  const run=new Function('workspaceSurface',`let suspended=false,updatePending=false,updating=false,busy=false,working=false,activeClient='01',failedPresentation=null,issue=null;const viewport={getBoundingClientRect:()=>({})};const operationIssue=e=>e;${fn};return (async()=>{await update();await update();})();`);
  await run({update:async()=>{if(++attempts===1)throw new Error('layout changed');}});
  expect(attempts).toBe(2);
});
it('rejects impossible areas and nonfinite or unsupported display scale before IPC',()=>{
  for(const rect of [{x:-1,y:0,width:800,height:600},{x:0,y:NaN,width:800,height:600},{x:0,y:0,width:0,height:600},{x:19999,y:0,width:800,height:600}])expect(()=>workspaceViewport(rect)).toThrow();
  for(const scale of [0,Infinity,NaN,9])expect(()=>workspaceViewport({x:0,y:0,width:800,height:600},scale)).toThrow();
});
