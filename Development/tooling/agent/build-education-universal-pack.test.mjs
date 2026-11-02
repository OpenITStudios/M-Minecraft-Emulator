// Authored regression fixtures only. No tests executed in this source-only stage.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {buildEducationMcpack} from './build-education-universal-pack.mjs';

const files=[
  ['manifest.json',new URL('./education-universal-pack/manifest.json',import.meta.url)],
  ['scripts/main.js',new URL('./education-universal-pack/scripts/main.js',import.meta.url)],
];
const read=async()=>Promise.all(files.map(async ([name,url])=>({
  name,content:await readFile(url),
})));

test('manifest is one reusable Education script pack with stable pinned identifiers',async()=>{
  const m=JSON.parse((await readFile(files[0][1])).toString('utf8'));
  assert.equal(m.format_version,2);
  assert.equal(m.header.name,'Lazy Tester - Universal Diagnostics');
  assert.deepEqual(m.header.version,[1,2,0]);
  assert.match(m.header.description,/lt:diag help/);
  assert.deepEqual(m.header.min_engine_version,[1,26,30]);
  assert.equal(m.header.uuid,'4af9d3f0-9fa1-4cb9-a011-fd37c1b2fb16');
  assert.deepEqual(m.modules.map(x=>x.type),['data','script']);
  assert.equal(m.modules[1].entry,'scripts/main.js');
  assert.deepEqual(m.dependencies,[{module_name:'@minecraft/server',version:'2.8.0'}]);
});

test('universal pack has no test-world-specific run ID or active logging by default',async()=>{
  const source=(await readFile(files[1][1])).toString('utf8');
  assert.match(source,/const COMMAND = "lazytester:diagnostics";/);
  assert.match(source,/const SIMPLE_COMMAND = "lt:diag";/);
  assert.match(source,/let active = false;/);
  assert.match(source,/const SUMMARY_TICKS = 200;/);
  assert.match(source,/const FOCUS_TICKS = 600;/);
  assert.match(source,/scriptEventReceive\.subscribe/);
  assert.match(source,/command==="combat on"/);
  assert.match(source,/command==="entities on"/);
  assert.match(source,/function start\(\)/);
  assert.match(source,/function stop\(\)/);
  assert.doesNotMatch(source,/getAllPlayers\(|getEntities\(|setDynamicProperty\(|runCommand\(|fetch\(/);
  assert.doesNotMatch(source,/\b(mc0[1-5]|my-world|my-map)\b/i);
});

test('ZIP archive contains only fixed files, correct local and central headers',async()=>{
  const entries=await read();
  const zip=buildEducationMcpack(entries);
  assert.ok(zip.length>0&&zip.length<256*1024);
  assert.equal(zip.readUInt32LE(0),0x04034b50);
  assert.equal(zip.readUInt16LE(8),0); // STORED method
  assert.equal(zip.readUInt32LE(zip.length-22),0x06054b50);
  const end=zip.length-22;
  assert.equal(zip.readUInt16LE(end+10),2);
  const dirOffset=zip.readUInt32LE(end+16);
  assert.equal(zip.readUInt32LE(dirOffset),0x02014b50);
  // No extra third file, zip slip or manifest nested under pack/.
  for(const name of ['manifest.json','scripts/main.js']){
    assert.ok(zip.includes(Buffer.from(name,'ascii')));
  }
  assert.equal(Buffer.compare(zip,buildEducationMcpack(entries)),0);
});

test('team analytics tier 1 is default; noisy entity and hurt subscriptions are opt-in only',async()=>{
  const src=(await readFile(files[1][1])).toString('utf8');
  const baseline=src.slice(src.indexOf('function start(){'),src.indexOf('// Explicit short-lived focus'));
  assert.match(baseline,/world\.afterEvents\.playerSpawn/);
  assert.match(baseline,/world\.afterEvents\.playerLeave/);
  assert.match(baseline,/world\.afterEvents\.playerDimensionChange/);
  assert.match(baseline,/FILTER_PLAYER/);
  assert.doesNotMatch(baseline,/entityHurt/);
  assert.doesNotMatch(baseline,/entitySpawn/);
  const focus=src.slice(src.indexOf('function enableFocus('),src.indexOf('function status()'));
  assert.match(focus,/subscribe\("combat",world\.afterEvents\.entityHurt/);
  assert.match(focus,/subscribe\("entities",world\.afterEvents\.entitySpawn/);
  assert.match(focus,/FOCUS_TICKS/);
  assert.match(focus,/focusGeneration\[group\]===generation/);
  // Comments may mention the team marker names as documentation.
  // Assert runtime marker ownership, not incidental source comments.
  assert.equal(src.includes('const MARKER = "[LT_EDU_DIAG]"'),true);
  assert.equal(src.includes('const MARKER = "[ANALYTICS]"'),false);
  assert.equal(src.includes('const MARKER = "[GAME_RESULTS]"'),false);
});

test('short help/on/off/status/more aliases preserve the old team command',async()=>{
  const source=(await readFile(files[1][1])).toString('utf8');
  assert.match(source,/if\(e\.id!==COMMAND&&e\.id!==SIMPLE_COMMAND\)return/);
  assert.match(source,/command==="on"\|\|command==="start"/);
  assert.match(source,/command==="off"\|\|command==="stop"/);
  assert.match(source,/command==="help"\|\|command==="\?"/);
  assert.match(source,/command==="status"/);
  assert.match(source,/command==="more"/);
  assert.match(source,/command==="combat on"/);
  assert.match(source,/command==="entities on"/);
  assert.match(source,/if\(!focus\.combat\)enableFocus\("combat",true\)/);
  assert.match(source,/if\(!focus\.entities\)enableFocus\("entities",true\)/);
  assert.match(source,/world\.sendMessage\(text\)/);
  // In-game chat only responds to explicit commands, never each tick/event.
  const heartbeat=source.slice(source.indexOf('// Single low-frequency heartbeat'));
  assert.doesNotMatch(heartbeat,/sendMessage\(/);
  assert.equal(source.includes('getAllPlayers('),false);
});

test('help is available even when capture is OFF and no active subscription exists',async()=>{
  const src=(await readFile(files[1][1])).toString('utf8');
  assert.match(src,/let active = false;/);
  const command=src.slice(src.indexOf('scriptEventReceive.subscribe'));
  assert.ok(command.indexOf('command==="help"')<
    command.indexOf('command==="on"'));
  assert.match(src,/function showHelp\(event\)/);
  assert.match(src,/for\(const line of HELP_LINES\)replyToOperator\(event,line\)/);
  assert.match(src,/const SIMPLE_COMMAND = "lt:diag"/);
  assert.match(src,/The canonical analytics\/scoring contracts remain owned by the team skills/);
});

test('ZIP refuses missing, unknown or oversized entries',()=>{
  const a=Buffer.from('{}'),b=Buffer.from('script');
  assert.throws(()=>buildEducationMcpack([{name:'../../secrets',content:a},
    {name:'scripts/main.js',content:b}]));
  assert.throws(()=>buildEducationMcpack([{name:'manifest.json',content:a}]));
  assert.throws(()=>buildEducationMcpack([{name:'manifest.json',content:a},
    {name:'scripts/main.js',content:Buffer.alloc(64*1024+1)}]));
});
