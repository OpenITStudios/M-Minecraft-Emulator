import {expect,it} from 'vitest';
import {workspaceViewport} from '../src/app/bridge/workspaceSurface.js';
import {observeDisplayScale} from '../src/app/workspaceGeometry.js';
import {readFileSync} from 'node:fs';
it('keeps focus authority in the foreground manager and geometry in the recovery helper',()=>{
  const source=readFileSync(new URL('../src-tauri/src/engine/workspace_surface/native.rs',import.meta.url),'utf8');
  const controller=readFileSync(new URL('../src-tauri/src/engine/workspace_surface/controller.rs',import.meta.url),'utf8');
  const helperPlacement=source.slice(source.indexOf('pub(super) fn place('),source.indexOf('pub(super) fn confirm_foreground('));
  expect(helperPlacement).toContain('SWP_NOZORDER');
  expect(helperPlacement).not.toContain('SetForegroundWindow');
  const foreground=source.slice(source.indexOf('pub(super) fn confirm_foreground('),source.indexOf('pub(super) fn hide('));
  expect(foreground).toContain('foreground != manager && foreground != window');
  expect(foreground).toMatch(/if activate\s*&& foreground == manager\s*&& GetWindowLongPtrW\(window, GWL_STYLE\) & WS_DISABLED as isize == 0/);
  expect(foreground.indexOf('return Ok(())')).toBeLessThan(foreground.indexOf('WS_DISABLED'));
  expect(foreground).toContain('SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE');
  expect(foreground).not.toContain('Workspace foreground activation refused');
  const passive=controller.slice(controller.indexOf('pub fn update(manager:'),controller.indexOf('#[cfg(all(test, windows))]'));
  expect(passive).toContain('lease.place(rect, false)');
  expect(controller).toContain('native::confirm_foreground(');
  expect(passive.indexOf('native::can_park(&saved.target)')).toBeLessThan(passive.indexOf('native::minimized(manager)'));
  expect(source).toContain('manager_foreground_exposes_game(&saved.target,window,manager,expected)');
  expect(source).not.toMatch(/AttachThreadInput|AllowSetForegroundWindow|SPI_SETFOREGROUNDLOCKTIMEOUT/);
});
it('keeps fractional CSS coordinates and explicit display scale for native conversion',()=>{
  expect(workspaceViewport({x:40.25,y:80.5,width:960.5,height:540.25},1.5)).toEqual({x:40.25,y:80.5,width:960.5,height:540.25,pixelRatio:1.5});
});
it('rejects impossible areas and nonfinite or unsupported display scale before IPC',()=>{
  for(const rect of [{x:-1,y:0,width:800,height:600},{x:0,y:NaN,width:800,height:600},{x:0,y:0,width:0,height:600},{x:19999,y:0,width:800,height:600}])expect(()=>workspaceViewport(rect)).toThrow();
  for(const scale of [0,Infinity,NaN,9])expect(()=>workspaceViewport({x:0,y:0,width:800,height:600},scale)).toThrow();
});

it('retained workspace windows require positive OS identity even when native queries fail',()=>{
  const source=readFileSync(new URL('../src-tauri/src/engine/workspace_surface/native.rs',import.meta.url),'utf8');
  const retired=source.slice(source.indexOf('pub(super) fn retired(target: &OwnedProcess)'),source.indexOf('pub(super) fn rectangle(manager:',source.indexOf('pub(super) fn retired(target: &OwnedProcess)')));
  expect(retired).toContain('return true;');
  expect(retired).toContain('if process.is_null() {');
  expect(retired).toContain('windows_process_time::creation_filetime');
  expect(retired).toContain('!birth.is_ok_and(|value| value == key.created_file_time)');
  expect(retired).not.toContain('return GetLastError() == ERROR_INVALID_PARAMETER;');
});

it('Play switch uses original session observation before retained or verified presentation',()=>{
  const source=readFileSync(new URL('../src-tauri/src/commands/native_clients.rs',import.meta.url),'utf8');
  const start=source.indexOf('pub async fn lazy_tester_present(');
  const end=source.indexOf('pub async fn lazy_tester_previews(',start);
  const handler=source.slice(start,end);
  expect(handler).toContain('present_retained(');
  expect(handler).toContain('SessionRequest::Inspect');
  expect(handler).toContain('.filter(|p| p.responding)');
  expect(handler.indexOf('SessionRequest::Inspect')).toBeLessThan(handler.indexOf('present_retained('));
  expect(handler.indexOf('present_retained(')).toBeLessThan(handler.indexOf('present_verified('));
  expect(handler).not.toContain('native_clients::inspect_native_workspace');
});


it('rearms display-scale observation once and removes the current listener on teardown',()=>{
  let ratio=1;
  let calls=0;
  const queries:{text:string;listeners:Set<()=>void>}[]=[];
  const host={get devicePixelRatio(){return ratio;},matchMedia(text:string){
    const listeners=new Set<()=>void>();queries.push({text,listeners});
    return {addEventListener(_event:string,cb:()=>void){listeners.add(cb);},removeEventListener(_event:string,cb:()=>void){listeners.delete(cb);}} as unknown as MediaQueryList;
  }};
  const stop=observeDisplayScale(host,()=>{calls++;});
  expect(queries[0].text).toBe('(resolution: 1dppx)');
  ratio=1.5;[...queries[0].listeners].forEach(cb=>cb());
  expect(queries[0].listeners.size).toBe(0);
  expect(queries[1].text).toBe('(resolution: 1.5dppx)');
  expect(calls).toBe(1);
  const queued=[...queries[1].listeners][0]!;
  stop();expect(queries[1].listeners.size).toBe(0);
  ratio=2;queued(); // An already queued callback cannot revive a disposed observer.
  expect(calls).toBe(1);
  expect(queries).toHaveLength(2);
});
it('routes native viewport placement through the shared edge conversion',()=>{
  const native=readFileSync(new URL('../src-tauri/src/engine/workspace_surface/native.rs',import.meta.url),'utf8');
  const placement=native.slice(native.indexOf('pub(super) fn rectangle(manager:'));
  expect(placement).toContain('= scaled_client_rectangle(viewport, scale);');
});


it('keeps resize observation available when resolution media queries are unsupported',()=>{
  const host={devicePixelRatio:1,matchMedia:undefined} as unknown as Pick<Window,'matchMedia'|'devicePixelRatio'>;
  const stop=observeDisplayScale(host,()=>{throw new Error('unexpected change');});
  expect(()=>stop()).not.toThrow();
});
