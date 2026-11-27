import {readFileSync} from 'node:fs';
import {describe, expect, it} from 'vitest';
const source=(name:string)=>readFileSync(new URL('../src/'+name,import.meta.url),'utf8');

describe('Selective Cleanup lifecycle source contract',()=>{
  it('does not offer unverified global cleanup settings as active preferences',()=>{
    const storage=source('DataStorage.svelte');
    expect(storage).toContain('Global Settings');
    expect(storage).toContain('All launchers');
    expect(storage).toContain('aria-label="Auto-clear Download Cache" checked={false} disabled');
    expect(storage).toContain('aria-label="Auto-clear Old Logs" checked={false} disabled');
    expect(storage).toContain('Keep logs for 7 days');
    expect(storage).toContain('disabled>Save Changes</button>');
    expect(storage).toContain('Storage details and maintenance');
  });
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
  it('clears an old review when switching to a cached category',()=>{
    const dialog=source('SelectiveCleanup.svelte');
    const switchStart=dialog.indexOf('async function load(next: SelectiveCategory');
    const switchEnd=dialog.indexOf('function toggle(',switchStart);
    const load=dialog.slice(switchStart,switchEnd);
    expect(load).toMatch(/category=next;\\s*\\/\\/[^\\n]*\\n\\s*receipt=null;\\s*issue='';/);
    expect(load.indexOf('receipt=null;')).toBeLessThan(load.indexOf('if(inventories[next]&&!refresh)return;'));
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
