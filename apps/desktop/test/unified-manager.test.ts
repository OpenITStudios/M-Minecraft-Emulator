import {it,expect} from 'vitest';
import {readFileSync} from 'node:fs';
import {parseManagerConfiguration,operationIssue} from '../src/app/bridge/managerConfiguration.js';
import {ownedClientStatus,clientActionBlockedReason} from '../src/app/presentation/unifiedManager.js';
import type {OwnedSlot} from '../src/app/bridge/ownedProviderApi.js';
const catalog=JSON.parse(readFileSync(new URL('../../../engine/isolation-native/owned-runtime/minecraft-targets.json',import.meta.url),'utf8'));
const config=()=>({observation:null,engineError:null,settings:{sessionDirectory:'D:\\engine',minecraftExecutable:'C:\\game.exe'},detectedMinecraft:'C:\\game.exe',targetError:null,catalog:structuredClone(catalog)});
it('tabs and cards share stale, pending, ambiguous and unresponsive action guards',()=>{
 const process={pid:77,createdFileTime:'134000000000000001',sessionId:1,windowHandle:'0x123',responding:true};
 const snapshot={nativeProcesses:[],clients:[{clientId:'02' as const,box:'MivubiEdu02',profileExists:true,settingsVerified:true,launchAllowed:true,boxedProcessCount:0,status:'STOPPED',processes:[]}]};
 expect(clientActionBlockedReason(snapshot,'02',true,false,false)).toMatch(/Sandboxie is retired/);
 expect(clientActionBlockedReason(snapshot,'02',false,false,false)).toMatch(/Check all clients/);
 expect(clientActionBlockedReason(snapshot,'02',true,true,false)).toMatch(/launch/);
 expect(clientActionBlockedReason({...snapshot,nativeProcesses:[{...process,responding:false}]},'01',true,false,false)).toMatch(/responding/);
 expect(clientActionBlockedReason({...snapshot,nativeProcesses:[process,process]},'01',true,false,false)).toMatch(/multiple/);
 expect(clientActionBlockedReason({...snapshot,nativeProcesses:[process]},'01',true,false,false)).toBe(null);
 expect(clientActionBlockedReason({...snapshot,clients:[{...snapshot.clients[0],launchAllowed:false}]},'02',true,false,false)).not.toBe(null);
 expect(clientActionBlockedReason(snapshot,'02',true,false,true)).toMatch(/closing/);
});
it('configuration has exactly four canonical targets and no runtime readiness authority',()=>{
  expect(parseManagerConfiguration(config()).catalog.targets.map(t=>t.id)).toEqual(['01','02','03','04']);
  const wrong=config();wrong.catalog.targets[2].profile='MivubiEdu02';expect(()=>parseManagerConfiguration(wrong)).toThrow();
  expect(()=>parseManagerConfiguration({...config(),settings:{sessionDirectory:3}})).toThrow();
  expect(parseManagerConfiguration({...config(),targetError:'installation changed',detectedMinecraft:null}).targetError).toBe('installation changed');
});
it('structured failures keep technical output out of the primary message',()=>{
  const issue=operationIssue({code:'LAUNCH_PENDING',message:'Launch pending',action:'Check again',details:'At line:123 secret-stack'});
  expect(issue.message).toBe('Launch pending');expect(issue.details).toContain('line:123');expect(issue.message).not.toContain('line:123');
  expect(operationIssue('Recent launch request').message).toContain('pending');
  expect(operationIssue('APP_COMPARTMENT_FEATURE_UNAVAILABLE: evaluation').action).toContain('five minutes');
  expect(operationIssue('Surface helper timed out after 3000 ms at place/position').action).toContain('Retry display');
  expect(operationIssue('Workspace game is covered; presentation was not confirmed').message).toBe('Minecraft is hidden behind the manager.');
});
it('states distinguish ready, blocked retry, active operation, missing profile and close acknowledgement',()=>{
  const slot:OwnedSlot={clientId:'02',box:'MivubiEdu02',profileExists:true,settingsVerified:true,launchAllowed:true,boxedProcessCount:0,status:'STOPPED',processes:[]};
  expect(ownedClientStatus(slot,true,false,null)).toBe('Unavailable');
  expect(ownedClientStatus({...slot,launchAllowed:false},true,false,null)).toBe('Unavailable');
  expect(ownedClientStatus(slot,true,true,'Launch')).toBe('Starting');
  expect(ownedClientStatus(slot,true,false,'Close')).toBe('Closing');
  expect(ownedClientStatus(slot,false,false,null)).toBe('Needs attention');
  expect(ownedClientStatus({...slot,status:'ABSENT'},true,false,null)).toBe('Unavailable');
  expect(ownedClientStatus({...slot,status:'RUNNING'},true,false,null)).toBe('Unavailable');
  const process={pid:77,createdFileTime:'134000000000000001',sessionId:1,windowHandle:'0x123',responding:true};
  expect(ownedClientStatus({...slot,status:'RUNNING',processes:[process]},true,false,null)).toBe('Running');
  expect(ownedClientStatus({...slot,status:'RUNNING',processes:[{...process,responding:false}]},true,false,null)).toBe('Needs attention');
});
it('mounted dashboard has no alternate pilot panel or automatic game launch',()=>{
  const source=readFileSync(new URL('../src/UnifiedManagerApp.svelte',import.meta.url),'utf8');
  expect(source).not.toMatch(/CompactManagerApp|inspectIsolatedClient02|setInterval/);
  expect(source).toContain('async function activateNative(client:ClientId)');
  expect(source).toContain('await nativeFeasibility.openOfficialMinecraft()');
  const mounted=source.slice(source.indexOf('onMount('),source.indexOf('</script>'));
  expect(mounted).not.toContain('openOfficialMinecraft');
  expect(source).toContain('managerConfiguration.load()');
  expect(source).not.toMatch(/managerConfiguration\.save|selectSession|selectEducationExe|Choose executable|Choose engine/);
  expect(source).toContain('Read-only status');expect(source).toContain('OwnedManagerPanel');
});

