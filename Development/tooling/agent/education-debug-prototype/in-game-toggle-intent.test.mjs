// AUTHORED_NOT_EXECUTED: synthetic Player / ScriptEvent projections only.
// No Minecraft Add-On, Script API subscription or native key hook runs.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {planPlayerSourcedDebugToggle} from './in-game-toggle-intent.mjs';

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
