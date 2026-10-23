import { invokeRuntime } from "./invokeRuntime.js";
import {parseObservedProcess,type OwnedProcess} from "./ownedProviderApi.js";

export interface NativeProcessCandidate {
  pid: number;
  windowHandle: string | null;
}

export interface NativeProcessObservation {
  schema: 1;
  evidenceClass: "NAME_MATCH_CANDIDATE_ONLY";
  interactiveSession: boolean;
  candidates: NativeProcessCandidate[];
}

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

const hasOnly = (value: Record<string, unknown>, fields: readonly string[]) =>
  Object.keys(value).length === fields.length &&
  fields.every((field) => Object.prototype.hasOwnProperty.call(value, field));

export function parseNativeObservation(value: unknown): NativeProcessObservation {
  if (!isRecord(value) || !hasOnly(value, [
    "schema", "evidenceClass", "interactiveSession", "candidates",
  ]) ||
    value.schema !== 1 ||
    value.evidenceClass !== "NAME_MATCH_CANDIDATE_ONLY" ||
    typeof value.interactiveSession !== "boolean" ||
    !Array.isArray(value.candidates) ||
    value.candidates.length > 16
  ) {
    throw new TypeError("Untrusted native process observation.");
  }

  const pids = new Set<number>();
  const handles = new Set<bigint>();
  const candidates: NativeProcessCandidate[] = [];

  for (const candidate of value.candidates) {
    if (!isRecord(candidate) || !hasOnly(candidate, ["pid", "windowHandle"])) {
      throw new TypeError("Invalid or duplicate native PID candidate.");
    }
    const pid = candidate.pid;
    if (typeof pid !== "number" || !Number.isSafeInteger(pid) ||
      pid <= 0 || pid > 0xffffffff || pids.has(pid)) {
      throw new TypeError("Invalid or duplicate native PID candidate.");
    }
    pids.add(pid);

    let windowHandle: string | null = null;
    if (candidate.windowHandle !== null) {
      if (typeof candidate.windowHandle !== "string" ||
        !/^0x[0-9a-f]{1,16}$/i.test(candidate.windowHandle) ||
        !/[1-9a-f]/i.test(candidate.windowHandle.slice(2))) {
        throw new TypeError("Invalid native window handle.");
      }
      const key = BigInt(candidate.windowHandle);
      if (handles.has(key)) throw new TypeError("Duplicate native window handle.");
      handles.add(key);
      windowHandle = candidate.windowHandle;
    }
    candidates.push({ pid, windowHandle });
  }

  return {
    schema: 1,
    evidenceClass: "NAME_MATCH_CANDIDATE_ONLY",
    interactiveSession: value.interactiveSession,
    candidates,
  };
}

export interface ProcessIdentityCandidate {
  pid: number;
  // Decimal Windows FILETIME, not a lossy JSON floating-point number.
  createdFileTime: string | null;
  windowHandle: string | null;
  inSelectedBox: boolean | null;
  imageEvidence: "UNKNOWN" | "SIGNED_EDUCATION_IMAGE_CANDIDATE";
}

export interface IsolatedClientObservation {
  schema: 6;
  boxName: "MinecraftClient02";
  stage: "INSPECT";
  boxConfigured: "YES" | "NO" | "UNKNOWN";
  boxSettingsSource: "BOX_SECTION_ENUMERATED" | "UNKNOWN";
  autoDelete: "ON" | "OFF" | "UNKNOWN";
  autoRemove: "ON" | "OFF" | "UNKNOWN";
  credentialHostAccess: "ON" | "OFF" | "UNKNOWN";
  membershipStatus: "OBSERVED" | "UNKNOWN";
  interactiveSession: boolean;
  processEvidenceClass: "NAME_MATCH_CANDIDATE_ONLY";
  candidateProcessCount: number;
  processCandidates: ProcessIdentityCandidate[];
  officialImageOutsideSelectedBoxCount: number;
  directPidQuery: "NOT_REQUESTED" | "UNAVAILABLE" | "QUERIED";
  signedImageOtherBoxCount: number;
  signedImageUnknownBoxCount: number;
  isolatedCandidateProcesses: number;
  nativeVisibleCandidates: number;
  isolatedVisibleCandidates: number;
  interpretation: "CONFIG_AND_PROCESS_CANDIDATES_ONLY_NO_AUTHENTICATION_PROOF";
}

