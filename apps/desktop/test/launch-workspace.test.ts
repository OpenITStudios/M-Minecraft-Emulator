import {expect,it,vi} from 'vitest';
import {observeWorkspaceLaunch} from '../src/app/presentation/launchWorkspace.js';
import type {OwnedSnapshot} from '../src/app/bridge/ownedProviderApi.js';
import {readFileSync} from 'node:fs';

it('selecting a running tab refreshes identity and presents it without launching or closing',async()=>{
  const source=readFileSync(new URL('../src/OwnedManagerPanel.svelte',import.meta.url),'utf8');
  const start=source.indexOf('export async function activateClient('),end=source.indexOf('async function play(',start);
  const fn=source.slice(start,end).replace('export ','').replace('client:ClientId','client');
  const run=new Function('act','play','openClient','snapshot',`let operationBusy=false,externalBusy=false,current=true,pending={};const actionBlockedReason=()=>null;${fn};return activateClient('02');`);
  const act=vi.fn(),play=vi.fn(),open=vi.fn();
  await run(act,play,open,{clients:[{clientId:'02',processes:[target]}],nativeProcesses:[]});
  expect(act).toHaveBeenCalledExactlyOnceWith('02','Inspect');
  expect(play).toHaveBeenCalledExactlyOnceWith('02',target);
  expect(open).not.toHaveBeenCalled();
});

it('keeps one session tab bar and removes the duplicate launch cards',()=>{
  const panel=readFileSync(new URL('../src/OwnedManagerPanel.svelte',import.meta.url),'utf8');
  const workspace=readFileSync(new URL('../src/NativeWorkspace.svelte',import.meta.url),'utf8');
  expect(panel).not.toContain('<article');
  expect(panel).not.toContain('>Open</button>');
  expect(workspace).toContain('role="tablist"');
  expect(workspace).toContain('tabStates[client]');
  expect(workspace).toContain('class="nw-feedback"');
  expect(workspace).not.toContain('class="um-alert"');
});

it('a user tab retry requires an empty freshly inspected slot with backend launch permission',async()=>{
  const source=readFileSync(new URL('../src/OwnedManagerPanel.svelte',import.meta.url),'utf8');
  const start=source.indexOf('export async function activateClient('),end=source.indexOf('async function play(',start);
  const fn=source.slice(start,end).replace('export ','').replace('client:ClientId','client');
  const run=new Function('act','play','openClient','snapshot',`let operationBusy=false,externalBusy=false,current=true,pending={'02':true};const actionBlockedReason=()=>pending['02']?'Pending':null;${fn};return activateClient('02');`);
  for(const allowed of [true,false]){
    const inspect=vi.fn(),play=vi.fn(),open=vi.fn();
    await run(inspect,play,open,{clients:[{clientId:'02',processes:[],launchAllowed:allowed}],nativeProcesses:[]});
    expect(inspect).toHaveBeenCalledExactlyOnceWith('02','Inspect');
    expect(open).toHaveBeenCalledTimes(allowed?1:0);expect(play).not.toHaveBeenCalled();
  }
});

it('resumes the retained game after a new profile launch produces no window',async()=>{
  const source=readFileSync(new URL('../src/OwnedManagerPanel.svelte',import.meta.url),'utf8');
  const start=source.indexOf('async function createAndOpen()'),end=source.indexOf('async function cancelSetup()',start);
  expect(start>=0&&end>start).toBe(true);
  const run=new Function('act','onResume',`return (async()=>{
    let setupClient='02',compatibility=false,current=true;
    const setupDialog={close(){}};
    const snapshot={clients:[{clientId:'02',settingsVerified:true}]};
    ${source.slice(start,end)}
    await createAndOpen();
  })()`);
  const act=vi.fn().mockResolvedValue(undefined),resume=vi.fn().mockResolvedValue(undefined);
  await run(act,resume);
  expect(act.mock.calls).toEqual([['02','Create'],['02','Launch']]);
  expect(resume).toHaveBeenCalledOnce();
});

it('only resumes a suspended workspace and preserves an already presented new game',async()=>{
  const source=readFileSync(new URL('../src/NativeWorkspace.svelte',import.meta.url),'utf8');
  const start=source.indexOf('export async function resume()'),end=source.indexOf('export async function reconcile(',start);
  expect(start>=0&&end>start).toBe(true);
  const run=new Function('suspended','present',`let activeClient='01',target={pid:77};
    ${source.slice(start,end).replace('export ','')};return resume();`);
  const present=vi.fn().mockResolvedValue(undefined);
  await run(false,present);expect(present).not.toHaveBeenCalled();
  await run(true,present);expect(present).toHaveBeenCalledExactlyOnceWith('01',{pid:77});
});

const target={pid:77,createdFileTime:'134000000000000001',sessionId:1,windowHandle:'0x123',responding:true};
const snapshot=(visible:boolean):OwnedSnapshot=>({schema:1,provider:'OWNED_V1186_LOCAL_TEST',operation:'Inspect',launcherExitCode:null,focusError:null,clients:[],nativeProcesses:visible?[target]:[]});
it('observes a delayed native window and presents it without a second launch',async()=>{
  const inspect=vi.fn().mockResolvedValueOnce(snapshot(false)).mockResolvedValue(snapshot(true));
  const present=vi.fn(),publish=vi.fn(),pause=vi.fn();
  expect(await observeWorkspaceLaunch('01',inspect,present,publish,pause)).toBe(true);
  expect(inspect).toHaveBeenCalledTimes(2);expect(present).toHaveBeenCalledExactlyOnceWith('01',target);
});
it('bounds observation and never presents a native process as an owned client',async()=>{
  const inspect=vi.fn().mockResolvedValue(snapshot(true)),present=vi.fn(),pause=vi.fn();
  expect(await observeWorkspaceLaunch('02',inspect,present,vi.fn(),pause)).toBe(false);
  expect(inspect).toHaveBeenCalledTimes(8);expect(pause).toHaveBeenCalledTimes(7);expect(present).not.toHaveBeenCalled();
});
