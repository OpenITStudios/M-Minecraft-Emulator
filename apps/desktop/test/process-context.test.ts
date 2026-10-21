import {it,expect,vi} from 'vitest';
vi.mock('../src/app/bridge/invokeRuntime.js',()=>({invokeRuntime:vi.fn()}));
import {invokeRuntime} from '../src/app/bridge/invokeRuntime.js';
import {compareProcessContexts,parseContextComparison} from '../src/app/bridge/processContext.js';

const result={sameWindowsUser:true,sameProfileDirectory:true,sameWindowsSession:true};
it('comparison accepts only three booleans, never raw identity or readiness fields',()=>{
  expect(parseContextComparison(result)).toEqual(result);
  for(const invalid of [null,[],{}, {...result,sameWindowsUser:'yes'}, {...result,sid:'private'}, {...result,isolated:true}])expect(()=>parseContextComparison(invalid)).toThrow();
});
it('comparison sends only exact process keys and preserves full-width FILETIME',async()=>{
  vi.mocked(invokeRuntime).mockResolvedValueOnce(result);
  const key={pid:42,createdFileTime:'18446744073709551615',windowHandle:'0x123'};
  expect(await compareProcessContexts(key,key)).toEqual(result);
  expect(invokeRuntime).toHaveBeenLastCalledWith('desktop_compare_process_contexts',{left:{pid:42,createdFileTime:key.createdFileTime},right:{pid:42,createdFileTime:key.createdFileTime}});
});
it('invalid identity is refused before invoking Windows; inspection errors stay unknown',async()=>{
  vi.mocked(invokeRuntime).mockClear();
  const valid={pid:42,createdFileTime:'133400000000000000'};
  for(const key of [{...valid,pid:0},{...valid,pid:1.5},{...valid,pid:2**32},{...valid,createdFileTime:'0'},{...valid,createdFileTime:'1.5'},{...valid,createdFileTime:'18446744073709551616'}])await expect(compareProcessContexts(key,valid)).rejects.toThrow();
  expect(invokeRuntime).not.toHaveBeenCalled();
  vi.mocked(invokeRuntime).mockRejectedValueOnce(new Error('Access denied'));
  await expect(compareProcessContexts(valid,valid)).rejects.toThrow('Access denied');
});
