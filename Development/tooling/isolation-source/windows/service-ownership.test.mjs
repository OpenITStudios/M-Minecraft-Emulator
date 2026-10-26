import test from "node:test";
import assert from "node:assert/strict";
import { generateKeyPairSync, sign } from "node:crypto";
import { reviewServiceOwnership, reviewProductionServiceOwnership } from "./service-ownership.mjs";
import { LAB_ATTESTATION_PURPOSE, PINNED_UPSTREAM } from "./attestation-gate.mjs";

const h = c => c.repeat(64);
const basic = () => ({
  transactionId: "01234567-89ab-4def-8123-456789abcdef",
  nonceSha256: h("a"),
  machineFingerprintSha256: h("b"),
  snapshotId: "11234567-89ab-4def-8123-456789abcdef",
  planSha256: h("c"),
  journalSha256: h("d"),
});
function fixture() {
  const expectedLabBindings = basic();
  const data = {schema:1,purpose:LAB_ATTESTATION_PURPOSE,
    upstreamCommit:PINNED_UPSTREAM,...expectedLabBindings,
    issuedAt:"2026-10-14T06:00:00.000Z",
    expiresAt:"2026-10-14T06:05:00.000Z",
    disposableLabOnly:true};
  const bytes = Buffer.from(JSON.stringify(data));
  const {privateKey,publicKey} = generateKeyPairSync("ed25519");
  const signedLabEnvelope = {
    schema:1,algorithm:"Ed25519",payloadBase64:bytes.toString("base64"),
    signatureBase64:sign(null,bytes,privateKey).toString("base64"),
  };
  const serviceObservation = {
    schema:1,evidenceClass:"WINDOWS_READ_ONLY_SCM_IMAGE_PROBE",
    serviceName:"SbieSvc",registration:"PRESENT",
    imagePath:'"D:\\Lab\\SbieSvc.exe"',
    expectedImagePath:"D:\\Lab\\SbieSvc.exe",
    registeredImagePathMatches:true,imageSha256:h("e"),
    imageSignatureStatus:"Valid",
    hostFingerprintSha256:expectedLabBindings.machineFingerprintSha256,
  };
  const transactionReceipt = {
    schema:1,transactionId:expectedLabBindings.transactionId,
    serviceName:"SbieSvc",creationStepId:"05",registrationBefore:"ABSENT",
    imagePath:serviceObservation.expectedImagePath,
    imageSha256:serviceObservation.imageSha256,
    planSha256:expectedLabBindings.planSha256,
    journalSha256:expectedLabBindings.journalSha256,
    createdByThisTransaction:true,
  };
  return {signedLabEnvelope,
    pinnedLabPublicKeyPem:publicKey.export({type:"spki",format:"pem"}),
    expectedLabBindings,serviceObservation,transactionReceipt,
    nowMs:Date.parse("2026-10-14T06:01:00.000Z")};
}
test("signed matching image+receipt is correlated but NEVER authorizes rollback",()=>{
  const o=reviewServiceOwnership(fixture());
  assert.equal(o.cryptographicLabClaimVerified,true);
  assert.equal(o.imageIdentityCorrelated,true);
  assert.equal(o.windowsServiceOwnershipIndependentlyAttested,false);
  assert.equal(o.rollbackPermitted,false);
  assert.equal(o.installationPermitted,false);
});
test("fake receipt flags and changed transaction ID fail",()=>{
  const f=fixture();
  for(const change of [
    {createdByThisTransaction:false},{transactionId:"01234567-89ab-4def-8123-456789abcdee"},
    {registrationBefore:"PRESENT"},{creationStepId:"03"},{journalSha256:h("f")},
    {imageSha256:h("f")},{imagePath:"C:\\Program Files\\Sandboxie-Plus\\SbieSvc.exe"},
    {ownerProofVerified:true},
  ]) assert.throws(()=>reviewServiceOwnership({
    ...f,transactionReceipt:{...f.transactionReceipt,...change}
  }),/journal and SCM image ownership/);
});
test("SCM outside exact binary or unverifiable signing cannot pass",()=>{
  const f=fixture();
  for(const change of [
    {registration:"UNKNOWN"},{registeredImagePathMatches:false},
    {imagePath:"C:\\Original\\SbieSvc.exe"},{imageSignatureStatus:"NotSigned"},
    {hostFingerprintSha256:h("f")},{imageSha256:"no hash"},
    {serviceName:"UnrelatedService"},{extra:"secret"},
  ]) assert.throws(()=>reviewServiceOwnership({
    ...f,serviceObservation:{...f.serviceObservation,...change}
  }),/SCM image identity/);
});
test("rollbacks require lab trusted key and fresh evidence",()=>{
  const f=fixture();
  assert.throws(()=>reviewServiceOwnership({...f,pinnedLabPublicKeyPem:""}),
    /no pinned lab signing/);
  assert.throws(()=>reviewServiceOwnership({
    ...f,nowMs:Date.parse("2026-10-14T07:00:00.000Z")
  }),/expired or future/);
});
test("driver receipt must name step 03 and SbieDrv.sys",()=>{
  const f=fixture();
  f.serviceObservation={
    ...f.serviceObservation,serviceName:"SbieDrv",
    imagePath:"D:\\Lab\\SbieDrv.sys",expectedImagePath:"D:\\Lab\\SbieDrv.sys"};
  f.transactionReceipt={
    ...f.transactionReceipt,serviceName:"SbieDrv",creationStepId:"03",
    imagePath:"D:\\Lab\\SbieDrv.sys"};
  assert.equal(reviewServiceOwnership(f).rollbackPermitted,false);
  f.transactionReceipt.creationStepId="05";
  assert.throws(()=>reviewServiceOwnership(f),/journal and SCM/);
});

test("production ownership gate is sealed even when fixture key and receipt match", () => {
  const f = fixture();
  assert.throws(() => reviewProductionServiceOwnership(f),
    /no pinned lab signing/);
});
