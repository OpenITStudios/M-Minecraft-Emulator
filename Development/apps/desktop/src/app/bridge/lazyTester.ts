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
  downloadCache?: ProfileStorage['cache'];
  resourcePacks?: {
    manifested:number; partial:boolean; issue:string|null;
    worldDeclarations?: {
      worldsScanned:number; worldsWithReferences:number; packReferences:number;
      partial:boolean; issue:string|null;
    };
  };
  gameSettings?: {hasSettings:boolean; backupAvailable:boolean; canReset:boolean; canRestore:boolean; issue:string|null; improvedInputResponseEnabled?:boolean; textureStreamingEnabled?:boolean};
  fullClear?: {canClear:boolean; pending:boolean; confirmationToken:string|null; issue:string|null};
}
// Backend implementation availability, never game sign-in or profile readiness.
export type NativeBackendSupport = 'WindowsNative' | 'Unsupported';
export function parseNativeBackendSupport(value: unknown): NativeBackendSupport {
  if (value !== 'WindowsNative' && value !== 'Unsupported') {
    throw new TypeError('Invalid native backend capability');
  }
  return value;
}
export type GraphicsPreset = 'Preserve' | 'High' | 'Low';
export type BackgroundAudio = 'Keep'|'Mute';
export interface StoragePreferences {autoClearWebCache:boolean; autoClearDownloadCache?:boolean; autoClearOldLogs?:boolean}
export interface SettingsReport {storagePreferences?:StoragePreferences; backgroundAudio?:BackgroundAudio; audioIssue?:string; graphicsPresets?:GraphicsPreset[]; activeFps:number; backgroundFps:number; renderDistanceChunks:number; canApplyPreset:boolean; changedProfiles:number|null}
export type SelectiveCategory = 'Worlds' | 'ResourcePacks' | 'BehaviorPacks';
export interface SelectiveInventory {slot: ProfileSlot; category: SelectiveCategory; items: {root: string; name: string; identity: string}[]}
export interface SelectiveSelection {category: SelectiveCategory; root: string; name: string; identity: string}
export interface SelectiveReviewReceipt {slot: ProfileSlot; bytes: number; files: number; items: SelectiveSelection[]; warnings: string[]; confirmationToken: string|null}
export interface SessionSnapshot {selectiveReview?: SelectiveReviewReceipt; selectiveInventory?: SelectiveInventory; schema: 1; profiles: ProfileSession[]; storage: ProfileStorage[] | null; settings?:SettingsReport; agentActivity?:AgentActivityReport}
export interface PreviewPlacement {slot: ProfileSlot; viewport: WorkspaceViewport}
export interface PreviewStatus {slot: ProfileSlot; registered: boolean; reason: string | null}
export interface PreviewReport {schema: 1; tiles: PreviewStatus[]}

