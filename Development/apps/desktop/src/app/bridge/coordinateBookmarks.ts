// Typed, bounded notebook bridge. User-supplied XYZ is never attributed to a
// Minecraft player/world or an authenticated MCxx actor by this component.
import {invokeRuntime} from './invokeRuntime.js';
import {profileSlots, type ProfileSlot} from './lazyTester.js';
import {formatUserPastedCoordinates} from '../coordinates.js';

export type BookmarkDimension = 'Unknown' | 'Overworld' | 'Nether' | 'TheEnd';

export interface CoordinateBookmark {
  readonly id: string;
  readonly slot: ProfileSlot;
  readonly name: string;
  readonly worldLabel: string;
  readonly dimension: BookmarkDimension;
  readonly xyz: string;
  readonly origin: 'USER_PASTED_UNVERIFIED';
}

export interface CoordinateBookmarkReport {
  readonly schema: 1;
  readonly items: readonly CoordinateBookmark[];
}

const dimensions: readonly BookmarkDimension[] = ['Unknown','Overworld','Nether','TheEnd'];
const isRecord = (value: unknown): value is Record<string,unknown> =>
  value !== null && typeof value === 'object' && !Array.isArray(value);
const exact = (value: Record<string,unknown>, keys: readonly string[]): boolean =>
  Object.keys(value).length === keys.length && keys.every(key=>Object.hasOwn(value,key));

function checkedLabel(value: string): string {
  if (typeof value !== 'string' || !value || value !== value.trim() ||
      new TextEncoder().encode(value).length > 64 || /[\u0000-\u001f\u007f-\u009f]/u.test(value)) {
    throw new TypeError('Provide a name/world label of up to 64 UTF-8 bytes');
  }
  return value;
}

export function parseCoordinateBookmarks(value: unknown): CoordinateBookmarkReport {
  if (!isRecord(value) || !exact(value,['schema','items']) ||
      value.schema !== 1 || !Array.isArray(value.items) || value.items.length > 100) {
    throw new TypeError('Invalid coordinate bookmark response');
  }
  const ids = new Set<string>();
  const items: CoordinateBookmark[] = [];
  for (const item of value.items) {
    if (!isRecord(item) || !exact(item,['id','slot','name','worldLabel','dimension','xyz','origin']) ||
        typeof item.id !== 'string' || !/^[a-f0-9]{32}$/.test(item.id) || ids.has(item.id) ||
        !profileSlots.includes(item.slot as ProfileSlot) ||
        typeof item.name !== 'string' || typeof item.worldLabel !== 'string' ||
        typeof item.xyz !== 'string' || !dimensions.includes(item.dimension as BookmarkDimension) ||
        item.origin !== 'USER_PASTED_UNVERIFIED') {
      throw new TypeError('Invalid saved coordinate bookmark');
    }
    checkedLabel(item.name);
    checkedLabel(item.worldLabel);
    if (formatUserPastedCoordinates(item.xyz,'XYZ') !== item.xyz) {
      throw new TypeError('Coordinate bookmark is not canonical XYZ');
    }
    ids.add(item.id);
    items.push({
      id:item.id,slot:item.slot as ProfileSlot,name:item.name,
      worldLabel:item.worldLabel,dimension:item.dimension as BookmarkDimension,
      xyz:item.xyz,origin:'USER_PASTED_UNVERIFIED',
    });
  }
  return {schema:1,items};
}

const request = async(action: Record<string,unknown>):Promise<CoordinateBookmarkReport> =>
  parseCoordinateBookmarks(await invokeRuntime<unknown>('lazy_tester_coordinate_bookmarks',{action}));

export const coordinateBookmarks = {
  list:()=>request({operation:'List'}),
  save:(input:{
    slot: ProfileSlot | null;
    name:string;
    worldLabel:string;
    dimension:BookmarkDimension;
    raw:string;
  })=>{
    if (input.slot===null || !profileSlots.includes(input.slot)) {
      throw new TypeError('Select a Minecraft launcher before saving a location');
    }
    if (!dimensions.includes(input.dimension)) {
      throw new TypeError('Choose a known dimension label');
    }
    const name=checkedLabel(input.name);
    const worldLabel=checkedLabel(input.worldLabel);
    const xyz=formatUserPastedCoordinates(input.raw,'XYZ');
    return request({operation:'Save',slot:input.slot,name,worldLabel,dimension:input.dimension,xyz});
  },
  delete:(id:string)=>{
    if (!/^[a-f0-9]{32}$/.test(id))throw new TypeError('Invalid bookmark identifier');
    return request({operation:'Delete',id});
  },
};
