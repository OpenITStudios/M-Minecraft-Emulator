// UI-only messages. Never display native error strings or use this projection
// to authorize Launch, Close, Restore or storage writes.
import type {NativeBackendSupport, ProfileSlot} from '../bridge/lazyTester.js';

const messages = {
  general: 'The action could not be confirmed. Check the latest session status before retrying.',
  backend: 'Windows game support could not be checked. Try again.',
  binding: 'Saved launcher data could not be verified. No game was launched.',
  connection: 'Minecraft session status is unavailable. Refresh to check your games.',
  launch: 'Launch could not be confirmed. Refresh this launcher before trying again.',
  selection: 'This Minecraft window could not be shown. Check the session before retrying.',
  workspace: 'The game view could not be updated. Your Minecraft sessions were not asked to close.',
  location: 'The launcher data location could not be confirmed. Review Data Storage before retrying.',
  storage: 'Storage information is unavailable. Refresh to check it again.',
  settings: 'The settings change could not be confirmed. Refresh before retrying.',
  performance: 'Frame limits could not be confirmed. Check Settings before retrying.',
  audio: 'Background audio could not be confirmed. Check Settings before retrying.',
  close: 'Minecraft did not confirm a normal exit. Check the game before retrying.',
  recovery: 'Window restoration was not confirmed. Check the affected Minecraft before retrying.',
  closeAll: 'Lazy Tester could not finish closing all games. Check the remaining sessions.',
  preview: 'Preview is unavailable. Select the Minecraft launcher to view its game window.',
  profile: 'This launcher needs attention. Refresh to check its status.',
  pack: 'Resource pack information is incomplete. No packs were changed.',
  worldPack: 'Saved world pack information is incomplete. No packs were changed.',
  cache: 'Cache cleanup could not be confirmed. Refresh Data Storage before retrying.',
  clearData: 'Data cleanup could not be confirmed. Review Data Storage before retrying.',
  gameSettings: 'Game settings could not be inspected. Review this launcher before retrying.',
  folder: 'The launcher data folder could not be opened. Review Data Storage.',
  event: 'The latest session update is unavailable. Refresh to check your games.',
} as const;

export type UserMessageKey = keyof typeof messages;
export const userMessage = (key: UserMessageKey): string => messages[key];

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
