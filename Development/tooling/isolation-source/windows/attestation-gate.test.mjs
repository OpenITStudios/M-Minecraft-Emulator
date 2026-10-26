import test from "node:test";
import assert from "node:assert/strict";
import { createHash, generateKeyPairSync, sign } from "node:crypto";
import { verifySignedLabAttestation, verifyProductionLabAttestation, PINNED_UPSTREAM,
  LAB_ATTESTATION_PURPOSE } from "./attestation-gate.mjs";

const hex = character => character.repeat(64);
const fields = () => ({
  schema: 1, purpose: LAB_ATTESTATION_PURPOSE,
  upstreamCommit: PINNED_UPSTREAM,
  transactionId: "01234567-89ab-4def-8123-456789abcdef",
  nonceSha256: hex("a"), machineFingerprintSha256: hex("b"),
  snapshotId: "11234567-89ab-4def-8123-456789abcdef",
  planSha256: hex("c"), journalSha256: hex("d"),
  issuedAt: "2026-10-14T06:00:00.000Z",
  expiresAt: "2026-10-14T06:05:00.000Z",
  disposableLabOnly: true,
});
const bindings = payload => ({
  transactionId: payload.transactionId,
  nonceSha256: payload.nonceSha256,
  machineFingerprintSha256: payload.machineFingerprintSha256,
  snapshotId: payload.snapshotId,
  planSha256: payload.planSha256,
  journalSha256: payload.journalSha256,
});
function signFixture(payload = fields()) {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const bytes = Buffer.from(JSON.stringify(payload), "utf8");
  return {
    envelope: { schema: 1, algorithm: "Ed25519",
      payloadBase64: bytes.toString("base64"),
      signatureBase64: sign(null, bytes, privateKey).toString("base64") },
    trustedPublicKeyPem: publicKey.export({ type: "spki", format: "pem" }),
    expected: bindings(payload),
    nowMs: Date.parse("2026-10-14T06:01:00.000Z"),
  };
}
test("valid pinned signing identity binds exact transaction and never authorizes an install", () => {
  const result = verifySignedLabAttestation(signFixture());
  assert.equal(result.signatureVerified, true);
  assert.equal(result.installationPermitted, false);
  assert.equal(result.rollbackPermitted, false);
  assert.equal(result.independentWindowsHostAttested, false);
  assert.match(result.signedPayloadSha256, /^[a-f0-9]{64}$/);
});
test("production verifier refuses when no lab trust anchor is configured", () => {
  const signed = signFixture(); delete signed.trustedPublicKeyPem;
  assert.throws(() => verifySignedLabAttestation(signed), /no pinned lab signing/);
});
test("forged envelope content and wrong signer are rejected", () => {
  const f = signFixture();
  const tampered = structuredClone(f);
  tampered.envelope.payloadBase64 = Buffer.from(JSON.stringify(
    { ...fields(), machineFingerprintSha256: hex("9") })).toString("base64");
  assert.throws(() => verifySignedLabAttestation(tampered), /signature invalid/);
  const other = signFixture();
  assert.throws(() => verifySignedLabAttestation({
    ...f, trustedPublicKeyPem: other.trustedPublicKeyPem,
  }), /signature invalid/);
});
test("signed evidence for different journal or snapshot cannot be reused", () => {
  const f = signFixture();
  for (const name of ["journalSha256", "planSha256", "snapshotId",
    "nonceSha256", "machineFingerprintSha256", "transactionId"]) {
    const change = { ...f, expected: { ...f.expected,
      [name]: name === "snapshotId" || name === "transactionId"
        ? "ffffffff-ffff-4fff-8fff-ffffffffffff" : hex("f") } };
    assert.throws(() => verifySignedLabAttestation(change), /not bound/);
  }
});
test("expired, future and overlong signed claims are refused", () => {
  const f = signFixture();
  assert.throws(() => verifySignedLabAttestation({
    ...f, nowMs: Date.parse("2026-10-14T06:06:00.000Z") }), /expired or future/);
  assert.throws(() => verifySignedLabAttestation({
    ...f, nowMs: Date.parse("2026-10-14T05:59:00.000Z") }), /expired or future/);
  const long = fields(); long.expiresAt = "2026-10-14T07:00:00.000Z";
  assert.throws(() => verifySignedLabAttestation(signFixture(long)), /expired or future/);
});
test("wrong upstream, added data fields and noncanonical encoding are refused", () => {
  const bad = { ...fields(), upstreamCommit: "0".repeat(40) };
  assert.throws(() => verifySignedLabAttestation(signFixture(bad)), /schema or scope/);
  const additional = { ...fields(), accountToken: "never read this" };
  assert.throws(() => verifySignedLabAttestation(signFixture(additional)), /schema or scope/);
  const f = signFixture(); f.envelope.payloadBase64 += "===";
  assert.throws(() => verifySignedLabAttestation(f), /invalid base64/);
});

test("production trust route cannot inherit a caller-supplied temporary key", () => {
  const f = signFixture();
  assert.throws(() => verifyProductionLabAttestation({
    envelope: f.envelope, expected: f.expected, nowMs: f.nowMs,
    trustedPublicKeyPem: f.trustedPublicKeyPem,
  }), /no pinned lab signing/);
});
