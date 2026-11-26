// Geometry is a UI observation, never native process/window authority.
import type {ProfileSlot} from './bridge/lazyTester.js';

export interface WorkspaceGeometry {
  readonly viewport: DOMRect;
  readonly tiles: readonly {slot: ProfileSlot; rect: DOMRect}[];
  readonly pixelRatio: number;
}

export function sameWorkspaceGeometry(a: WorkspaceGeometry,b: WorkspaceGeometry): boolean {
  const sameRect=(x: DOMRect,y: DOMRect)=>x.x===y.x && x.y===y.y &&
    x.width===y.width && x.height===y.height;
  return a.pixelRatio===b.pixelRatio && sameRect(a.viewport,b.viewport) &&
    a.tiles.length===b.tiles.length && a.tiles.every((tile,index)=>
      tile.slot===b.tiles[index]!.slot && sameRect(tile.rect,b.tiles[index]!.rect));
}
