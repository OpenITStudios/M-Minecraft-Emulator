import {invokeRuntime} from './invokeRuntime.js';
import {parseObservedProcess, type OwnedProcess} from './observedProcess.js';
import {previewViewport, workspaceViewport, type WorkspaceRectangle, type WorkspaceViewport} from './workspaceSurface.js';

export const profileSlots = [1, 2, 3, 4, 5] as const;
export type ProfileSlot = typeof profileSlots[number];
export interface ProfileSession {
  slot: ProfileSlot;
  state: 'STOPPED' | 'STARTING' | 'RUNNING' | 'NOT_RESPONDING' | 'ATTENTION';
  process: OwnedProcess | null;
  issue: string | null;
}
export interface ProfileStorage {
  slot: ProfileSlot; bytes: number; files: number; partial: boolean;
  disk: {availableBytes: number; totalBytes: number} | null;
  cache: {bytes: number; files: number; partial: boolean; canClear: boolean; issue: string | null};
}
export interface SettingsReport {activeFps:number; backgroundFps:number; renderDistanceChunks:number; canApplyPreset:boolean; changedProfiles:number|null}
export interface SessionSnapshot {schema: 1; profiles: ProfileSession[]; storage: ProfileStorage[] | null; settings?:SettingsReport}
export interface PreviewPlacement {slot: ProfileSlot; viewport: WorkspaceViewport}
export interface PreviewStatus {slot: ProfileSlot; registered: boolean; reason: string | null}
export interface PreviewReport {schema: 1; tiles: PreviewStatus[]}
export interface ProfileBinding {schema: 1; required: boolean; configured: boolean; issue: string | null}
export interface DataLocation {schema:1; defaultPath:string; path:string|null; profiles:{slot:ProfileSlot;path:string}[]; canLocate:boolean}
const record = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value);
const exact = (value: Record<string, unknown>, names: string[]) =>
  Object.keys(value).length === names.length && names.every(name => Object.hasOwn(value, name));

export function validFpsLimits(active:unknown,background:unknown):boolean {
  return Number.isInteger(active)&&Number(active)>=1&&Number(active)<=1000&&
    Number.isInteger(background)&&Number(background)>=1&&Number(background)<=Number(active);
}

export function parseDataLocation(value:unknown):DataLocation {
  const path=(value:unknown):value is string=>typeof value==='string'&&value.length>0&&value.length<=4096&&!/[\x00-\x1f]/.test(value);
  if(!record(value)||!exact(value,['schema','defaultPath','path','profiles','canLocate'])||value.schema!==1||!path(value.defaultPath)||
      !(value.path===null||path(value.path))||typeof value.canLocate!=='boolean'||!Array.isArray(value.profiles)||
      value.profiles.length!==(value.path===null?0:5))throw new TypeError('Invalid data location');
  value.profiles.forEach((profile,index)=>{
    if(!record(profile)||!exact(profile,['slot','path'])||profile.slot!==index+1||!path(profile.path))throw new TypeError('Invalid profile location');
  });
  return value as unknown as DataLocation;
}

export function parseProfileBinding(value: unknown): ProfileBinding {
  if (!record(value) || !exact(value, ['schema','required','configured','issue']) || value.schema !== 1 ||
      typeof value.required !== 'boolean' || typeof value.configured !== 'boolean' ||
      !(value.issue === null || typeof value.issue === 'string' && value.issue.length <= 8192) ||
      value.configured && value.issue !== null || !value.required && !value.configured) throw new TypeError('Invalid profile binding');
  return value as unknown as ProfileBinding;
}

