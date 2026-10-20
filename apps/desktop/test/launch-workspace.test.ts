import {expect,it,vi} from 'vitest';
import {observeWorkspaceLaunch} from '../src/app/presentation/launchWorkspace.js';
import type {OwnedSnapshot} from '../src/app/bridge/ownedProviderApi.js';

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
