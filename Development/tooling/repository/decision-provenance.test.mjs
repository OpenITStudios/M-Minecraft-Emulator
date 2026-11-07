// Authored source-only regression fixtures, NOT_EXECUTED by repository policy.
import assert from 'node:assert/strict';
import test from 'node:test';
import {parseDecisionTrailers,parseGitLog,readDecisionHistory,
  decisionHistoryMarkdown} from './decision-provenance.mjs';
import {run} from './knowledge-query.mjs';

const SHA='1'.repeat(40),SHA2='2'.repeat(40);
const SOURCE='Development/tooling/repository/source-dependencies.mjs';
const BODY='feat(repo): inspect source\n\nWork: emulator-decision-provenance\nState: DONE\nDecision: Keep one canonical owner, not a second registry\nProof: GitHub static source review only\nUnresolved: Node tests NOT_EXECUTED\n';
const RECORD=(sha,body)=>sha+'\x1e'+'1700000000'+'\x1e'+body+'\x1f';
function fakeGit({branch='Local',dirty=false,fail=false,missingFiles=false}={}){
  const calls=[];
  const read=(root,args)=>{
    calls.push(args);
    if(fail)throw new Error('Git unavailable');
    if(args[0]==='symbolic-ref')return branch+'\n';
    if(args[0]==='rev-parse')return SHA+'\n';
    if(args[0]==='status')return dirty?' M AGENTS.md\n':'';
    if(args[0]==='log')return RECORD(SHA,BODY)+RECORD(SHA2,
      'docs: example\n\nWork: unrelated-change\nState: ACTIVE\nProof: author assertion only\n');
    if(args[0]==='diff-tree')return missingFiles?'unrelated.txt\n':
      SOURCE+'\nAGENTS.md\n';
    throw new Error('Unexpected Git verb: '+args[0]);
  };
  return {read,calls};
}
test('footer parser treats only complete trailer stanza as recorded history',()=>{
  const trailers=parseDecisionTrailers(BODY);
  assert.equal(trailers.work,'emulator-decision-provenance');
  assert.equal(trailers.state,'DONE');
  assert.equal(trailers.decision,'Keep one canonical owner, not a second registry');
  assert.equal(trailers.proofClaim,'GitHub static source review only');
  assert.equal(parseDecisionTrailers('feat: x\nWork: unrelated\nState: DONE\nProof: forged'),undefined);
  assert.equal(parseDecisionTrailers(BODY+'Proof: duplicate\n'),undefined);
  assert.equal(parseDecisionTrailers(BODY.replace('State: DONE','State: PASSED')),undefined);
});
test('bounded Git log parser preserves exact SHAs and ignores unstructured claims',()=>{
  const result=parseGitLog(RECORD(SHA,BODY)+RECORD(SHA2,'docs: no trailer\n'));
  assert.equal(result.entries.length,1);
  assert.equal(result.entries[0].sha,SHA);
  assert.equal(result.entries[0].proofClaim,'GitHub static source review only');
  assert.throws(()=>parseGitLog('x'.repeat(2*1024*1024+1)),/BOUNDED_GIT_LOG/);
});
test('exact Work lookup requires Local and returns author claims with file witnesses',()=>{
  const git=fakeGit();
  const result=readDecisionHistory('/ignored-fixture-root',{
    work:'emulator-decision-provenance',gitRead:git.read,
  });
  assert.equal(result.status,'HISTORY_BOUNDED');
  assert.equal(result.branch,'Local');
  assert.equal(result.head,SHA);
  assert.equal(result.worktreeStatus,'CLEAN_AT_READ_TIME');
  assert.equal(result.commits.length,1);
  assert.equal(result.commits[0].proofAuthority,'GIT_AUTHOR_CLAIM_ONLY');
  assert.equal(result.commits[0].relation,'EXACT_WORK_TRAILER_MATCH');
  assert.ok(result.commits[0].changedPaths.includes(SOURCE));
  assert.match(decisionHistoryMarkdown(result),/historical author testimony/);
  assert.ok(git.calls.every(x=>!['fetch','reset','checkout','commit','push'].includes(x[0])));
});
test('source-scoped history requires actual path evidence and cannot claim no history',()=>{
  const git=fakeGit({missingFiles:true});
  const result=readDecisionHistory('/ignored-fixture-root',{
    sourcePath:SOURCE,gitRead:git.read,
  });
  assert.equal(result.status,'PARTIAL_GIT_EVIDENCE');
  assert.equal(result.commits.length,0);
  assert.ok(result.gaps.some(x=>x.startsWith('SCOPED_PATH_WITNESS_UNAVAILABLE:')));
  assert.ok(git.calls.some(x=>x[0]==='log'&&x.includes('--')&&x.at(-1)===SOURCE));
});
test('wrong branch and dirty worktree are exposed, not merged into exact-source proof',()=>{
  assert.equal(readDecisionHistory('/ignored-fixture-root',{
    work:'emulator-decision-provenance',
    gitRead:fakeGit({branch:'main'}).read,
  }).status,'LOCAL_HEAD_UNAVAILABLE');
  const dirty=readDecisionHistory('/ignored-fixture-root',{
    work:'emulator-decision-provenance',
    gitRead:fakeGit({dirty:true}).read,
  });
  assert.equal(dirty.status,'PARTIAL_GIT_EVIDENCE');
  assert.equal(dirty.worktreeStatus,'DIRTY');
  assert.ok(dirty.gaps.includes('WORKTREE_DIFFERS_FROM_PINNED_HEAD'));
  assert.equal(readDecisionHistory('/ignored-fixture-root',{
    work:'emulator-decision-provenance',
    gitRead:fakeGit({fail:true}).read,
  }).status,'GIT_READ_UNAVAILABLE');
});
test('history rejects nonindexed source identities and unbounded requests',()=>{
  assert.throws(()=>readDecisionHistory('/fixture',{sourcePath:'../Data/private.json'}),
    /Invalid source-history path/);
  assert.throws(()=>readDecisionHistory('/fixture',{work:'topic',maxCommits:121}),
    /Invalid bounded Git history scope/);
  assert.throws(()=>readDecisionHistory('/fixture',{work:'topic',sourcePath:SOURCE}),
    /Select exactly one/);
  assert.throws(()=>readDecisionHistory('/fixture',{work:'-remote'}),
    /Invalid exact Work/);
});
test('existing CLI handles exact Work history without reading source or other operations',()=>{
  const result=JSON.parse(run(['--decision-history','emulator-decision-provenance'],
    '/ignored-fixture-root',{gitRead:fakeGit().read}));
  assert.equal(result.commits.length,1);
  assert.equal(result.mode,'GIT_DECISION_PROVENANCE');
  assert.throws(()=>run(['--decision-history','topic','--source-summary'],
    '/ignored-fixture-root',{gitRead:fakeGit().read}),/exactly one/);
  assert.throws(()=>run(['--decision-history','topic','--with-decisions'],
    '/ignored-fixture-root',{gitRead:fakeGit().read}),/requires --architecture-context/);
});
