// UI-only scheduling policy. It never owns a native process or authorizes
// Launch/Close; those remain under the Rust controller and Tauri admission.
import type {ProfileSession, SettingsReport, AgentActivityReport} from './bridge/lazyTester.js';

// Compare every field of the validated profile contract, including issue
// text. This only suppresses redundant UI assignments, never native reads.
export function sameProfileObservation(
  current: readonly ProfileSession[], incoming: readonly ProfileSession[],
): boolean {
  return current.length === incoming.length && current.every((profile, index) => {
    const next = incoming[index]!;
    if (profile.slot !== next.slot || profile.state !== next.state ||
        profile.issue !== next.issue) return false;
    const a = profile.process, b = next.process;
    if (a === null || b === null) return a === b;
    return a.pid === b.pid && a.createdFileTime === b.createdFileTime &&
      a.sessionId === b.sessionId && a.windowHandle === b.windowHandle &&
      a.responding === b.responding;
  });
}

// Only validated wire reports reach these comparisons. Optional fields and
// ordered preset/item arrays remain significant; freshness is never inferred.
export function sameSettingsObservation(a: SettingsReport | null, b: SettingsReport): boolean {
  return a !== null && a.backgroundAudio === b.backgroundAudio &&
    a.audioIssue === b.audioIssue &&
    a.storagePreferences?.autoClearWebCache === b.storagePreferences?.autoClearWebCache &&
    a.storagePreferences?.autoClearDownloadCache === b.storagePreferences?.autoClearDownloadCache &&
    a.storagePreferences?.autoClearOldLogs === b.storagePreferences?.autoClearOldLogs &&
    a.activeFps === b.activeFps &&
    a.backgroundFps === b.backgroundFps && a.renderDistanceChunks === b.renderDistanceChunks &&
    a.canApplyPreset === b.canApplyPreset && a.changedProfiles === b.changedProfiles &&
    (a.graphicsPresets === b.graphicsPresets ||
      (a.graphicsPresets !== undefined && b.graphicsPresets !== undefined &&
        a.graphicsPresets.length === b.graphicsPresets.length &&
        a.graphicsPresets.every((preset, index) => preset === b.graphicsPresets![index])));
}

export function sameAgentObservation(a: AgentActivityReport | null, b: AgentActivityReport | null): boolean {
  if (a === null || b === null) return a === b;
  // Validated, bounded JSON records; compare the entire report, including
  // snapshotAt and stale. Different serialization order may cause an extra
  // update but can never suppress a changed field.
  return JSON.stringify(a) === JSON.stringify(b);
}

export interface CloseReview {
  readonly nativeExitReview: boolean;
  readonly closeAllRunning: boolean;
  readonly selectedReview: boolean;
  readonly selectedCloseRunning: boolean;
}

export function closeReviewActive(state: CloseReview): boolean {
  return state.nativeExitReview || state.closeAllRunning ||
    state.selectedReview || state.selectedCloseRunning;
}

// A read-only response started before a close review is never allowed to
// overwrite current UI state, even if the review was canceled meanwhile.
export function observationStillCurrent(
  issued: number, current: number, state: CloseReview,
): boolean {
  return issued === current && !closeReviewActive(state);
}

// A PreviewReport is only an observation about one particular process set,
// selected primary, and manager viewport. Never let a late native report
// promote an obsolete DWM thumbnail after review, minimize or geometry change.
export interface PreviewLayoutIdentity {
  readonly observationEpoch: number;
  readonly processIdentity: string;
  readonly primarySlot: number;
}
export interface PreviewLayoutObservation extends PreviewLayoutIdentity {
  readonly dirty: boolean;
  readonly managerVisible: boolean;
  readonly workspaceAvailable: boolean;
}
export function previewLayoutStillCurrent(
  issued: PreviewLayoutIdentity,
  current: PreviewLayoutObservation,
  review: CloseReview,
): boolean {
  return observationStillCurrent(issued.observationEpoch, current.observationEpoch, review) &&
    issued.processIdentity === current.processIdentity &&
    issued.primarySlot === current.primarySlot &&
    !current.dirty && current.managerVisible && current.workspaceAvailable;
}

// A successful native workspace release is a completed side effect, even if
// a newer close-review epoch has invalidated the Refresh response. Retire only
// the old selected UI binding, never a different slot/process that could have
// been selected in the meantime. This is NOT native window ownership.
export interface SelectedWindowBinding {
  readonly slot: number | null;
  readonly identity: string;
}
export function releasedSelectionStillCurrent(
  released: SelectedWindowBinding,
  current: SelectedWindowBinding,
): boolean {
  return released.slot !== null && released.slot === current.slot &&
    released.identity === current.identity;
}

// Keep the 2s observer for live sessions, recovery and unresolved workspace
// state. Healthy idle Play uses the same bounded 10s cadence as idle
// Settings/Storage. Explicit actions and native mutation admission are unchanged.
export function periodicInspectionDue(
  page: 'workspace' | 'storage' | 'settings',
  runningCount: number,
  available: boolean,
  elapsedMs: number,
  workspaceSettledIdle = false,
): boolean {
  return (page === 'workspace' && !workspaceSettledIdle) ||
    runningCount > 0 || !available || elapsedMs >= 10_000;
}


// The mounted Svelte UI still owns queue payloads and executes all actions.
// This read-only selector reproduces the *first eligible* legacy reactive
// dispatch rule without creating a queue worker or second state authority.
// In particular, explicit refresh has precedence over deferred Launch and
// STARTING does not block legacy selection/layout scheduling.
export interface PendingUiQueueState {
  readonly mounted: boolean;
  readonly nativeBackend: 'WindowsNative' | 'Unsupported' | null;
  readonly closeInProgress: boolean;
  readonly closeReviewPending: boolean;
  readonly selectedReviewGeometryOnly?: boolean;
  readonly initializing: boolean;
  readonly busy: boolean;
  readonly refreshing: boolean;
  readonly resizing: boolean;
  readonly queuedAudio: boolean;
  readonly queuedPerformance: boolean;
  readonly queuedPage: boolean;
  readonly manualRefreshRequested: boolean;
  readonly queuedLaunch: boolean;
  readonly queuedSelection: boolean;
  readonly dirty: boolean;
}

export type PendingUiAction =
  | 'audio' | 'performance' | 'navigation'
  | 'manualRefresh' | 'launch' | 'selection' | 'layout';

export function nextPendingUiAction(state: PendingUiQueueState): PendingUiAction | null {
  // Native layout can otherwise occupy the resize/admission lane before
  // initializeLaunchers performs its first session inspection.
  if (!state.mounted || state.nativeBackend !== 'WindowsNative' ||
      state.initializing ||
      state.closeInProgress ||
      state.busy || state.refreshing || state.resizing) return null;
  // A selected close confirmation may move the retained native surface,
  // but must never dispatch inspection, selection or lifecycle actions.
  if (state.closeReviewPending)
    return state.selectedReviewGeometryOnly && state.dirty ? 'layout' : null;

  // Each earlier return establishes that the higher-priority lane is empty.
  if (state.queuedAudio) return 'audio';
  if (state.queuedPerformance) return 'performance';
  if (state.queuedPage) return 'navigation';
  if (state.manualRefreshRequested) return 'manualRefresh';
  if (state.queuedLaunch) return 'launch';
  if (state.queuedSelection) return 'selection';
  if (state.dirty) return 'layout';
  return null;
}
