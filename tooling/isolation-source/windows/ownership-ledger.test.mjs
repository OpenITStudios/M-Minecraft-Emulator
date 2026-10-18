import test from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createDisposableProvisionPlan } from "../provision-disposable.mjs";
import { createTransactionJournal, openTransactionJournal, simulateProvisionTransaction } from "../execute-disposable.mjs";
import {
  appendServiceObservation, inspectOwnershipReview, recordOperatorReviewIntent,
} from "./ownership-ledger.mjs";

const H = "a".repeat(64);
function fixture() {
  const paths = ["SbieDrv.sys","SbieSvc.exe","SbieDll.dll","SbieMsg.dll",
    "kmdutil.exe","32/SbieDll.dll","32/SbieSvc.exe",
    "LICENSES/Sandboxie/COPYING","LICENSES/LICENSE.Plus"];
  const payload = {
    schema: 1, state: "ARTIFACTS_STAGED_NOT_INSTALLABLE",
    upstreamRepository: "sandboxie-plus/Sandboxie",
    upstreamTag: "v1.18.5",
    upstreamCommit: "abd4c240031cfb46666daa74ca47ed4e48b561e4",
    windowsKernelDriverLoaded: false, minecraftGameplayVerified: false,
    sandboxieExternalDependencyRemoved: false, publicDistributionApproved: false,
    artifacts: paths.map(path=>({ path, sizeBytes: 200, sha256: H, source:"Sandboxie" })),
  };
  const host = {
    schema: 1, state: "BLOCKED_UNVERIFIED_OWNED_DRIVER",
    interactiveSession: true, serviceRegistration: "ABSENT",
    driverRegistration: "ABSENT", serviceNamespaceAvailable: true,
    providerSelected: false, trustedDriverLoaded: false, activationAllowed: false,
  };
  const target = { kind: "DEDICATED_DISPOSABLE_WINDOWS_VM",
    hasAccountBearingBox: false, containsOfficialSandboxie: false,
    allowProductionInstall: false };
  const plan = createDisposableProvisionPlan({payload,host,target});
  const parent = mkdtempSync(join(tmpdir(), "owned-ledger-"));
  const journal = createTransactionJournal(join(parent,"receipt"),plan);
  return {parent, plan, journal, cleanup:()=>rmSync(parent,{recursive:true,force:true})};
}
function operator(f) {
  const snapshot = f.journal.read();
  return {
    transactionId: snapshot.transactionId,
    planFingerprint: snapshot.planFingerprint,
    action: "REVIEW_OWNED_SERVICE_RECOVERY",
    acknowledgment: "REVIEW ONLY - NEVER INSTALL STOP OR DELETE",
  };
}
function scm(component, extra={}) {
  return {
    schema:1,evidenceClass:"WINDOWS_READ_ONLY_SCM_IMAGE_PROBE",
    serviceName: component, registration:"PRESENT",
    imagePath: component === "SbieDrv" ? "D:\\Lab\\SbieDrv.sys" : "D:\\Lab\\SbieSvc.exe",
    expectedImagePath: component === "SbieDrv" ? "D:\\Lab\\SbieDrv.sys" : "D:\\Lab\\SbieSvc.exe",
    registeredImagePathMatches: true,
    imageSha256: H,
    imageSignatureStatus: "Valid",
    hostFingerprintSha256: "b".repeat(64),
    ...extra,
  };
}
const recordIntent = f => recordOperatorReviewIntent({
  plan:f.plan,journal:f.journal,request:operator(f),
  now:new Date("2026-10-14T06:00:00Z"),
});
const append = (f,component,observation=scm(component)) =>
  appendServiceObservation({plan:f.plan,journal:f.journal,observation,component,
    now:new Date("2026-10-14T06:00:01Z")});

