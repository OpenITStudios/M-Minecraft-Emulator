import {expect,it} from 'vitest';
import {parseObservedProcess,sameWindowIdentity,windowIdentityKey} from '../src/app/bridge/observedProcess.js';
const process=()=>({pid:77,createdFileTime:'134000000000000001',sessionId:1,windowHandle:'0x123',responding:true});
it('preserves exact process metadata without claiming a window or signed-in account',()=>{
  expect(parseObservedProcess(process())).toEqual(process());
  expect(parseObservedProcess({...process(),windowHandle:'0x0',responding:false}).responding).toBe(false);
  expect(()=>parseObservedProcess({...process(),authenticated:true})).toThrow();
});
it('rejects malformed OS identity bounds and coercible values',()=>{
  for(const patch of [{pid:0},{pid:0x100000000},{pid:'77'},{createdFileTime:'0'},
    {createdFileTime:'18446744073709551616'},{createdFileTime:134000000000000001},
    {sessionId:-1},{sessionId:0x100000000},{windowHandle:'0x123456789abcdef01'},
    {windowHandle:'123'},{responding:1}])expect(()=>parseObservedProcess({...process(),...patch})).toThrow();
});

it('keeps exact window identity separate from responsiveness and absence',()=>{
  const a=parseObservedProcess(process());
  const b=parseObservedProcess({...process(),responding:false});
  expect(sameWindowIdentity(a,b)).toBe(true);
  expect(windowIdentityKey(a)).toBe(windowIdentityKey(b));
  expect(windowIdentityKey(a)).toBe('77/134000000000000001/1/0x123');
  for(const patch of [{pid:78},{createdFileTime:'134000000000000002'},
    {sessionId:2},{windowHandle:'0x124'},{windowHandle:'0x0123'}]){
    const other=parseObservedProcess({...process(),...patch});
    expect(sameWindowIdentity(a,other)).toBe(false);
    expect(windowIdentityKey(a)).not.toBe(windowIdentityKey(other));
  }
  expect(windowIdentityKey(null)).toBe('');
  expect(windowIdentityKey(undefined)).toBe('');
});
it('does not admit a speculative Mac identity through the Windows schema',()=>{
  expect(()=>parseObservedProcess({...process(),platform:'macos'})).toThrow();
  expect(()=>parseObservedProcess({pid:77,platform:'macos',startTime:'1340',windowId:123,responding:true})).toThrow();
});
