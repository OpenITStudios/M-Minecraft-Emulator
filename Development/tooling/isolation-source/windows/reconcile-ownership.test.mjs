import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtempSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createDisposableProvisionPlan } from '../provision-disposable.mjs';
import { createTransactionJournal, simulateProvisionTransaction } from '../execute-disposable.mjs';
import { appendServiceObservation, recordOperatorReviewIntent } from './ownership-ledger.mjs';
import { correlateStoredServiceEvidence, reconcileSignedStoredEvidence, reportRollbackReconciliation } from './reconcile-ownership.mjs';

const hash = letter => letter.repeat(64);
const root = 'D:\\MIVUBI\\LabPayload';
const filename = component => root + '\\' +
  (component === 'SbieDrv' ? 'SbieDrv.sys' : 'SbieSvc.exe');
const observation = (component, changes = {}) => ({
  schema: 1, evidenceClass: 'WINDOWS_READ_ONLY_SCM_IMAGE_PROBE',
  serviceName: component, registration: 'PRESENT',
  imagePath: '"' + filename(component) + '"', expectedImagePath: filename(component),
  registeredImagePathMatches: true, imageSha256: hash('a'),
  imageSignatureStatus: 'Valid', hostFingerprintSha256: hash('b'),
  ...changes,
});
function fixture() {
  const paths = ['SbieDrv.sys', 'SbieSvc.exe', 'SbieDll.dll', 'SbieMsg.dll',
    'kmdutil.exe', '32/SbieDll.dll', '32/SbieSvc.exe',
    'LICENSES/Sandboxie/COPYING', 'LICENSES/LICENSE.Plus'];
  const payload = { schema: 1, state: 'ARTIFACTS_STAGED_NOT_INSTALLABLE',
    upstreamRepository: 'sandboxie-plus/Sandboxie', upstreamTag: 'v1.18.5',
    upstreamCommit: 'abd4c240031cfb46666daa74ca47ed4e48b561e4',
    windowsKernelDriverLoaded: false, minecraftGameplayVerified: false,
    sandboxieExternalDependencyRemoved: false, publicDistributionApproved: false,
    artifacts: paths.map(path => ({ path, sizeBytes: 16, sha256: hash('a'), source: 'Sandboxie' })),
  };
  const host = { schema: 1, state: 'BLOCKED_UNVERIFIED_OWNED_DRIVER',
    interactiveSession: true, serviceRegistration: 'ABSENT', driverRegistration: 'ABSENT',
    serviceNamespaceAvailable: true, providerSelected: false,
    trustedDriverLoaded: false, activationAllowed: false };
  const target = {kind: 'DEDICATED_DISPOSABLE_WINDOWS_VM', hasAccountBearingBox: false,
    containsOfficialSandboxie: false, allowProductionInstall: false};
  const plan = createDisposableProvisionPlan({payload, host, target});
  const parent = mkdtempSync(join(tmpdir(), 'owned-reconciliation-'));
  const journal = createTransactionJournal(join(parent, 'receipt'), plan);
  const state = journal.read();
  recordOperatorReviewIntent({plan, journal, request: {
    transactionId: state.transactionId, planFingerprint: state.planFingerprint,
    action: 'REVIEW_OWNED_SERVICE_RECOVERY',
    acknowledgment: 'REVIEW ONLY - NEVER INSTALL STOP OR DELETE',
  }, now: new Date('2026-10-14T06:00:00.000Z')});
  for (const component of ['SbieDrv', 'SbieSvc']) {
    appendServiceObservation({plan,journal,component,observation:observation(component),
      now: new Date('2026-10-14T06:00:01.000Z')});
  }
  return {plan,journal,parent,cleanup:()=>rmSync(parent,{recursive:true,force:true})};
}
function correlate(f, observations = [observation('SbieDrv'),observation('SbieSvc')]) {
  return correlateStoredServiceEvidence({plan:f.plan,journal:f.journal,
    payloadRoot:root,serviceObservations:observations});
}
test('matches exact persisted journal bytes and both pinned image identities without enabling rollback', () => {
  const f = fixture();try {
    const result=correlate(f);
    assert.equal(result.state,'LOCAL_EVIDENCE_CORRELATED_NOT_AUTHORIZED');
    assert.equal(result.journalSha256,createHash('sha256').update(readFileSync(f.journal.location)).digest('hex'));
    assert.equal(result.planSha256,createHash('sha256').update(JSON.stringify(f.plan)).digest('hex'));
    assert.deepEqual(result.verifiedComponents,['SbieDrv','SbieSvc']);
    assert.equal(result.installationPermitted,false);
    assert.equal(result.canDeleteService,false);
    assert.equal(result.windowsServiceOwnershipVerified,false);
  }finally{f.cleanup()}
});
test('rejects missing or repeated SCM observations instead of treating an incomplete review as ownership',()=>{
  const f=fixture();try {
    assert.throws(()=>correlate(f,[observation('SbieSvc')]),/two read-only component observations/);
    assert.throws(()=>correlate(f,[observation('SbieSvc'),observation('SbieSvc')]),/without duplicates/);
  }finally{f.cleanup()}
});
test('rejects service image outside pinned payload and changed SHA',()=>{
  const f=fixture();try {
    assert.throws(()=>correlate(f,[observation('SbieDrv',{imagePath:'C:\\Other\\SbieDrv.sys'}),observation('SbieSvc')]),/does not match/);
    assert.throws(()=>correlate(f,[observation('SbieDrv'),observation('SbieSvc',{imageSha256:hash('f')})]),/does not match/);
  }finally{f.cleanup()}
});
test('refuses conflicting host identities, unsigned images and unknown SCM registration',()=>{
  const f=fixture();try {
    for(const change of [{hostFingerprintSha256:hash('c')},{imageSignatureStatus:'NotSigned'},
      {registration:'UNKNOWN'},{registeredImagePathMatches:false}]) {
      assert.throws(()=>correlate(f,[observation('SbieDrv'),observation('SbieSvc',change)]),/does not match/);
    }
  }finally{f.cleanup()}
});
test('refuses an altered persisted hash chain even if caller provides a fresh-looking observation',()=>{
  const f=fixture();try {
    const altered=JSON.parse(readFileSync(f.journal.location,'utf8'));
    altered.serviceEvidence[0].signatureStatus='NotSigned';
    writeFileSync(f.journal.location,JSON.stringify(altered));
    assert.throws(()=>correlate(f),/hash chain altered/);
  }finally{f.cleanup()}
});
test('signed production reconciliation always refuses without a source-pinned trust key',()=>{
  const f=fixture();try {
    assert.throws(()=>reconcileSignedStoredEvidence({plan:f.plan,journal:f.journal,
      payloadRoot:root,serviceObservations:[observation('SbieDrv'),observation('SbieSvc')],
      nonceSha256:hash('e'),snapshotId:'11234567-89ab-4def-8123-456789abcdef',
      signedLabEnvelope:{schema:1,algorithm:'Ed25519',payloadBase64:'e30=',signatureBase64:'YQ=='},
      nowMs:Date.parse('2026-10-14T06:01:00.000Z')}),/no pinned lab signing/);
  }finally{f.cleanup()}
});
test('invalid challenge and snapshot IDs fail before signed approval',()=>{
  const f=fixture();try {
    assert.throws(()=>reconcileSignedStoredEvidence({plan:f.plan,journal:f.journal,payloadRoot:root,
      serviceObservations:[observation('SbieDrv'),observation('SbieSvc')],
      nonceSha256:'INVALID',snapshotId:'bad',signedLabEnvelope:{}}),/missing signed challenge/);
  }finally{f.cleanup()}
});