export interface AgentProbeCounts {
  expected:number;
  pass:number;
  fail:number;
  info:number;
  look:number;
}
export interface AgentActivityItem {
  kind:'AI_INPUT'|'SKILLS_PROBE'|'VISUAL_OBSERVATION';
  slot:ProfileSlot;
  status:string;
  importance:'INFORMATION'|'ATTENTION';
  evidence:string;
  reportedOutcome?:string;
  probeCounts?:AgentProbeCounts;
}
export interface AgentActivityReport {
  schema:1;
  source:'CURRENT_MCP_PROCESS_MEMORY_ONLY';
  snapshotAt:string;
  displayHint:'HIDDEN_NO_LOCAL_SIGNAL'|'ACTIVITY'|'ATTENTION';
  desktopControl:'NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER';
  minecraftInputEffect:'NOT_VERIFIED';
  mapGameplayAcceptance:'NOT_TESTED';
  items:AgentActivityItem[];
  stale:boolean;
}
const agentItemStatuses:Record<AgentActivityItem['kind'],Record<string,string>>={
  AI_INPUT:{
    NATIVE_RELEASE_UNCONFIRMED:'LOCAL_MCP_RECOVERY_HOLD',
    STARTUP_RECONCILIATION_REQUIRED:'LOCAL_MCP_RECOVERY_HOLD',
    EXPERIMENTAL_INPUT_REQUEST_IN_FLIGHT:'LOCAL_MCP_REQUEST_ONLY',
  },
  SKILLS_PROBE:{
    PROBE_REPORT_AVAILABLE:'ORIGINAL_PROBE_LOG_UNVERIFIED_ORIGIN',
    LOCAL_QA_REVIEW_IN_PROGRESS:'MCP_PROCESS_ONLY',
    LOCAL_QA_PREPARATION_IN_PROGRESS:'MCP_PROCESS_ONLY',
    AWAITING_AUTHORIZED_COMMAND_OR_EVIDENCE:'SOURCE_CHECKPOINT_ONLY',
    CHECKPOINT_EXPIRED_OR_UNAVAILABLE:'SOURCE_CHECKPOINT_ONLY',
  },
  VISUAL_OBSERVATION:{
    SAMPLER_ACTIVE_NO_RECENT_FRAME:'LOCAL_SAMPLER_ONLY',
    SAMPLER_ACTIVE_RECENT_FRAME:'LOCAL_SAMPLER_ONLY',
  },
};
export function parseAgentActivity(value:unknown):AgentActivityReport {
  if(!record(value)||
    !exact(value,['schema','source','snapshotAt','displayHint','desktopControl',
      'minecraftInputEffect','mapGameplayAcceptance','items','stale'])||
    value.schema!==1||value.source!=='CURRENT_MCP_PROCESS_MEMORY_ONLY'||
    typeof value.snapshotAt!=='string'||
    !/^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d{3}Z$/.test(value.snapshotAt)||
    !Number.isFinite(Date.parse(value.snapshotAt))||
    !['HIDDEN_NO_LOCAL_SIGNAL','ACTIVITY','ATTENTION'].includes(String(value.displayHint))||
    value.desktopControl!=='NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER'||
    value.minecraftInputEffect!=='NOT_VERIFIED'||
    value.mapGameplayAcceptance!=='NOT_TESTED'||
    typeof value.stale!=='boolean'||!Array.isArray(value.items)||value.items.length>15)
    throw new TypeError('Invalid agent activity status');
  const seen=new Set<string>();
  let attention=false;
  for(const entry of value.items) {
    if(!record(entry)||!['AI_INPUT','SKILLS_PROBE','VISUAL_OBSERVATION'].includes(String(entry.kind))||
       !exact(entry,['kind','slot','status','importance','evidence',
         ...(Object.hasOwn(entry,'reportedOutcome')?['reportedOutcome']:[]),
         ...(Object.hasOwn(entry,'probeCounts')?['probeCounts']:[])])||
       !profileSlots.includes(entry.slot as ProfileSlot)||
       !['INFORMATION','ATTENTION'].includes(String(entry.importance))||
       typeof entry.status!=='string'||typeof entry.evidence!=='string')throw new TypeError('Invalid agent activity item');
    const allowed=agentItemStatuses[entry.kind as AgentActivityItem['kind']];
    if(!Object.hasOwn(allowed,entry.status)||allowed[entry.status]!==entry.evidence)
      throw new TypeError('Invalid agent activity evidence');
    const key=String(entry.kind)+':'+String(entry.slot);
    if(seen.has(key))throw new TypeError('Duplicate agent activity');
    seen.add(key);
    attention ||= entry.importance==='ATTENTION';
    const important=['NATIVE_RELEASE_UNCONFIRMED',
      'STARTUP_RECONCILIATION_REQUIRED',
      'CHECKPOINT_EXPIRED_OR_UNAVAILABLE'].includes(entry.status) ||
      entry.status==='PROBE_REPORT_AVAILABLE' &&
        entry.reportedOutcome!=='PROBE_REPORTED_PASS';
    if((entry.importance==='ATTENTION')!==important)
      throw new TypeError('Incorrect agent activity importance');
    const terminal=entry.kind==='SKILLS_PROBE'&&entry.status==='PROBE_REPORT_AVAILABLE';
    if(terminal?!['PROBE_REPORTED_PASS','FAIL','NEEDS_REVIEW','INCOMPLETE','BLOCKED',
      'UNRECOGNIZED_TERMINAL_STATUS'].includes(String(entry.reportedOutcome)):
      Object.hasOwn(entry,'reportedOutcome'))
      throw new TypeError('Invalid reported Probe outcome');
    if(Object.hasOwn(entry,'probeCounts')) {
      const counts=entry.probeCounts;
      if(!terminal ||
        !['PROBE_REPORTED_PASS','FAIL','NEEDS_REVIEW'].includes(String(entry.reportedOutcome)) ||
        !record(counts) ||
        !exact(counts,['expected','pass','fail','info','look']) ||
        typeof counts.expected!=='number'||!Number.isSafeInteger(counts.expected)||counts.expected<1||counts.expected>80||
        !['pass','fail','info','look'].every(k=>Number.isSafeInteger(counts[k])&&
          Number(counts[k])>=0&&Number(counts[k])<=80) ||
        Number(counts.pass)+Number(counts.fail)+Number(counts.info)+Number(counts.look)!==counts.expected ||
        entry.reportedOutcome==='PROBE_REPORTED_PASS'&&counts.pass!==counts.expected ||
        entry.reportedOutcome==='FAIL'&&counts.fail===0 ||
        entry.reportedOutcome==='NEEDS_REVIEW'&&
          (counts.fail!==0||Number(counts.info)+Number(counts.look)===0))
        throw new TypeError('Invalid original Probe counts');
    }
  }
  const expected=attention?'ATTENTION':value.items.length?'ACTIVITY':'HIDDEN_NO_LOCAL_SIGNAL';
  if(value.displayHint!==expected || value.stale===true&&value.displayHint!=='ATTENTION')
    throw new TypeError('Invalid agent activity importance');
  return value as unknown as AgentActivityReport;
}

