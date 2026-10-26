import {invokeRuntime} from './invokeRuntime.js';
import {parseObservedProcess, type OwnedProcess} from './ownedProviderApi.js';
import {previewViewport, workspaceViewport, type WorkspaceRectangle, type WorkspaceViewport} from './workspaceSurface.js';

export const profileSlots = [1, 2, 3, 4, 5] as const;
export type ProfileSlot = typeof profileSlots[number];
export interface ProfileSession {
  slot: ProfileSlot;
  state: 'STOPPED' | 'STARTING' | 'RUNNING' | 'NOT_RESPONDING' | 'ATTENTION';
  process: OwnedProcess | null;
  issue: string | null;
}
export interface ProfileStorage {slot: ProfileSlot; bytes: number; files: number; partial: boolean}
export interface SessionSnapshot {schema: 1; profiles: ProfileSession[]; storage: ProfileStorage[] | null}
export interface PreviewPlacement {slot: ProfileSlot; viewport: WorkspaceViewport}
export interface PreviewStatus {slot: ProfileSlot; registered: boolean; reason: string | null}
export interface PreviewReport {schema: 1; tiles: PreviewStatus[]}
const record = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value);
const exact = (value: Record<string, unknown>, names: string[]) =>
  Object.keys(value).length === names.length && names.every(name => Object.hasOwn(value, name));

export function parseSession(value: unknown): SessionSnapshot {
  if (!record(value) || !exact(value, ['schema', 'profiles', 'storage']) || value.schema !== 1 ||
      !Array.isArray(value.profiles) || value.profiles.length !== 5) throw new TypeError('Invalid session response');
  const pids = new Set<number>();
  value.profiles.forEach((profile: unknown, index: number) => {
    if (!record(profile) || !exact(profile, ['slot', 'state', 'process', 'issue']) ||
        profile.slot !== index + 1 || !['STOPPED', 'STARTING', 'RUNNING', 'NOT_RESPONDING', 'ATTENTION'].includes(String(profile.state)) ||
        !(profile.issue === null || typeof profile.issue === 'string' && profile.issue.length <= 8192)) throw new TypeError('Invalid profile response');
    if (profile.process !== null) {
      const process = parseObservedProcess(profile.process);
      if (pids.has(process.pid) || !['STARTING', 'RUNNING', 'NOT_RESPONDING'].includes(String(profile.state))) throw new TypeError('Inconsistent profile identity');
      pids.add(process.pid);
      if (profile.state === 'RUNNING' && (!process.responding || BigInt(process.windowHandle) === 0n)) throw new TypeError('Invalid running game');
    } else if (profile.state === 'RUNNING') throw new TypeError('Running game has no process');
  });
  if (value.storage !== null) {
    if (!Array.isArray(value.storage) || value.storage.length !== 5) throw new TypeError('Invalid storage response');
    value.storage.forEach((storage: unknown, index: number) => {
      if (!record(storage) || !exact(storage, ['slot', 'bytes', 'files', 'partial']) || storage.slot !== index + 1 ||
          !Number.isSafeInteger(storage.bytes) || Number(storage.bytes) < 0 || !Number.isInteger(storage.files) ||
          Number(storage.files) < 0 || Number(storage.files) > 100000 || typeof storage.partial !== 'boolean') throw new TypeError('Invalid storage metadata');
    });
  }
  return value as unknown as SessionSnapshot;
}

export function nextProfile(profiles: ProfileSession[], active: ProfileSlot | null, direction: 1 | -1): ProfileSlot | null {
  const playable = profiles.filter(profile => profile.state === 'RUNNING' && profile.process?.responding);
  if (!playable.length) return null;
  const index = playable.findIndex(profile => profile.slot === active);
  return playable[index < 0 ? (direction === 1 ? 0 : playable.length - 1) :
    (index + direction + playable.length) % playable.length]!.slot;
}

export function parsePreviewReport(value:unknown,slots:readonly ProfileSlot[]):PreviewReport {
  if(!record(value)||!exact(value,['schema','tiles'])||value.schema!==1||!Array.isArray(value.tiles)||value.tiles.length!==slots.length||slots.length>4)throw new TypeError('Invalid preview response');
  const seen=new Set<number>();
  for(const tile of value.tiles){
    if(!record(tile)||!exact(tile,['slot','registered','reason'])||!slots.includes(tile.slot as ProfileSlot)||seen.has(Number(tile.slot))||
      typeof tile.registered!=='boolean'||!(tile.reason===null||typeof tile.reason==='string'&&tile.reason.length<=8192))throw new TypeError('Invalid preview status');
    if(tile.registered&&tile.reason!==null)throw new TypeError('Registered preview has a conflicting failure');
    seen.add(Number(tile.slot));
  }
  return value as unknown as PreviewReport;
}

export const lazyTester = {
  inspect: async () => parseSession(await invokeRuntime<unknown>('lazy_tester_session', {request: {operation: 'Inspect'}})),
  launch: async (slot: ProfileSlot) => parseSession(await invokeRuntime<unknown>('lazy_tester_session', {request: {operation: 'Launch', slot}})),
  storage: async () => parseSession(await invokeRuntime<unknown>('lazy_tester_session', {request: {operation: 'InspectStorage'}})),
  openFolder: async (slot: ProfileSlot) => parseSession(await invokeRuntime<unknown>('lazy_tester_session', {request: {operation: 'OpenProfileFolder', slot}})),
  present: (slot: ProfileSlot, rect: WorkspaceRectangle) => invokeRuntime<void>('lazy_tester_present', {
    slot, viewport: workspaceViewport(rect, window.devicePixelRatio),
  }),
  previews: async(rect:WorkspaceRectangle,tiles:{slot:ProfileSlot;rect:WorkspaceRectangle}[])=>{
    const placements:PreviewPlacement[]=tiles.map(tile=>({slot:tile.slot,viewport:previewViewport(tile.rect,window.devicePixelRatio)}));
    return parsePreviewReport(await invokeRuntime<unknown>('lazy_tester_previews',{
      viewport:workspaceViewport(rect,window.devicePixelRatio),tiles:placements,
    }),tiles.map(tile=>tile.slot));
  },
  clearPreviews:()=>invokeRuntime<void>('lazy_tester_clear_previews'),
};
