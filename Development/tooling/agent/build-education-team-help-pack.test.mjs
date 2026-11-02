// Source-authored fixtures ONLY. No Node/game tests executed at this SHA.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {buildEducationMcpack} from './build-education-team-help-pack.mjs';

const entries=[
  ['manifest.json',new URL('./education-team-help-pack/manifest.json',import.meta.url)],
  ['scripts/main.js',new URL('./education-team-help-pack/scripts/main.js',import.meta.url)]
];
async function readEntries(){
  return Promise.all(entries.map(async ([name,url])=>
    ({name,content:await readFile(url)})));
}

test('major version retires own telemetry but retains original Education pack UUID',async()=>{
  const m=JSON.parse((await readFile(entries[0][1])).toString('utf8'));
  assert.equal(m.format_version,2);
  assert.equal(m.header.name,'Lazy Tester - Team Commands Help');
  assert.equal(m.header.uuid,'4af9d3f0-9fa1-4cb9-a011-fd37c1b2fb16');
  assert.deepEqual(m.header.version,[2,0,0]);
  assert.deepEqual(m.modules.map(x=>x.version),[[2,0,0],[2,0,0]]);
  assert.deepEqual(m.header.min_engine_version,[1,26,30]);
  assert.deepEqual(m.dependencies,[{module_name:'@minecraft/server',version:'2.8.0'}]);
  assert.equal(m.modules[1].entry,'scripts/main.js');
});

test('help displays canonical developer commands but contains no custom analytics',async()=>{
  const source=(await readFile(entries[1][1])).toString('utf8');
  assert.match(source,/const HELP_EVENT = "lazytester:help"/);
  assert.match(source,/const HELP_MESSAGE = "commands"/);
  for(const cmd of [
    '/scriptevent mivubi:get_analytics',
    '/scriptevent daigon:get_all_scoring',
    '/scriptevent probe:list',
    '/scriptevent probe:run auto',
    '/scriptevent probe:report',
  ])assert.equal(source.includes(cmd),true,cmd);
  assert.equal(source.includes('/scriptevent lt:diag'),false);
  assert.equal(source.includes('[LT_EDU_DIAG]'),false);
  assert.equal(source.includes('initAnalyticsEvents('),false);
  assert.equal(source.includes('initGameResultsExport('),false);
  assert.equal(source.includes('system.runInterval('),false);
  assert.equal(source.includes('world.afterEvents'),false);
  assert.equal(source.includes('console.log('),false);
  assert.equal(source.includes('console.warn('),false);
  assert.equal(source.includes('world.sendMessage('),false);
  assert.equal(source.includes('player.sendMessage('),true);
});

test('help is player-scoped, has no command execution or export side effects',async()=>{
  const source=(await readFile(entries[1][1])).toString('utf8');
  assert.match(source,/event\.sourceEntity/);
  assert.match(source,/player instanceof Player/);
  assert.match(source,/event\.id !== HELP_EVENT/);
  assert.match(source,/event\.message \?\? ""/);
  assert.doesNotMatch(source,/runCommand\(|getAllPlayers\(|setDynamicProperty\(|setScore\(|getEntities\(/);
  assert.doesNotMatch(source,/fetch\(|XMLHttpRequest|createServer/);
});

test('fixed ZIP root contains only two required files',async()=>{
  const e=await readEntries();
  const bytes=buildEducationMcpack(e);
  assert.ok(bytes.length>0&&bytes.length<256*1024);
  assert.equal(bytes.readUInt32LE(0),0x04034b50);
  assert.equal(bytes.readUInt16LE(8),0);
  assert.equal(bytes.readUInt32LE(bytes.length-22),0x06054b50);
  assert.equal(bytes.readUInt16LE(bytes.length-22+10),2);
  for(const name of ['manifest.json','scripts/main.js']){
    assert.ok(bytes.includes(Buffer.from(name,'ascii')));
  }
  assert.equal(Buffer.compare(bytes,buildEducationMcpack(e)),0);
});

test('ZIP rejects altered entry names and oversized payloads',()=>{
  const good=Buffer.from('{}'),module=Buffer.from('script');
  assert.throws(()=>buildEducationMcpack([{name:'../manifest.json',content:good},
    {name:'scripts/main.js',content:module}]));
  assert.throws(()=>buildEducationMcpack([{name:'manifest.json',content:good}]));
  assert.throws(()=>buildEducationMcpack([{name:'manifest.json',content:good},
    {name:'scripts/main.js',content:Buffer.alloc(65537)}]));
});
