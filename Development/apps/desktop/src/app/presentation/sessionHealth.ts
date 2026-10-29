// Pure, read-only projection of the already validated native session/preview reports.
// This module does not authorize launch, close, window recovery or data changes.
import type {ProfileSession, ProfileSlot, PreviewStatus} from '../bridge/lazyTester.js';
import {userMessage} from './sessionFeedback.js';

export type SessionHealthKind = 'window' | 'launch' | 'launcher' | 'preview';
export interface SessionHealthIssue {
  readonly slot: ProfileSlot;
  readonly kind: SessionHealthKind;
  readonly title: string;
  readonly guidance: string;
}
export interface SessionHealthObservation {
  readonly profiles: readonly ProfileSession[];
  readonly previewStatuses: readonly PreviewStatus[];
  readonly unresolvedRecovery: ReadonlySet<ProfileSlot>;
  readonly separateWindows: ReadonlySet<ProfileSlot>;
  readonly primarySlot: ProfileSlot;
  readonly observed: boolean;
  readonly previewsVisible: boolean;
}

// One actionable, authored message per profile. Unknown/partial results
// never become a gameplay-ready claim or a new lifecycle decision.
export function projectSessionHealth(observation: SessionHealthObservation): SessionHealthIssue[] {
  if (!observation.observed) return [];
  const failedPreviews = new Set(observation.previewStatuses
    .filter(status => !status.registered && status.reason !== null)
    .map(status => status.slot));
  const issues: SessionHealthIssue[] = [];
  for (const profile of observation.profiles) {
    const slot = profile.slot;
    // Original native restoration has its own persistent, higher-priority
    // warning in Play. Repeating it here would confuse the user.
    if (observation.unresolvedRecovery.has(slot)) continue;
    if (profile.state === 'NOT_RESPONDING') {
      issues.push({
        slot, kind: 'window',
        title: profile.process ? 'Window not responding' : 'Game window unavailable',
        guidance: profile.process
          ? 'Check the Minecraft window, then refresh its status.'
          : 'No usable game window was found. Refresh before another action.',
      });
    } else if (profile.state === 'STARTING' && profile.issue !== null) {
      issues.push({
        slot, kind: 'launch', title: 'Start not confirmed',
        guidance: 'Minecraft may still be starting. Refresh before retrying.',
      });
    } else if (profile.state === 'ATTENTION' || profile.issue !== null) {
      issues.push({
        slot, kind: 'launcher', title: 'Launcher needs attention',
        guidance: userMessage('profile'),
      });
    } else if (profile.state === 'RUNNING' &&
      observation.previewsVisible && slot !== observation.primarySlot &&
      !observation.separateWindows.has(slot) && failedPreviews.has(slot)) {
      issues.push({
        slot, kind: 'preview', title: 'Preview unavailable',
        guidance: 'The game may still be running. Select it to open the game window, or use Refresh.',
      });
    }
  }
  return issues;
}
