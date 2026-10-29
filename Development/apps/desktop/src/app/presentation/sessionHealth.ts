// Pure, read-only projection of the already validated native session/preview reports.
// This module does not authorize launch, close, window recovery or data changes.
import type {ProfileSession, ProfileSlot, PreviewStatus} from '../bridge/lazyTester.js';
import {userMessage} from './sessionFeedback.js';

export type SessionHealthKind = 'window' | 'launch' | 'launcher' | 'preview';
export type SessionHealthAction = 'refresh' | 'open';
export interface SessionHealthIssue {
  readonly slot: ProfileSlot;
  readonly kind: SessionHealthKind;
  readonly title: string;
  readonly guidance: string;
  // UI suggestion only; native admission is never delegated to this projection.
  readonly action: SessionHealthAction;
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
          ? 'The game may still be running. Check its window before another action.'
          : 'Minecraft may still be running. Do not launch it again yet.',
        action: 'refresh',
      });
    } else if (profile.state === 'STARTING' && profile.issue !== null) {
      issues.push({
        slot, kind: 'launch', title: 'Start not confirmed',
        guidance: 'Minecraft may still be starting. Do not launch it again yet.',
        action: 'refresh',
      });
    } else if (profile.state === 'ATTENTION' || profile.issue !== null) {
      issues.push({
        slot, kind: 'launcher', title: 'Launcher needs attention',
        guidance: userMessage('profile'),
        action: 'refresh',
      });
    } else if (profile.state === 'RUNNING' &&
      observation.previewsVisible && slot !== observation.primarySlot &&
      !observation.separateWindows.has(slot) && failedPreviews.has(slot)) {
      issues.push({
        slot, kind: 'preview', title: 'Preview unavailable',
        guidance: 'The preview failed, but the game window is still responding.',
        action: 'open',
      });
    }
  }
  return issues;
}

/**
 * Concise launcher-card hints, not a second launch gate. Connection failures
 * have one common notice in Play; untrusted native issue text is never shown.
 */
export function launcherCardGuidance(profile: ProfileSession, observed: boolean): string | null {
  if (!observed) return null;
  switch (profile.state) {
    case 'STARTING': return 'Minecraft is starting. Wait for its window before trying again.';
    case 'NOT_RESPONDING': return 'Minecraft may still be running. Check its window, then choose Check status.';
    case 'ATTENTION': return 'Review this launcher with Check status before trying Launch again.';
    case 'STOPPED': return profile.process !== null || profile.issue !== null
      ? 'The previous session is not fully verified. Refresh before trying again.'
      : null;
    default: return null;
  }
}
