// SOURCE-ONLY fixtures. Node tests, pack build and Minecraft are NOT_EXECUTED.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {buildEducationMcpack} from './build-education-team-help-pack.mjs';

const source=[
  ['manifest.json',new URL('./education-team-help-pack/manifest.json',import.meta.url)],
  ['functions/help.mcfunction',new URL('./education-team-help-pack/functions/help.mcfunction',import.meta.url)],
  ['functions/lazytester/help.mcfunction',new URL('./education-team-help-pack/functions/lazytester/help.mcfunction',import.meta.url)],
];
const read=async()=>Promise.all(source.map(async([name,url])=>({
  name,content:await readFile(url),
})));

test('version 2.2 retires Script API; one data-only pack UUID is preserved',async()=>{
  const m=JSON.parse((await readFile(source[0][1])).toString('utf8'));
  assert.equal(m.format_version,2);
  assert.equal(m.header.name,'Lazy Tester - Team Commands Help');
  assert.equal(m.header.uuid,'4af9d3f0-9fa1-4cb9-a011-fd37c1b2fb16');
  assert.deepEqual(m.header.version,[2,2,0]);
  assert.deepEqual(m.header.min_engine_version,[1,26,30]);
  assert.deepEqual(m.modules,[{
    type:'data',uuid:'736d52ad-7c03-4a4b-880d-25a89072586f',
    version:[2,2,0],
  }]);
  assert.equal('dependencies' in m,false);
  assert.equal(m.header.description.includes('/function help'),true);
});

test('a single /function help alias routes into one canonical help listing',async()=>{
  const alias=(await readFile(source[1][1])).toString('utf8');
  const commands=alias.split(/\r?\n/).filter(x=>x.trim()&&!x.startsWith('#'));
  assert.deepEqual(commands,['function lazytester/help']);
  assert.equal(alias.includes('tellraw'),false);
});

test('canonical Minecraft function displays team source commands privately',async()=>{
  const code=(await readFile(source[2][1])).toString('utf8');
  const lines=code.split(/\r?\n/).filter(x=>x.trim()&&!x.startsWith('#'));
  assert.ok(lines.length>=5&&lines.length<=18);
  for(const line of lines){
    assert.ok(line.startsWith('tellraw @s '));
    const m=JSON.parse(line.slice('tellraw @s '.length));
    assert.deepEqual(Object.keys(m),['rawtext']);
    assert.equal(m.rawtext.length,1);
    assert.equal(typeof m.rawtext[0].text,'string');
  }
  for(const command of [
    '/scriptevent mivubi:get_analytics',
    '/scriptevent daigon:get_all_scoring',
    '/scriptevent probe:list',
    '/scriptevent probe:run auto',
    '/scriptevent probe:report',
  ])assert.ok(code.includes(command),command);
  assert.equal(code.includes('scriptevent lazytester:help'),false);
  assert.equal(code.includes('LT_EDU_DIAG'),false);
  assert.equal(code.includes('function lt:diag'),false);
  assert.equal(code.includes('tellraw @a'),false);
  assert.ok(code.includes('Settings > Show Coordinates'));
  assert.ok(code.includes('Ctrl+Alt+C'));
  assert.ok(code.includes('Saved Locations ada di aplikasi Lazy Tester'));
});

test('ZIP has exactly manifest and two static function files',async()=>{
  const entries=await read();
  const packed=buildEducationMcpack(entries);
  const end=packed.length-22;
  assert.equal(packed.readUInt32LE(0),0x04034b50);
  assert.equal(packed.readUInt32LE(end),0x06054b50);
  assert.equal(packed.readUInt16LE(end+8),3);
  assert.equal(packed.readUInt16LE(end+10),3);
  assert.equal(packed.readUInt16LE(8),0); // ZIP STORE
  for(const name of entries.map(x=>x.name)) {
    assert.ok(packed.includes(Buffer.from(name,'ascii')));
  }
  assert.equal(packed.includes(Buffer.from('scripts/main.js')),false);
  assert.equal(Buffer.compare(packed,buildEducationMcpack(entries)),0);
});

test('ZIP rejects alternate paths, missing functions and oversized content',()=>{
  const x=Buffer.from('{}'),y=Buffer.from('function lazytester/help\n');
  assert.throws(()=>buildEducationMcpack([
    {name:'manifest.json',content:x},
    {name:'functions/../help.mcfunction',content:y},
    {name:'functions/lazytester/help.mcfunction',content:y},
  ]));
  assert.throws(()=>buildEducationMcpack([{name:'manifest.json',content:x}]));
  assert.throws(()=>buildEducationMcpack([
    {name:'manifest.json',content:x},
    {name:'functions/help.mcfunction',content:y},
    {name:'functions/lazytester/help.mcfunction',content:Buffer.alloc(65537)},
  ]));
});