// An existing workspace-surface-error string carries this observation-only
// diagnostic. It is not game readiness, and has no authority to launch/close.
export interface DwmPreviewInvalidation {slot: ProfileSlot; process: OwnedProcess; reason: string}
const dwmInvalidationPrefix = 'dwm-preview-invalidated:';
export function parseDwmPreviewInvalidation(value: unknown): DwmPreviewInvalidation | null {
  if(typeof value !== 'string' || !value.startsWith(dwmInvalidationPrefix))return null;
  let data:unknown;
  try { data=JSON.parse(value.slice(dwmInvalidationPrefix.length)); }
  catch { throw new TypeError('Invalid DWM preview diagnostic'); }
  if(!record(data) || !exact(data,['schema','slot','process','reason']) || data.schema!==1 ||
      !profileSlots.includes(data.slot as ProfileSlot) ||
      typeof data.reason!=='string'||data.reason.length===0||data.reason.length>2048) {
    throw new TypeError('Invalid DWM preview diagnostic');
  }
  return {slot:data.slot as ProfileSlot,process:parseObservedProcess(data.process),reason:data.reason};
}
export function dwmPreviewInvalidationApplies(
  notice:DwmPreviewInvalidation, profile:ProfileSession|undefined,
):boolean {
  const p=profile?.process, n=notice.process;
  return !!p && profile?.state==='RUNNING' && profile.slot===notice.slot &&
    p.pid===n.pid && p.createdFileTime===n.createdFileTime &&
    p.sessionId===n.sessionId && p.windowHandle===n.windowHandle;
}
export interface ProfileBinding {schema: 1; required: boolean; configured: boolean; issue: string | null}
export interface DataLocation {schema:1; defaultPath:string; path:string|null; profiles:{slot:ProfileSlot;path:string}[]; canLocate:boolean}
const record = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value);
const exact = (value: Record<string, unknown>, names: string[]) =>
  Object.keys(value).length === names.length && names.every(name => Object.hasOwn(value, name));

