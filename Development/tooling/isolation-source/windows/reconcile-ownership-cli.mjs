// CLI for read-only reconciliation. No SCM commands, installer or Tauri route.
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {openTransactionJournal} from '../execute-disposable.mjs';
import {correlateStoredServiceEvidence, reportRollbackReconciliation} from './reconcile-ownership.mjs';

function boundedJson(path, limit = 65536) {
  const buffer = readFileSync(resolve(path));
  if (buffer.length > limit) throw new Error('Oversized reconciliation input');
  return JSON.parse(buffer.toString('utf8').replace(/^\uFEFF/, ''));
}
export function reconcileFromCli(args) {
  const rollback = Array.isArray(args) && args[0] === '--rollback-report';
  const input = rollback ? args.slice(1) : args;
  if (!Array.isArray(input) || input.length !== 10 ||
      input[0] !== '--plan' || input[2] !== '--journal-directory' ||
      input[4] !== '--payload-root' || input[6] !== '--driver-scm' ||
      input[8] !== '--service-scm') {
    throw new Error('Usage: reconcile-ownership-cli.mjs [--rollback-report] --plan <plan.json> --journal-directory <existing> --payload-root <Windows-candidate> --driver-scm <read-only.json> --service-scm <read-only.json>');
  }
  const plan = boundedJson(input[1]);
  const journal = openTransactionJournal(input[3], plan);
  const evidence = {plan, journal, payloadRoot:input[5],
    serviceObservations:[boundedJson(input[7],8192),boundedJson(input[9],8192)]};
  return rollback
    ? reportRollbackReconciliation(evidence)
    : correlateStoredServiceEvidence(evidence);
}
// Deliberately no CLI input for a trust key, signed-claim override, or
// privileged Apply. The production-signed API is exported from the module
// and stays blocked until a source-reviewed trust anchor is configured.
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {process.stdout.write(JSON.stringify(reconcileFromCli(process.argv.slice(2)),null,2)+'\n')}
  catch(error){process.stderr.write(error.message+'\n');process.exitCode=1}
}
