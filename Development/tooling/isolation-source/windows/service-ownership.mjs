// Correlate signed lab claims, journal intent, and live read-only SCM data.
// An identity match is NOT authority to install/stop/delete a Windows service.
import {
  verifySignedLabAttestation, PRODUCTION_LAB_TRUST_KEY_PEM,
} from "./attestation-gate.mjs";

const SHA = /^[a-f0-9]{64}$/;
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const fail = message => { throw new Error("Service ownership review refused: " + message); };
const isRecord = x => x !== null && typeof x === "object" && !Array.isArray(x);
const fieldsExactly = (x, fields) => isRecord(x) &&
  Object.keys(x).length === fields.length &&
  fields.every(key => Object.prototype.hasOwnProperty.call(x, key));
const IMAGE = Object.freeze({ SbieSvc: "SbieSvc.exe", SbieDrv: "SbieDrv.sys" });
const INSTALL_STEP = Object.freeze({ SbieSvc: "05", SbieDrv: "03" });

export function reviewServiceOwnership({
  signedLabEnvelope, pinnedLabPublicKeyPem, expectedLabBindings,
  serviceObservation, transactionReceipt, nowMs,
}) {
  // Attestation is verified against a trust key supplied by reviewed SOURCE.
  // The caller's JSON must never supply or select its own trust anchor.
  const lab = verifySignedLabAttestation({
    envelope: signedLabEnvelope, expected: expectedLabBindings,
    trustedPublicKeyPem: pinnedLabPublicKeyPem, nowMs,
  });
  if (!fieldsExactly(serviceObservation, ["schema","evidenceClass",
    "serviceName","registration","imagePath","expectedImagePath",
    "registeredImagePathMatches","imageSha256","imageSignatureStatus",
    "hostFingerprintSha256"]) ||
    serviceObservation.schema !== 1 ||
    serviceObservation.evidenceClass !== "WINDOWS_READ_ONLY_SCM_IMAGE_PROBE" ||
    !Object.hasOwn(IMAGE, serviceObservation.serviceName) ||
    serviceObservation.registration !== "PRESENT" ||
    serviceObservation.registeredImagePathMatches !== true ||
    (serviceObservation.imagePath !== serviceObservation.expectedImagePath &&
      serviceObservation.imagePath !== '"' + serviceObservation.expectedImagePath + '"') ||
    !SHA.test(serviceObservation.imageSha256) ||
    serviceObservation.imageSignatureStatus !== "Valid" ||
    serviceObservation.hostFingerprintSha256 !==
      expectedLabBindings.machineFingerprintSha256) {
    fail("SCM image identity absent, unknown or not matching the attested host");
  }
  if (!fieldsExactly(transactionReceipt, ["schema","transactionId",
    "serviceName","creationStepId","registrationBefore","imagePath",
    "imageSha256","planSha256","journalSha256","createdByThisTransaction"]) ||
    transactionReceipt.schema !== 1 ||
    !UUID.test(transactionReceipt.transactionId) ||
    transactionReceipt.transactionId !== expectedLabBindings.transactionId ||
    transactionReceipt.serviceName !== serviceObservation.serviceName ||
    transactionReceipt.creationStepId !== INSTALL_STEP[transactionReceipt.serviceName] ||
    transactionReceipt.registrationBefore !== "ABSENT" ||
    transactionReceipt.createdByThisTransaction !== true ||
    transactionReceipt.imagePath !== serviceObservation.expectedImagePath ||
    transactionReceipt.imageSha256 !== serviceObservation.imageSha256 ||
    transactionReceipt.planSha256 !== expectedLabBindings.planSha256 ||
    transactionReceipt.journalSha256 !== expectedLabBindings.journalSha256) {
    fail("journal and SCM image ownership evidence are inconsistent");
  }
  // This is corroboration of READ-ONLY evidence. The signature is on a lab
  // statement; it is not Microsoft's driver approval or SCM ownership proof.
  return Object.freeze({
    schema: 1,
    state: "CORRELATED_SERVICE_EVIDENCE_NOT_AUTHORIZED_FOR_ROLLBACK",
    serviceName: transactionReceipt.serviceName,
    signedLabPayloadSha256: lab.signedPayloadSha256,
    imageSha256: serviceObservation.imageSha256,
    sourceCommit: lab.upstreamCommit,
    imageIdentityCorrelated: true,
    cryptographicLabClaimVerified: true,
    windowsServiceOwnershipIndependentlyAttested: false,
    rollbackPermitted: false,
    installationPermitted: false,
  });
}

/**
 * Production-sealed ownership review: NEVER accepts a trust key supplied by
 * the caller's JSON or test fixture. No key is pinned, so this fails closed.
 */
export function reviewProductionServiceOwnership(inputs) {
  const { pinnedLabPublicKeyPem: _ignored, ...facts } = inputs;
  return reviewServiceOwnership({
    ...facts, pinnedLabPublicKeyPem: PRODUCTION_LAB_TRUST_KEY_PEM,
  });
}
