// Read-only reconciliation of the existing disposable simulation journal,
// a fresh Windows SCM image observation, and a production-signed lab claim.
// This module is NOT an installation, driver-load, stop or rollback gate.
import { createHash } from 'node:crypto';
import { lstatSync, readFileSync } from 'node:fs';
import { inspectOwnershipReview } from './ownership-ledger.mjs';
import { inspectTransactionJournal } from '../execute-disposable.mjs';
import { compileTransportStep } from './transport-commands.mjs';
import { verifyProductionLabAttestation, PINNED_UPSTREAM } from './attestation-gate.mjs';

const SHA = /^[a-f0-9]{64}$/;
const UUID = /^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$/i;
const COMPONENTS = Object.freeze(['SbieDrv', 'SbieSvc']);
const DIGEST = data => createHash('sha256').update(data).digest('hex');
const fail = detail => { throw new Error('Owned reconciliation refused: ' + detail); };
const exact = (value, keys) => value !== null && typeof value === 'object' &&
  !Array.isArray(value) && Object.keys(value).length === keys.length &&
  keys.every(key => Object.hasOwn(value, key));

function journalFileBytes(journal) {
  if (!journal || typeof journal.read !== 'function' ||
      typeof journal.location !== 'string') fail('existing canonical journal required');
  const status = lstatSync(journal.location);
  if (!status.isFile() || status.isSymbolicLink() || status.size < 2 || status.size > 32768) {
    fail('canonical journal is not an ordinary bounded file');
  }
  const bytes = readFileSync(journal.location);
  if (bytes.length !== status.size) fail('journal changed while being read');
  const stored = JSON.parse(bytes.toString('utf8'));
  if (JSON.stringify(stored) !== JSON.stringify(journal.read())) {
    fail('journal changed between independent reads');
  }
  return { stored, sha256: DIGEST(bytes) };
}

function validateObservedImage(actual, ledgerEvent, spec, fingerprint) {
  const keys = ['schema', 'evidenceClass', 'serviceName', 'registration',
    'imagePath', 'expectedImagePath', 'registeredImagePathMatches',
    'imageSha256', 'imageSignatureStatus', 'hostFingerprintSha256'];
  if (!exact(actual, keys) || actual.schema !== 1 ||
      actual.evidenceClass !== 'WINDOWS_READ_ONLY_SCM_IMAGE_PROBE' ||
      actual.serviceName !== ledgerEvent.component || actual.registration !== 'PRESENT' ||
      ledgerEvent.registration !== 'PRESENT' ||
      !actual.registeredImagePathMatches || !ledgerEvent.registeredImagePathMatches ||
      actual.expectedImagePath !== spec.requiredImagePath ||
      ![spec.requiredImagePath, '"' + spec.requiredImagePath + '"'].includes(actual.imagePath) ||
      actual.imageSha256 !== spec.requiredImageSha256 ||
      actual.imageSha256 !== ledgerEvent.observedImageSha256 ||
      ledgerEvent.expectedImageSha256 !== spec.requiredImageSha256 ||
      actual.imageSignatureStatus !== 'Valid' || ledgerEvent.signatureStatus !== 'Valid' ||
      !SHA.test(actual.hostFingerprintSha256) ||
      (fingerprint !== null && actual.hostFingerprintSha256 !== fingerprint)) {
    fail('live SCM image does not match the bound journal and pinned payload');
  }
  return actual.hostFingerprintSha256;
}

/** Independently re-read the CURRENT journal bytes; do not trust a caller's journal hash. */
export function correlateStoredServiceEvidence({plan, journal, payloadRoot, serviceObservations}) {
  const review = inspectOwnershipReview({plan, journal}); // verifies ledger chain
  const {stored, sha256: journalSha256} = journalFileBytes(journal);
  if (review.state !== 'SCM_REVIEW_RECORDED_NOT_AUTHORIZATION' ||
      !Array.isArray(stored.serviceEvidence) || stored.serviceEvidence.length !== 2 ||
      !Array.isArray(serviceObservations) || serviceObservations.length !== 2 ||
      stored.upstreamCommit !== PINNED_UPSTREAM || !UUID.test(stored.transactionId) ||
      stored.actualDriverInstalled !== false || stored.rollbackAuthorized !== false ||
      stored.providerSelected !== false) {
    fail('two read-only component observations and a bound simulation journal required');
  }
  const names = new Set(serviceObservations.map(item => item?.serviceName));
  if (names.size !== COMPONENTS.length || COMPONENTS.some(name => !names.has(name))) {
    fail('driver and service observations are both required, without duplicates');
  }
  let fingerprint = null;
  for (const component of COMPONENTS) {
    const entry = stored.serviceEvidence.find(item => item.component === component);
    const observed = serviceObservations.find(item => item.serviceName === component);
    if (!entry || !observed) fail('component evidence missing');
    const stepId = component === 'SbieDrv' ? '02' : '04';
    const spec = compileTransportStep({plan, payloadRoot, stepId});
    if (spec.executionAllowed !== false || spec.component !== component) {
      fail('owned transport descriptor unexpectedly authorizes a service');
    }
    fingerprint = validateObservedImage(observed, entry, spec, fingerprint);
  }
  const planSha256 = DIGEST(JSON.stringify(plan));
  if (stored.planFingerprint !== planSha256) fail('journal belongs to a different plan');
  return Object.freeze({
    schema: 1, state: 'LOCAL_EVIDENCE_CORRELATED_NOT_AUTHORIZED',
    upstreamCommit: PINNED_UPSTREAM, transactionId: stored.transactionId,
    planSha256, journalSha256, machineFingerprintSha256: fingerprint,
    verifiedComponents: Object.freeze([...COMPONENTS]),
    signedLabVerified: false, independentWindowsHostAttested: false,
    windowsServiceOwnershipVerified: false, installationPermitted: false,
    canStopService: false, canDeleteService: false, rollbackPermitted: false,
  });
}

