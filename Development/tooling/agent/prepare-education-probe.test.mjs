// Regression fixtures are AUTHORED_ONLY: execution needs separate approval.
// Pack creation and game activation must not happen during tests.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {buildEducationProbePack} from './prepare-education-probe.mjs';

const run='edu-abcdef123456';
const uuids=[
  '11111111-1111-4111-8111-111111111111',
  '22222222-2222-4222-8222-222222222222',
  '33333333-3333-4333-8333-333333333333',
];
const getSource=()=>readFile(new URL('./education-script-probe.js',import.meta.url),'utf8');

test('builds a self-contained test-only behavior pack from the one canonical emitter',async()=>{
  const result=buildEducationProbePack({
    run,apiVersion:'1.17.0',probeSource:await getSource(),uuids,
  });
  assert.equal(result.run,run);
  assert.equal(result.manifest.format_version,2);
  assert.equal(result.manifest.header.uuid,uuids[0]);
  assert.deepEqual(result.manifest.header.min_engine_version,[1,21,60]);
  assert.deepEqual(result.manifest.modules.map(m=>m.type),['data','script']);
  assert.equal(result.manifest.modules[1].entry,'scripts/main.js');
  assert.equal(result.manifest.modules[1].language,'javascript');
  assert.deepEqual(result.manifest.dependencies,[{
    module_name:'@minecraft/server',version:'1.17.0',
  }]);
  assert.deepEqual(JSON.parse(result.manifestText),result.manifest);
  assert.match(result.mainJs,/^import \{system\} from '@minecraft\/server';/m);
  assert.ok(result.mainJs.endsWith(
    'emitEducationScriptLogProbe("edu-abcdef123456");\n'));
  assert.equal(result.mainJs.includes('getAllPlayers('),false);
  assert.equal(result.mainJs.includes('runCommand('),false);
  assert.equal(result.mainJs.includes('http://'),false);
});

test('stable v2 candidate also uses a nonexperimental manifest with explicit dependency',async()=>{
  const result=buildEducationProbePack({
    run,apiVersion:'2.0.0',probeSource:await getSource(),uuids,
  });
  assert.deepEqual(result.manifest.header.min_engine_version,[1,21,90]);
  assert.equal(result.manifest.dependencies[0].version,'2.0.0');
  assert.equal('capabilities' in result.manifest,false);
  assert.equal(result.manifest.format_version,2);
});

test('invalid versions, duplicate IDs, malformed source and unsafe run IDs fail closed',async()=>{
  const probeSource=await getSource();
  const valid={run,apiVersion:'1.17.0',probeSource,uuids};
  for(const v of ['2.7.0-beta','2.7.0-rc.1','0.0.0','latest','1.2','1.2.3.4']) {
    assert.throws(()=>buildEducationProbePack({...valid,apiVersion:v}));
  }
  for(const bad of ['edu-../../../x','../secret','manual-run','edu-1234',0]) {
    assert.throws(()=>buildEducationProbePack({...valid,run:bad}));
  }
  assert.throws(()=>buildEducationProbePack({...valid,uuids:[uuids[0],uuids[0],uuids[2]]}));
  assert.throws(()=>buildEducationProbePack({...valid,uuids:['bad',uuids[1],uuids[2]]}));
  assert.throws(()=>buildEducationProbePack({...valid,probeSource:'console.log("hello")\n'}));
  assert.throws(()=>buildEducationProbePack({...valid,probeSource:probeSource+' '.repeat(4097)}));
});

test('run is represented only as an encoded literal call, never executable input',async()=>{
  const source=await getSource();
  const result=buildEducationProbePack({
    run,apiVersion:'1.17.0',probeSource:source,uuids,
  });
  assert.equal(result.mainJs.slice(0,source.length),source);
  assert.equal(result.mainJs.match(/\nemitEducationScriptLogProbe\("/g)?.length,1);
  assert.equal(result.mainJs.includes('eval('),false);
});
