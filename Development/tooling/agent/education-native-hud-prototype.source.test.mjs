// SOURCE_ONLY regression contracts. AUTHORED_NOT_EXECUTED by ChatGPT/GitHub.
// These checks cannot prove Education 26.33 loads or renders a JSON UI patch.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile,mkdtemp,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {validateHudSource,validatePinnedVanillaSample,inspectVanillaHudStructure,parseStrictHudJson,ordinaryFile,sameSourceFile} from './check-education-native-hud-source.mjs';

const rp = new URL('./education-native-hud-prototype/', import.meta.url);
const read = async name => JSON.parse(await readFile(new URL(name, rp), 'utf8'));

test('prototype remains resource-only, with no new Script API or dependencies', async () => {
  const m = await read('manifest.json');
  assert.deepEqual(Object.keys(m), ['format_version','header','modules','metadata']);
  assert.equal(m.format_version, 2);
  assert.equal(m.header.name, 'Lazy Tester - Native Coordinates HUD (Experimental)');
  assert.deepEqual(m.header.version, [0,3,0]);
  assert.equal(m.header.pack_scope,'world');
  assert.equal(m.header.description,'Opt-in Education JSON UI layout candidate; adds one visibility-bound sibling after vanilla XYZ. No gameplay, Save, or telemetry.');
  assert.deepEqual(m.header.min_engine_version, [1,26,30]);
  assert.equal(m.modules.length, 1);
  assert.deepEqual(m.modules[0].version, [0,3,0]);
  assert.equal(m.modules[0].type, 'resources');
  for (const key of ['dependencies','capabilities','scripts']) {
    assert.equal(Object.hasOwn(m,key),false);
  }
});

test('hint is one standalone sibling AFTER vanilla player_position, never inside it', async () => {
  const hud = await read('ui/hud_screen.json');
  assert.deepEqual(Object.keys(hud), ['namespace','lt_native_coordinates_hint','root_panel/chat_stack']);
  assert.equal(hud.namespace, 'hud');
  assert.equal(Object.hasOwn(hud,'player_position'),false);
  assert.equal(Object.hasOwn(hud,'root_panel'),false);
  const mod = hud['root_panel/chat_stack']?.modifications;
  assert.equal(mod?.length,1);
  assert.deepEqual(Object.keys(mod[0]), ['control_name','operation','value']);
  assert.equal(mod[0].control_name,'player_position');
  assert.equal(mod[0].operation,'insert_after');
  assert.deepEqual(mod[0].value, [
    {'lt_native_coordinates_hint@hud.lt_native_coordinates_hint':{}},
  ]);
});

test('hint is compact and follows vanilla coordinate visibility', async () => {
  const hud = await read('ui/hud_screen.json');
  const hint = hud.lt_native_coordinates_hint;
  assert.deepEqual(Object.keys(hint), [
    'type','text','localize','font_size','shadow','layer','color','offset','bindings',
  ]);
  assert.equal(hint.type,'label');
  assert.equal(hint.text,'LT: XYZ notes in app');
  assert.ok(hint.text.length <= 24);
  assert.equal(hint.localize,false);
  assert.equal(hint.font_size,'small');
  assert.equal(hint.shadow,true);
  assert.equal(hint.layer,1);
  assert.deepEqual(hint.offset,[2,0]);
  assert.deepEqual(hint.bindings,[{
    binding_name:'#player_position_visible',
    binding_name_override:'#visible',
    binding_type:'global',
  }]);
  // Avoid a content-relative percent dimension that could create circular
  // size/overlap in vanilla player_position's 100%c auto-sizing image.
  assert.equal(Object.hasOwn(hint,'size'),false);
  for(const key of ['button_mappings','pressed_button_name','factory','on_click','script']) {
    assert.equal(Object.hasOwn(hint,key),false);
  }
});

test('one HUD file never replaces coordinates, their binding, or global game controls', async () => {
  const raw = await readFile(new URL('ui/hud_screen.json',rp),'utf8');
  for (const forbidden of [
    '"player_position":','"root_panel":','"hud_screen":','"hud_content":',
    '"#player_position_text"','"#hud_visible"','"#hud_title_text_string"',
    '"#text"','"button_mappings"','"pressed_button_name"',
    '"runCommand"','"scriptevent"','"clipboard"','"dynamicProperty"',
    '"world.getPlayers"','"teleport"','"Save Position"'
  ]) {
    assert.equal(raw.includes(forbidden),false,forbidden);
  }
});


