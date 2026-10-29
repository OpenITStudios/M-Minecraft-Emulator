// UI-only scheduling policy. It never owns a native process or authorizes
// Launch/Close; those remain under the Rust controller and Tauri admission.
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

// The existing 2s observer remains immediate for live games, launcher and
 // workspace views, and connection recovery. Settings/storage with no active
 // game use a bounded 10s read-only cadence; user actions still refresh
 // immediately and Rust rechecks all admissions before mutations.
export function periodicInspectionDue(
  page: 'workspace' | 'launchers' | 'storage' | 'settings',
  runningCount: number,
  available: boolean,
  elapsedMs: number,
): boolean {
  return page === 'workspace' || page === 'launchers' ||
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
  readonly initializing: boolean;
  readonly busy: boolean;
  readonly refreshing: boolean;
  readonly resizing: boolean;
  readonly queuedAudio: boolean;
  readonly queuedPerformance: boolean;
  readonly queuedPreset: boolean;
  readonly queuedPage: boolean;
  readonly manualRefreshRequested: boolean;
  readonly queuedLaunch: boolean;
  readonly queuedSelection: boolean;
  readonly dirty: boolean;
}

export type PendingUiAction =
  | 'audio' | 'performance' | 'preset' | 'navigation'
  | 'manualRefresh' | 'launch' | 'selection' | 'layout';

export function nextPendingUiAction(state: PendingUiQueueState): PendingUiAction | null {
  if (!state.mounted || state.nativeBackend !== 'WindowsNative' ||
      state.closeInProgress || state.closeReviewPending ||
      state.busy || state.refreshing || state.resizing) return null;

  if (!state.initializing) {
    if (state.queuedAudio) return 'audio';
    if (!state.queuedAudio && state.queuedPerformance) return 'performance';
    if (!state.queuedAudio && !state.queuedPerformance && state.queuedPreset) return 'preset';
    if (!state.queuedAudio && !state.queuedPerformance &&
        !state.queuedPreset && state.queuedPage) return 'navigation';
    // The original read-only refresh and Launch lanes do not separately
    // require queuedAudio to be empty; the audio lane already runs first.
    if (!state.queuedPerformance && !state.queuedPreset && !state.queuedPage) {
      if (state.manualRefreshRequested) return 'manualRefresh';
      if (state.queuedLaunch) return 'launch';
    }
  }

  if (!state.queuedAudio && !state.queuedPerformance &&
      !state.queuedPreset && !state.queuedPage) {
    if (state.queuedSelection) return 'selection';
    if (state.dirty) return 'layout';
  }
  return null;
}
