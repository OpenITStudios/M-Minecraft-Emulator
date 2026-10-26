import { randomBytes } from 'node:crypto';
import {
  closeSync, existsSync, lstatSync, mkdirSync, openSync, readFileSync,
  renameSync, unlinkSync, writeFileSync,
} from 'node:fs';
import { basename, dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..', '..');
const allowedStages = new Set(['setup', 'verify-source', 'source-integrity']);
const shaPattern = /^[0-9a-f]{40}$/;
const runPattern = /^[0-9]{8}T[0-9]{6}Z-[a-f0-9]{8}-preflight-[a-f0-9]{8}$/;

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

function ensureRunFolder(root, folder) {
  const runs = runsDirectory(root);
  const actual = resolve(folder);
  if (dirname(actual) !== runs || !runPattern.test(basename(actual))) {
    throw new Error('Only an immediate preflight run directory may be finalized.');
  }
  ensurePlainDir(actual);
  const manifestFile = join(actual, 'manifest.json');
  const stat = lstatSync(manifestFile);
  if (!stat.isFile() || stat.isSymbolicLink()) {
    throw new Error('Run manifest is not a regular file.');
  }
  return manifestFile;
}

function runPrefix(commit, now) {
  return `${now.toISOString().replace(/[-:]/g, '').replace(/\.\d{3}Z$/, 'Z')}-${commit.slice(0, 8)}-preflight-`;
}

export function beginPreflight(root, commit, now = new Date()) {
  if (!shaPattern.test(commit)) throw new Error('A complete, lowercase Git commit SHA is required.');
  if (!(now instanceof Date) || !Number.isFinite(now.getTime())) throw new Error('Invalid UTC timestamp.');
  const runs = runsDirectory(root);
  const folder = join(runs, `${runPrefix(commit, now)}${randomBytes(4).toString('hex')}`);
  mkdirSync(folder, { mode: 0o700 });
  const record = {
    schema: 1,
    scenario: 'SOURCE_PREFLIGHT',
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
  writeFileSync(join(folder, 'manifest.json'), `${JSON.stringify(record, null, 2)}\n`, { flag: 'wx', mode: 0o600 });
  return folder;
}

export function finishPreflight(root, folder, status, stage, now = new Date()) {
  if (!['PASS', 'FAIL'].includes(status) ||
      !(stage === 'complete' && status === 'PASS' || allowedStages.has(stage) && status === 'FAIL')) {
    throw new Error('Invalid preflight result or stage.');
  }
  const path = ensureRunFolder(root, folder);
  const lockPath = join(dirname(path), '.finalize.lock');
  // Atomic exclusive creation serializes independent finish processes.
  // A stale lock blocks completion instead of overwriting a prior result.
  const lock = openSync(lockPath, 'wx', 0o600);
  let temporary;
  try {
    const data = JSON.parse(readFileSync(path, 'utf8'));
    if (data.schema !== 1 || data.scenario !== 'SOURCE_PREFLIGHT' ||
        data.source?.branch !== 'Local' || !shaPattern.test(data.source?.commit) ||
        typeof data.startedUtc !== 'string' || !Number.isFinite(Date.parse(data.startedUtc)) ||
        runPrefix(data.source.commit, new Date(data.startedUtc)) !==
          basename(dirname(path)).slice(0, -8) ||
        data.status !== 'RUNNING' || data.finishedUtc !== null ||
        data.lastStage !== 'not-started' || data.sourceVerification !== 'NOT_EXECUTED' ||
        data.packageVerification !== 'NOT_EXECUTED' || data.windowsGameplay !== 'NOT_EXECUTED' ||
        data.privacy !== 'METADATA_ONLY_NO_RAW_LOGS' ||
        !Array.isArray(data.evidence) || data.evidence.length !== 0) {
      throw new Error('Run manifest identity is invalid or has already been finalized.');
    }
    if (!(now instanceof Date) || !Number.isFinite(now.getTime()) ||
        now.getTime() < Date.parse(data.startedUtc)) {
      throw new Error('Run completion time predates its start.');
    }
    data.status = status;
    data.lastStage = stage;
    data.finishedUtc = now.toISOString();
    data.sourceVerification = status === 'PASS' ? 'PASS' : 'INCOMPLETE_OR_FAILED';
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
const directCall = process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (directCall) {
  try {
    const [action, ...args] = process.argv.slice(2);
    if (action === 'start' && args.length === 2 && args[0] === 'preflight') {
      console.log(beginPreflight(repoRoot, args[1]));
    } else if (action === 'finish' && args.length === 3) {
      finishPreflight(repoRoot, args[0], args[1], args[2]);
    } else {
      throw new Error('Usage: session.mjs start preflight <full-sha> | finish <run-folder> PASS complete | finish <run-folder> FAIL <stage>');
    }
  } catch (error) {
    console.error(`Local-test record failed: ${error.message}`);
    process.exitCode = 1;
  }
}
