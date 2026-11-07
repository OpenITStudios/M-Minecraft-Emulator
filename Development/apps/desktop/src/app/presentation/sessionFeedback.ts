// UI-only messages. Never display native error strings or use this projection
// to authorize Launch, Close, Restore or storage writes.
import type {NativeBackendSupport, ProfileSlot} from '../bridge/lazyTester.js';

export type FeedbackArea = 'Application' | 'Connection' | 'Launcher' | 'Game' | 'Display' | 'Settings' | 'Storage';

// Operation-scoped user feedback only. A native exception's arbitrary
// details must never select a category, code, or permission to retry.
const feedback = {
  general: {code:'LT-APP-01',area:'Application',message:'Refresh status before trying this action again.'},
  backend: {code:'LT-APP-02',area:'Application',message:'Try checking Windows support again.'},
  binding: {code:'LT-PRO-01',area:'Launcher',message:'Saved launcher data could not be verified. No game was launched.'},
  connection: {code:'LT-CON-01',area:'Connection',message:'Refresh status before launching another game.'},
  launch: {code:'LT-GAM-01',area:'Game',message:'The game may still be starting. Refresh before launching again.'},
  selection: {code:'LT-DIS-01',area:'Display',message:'Refresh status before selecting this game again.'},
  workspace: {code:'LT-DIS-02',area:'Display',message:'No game was closed. Refresh to check the view and session status.'},
  location: {code:'LT-STO-01',area:'Storage',message:'Check the folder in Data Storage before retrying.'},
  storage: {code:'LT-STO-02',area:'Storage',message:'Refresh to check storage again.'},
  settings: {code:'LT-SET-01',area:'Settings',message:'Refresh before trying the settings change again.'},
  performance: {code:'LT-SET-02',area:'Settings',message:'Check FPS limits in Settings before retrying.'},
  audio: {code:'LT-SET-03',area:'Settings',message:'Check background audio in Settings before retrying.'},
  close: {code:'LT-GAM-02',area:'Game',message:'Check the game window before trying to close it again.'},
  recovery: {code:'LT-DIS-03',area:'Display',message:'The game may still be running. Refresh checks status, not window restoration.'},
  closeAll: {code:'LT-GAM-03',area:'Game',message:'Some games may still be running. Check them in Play.'},
  preview: {code:'LT-DIS-04',area:'Display',message:'The game may still be running. Check its card or Refresh.'},
  profile: {code:'LT-PRO-02',area:'Launcher',message:'Refresh status before trying another launch.'},
  pack: {code:'LT-STO-03',area:'Storage',message:'No packs were changed. Refresh to check again.'},
  worldPack: {code:'LT-STO-04',area:'Storage',message:'No packs were changed. Refresh saved-world pack details.'},
  cache: {code:'LT-STO-05',area:'Storage',message:'Refresh Data Storage before trying again.'},
  clearData: {code:'LT-STO-06',area:'Storage',message:'Cleanup may be unfinished. Check Data Storage before retrying.'},
  gameSettings: {code:'LT-SET-04',area:'Settings',message:'Check this profile in Data Storage before retrying.'},
  folder: {code:'LT-STO-07',area:'Storage',message:'Check the folder location in Data Storage.'},
  event: {code:'LT-APP-03',area:'Application',message:'Refresh to check session status.'},
  graphicsBlocked: {code:'LT-SET-05',area:'Settings',message:'Close all Minecraft sessions, then Refresh before saving graphics.'},
  performanceBlocked: {code:'LT-SET-06',area:'Settings',message:'Refresh before saving FPS limits.'},
  windowControl: {code:'LT-DIS-05',area:'Display',message:'Refresh status before trying the window action again.'},
  windowStatus: {code:'LT-DIS-06',area:'Display',message:'You can keep using Lazy Tester. Refresh to check.'},
  windowClose: {code:'LT-APP-04',area:'Application',message:'Check running games before trying to exit again.'},
  closeIncomplete: {code:'LT-GAM-04',area:'Game',message:'Check the remaining sessions in Play.'},
  closeRejected: {code:'LT-APP-05',area:'Application',message:'Another operation is running. Try again when it finishes.'},
} as const satisfies Record<string, {
  readonly code: `LT-${string}`;
  readonly area: FeedbackArea;
  readonly message: string;
}>;

export type UserMessageKey = keyof typeof feedback;
export function userFeedback(key: UserMessageKey) {return feedback[key];}
export function userMessage(key: UserMessageKey): string {return feedback[key].message;}

/** Stable, authored headings for existing feedback only; never classify a raw
 * native exception, infer success/failure, or authorize another operation.
 */
export function feedbackHeading(key: UserMessageKey): string {
  switch (key) {
    case 'backend': return 'Windows support check unavailable';
    case 'binding': return 'Launcher profiles not verified';
    case 'connection':
    case 'event':
    case 'windowStatus': return 'Session status unavailable';
    case 'launch': return 'Game launch not confirmed';
    case 'selection':
    case 'workspace':
    case 'preview':
    case 'recovery':
    case 'windowControl': return 'Game display needs attention';
    case 'location':
    case 'folder': return 'Data folder needs attention';
    case 'storage':
    case 'pack':
    case 'worldPack': return 'Storage information incomplete';
    case 'cache': return 'Cache cleanup not confirmed';
    case 'clearData': return 'Data cleanup not confirmed';
    case 'settings':
    case 'performance':
    case 'audio':
    case 'gameSettings':
    case 'graphicsBlocked':
    case 'performanceBlocked': return 'Settings need attention';
    case 'close':
    case 'closeAll':
    case 'closeIncomplete':
    case 'closeRejected': return 'Game exit not confirmed';
    case 'windowClose': return 'App exit not confirmed';
    case 'profile': return 'Launcher needs attention';
    default: return 'Action not confirmed';
  }
}

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
  return sessionObserved ? 'Profile data verified' : 'Session status unavailable';
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