// Pure presentation projections of the single inspected IPC snapshot.
// These never authorize a launch: the backend repeats the full preflight.
export function isSandboxieBoxPrepared(observation: IsolatedClientObservation | undefined): boolean {
  return observation?.interactiveSession === true &&
    observation.boxConfigured === "YES" &&
    observation.boxSettingsSource === "BOX_SECTION_ENUMERATED" &&
    observation.autoDelete === "OFF" &&
    observation.autoRemove === "OFF" &&
    observation.credentialHostAccess === "OFF";
}

export function canRequestClient02(observation: IsolatedClientObservation | undefined): boolean {
  return isSandboxieBoxPrepared(observation) &&
    observation?.membershipStatus === "OBSERVED" &&
    observation.isolatedCandidateProcesses === 0;
}

export function parseIsolatedClientObservation(value: unknown): IsolatedClientObservation {
  const keys = [
    "schema", "boxName", "stage", "boxConfigured", "boxSettingsSource",
    "autoDelete", "autoRemove", "credentialHostAccess", "membershipStatus",
    "interactiveSession", "processEvidenceClass", "candidateProcessCount",
    "processCandidates", "officialImageOutsideSelectedBoxCount", "directPidQuery",
    "signedImageOtherBoxCount", "signedImageUnknownBoxCount", "isolatedCandidateProcesses", "nativeVisibleCandidates", "isolatedVisibleCandidates", "interpretation",
  ];
  const flag = (v: unknown): v is "ON" | "OFF" | "UNKNOWN" =>
    v === "ON" || v === "OFF" || v === "UNKNOWN";
  const bounded = (v: unknown): v is number =>
    typeof v === "number" && Number.isSafeInteger(v) && v >= 0 && v <= 16;
  if (!isRecord(value) || !hasOnly(value, keys) ||
    value.schema !== 6 || value.boxName !== "MinecraftClient02" ||
    value.stage !== "INSPECT" ||
    !["YES", "NO", "UNKNOWN"].includes(value.boxConfigured as string) ||
    !["BOX_SECTION_ENUMERATED", "UNKNOWN"].includes(value.boxSettingsSource as string) ||
    !flag(value.autoDelete) || !flag(value.autoRemove) ||
    !flag(value.credentialHostAccess) ||
    !["OBSERVED", "UNKNOWN"].includes(value.membershipStatus as string) ||
    (value.membershipStatus === "OBSERVED" && value.boxConfigured !== "YES") ||
    (value.boxSettingsSource === "BOX_SECTION_ENUMERATED" && value.boxConfigured !== "YES") ||
    (value.credentialHostAccess !== "UNKNOWN" && value.boxSettingsSource !== "BOX_SECTION_ENUMERATED") ||
    typeof value.interactiveSession !== "boolean" ||
    value.processEvidenceClass !== "NAME_MATCH_CANDIDATE_ONLY" ||
    value.interpretation !== "CONFIG_AND_PROCESS_CANDIDATES_ONLY_NO_AUTHENTICATION_PROOF" ||
    !bounded(value.candidateProcessCount) ||
    !Array.isArray(value.processCandidates) ||
    value.processCandidates.length !== value.candidateProcessCount ||
    !bounded(value.officialImageOutsideSelectedBoxCount) ||
    (value.officialImageOutsideSelectedBoxCount as number) > (value.candidateProcessCount as number) ||
    !["NOT_REQUESTED", "UNAVAILABLE", "QUERIED"].includes(value.directPidQuery as string) ||
    !bounded(value.signedImageOtherBoxCount) ||
    !bounded(value.signedImageUnknownBoxCount) ||
    (value.signedImageOtherBoxCount as number) > (value.officialImageOutsideSelectedBoxCount as number) ||
    (value.signedImageUnknownBoxCount as number) > (value.officialImageOutsideSelectedBoxCount as number) ||
    (value.signedImageOtherBoxCount as number) +
      (value.signedImageUnknownBoxCount as number) !== value.officialImageOutsideSelectedBoxCount ||
    (value.directPidQuery === "NOT_REQUESTED" && value.officialImageOutsideSelectedBoxCount !== 0) ||
    (value.directPidQuery !== "NOT_REQUESTED" && value.officialImageOutsideSelectedBoxCount === 0) ||
    (value.directPidQuery !== "QUERIED" && value.signedImageOtherBoxCount !== 0) ||
    !bounded(value.isolatedCandidateProcesses) ||
    (value.isolatedCandidateProcesses as number) > (value.candidateProcessCount as number) ||
    !bounded(value.nativeVisibleCandidates) ||
    !bounded(value.isolatedVisibleCandidates) ||
    (value.isolatedVisibleCandidates as number) > (value.isolatedCandidateProcesses as number) ||
    (value.nativeVisibleCandidates as number) + (value.isolatedVisibleCandidates as number) > (value.candidateProcessCount as number) ||
    (value.membershipStatus === "UNKNOWN" &&
      (value.nativeVisibleCandidates !== 0 || value.isolatedVisibleCandidates !== 0 ||
       value.isolatedCandidateProcesses !== 0))
  ) {
    throw new TypeError("Untrusted isolated Minecraft process observation.");
  }
  const pids = new Set<number>();
  const windows = new Set<bigint>();
  let boxed = 0, boxedVisible = 0, outsideVisible = 0, officialOutside = 0;
  for (const candidate of value.processCandidates as unknown[]) {
    if (!isRecord(candidate) ||
      !hasOnly(candidate, ["pid", "createdFileTime", "windowHandle", "inSelectedBox", "imageEvidence"]) ||
      typeof candidate.pid !== "number" ||
      !Number.isSafeInteger(candidate.pid) ||
      candidate.pid < 1 || candidate.pid > 0xffffffff ||
      pids.has(candidate.pid)) {
      throw new TypeError("Invalid process identity PID.");
    }
    pids.add(candidate.pid);
    if (candidate.createdFileTime !== null) {
      if (typeof candidate.createdFileTime !== "string" ||
        !/^[1-9][0-9]{0,19}$/.test(candidate.createdFileTime) ||
        BigInt(candidate.createdFileTime) > 0xffffffffffffffffn) {
        throw new TypeError("Invalid process creation FILETIME.");
      }
    }
    if (candidate.inSelectedBox !== null &&
      typeof candidate.inSelectedBox !== "boolean") {
      throw new TypeError("Invalid process membership.");
    }
    if (value.membershipStatus === "UNKNOWN" && candidate.inSelectedBox !== null) {
      throw new TypeError("Unknown membership cannot attribute process identity.");
    }
    if (candidate.inSelectedBox === true) boxed++;
    if (candidate.imageEvidence === "SIGNED_EDUCATION_IMAGE_CANDIDATE") {
      if (candidate.windowHandle === null ||
          (candidate.inSelectedBox !== true && candidate.inSelectedBox !== false) ||
          (candidate.inSelectedBox === true && candidate.createdFileTime === null)) {
        throw new TypeError("Signed Education image requires a visible attributed process.");
      }
      if (candidate.inSelectedBox === false) officialOutside++;
    } else if (candidate.imageEvidence !== "UNKNOWN") {
      throw new TypeError("Unrecognized executable image evidence.");
    }
    if (candidate.windowHandle !== null) {
      if (typeof candidate.windowHandle !== "string" ||
        !/^0x[0-9a-f]{1,16}$/i.test(candidate.windowHandle)) {
        throw new TypeError("Invalid process identity HWND.");
      }
      const hwnd = BigInt(candidate.windowHandle);
      if (hwnd === 0n || windows.has(hwnd)) {
        throw new TypeError("Duplicate or zero process identity HWND.");
      }
      windows.add(hwnd);
      if (candidate.inSelectedBox === true) boxedVisible++;
      if (candidate.inSelectedBox === false) outsideVisible++;
    }
  }
  if (boxed !== value.isolatedCandidateProcesses ||
    officialOutside !== value.officialImageOutsideSelectedBoxCount ||
    boxedVisible !== value.isolatedVisibleCandidates ||
    outsideVisible !== value.nativeVisibleCandidates) {
    throw new TypeError("Process candidates contradict snapshot counters.");
  }
  return value as unknown as IsolatedClientObservation;
}


