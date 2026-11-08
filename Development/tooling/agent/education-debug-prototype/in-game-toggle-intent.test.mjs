// AUTHORED_NOT_EXECUTED: synthetic Player / ScriptEvent projections only.
// No Minecraft Add-On, Script API subscription or native key hook runs.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {planPlayerSourcedDebugToggle, planVisiblePlayerDebugFrame} from './in-game-toggle-intent.mjs';

const eventId='example:debug_toggle'; // Test-only; NOT a published team wire ID.
function player(x=-385.25) {
  return {
    typeId:'minecraft:player',
    location:{x,y:64,z:-60.1},
    getRotation(){return {x:0,y:180}},
    dimension:{id:'minecraft:overworld'},
  };
}
function origin(p,extra={}) {
  return {id:eventId,message:'toggle',sourceType:'Entity',sourceEntity:p,...extra};
}

test('same-player command intent produces in-game show then hide, never a desktop UI',()=>{
  const p=player();
  const show=planPlayerSourcedDebugToggle(origin(p),{eventId,currentlyVisible:false});
  assert.equal(show.kind,'show');
  assert.equal(show.player,p,'Only the exact originating Player may own the displayed data');
  assert.equal(show.nextVisible,true);
  assert.ok(show.lines.includes('Chunk: -25 / -4'));
  assert.equal(show.copyCoordinatesText,'-386 64 -61');
  assert.ok(Object.isFrozen(show)&&Object.isFrozen(show.lines));
  const hide=planPlayerSourcedDebugToggle(origin(p),{eventId,currentlyVisible:show.nextVisible});
  assert.deepEqual(hide,{kind:'hide',player:p,nextVisible:false});
  assert.equal(Object.hasOwn(show,'clipboardWritten'),false);
  assert.equal(Object.hasOwn(show,'f3Received'),false);
});

test('each player has an independent proposed state, not a shared MCxx owner',()=>{
  const first=player(),second=player(8);
  const mc01=planPlayerSourcedDebugToggle(origin(first),{eventId,currentlyVisible:false});
  const mc02=planPlayerSourcedDebugToggle(origin(second),{eventId,currentlyVisible:false});
  assert.equal(mc01.player,first); assert.equal(mc02.player,second);
  assert.equal(mc02.copyCoordinatesText,'8 64 -61');
  const close01=planPlayerSourcedDebugToggle(origin(first),{eventId,currentlyVisible:mc01.nextVisible});
  assert.equal(close01.nextVisible,false);
  assert.equal(mc02.nextVisible,true);
});

test('refuses blocks, NPCs, server commands, other entities and unrelated events',()=>{
  const p=player();
  const original=origin(p);
  const forbidden=[
    null,
    {...original,id:'example:unrelated'},
    {...original,message:''},
    {...original,message:'F3'},
    {...original,sourceType:'Server'},
    {...original,sourceType:'Block',sourceBlock:{typeId:'minecraft:stone'}},
    {...original,sourceType:'NPCDialogue',initiator:p},
    {...original,sourceEntity:undefined},
    {...original,sourceEntity:{...p,typeId:'minecraft:zombie'}},
    {...original,sourceEntity:p,sourceBlock:{}},
    {...original,sourceEntity:p,initiator:p},
  ];
  for(const ev of forbidden){
    assert.equal(planPlayerSourcedDebugToggle(ev,{eventId,currentlyVisible:false}),null);
  }
});

test('refuses unverified team event contracts and invalid Player readings',()=>{
  for(const contract of [
    {eventId:'',currentlyVisible:false},
    {eventId:'debug',currentlyVisible:false},
    {eventId:'Example:Uppercase',currentlyVisible:false},
    {eventId,currentlyVisible:'false'},
    {eventId,currentlyVisible:null},
  ])assert.throws(()=>planPlayerSourcedDebugToggle(origin(player()),contract),TypeError);

  const unbound=player();
  unbound.getRotation=()=>{throw new Error('Untrusted native failure');};
  assert.equal(planPlayerSourcedDebugToggle(origin(unbound),{eventId,currentlyVisible:false}),null);
  const invalid=player();
  invalid.location.x=NaN;
  assert.equal(planPlayerSourcedDebugToggle(origin(invalid),{eventId,currentlyVisible:false}),null);
  const invalidDim=player();
  invalidDim.dimension={id:'custom:forged'};
  assert.equal(planPlayerSourcedDebugToggle(origin(invalidDim),{eventId,currentlyVisible:false}),null);
  const retired=player();
  Object.defineProperty(retired,'typeId',{get(){throw new Error('retired entity');}});
  assert.equal(planPlayerSourcedDebugToggle(origin(retired),{eventId,currentlyVisible:false}),null);
  assert.equal(planPlayerSourcedDebugToggle(origin(retired),{eventId,currentlyVisible:true}),null);
});

