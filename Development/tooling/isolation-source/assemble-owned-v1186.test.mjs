import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import {
  existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync,
  writeFileSync,
} from 'node:fs';
import { dirname, join } from 'node:path';
import { tmpdir } from 'node:os';
import { stageOwnedV1186 } from './assemble-owned-v1186.mjs';

const sha = value => createHash('sha256').update(value).digest('hex');
const gitBlobId = value => createHash('sha1')
  .update('blob ' + value.length + '\0').update(value).digest('hex');
const HEAD='a'.repeat(40);
const TREE='b'.repeat(40);
const profile = JSON.parse(readFileSync(
  new URL('../../engine/isolation-native/owned-runtime/build-profile-v1186.json', import.meta.url),
  'utf8',
));
function fixture() {
  const base = mkdtempSync(join(tmpdir(),'mivubi-owned-v1186-'));
  const repository = join(base,'repo');
  const outputDirectory = join(repository,'workspace','packages','payload');
  const buildReport = join(repository,'workspace','builds','native-build.json');
  mkdirSync(join(repository,'workspace','packages'),{recursive:true});
  mkdirSync(join(repository,'workspace','builds'),{recursive:true});
  function file(relative, bytes=Buffer.from('owned test: '+relative)) {
    const full = join(repository,...relative.split('/'));
    mkdirSync(dirname(full),{recursive:true});
    writeFileSync(full,bytes);
    return bytes;
  }
  file('Development/engine/isolation-native/owned-runtime/build-profile-v1186.json',
    JSON.stringify(profile));
  const artifacts=profile.requiredArtifacts.map(path=>{
    const bytes=file('Development/engine/isolation-owned/'+path);
    return {path,bytes:bytes.length,sha256:sha(bytes)};
  });
  const resourceBlobs = new Map();
  for (const path of [...profile.sourceResources,...profile.originalLicensesMustBeRetained]) {
    resourceBlobs.set(path, gitBlobId(file('Development/engine/isolation-owned/'+path,
      Buffer.from('owned test: '+path+'\n'))));
  }
  const report={
    schema:1,state:'NATIVE_BUILD_ARTIFACTS_CAPTURED_NOT_INSTALLABLE',
    sourceVersion:'v1.18.6',repoHead:HEAD,upstreamTree:profile.pristineSourceTree,
    ownedTree:TREE,nativeModifiedFiles:15,
    compiler:{path:'C:\\VS\\MSBuild.exe',fileVersion:'17.0',sha256:'c'.repeat(64)},
    buildStages:profile.buildStages.map(row=>({
      stage:row.id,platform:row.platform,exitCode:0,
      stdoutLog:row.id+'.stdout.log',stderrLog:row.id+'.stderr.log',
    })),
    artifacts,driverBuilt:true,driverSigned:false,driverLoaded:false,
    installPermitted:false,activeProviderChanged:false,
    minecraftGameplayVerified:false,
  };
  const save=()=>writeFileSync(buildReport,JSON.stringify(report));
  save();
  let trackedStatus = '';
  let afterTrackedStatus = () => {};
  const adapters={
    verifySource:()=>({head:HEAD,ownedTree:TREE,nativeModifiedFiles:15}),
    readGit:(_root,...args)=>{
      const key=args.join(' ');
      if(key==='branch --show-current')return 'Local';
      if(key==='rev-parse HEAD')return HEAD;
      if(key==='rev-parse HEAD:Development/engine/isolation-owned')return TREE;
      if(key==='rev-parse HEAD:Development/engine/isolation-candidates/sandboxie-v1.18.6')
        return profile.pristineSourceTree;
      if(key==='status --porcelain=v1 --untracked-files=no -- Development/engine/isolation-owned Development/engine/isolation-native/owned-runtime/build-profile-v1186.json') {
        const status = trackedStatus;
        afterTrackedStatus();
        return status;
      }
      if(args[0]==='ls-tree' && args[1]==='-r' &&
          args[2]==='--full-tree' && args[3]==='HEAD' && args[4]==='--') {
        const prefix='Development/engine/isolation-owned/';
        assert.deepEqual(args.slice(5),[...resourceBlobs.keys()].map(p=>prefix+p));
        return [...resourceBlobs].map(([path,hash])=>
          '100644 blob '+hash+'\t'+prefix+path).join('\n');
      }
      throw Error('Unexpected Git command in fixture: '+key);
    },
  };
  return {
    base,repository,outputDirectory,buildReport,report,save,
    setTrackedStatus:(status)=>{ trackedStatus=status; },
    afterTrackedStatus:(action)=>{ afterTrackedStatus=action; },
    run:(destination=outputDirectory, reportPath=buildReport)=>
      stageOwnedV1186({repository,buildReport:reportPath,outputDirectory:destination},adapters),
    clean:()=>rmSync(base,{force:true,recursive:true}),
  };
}
test('owned v1.18.6 build produces a closed read-only 25-file engine payload',async()=>{
  const f=fixture();try{
    const result=await f.run();
    assert.equal(result.state,'MIVUBI_V1186_CORE_STAGED_NOT_INSTALLABLE');
    assert.equal(result.artifactCount,25);
    assert.equal(result.providerSelected,false);
    assert.equal(result.installPermitted,false);
    const manifest=JSON.parse(readFileSync(join(f.outputDirectory,'engine-provenance.json'),'utf8'));
    assert.equal(manifest.schema,2);
    assert.equal(manifest.sourceVersion,'v1.18.6');
    assert.equal(manifest.upstreamCommit,profile.sourceCommit);
    assert.equal(manifest.ownedRepoHead,HEAD);
    assert.equal(manifest.ownedTree,TREE);
    assert.equal(manifest.nativeModifiedFiles,15);
    assert.equal(manifest.canCoexistWithOfficialSandboxie,false);
    assert.equal(manifest.windowsKernelDriverLoaded,false);
    assert.equal(manifest.artifacts.length,25);
    assert.ok(existsSync(join(f.outputDirectory,'SbieDrv.sys')));
    assert.ok(existsSync(join(f.outputDirectory,'32','SbieDll.dll')));
    assert.ok(existsSync(join(f.outputDirectory,'Templates.ini')));
    assert.ok(existsSync(join(f.outputDirectory,'LICENSES','LICENSE.Classic')));
    assert.ok(existsSync(join(f.outputDirectory,'LICENSES','Sandboxie','COPYING')));
    assert.match(readFileSync(join(f.outputDirectory,'NOT-INSTALLABLE.txt'),'utf8'),/NOT installable or co-installable/);
  }finally{f.clean();}
});