/** Production-only signer path. The source-pinned trust anchor is currently empty. */
export function reconcileSignedStoredEvidence({plan, journal, payloadRoot,
  serviceObservations, signedLabEnvelope, nonceSha256, snapshotId, nowMs = Date.now()}) {
  const local = correlateStoredServiceEvidence({plan, journal, payloadRoot, serviceObservations});
  if (!SHA.test(nonceSha256) || !UUID.test(snapshotId)) {
    fail('missing signed challenge nonce or snapshot identity');
  }
  const expected = {
    transactionId: local.transactionId, nonceSha256,
    machineFingerprintSha256: local.machineFingerprintSha256,
    snapshotId, planSha256: local.planSha256, journalSha256: local.journalSha256,
  };
  // Cannot be bypassed by user-provided keys; with no reviewed production key
  // pinned this FAILS CLOSED, even if a local JSON claims Valid signatures.
  const signed = verifyProductionLabAttestation({envelope: signedLabEnvelope, expected, nowMs});
  return Object.freeze({
    ...local, state: 'SIGNED_CLAIM_CORRELATED_STILL_NOT_AUTHORIZED',
    signedLabPayloadSha256: signed.signedPayloadSha256,
    signedLabVerified: true,
    independentWindowsHostAttested: false, windowsServiceOwnershipVerified: false,
    installationPermitted: false, canStopService: false,
    canDeleteService: false, rollbackPermitted: false,
  });
}


/**
 * Read-only rollback diagnosis for partial/failed transactions.
 *
 * A simulated receipt cannot prove a real driver/service was created. This
 * report compares the recorded creation step, previous SCM observation and
 * a fresh caller-provided SCM observation; it NEVER returns deletion authority.
 */
