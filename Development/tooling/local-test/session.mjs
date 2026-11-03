import { randomBytes } from 'node:crypto';
import {
  closeSync, existsSync, lstatSync, mkdirSync, openSync, readFileSync,
  renameSync, unlinkSync, writeFileSync,
} from 'node:fs';
import { basename, dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..', '..');
const stagesByKind = Object.freeze({
  preflight: new Set(['setup', 'verify-source', 'source-integrity']),
  quick: new Set(['verify-quick', 'source-integrity']),
});
const shaPattern = /^[0-9a-f]{40}$/;
const runPattern = /^[0-9]{8}T[0-9]{6}Z-[a-f0-9]{8}-(?:preflight|quick|smoke)-[a-f0-9]{8}$/;
const smokeRunPattern = /^[0-9]{8}T[0-9]{6}Z-[a-f0-9]{8}-smoke-[a-f0-9]{8}$/;
const operatorResults = new Set(['NOT_REPORTED','PASS','FAIL','BLOCKED','INCONCLUSIVE']);
const maxSmokeBytes = 16 * 1024;

function ensurePlainDir(path) {
  const stat = lstatSync(path);
  if (!stat.isDirectory() || stat.isSymbolicLink()) {
    throw new Error('Local-test storage must use real directories, not links.');
  }
}

function runsDirectory(root) {
  const workspace = join(root, 'workspace');
  ensurePlainDir(workspace);
  const runs = join(workspace, 'runs');
  if (!existsSync(runs)) mkdirSync(runs, { mode: 0o700 });
  ensurePlainDir(runs);
  return runs;
}

function ensureRunFolder(root, folder, kind) {
  const runs = runsDirectory(root);
  const actual = resolve(folder);
  if (dirname(actual) !== runs || !runPattern.test(basename(actual)) ||
      !basename(actual).includes(`-${kind}-`)) {
    throw new Error(`Only an immediate ${kind} run directory may be finalized.`);
  }
  ensurePlainDir(actual);
  const manifestFile = join(actual, 'manifest.json');
  const stat = lstatSync(manifestFile);
  if (!stat.isFile() || stat.isSymbolicLink()) {
    throw new Error('Run manifest is not a regular file.');
  }
  return manifestFile;
}

function runPrefix(commit, now, kind) {
  return `${now.toISOString().replace(/[-:]/g, '').replace(/\.\d{3}Z$/, 'Z')}-${commit.slice(0, 8)}-${kind}-`;
}

function beginSourceRun(root, commit, kind, now = new Date()) {
  if (!shaPattern.test(commit)) throw new Error('A complete, lowercase Git commit SHA is required.');
  if (!(now instanceof Date) || !Number.isFinite(now.getTime())) throw new Error('Invalid UTC timestamp.');
  const runs = runsDirectory(root);
  const folder = join(runs, `${runPrefix(commit, now, kind)}${randomBytes(4).toString('hex')}`);
  mkdirSync(folder, { mode: 0o700 });
  const record = {
    schema: 1,
    scenario: kind === 'preflight' ? 'SOURCE_PREFLIGHT' : 'SOURCE_QUICK',
    source: { branch: 'Local', commit },
    startedUtc: now.toISOString(),
    finishedUtc: null,
    status: 'RUNNING',
    lastStage: 'not-started',
    sourceVerification: 'NOT_EXECUTED',
    packageVerification: 'NOT_EXECUTED',
    windowsGameplay: 'NOT_EXECUTED',
    evidence: [],
    privacy: 'METADATA_ONLY_NO_RAW_LOGS',
  };
  if (kind === 'quick') record.quickSourceScreen = 'NOT_EXECUTED';
  writeFileSync(join(folder, 'manifest.json'), `${JSON.stringify(record, null, 2)}\n`, { flag: 'wx', mode: 0o600 });
  return folder;
}

export function beginPreflight(root, commit, now = new Date()) {
  return beginSourceRun(root, commit, 'preflight', now);
}

export function beginQuick(root, commit, now = new Date()) {
  return beginSourceRun(root, commit, 'quick', now);
}

function finishSourceRun(root, folder, status, stage, kind, now = new Date()) {
  if (!['PASS', 'FAIL'].includes(status) ||
      !(stage === 'complete' && status === 'PASS' ||
        stagesByKind[kind]?.has(stage) && status === 'FAIL')) {
    throw new Error(`Invalid ${kind} result or stage.`);
  }
  const path = ensureRunFolder(root, folder, kind);
  const lockPath = join(dirname(path), '.finalize.lock');
  // Atomic exclusive creation serializes independent finish processes.
  // A stale lock blocks completion instead of overwriting a prior result.
  const lock = openSync(lockPath, 'wx', 0o600);
  let temporary;
  try {
    const data = JSON.parse(readFileSync(path, 'utf8'));
    if (data.schema !== 1 || data.scenario !== (kind === 'preflight' ? 'SOURCE_PREFLIGHT' : 'SOURCE_QUICK') ||
        data.source?.branch !== 'Local' || !shaPattern.test(data.source?.commit) ||
        typeof data.startedUtc !== 'string' || !Number.isFinite(Date.parse(data.startedUtc)) ||
        runPrefix(data.source.commit, new Date(data.startedUtc), kind) !==
          basename(dirname(path)).slice(0, -8) ||
        data.status !== 'RUNNING' || data.finishedUtc !== null ||
        data.lastStage !== 'not-started' || data.sourceVerification !== 'NOT_EXECUTED' ||
        data.packageVerification !== 'NOT_EXECUTED' || data.windowsGameplay !== 'NOT_EXECUTED' ||
        data.privacy !== 'METADATA_ONLY_NO_RAW_LOGS' ||
        !Array.isArray(data.evidence) || data.evidence.length !== 0 ||
        (kind === 'quick' ? data.quickSourceScreen !== 'NOT_EXECUTED' :
          Object.hasOwn(data, 'quickSourceScreen'))) {
      throw new Error('Run manifest identity is invalid or has already been finalized.');
    }
    if (!(now instanceof Date) || !Number.isFinite(now.getTime()) ||
        now.getTime() < Date.parse(data.startedUtc)) {
      throw new Error('Run completion time predates its start.');
    }
    data.status = status;
    data.lastStage = stage;
    data.finishedUtc = now.toISOString();
    if (kind === 'preflight') {
      data.sourceVerification = status === 'PASS' ? 'PASS' : 'INCOMPLETE_OR_FAILED';
    } else {
      // Quick screening is NEVER a complete source/regression acceptance result.
      data.quickSourceScreen = status === 'PASS' ? 'PASS' : 'INCOMPLETE_OR_FAILED';
      data.sourceVerification = 'NOT_EXECUTED';
    }
    data.durationMilliseconds = now.getTime() - Date.parse(data.startedUtc);
    // These must never be upgraded by a source-only command.
    data.packageVerification = 'NOT_EXECUTED';
    data.windowsGameplay = 'NOT_EXECUTED';
    temporary = join(dirname(path), `.manifest-${randomBytes(6).toString('hex')}.tmp`);
    writeFileSync(temporary, `${JSON.stringify(data, null, 2)}\n`, { flag: 'wx', mode: 0o600 });
    renameSync(temporary, path);
    temporary = undefined;
    return data;
  } finally {
    if (temporary && existsSync(temporary)) unlinkSync(temporary);
    closeSync(lock);
    unlinkSync(lockPath);
  }
}
export function finishPreflight(root, folder, status, stage, now = new Date()) {
  return finishSourceRun(root, folder, status, stage, 'preflight', now);
}

export function finishQuick(root, folder, status, stage, now = new Date()) {
  return finishSourceRun(root, folder, status, stage, 'quick', now);
}


// Manual smoke evidence shares this local-test storage owner but NEVER runs the
// game or upgrades self-reported observations to authenticated acceptance.
function assertSmokeCaseIds(caseIds) {
  if (!Array.isArray(caseIds) || caseIds.length !== 13 ||
      caseIds.some((id, i) => id !== 'LT-' + String(i + 1).padStart(2, '0'))) {
    throw new Error('Expected exactly the canonical thirteen local-smoke case IDs');
  }
}
function exactObject(value, keys) {
  return value !== null && typeof value === 'object' && !Array.isArray(value) &&
    Object.keys(value).length === keys.length &&
    keys.every(key => Object.hasOwn(value, key));
}
function smokeFile(root, runId) {
  if (typeof runId !== 'string' || !smokeRunPattern.test(runId)) {
    throw new Error('Invalid bounded local-smoke run ID');
  }
  const folder = join(runsDirectory(root), runId);
  ensurePlainDir(folder);
  const file = join(folder, 'manifest.json');
  const stat = lstatSync(file);
  if (!stat.isFile() || stat.isSymbolicLink() || stat.nlink !== 1 ||
      stat.size < 1 || stat.size > maxSmokeBytes) {
    throw new Error('Unsafe local-smoke manifest');
  }
  return {folder, file};
}
function loadSmoke(root, runId, caseIds) {
  assertSmokeCaseIds(caseIds);
  const {folder, file} = smokeFile(root, runId);
  const data = JSON.parse(readFileSync(file, 'utf8'));
  const statuses = data?.cases;
  const start = Date.parse(data?.startedUtc);
  const updated = Date.parse(data?.updatedUtc);
  if (!exactObject(data, ['schema','scenario','source','startedUtc',
        'updatedUtc','privacy','trust','sourceVerification',
        'packageVerification','minecraftRuntime','cases']) ||
      data.schema !== 1 || data.scenario !== 'LOCAL_SMOKE_OPERATOR_REPORT' ||
      !exactObject(data.source, ['branch','commit']) ||
      data.source.branch !== 'Local' || !shaPattern.test(data.source.commit) ||
      !Number.isFinite(start) || !Number.isFinite(updated) || updated < start ||
      runPrefix(data.source.commit, new Date(start), 'smoke') !== runId.slice(0, -8) ||
      data.privacy !== 'METADATA_ONLY_NO_RAW_LOGS' ||
      data.trust !== 'OPERATOR_REPORTED_NOT_INDEPENDENTLY_ATTESTED' ||
      data.sourceVerification !== 'NOT_EXECUTED' ||
      data.packageVerification !== 'NOT_EXECUTED' ||
      data.minecraftRuntime !== 'NOT_INDEPENDENTLY_VERIFIED' ||
      !Array.isArray(statuses) || statuses.length !== caseIds.length) {
    throw new Error('Invalid local-smoke manifest identity or trust boundary');
  }
  let stopped = false, pending = false;
  for (let i = 0; i < caseIds.length; i++) {
    const row = statuses[i];
    if (!exactObject(row, ['id','reportedStatus','reportedUtc']) ||
        row.id !== caseIds[i] || !operatorResults.has(row.reportedStatus) ||
        (row.reportedStatus === 'NOT_REPORTED' ? row.reportedUtc !== null :
          typeof row.reportedUtc !== 'string' ||
          !Number.isFinite(Date.parse(row.reportedUtc)) ||
          Date.parse(row.reportedUtc) < start || Date.parse(row.reportedUtc) > updated)) {
      throw new Error('Invalid local-smoke case evidence');
    }
    if (row.reportedStatus === 'NOT_REPORTED') pending = true;
    else if (pending || stopped) throw new Error('Local-smoke records out of order or past stop gate');
    else if (row.reportedStatus !== 'PASS') stopped = true;
  }
  return {folder, file, data};
}
export function beginSmoke(root, commit, caseIds, now = new Date()) {
  assertSmokeCaseIds(caseIds);
  if (!shaPattern.test(commit)) throw new Error('Complete lowercase commit SHA required');
  if (!(now instanceof Date) || !Number.isFinite(now.getTime())) {
    throw new Error('Invalid local-smoke start time');
  }
  const runs = runsDirectory(root);
  const runId = runPrefix(commit, now, 'smoke') + randomBytes(4).toString('hex');
  const folder = join(runs, runId);
  mkdirSync(folder, {mode:0o700});
  const data = {
    schema:1,
    scenario:'LOCAL_SMOKE_OPERATOR_REPORT',
    source:{branch:'Local',commit},
    startedUtc:now.toISOString(),
    updatedUtc:now.toISOString(),
    privacy:'METADATA_ONLY_NO_RAW_LOGS',
    trust:'OPERATOR_REPORTED_NOT_INDEPENDENTLY_ATTESTED',
    sourceVerification:'NOT_EXECUTED',
    packageVerification:'NOT_EXECUTED',
    minecraftRuntime:'NOT_INDEPENDENTLY_VERIFIED',
    cases:caseIds.map(id=>({id,reportedStatus:'NOT_REPORTED',reportedUtc:null})),
  };
  writeFileSync(join(folder,'manifest.json'),JSON.stringify(data,null,2) + '\n',
    {flag:'wx',mode:0o600});
  return {runId,manifestRelativePath:'workspace/runs/'+runId+'/manifest.json',
    ...summarizeSmoke(data)};
}
export function summarizeSmoke(data) {
  const cases = data.cases.map(({id,reportedStatus})=>({id,reportedStatus}));
  const blocker = cases.find(c=>c.reportedStatus !== 'PASS' &&
    c.reportedStatus !== 'NOT_REPORTED');
  const pending = cases.find(c=>c.reportedStatus === 'NOT_REPORTED');
  const counts = Object.fromEntries([...operatorResults].map(x=>
    [x,cases.filter(c=>c.reportedStatus===x).length]));
  return {
    profile:'local-smoke',sourceSha:data.source.commit,
    trust:'OPERATOR_REPORTED_NOT_INDEPENDENTLY_ATTESTED',
    reportStatus:blocker?'STOPPED_OPERATOR_REPORTED':pending?
      'INCOMPLETE_OPERATOR_REPORTED':'COMPLETE_OPERATOR_REPORTED_UNVERIFIED',
    reportedCounts:counts,
    firstBlocker:blocker??null,
    nextCaseId:blocker?null:(pending?.id??null),
    cases,sourceVerification:'NOT_EXECUTED',
    packageVerification:'NOT_EXECUTED',
    minecraftRuntime:'NOT_INDEPENDENTLY_VERIFIED',
  };
}
export function readSmoke(root, runId, caseIds) {
  const {data} = loadSmoke(root, runId, caseIds);
  return summarizeSmoke(data);
}
export function recordSmoke(root, runId, commit, caseIds, caseId,
                            reportedStatus, now = new Date()) {
  assertSmokeCaseIds(caseIds);
  if (!shaPattern.test(commit) ||
      !caseIds.includes(caseId) ||
      !operatorResults.has(reportedStatus) || reportedStatus === 'NOT_REPORTED' ||
      !(now instanceof Date) || !Number.isFinite(now.getTime())) {
    throw new Error('Invalid operator-reported local-smoke result');
  }
  // Lock belongs to this exact report only, never a global game/runtime lock.
  const {folder} = smokeFile(root, runId);
  const lockPath = join(folder, '.operator-record.lock');
  const lock = openSync(lockPath, 'wx', 0o600);
  let temporary;
  try {
    const {data} = loadSmoke(root, runId, caseIds);
    if (data.source.commit !== commit) throw new Error('Local-smoke HEAD differs from original run');
    const summary = summarizeSmoke(data);
    if (!summary.nextCaseId || summary.nextCaseId !== caseId) {
      throw new Error('Only the next unreported case may be recorded; start a new run after a stop gate');
    }
    if (now.getTime() < Date.parse(data.updatedUtc)) {
      throw new Error('Local-smoke recorded time predates prior evidence');
    }
    const row = data.cases.find(c=>c.id===caseId);
    row.reportedStatus = reportedStatus;
    row.reportedUtc = now.toISOString();
    data.updatedUtc = now.toISOString();
    temporary = join(folder,'.smoke-'+randomBytes(6).toString('hex')+'.tmp');
    writeFileSync(temporary, JSON.stringify(data,null,2)+'\n',{flag:'wx',mode:0o600});
    renameSync(temporary,join(folder,'manifest.json'));
    temporary=undefined;
    return summarizeSmoke(data);
  } finally {
    if (temporary && existsSync(temporary)) unlinkSync(temporary);
    closeSync(lock);
    unlinkSync(lockPath);
  }
}

const directCall = process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (directCall) {
  try {
    const [action, ...args] = process.argv.slice(2);
    if (action === 'start' && args.length === 2 && args[0] === 'preflight') {
      console.log(beginPreflight(repoRoot, args[1]));
    } else if (action === 'start' && args.length === 2 && args[0] === 'quick') {
      console.log(beginQuick(repoRoot, args[1]));
    } else if (action === 'finish' && args.length === 3) {
      finishPreflight(repoRoot, args[0], args[1], args[2]);
    } else if (action === 'finish-quick' && args.length === 3) {
      finishQuick(repoRoot, args[0], args[1], args[2]);
    } else {
      throw new Error('Usage: session.mjs start preflight|quick <sha> | finish <folder> PASS complete | finish-quick <folder> PASS complete | FAIL <stage>');
    }
  } catch (error) {
    console.error(`Local-test record failed: ${error.message}`);
    process.exitCode = 1;
  }
}
