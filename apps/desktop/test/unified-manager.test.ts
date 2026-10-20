import {it,expect} from 'vitest';
import {readFileSync} from 'node:fs';
import {parseManagerConfiguration,operationIssue} from '../src/app/bridge/managerConfiguration.js';
import {ownedClientStatus} from '../src/app/presentation/unifiedManager.js';
import type {OwnedSlot} from '../src/app/bridge/ownedProviderApi.js';
const catalog=JSON.parse(readFileSync(new URL('../../../engine/isolation-native/owned-runtime/minecraft-targets.json',import.meta.url),'utf8'));
const config=()=>({observation:null,engineError:null,settings:{sessionDirectory:'D:\\engine',minecraftExecutable:'C:\\game.exe'},detectedMinecraft:'C:\\game.exe',targetError:null,catalog:structuredClone(catalog)});
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
});
it('states distinguish ready, blocked retry, active operation, missing profile and close acknowledgement',()=>{
  const slot:OwnedSlot={clientId:'02',box:'MivubiEdu02',profileExists:true,settingsVerified:true,launchAllowed:true,boxedProcessCount:0,status:'STOPPED',processes:[]};
  expect(ownedClientStatus(slot,true,false,null)).toBe('Ready');
  expect(ownedClientStatus({...slot,launchAllowed:false},true,false,null)).toBe('Needs attention');
  expect(ownedClientStatus(slot,true,true,'Launch')).toBe('Starting');
  expect(ownedClientStatus(slot,true,false,'Close')).toBe('Closing');
  expect(ownedClientStatus(slot,false,false,null)).toBe('Needs attention');
  expect(ownedClientStatus({...slot,status:'ABSENT'},true,false,null)).toBe('Not configured');
  expect(ownedClientStatus({...slot,status:'RUNNING'},true,false,null)).toBe('Needs attention');
  const process={pid:77,createdFileTime:'134000000000000001',sessionId:1,windowHandle:'0x123',responding:true};
  expect(ownedClientStatus({...slot,status:'RUNNING',processes:[process]},true,false,null)).toBe('Window detected');
  expect(ownedClientStatus({...slot,status:'RUNNING',processes:[{...process,responding:false}]},true,false,null)).toBe('Needs attention');
});
it('mounted dashboard has no alternate pilot panel or automatic game launch',()=>{
  const source=readFileSync(new URL('../src/UnifiedManagerApp.svelte',import.meta.url),'utf8');
  expect(source).not.toMatch(/CompactManagerApp|inspectIsolatedClient02|setInterval|openOfficialMinecraft/);
  expect(source).toContain('managerConfiguration.load()');
  expect(source).not.toMatch(/managerConfiguration\.save|selectSession|selectEducationExe|Choose executable|Choose engine/);
  expect(source).toContain('Automatic setup');expect(source).toContain('OwnedManagerPanel');
});