test('read-only preflight rejects global scope, future scripts and fake buttons', async () => {
  const originalManifest=await read('manifest.json');
  const originalHud=await read('ui/hud_screen.json');
  assert.equal(validateHudSource(originalManifest,originalHud).packScope,'world');

  const badManifest=structuredClone(originalManifest);
  delete badManifest.header.pack_scope; // absent defaults to "any": unsafe globally
  assert.throws(()=>validateHudSource(badManifest,originalHud),/HUD_SOURCE_REFUSED/);
  badManifest.header.pack_scope='global';
  assert.throws(()=>validateHudSource(badManifest,originalHud),/HUD_SOURCE_REFUSED/);
  badManifest.header.pack_scope='world';
  badManifest.modules.push({type:'script',uuid:badManifest.modules[0].uuid,version:[0,3,0]});
  assert.throws(()=>validateHudSource(badManifest,originalHud),/HUD_SOURCE_REFUSED/);

  const badHud=structuredClone(originalHud);
  badHud.lt_native_coordinates_hint.type='button';
  assert.throws(()=>validateHudSource(originalManifest,badHud),/HUD_SOURCE_REFUSED/);
  badHud.lt_native_coordinates_hint.type='label';
  badHud.player_position={controls:[]};
  assert.throws(()=>validateHudSource(originalManifest,badHud),/HUD_SOURCE_REFUSED/);
});

test('official Mojang witness is pinned by raw Git blob hash, never accepted by filename',()=>{
  assert.throws(()=>validatePinnedVanillaSample(Buffer.from('fake json')),/HUD_SOURCE_REFUSED/);
  assert.throws(()=>validatePinnedVanillaSample(Buffer.alloc(2048)),/HUD_SOURCE_REFUSED/);
});


const syntheticBedrockJsonc=String.raw`{
  // This deliberately contains comments, a trailing comma and a literal
  // string with a URL. A simplistic regex that deletes // in strings fails.
  "test_url": "https://minecraft.net/vanilla",
  "root_panel": {
    "controls": [
      {
        "chat_stack": {
          "type": "stack_panel",
          "orientation": "vertical",
          "controls": [
            {"paper_doll_padding": {}},
            {"player_position@hud.player_position": {}},
            {"number_of_days_played@hud.number_of_days_played": {}},
          ],
        }
      },
    ]
  },
  "player_position": {
    "type": "image",
    "size": ["100%c + 6px", "100%c + 2px"],
    "controls": [
      {"player_position_text": {
        "type": "label",
        "bindings": [
          {"binding_name":"#player_position_text",
           "binding_name_override":"#text",
           "binding_type":"global"},
        ]
      }}
    ],
    "bindings": [
      {"binding_name":"#player_position_visible",
       "binding_name_override":"#visible",
       "binding_type":"global"},
    ]
  }
}`;

test('JSONC structure diagnostic locates the actual vanilla sibling and preserves bindings',()=>{
  const report=inspectVanillaHudStructure(syntheticBedrockJsonc);
  assert.equal(report.sampleStatus,'STRUCTURAL_REVIEW_UNTRUSTED_UNTIL_SHA_CHECK');
  assert.equal(report.chatStackChildren,3);
  assert.equal(report.nativePositionIndex,1);
  assert.equal(report.nextControl,'number_of_days_played@hud.number_of_days_played');
  assert.equal(report.nativeXyzBinding,'#player_position_text');
  assert.equal(report.nativeVisibilityBinding,'#player_position_visible');
  assert.equal(report.educationCompatibility,'NOT_VERIFIED');
});

