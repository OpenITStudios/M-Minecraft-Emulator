// Canonical ownership-review extension of execute-disposable.mjs's journal.
// No second lifecycle manager or persisted file: this writes only through the
// existing transaction journal. No KmdUtil, service stop/delete or game calls.
import { createHash } from "node:crypto";

const PIN = "abd4c240031cfb46666daa74ca47ed4e48b561e4";
const SHA = /^[a-f0-9]{64}$/;
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const ACTION = "REVIEW_OWNED_SERVICE_RECOVERY";
const ACK = "REVIEW ONLY - NEVER INSTALL STOP OR DELETE";
const IMAGES = Object.freeze({ SbieDrv: ["03", "SbieDrv.sys"],
  SbieSvc: ["05", "SbieSvc.exe"] });
const fail = reason => { throw new Error("Service ownership ledger rejected: " + reason); };
const record = x => x !== null && typeof x === "object" && !Array.isArray(x);
const exact = (x, keys) => record(x) && Object.keys(x).length === keys.length &&
  keys.every(k => Object.prototype.hasOwnProperty.call(x, k));
const digest = x => createHash("sha256").update(JSON.stringify(x)).digest("hex");

function verifyPlanAndJournal(plan, journal) {
  if (plan?.schema !== 1 || plan.upstreamCommit !== PIN ||
    plan.state !== "SOURCE_PLAN_ONLY_DRIVER_INSTALL_BLOCKED" ||
    plan.installationPermitted !== false ||
    !Array.isArray(plan.installSteps) || plan.installSteps.length !== 7 ||
    !Array.isArray(plan.rollbackSteps) || plan.rollbackSteps.length !== 5 ||
    typeof journal?.read !== "function" || typeof journal?.replace !== "function") {
    fail("invalid source plan or existing transaction journal");
  }
  const snapshot = journal.read();
  if (snapshot?.schema !== 1 || snapshot.upstreamCommit !== PIN ||
    snapshot.planFingerprint !== digest(plan) || !UUID.test(snapshot.transactionId) ||
    snapshot.executionClass !== "DISPOSABLE_SIMULATION_ONLY" ||
    snapshot.actualDriverInstalled !== false ||
    snapshot.providerSelected !== false || snapshot.rollbackAuthorized !== false) {
    fail("journal not bound to exact source plan or claims privileged effects");
  }
  return snapshot;
}
export function validateOwnershipLedger(value) {
  const intent = value.operatorReviewIntent;
  const events = value.serviceEvidence ?? [];
  if (intent !== undefined && intent !== null) {
    if (!exact(intent, ["transactionId","action","acknowledgment",
      "planFingerprint","acknowledgedAt"]) ||
      intent.transactionId !== value.transactionId || intent.action !== ACTION ||
      intent.acknowledgment !== ACK || intent.planFingerprint !== value.planFingerprint ||
      typeof intent.acknowledgedAt !== "string" ||
      !Number.isFinite(Date.parse(intent.acknowledgedAt))) {
      fail("untrusted or altered operator review intent");
    }
  }
  if (!Array.isArray(events) || events.length > 8 || (events.length && !intent)) {
    fail("ownership event list invalid or review intent absent");
  }
  let previousHash = "0".repeat(64);
  let seen = new Set();
  for (let index = 0; index < events.length; index++) {
    const entry = events[index];
    if (!exact(entry, ["sequence","transactionId","component","creationStep",
      "registration","expectedImageSha256","observedImageSha256",
      "registeredImagePathMatches","signatureStatus","observationClass",
      "observedAt","previousHash","entryHash"]) ||
      entry.sequence !== index + 1 || entry.transactionId !== value.transactionId ||
      !Object.hasOwn(IMAGES, entry.component) ||
      entry.creationStep !== IMAGES[entry.component][0] ||
      !["PRESENT","ABSENT","UNKNOWN"].includes(entry.registration) ||
      !SHA.test(entry.expectedImageSha256) ||
      (entry.observedImageSha256 !== null && !SHA.test(entry.observedImageSha256)) ||
      typeof entry.registeredImagePathMatches !== "boolean" ||
      !["Valid","NotSigned","Unknown"].includes(entry.signatureStatus) ||
      entry.observationClass !== "WINDOWS_READ_ONLY_SCM_IMAGE_PROBE" ||
      typeof entry.observedAt !== "string" ||
      !Number.isFinite(Date.parse(entry.observedAt)) ||
      entry.previousHash !== previousHash ||
      !SHA.test(entry.entryHash) || seen.has(entry.component)) {
      fail("untrusted, reordered or repeated SCM observation");
    }
    const { entryHash, ...unsigned } = entry;
    if (digest(unsigned) !== entryHash) fail("SCM observation hash chain altered");
    seen.add(entry.component);
    previousHash = entryHash;
  }
  return true;
}

/**
 * Record explicit human intent for REVIEW ONLY. This does not attest the
 * operator's identity, a disposable host, or a privileged authorization.
 */
