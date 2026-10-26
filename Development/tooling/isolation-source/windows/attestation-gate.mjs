// Cryptographic review of lab evidence. Pure read-only: never a driver
// installation permit. Public trust keys come ONLY from reviewed source,
// never from a caller-supplied JSON envelope or user-selected file.
import { createPublicKey, createHash, verify } from "node:crypto";

export const PINNED_UPSTREAM = "abd4c240031cfb46666daa74ca47ed4e48b561e4";
export const LAB_ATTESTATION_PURPOSE = "MIVUBI_ISOLATION_DISPOSABLE_LAB_V1";
const SHA256 = /^[a-f0-9]{64}$/;
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const B64 = /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/;
const fail = reason => { throw new Error("Disposable attestation rejected: " + reason); };
const plain = x => x !== null && typeof x === "object" && !Array.isArray(x);
const keysAre = (x, keys) => plain(x) && Object.keys(x).length === keys.length &&
  keys.every(k => Object.prototype.hasOwnProperty.call(x, k));
function decode64(s, max) {
  if (typeof s !== "string" || !s || s.length > max || !B64.test(s)) fail("invalid base64 evidence");
  const bytes = Buffer.from(s, "base64");
  if (bytes.toString("base64") !== s) fail("noncanonical base64 evidence");
  return bytes;
}

// This is an EMPTY production trust anchor by default. Inserting a real lab
// public key requires explicit review; private keys never belong in source.
export const PRODUCTION_LAB_TRUST_KEY_PEM = "";

export function verifySignedLabAttestation({
  envelope, expected, trustedPublicKeyPem = PRODUCTION_LAB_TRUST_KEY_PEM,
  nowMs = Date.now(),
}) {
  if (!keysAre(envelope, ["schema", "algorithm", "payloadBase64", "signatureBase64"]) ||
      envelope.schema !== 1 || envelope.algorithm !== "Ed25519") fail("invalid signed envelope");
  if (typeof trustedPublicKeyPem !== "string" || !trustedPublicKeyPem.trim()) {
    fail("no pinned lab signing public key configured");
  }
  const publicKey = createPublicKey(trustedPublicKeyPem);
  if (publicKey.asymmetricKeyType !== "ed25519") fail("wrong trust key algorithm");
  const payloadBytes = decode64(envelope.payloadBase64, 8192);
  const signature = decode64(envelope.signatureBase64, 256);
  if (signature.length !== 64 || !verify(null, payloadBytes, publicKey, signature)) {
    fail("cryptographic signature invalid");
  }
  let facts;
  try { facts = JSON.parse(payloadBytes.toString("utf8")); } catch { fail("invalid signed payload"); }
  const fields = ["schema","purpose","upstreamCommit","transactionId","nonceSha256",
    "machineFingerprintSha256","snapshotId","planSha256","journalSha256",
    "issuedAt","expiresAt","disposableLabOnly"];
  if (!keysAre(facts,fields) || facts.schema !== 1 ||
      facts.purpose !== LAB_ATTESTATION_PURPOSE || facts.upstreamCommit !== PINNED_UPSTREAM ||
      facts.disposableLabOnly !== true || !UUID.test(facts.transactionId) ||
      !UUID.test(facts.snapshotId) || !SHA256.test(facts.nonceSha256) ||
      !SHA256.test(facts.machineFingerprintSha256) || !SHA256.test(facts.planSha256) ||
      !SHA256.test(facts.journalSha256)) fail("signed payload schema or scope invalid");
  const issued = Date.parse(facts.issuedAt), expires = Date.parse(facts.expiresAt);
  if (!Number.isSafeInteger(nowMs) || !Number.isFinite(issued) || !Number.isFinite(expires) ||
      expires - issued < 1 || expires - issued > 10 * 60 * 1000 ||
      nowMs < issued || nowMs >= expires) fail("expired or future lab evidence");
  const required = ["transactionId","nonceSha256","machineFingerprintSha256",
    "snapshotId","planSha256","journalSha256"];
  if (!plain(expected) || required.some(field =>
    facts[field] !== expected[field])) fail("signed evidence not bound to current machine and transaction");
  return Object.freeze({
    schema: 1,
    state: "SIGNED_LAB_CLAIM_MATCHES_BINDINGS_NOT_WINDOWS_ATTESTATION",
    signedPayloadSha256: createHash("sha256").update(payloadBytes).digest("hex"),
    upstreamCommit: PINNED_UPSTREAM,
    signatureVerified: true,
    independentWindowsHostAttested: false,
    serviceOwnershipVerified: false,
    installationPermitted: false,
    rollbackPermitted: false,
  });
}

/**
 * Only this entrypoint may be used for product/lab integration; its trust
 * anchor cannot be supplied by an arbitrary envelope or CLI argument.
 * With no approved source-pinned public key this ALWAYS refuses.
 */
export function verifyProductionLabAttestation({ envelope, expected, nowMs = Date.now() }) {
  return verifySignedLabAttestation({
    envelope, expected, nowMs,
    trustedPublicKeyPem: PRODUCTION_LAB_TRUST_KEY_PEM,
  });
}
