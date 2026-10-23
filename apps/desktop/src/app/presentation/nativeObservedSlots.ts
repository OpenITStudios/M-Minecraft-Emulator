import type {ClientId, OwnedProcess} from '../bridge/ownedProviderApi.js';

// Presentation-only assignments. These are transient window references,
// never virtual profiles, independently authenticated accounts or launch rights.
export type ObservedSlotMap = Partial<Record<ClientId, OwnedProcess>>;
export const observedIds:readonly ClientId[] = ['01','02','03','04'];

export function sameObservedWindow(left:OwnedProcess,right:OwnedProcess):boolean {
  return left.pid===right.pid &&
    left.createdFileTime===right.createdFileTime &&
    left.sessionId===right.sessionId &&
    left.windowHandle.toLowerCase()===right.windowHandle.toLowerCase();
}

export function reconcileObservedSlots(previous:ObservedSlotMap, processes:readonly OwnedProcess[]):ObservedSlotMap {
  if(processes.length>observedIds.length)throw new TypeError('Too many observed game windows.');
  const pids=new Set<number>(),windows=new Set<string>();
  for(const process of processes) {
    const window=process.windowHandle.toLowerCase();
    if(pids.has(process.pid)||(window!=='0x0'&&windows.has(window)))
      throw new TypeError('Duplicated observed Windows game identity.');
    pids.add(process.pid);
    if(window!=='0x0')windows.add(window);
  }
  const result:ObservedSlotMap={},assigned=new Set<number>();
  // Preserve existing tab identity as long as its exact process/window lives.
  for(const id of observedIds) {
    const old=previous[id];
    const index=old?processes.findIndex(p=>sameObservedWindow(old,p)):-1;
    if(index>=0&&!assigned.has(index)){
      result[id]=processes[index];
      assigned.add(index);
    }
  }
  // New windows use only empty slots; never shift surviving windows.
  for(let index=0;index<processes.length;index++) {
    if(assigned.has(index))continue;
    const free=observedIds.find(id=>!result[id]);
    if(!free)throw new TypeError('No free observed window slot.');
    result[free]=processes[index];
  }
  return result;
}
