// SOURCE_ONLY regression contracts. AUTHORED_NOT_EXECUTED by ChatGPT/GitHub.
// These checks cannot prove Education 26.33 loads or renders a JSON UI patch.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {validateHudSource,validatePinnedVanillaSample} from './check-education-native-hud-source.mjs';

const rp = new URL('./education-native-hud-prototype/', import.meta.url);
const read = async name => JSON.parse(await readFile(new URL(name, rp), 'utf8'));

test('prototype remains resource-only, with no new Script API or dependencies', async () => {
  const m = await read('manifest.json');
  assert.deepEqual(Object.keys(m), ['format_version','header','modules','metadata']);
  assert.equal(m.format_version, 2);
  assert.equal(m.header.name, 'Lazy Tester - Native Coordinates HUD (Experimental)');
  assert.deepEqual(m.header.version, [0,3,0]);
  assert.equal(m.header.pack_scope,'world');
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
