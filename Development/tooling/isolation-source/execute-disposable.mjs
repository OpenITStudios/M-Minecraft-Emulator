// Disposable engine transaction executor CORE. It never launches a command
// or installs services. A Windows transport is a distinct security boundary.
import { createHash, randomUUID } from "node:crypto";
import {
  closeSync, existsSync, fsyncSync, mkdirSync, openSync, readFileSync,
  renameSync, rmSync, writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { lstatSync, realpathSync } from "node:fs";
import {
  DRIVER_SERVICE, USER_SERVICE, UPSTREAM_SHA, evaluateProvisionReceipt,
} from "./provision-disposable.mjs";
import { validateOwnershipLedger } from "./windows/ownership-ledger.mjs";

const PLAN_STATE = "SOURCE_PLAN_ONLY_DRIVER_INSTALL_BLOCKED";
const JOURNAL_SCHEMA = 1;
const MAX_STEPS = 7;
const EXPECTED = [
  ["01", DRIVER_SERVICE, "RECHECK_SCM_NAMES_ABSENT"],
  ["02", DRIVER_SERVICE, "VERIFY_SIGNED_DRIVER_AND_BINARY_IDENTITY"],
  ["03", DRIVER_SERVICE, "INSTALL_KERNEL_MINIFILTER_VIA_UPSTREAM_KMDUTIL"],
  ["04", USER_SERVICE, "VERIFY_SIGNED_SERVICE_AND_BINARY_IDENTITY"],
  ["05", USER_SERVICE, "INSTALL_USER_SERVICE_VIA_UPSTREAM_KMDUTIL"],
  ["06", USER_SERVICE, "START_AND_READBACK_SIGNED_SERVICE"],
  ["07", DRIVER_SERVICE, "ATTEST_KERNEL_USERMODE_ABI_AND_BOX_ISOLATION"],
];
function planFingerprint(plan) {
  return createHash("sha256").update(JSON.stringify(plan)).digest("hex");
}
const GUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

function ensure(ok, message) {
  if (!ok) throw new Error("Owned transaction rejected: " + message);
}
function recordIsValid(record, expected) {
  return record && Object.keys(record).sort().join(",") ===
    "component,id,outcome,stepAction".split(",").sort().join(",") &&
    record.id === expected.id && record.component === expected.component &&
    record.stepAction === expected.action &&
    ["CONFIRMED", "FAILED", "UNKNOWN"].includes(record.outcome);
}
function immutableReceipt(journal) {
  return {
    schema: 1,
    steps: journal.records.map(({ id, component, outcome }) =>
      ({ id, component, outcome })),
  };
}
function validateJournal(plan, journal) {
  ensure(journal?.schema === JOURNAL_SCHEMA &&
    GUID.test(journal.transactionId) &&
    journal.upstreamCommit === plan.upstreamCommit &&
    journal.planFingerprint === planFingerprint(plan) &&
    journal.executionClass === "DISPOSABLE_SIMULATION_ONLY" &&
    ["CREATED", "SIMULATED_STEP_CONFIRMED", "AMBIGUOUS_RECONCILIATION_REQUIRED"].includes(journal.status) &&
    journal.actualDriverInstalled === false && journal.providerSelected === false &&
    journal.rollbackAuthorized === false &&
    Array.isArray(journal.records) && journal.records.length <= MAX_STEPS &&
    journal.records.length <= plan.installSteps.length,
  "invalid durable journal identity");
  for (let i = 0; i < journal.records.length; i++) {
    ensure(recordIsValid(journal.records[i], plan.installSteps[i]),
      "journal contains unrecognized or reordered operation");
    if (i > 0) ensure(journal.records[i - 1].outcome === "CONFIRMED",
      "journal continued past failed or unknown operation");
  }
  // The sole transaction journal also owns operator review and SCM evidence:
  // never introduce a second lifecycle store or infer ownership from records.
  validateOwnershipLedger(journal);
}
function checkPlan(plan) {
  ensure(plan?.schema === 1 && plan.state === PLAN_STATE &&
    plan.upstreamCommit === UPSTREAM_SHA &&
    plan.installationPermitted === false && plan.providerSelected === false &&
    plan.codeSignatureVerified === false &&
    Array.isArray(plan.installSteps) && plan.installSteps.length === MAX_STEPS &&
    plan.installSteps.every((step, i) => step.id === EXPECTED[i][0] &&
      step.component === EXPECTED[i][1] && step.action === EXPECTED[i][2]) &&
    Array.isArray(plan.rollbackSteps) && plan.rollbackSteps.length === 5,
  "plan must retain the active driver-install blocker");
}

function writeDurableJson(path, value) {
  const temp = path + "." + randomUUID() + ".tmp";
  let fd;
  try {
    fd = openSync(temp, "wx", 0o600);
    const buffer = Buffer.from(JSON.stringify(value, null, 2) + "\n", "utf8");
    ensure(buffer.byteLength < 32768, "journal exceeded bounded size");
    writeFileSync(fd, buffer);
    fsyncSync(fd);
    closeSync(fd); fd = undefined;
    renameSync(temp, path);
    // A file replace does not attest kernel state; it only records intent.
  } finally {
    if (fd !== undefined) closeSync(fd);
    rmSync(temp, { force: true });
  }
}

function bindTransactionJournal(path, plan) {
  return {
    read() {
      const buf = readFileSync(path);
      ensure(buf.length < 32768, "journal too large");
      return JSON.parse(buf.toString("utf8"));
    },
    replace(next) {
      const previous = this.read();
      validateJournal(plan, previous);
      validateJournal(plan, next);
      ensure(next.actualDriverInstalled === false &&
        next.providerSelected === false && next.rollbackAuthorized === false,
      "journal claims privileged effects");
      // Existing step history may only append, or resolve the LAST UNKNOWN.
      ensure(next.records.length >= previous.records.length &&
        next.records.length <= previous.records.length + 1,
        "provisioning step history cannot be deleted or skipped");
      for (let i = 0; i < previous.records.length; i++) {
        const before = previous.records[i], after = next.records[i];
        ensure(before.id === after.id && before.component === after.component &&
          before.stepAction === after.stepAction &&
          (before.outcome === after.outcome ||
            (i === previous.records.length - 1 && before.outcome === "UNKNOWN" &&
              ["CONFIRMED", "FAILED"].includes(after.outcome))),
          "previous provisioning outcome is immutable");
      }
      if (previous.operatorReviewIntent !== undefined) {
        ensure(JSON.stringify(previous.operatorReviewIntent) ===
          JSON.stringify(next.operatorReviewIntent), "review intent cannot change");
      }
      const oldEvidence = previous.serviceEvidence ?? [];
      const evidence = next.serviceEvidence ?? [];
      ensure(evidence.length >= oldEvidence.length &&
        evidence.length <= oldEvidence.length + 1 &&
        oldEvidence.every((event, i) => JSON.stringify(event) === JSON.stringify(evidence[i])),
        "previous SCM evidence cannot be removed, edited or reordered");
      writeDurableJson(path, next);
    },
    location: path,
  };
}

/**
 * Stores a local, exclusive, durable simulation journal. Never points at
 * an engine payload, profile directory or existing journal. No auto-cleanup
 * of receipts: later validation should be able to inspect uncertainty.
 */
export function createTransactionJournal(destination, plan) {
  checkPlan(plan);
  ensure(typeof destination === "string" && destination.trim(), "journal path required");
  const absolute = resolve(destination);
  const parent = dirname(absolute);
  ensure(existsSync(parent) && !existsSync(absolute), "journal target already exists or parent missing");
  const parentStats = lstatSync(parent);
  const actualParent = realpathSync(parent);
  const parentMatches = process.platform === "win32"
    ? actualParent.toLowerCase() === parent.toLowerCase() : actualParent === parent;
  ensure(parentStats.isDirectory() && !parentStats.isSymbolicLink() &&
    parentMatches, "journal parent must not be redirected");
  mkdirSync(absolute, { recursive: false, mode: 0o700 });
  const path = join(absolute, "transaction.json");
  const starting = {
    schema: JOURNAL_SCHEMA, transactionId: randomUUID(),
    upstreamCommit: plan.upstreamCommit,
    planFingerprint: planFingerprint(plan),
    executionClass: "DISPOSABLE_SIMULATION_ONLY",
    status: "CREATED", records: [],
    actualDriverInstalled: false, providerSelected: false,
    rollbackAuthorized: false,
  };
  writeDurableJson(path, starting);
  return bindTransactionJournal(path, plan);
}


// Reopen the EXISTING canonical journal for a separate review session.
// This is never a resume/replay of a privileged operation.
export function openTransactionJournal(directory, plan) {
  checkPlan(plan);
  ensure(typeof directory === "string" && directory.trim(), "journal directory required");
  const root = resolve(directory);
  const rootStat = lstatSync(root);
  ensure(rootStat.isDirectory() && !rootStat.isSymbolicLink() &&
    (process.platform === "win32"
      ? realpathSync(root).toLowerCase() === root.toLowerCase()
      : realpathSync(root) === root), "journal directory must not be redirected");
  const path = join(root, "transaction.json");
  const fileStat = lstatSync(path);
  ensure(fileStat.isFile() && !fileStat.isSymbolicLink(),
    "journal must be an ordinary file");
  const journal = bindTransactionJournal(path, plan);
  validateJournal(plan, journal.read());
  return journal;
}

/**
 * Executes the actual SEQUENCER against a caller-supplied SIMULATION adapter.
 * Each step is journaled UNKNOWN before invoking the adapter, and durable
 * after a confirmed result. Any exception leaves UNKNOWN for reconciliation.
 * A future Windows executor MUST live outside this function and cannot
 * be silently enabled by passing a shell adapter from Tauri.
 */
export async function simulateProvisionTransaction(plan, journal, adapter) {
  checkPlan(plan);
  ensure(adapter?.kind === "IN_MEMORY_SIMULATION" &&
    typeof adapter.execute === "function", "only in-memory simulated adapters accepted");
  const value = journal.read();
  validateJournal(plan, value);
  ensure(value.status === "CREATED" && value.records.length === 0,
    "a started or replayed transaction may not be executed");
  const state = JSON.parse(JSON.stringify(value));
  for (const step of plan.installSteps) {
    const item = {
      id: step.id, component: step.component,
      stepAction: step.action, outcome: "UNKNOWN",
    };
    state.records.push(item);
    state.status = "AMBIGUOUS_RECONCILIATION_REQUIRED";
    journal.replace(state); // WRITE-AHEAD intent before simulated side effect.
    try {
      const result = await adapter.execute(Object.freeze({ ...step }));
      ensure(result && ["CONFIRMED", "FAILED", "UNKNOWN"].includes(result.outcome),
        "untrusted simulated step result");
      item.outcome = result.outcome;
      state.status = result.outcome === "CONFIRMED"
        ? "SIMULATED_STEP_CONFIRMED" : "AMBIGUOUS_RECONCILIATION_REQUIRED";
      journal.replace(state);
      if (result.outcome !== "CONFIRMED") break;
    } catch {
      // A failed confirmation write can never be interpreted as confirmed.
      item.outcome = "UNKNOWN";
      // No retry; journal's already-durable UNKNOWN prevents replay.
      state.status = "AMBIGUOUS_RECONCILIATION_REQUIRED";
      journal.replace(state);
      break;
    }
  }
  const assessment = evaluateProvisionReceipt(plan, immutableReceipt(state));
  return {
    schema: 1, state: state.records.length === plan.installSteps.length &&
      state.records.every(r => r.outcome === "CONFIRMED")
      ? "SIMULATION_COMPLETE_NOT_INSTALLED" : "SIMULATION_STOPPED_RECONCILIATION_REQUIRED",
    journal: journal.location, assessment,
    simulatedStepCount: state.records.length,
    installed: false, driverLoaded: false,
    rollbackAuthorized: false, providerSelected: false,
  };
}

export function inspectTransactionJournal(plan, journal) {
  checkPlan(plan);
  const existing = journal.read();
  validateJournal(plan, existing);
  return {
    schema: 1, state: existing.status,
    recordedSteps: existing.records.length,
    canResumeAutomatically: false,
    canRollbackAutomatically: false,
    driverInstalled: false, providerSelected: false,
  };
}
