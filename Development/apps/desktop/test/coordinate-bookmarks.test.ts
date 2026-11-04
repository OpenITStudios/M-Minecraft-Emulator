import {describe, expect, it} from 'vitest';
import {parseCoordinateBookmarks} from '../src/app/bridge/coordinateBookmarks.js';

const valid={
  schema:1,
  items:[{
    id:'0123456789abcdef0123456789abcdef',
    slot:2,
    name:'Spawn QA',
    worldLabel:'World A',
    dimension:'Unknown',
    xyz:'-45.125 64 123.456',
    origin:'USER_PASTED_UNVERIFIED',
  }],
};

describe('native-first manual coordinate notebook response',()=>{
  it('accepts only explicit unverified notes with a bounded MCxx label',()=>{
    const parsed=parseCoordinateBookmarks(valid);
    expect(parsed.schema).toBe(1);
    expect(parsed.items[0].slot).toBe(2);
    expect(parsed.items[0].worldLabel).toBe('World A');
    expect(parsed.items[0].xyz).toBe('-45.125 64 123.456');
  });
  it('rejects forged game-origin and malformed or duplicate notes',()=>{
    const item=valid.items[0];
    for(const bad of [
      {...item,origin:'VERIFIED_MINECRAFT'},
      {...item,slot:6},
      {...item,dimension:'minecraft:overworld'},
      {...item,xyz:'1e3 2 3'},
      {...item,xyz:'1 2 3; /tp @s 10 20 30'},
      {...item,name:'\r\n'},
      {...item,worldLabel:''},
      {...item,playerName:'unknown'},
    ]) {
      expect(()=>parseCoordinateBookmarks({schema:1,items:[bad]})).toThrow();
    }
    expect(()=>parseCoordinateBookmarks({schema:1,items:[item,item]})).toThrow();
    expect(()=>parseCoordinateBookmarks({schema:2,items:[item]})).toThrow();
  });
  it('rejects an incomplete or oversized response without inventing defaults',()=>{
    expect(()=>parseCoordinateBookmarks({schema:1})).toThrow();
    expect(()=>parseCoordinateBookmarks({...valid,items:Array(101).fill(valid.items[0])})).toThrow();
    expect(()=>parseCoordinateBookmarks(null)).toThrow();
  });
});