export function parseSession(value: unknown): SessionSnapshot {
  if (!record(value) || !exact(value, Object.hasOwn(value,'settings') ? ['schema','profiles','storage','settings'] : ['schema', 'profiles', 'storage']) || value.schema !== 1 ||
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
      if (!record(storage) || !exact(storage, ['slot', 'bytes', 'files', 'partial', 'disk', 'cache']) || storage.slot !== index + 1 ||
          !Number.isSafeInteger(storage.bytes) || Number(storage.bytes) < 0 || !Number.isInteger(storage.files) ||
          Number(storage.files) < 0 || Number(storage.files) > 100000 || typeof storage.partial !== 'boolean') throw new TypeError('Invalid storage metadata');
      const {disk, cache} = storage;
      if (disk !== null && (!record(disk) || !exact(disk, ['availableBytes', 'totalBytes']) ||
          !Number.isSafeInteger(disk.availableBytes) || Number(disk.availableBytes) < 0 ||
          !Number.isSafeInteger(disk.totalBytes) || Number(disk.totalBytes) <= 0 ||
          Number(disk.availableBytes) > Number(disk.totalBytes))) throw new TypeError('Invalid disk metadata');
      if (!record(cache) || !exact(cache, ['bytes', 'files', 'partial', 'canClear', 'issue']) ||
          !Number.isSafeInteger(cache.bytes) || Number(cache.bytes) < 0 || !Number.isInteger(cache.files) ||
          Number(cache.files) < 0 || Number(cache.files) > 2048 || typeof cache.partial !== 'boolean' ||
          typeof cache.canClear !== 'boolean' || !(cache.issue === null || typeof cache.issue === 'string' && cache.issue.length <= 8192) ||
          cache.canClear && (cache.partial || cache.files === 0 || cache.issue !== null ||
            (value.profiles as ProfileSession[]).some(profile => ['STARTING', 'RUNNING', 'NOT_RESPONDING'].includes(profile.state)))) throw new TypeError('Invalid cache metadata');
    });
  }
  if(Object.hasOwn(value,'settings')) {
    const settings=value.settings;
    if(!record(settings)||!exact(settings,['activeFps','backgroundFps','renderDistanceChunks','canApplyPreset','changedProfiles'])||
      !Number.isInteger(settings.activeFps)||Number(settings.activeFps)<1||Number(settings.activeFps)>1000||
      !Number.isInteger(settings.backgroundFps)||Number(settings.backgroundFps)<1||Number(settings.backgroundFps)>Number(settings.activeFps)||
      settings.renderDistanceChunks!==12||typeof settings.canApplyPreset!=='boolean'||
      !(settings.changedProfiles===null||Number.isInteger(settings.changedProfiles)&&Number(settings.changedProfiles)>=0&&Number(settings.changedProfiles)<=5)||
      settings.canApplyPreset&&(value.profiles as ProfileSession[]).some(p=>['STARTING','RUNNING','NOT_RESPONDING'].includes(p.state))||
      settings.changedProfiles!==null&&!settings.canApplyPreset)throw new TypeError('Invalid settings response');
  }
  return value as unknown as SessionSnapshot;
}

function settingsSession(value:unknown):SessionSnapshot & {settings:SettingsReport} {
  const snapshot=parseSession(value);
  if(!snapshot.settings)throw new TypeError('Settings are unavailable. Update Lazy Tester and try again.');
  return snapshot as SessionSnapshot & {settings:SettingsReport};
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
  settings:async()=>settingsSession(await invokeRuntime<unknown>('lazy_tester_session',{request:{operation:'InspectSettings'}})),
  savePerformance:async(activeFps:number,backgroundFps:number)=>{
    if(!validFpsLimits(activeFps,backgroundFps))throw new TypeError('Use whole FPS limits from 1 to 1000. Background FPS must not exceed active FPS.');
    return settingsSession(await invokeRuntime<unknown>('lazy_tester_session',{request:{operation:'SavePerformance',activeFps,backgroundFps}}));
  },
  applyPreset:async()=>settingsSession(await invokeRuntime<unknown>('lazy_tester_session',{request:{operation:'ApplyPreset'}})),
  dataLocation:async(action:'Inspect'|'Open'|'Locate'|'Choose'='Inspect')=>parseDataLocation(await invokeRuntime<unknown>('lazy_tester_data_location',{action})),
  binding: async (importProfiles = false) => parseProfileBinding(await invokeRuntime<unknown>('lazy_tester_profile_binding', {import: importProfiles})),
  inspect: async () => parseSession(await invokeRuntime<unknown>('lazy_tester_session', {request: {operation: 'Inspect'}})),
  launch: async (slot: ProfileSlot) => parseSession(await invokeRuntime<unknown>('lazy_tester_session', {request: {operation: 'Launch', slot}})),
  storage: async () => parseSession(await invokeRuntime<unknown>('lazy_tester_session', {request: {operation: 'InspectStorage'}})),
  openFolder: async (slot: ProfileSlot) => parseSession(await invokeRuntime<unknown>('lazy_tester_session', {request: {operation: 'OpenProfileFolder', slot}})),
  clearCache: async (slot: ProfileSlot) => parseSession(await invokeRuntime<unknown>('lazy_tester_session', {request: {operation: 'ClearProfileCache', slot}})),
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
