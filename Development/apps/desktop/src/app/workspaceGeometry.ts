// Geometry is a UI observation, never native process/window authority.
import type {ProfileSlot} from './bridge/lazyTester.js';

export interface WorkspaceGeometry {
  readonly viewport: DOMRect;
  readonly tiles: readonly {slot: ProfileSlot; rect: DOMRect}[];
  readonly pixelRatio: number;
}

// One event-driven observer, no timer. Re-arm only when the old resolution
// query changes, and release the exact listener on teardown.
export function observeDisplayScale(host: Pick<Window,'matchMedia'|'devicePixelRatio'>, onChange:()=>void):()=>void {
  let query:MediaQueryList|undefined;
  let disposed=false;
  function changed():void {
    if(disposed)return;
    arm();
    onChange();
  }
  function arm():void {
    query?.removeEventListener('change',changed);
    const scale=host.devicePixelRatio;
    query=Number.isFinite(scale)&&scale>0&&typeof host.matchMedia==='function'?host.matchMedia(`(resolution: ${scale}dppx)`):undefined;
    if(query&&typeof query.addEventListener!=='function')query=undefined;
    query?.addEventListener('change',changed);
  }
  arm();
  return ()=>{disposed=true;query?.removeEventListener('change',changed);query=undefined;};
}

export function sameWorkspaceGeometry(a: WorkspaceGeometry,b: WorkspaceGeometry): boolean {
  const sameRect=(x: DOMRect,y: DOMRect)=>x.x===y.x && x.y===y.y &&
    x.width===y.width && x.height===y.height;
  return a.pixelRatio===b.pixelRatio && sameRect(a.viewport,b.viewport) &&
    a.tiles.length===b.tiles.length && a.tiles.every((tile,index)=>
      tile.slot===b.tiles[index]!.slot && sameRect(tile.rect,b.tiles[index]!.rect));
}