it('observed legacy snapshot never promotes independent-engine readiness in mounted UI',()=>{
  const source=readFileSync(new URL('../src/UnifiedManagerApp.svelte',import.meta.url),'utf8');
  // checked is only a snapshot-present flag; this cannot be advertised as
  // verified engine isolation or even as a connected independent provider.
  expect(source).toContain('snapshot=config.observation;checked=!!snapshot');
  expect(source).toContain('Native Windows · legacy recovery on request');
  expect(source).toContain('Observed · diagnostic only');
  expect(source).toContain("config=sessionView==='legacy'?await managerConfiguration.loadLegacy():await managerConfiguration.load()");
  expect(source).toContain('on:click={openLegacy}');
  expect(source).toContain('Not inspected automatically · open recovery explicitly');
  expect(source).toContain('Not implemented · Client 02–04 Launch/Provision disabled.');
  expect(source).toContain('account isolation or gameplay readiness');
  expect(source).not.toMatch(/Connected · verified for this session|class:ready=\{checked\}|Minecraft and the engine are detected automatically\./);
  expect(source).not.toContain('Engine location</h3>');
  // Windows identity checks still provide their own explicit non-account disclaimer.
  expect(source).toContain('Account isolation remains unproven.');
  expect(source).toContain('releaseOnly sessionDirectory=""');
  expect(source).toContain('Independent of retired engine snapshots');
});

it('native observed windows are primary even when a historical session can be recovered',()=>{
  const source=readFileSync(new URL('../src/UnifiedManagerApp.svelte',import.meta.url),'utf8');
  expect(source).toContain("sessionView:'native'|'legacy'='native'");
  expect(source).toContain("config && snapshot && sessionView==='legacy'");
  expect(source).toContain('reconcileObservedSlots(observedSlots,nativeSession.processes)');
  expect(source).toContain('This slot has no observed window. New isolated Minecraft 02–04 launches are not available.');
  expect(source).toContain('Inspect existing-session recovery');
  expect(source).not.toContain('for(const session of running)');
  const bootstrap=readFileSync(new URL('../src-tauri/src/app_bootstrap.rs',import.meta.url),'utf8');
  expect(bootstrap).toContain('commands::native_clients::desktop_present_observed_window,');
});


it('normal Minecraft startup does not query the retired service or execute owned test recovery',()=>{
  const backend=readFileSync(new URL('../src-tauri/src/engine/manager_settings.rs',import.meta.url),'utf8');
  const command=readFileSync(new URL('../src-tauri/src/commands/native_clients.rs',import.meta.url),'utf8');
  const bridge=readFileSync(new URL('../src/app/bridge/managerConfiguration.ts',import.meta.url),'utf8');
  expect(backend).toContain('prepare_with_legacy_mode(requested, false)');
  expect(backend).toContain('prepare_with_legacy_mode(requested, true)');
  const p=backend.slice(backend.indexOf('fn prepare_with_legacy_mode('));
  const guard=p.indexOf('if include_legacy {');
  expect(guard).toBeGreaterThan(0);
  expect(p.indexOf('discover_active_session()')).toBeGreaterThan(guard);
  expect(p.indexOf('OwnedTestBackend { request: &request }')).toBeGreaterThan(guard);
  expect(command).toContain('include_legacy: Option<bool>');
  expect(command).toContain('include_legacy == Some(true)');
  expect(bridge).toContain('includeLegacy:true');
  expect(bridge).toContain("{settings:null}");
});