export function recordOperatorReviewIntent({ plan, journal, request, now = new Date() }) {
  const snapshot = verifyPlanAndJournal(plan, journal);
  if (snapshot.operatorReviewIntent || (snapshot.serviceEvidence?.length ?? 0)) {
    fail("operator review already recorded; never overwrite");
  }
  if (!exact(request, ["transactionId","action","acknowledgment",
      "planFingerprint"]) ||
      request.transactionId !== snapshot.transactionId ||
      request.planFingerprint !== snapshot.planFingerprint ||
      request.action !== ACTION || request.acknowledgment !== ACK ||
      !(now instanceof Date) || !Number.isFinite(now.getTime())) {
    fail("review consent does not match the journal transaction");
  }
  snapshot.operatorReviewIntent = {
    ...request, acknowledgedAt: now.toISOString(),
  };
  snapshot.serviceEvidence = [];
  validateOwnershipLedger(snapshot);
  journal.replace(snapshot);
  return {
    schema: 1, state: "OPERATOR_REVIEW_INTENT_RECORDED_NOT_AUTHORIZATION",
    transactionId: snapshot.transactionId,
    installationPermitted: false, rollbackPermitted: false,
    canStopService: false, canDeleteService: false,
  };
}

/**
 * Append bounded read-only SCM observations for forensic reconciliation.
 * An observation, even Authenticode Valid, is NOT service ownership proof.
 */
export function appendServiceObservation({
  plan, journal, observation, component, now = new Date(),
}) {
  const snapshot = verifyPlanAndJournal(plan, journal);
  validateOwnershipLedger(snapshot);
  if (!snapshot.operatorReviewIntent || !Object.hasOwn(IMAGES, component) ||
      !(now instanceof Date) || !Number.isFinite(now.getTime())) {
    fail("read-only review prerequisites missing");
  }
  const [creationStep, imageFile] = IMAGES[component];
  const buildStage = plan.installSteps.find(step => step.id === (component === "SbieDrv" ? "02" : "04"));
  if (!buildStage || buildStage.path !== imageFile ||
    !SHA.test(buildStage.sha256)) fail("missing source build image SHA-256");
  if (!exact(observation, ["schema","evidenceClass","serviceName",
    "registration","imagePath","expectedImagePath",
    "registeredImagePathMatches","imageSha256","imageSignatureStatus",
    "hostFingerprintSha256"]) ||
    observation.schema !== 1 ||
    observation.evidenceClass !== "WINDOWS_READ_ONLY_SCM_IMAGE_PROBE" ||
    observation.serviceName !== component ||
    !["PRESENT","ABSENT","UNKNOWN"].includes(observation.registration) ||
    typeof observation.registeredImagePathMatches !== "boolean" ||
    !["Valid","NotSigned","Unknown"].includes(observation.imageSignatureStatus) ||
    typeof observation.imageSha256 !== "string" ||
    (observation.imageSha256 !== "" && !SHA.test(observation.imageSha256)) ||
    typeof observation.imagePath !== "string" ||
    typeof observation.expectedImagePath !== "string" ||
    typeof observation.hostFingerprintSha256 !== "string" ||
    (observation.hostFingerprintSha256 !== "" && !SHA.test(observation.hostFingerprintSha256))) {
    fail("SCM evidence shape untrusted");
  }
  const events = snapshot.serviceEvidence ?? [];
  if (events.length >= 2 || events.some(x => x.component === component)) {
    fail("service already reviewed or event limit exceeded");
  }
  const entry = {
    sequence: events.length + 1, transactionId: snapshot.transactionId,
    component, creationStep,
    registration: observation.registration,
    expectedImageSha256: buildStage.sha256,
    observedImageSha256: observation.imageSha256 || null,
    registeredImagePathMatches: observation.registeredImagePathMatches,
    signatureStatus: observation.imageSignatureStatus,
    observationClass: observation.evidenceClass,
    observedAt: now.toISOString(),
    previousHash: events.at(-1)?.entryHash ?? "0".repeat(64),
  };
  const completed = { ...entry, entryHash: digest(entry) };
  snapshot.serviceEvidence = [...events, completed];
  validateOwnershipLedger(snapshot);
  journal.replace(snapshot);
  return {
    schema: 1, state: "SCM_EVIDENCE_APPEND_ONLY_REVIEW",
    component, sequence: completed.sequence,
    expectedImageMatchesObservation: observation.registration === "PRESENT" &&
      observation.registeredImagePathMatches &&
      observation.imageSha256 === buildStage.sha256 &&
      observation.imageSignatureStatus === "Valid",
    independentlyOwnedService: false,
    rollbackPermitted: false, installationPermitted: false,
  };
}

export function inspectOwnershipReview({ plan, journal }) {
  const snapshot = verifyPlanAndJournal(plan, journal);
  validateOwnershipLedger(snapshot);
  const events = snapshot.serviceEvidence ?? [];
  return {
    schema: 1, state: events.length
      ? "SCM_REVIEW_RECORDED_NOT_AUTHORIZATION"
      : snapshot.operatorReviewIntent
        ? "OPERATOR_REVIEW_PENDING_SCM_EVIDENCE"
        : "NO_OPERATOR_REVIEW_INTENT",
    reviewedComponents: events.map(e => e.component),
    transactionId: snapshot.transactionId,
    canRetry: false, canInstall: false, canStopService: false,
    canDeleteService: false, rollbackPermitted: false,
    windowsOwnershipVerified: false,
  };
}
