import {readFileSync} from 'node:fs';
import {describe, expect, it} from 'vitest';
const source=(name:string)=>readFileSync(new URL('../src/'+name,import.meta.url),'utf8');

describe('Selective Cleanup lifecycle source contract',()=>{
  it('invalidates outstanding receipts before closing and on destruction',()=>{
    const dialog=source('SelectiveCleanup.svelte');
    expect(dialog).toContain('function close(): void');
    expect(dialog).toContain('closed=true;');
    expect(dialog).toContain('generation++;');
    expect(dialog).toContain('if(request!==generation)return;');
    expect(dialog).toContain('if(request===generation&&reviewHeading?.isConnected)');
    expect(dialog).toContain('on:cancel|preventDefault={close}');
    expect(dialog).not.toContain('clearData(');
  });
  it('shares navigation guard and restores focus to a persistent heading if trigger is unavailable',()=>{
    const storage=source('DataStorage.svelte');
    const app=source('LazyTesterApp.svelte');
    expect(storage).toContain('selectedStorage.fullClear?.pending === true');
    expect(storage).toContain('else if(storageOverviewHeading?.isConnected)');
    expect(storage).toContain('selectiveOpen = selectiveSlot !== null');
    expect(app).toContain('navigationReviewOpen = selectiveOpen ||');
    expect(app).toContain('bind:selectiveOpen');
  });
});
