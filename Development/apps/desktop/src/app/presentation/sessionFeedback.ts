// UI-only messages. Never display native error strings or use this projection
// to authorize Launch, Close, Restore or storage writes.
import type {NativeBackendSupport, ProfileSlot} from '../bridge/lazyTester.js';

export type FeedbackArea = 'Application' | 'Connection' | 'Launcher' | 'Game' | 'Display' | 'Settings' | 'Storage';

// Operation-scoped user feedback only. A native exception's arbitrary
// details must never select a category, code, or permission to retry.
const feedback = {
  general: {code:'LT-APP-01',area:'Application',message:'The action could not be confirmed. Check the latest session status before retrying.'},
  backend: {code:'LT-APP-02',area:'Application',message:'Windows game support could not be checked. Try again.'},
  binding: {code:'LT-PRO-01',area:'Launcher',message:'Saved launcher data could not be verified. No game was launched.'},
  connection: {code:'LT-CON-01',area:'Connection',message:'Minecraft session status is unavailable. Refresh to check your games.'},
  launch: {code:'LT-GAM-01',area:'Game',message:'Launch could not be confirmed. Refresh this launcher before trying again.'},
  selection: {code:'LT-DIS-01',area:'Display',message:'This Minecraft window could not be shown. Check the session before retrying.'},
  workspace: {code:'LT-DIS-02',area:'Display',message:'The game view could not be updated. Your Minecraft sessions were not asked to close.'},
  location: {code:'LT-STO-01',area:'Storage',message:'The launcher data location could not be confirmed. Review Data Storage before retrying.'},
  storage: {code:'LT-STO-02',area:'Storage',message:'Storage information is unavailable. Refresh to check it again.'},
  settings: {code:'LT-SET-01',area:'Settings',message:'The settings change could not be confirmed. Refresh before retrying.'},
  performance: {code:'LT-SET-02',area:'Settings',message:'Frame limits could not be confirmed. Check Settings before retrying.'},
  audio: {code:'LT-SET-03',area:'Settings',message:'Background audio could not be confirmed. Check Settings before retrying.'},
  close: {code:'LT-GAM-02',area:'Game',message:'Minecraft did not confirm a normal exit. Check the game before retrying.'},
  recovery: {code:'LT-DIS-03',area:'Display',message:'Window restoration was not confirmed. Check the affected Minecraft before retrying.'},
  closeAll: {code:'LT-GAM-03',area:'Game',message:'Lazy Tester could not finish closing all games. Check the remaining sessions.'},
  preview: {code:'LT-DIS-04',area:'Display',message:'Preview is unavailable. Select the Minecraft launcher to view its game window.'},
  profile: {code:'LT-PRO-02',area:'Launcher',message:'This launcher needs attention. Refresh to check its status.'},
  pack: {code:'LT-STO-03',area:'Storage',message:'Resource pack information is incomplete. No packs were changed.'},
  worldPack: {code:'LT-STO-04',area:'Storage',message:'Saved world pack information is incomplete. No packs were changed.'},
  cache: {code:'LT-STO-05',area:'Storage',message:'Cache cleanup could not be confirmed. Refresh Data Storage before retrying.'},
  clearData: {code:'LT-STO-06',area:'Storage',message:'Data cleanup could not be confirmed. Review Data Storage before retrying.'},
  gameSettings: {code:'LT-SET-04',area:'Settings',message:'Game settings could not be inspected. Review this launcher before retrying.'},
  folder: {code:'LT-STO-07',area:'Storage',message:'The launcher data folder could not be opened. Review Data Storage.'},
  event: {code:'LT-APP-03',area:'Application',message:'The latest session update is unavailable. Refresh to check your games.'},
  graphicsBlocked: {code:'LT-SET-05',area:'Settings',message:'Graphics preset could not be applied. Close Minecraft sessions and refresh, then try again.'},
  performanceBlocked: {code:'LT-SET-06',area:'Settings',message:'Frame limits could not be saved. Refresh and try again.'},
} as const satisfies Record<string, {
  readonly code: `LT-${string}`;
  readonly area: FeedbackArea;
  readonly message: string;
}>;

export type UserMessageKey = keyof typeof feedback;
export function userFeedback(key: UserMessageKey) {return feedback[key];}
export function userMessage(key: UserMessageKey): string {return feedback[key].message;}

// A successful idle Inspect does not prove an active controller, sign-in or gameplay.
// Binding is only a verified storage identity, never an authenticated Minecraft account.
export function sessionConnectionLabel(
  initializing: boolean,
  backend: NativeBackendSupport | null,
  bindingConfigured: boolean,
  sessionObserved: boolean,
): string {
  if (initializing) return 'Checking profiles…';
  if (backend === 'Unsupported') return 'Not supported';
  if (backend !== 'WindowsNative') return 'Status unavailable';
  if (!bindingConfigured) return 'Profiles unavailable';
  return sessionObserved ? 'Profiles verified' : 'Session status unavailable';
}

// UI-only unresolved warnings, deliberately not lifecycle/admission state.
// A generic STOPPED observation must never clear a missing native restore receipt.
export function updateRecoverySlots(
  current: ReadonlySet<ProfileSlot>,
  slot: ProfileSlot,
  recovered: boolean,
): Set<ProfileSlot> {
  const next = new Set(current);
  if (recovered) next.delete(slot);
  else next.add(slot);
  return next;
}
export function recoveryLabels(slots: ReadonlySet<ProfileSlot>): string {
  return [...slots].sort((a,b)=>a-b).map(slot=>'Minecraft - '+slot).join(', ');
}
