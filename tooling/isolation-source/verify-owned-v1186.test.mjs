import test from 'node:test';
import assert from 'node:assert/strict';
import {assessOwnedV1186, V1186_FILE_COUNT, BUILD_FILES, applyOwnedBuildPathFix} from './verify-owned-v1186.mjs';
import {readFileSync} from 'node:fs';
import {applyOwnedCredentialIsolation,CREDENTIAL_FILE,STORAGE_FILE,applyOwnedProtectedStorageIsolation} from './owned-credential-policy.mjs';

test('owned credential policy is reconstructed exactly and rejects anchor drift',()=>{
  const pristine=readFileSync(new URL('../../engine/isolation-candidates/sandboxie-v1.18.6/'+CREDENTIAL_FILE,import.meta.url),'utf8');
  const owned=readFileSync(new URL('../../engine/isolation-owned/'+CREDENTIAL_FILE,import.meta.url),'utf8').replaceAll('\r\n','\n');
  assert.equal(applyOwnedCredentialIsolation(pristine),owned);
  assert.throws(()=>applyOwnedCredentialIsolation(owned),/anchor drift/);
  assert.throws(()=>applyOwnedCredentialIsolation(pristine.replace('_FX BOOL Cred_CredReadW(', '_FX BOOL DriftedRead(')),/routine|anchor/);
});
test('App Compartment cannot bypass the reviewed private storage hook',()=>{
  const pristine=readFileSync(new URL('../../engine/isolation-candidates/sandboxie-v1.18.6/'+STORAGE_FILE,import.meta.url),'utf8');
  const owned=readFileSync(new URL('../../engine/isolation-owned/'+STORAGE_FILE,import.meta.url),'utf8').replaceAll('\r\n','\n');
  assert.equal(applyOwnedProtectedStorageIsolation(pristine),owned);
  assert.throws(()=>applyOwnedProtectedStorageIsolation(owned),/drift/);
});

const approved = ['Sandboxie/common/my_version.h',
  'Sandboxie/core/drv/api.c', 'Sandboxie/apps/start/start.cpp'];
function fixture() {
  const candidate = new Map(), owned = new Map();
  for (let i = 0; i < V1186_FILE_COUNT - approved.length; i++) {
    const p = 'Sandboxie/common/file-' + i + '.h';
    const v = {mode:'100644', sha:'a'.repeat(40)};
    candidate.set(p,v); owned.set(p,{...v});
  }
  for (const p of approved) {
    candidate.set(p,{mode:'100644',sha:'a'.repeat(40)});
    owned.set(p,{mode:'100644',sha:'b'.repeat(40)});
  }
  return {candidate, owned};
}
test('only complete pinned source and the reviewed private delta are accepted', () => {
  const f = fixture(), result = assessOwnedV1186(f.candidate,f.owned,approved);
  assert.equal(result.sourceFiles,1627);
  assert.equal(result.nativeModifiedFiles,3);
  assert.equal(result.nativeBuildVerified,false);
});
test('missing upstream path or independently added path is refused', () => {
  const f = fixture();f.owned.delete('Sandboxie/common/file-0.h');
  assert.throws(()=>assessOwnedV1186(f.candidate,f.owned,approved),/incomplete/);
  f.owned.set('unexpected/source.c',{mode:'100644',sha:'a'.repeat(40)});
  assert.throws(()=>assessOwnedV1186(f.candidate,f.owned,approved),/missing/);
});
test('an unapproved native source modification is refused', () => {
  const f = fixture();
  f.owned.set('Sandboxie/common/file-0.h',{mode:'100644',sha:'c'.repeat(40)});
  assert.throws(()=>assessOwnedV1186(f.candidate,f.owned,approved),/exact approved/);
});
test('file-mode changes and accidentally unmodified overlay paths are refused', () => {
  const f = fixture();
  f.owned.set('Sandboxie/common/file-0.h',{mode:'100755',sha:'a'.repeat(40)});
  assert.throws(()=>assessOwnedV1186(f.candidate,f.owned,approved),/mode/);
  const g = fixture();
  g.owned.set(approved[0],{...g.candidate.get(approved[0])});
  assert.throws(()=>assessOwnedV1186(g.candidate,g.owned,approved),/exact approved/);
});

test('owned build projects differ only by exact path quoting from pristine upstream', () => {
  for (const path of BUILD_FILES) {
    const pristine = readFileSync(new URL('../../engine/isolation-candidates/sandboxie-v1.18.6/' + path,
      import.meta.url), 'utf8');
    const owned = readFileSync(new URL('../../engine/isolation-owned/' + path,
      import.meta.url), 'utf8').replace(/\r\n/g, '\n');
    assert.equal(applyOwnedBuildPathFix(path, pristine), owned);
    assert.throws(() => applyOwnedBuildPathFix(path, owned), /anchor drift/);
    assert.throws(() => applyOwnedBuildPathFix(path, ''), /anchor drift/);
  }
  assert.throws(() => applyOwnedBuildPathFix('unreviewed.vcxproj', ''), /unknown/);
});
