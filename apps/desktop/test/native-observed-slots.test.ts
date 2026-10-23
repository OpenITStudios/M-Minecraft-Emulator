import {expect,it} from 'vitest';
import {reconcileObservedSlots,observedIds,sameObservedWindow} from '../src/app/presentation/nativeObservedSlots.js';

const process=(pid:number)=>({
  pid,createdFileTime:String(134000000000000000n+BigInt(pid)),
  sessionId:1,windowHandle:'0x'+pid.toString(16),responding:true,
});

it('assigns up to four observed windows without ever reporting isolated accounts',()=>{
  const result=reconcileObservedSlots({},[11,12,13,14].map(process));
  expect(observedIds.map(id=>result[id]?.pid)).toEqual([11,12,13,14]);
  expect(result).not.toHaveProperty('isolationReady');
  expect(result).not.toHaveProperty('account');
});

it('keeps surviving tab identity stable when process order changes or one exits',()=>{
  const initial=reconcileObservedSlots({},[11,12,13].map(process));
  const reordered=reconcileObservedSlots(initial,[13,11,12].map(process));
  expect(observedIds.map(id=>reordered[id]?.pid)).toEqual([11,12,13,undefined]);
  const gone=reconcileObservedSlots(reordered,[13,12].map(process));
  expect(observedIds.map(id=>gone[id]?.pid)).toEqual([undefined,12,13,undefined]);
  const newcomer=reconcileObservedSlots(gone,[13,14,12].map(process));
  expect(observedIds.map(id=>newcomer[id]?.pid)).toEqual([14,12,13,undefined]);
});

it('does not carry window identity through PID reuse, window replacement or creation change',()=>{
  const old=process(11);
  const changedTime={...old,createdFileTime:'134000000000000800'};
  const changedWindow={...old,windowHandle:'0xdec'};
  expect(sameObservedWindow(old,changedTime)).toBe(false);
  expect(sameObservedWindow(old,changedWindow)).toBe(false);
  expect(reconcileObservedSlots({'02':old},[changedTime])['02']).toBe(undefined);
  expect(reconcileObservedSlots({'02':old},[changedWindow])['02']).toBe(undefined);
});

it('rejects ambiguous duplicate PIDs/windows and more than four observed windows',()=>{
  expect(()=>reconcileObservedSlots({},[process(11),process(11)])).toThrow();
  expect(()=>reconcileObservedSlots({},[process(11),{...process(12),windowHandle:process(11).windowHandle}])).toThrow();
  expect(()=>reconcileObservedSlots({},[11,12,13,14,15].map(process))).toThrow();
});