export function reportRollbackReconciliation({plan, journal, payloadRoot, serviceObservations}) {
  inspectTransactionJournal(plan, journal); // validates sequence and pinned plan
  inspectOwnershipReview({plan, journal});  // validates operator intent + hash chain
  const {stored, sha256: journalSha256} = journalFileBytes(journal);

  if (!Array.isArray(serviceObservations) || serviceObservations.length !== 2 ||
      new Set(serviceObservations.map(v => v?.serviceName)).size !== 2 ||
      COMPONENTS.some(name => !serviceObservations.some(v => v.serviceName === name)) ||
      !Array.isArray(stored.records) || !Array.isArray(stored.serviceEvidence ?? []) ||
      stored.upstreamCommit !== PINNED_UPSTREAM ||
      stored.actualDriverInstalled !== false ||
      stored.providerSelected !== false || stored.rollbackAuthorized !== false) {
    fail('two component observations and an unprivileged journal are mandatory');
  }
  const observations = [];
  let fingerprint = null;
  for (const component of COMPONENTS) {
    const actual = serviceObservations.find(v => v.serviceName === component);
    const keys = ['schema', 'evidenceClass', 'serviceName', 'registration',
      'imagePath', 'expectedImagePath', 'registeredImagePathMatches',
      'imageSha256', 'imageSignatureStatus', 'hostFingerprintSha256'];
    if (!exact(actual, keys) || actual.schema !== 1 ||
        actual.evidenceClass !== 'WINDOWS_READ_ONLY_SCM_IMAGE_PROBE' ||
        !['PRESENT','ABSENT','UNKNOWN'].includes(actual.registration) ||
        typeof actual.imagePath !== 'string' || typeof actual.expectedImagePath !== 'string' ||
        typeof actual.registeredImagePathMatches !== 'boolean' ||
        typeof actual.imageSha256 !== 'string' ||
        !['Valid','NotSigned','Unknown'].includes(actual.imageSignatureStatus) ||
        typeof actual.hostFingerprintSha256 !== 'string' ||
        (actual.hostFingerprintSha256 !== '' && !SHA.test(actual.hostFingerprintSha256)) ||
        (actual.imageSha256 !== '' && !SHA.test(actual.imageSha256))) {
      fail('untrusted Windows SCM observation shape');
    }
    // Two observations must belong to one hashed machine. An unavailable
    // identity is reported as UNKNOWN, never replaced with a dummy ID.
    if (fingerprint !== null && actual.hostFingerprintSha256 !== fingerprint) {
      fail('driver and service host fingerprints disagree');
    }
    fingerprint = actual.hostFingerprintSha256;
    const imageStepId = component === 'SbieDrv' ? '02' : '04';
    const creationId = component === 'SbieDrv' ? '03' : '05';
    const spec = compileTransportStep({plan,payloadRoot,stepId:imageStepId});
    if (spec.executionAllowed !== false || spec.component !== component) {
      fail('transport specification claims privileged authority');
    }
    const recorded = stored.records.find(v => v.id === creationId);
    const previous = (stored.serviceEvidence ?? []).find(v => v.component === component);
    const matchesImage = actual.registration === 'PRESENT' &&
      actual.registeredImagePathMatches === true &&
      actual.expectedImagePath === spec.requiredImagePath &&
      [spec.requiredImagePath, '"' + spec.requiredImagePath + '"'].includes(actual.imagePath) &&
      actual.imageSha256 === spec.requiredImageSha256 &&
      actual.imageSignatureStatus === 'Valid';
    const matchesPrevious = previous === undefined ? null :
      previous.registration === actual.registration &&
      previous.registeredImagePathMatches === actual.registeredImagePathMatches &&
      previous.observedImageSha256 === (actual.imageSha256 || null) &&
      previous.signatureStatus === actual.imageSignatureStatus;
    observations.push(Object.freeze({
      component,
      creationStepId: creationId,
      recordedCreationOutcome: recorded?.outcome ?? 'NOT_RECORDED',
      scmRegistration: actual.registration,
      pinnedImageMetadataMatches: matchesImage,
      historicalObservationMatches: matchesPrevious,
      // SOURCE simulation journal does not establish creation ownership.
      provenCreatedByThisTransaction: false,
      serviceStopPermitted: false,
      serviceDeletePermitted: false,
    }));
  }
  const states = observations.map(x => x.scmRegistration);
  let state;
  if (states.includes('UNKNOWN') || !fingerprint) {
    state = 'MANUAL_REVIEW_UNKNOWN_SCM_OR_HOST';
  } else if (observations.some(x =>
    x.recordedCreationOutcome === 'FAILED' ||
    x.recordedCreationOutcome === 'UNKNOWN')) {
    state = 'MANUAL_REVIEW_AMBIGUOUS_CREATION_RECORD';
  } else if (observations.some(x => x.historicalObservationMatches === false)) {
    state = 'MANUAL_REVIEW_CHANGED_SINCE_JOURNAL_OBSERVATION';
  } else if (observations.some(x =>
    x.scmRegistration === 'PRESENT' && !x.pinnedImageMetadataMatches)) {
    state = 'MANUAL_REVIEW_IMAGE_MISMATCH';
  } else if (observations.some(x =>
    x.scmRegistration === 'PRESENT' &&
    x.recordedCreationOutcome !== 'CONFIRMED')) {
    state = 'MANUAL_REVIEW_UNATTRIBUTED_SERVICE_NAME';
  } else if (states.every(x => x === 'ABSENT')) {
    state = 'NO_SCM_REGISTRATIONS_OBSERVED_NO_RUNTIME_PROOF';
  } else if (states.every(x => x === 'PRESENT')) {
    state = 'MATCHED_SIMULATION_ONLY_NOT_SERVICE_OWNERSHIP';
  } else {
    state = 'MANUAL_REVIEW_PARTIAL_SERVICE_REGISTRATION';
  }
  return Object.freeze({
    schema: 1, state,
    transactionId: stored.transactionId,
    upstreamCommit: PINNED_UPSTREAM,
    journalSha256, planSha256: DIGEST(JSON.stringify(plan)),
    observations: Object.freeze(observations),
    journalIsSimulationOnly: true,
    ownershipIndependentlyProven: false,
    verifiedWindowsHost: false,
    rollbackPermitted: false,
    canStopService: false,
    canDeleteService: false,
    canInstallDriver: false,
    canResumeAutomatically: false,
  });
}
