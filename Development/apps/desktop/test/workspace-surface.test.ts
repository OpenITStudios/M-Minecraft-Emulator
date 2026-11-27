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

it('Play switch uses one verified presentation path after original session observation',()=>{
  const source=readFileSync(new URL('../src-tauri/src/commands/native_clients.rs',import.meta.url),'utf8');
  const start=source.indexOf('pub async fn lazy_tester_present(');
  const end=source.indexOf('pub async fn lazy_tester_previews(',start);
  const handler=source.slice(start,end);
  expect(handler).not.toContain('present_retained(');
  expect(handler).toContain('SessionRequest::Inspect');
  expect(handler).toContain('.filter(|p| p.responding)');
  expect(handler.indexOf('SessionRequest::Inspect')).toBeLessThan(handler.indexOf('present_verified('));
  expect(handler.match(/present_verified\(/g)).toHaveLength(1);
  expect(handler).not.toContain('native_clients::inspect_native_workspace');
});


it('empty unowned layout performs no session inspection but retained or requested sources still do',()=>{
  const source=readFileSync(new URL('../src-tauri/src/commands/native_clients.rs',import.meta.url),'utf8');
  const handler=source.slice(source.indexOf('pub async fn lazy_tester_previews('),source.indexOf('pub async fn lazy_tester_clear_previews('));
  expect(handler).toMatch(/if tiles\.is_empty\(\) && !crate::engine::workspace_surface::is_active\(\) \{\s*Vec::new\(\)\s*\} else \{[\s\S]*?SessionRequest::Inspect/);
  expect(handler.indexOf('run_manager_operation')).toBeLessThan(handler.indexOf('tiles.is_empty()'));
  expect(handler.match(/SessionRequest::Inspect/g)).toHaveLength(1);
  expect(handler.indexOf('configure_previews(')).toBeGreaterThan(handler.indexOf('SessionRequest::Inspect'));
});

it('validates preview overlap in one manager coordinate sample and keeps fresh native placement',()=>{
  const previews=readFileSync(new URL('../src-tauri/src/engine/workspace_surface/previews.rs',import.meta.url),'utf8');
  const admission=previews.slice(previews.indexOf('let active = controller::active_preview_slot'),previews.indexOf('let monitored ='));
  expect(admission.match(/native::capture_manager_geometry\(manager\)/g)).toHaveLength(1);
  expect(admission).toContain('geometry.rectangle(manager, viewport.validate()?)');
  expect(admission).toContain('geometry.rectangle(manager, tile.viewport)');
  expect(admission).not.toContain('native::rectangle(');
  expect(admission).toContain('overlaps(main, *rect)');
  expect(admission).toContain('require_current_admission(&issued_epochs)');
  const controller=readFileSync(new URL('../src-tauri/src/engine/workspace_surface/controller.rs',import.meta.url),'utf8');
  const adoption=controller.slice(controller.indexOf('pub(super) fn adopt_preview_sources'),controller.indexOf('#[cfg(test)]\nfn present('));
  expect(adoption).toContain('native::rectangle(manager, viewport.validate()?)');
  const worker=readFileSync(new URL('../src-tauri/src/engine/workspace_surface/previews/windows.rs',import.meta.url),'utf8');
  expect(worker).toContain('let rect = native::rectangle(manager, tile.viewport)?;');
  expect(worker).toContain('let rect=native::rectangle(self.manager,self.viewport)?;');
});

it('borrows requested preview sources without a second cloned process roster',()=>{
  const previews=readFileSync(new URL('../src-tauri/src/engine/workspace_surface/previews.rs',import.meta.url),'utf8');
  const adoption=previews.slice(previews.indexOf('let monitored ='),previews.indexOf('let primary =',previews.indexOf('let monitored =')));
  expect(adoption).toContain('profiles.iter().filter(');
  expect(adoption).not.toMatch(/\.cloned\(|\.collect\(/);
  expect(adoption).toContain('adopt_preview_sources(manager, viewport, monitored)');
  expect(adoption).toContain('require_current_admission(&issued_epochs)');
});

it('skips native placement work for an empty workspace while preserving DWM and contended admission',()=>{
  const bootstrap=readFileSync(new URL('../src-tauri/src/app_bootstrap.rs',import.meta.url),'utf8');
  const controller=readFileSync(new URL('../src-tauri/src/engine/workspace_surface/controller.rs',import.meta.url),'utf8');
  const queue=bootstrap.slice(bootstrap.indexOf('fn queue_workspace_update('),bootstrap.indexOf('fn show_startup_error('));
  expect(queue).toContain('if !crate::engine::workspace_surface::is_active()');
  expect(queue.indexOf('if !crate::engine::workspace_surface::is_active()')).toBeLessThan(queue.indexOf('PENDING.lock()'));
  expect(queue.indexOf('if !crate::engine::workspace_surface::is_active()')).toBeLessThan(queue.indexOf('spawn_blocking('));
  const active=controller.slice(controller.indexOf('pub fn is_active()'),controller.indexOf('pub fn release_for_close()'));
  expect(active).toContain('s.active.is_some() || !s.parked.is_empty()');
  expect(active).toContain('.unwrap_or(true)');
  const events=bootstrap.slice(bootstrap.indexOf('if matches!(event, tauri::WindowEvent::Moved'),bootstrap.indexOf('.run(context)'));
  expect(events.match(/workspace_surface::refresh_previews\(\)/g)).toHaveLength(2);
  const foreground=bootstrap.slice(bootstrap.indexOf('unsafe extern "system" fn changed('),bootstrap.indexOf('pub fn start(window: tauri::Window)'));
  expect(foreground).toContain('workspace_surface::refresh_previews()');
  expect(foreground).toContain('super::queue_workspace_update(window.clone())');
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