export interface OwnedCandidateInspection {
  schema: 1;
  state: "CANDIDATE_INTEGRITY_VERIFIED_NOT_INSTALLABLE";
  upstreamCommit:
    | "abd4c240031cfb46666daa74ca47ed4e48b561e4"
    | "cdba7096240b93d79c5373aa81a75a1c5c71b883";
  verifiedArtifactCount: number;
  requiredArtifactCount: number;
  providerSelected: false;
  driverLoadVerified: false;
  gameplayVerified: false;
  installPermitted: false;
}

// Candidate checks cannot be interpreted as a ready, installed engine.
// Same Rust command inspects both legacy v1.18.5 and current owned v1.18.6.
// Exact version/required-file pairing matters; a checksum match alone never
// proves a driver was signed, installed, selected, or safe to use.
export function parseOwnedCandidateInspection(value: unknown): OwnedCandidateInspection {
  if (!isRecord(value) || !hasOnly(value, [
    "schema", "state", "upstreamCommit", "verifiedArtifactCount",
    "requiredArtifactCount", "providerSelected", "driverLoadVerified",
    "gameplayVerified", "installPermitted",
  ]) || value.schema !== 1 ||
    value.state !== "CANDIDATE_INTEGRITY_VERIFIED_NOT_INSTALLABLE" ||
    typeof value.verifiedArtifactCount !== "number" ||
    !Number.isSafeInteger(value.verifiedArtifactCount) ||
    (value.verifiedArtifactCount as number) > 48 ||
    value.providerSelected !== false || value.driverLoadVerified !== false ||
    value.gameplayVerified !== false || value.installPermitted !== false) {
    throw new TypeError("Untrusted owned isolation candidate observation.");
  }
  const required = value.upstreamCommit === "cdba7096240b93d79c5373aa81a75a1c5c71b883"
    ? 25
    : value.upstreamCommit === "abd4c240031cfb46666daa74ca47ed4e48b561e4"
      ? 24
      : null;
  if (required === null || value.requiredArtifactCount !== required ||
    (value.verifiedArtifactCount as number) < required) {
    throw new TypeError("Untrusted owned isolation candidate version or required files.");
  }
  return value as unknown as OwnedCandidateInspection;
}

