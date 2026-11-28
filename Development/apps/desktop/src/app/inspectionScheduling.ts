// UI-only scheduling policy. It never owns a native process or authorizes
// Launch/Close; those remain under the Rust controller and Tauri admission.
import type {ProfileSession, SettingsReport, AgentActivityReport} from './bridge/lazyTester.js';

// Compare the layout-relevant fields of the validated profile contract.
// Issue text changes feedback, not the native geometry or source identity.
function sameLayoutProfile(current: ProfileSession, incoming: ProfileSession): boolean {
  if (current.slot !== incoming.slot || current.state !== incoming.state) return false;
  const a = current.process, b = incoming.process;
  if (a === null || b === null) return a === b;
  return a.pid === b.pid && a.createdFileTime === b.createdFileTime &&
    a.sessionId === b.sessionId && a.windowHandle === b.windowHandle &&
    a.responding === b.responding;
}

export function sameProfileLayout(
  current: readonly ProfileSession[], incoming: readonly ProfileSession[],
): boolean {
  return current.length === incoming.length && current.every((profile, index) =>
    sameLayoutProfile(profile, incoming[index]!));
}

// Full observation additionally compares issue text. This only suppresses
// redundant UI assignments, never native reads.
export function sameProfileObservation(
  current: readonly ProfileSession[], incoming: readonly ProfileSession[],
): boolean {
  return current.length === incoming.length && current.every((profile, index) =>
    profile.issue === incoming[index]!.issue && sameLayoutProfile(profile, incoming[index]!));
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
  if (a === b) return true;
  if (a === null || b === null) return false;
  // Compare the complete validated report without allocating JSON strings.
  // Property insertion order is irrelevant; item order and freshness are not.
  return a.schema === b.schema && a.source === b.source &&
    a.snapshotAt === b.snapshotAt && a.displayHint === b.displayHint &&
    a.desktopControl === b.desktopControl &&
    a.minecraftInputEffect === b.minecraftInputEffect &&
    a.mapGameplayAcceptance === b.mapGameplayAcceptance && a.stale === b.stale &&
    a.items.length === b.items.length && a.items.every((item, index) => {
      const next = b.items[index]!;
      const x = item.probeCounts, y = next.probeCounts;
      return item.kind === next.kind && item.slot === next.slot &&
        item.status === next.status && item.importance === next.importance &&
        item.evidence === next.evidence && item.reportedOutcome === next.reportedOutcome &&
        (x === y || (x !== undefined && y !== undefined &&
          x.expected === y.expected && x.pass === y.pass && x.fail === y.fail &&
          x.info === y.info && x.look === y.look));
    });
}

// Only healthy live Play uses this hint. Routine local observation is not
// a lifecycle warning; attention, stale and in-flight input retain the fast
// cadence. Idle still requires no report so its passive read is unchanged.
export function agentNeedsFastInspection(report:AgentActivityReport|null):boolean {
  return report !== null && (report.stale || report.displayHint === 'ATTENTION' ||
    report.items.some(item=>item.importance === 'ATTENTION' ||
      item.status === 'EXPERIMENTAL_INPUT_REQUEST_IN_FLIGHT'));
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
  readonly profiles: readonly ProfileSession[];
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
    sameProfileLayout(issued.profiles, current.profiles) &&
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

// Schedule the actual deadline. Only a settled Play observation with
// current selected identity and background preview coverage may slow live
// inspection. Neither coverage nor an event certifies native liveness.
export function periodicInspectionInterval(
  page: 'workspace' | 'storage' | 'settings',
  runningCount: number,
  available: boolean,
  workspaceSettledIdle = false,
  workspaceSettledLive = false,
): number {
  if (!available) return 2000;
  if (page === 'workspace' && runningCount > 0 && workspaceSettledLive) return 5000;
  return (page === 'workspace' && !workspaceSettledIdle) ||
    runningCount > 0 ? 2000 : 10_000;
}

// The same idle hint can skip an unchanged Settings/Storage observation.
// Periodic full settings receipts still discover external edits and errors.
export function passiveIdleAllowed(
  page: 'workspace' | 'settings' | 'storage',
  available: boolean,
  settledIdle: boolean,
  hinted: boolean,
  settings: SettingsReport | null,
  settingsAgeMs: number,
): boolean {
  if (!available || !settledIdle || hinted) return false;
  if (page === 'workspace') return true;
  return settings !== null && settings.canApplyPreset &&
    settings.backgroundAudio !== undefined && !settings.audioIssue &&
    settings.storagePreferences !== undefined && settingsAgeMs >= 0 && settingsAgeMs < 60_000;
}

// A hint accelerates the next read, never resets its last-start clock or
// bypasses the existing admission lane. Bursts coalesce at the 2s floor.
export function sessionInspectionDeadline(lastStartedAt: number, interval: number, hinted: boolean): number {
  return lastStartedAt + (hinted ? Math.min(interval, 2000) : interval);
}

// A blocked lane has no deadline. A recording never accelerates session
// inspection; the single UI timer wakes for the earlier independent read.
export function nextBackgroundDeadline(session: number | null, recording: number | null): number | null {
  if (session === null) return recording;
  if (recording === null) return session;
  return Math.min(session, recording);
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