export function validFpsLimits(active:unknown,background:unknown):boolean {
  return Number.isInteger(active)&&Number(active)>=5&&Number(active)<=1000&&
    Number.isInteger(background)&&Number(background)>=5&&Number(background)<=Number(active);
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
  if (!record(value) || !exact(value, ['schema','profiles','storage',
     ...(Object.hasOwn(value,'settings')?['settings']:[]),
     ...(Object.hasOwn(value,'selectiveInventory')?['selectiveInventory']:[]),
     ...(Object.hasOwn(value,'selectiveReview')?['selectiveReview']:[]),
     ...(Object.hasOwn(value,'agentActivity')?['agentActivity']:[])]) || value.schema !== 1 ||
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
      if (!record(storage) || !exact(storage, ['slot','bytes','files','partial','disk','cache',...(Object.hasOwn(storage,'downloadCache')?['downloadCache']:[]),...(Object.hasOwn(storage,'gameSettings')?['gameSettings']:[]),...(Object.hasOwn(storage,'resourcePacks')?['resourcePacks']:[]),...(Object.hasOwn(storage,'fullClear')?['fullClear']:[])]) || storage.slot !== index + 1 ||
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
            (value.profiles as ProfileSession[]).some(profile => profile.slot === storage.slot && ['STARTING', 'RUNNING', 'NOT_RESPONDING'].includes(profile.state)))) throw new TypeError('Invalid cache metadata');
      if(Object.hasOwn(storage,'downloadCache')) {
        const download=storage.downloadCache;
        if(!record(download)||!exact(download,['bytes','files','partial','canClear','issue'])||
          !Number.isSafeInteger(download.bytes)||Number(download.bytes)<0||
          !Number.isInteger(download.files)||Number(download.files)<0||Number(download.files)>2048||
          typeof download.partial!=='boolean'||typeof download.canClear!=='boolean'||
          !(download.issue===null||typeof download.issue==='string'&&download.issue.length<=8192)||
          download.canClear&&(download.partial||download.files===0||download.issue!==null||
            (value.profiles as ProfileSession[]).some(profile=>profile.slot===storage.slot&&['STARTING','RUNNING','NOT_RESPONDING'].includes(profile.state))))
          throw new TypeError('Invalid download cache metadata');
      }
      // Pack folders are a read-only inventory, never evidence that a world
      // has activated any of them or that game RAM can be reclaimed.
      if(Object.hasOwn(storage,'resourcePacks')) {
        const packs=storage.resourcePacks;
        if(!record(packs)||!exact(packs,['manifested','partial','issue',
          ...(Object.hasOwn(packs,'worldDeclarations')?['worldDeclarations']:[])])||
          !Number.isInteger(packs.manifested)||Number(packs.manifested)<0||Number(packs.manifested)>128||
          typeof packs.partial!=='boolean'||!(packs.issue===null||typeof packs.issue==='string'&&packs.issue.length<=8192)||
          packs.partial!== (packs.issue!==null) || packs.partial&&packs.manifested!==0)
          throw new TypeError('Invalid resource pack inventory');
        if(Object.hasOwn(packs,'worldDeclarations')) {
          const worlds=packs.worldDeclarations;
          if(!record(worlds)||!exact(worlds,['worldsScanned','worldsWithReferences','packReferences','partial','issue'])||
            !Number.isInteger(worlds.worldsScanned)||Number(worlds.worldsScanned)<0||Number(worlds.worldsScanned)>128||
            !Number.isInteger(worlds.worldsWithReferences)||Number(worlds.worldsWithReferences)<0||
            Number(worlds.worldsWithReferences)>Number(worlds.worldsScanned)||
            !Number.isInteger(worlds.packReferences)||Number(worlds.packReferences)<0||Number(worlds.packReferences)>256||
            typeof worlds.partial!=='boolean'||
            !(worlds.issue===null||typeof worlds.issue==='string'&&worlds.issue.length<=8192)||
            worlds.partial!==(worlds.issue!==null)||
            (worlds.partial&&(worlds.worldsScanned!==0||worlds.worldsWithReferences!==0||worlds.packReferences!==0))||
            (!worlds.partial&&worlds.worldsWithReferences===0&&worlds.packReferences!==0)||
            (!worlds.partial&&Number(worlds.worldsWithReferences)>Number(worlds.packReferences))) {
            throw new TypeError('Invalid resource pack declarations');
          }
        }
      }
      if(Object.hasOwn(storage,'gameSettings')) {
        const game=storage.gameSettings;
        const active=(value.profiles as ProfileSession[]).some(profile=>profile.slot===storage.slot&&['STARTING','RUNNING','NOT_RESPONDING'].includes(profile.state));
        if(!record(game)||!exact(game,['hasSettings','backupAvailable','canReset','canRestore','issue',
          ...(Object.hasOwn(game,'improvedInputResponseEnabled')?['improvedInputResponseEnabled']:[]),
          ...(Object.hasOwn(game,'textureStreamingEnabled')?['textureStreamingEnabled']:[])])||
          ['hasSettings','backupAvailable','canReset','canRestore'].some(key=>typeof game[key]!=='boolean')||
          (Object.hasOwn(game,'improvedInputResponseEnabled')&&
            (typeof game.improvedInputResponseEnabled!=='boolean'||!game.hasSettings||game.issue!==null))||
          (Object.hasOwn(game,'textureStreamingEnabled')&&
            (typeof game.textureStreamingEnabled!=='boolean'||!game.hasSettings||game.issue!==null))||
          !(game.issue===null||typeof game.issue==='string'&&game.issue.length<=8192)||
          game.canReset&&(!game.hasSettings||game.issue!==null||active)||
          game.canRestore&&(!game.backupAvailable||game.issue!==null||active))throw new TypeError('Invalid settings recovery metadata');
      }
      if(Object.hasOwn(storage,'fullClear')) {
        const full=storage.fullClear;
        const active=(value.profiles as ProfileSession[]).some(profile=>profile.slot===storage.slot&&['STARTING','RUNNING','NOT_RESPONDING'].includes(profile.state));
        if(!record(full)||!exact(full,['canClear','pending','confirmationToken','issue'])||typeof full.canClear!=='boolean'||typeof full.pending!=='boolean'||
          !(full.confirmationToken===null||typeof full.confirmationToken==='string'&&/^[0-9a-f]{64}$/.test(full.confirmationToken))||
          !(full.issue===null||typeof full.issue==='string'&&full.issue.length<=8192)||
          full.canClear&&(full.confirmationToken===null||full.issue!==null||active))throw new TypeError('Invalid full-clear metadata');
        if(full.pending&&(cache.canClear||(record(storage.downloadCache)&&storage.downloadCache.canClear===true)))throw new TypeError('Pending full clear conflicts with cache admission');
      }
    });
  }
  if(Object.hasOwn(value,'settings')) {
    const settings=value.settings;
    if(!record(settings)||!exact(settings,['activeFps','backgroundFps','renderDistanceChunks','canApplyPreset','changedProfiles',...['graphicsPresets','backgroundAudio','audioIssue','storagePreferences'].filter(key=>Object.hasOwn(settings,key))])||
      Object.hasOwn(settings,'storagePreferences')&&(!record(settings.storagePreferences)||!exact(settings.storagePreferences,['autoClearWebCache',...['autoClearDownloadCache','autoClearOldLogs'].filter(key=>Object.hasOwn(settings.storagePreferences as object,key))])||typeof settings.storagePreferences.autoClearWebCache!=='boolean'||Object.hasOwn(settings.storagePreferences,'autoClearDownloadCache')&&typeof settings.storagePreferences.autoClearDownloadCache!=='boolean'||Object.hasOwn(settings.storagePreferences,'autoClearOldLogs')&&typeof settings.storagePreferences.autoClearOldLogs!=='boolean')||
      Object.hasOwn(settings,'backgroundAudio')&&!['Keep','Mute'].includes(String(settings.backgroundAudio))||
      Object.hasOwn(settings,'audioIssue')&&(!Object.hasOwn(settings,'backgroundAudio')||typeof settings.audioIssue!=='string'||settings.audioIssue.length>8192)||
      Object.hasOwn(settings,'graphicsPresets')&&(!Array.isArray(settings.graphicsPresets)||settings.graphicsPresets.length!==5||settings.graphicsPresets.some(p=>p!=='Preserve'&&p!=='High'&&p!=='Low'))||
      !Number.isInteger(settings.activeFps)||Number(settings.activeFps)<1||Number(settings.activeFps)>1000||
      !Number.isInteger(settings.backgroundFps)||Number(settings.backgroundFps)<1||Number(settings.backgroundFps)>Number(settings.activeFps)||
      settings.renderDistanceChunks!==12||typeof settings.canApplyPreset!=='boolean'||
      !(settings.changedProfiles===null||Number.isInteger(settings.changedProfiles)&&Number(settings.changedProfiles)>=0&&Number(settings.changedProfiles)<=5)||
      settings.canApplyPreset&&(value.profiles as ProfileSession[]).some(p=>['STARTING','RUNNING','NOT_RESPONDING'].includes(p.state))||
      settings.changedProfiles!==null&&!settings.canApplyPreset)throw new TypeError('Invalid settings response');
  }
  if(Object.hasOwn(value,'selectiveInventory')) {
    const inventory=value.selectiveInventory;
    if(!record(inventory)||!exact(inventory,['slot','category','items'])||
      !profileSlots.includes(inventory.slot as ProfileSlot)||!Array.isArray(inventory.items)||inventory.items.length>128||
      (value.profiles as ProfileSession[]).some(p=>p.slot===inventory.slot&&p.state!=='STOPPED'))
      throw new TypeError('Invalid selective inventory');
    const roots=inventory.category==='Worlds'?['minecraftWorlds']:
      inventory.category==='ResourcePacks'?['resource_packs','development_resource_packs']:
      inventory.category==='BehaviorPacks'?['behavior_packs','development_behavior_packs']:[];
    if(!roots.length)throw new TypeError('Invalid inventory category');
    const keys=new Set<string>();
    const ids=new Set<string>();
    for(const item of inventory.items) {
      if(!record(item)||!exact(item,['root','name','identity'])||typeof item.identity!=='string'||!/^[0-9a-f]{64}$/.test(item.identity)||typeof item.root!=='string'||!roots.includes(item.root)||
        typeof item.name!=='string'||!item.name.length||item.name.length>255||item.name==='.'||item.name==='..'||
        /[\\/\x00-\x1f:]/.test(item.name))throw new TypeError('Invalid inventory item');
      const key=item.root+'/'+item.name.toLowerCase();
      if(keys.has(key)||ids.has(item.identity))throw new TypeError('Duplicate inventory item');
      keys.add(key);ids.add(item.identity);
    }
  }
  if(Object.hasOwn(value,'selectiveReview')) {
    const review=value.selectiveReview;
    if(!record(review)||!exact(review,['slot','items','warnings','confirmationToken','bytes','files'])||
      !Number.isSafeInteger(review.bytes)||Number(review.bytes)<0||
      !Number.isInteger(review.files)||Number(review.files)<0||Number(review.files)>2048||
      !profileSlots.includes(review.slot as ProfileSlot)||
      (value.profiles as ProfileSession[]).some(p=>p.slot===review.slot&&p.state!=='STOPPED')||
      !Array.isArray(review.items)||!review.items.length||review.items.length>128||
      !Array.isArray(review.warnings)||review.warnings.length>385||
      review.warnings.some(w=>typeof w!=='string'||w.length>1024)||
      !(review.confirmationToken===null||typeof review.confirmationToken==='string'&&/^[0-9a-f]{64}$/.test(review.confirmationToken))||
      (review.warnings.length>0)!==(review.confirmationToken===null))throw new TypeError('Invalid selective review');
    const ids=new Set<string>();
    const names=new Set<string>();
    for(const item of review.items) {
      if(!record(item)||!exact(item,['category','root','name','identity'])||
        !['Worlds','ResourcePacks','BehaviorPacks'].includes(String(item.category))||
        typeof item.root!=='string'||typeof item.name!=='string'||!item.name.length||item.name.length>255||item.name==='.'||item.name==='..'||/[\\/\x00-\x1f:]/.test(item.name)||
        typeof item.identity!=='string'||!/^[0-9a-f]{64}$/.test(item.identity)||ids.has(item.identity))
        throw new TypeError('Invalid reviewed item');
      const roots=item.category==='Worlds'?['minecraftWorlds']:item.category==='ResourcePacks'?['resource_packs','development_resource_packs']:['behavior_packs','development_behavior_packs'];
      if(!roots.includes(item.root))throw new TypeError('Invalid reviewed root');
      const key=item.root+'/'+item.name.toLowerCase();
      if(names.has(key))throw new TypeError('Duplicate reviewed item');
      names.add(key);ids.add(item.identity);
      if(item.category!=='Worlds'&&review.confirmationToken!==null)throw new TypeError('Pack deletion remains unavailable');
    }
  }
  if(Object.hasOwn(value,'agentActivity'))parseAgentActivity(value.agentActivity);
  return value as unknown as SessionSnapshot;
}

