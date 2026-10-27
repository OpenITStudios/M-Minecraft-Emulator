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
