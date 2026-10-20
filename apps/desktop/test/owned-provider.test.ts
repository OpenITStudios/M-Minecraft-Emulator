import {expect,it} from "vitest";
import {readFileSync} from "node:fs";
import {parseOwnedSnapshot} from "../src/app/bridge/ownedProviderApi.js";
const stopped=()=>({schema:1,provider:"OWNED_V1186_LOCAL_TEST",operation:"Inspect",launcherExitCode:null,focusError:null,nativeProcesses:[],clients:["02","03","04"].map(clientId=>({clientId,box:`MivubiEdu${clientId}`,profileExists:true,settingsVerified:true,boxedProcessCount:0,launchAllowed:true,status:"STOPPED",processes:[]}))});
const process=()=>({pid:77,createdFileTime:"134000000000000001",sessionId:1,windowHandle:"0x123",responding:true});
it("accepts exactly three known owned profiles without inferring gameplay",()=>{
  const result=parseOwnedSnapshot(stopped());expect(result.clients.map(s=>s.clientId)).toEqual(["02","03","04"]);
  expect(result).not.toHaveProperty("authenticated");expect(result).not.toHaveProperty("gameplayVerified");
});
it("rejects foreign provider, profiles, reordered slots and unknown authority flags",()=>{
  expect(()=>parseOwnedSnapshot({...stopped(),provider:"OFFICIAL_SANDBOXIE"})).toThrow();
  const wrong=stopped();wrong.clients[1].box="foreign";expect(()=>parseOwnedSnapshot(wrong)).toThrow();
  const reordered=stopped();reordered.clients.reverse();expect(()=>parseOwnedSnapshot(reordered)).toThrow();
  expect(()=>parseOwnedSnapshot({...stopped(),ready:true})).toThrow();
});
it("rejects a PID attributed to native and owned at the same time",()=>{
  const snapshot=stopped() as unknown as Record<string,unknown>;
  snapshot.nativeProcesses=[process()];
  const slots=snapshot.clients as Record<string,unknown>[];
  Object.assign(slots[0],{boxedProcessCount:1,launchAllowed:false,status:"RUNNING",processes:[process()]});
  expect(()=>parseOwnedSnapshot(snapshot)).toThrow();
});
it("rejects missing start time, false running window and inconsistent empty state",()=>{
  for(const patch of [{createdFileTime:"0"},{createdFileTime:134000000000000001},{windowHandle:"0x0"},{sessionId:-1}]){
    const snapshot=stopped() as unknown as Record<string,unknown>;
    Object.assign((snapshot.clients as Record<string,unknown>[])[0],{boxedProcessCount:1,launchAllowed:false,status:"RUNNING",processes:[{...process(),...patch}]});
    expect(()=>parseOwnedSnapshot(snapshot)).toThrow();
  }
  const snapshot=stopped();snapshot.clients[0].boxedProcessCount=1;expect(()=>parseOwnedSnapshot(snapshot)).toThrow();
});
it("mounted owned UI never auto-launches, force-kills, polls, or treats launcher exit as readiness",()=>{
  const panel=readFileSync(new URL("../src/OwnedManagerPanel.svelte",import.meta.url),"utf8");
  expect(panel).toContain('operationBusy||externalBusy');
  expect(panel).toContain('pending[client]=true');
  expect(panel).toContain('Close normally');
  expect(panel).toContain('Retry after empty check');
  expect(panel).not.toMatch(/setInterval|setTimeout|onMount|TerminateProcess/);
});
it("represents focus refusal separately from a successful launch and rejects malformed focus feedback",()=>{
  const value={...stopped(),operation:'Launch',launcherExitCode:0,focusError:'Windows refused focus'};
  expect(parseOwnedSnapshot(value).launcherExitCode).toBe(0);
  expect(parseOwnedSnapshot(value).focusError).toContain('refused');
  expect(()=>parseOwnedSnapshot({...value,focusError:123})).toThrow();
  expect(()=>parseOwnedSnapshot({...value,focusError:'x'.repeat(2049)})).toThrow();
  const panel=readFileSync(new URL('../src/OwnedManagerPanel.svelte',import.meta.url),'utf8');
  expect(panel).toContain('opened in the background. Click Focus to show it.');
});