export interface OwnedHostInspection {
  schema: 1;
  interactiveSession: boolean;
  state: "BLOCKED_SERVICE_NAMESPACE_IN_USE" | "BLOCKED_SERVICE_PROBE_UNKNOWN" |
    "BLOCKED_NON_INTERACTIVE_SESSION" | "BLOCKED_UNVERIFIED_OWNED_DRIVER";
  serviceRegistration: "PRESENT" | "ABSENT" | "UNKNOWN";
  driverRegistration: "PRESENT" | "ABSENT" | "UNKNOWN";
  serviceNamespaceAvailable: boolean;
  providerSelected: false;
  trustedDriverLoaded: false;
  gameplayVerified: false;
  activationAllowed: false;
}

export function parseOwnedHostInspection(value: unknown): OwnedHostInspection {
  if (!isRecord(value) || !hasOnly(value, [
    "schema", "state", "interactiveSession", "serviceRegistration", "driverRegistration",
    "serviceNamespaceAvailable", "providerSelected", "trustedDriverLoaded",
    "gameplayVerified", "activationAllowed",
  ]) || value.schema !== 1 || typeof value.interactiveSession !== "boolean" ||
    !["PRESENT", "ABSENT", "UNKNOWN"].includes(value.serviceRegistration as string) ||
    !["PRESENT", "ABSENT", "UNKNOWN"].includes(value.driverRegistration as string) ||
    typeof value.serviceNamespaceAvailable !== "boolean" ||
    value.providerSelected !== false || value.trustedDriverLoaded !== false ||
    value.gameplayVerified !== false || value.activationAllowed !== false
  ) {
    throw new TypeError("Untrusted owned engine host collision evidence.");
  }
  const unknown = value.serviceRegistration === "UNKNOWN" || value.driverRegistration === "UNKNOWN";
  const empty = value.serviceRegistration === "ABSENT" && value.driverRegistration === "ABSENT";
  const status = value.interactiveSession === false
    ? "BLOCKED_NON_INTERACTIVE_SESSION"
    : (value.serviceRegistration === "PRESENT" || value.driverRegistration === "PRESENT")
      ? "BLOCKED_SERVICE_NAMESPACE_IN_USE"
      : unknown ? "BLOCKED_SERVICE_PROBE_UNKNOWN" : "BLOCKED_UNVERIFIED_OWNED_DRIVER";
  if (value.serviceNamespaceAvailable !== empty || value.state !== status) {
    throw new TypeError("Contradictory owned engine host service evidence.");
  }
  return value as unknown as OwnedHostInspection;
}

