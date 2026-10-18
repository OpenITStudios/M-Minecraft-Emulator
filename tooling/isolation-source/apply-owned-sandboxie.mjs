/**
 * Apply the existing reviewed MIVUBI native fork edits DIRECTLY to the
 * app-owned full Sandboxie source tree, as one normal Local Git commit.
 *
 * CLI: node tooling/isolation-source/apply-owned-sandboxie.mjs \
 *   --seed engine/isolation-upstream/Sandboxie --expect-head <after-import SHA>
 *
 * No Windows service, registry, Minecraft data, or upstream repo is touched.
 */
import {spawnSync} from 'node:child_process';
import {existsSync, readFileSync, lstatSync, realpathSync} from 'node:fs';
import {resolve,join,dirname} from 'node:path';
import {sameGitDirectory} from './git-directory-identity.mjs';
import {fileURLToPath} from 'node:url';
import {verifyPinnedSeed} from './bootstrap-fork.mjs';
import {
  SOURCE_FILES, buildPrivateSourcePatch, buildExpectedPrivateSources,
  verifyExactPrivateSourceBytes,
} from '../../engine/isolation-native/fork-overlay/apply-private-source.mjs';
import {OWNED_PATH} from './import-full-sandboxie.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const PIN = 'abd4c240031cfb46666daa74ca47ed4e48b561e4';
function guard(ok, msg){if(!ok)throw Error('Owned Sandboxie source patch refused: '+msg)}
function git(root,args,input=undefined){
 const p=spawnSync('git',['-C',root,...args],{input,encoding:'utf8',windowsHide:true,
   timeout:120000,maxBuffer:8*1024*1024});
 guard(!p.error&&p.status===0,`git ${args[0]}: ${(p.stderr||p.error?.message||'').slice(0,750)}`);
 return (p.stdout||'').trim();
}
function ordinaryDir(root){try{const st=lstatSync(root);return st.isDirectory()&&
 !st.isSymbolicLink()&&realpathSync(root)===resolve(root)}catch{return false}}

export function applyOwnedSandboxie({repoRoot,seedRoot,expectedHead,upstreamCommit=PIN,
 tag='v1.18.5'}) {
 guard(typeof expectedHead==='string'&&/^[0-9a-f]{40}$/.test(expectedHead),'exact Local HEAD required');
 const root=resolve(repoRoot),seed=resolve(root,seedRoot),owned=join(root,...OWNED_PATH.split('/'));
 guard(ordinaryDir(root)&&sameGitDirectory(root,git(root,['rev-parse','--show-toplevel'])),'repo root invalid');
 guard(git(root,['branch','--show-current'])==='Local','only Local branch may be modified');
 guard(git(root,['rev-parse','HEAD'])===expectedHead,'Local HEAD has changed');
 guard(git(root,['status','--porcelain=v1','--untracked-files=all'])==='','working tree is dirty');
 guard(ordinaryDir(owned),'full owned source must be imported first');
 verifyPinnedSeed(seed,{expectedCommit:upstreamCommit,tag});
 const originalTree=git(seed,['rev-parse',upstreamCommit+'^{tree}']);
 guard(git(root,['rev-parse',`HEAD:${OWNED_PATH}`])===originalTree,
   'owned source is not the complete unmodified pinned Sandboxie snapshot');
 const upstream=Object.fromEntries(SOURCE_FILES.map(p=>[p,
   readFileSync(join(owned,...p.split('/')),'utf8')]));
 const patch=buildPrivateSourcePatch(upstream);
 const expected=buildExpectedPrivateSources(upstream);
 guard(Object.keys(expected).length===SOURCE_FILES.length,'incomplete private patch');
 // Git applies changes to the COPY inside the application, never to the seed.
 git(root,['apply','--check','--index','--whitespace=error',
   '--directory='+OWNED_PATH,'-'],patch);
 git(root,['apply','--index','--whitespace=error',
   '--directory='+OWNED_PATH,'-'],patch);
 const edited=Object.fromEntries(SOURCE_FILES.map(p=>[p,
   readFileSync(join(owned,...p.split('/')),'utf8')]));
 verifyExactPrivateSourceBytes(upstream,edited);
 const changed=git(root,['diff','--cached','--name-only']).split('\n').filter(Boolean);
 const expectedPaths=SOURCE_FILES.map(p=>OWNED_PATH+'/'+p).sort();
 guard(JSON.stringify(changed)===JSON.stringify(expectedPaths),
   'staged file list includes an unexpected source');
 git(root,['diff','--cached','--check']);
 git(root,['commit','-m','feat(engine): apply MIVUBI native core changes to owned Sandboxie source']);
 guard(git(root,['status','--porcelain=v1','--untracked-files=all'])==='',
   'app Git worktree dirty after committing native source');
 return {state:'OWNED_NATIVE_SANDBOXIE_SOURCE_MODIFIED_NOT_BUILT',
   appCommit:git(root,['rev-parse','HEAD']), upstreamCommit,
   modifiedSourceCount:SOURCE_FILES.length, ownedPath:OWNED_PATH,
   upstreamUntouched:true, driverBuilt:false, driverInstalled:false,
   minecraftValidated:false};
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)){
 try{
  const args=process.argv.slice(2);
  if(args.length!==4||args[0]!=='--seed'||args[2]!=='--expect-head')
    throw Error('Usage: node apply-owned-sandboxie.mjs --seed <pinned-source-checkout> --expect-head <post-import Local SHA>');
  process.stdout.write(JSON.stringify(applyOwnedSandboxie({repoRoot:ROOT,seedRoot:args[1],expectedHead:args[3]}),null,2)+'\n');
 }catch(e){process.stderr.write(e.message+'\n');process.exitCode=1}
}
