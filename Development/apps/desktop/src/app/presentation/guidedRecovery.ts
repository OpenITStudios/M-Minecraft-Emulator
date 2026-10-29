// Pure UI gate for the existing, manually triggered recovery affordances.
// Never authorizes a Minecraft operation: the native controller still verifies
// process/window identity and owns admission, lifecycle and data.
import type {ProfileSession, ProfileSlot} from '../bridge/lazyTester.js';
import type {SessionHealthAction, SessionHealthIssue} from './sessionHealth.js';

export interface GuidedRecoveryGate {
  readonly blocked: boolean;
  readonly unresolvedRecovery: ReadonlySet<ProfileSlot>;
  readonly separateWindows: ReadonlySet<ProfileSlot>;
  readonly primarySlot: ProfileSlot;
}

export function guidedRecoveryAllowed(
  issue: SessionHealthIssue | undefined,
  profile: ProfileSession | undefined,
  slot: ProfileSlot,
  action: SessionHealthAction,
  gate: GuidedRecoveryGate,
): boolean {
  if (gate.blocked || gate.unresolvedRecovery.has(slot) ||
      !issue || issue.slot !== slot || issue.action !== action ||
      !profile || profile.slot !== slot) return false;
  if (action === 'refresh') return true;
  // A failed DWM tile alone must never enable Launch or claim gameplay-ready.
  // The native Play command independently rechecks the original HWND.
  return issue.kind === 'preview' && action === 'open' &&
    profile.state === 'RUNNING' && profile.process?.responding === true &&
    !gate.separateWindows.has(slot) && gate.primarySlot !== slot;
}
