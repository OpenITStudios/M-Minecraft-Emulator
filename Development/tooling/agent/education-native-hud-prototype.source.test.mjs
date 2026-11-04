// Authored SOURCE_ONLY guards. NOT_EXECUTED; cannot establish that
// Minecraft Education 26.33 accepts any JSON UI override.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';

const rp=new URL('./education-native-hud-prototype/',import.meta.url);
const load=async(name)=>JSON.parse(await readFile(new URL(name,rp),'utf8'));

test('experimental pack is resources-only and cannot install a script runtime',async()=>{
  const manifest=await load('manifest.json');
  assert.equal(manifest.format_version,2);
  assert.equal(manifest.header.name,'Lazy Tester - Native Coordinates HUD (Experimental)');
  assert.deepEqual(manifest.header.version,[0,1,0]);
  assert.deepEqual(manifest.header.min_engine_version,[1,26,30]);
  assert.equal(manifest.modules.length,1);
  assert.equal(manifest.modules[0].type,'resources');
  assert.equal('dependencies' in manifest,false);
  assert.equal('capabilities' in manifest,false);
  assert.equal('scripts' in manifest,false);
});

test('patch adds one inert label to existing player_position.controls',async()=>{
  const hud=await load('ui/hud_screen.json');
  assert.deepEqual(Object.keys(hud),['namespace','player_position']);
  assert.equal(hud.namespace,'hud');
  assert.deepEqual(Object.keys(hud.player_position),['modifications']);
  const steps=hud.player_position.modifications;
  assert.equal(steps.length,1);
  assert.deepEqual(Object.keys(steps[0]),['array_name','operation','value']);
  assert.equal(steps[0].array_name,'controls');
  assert.equal(steps[0].operation,'insert_back');
  assert.equal(steps[0].value.length,1);
  const controls=steps[0].value[0];
  assert.deepEqual(Object.keys(controls),['lt_native_coordinates_hint']);
  const label=controls.lt_native_coordinates_hint;
  assert.equal(label.type,'label');
  assert.equal(label.text,'LT: Save XYZ in Lazy Tester');
  assert.equal(label.localize,false);
  assert.equal(label.shadow,true);
  assert.equal(label.layer,2);
  assert.deepEqual(label.offset,[0,4]);
  assert.equal('bindings' in label,false);
  assert.equal('button_mappings' in label,false);
  assert.equal('factory' in label,false);
});

test('does not overwrite native position text, visibility or vanilla HUD root',async()=>{
  const text=await readFile(new URL('ui/hud_screen.json',rp),'utf8');
  for(const forbidden of [
    '"root_panel"','"hud_content"','"hud_screen"','"player_position_text"',
    '#player_position_text','#player_position_visible','#hud_visible',
    '#hud_title_text_string','"pressed_button_name"','"script"',
    'runCommand','scriptevent','clipboard','dynamicProperty'
  ]) assert.ok(!text.includes(forbidden),forbidden);
  assert.ok(!text.includes('Save Position'));
  assert.ok(!text.includes('Teleport'));
});