export interface VirtualBoxInventory {
  schema: 1;
  source: "EXISTING_OFFICIAL_SANDBOXIE_PILOT";
  evidence: "SECTION_NAME_CANDIDATES";
  boxes: string[];
  ownedProviderSelected: false;
  gameplayVerified: false;
}

// Configuration section-name candidates only. These are NOT authenticated boxes,
// MIVUBI-owned virtuals or evidence of gameplay readiness.
export function parseVirtualBoxInventory(value: unknown): VirtualBoxInventory {
  if (!isRecord(value) || !hasOnly(value, [
    "schema", "source", "evidence", "boxes", "ownedProviderSelected", "gameplayVerified",
  ]) || value.schema !== 1 ||
    value.source !== "EXISTING_OFFICIAL_SANDBOXIE_PILOT" ||
    value.evidence !== "SECTION_NAME_CANDIDATES" ||
    value.ownedProviderSelected !== false || value.gameplayVerified !== false ||
    !Array.isArray(value.boxes) || value.boxes.length > 128) {
    throw new TypeError("Untrusted existing virtual inventory.");
  }
  const names = new Set<string>();
  for (const item of value.boxes) {
    if (typeof item !== "string" || !/^[A-Za-z0-9_]{1,32}$/.test(item) ||
      names.has(item.toLowerCase())) {
      throw new TypeError("Invalid or duplicate existing virtual name.");
    }
    names.add(item.toLowerCase());
  }
  return value as unknown as VirtualBoxInventory;
}

/** The Rust catalog is the only authority for all four identities/routes.
 * This transport validator must not infer that a mounted route means a
 * verified Minecraft window or an installed MIVUBI driver.
 */
export type InstanceSlotId = "01" | "02" | "03" | "04";
export interface InstanceSlotDescriptor {
  id: InstanceSlotId;
  kind: "NATIVE" | "ISOLATED";
  route: "OFFICIAL_WINDOWS" | "OFFICIAL_SANDBOXIE" | "OWNED_V1186";
  availability: "PILOT_ROUTE_ONLY" | "NOT_AVAILABLE";
}
export function parseInstanceSlots(value: unknown): InstanceSlotDescriptor[] {
  if (!Array.isArray(value) || value.length !== 4) {
    throw new TypeError("Incomplete four-slot instance catalog.");
  }
  const ids: InstanceSlotId[] = ["01", "02", "03", "04"];
  const routes: InstanceSlotDescriptor["route"][] = [
    "OFFICIAL_WINDOWS", "OFFICIAL_SANDBOXIE", "OWNED_V1186", "OWNED_V1186",
  ];
  for (const [index, item] of value.entries()) {
    const expected = ids[index];
    if (!isRecord(item) || !hasOnly(item, ["id", "kind", "route", "availability"]) ||
        item.id !== expected || item.route !== routes[index] ||
        item.kind !== (index === 0 ? "NATIVE" : "ISOLATED") ||
        item.availability !== (index < 2 ? "PILOT_ROUTE_ONLY" : "NOT_AVAILABLE")) {
      throw new TypeError("Untrusted or unsupported four-slot catalog.");
    }
  }
  return value as InstanceSlotDescriptor[];
}

/** Operator-requested candidate-only discovery. No per-slot login, process
 * ownership for Host 01, or lifecycle readiness is claimed.
 */