test("operator intent is transaction-scoped and cannot approve Windows operations",()=>{
  const f=fixture();try{
    const output=recordIntent(f);
    assert.equal(output.state,"OPERATOR_REVIEW_INTENT_RECORDED_NOT_AUTHORIZATION");
    assert.equal(output.rollbackPermitted,false);
    assert.equal(inspectOwnershipReview(f).canStopService,false);
    assert.equal(f.journal.read().operatorReviewIntent.transactionId,f.journal.read().transactionId);
  }finally{f.cleanup()}
});
test("two SCM observations append in order and never prove ownership",()=>{
  const f=fixture();try{
    recordIntent(f);
    const first=append(f,"SbieDrv");
    const second=append(f,"SbieSvc");
    assert.equal(first.expectedImageMatchesObservation,true);
    assert.equal(second.sequence,2);
    assert.equal(second.rollbackPermitted,false);
    const review=inspectOwnershipReview(f);
    assert.deepEqual(review.reviewedComponents,["SbieDrv","SbieSvc"]);
    assert.equal(review.windowsOwnershipVerified,false);
    const events=f.journal.read().serviceEvidence;
    assert.equal(events[1].previousHash,events[0].entryHash);
  }finally{f.cleanup()}
});
test("operator consent refuses wrong transaction, forged phrase or replay",()=>{
  const f=fixture();try{
    for(const changed of [
      {transactionId:"01234567-89ab-4def-8123-456789abcdef"},
      {planFingerprint:"f".repeat(64)},
      {acknowledgment:"INSTALL AND DELETE"},
      {action:"DELETE_SERVICE"},
    ])assert.throws(()=>recordOperatorReviewIntent({
      plan:f.plan,journal:f.journal,request:{...operator(f),...changed}
    }),/consent does not match/);
    recordIntent(f);
    assert.throws(()=>recordIntent(f),/already recorded/);
  }finally{f.cleanup()}
});
test("SCM observation without intent or with forged identity is refused",()=>{
  const f=fixture();try{
    assert.throws(()=>append(f,"SbieSvc"),/review prerequisites/);
    recordIntent(f);
    assert.throws(()=>append(f,"SbieDrv",scm("SbieSvc")) ,/evidence shape untrusted/);
    assert.throws(()=>append(f,"Unrelated",scm("Unrelated")) ,/review prerequisites/);
    assert.throws(()=>append(f,"SbieDrv",scm("SbieDrv",{imageSha256:"invalid"})),
      /evidence shape untrusted/);
  }finally{f.cleanup()}
});
test("mismatch is recorded for review, never promoted to authorization",()=>{
  const f=fixture();try{
    recordIntent(f);
    const result=append(f,"SbieSvc",scm("SbieSvc",{
      registration:"UNKNOWN",registeredImagePathMatches:false,
      imageSha256:"",imageSignatureStatus:"Unknown",
    }));
    assert.equal(result.expectedImageMatchesObservation,false);
    assert.equal(result.independentlyOwnedService,false);
    assert.equal(result.installationPermitted,false);
  }finally{f.cleanup()}
});
test("journal rejects rewriting old evidence and reordering entries",()=>{
  const f=fixture();try{
    recordIntent(f);append(f,"SbieSvc");
    const edited=f.journal.read();edited.serviceEvidence[0].signatureStatus="NotSigned";
    assert.throws(()=>f.journal.replace(edited),/hash chain altered/);
    const removed=f.journal.read();removed.serviceEvidence=[];
    assert.throws(()=>f.journal.replace(removed),/cannot be removed/);
    const reread=f.journal.read();
    assert.equal(reread.serviceEvidence[0].signatureStatus,"Valid");
  }finally{f.cleanup()}
});
test("tampering with stored hash chain is detected on read",()=>{
  const f=fixture();try{
    recordIntent(f);append(f,"SbieDrv");
    const path=f.journal.location;
    const changed=JSON.parse(readFileSync(path,"utf8"));
    // Alter a valid existing field: verification must reach the hash-chain gate.
    changed.serviceEvidence[0].signatureStatus="NotSigned";
    writeFileSync(path,JSON.stringify(changed));
    assert.throws(()=>inspectOwnershipReview(f),/hash chain altered/);
  }finally{f.cleanup()}
});
test("SIMULATION receipt is still replay-protected after a review",async()=>{
  const f=fixture();try{
    recordIntent(f);append(f,"SbieDrv");
    const out=await simulateProvisionTransaction(f.plan,f.journal,{
      kind:"IN_MEMORY_SIMULATION",execute:async()=>({outcome:"UNKNOWN"}),
    });
    assert.equal(out.installed,false);
    assert.equal(f.journal.read().serviceEvidence.length,1);
    await assert.rejects(simulateProvisionTransaction(f.plan,f.journal,{
      kind:"IN_MEMORY_SIMULATION",execute:async()=>({outcome:"CONFIRMED"}),
    }),/replayed transaction/);
    assert.equal(inspectOwnershipReview(f).rollbackPermitted,false);
  }finally{f.cleanup()}
});

test("reopening the same canonical journal preserves review, but never resumes effects",()=>{
  const f=fixture();try{
    recordIntent(f);
    append(f,"SbieSvc");
    const opened=openTransactionJournal(join(f.parent,"receipt"),f.plan);
    const snapshot=inspectOwnershipReview({plan:f.plan,journal:opened});
    assert.equal(snapshot.state,"SCM_REVIEW_RECORDED_NOT_AUTHORIZATION");
    assert.deepEqual(snapshot.reviewedComponents,["SbieSvc"]);
    assert.equal(snapshot.canDeleteService,false);
    const outdated=opened.read();outdated.operatorReviewIntent.acknowledgment="INSTALL";
    assert.throws(()=>opened.replace(outdated),/operator review intent|untrusted or altered/);
  }finally{f.cleanup()}
});
test("cross-plan journal access cannot inherit prior operator intent or service evidence",()=>{
  const f=fixture();try {
    recordIntent(f);append(f,"SbieDrv");
    const changed=JSON.parse(JSON.stringify(f.plan));
    changed.installSteps[1].sha256="f".repeat(64);
    assert.throws(()=>openTransactionJournal(join(f.parent,"receipt"),changed),
      /invalid durable journal identity/);
  } finally {f.cleanup()}
});
