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
