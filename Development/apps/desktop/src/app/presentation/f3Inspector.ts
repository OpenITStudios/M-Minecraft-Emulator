// Read-only F3-style desktop projection of the EXISTING validated native
// session snapshot. This does not read the game, own a process, or claim
// authenticated local-player/world coordinates or measured display FPS.
import type {ProfileSession, ProfileSlot, SettingsReport} from '../bridge/lazyTester.js';

export type F3SourceStatus = 'NO_SELECTION' | 'ACTION_PENDING' | 'NOT_CURRENT' | 'NATIVE_PROCESS_OBSERVED';

export interface F3Observation {
  readonly selectedSlot: ProfileSlot | null;
  readonly profile: ProfileSession | undefined;
  // Derived by the existing Play owner from its verified selection binding,
  // current session observation, window state and retained process birth.
  // UI presentation only; it is not proof of game/player/world identity.
  readonly currentSelection: boolean;
  // An original Play launch/close/switch pending gate, not a game-owner claim.
  readonly actionPending: boolean;
  readonly settings: SettingsReport | null;
}

export interface F3InspectorView {
  readonly sourceStatus: F3SourceStatus;
  readonly clientLabel: string;
  readonly summary: string;
  readonly nativeState: string;
  readonly processId: string;
  readonly windowsSession: string;
  readonly nativeWindow: string;
  readonly configuredFps: string;
  // Explicit unknowns cannot be promoted by a caller-provided payload.
  readonly measuredFps: 'Not exposed to desktop';
  readonly coordinates: 'Not connected';
  readonly rotation: 'Not connected';
  readonly dimension: 'Not connected';
}

const UNKNOWN = '—';
const NATIVE_STATES: Record<ProfileSession['state'], string> = {
  STOPPED: 'Not running',
  STARTING: 'Starting',
  RUNNING: 'Running (native process)',
  NOT_RESPONDING: 'Not responding',
  ATTENTION: 'Needs attention',
};

export function projectF3Inspector(value: F3Observation): F3InspectorView {
  const clientLabel = value.selectedSlot === null
    ? 'No selected client'
    : 'MC' + String(value.selectedSlot).padStart(2, '0');
  const unavailable = (sourceStatus: F3SourceStatus, summary: string): F3InspectorView => ({
    sourceStatus, clientLabel, summary, nativeState: 'Unavailable',
    processId: UNKNOWN, windowsSession: UNKNOWN, nativeWindow: 'Unavailable',
    configuredFps: 'Not available',
    measuredFps: 'Not exposed to desktop',
    coordinates: 'Not connected', rotation: 'Not connected', dimension: 'Not connected',
  });

  if (value.actionPending) {
    return unavailable('ACTION_PENDING', 'Launcher action pending. Native F3 details are withheld until it finishes.');
  }
  if (value.selectedSlot === null) {
    return unavailable('NO_SELECTION', 'Select a Minecraft launcher in Play to inspect its native process.');
  }
  if (!value.currentSelection || value.profile?.slot !== value.selectedSlot) {
    return unavailable('NOT_CURRENT', 'Native selection is not current. Refresh or select the game again.');
  }
  const profile = value.profile;
  const process = profile.process;
  if (!process || !['STARTING', 'RUNNING', 'NOT_RESPONDING'].includes(profile.state) ||
      !Number.isSafeInteger(process.pid) || process.pid <= 0 ||
      !Number.isSafeInteger(process.sessionId) || process.sessionId < 0) {
    return unavailable('NOT_CURRENT', 'No current native process is available for this selection.');
  }

  const configuredFps = value.settings &&
    Number.isInteger(value.settings.activeFps) &&
    Number.isInteger(value.settings.backgroundFps)
      ? 'Last reported targets: active ' + value.settings.activeFps + ' / background ' +
        value.settings.backgroundFps + ' FPS (not measured or confirmed for this game)'
      : 'Not available from current settings';
  return {
    sourceStatus: 'NATIVE_PROCESS_OBSERVED',
    clientLabel,
    summary: 'Last Windows session observation matched this client. Minecraft player, world and gameplay are NOT verified.',
    nativeState: NATIVE_STATES[profile.state],
    processId: String(process.pid),
    windowsSession: String(process.sessionId),
    nativeWindow: process.responding
      ? 'Responding (Windows observation)'
      : 'Not responding (Windows observation)',
    configuredFps,
    measuredFps: 'Not exposed to desktop',
    coordinates: 'Not connected',
    rotation: 'Not connected',
    dimension: 'Not connected',
  };
}