test('JSONC structure refuses missing/duplicated/nonadjacent coordinate instance',()=>{
  const replace=(before,after)=>syntheticBedrockJsonc.replace(before,after);
  for(const source of [
    replace('"chat_stack"', '"renamed_chat_stack"'),
    replace('"player_position@hud.player_position"', '"other_position@hud.player_position"'),
    replace('"number_of_days_played@hud.number_of_days_played"', '"renamed_days"'),
    replace('{"player_position@hud.player_position": {}},',
      '{"player_position@hud.player_position": {}}, {"player_position@hud.player_position": {}},'),
    replace('{"player_position@hud.player_position": {}},',
      '{"player_position@hud.player_position": {}}, {"intermediate_hint": {}},'),
    replace('"#player_position_visible"', '"#some_other_visible"'),
    replace('"#player_position_text"', '"#some_other_text"'),
    replace('"orientation": "vertical"', '"orientation": "horizontal"'),
  ]) {
    assert.throws(()=>inspectVanillaHudStructure(source),/HUD_SOURCE_REFUSED/);
  }
});


test('preflight rejects duplicated property names even when JSON.parse keeps the safe last value',async()=>{
  const manifest=await readFile(new URL('manifest.json',rp),'utf8');
  const original=JSON.parse(manifest);
  const duplicated=manifest.replace('"header": {',
    '"header": {"pack_scope": "any"},\n  "header": {');
  assert.deepEqual(JSON.parse(duplicated),original);
  assert.throws(()=>parseStrictHudJson(duplicated),/HUD_SOURCE_REFUSED: duplicate JSON object key/);
  const hud=await readFile(new URL('ui/hud_screen.json',rp),'utf8');
  const repeated=hud.replace('"namespace": "hud",',
    '"namespace": "fake",\n  "namespace": "hud",');
  assert.deepEqual(JSON.parse(repeated),JSON.parse(hud));
  assert.throws(()=>parseStrictHudJson(repeated),/duplicate JSON object key/);
  assert.throws(()=>parseStrictHudJson(String.raw`{"name":1,"\u006eame":2}`),/duplicate JSON object key/);
  const changedDescription=structuredClone(original);
  changedDescription.header.description='Verified Minecraft Education production HUD';
  assert.throws(()=>validateHudSource(changedDescription,JSON.parse(hud)),/HUD_SOURCE_REFUSED/);
});

test('JSONC structure diagnostic refuses duplicate structural keys after comments are stripped',()=>{
  const duplicate=syntheticBedrockJsonc.replace(
    '"orientation": "vertical",',
    '"orientation": "horizontal",\n          "orientation": "vertical",');
  assert.throws(()=>inspectVanillaHudStructure(duplicate),/duplicate JSON object key/);
});

test('Windows path/fstat device ids can differ without accepting replacement identities',()=>{
  const stat=dev=>({
    isFile:()=>true,isSymbolicLink:()=>false,
    nlink:1n,dev:BigInt(dev),ino:42n,size:15n,mtimeNs:100n,ctimeNs:101n
  });
  const named=stat(0),opened=stat(34123);
  assert.equal(sameSourceFile(named,opened,{openedHandle:true,platform:'win32'}),true);
  assert.equal(sameSourceFile(named,opened,{openedHandle:false,platform:'win32'}),false);
  assert.equal(sameSourceFile(named,opened,{openedHandle:true,platform:'linux'}),false);
  assert.equal(sameSourceFile(named,{...opened,ino:43n},{openedHandle:true,platform:'win32'}),false);
  assert.equal(sameSourceFile(named,{...opened,mtimeNs:101n},{openedHandle:true,platform:'win32'}),false);
  assert.equal(sameSourceFile(named,{...opened,nlink:2n},{openedHandle:true,platform:'win32'}),false);
});

test('read-only preflight uses a bounded ordinary file handle',async()=>{
  const directory=await mkdtemp(join(tmpdir(),'lazy-hud-source-'));
  const sample=join(directory,'fixed.json');
  try{
    const input=Buffer.from('{"scope":"local"}\n');
    await writeFile(sample,input);
    assert.deepEqual(await ordinaryFile(sample,1024),input);
    await assert.rejects(ordinaryFile(sample,4),/HUD_SOURCE_REFUSED: invalid or unsafe source file/);
  }finally{await rm(directory,{recursive:true,force:true});}
});

test('synthetic JSONC structure never substitutes for the official Mojang blob SHA',()=>{
  assert.throws(
    ()=>validatePinnedVanillaSample(Buffer.from(syntheticBedrockJsonc,'utf8')),
    /HUD_SOURCE_REFUSED.*reference is not the exact pinned/,
  );
});