test('CRLF checkout config and licenses stage as exact committed blob bytes', async () => {
  const f = fixture();
  try {
    const expected = new Map();
    for (const path of ['Sandboxie/install/Templates.ini', 'LICENSE.Plus']) {
      const source = join(f.repository, 'Development/engine/isolation-owned', path);
      const bytes = readFileSync(source);
      expected.set(path, bytes);
      writeFileSync(source, Buffer.from(bytes.toString('latin1').replace(/\n/g, '\r\n'), 'latin1'));
    }
    await f.run();
    assert.deepEqual(readFileSync(join(f.outputDirectory, 'Templates.ini')),
      expected.get('Sandboxie/install/Templates.ini'));
    assert.deepEqual(readFileSync(join(f.outputDirectory, 'LICENSES/LICENSE.Plus')),
      expected.get('LICENSE.Plus'));
  } finally { f.clean(); }
});
test('payload and report cannot escape dedicated ignored workspace folders',async()=>{
  const f=fixture();try{
    // Make the invalid source-side parent real so the workspace allowlist,
    // not an unrelated missing-parent preflight, is the guard being tested.
    mkdirSync(join(f.repository,'apps','desktop'),{recursive:true});
    await assert.rejects(f.run(join(f.base,'outside')), /workspace\/packages/);
    await assert.rejects(f.run(join(f.repository,'apps','desktop','payload')), /workspace\/packages/);
    await assert.rejects(f.run(undefined,join(f.base,'native-build.json')), /workspace\/builds/);
    assert.equal(existsSync(f.outputDirectory),false);
  }finally{f.clean();}
});
test('build report claiming loaded driver or signing cannot create package',async()=>{
  for(const key of ['driverSigned','driverLoaded','installPermitted',
    'activeProviderChanged','minecraftGameplayVerified']){
    const f=fixture();try{
      f.report[key]=true;f.save();
      await assert.rejects(f.run(),/unsafe runtime activation/);
      assert.equal(existsSync(f.outputDirectory),false);
    }finally{f.clean();}
  }
});
test('rejects missing, corrupt, reordered or skipped native build files',async()=>{
  const f=fixture();try{
    f.report.buildStages[2].exitCode=1;f.save();
    await assert.rejects(f.run(),/build stages not completed/);
    f.report.buildStages[2].exitCode=0;
    f.report.artifacts.reverse();f.save();
    await assert.rejects(f.run(),/artifact paths/);
    f.report.artifacts.reverse();f.save();
    writeFileSync(join(f.repository,'Development/engine/isolation-owned',
      ...profile.requiredArtifacts[0].split('/')),'tampered');
    await assert.rejects(f.run(),/native output bytes differ/);
    assert.equal(existsSync(f.outputDirectory),false);
  }finally{f.clean();}
});
test('missing licensing/source resource and existing output preserve data',async()=>{
  const f=fixture();try{
    rmSync(join(f.repository,'Development/engine/isolation-owned/LICENSE.Plus'));
    await assert.rejects(f.run());
    assert.equal(existsSync(f.outputDirectory),false);
  }finally{f.clean();}
  const g=fixture();try{
    mkdirSync(g.outputDirectory);
    writeFileSync(join(g.outputDirectory,'existing.txt'),'keep data');
    await assert.rejects(g.run(),/destination already exists/);
    assert.equal(readFileSync(join(g.outputDirectory,'existing.txt'),'utf8'),'keep data');
  }finally{g.clean();}
});
test('untrusted source HEAD, tree, report and path mismatch fail closed',async()=>{
  const f=fixture();try{
    for(const edit of [
      r=>r.repoHead='d'.repeat(40),
      r=>r.ownedTree='d'.repeat(40),
      r=>r.upstreamTree='d'.repeat(40),
      r=>r.state='BUILD_FAILED_OR_INCOMPLETE',
      r=>r.nativeModifiedFiles=10,
      r=>r.artifacts.pop(),
    ]){
      const r=structuredClone(f.report);edit(r);
      writeFileSync(f.buildReport,JSON.stringify(r));
      await assert.rejects(f.run());
      assert.equal(existsSync(f.outputDirectory),false);
    }
  }finally{f.clean();}
});