function settingsSession(value:unknown):SessionSnapshot & {settings:SettingsReport} {
  const snapshot=parseSession(value);
  if(!snapshot.settings)throw new TypeError('Settings are unavailable. Update Lazy Tester and try again.');
  return snapshot as SessionSnapshot & {settings:SettingsReport};
}

// UI preflight only. ATTENTION can mean a previously exited game that is
// safe to retry manually; it can also mean a missing store. Never infer native
// permission from the issue text. The Rust controller reaps, checks retained
// storage, and refuses unsafe retries. STARTING/NOT_RESPONDING stay blocked.
export function canRequestLaunch(profile:ProfileSession|undefined):boolean {
  return !!profile && profile.process===null &&
    (profile.state==='ATTENTION' ||
      (profile.state==='STOPPED' && profile.issue===null));
}

export function firstEmbeddedProfile(profiles:ProfileSession[],excluded:ReadonlySet<ProfileSlot>):ProfileSlot|null {
  return profiles.find(profile=>profile.state==='RUNNING'&&!excluded.has(profile.slot))?.slot??null;
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

export interface SelectedCloseReport {
  schema:1;
  presentationReleased:boolean;
  snapshot:SessionSnapshot|null;
  issue:string|null;
}
export function parseSelectedClose(value:unknown):SelectedCloseReport {
  if(!record(value)||!exact(value,['schema','presentationReleased','snapshot','issue'])||
     value.schema!==1||typeof value.presentationReleased!=='boolean'||
     !(value.issue===null||typeof value.issue==='string'&&value.issue.length>0&&value.issue.length<=8192)||
     (value.snapshot===null)!==(value.issue!==null)||
     (!value.presentationReleased && value.snapshot!==null)) throw new TypeError('Invalid selected close receipt');
  const snapshot=value.snapshot===null?null:parseSession(value.snapshot);
  return {schema:1,presentationReleased:value.presentationReleased as boolean,snapshot,issue:value.issue as string|null};
}

export interface ScreenshotReceipt {slot:ProfileSlot; width:number; height:number; bytes:number}
export function parseScreenshotReceipt(value:unknown,slot:ProfileSlot):ScreenshotReceipt|null {
  if(value===null)return null;
  if(!record(value)||!exact(value,['slot','width','height','bytes'])||value.slot!==slot||
    ![value.width,value.height].every(n=>typeof n==='number'&&Number.isInteger(n)&&n>0&&n<=4096)||
    typeof value.bytes!=='number'||!Number.isInteger(value.bytes)||value.bytes<26||value.bytes>80*1024*1024)
    throw new TypeError('Invalid screenshot receipt');
  return value as unknown as ScreenshotReceipt;
}
export type RecordingState='Starting'|'Recording'|'Stopping'|'Saved'|'Failed';
export interface RecordingReceipt {slot:ProfileSlot; state:RecordingState; frames:number; recoveryFile:string|null}
export function recordingActive(receipt:RecordingReceipt|undefined):boolean {
  return receipt!==undefined&&['Starting','Recording','Stopping'].includes(receipt.state);
}
export function parseRecordingStatus(value:unknown):RecordingReceipt[] {
  if(!Array.isArray(value)||value.length>5)throw new TypeError('Invalid recording status');
  const seen=new Set<number>();
  return value.map((item:unknown)=>{
    if(!record(item)||!exact(item,['slot','state','frames','recoveryFile'])||
      !profileSlots.includes(item.slot as ProfileSlot)||seen.has(item.slot as number)||
      typeof item.state!=='string'||!['Starting','Recording','Stopping','Saved','Failed'].includes(item.state)||
      typeof item.frames!=='number'||!Number.isSafeInteger(item.frames)||item.frames<0||
      ((item.state==='Recording'||item.state==='Saved')&&item.frames===0)||
      (item.state==='Starting'&&item.frames!==0)||
      (item.recoveryFile!==null&&(item.state!=='Failed'||item.frames===0||typeof item.recoveryFile!=='string'||
        !/^\.lazy-tester-video-[A-Za-z0-9_-]{1,64}\.part$/.test(item.recoveryFile))))
      throw new TypeError('Invalid recording status');
    seen.add(item.slot as number);
    return item as unknown as RecordingReceipt;
  });
}
export const lazyTester = {
  recordingAvailable:async()=>await invokeRuntime<unknown>('lazy_tester_recording_available')===true,
  recordingStatus:async()=>parseRecordingStatus(await invokeRuntime<unknown>('lazy_tester_recording_status')),
  startRecording:async(slot:ProfileSlot,expected:OwnedProcess)=>{
    const result=await invokeRuntime<unknown>('lazy_tester_start_recording',{slot,expected});
    if(typeof result!=='boolean')throw new TypeError('Invalid recording start receipt');
    return result;
  },
  stopRecording:async(slot:ProfileSlot)=>{await invokeRuntime<void>('lazy_tester_stop_recording',{slot});},
  screenshotAvailable:async()=>await invokeRuntime<unknown>('lazy_tester_screenshot_available')===true,
  screenshot:async(slot:ProfileSlot,expected:OwnedProcess)=>parseScreenshotReceipt(
    await invokeRuntime<unknown>('lazy_tester_screenshot',{slot,expected}),slot),
  saveStoragePreferences:async(preferences:StoragePreferences)=>{
    if(typeof preferences.autoClearWebCache!=='boolean'||preferences.autoClearDownloadCache===true||preferences.autoClearOldLogs===true)throw new TypeError('Unverified storage cleanup preference');
    const snapshot=settingsSession(await invokeRuntime<unknown>('lazy_tester_session',{request:{operation:'SaveStoragePreferences',preferences}}));
    const saved=snapshot.settings.storagePreferences;
    if(!saved||saved.autoClearWebCache!==preferences.autoClearWebCache||(saved.autoClearDownloadCache??false)!==(preferences.autoClearDownloadCache??false)||(saved.autoClearOldLogs??false)!==(preferences.autoClearOldLogs??false))throw new TypeError('Storage preference could not be confirmed');
    return snapshot;
  },
  saveAudio:async(mode:BackgroundAudio)=>{
    if(!['Keep','Mute'].includes(mode))throw new TypeError('Invalid background audio preference');
    return settingsSession(await invokeRuntime<unknown>('lazy_tester_session',{request:{operation:'SaveAudio',mode}}));
  },
  reportStartup:(report:{connection:string;issue:string;cards:number;enabled:number})=>invokeRuntime<void>('startup_diagnostic_result',report),
  backendSupport:async()=>parseNativeBackendSupport(await invokeRuntime<unknown>('lazy_tester_backend_support')),
  settings:async()=>settingsSession(await invokeRuntime<unknown>('lazy_tester_session',{request:{operation:'InspectSettings'}})),
  savePerformance:async(activeFps:number,backgroundFps:number)=>{
    if(!validFpsLimits(activeFps,backgroundFps))throw new TypeError('Use whole FPS limits from 5 to 1000. Background FPS must not exceed active FPS.');
    return settingsSession(await invokeRuntime<unknown>('lazy_tester_session',{request:{operation:'SavePerformance',activeFps,backgroundFps}}));
  },
  dataLocation:async(action:'Inspect'|'Open'|'Locate'|'Choose'='Inspect')=>parseDataLocation(await invokeRuntime<unknown>('lazy_tester_data_location',{action})),
  binding: async (importProfiles = false) => parseProfileBinding(await invokeRuntime<unknown>('lazy_tester_profile_binding', {import: importProfiles})),
  inspect: async () => parseSession(await invokeRuntime<unknown>('lazy_tester_session', {request: {operation: 'Inspect'}})),
  // Only the healthy idle scheduler accepts null; ordinary Inspect still
  // requires a complete validated SessionSnapshot.
  inspectIdle: async () => {
    const value=await invokeRuntime<unknown>('lazy_tester_session', {request:{operation:'Inspect'},passiveIdle:true});
    return value===null ? null : parseSession(value);
  },
  launch: async (slot: ProfileSlot) => parseSession(await invokeRuntime<unknown>('lazy_tester_session', {request: {operation: 'Launch', slot}})),
  closeOne: async (slot: ProfileSlot) => {
    if (!profileSlots.includes(slot)) throw new TypeError('Unknown Minecraft profile');
    return parseSelectedClose(await invokeRuntime<unknown>('lazy_tester_close_one', {slot}));
  },
  storage: async () => parseSession(await invokeRuntime<unknown>('lazy_tester_session', {request: {operation: 'InspectStorage'}})),
  selectiveInventory: async (slot: ProfileSlot, category: SelectiveCategory): Promise<SelectiveInventory> => {
    const snapshot=parseSession(await invokeRuntime<unknown>('lazy_tester_session',{request:{operation:'InspectSelectiveInventory',slot,category}}));
    const inventory=snapshot.selectiveInventory;
    if(!inventory||inventory.slot!==slot||inventory.category!==category)throw new TypeError('Inventory receipt does not match request');
    return inventory;
  },
  reviewSelectiveInventory: async (slot: ProfileSlot, selections: SelectiveSelection[]): Promise<SelectiveReviewReceipt> => {
    const snapshot=parseSession(await invokeRuntime<unknown>('lazy_tester_session',{request:{operation:'ReviewSelectiveInventory',slot,selections}}));
    const review=snapshot.selectiveReview;
    if(!review||review.slot!==slot||review.items.length!==selections.length||
      !review.items.every((item,i)=>item.identity===selections[i].identity&&item.category===selections[i].category&&item.root===selections[i].root&&item.name===selections[i].name))
      throw new TypeError('Review receipt does not match request');
    return review;
  },
  confirmSelectedPacks: async (slot: ProfileSlot, receipt: SelectiveReviewReceipt) => {
    if (!profileSlots.includes(slot)||receipt.slot!==slot||receipt.warnings.length||!receipt.confirmationToken||!/^[0-9a-f]{64}$/.test(receipt.confirmationToken)||!receipt.items.length||receipt.items.some(item=>item.category==='Worlds')) throw new TypeError('Invalid pack cleanup confirmation');
    return parseSession(await invokeRuntime<unknown>('lazy_tester_session',{request:{operation:'ConfirmSelectedPacks',slot,selections:receipt.items,confirmationToken:receipt.confirmationToken}}));
  },
  confirmSelectedWorlds: async (slot: ProfileSlot, receipt: SelectiveReviewReceipt) => {
    if (!profileSlots.includes(slot)||receipt.slot!==slot||receipt.warnings.length||!receipt.confirmationToken||!/^[0-9a-f]{64}$/.test(receipt.confirmationToken)||!receipt.items.length||receipt.items.some(item=>item.category!=='Worlds')) throw new TypeError('Invalid world cleanup confirmation');
    return parseSession(await invokeRuntime<unknown>('lazy_tester_session',{request:{operation:'ConfirmSelectedWorlds',slot,selections:receipt.items,confirmationToken:receipt.confirmationToken}}));
  },
  openFolder: async (slot: ProfileSlot) => parseSession(await invokeRuntime<unknown>('lazy_tester_session', {request: {operation: 'OpenProfileFolder', slot}})),
  clearDownloadCache: async (slot: ProfileSlot) => {
    if(!profileSlots.includes(slot))throw new TypeError('Unknown Minecraft profile');
    const snapshot=parseSession(await invokeRuntime<unknown>('lazy_tester_session',{request:{operation:'ClearProfileDownloadCache',slot}}));
    if(!snapshot.storage)throw new TypeError('Download cache result unavailable');
    return snapshot;
  },
  clearCache: async (slot: ProfileSlot) => parseSession(await invokeRuntime<unknown>('lazy_tester_session', {request: {operation: 'ClearProfileCache', slot}})),
  recoverSettings: async (slot: ProfileSlot, action:'Reset'|'Restore') => {
    if(!profileSlots.includes(slot)||!['Reset','Restore'].includes(action))throw new TypeError('Invalid settings recovery request');
    return parseSession(await invokeRuntime<unknown>('lazy_tester_session', {request:{operation:action==='Reset'?'ResetGameSettings':'RestoreGameSettings',slot}}));
  },
  clearData: async (slot:ProfileSlot,confirmationToken:string,confirmationName:string) => {
    if(!profileSlots.includes(slot)||!/^[0-9a-f]{64}$/.test(confirmationToken)||confirmationName!==`Minecraft - ${slot}`)throw new TypeError('Confirm the exact launcher name before clearing all data');
    return parseSession(await invokeRuntime<unknown>('lazy_tester_session',{request:{operation:'ClearProfileData',slot,confirmationToken,confirmationName}}));
  },
  present: (slot: ProfileSlot, rect: WorkspaceRectangle, pixelRatio=window.devicePixelRatio) => invokeRuntime<void>('lazy_tester_present', {
    slot, viewport: workspaceViewport(rect, pixelRatio),
  }),
  previews: async(rect:WorkspaceRectangle,tiles:{slot:ProfileSlot;rect:WorkspaceRectangle}[],pixelRatio=window.devicePixelRatio)=>{
    const placements:PreviewPlacement[]=tiles.map(tile=>({slot:tile.slot,viewport:previewViewport(tile.rect,pixelRatio)}));
    return parsePreviewReport(await invokeRuntime<unknown>('lazy_tester_previews',{
      viewport:workspaceViewport(rect,pixelRatio),tiles:placements,
    }),tiles.map(tile=>tile.slot));
  },
  clearPreviews:()=>invokeRuntime<void>('lazy_tester_clear_previews'),
};