export interface SlotDiscovery {
  id: InstanceSlotId;
  evidence: "NAME_MATCH_UNATTRIBUTED" | "BOX_PROCESS_CANDIDATES_ONLY" |
    "UNKNOWN" | "PROVIDER_NOT_AVAILABLE";
  candidateCount: number | null;
}
export interface InstanceOverview {
  schema: 1;
  slots: SlotDiscovery[];
}
export function parseInstanceOverview(value: unknown): InstanceOverview {
  if (!isRecord(value) || !hasOnly(value, ["schema", "slots"]) ||
      value.schema !== 1 || !Array.isArray(value.slots) || value.slots.length !== 4) {
    throw new TypeError("Incomplete four-slot process observation.");
  }
  const ids: InstanceSlotId[] = ["01", "02", "03", "04"];
  for (const [index, item] of value.slots.entries()) {
    if (!isRecord(item) || !hasOnly(item, ["id", "evidence", "candidateCount"]) ||
        item.id !== ids[index]) {
      throw new TypeError("Unexpected process discovery identity.");
    }
    const evidence = item.evidence;
    const supported = index === 0
      ? ["NAME_MATCH_UNATTRIBUTED", "UNKNOWN"]
      : index === 1 ? ["BOX_PROCESS_CANDIDATES_ONLY", "UNKNOWN"]
      : ["PROVIDER_NOT_AVAILABLE"];
    if (!supported.includes(evidence as string) ||
        (evidence === "UNKNOWN" || evidence === "PROVIDER_NOT_AVAILABLE"
          ? item.candidateCount !== null
          : !Number.isInteger(item.candidateCount) ||
            (item.candidateCount as number) < 0 || (item.candidateCount as number) > 16)) {
      throw new TypeError("Invalid or escalated process discovery evidence.");
    }
  }
  return value as unknown as InstanceOverview;
}

/** One on-demand focus snapshot, not a gameplay/FPS claim.
 * Only verified Client 02 windows are attributable at this stage.
 */
export interface InstanceFocusEvidence {
  id: InstanceSlotId;
  evidence: "UNATTRIBUTED" | "FOREGROUND" | "NOT_FOREGROUND" |
    "UNKNOWN" | "PROVIDER_NOT_AVAILABLE";
}
export interface InstanceFocusObservation {
  schema: 2;
  mode: "OBSERVE_ONLY";
  slots: InstanceFocusEvidence[];
}
export function parseInstanceFocus(value: unknown): InstanceFocusObservation {
  if (!isRecord(value) || !hasOnly(value, ["schema", "mode", "slots"]) ||
      value.schema !== 2 || value.mode !== "OBSERVE_ONLY" ||
      !Array.isArray(value.slots) || value.slots.length !== 4) {
    throw new TypeError("Malformed four-slot focus observation.");
  }
  const ids: InstanceSlotId[] = ["01", "02", "03", "04"];
  for (const [index, slot] of value.slots.entries()) {
    const allowed = index === 0 ? ["UNATTRIBUTED"]
      : index === 1 ? ["FOREGROUND", "NOT_FOREGROUND", "UNKNOWN"]
      : ["PROVIDER_NOT_AVAILABLE"];
    if (!isRecord(slot) || !hasOnly(slot, ["id", "evidence"]) ||
        slot.id !== ids[index] || !allowed.includes(slot.evidence as string)) {
      throw new TypeError("Unsupported focus attribution or elevated slot evidence.");
    }
  }
  return value as unknown as InstanceFocusObservation;
}

/** Aggregated Windows focus-event counter, never a per-game FPS signal. */
export interface ForegroundEventObserverStatus {
  schema: 1;
  mode: "OBSERVE_ONLY";
  enabled: boolean;
  eventCount: number;
}
export function parseForegroundEventObserverStatus(value: unknown): ForegroundEventObserverStatus {
  if (!isRecord(value) || !hasOnly(value, ["schema", "mode", "enabled", "eventCount"]) ||
      value.schema !== 1 || value.mode !== "OBSERVE_ONLY" ||
      typeof value.enabled !== "boolean" ||
      !Number.isSafeInteger(value.eventCount) || (value.eventCount as number) < 0) {
    throw new TypeError("Untrusted focus-event observer status.");
  }
  return value as unknown as ForegroundEventObserverStatus;
}


/** Registered game executable path only; never account or isolation evidence. */
export interface NativeWorkspaceObservation {
  schema:1;
  evidenceClass:"REGISTERED_IMAGE_PATH_CANDIDATE_ONLY";
  processes:OwnedProcess[];
}
export function parseNativeWorkspaceObservation(value:unknown):NativeWorkspaceObservation {
  if(!isRecord(value)||!hasOnly(value,["schema","evidenceClass","processes"])||
    value.schema!==1||value.evidenceClass!=="REGISTERED_IMAGE_PATH_CANDIDATE_ONLY"||
    !Array.isArray(value.processes)||value.processes.length>4)
    throw new TypeError("Untrusted native workspace observation.");
  const pids=new Set<number>(),windows=new Set<bigint>();
  for(const p of value.processes) {
    const process=parseObservedProcess(p);
    const window=BigInt(process.windowHandle);
    if(process.sessionId===0||pids.has(process.pid)||
       (window!==0n&&windows.has(window)))
      throw new TypeError("Duplicated or noninteractive native game identity.");
    pids.add(process.pid);
    if(window!==0n)windows.add(window);
  }
  return value as unknown as NativeWorkspaceObservation;
}

