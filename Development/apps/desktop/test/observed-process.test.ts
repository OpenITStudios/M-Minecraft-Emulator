import {expect,it} from 'vitest';
import {parseObservedProcess} from '../src/app/bridge/observedProcess.js';
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

