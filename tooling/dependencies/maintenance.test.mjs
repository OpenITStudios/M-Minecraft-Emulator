import test from 'node:test';
import assert from 'node:assert/strict';
import {stableRelease} from './upgrade-frontend.mjs';
import {auditDependencies} from './audit.mjs';
test('stable registry selection rejects previews and unavailable metadata', async()=>{
  const response=version=>async()=>({ok:true,json:async()=>({version})});
  assert.equal(await stableRelease('vite',response('8.3.4')),'vite@8.3.4');
  await assert.rejects(stableRelease('vite',response('9.0.0-beta.1')));
  await assert.rejects(stableRelease('vite',async()=>({ok:false,status:503})));
});
test('audit continues all graphs, treats outdated findings as informational, and propagates failures',()=>{
  let calls=0;
  assert.equal(auditDependencies((command,args)=>{calls++;return {status:args.includes('outdated')?1:0,stdout:'{"vite":{"current":"1.0.0","latest":"2.0.0"}}'};}),0);
  assert.equal(calls,7);
  assert.equal(auditDependencies((command,args)=>({status:args.includes('outdated')?1:0,stdout:'{"error":{"code":"EAI_AGAIN"}}'})),1);
  calls=0;
  assert.equal(auditDependencies((command,args)=>{calls++;return {status:args.includes('audit')?1:0};}),1);
  assert.equal(calls,7);
  assert.equal(auditDependencies(()=>({error:Error('tool unavailable'),status:null})),1);
});