export const nativeFeasibility = {
  inspectNativeWorkspace:async():Promise<NativeWorkspaceObservation>=>
    parseNativeWorkspaceObservation(await invokeRuntime<unknown>("desktop_inspect_native_workspace")),
  // Normal Windows activation, not a second Minecraft launch request.
  focusObservedNativeWindow: (target: OwnedProcess): Promise<void> =>
    invokeRuntime<void>("desktop_focus_observed_native_window", { target }),
  readFocusObserver: async (): Promise<ForegroundEventObserverStatus> =>
    parseForegroundEventObserverStatus(await invokeRuntime<unknown>("desktop_read_focus_observer")),
  startFocusObserver: async (): Promise<ForegroundEventObserverStatus> =>
    parseForegroundEventObserverStatus(await invokeRuntime<unknown>("desktop_start_focus_observer")),
  stopFocusObserver: async (): Promise<ForegroundEventObserverStatus> =>
    parseForegroundEventObserverStatus(await invokeRuntime<unknown>("desktop_stop_focus_observer")),
  inspectInstanceFocus: async (target: ProcessIdentityCandidate): Promise<InstanceFocusObservation> =>
    parseInstanceFocus(await invokeRuntime<unknown>("desktop_inspect_instance_focus", { target })),
  inspectInstanceOverview: async (): Promise<InstanceOverview> =>
    parseInstanceOverview(await invokeRuntime<unknown>("desktop_inspect_instance_overview")),
  listInstanceSlots: async (): Promise<InstanceSlotDescriptor[]> =>
    parseInstanceSlots(await invokeRuntime<unknown>("desktop_list_instance_slots")),
  // Only the official registered Start app is requested, never a copied EXE.
  openOfficialMinecraft: () => invokeRuntime<void>("desktop_open_native_minecraft"),
  selectEducationExe: () => invokeRuntime<string | null>("desktop_select_education_exe"),
  requestSandboxClient02: (executable: string, licensed: boolean) =>
    invokeRuntime<void>("desktop_request_sandbox_client_02", { executable, licensed }),
  focusObservedClient02: (target: ProcessIdentityCandidate): Promise<void> =>
    invokeRuntime<void>("desktop_focus_observed_client_02", { target }),
  placeObservedClient02: (target: ProcessIdentityCandidate, side: "LEFT" | "RIGHT"): Promise<void> =>
    invokeRuntime<void>("desktop_place_observed_client_02", { target, side }),
  inspectIsolatedClient02: async (): Promise<IsolatedClientObservation> =>
    parseIsolatedClientObservation(await invokeRuntime<unknown>("desktop_inspect_sandbox_client_02")),
  inspectCandidates: async (): Promise<NativeProcessObservation> =>
    parseNativeObservation(await invokeRuntime<unknown>("desktop_inspect_native_candidates")),
  // Exposes a manually selected candidate directory for source-first work;
  // it does not select/install or launch the owned isolation provider.
  inspectOwnedCandidate: async (directory: string): Promise<OwnedCandidateInspection> =>
    parseOwnedCandidateInspection(await invokeRuntime<unknown>(
      "desktop_inspect_owned_engine_candidate", { directory })),
  selectOwnedCandidateDirectory: (): Promise<string | null> =>
    invokeRuntime<string | null>("desktop_select_owned_engine_candidate_directory"),
  discoverExistingVirtuals: async (): Promise<VirtualBoxInventory> =>
    parseVirtualBoxInventory(await invokeRuntime<unknown>("desktop_discover_existing_virtuals")),
  inspectOwnedHost: async (): Promise<OwnedHostInspection> =>
    parseOwnedHostInspection(await invokeRuntime<unknown>("desktop_inspect_owned_engine_host")),
};
