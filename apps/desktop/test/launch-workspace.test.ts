import {expect,it,vi} from 'vitest';
import {observeWorkspaceLaunch} from '../src/app/presentation/launchWorkspace.js';
import type {OwnedSnapshot} from '../src/app/bridge/ownedProviderApi.js';
import {readFileSync} from 'node:fs';

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