test('rollback report identifies service names that were not created by this simulation journal',()=>{
  const f=fixture();try {
    const report=reportRollbackReconciliation({plan:f.plan,journal:f.journal,
      payloadRoot:root,serviceObservations:[observation('SbieDrv'),observation('SbieSvc')]});
    assert.equal(report.state,'MANUAL_REVIEW_UNATTRIBUTED_SERVICE_NAME');
    assert.deepEqual(report.observations.map(x=>x.recordedCreationOutcome),['NOT_RECORDED','NOT_RECORDED']);
    assert.equal(report.ownershipIndependentlyProven,false);
    assert.equal(report.canStopService,false);
    assert.equal(report.canDeleteService,false);
  }finally{f.cleanup()}
});
test('rollback report detects SCM registration removed since recorded observation',()=>{
  const f=fixture();try {
    const current=[observation('SbieDrv'),observation('SbieSvc',{
      registration:'ABSENT',registeredImagePathMatches:false,
      imagePath:'',imageSha256:'',imageSignatureStatus:'Unknown',
    })];
    const out=reportRollbackReconciliation({plan:f.plan,journal:f.journal,
      payloadRoot:root,serviceObservations:current});
    assert.equal(out.state,'MANUAL_REVIEW_CHANGED_SINCE_JOURNAL_OBSERVATION');
    assert.equal(out.canDeleteService,false);
  }finally{f.cleanup()}
});
test('rollback report refuses mismatched path even if hash and journal name are unchanged',()=>{
  const f=fixture();try {
    const current=[observation('SbieDrv',{imagePath:'C:\\Other\\SbieDrv.sys'}),observation('SbieSvc')];
    const report=reportRollbackReconciliation({plan:f.plan,journal:f.journal,
      payloadRoot:root,serviceObservations:current});
    assert.equal(report.state,'MANUAL_REVIEW_IMAGE_MISMATCH');
    assert.equal(report.observations[0].pinnedImageMetadataMatches,false);
  }finally{f.cleanup()}
});
test('rollback report identifies ambiguous simulated creation without retry',async()=>{
  const f=fixture();try {
    let count=0;
    await simulateProvisionTransaction(f.plan,f.journal,{kind:'IN_MEMORY_SIMULATION',
      execute:async()=>({outcome:++count===3?'UNKNOWN':'CONFIRMED'})});
    const report=reportRollbackReconciliation({plan:f.plan,journal:f.journal,
      payloadRoot:root,serviceObservations:[observation('SbieDrv'),observation('SbieSvc')]});
    assert.equal(report.state,'MANUAL_REVIEW_AMBIGUOUS_CREATION_RECORD');
    assert.equal(report.observations[0].recordedCreationOutcome,'UNKNOWN');
    assert.equal(report.canResumeAutomatically,false);
  }finally{f.cleanup()}
});
test('successful simulated stages still cannot establish ownership of Windows services',async()=>{
  const f=fixture();try {
    await simulateProvisionTransaction(f.plan,f.journal,{kind:'IN_MEMORY_SIMULATION',
      execute:async()=>({outcome:'CONFIRMED'})});
    const report=reportRollbackReconciliation({plan:f.plan,journal:f.journal,
      payloadRoot:root,serviceObservations:[observation('SbieDrv'),observation('SbieSvc')]});
    assert.equal(report.state,'MATCHED_SIMULATION_ONLY_NOT_SERVICE_OWNERSHIP');
    assert.equal(report.observations[0].provenCreatedByThisTransaction,false);
    assert.equal(report.canInstallDriver,false);
    assert.equal(report.canDeleteService,false);
  }finally{f.cleanup()}
});
test('rollback report rejects missing or duplicated service observations',()=>{
  const f=fixture();try {
    const one=[observation('SbieDrv')];
    assert.throws(()=>reportRollbackReconciliation({plan:f.plan,journal:f.journal,
      payloadRoot:root,serviceObservations:one}),/two component observations/);
    const duplicate=[observation('SbieDrv'),observation('SbieDrv')];
    assert.throws(()=>reportRollbackReconciliation({plan:f.plan,journal:f.journal,
      payloadRoot:root,serviceObservations:duplicate}),/two component observations/);
  }finally{f.cleanup()}
});
test('rollback report rejects different host identities or malformed source evidence',()=>{
  const f=fixture();try {
    const differentHost=[observation('SbieDrv'),observation('SbieSvc',{hostFingerprintSha256:hash('c')})];
    assert.throws(()=>reportRollbackReconciliation({plan:f.plan,journal:f.journal,
      payloadRoot:root,serviceObservations:differentHost}),/host fingerprints disagree/);
    const fake=[observation('SbieDrv',{unrecognizedMetadata:true}),observation('SbieSvc')];
    assert.throws(()=>reportRollbackReconciliation({plan:f.plan,journal:f.journal,
      payloadRoot:root,serviceObservations:fake}),/untrusted Windows SCM observation/);
  }finally{f.cleanup()}
});