test('owned v1.18.6 staging refuses uncommitted tracked engine resources and profile', async()=>{
  const f=fixture();try{
    for(const status of [
      ' M Development/engine/isolation-owned/Sandboxie/install/Templates.ini',
      'M  Development/engine/isolation-owned/Sandboxie/core/dll/sbieapi.c',
      ' M Development/engine/isolation-native/owned-runtime/build-profile-v1186.json',
    ]){
      f.setTrackedStatus(status);
      await assert.rejects(f.run(),/tracked engine source\/profile differs/);
      assert.equal(existsSync(f.outputDirectory),false,
        'dirty tracked inputs must not publish a payload');
    }
    f.setTrackedStatus('');
    assert.equal((await f.run()).artifactCount,25);
  }finally{f.clean();}
});

test('Git detects staged and unstaged tracked engine edits, not ignored binary outputs',()=>{
  const base=mkdtempSync(join(tmpdir(),'mivubi-own-git-status-'));
  const command=(...args)=>{
    const result=spawnSync('git',args,{cwd:base,encoding:'utf8'});
    assert.equal(result.status,0,result.stderr||'git fixture command failed');
    return result.stdout.trim();
  };
  try{
    command('init','-q');
    command('config','user.email','test@example.invalid');
    command('config','user.name','MIVUBI staging test');
    mkdirSync(join(base,'Development/engine','isolation-owned'),{recursive:true});
    writeFileSync(join(base,'.gitignore'),'*.sys\n');
    const path='Development/engine/isolation-owned/Templates.ini';
    writeFileSync(join(base,path),'original');
    command('add','.gitignore',path);
    command('commit','-qm','baseline');
    writeFileSync(join(base,'Development/engine/isolation-owned/SbieDrv.sys'),'build product');
    const status=()=>command('status','--porcelain=v1','--untracked-files=no',
      '--','Development/engine/isolation-owned');
    assert.equal(status(),'');
    writeFileSync(join(base,path),'changed');
    assert.match(status(),/Templates\.ini/);
    command('add',path);
    assert.match(status(),/Templates\.ini/);
  }finally{rmSync(base,{recursive:true,force:true});}
});

test('committed resource blob binding rejects a config edited after a clean Git status',async()=>{
  const f=fixture();try{
    f.afterTrackedStatus(()=>{
      f.afterTrackedStatus(()=>{});
      writeFileSync(join(f.repository,'Development/engine/isolation-owned',
        ...profile.sourceResources[0].split('/')), 'uncommitted late edit');
    });
    await assert.rejects(f.run(), /copied resource differs from committed Git HEAD/);
    assert.equal(existsSync(f.outputDirectory),false);
  }finally{f.clean();}
});

test('committed resource blob binding rejects modified license despite a clean early snapshot',async()=>{
  const f=fixture();try{
    f.afterTrackedStatus(()=>{
      f.afterTrackedStatus(()=>{});
      writeFileSync(join(f.repository,'Development/engine/isolation-owned/LICENSE.Plus'),
        'replaced licensing text');
    });
    await assert.rejects(f.run(), /copied resource differs from committed Git HEAD: LICENSE.Plus/);
    assert.equal(existsSync(f.outputDirectory),false);
  }finally{f.clean();}
});