test('hide is a player-specific UI intent and never calls a stale Player getter',()=>{
  const stale=player();
  Object.defineProperty(stale,'location',{get(){throw new Error('stale');}});
  stale.getRotation=()=>{throw new Error('stale');};
  assert.deepEqual(planPlayerSourcedDebugToggle(origin(stale),{eventId,currentlyVisible:true}),
    {kind:'hide',player:stale,nextVisible:false});
});

test('visible in-game debug data changes with position without a second toggle',()=>{
  const p=player();
  const opened=planPlayerSourcedDebugToggle(origin(p),{eventId,currentlyVisible:false});
  assert.equal(opened.kind,'show');
  const idle=planVisiblePlayerDebugFrame(p,{
    visible:opened.nextVisible,
    previousLines:opened.lines,
    previousCoordinatesText:opened.copyCoordinatesText,
  });
  assert.deepEqual(idle,{kind:'unchanged',player:p,nextVisible:true},
    'No needless render request while player data is unchanged');
  p.location.x = -384.2;
  const moved=planVisiblePlayerDebugFrame(p,{
    visible:true,previousLines:opened.lines,
    previousCoordinatesText:opened.copyCoordinatesText,
  });
  assert.equal(moved.kind,'update');
  assert.equal(moved.player,p);
  assert.equal(moved.copyCoordinatesText,'-385 64 -61');
  assert.ok(moved.lines.includes('XYZ: -384.20 / 64.00 / -60.10'));
  assert.equal(Object.isFrozen(moved.lines),true);
  assert.equal(Object.hasOwn(moved,'clipboardWritten'),false);
});

test('facing and dimension refresh even if block coordinates do not change',()=>{
  const p=player();
  const opened=planPlayerSourcedDebugToggle(origin(p),{eventId,currentlyVisible:false});
  p.getRotation=()=>({x:-20,y:90});
  p.dimension={id:'minecraft:nether'};
  const update=planVisiblePlayerDebugFrame(p,{
    visible:true, previousLines:opened.lines,
    previousCoordinatesText:opened.copyCoordinatesText,
  });
  assert.equal(update.kind,'update');
  assert.equal(update.lines[5],'Facing: West (-X)');
  assert.equal(update.lines[6],'Rotation: yaw 90.00 / pitch -20.00');
  assert.equal(update.lines[7],'Dimension: Nether');
  assert.equal(update.copyCoordinatesText,opened.copyCoordinatesText);
});

test('hidden panels never read player data or issue a UI update',()=>{
  const p={typeId:'minecraft:player'};
  Object.defineProperty(p,'location',{get(){throw new Error('must not be read');}});
  p.getRotation=()=>{throw new Error('must not be called');};
  assert.equal(planVisiblePlayerDebugFrame(p,{visible:false}),null);
  assert.throws(()=>planVisiblePlayerDebugFrame(p,{visible:'false'}),TypeError);
  assert.equal(planVisiblePlayerDebugFrame(null,{visible:false}),null);
  assert.equal(planVisiblePlayerDebugFrame({typeId:'minecraft:zombie'},{visible:true}),null);
});

test('invalid player refresh clears an old panel rather than retaining stale coordinates',()=>{
  const p=player();
  p.location.x=Number.NaN;
  const state=planVisiblePlayerDebugFrame(p,{visible:true});
  assert.deepEqual(state,{kind:'hide',player:p,nextVisible:false});
  const retired=player();
  Object.defineProperty(retired,'typeId',{get(){throw new Error('entity no longer valid');}});
  assert.deepEqual(planVisiblePlayerDebugFrame(retired,{visible:true}),
    {kind:'hide',player:retired,nextVisible:false});
});

test('five player refresh intents remain independent without MCxx name or PID routing',()=>{
  const players=[-385,7,31,90,-12].map(x=>player(x));
  const states=players.map(p=>planVisiblePlayerDebugFrame(p,{visible:true}));
  assert.deepEqual(states.map(x=>x.player),players);
  assert.deepEqual(states.map(x=>x.copyCoordinatesText),[
    '-385 64 -61','7 64 -61','31 64 -61','90 64 -61','-12 64 -61',
  ]);
  players[1].location.x=8.2;
  const updated=planVisiblePlayerDebugFrame(players[1],{
    visible:true,
    previousLines:states[1].lines,
    previousCoordinatesText:states[1].copyCoordinatesText,
  });
  assert.equal(updated.kind,'update');
  assert.equal(updated.copyCoordinatesText,'8 64 -61');
  assert.equal(states[0].copyCoordinatesText,'-385 64 -61');
});
