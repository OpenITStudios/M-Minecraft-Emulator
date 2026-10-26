import test from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  createTransactionJournal, simulateProvisionTransaction,
  inspectTransactionJournal,
} from "./execute-disposable.mjs";

const ACTIONS = [
  ["SbieDrv","RECHECK_SCM_NAMES_ABSENT"],
  ["SbieDrv","VERIFY_SIGNED_DRIVER_AND_BINARY_IDENTITY"],
  ["SbieDrv","INSTALL_KERNEL_MINIFILTER_VIA_UPSTREAM_KMDUTIL"],
  ["SbieSvc","VERIFY_SIGNED_SERVICE_AND_BINARY_IDENTITY"],
  ["SbieSvc","INSTALL_USER_SERVICE_VIA_UPSTREAM_KMDUTIL"],
  ["SbieSvc","START_AND_READBACK_SIGNED_SERVICE"],
  ["SbieDrv","ATTEST_KERNEL_USERMODE_ABI_AND_BOX_ISOLATION"],
];
const plan = () => ({
  schema: 1, state: "SOURCE_PLAN_ONLY_DRIVER_INSTALL_BLOCKED",
  upstreamCommit: "abd4c240031cfb46666daa74ca47ed4e48b561e4",
  installSteps: ACTIONS.map(([component,action], i) => ({
    id: String(i+1).padStart(2, "0"), component, action,
  })),
  rollbackSteps: Array.from({length: 5}, (_, i) => ({id: "R" + (i+1)})),
  installationPermitted: false, providerSelected: false,
  codeSignatureVerified: false,
});
function setup() {
  const base = mkdtempSync(join(tmpdir(), "owned-transaction-"));
  const path = join(base, "receipt");
  const p = plan();
  const journal = createTransactionJournal(path, p);
  return {base, path, plan: p, journal, cleanup: () => rmSync(base, {recursive:true, force:true})};
}
const adapter = fn => ({kind: "IN_MEMORY_SIMULATION", execute: fn});
test("all seven stages are journaled and never report an installed driver", async () => {
  const f = setup(); try {
    const result = await simulateProvisionTransaction(f.plan, f.journal,
      adapter(async () => ({outcome:"CONFIRMED"})));
    assert.equal(result.state, "SIMULATION_COMPLETE_NOT_INSTALLED");
    assert.equal(result.installed, false);
    assert.equal(result.driverLoaded, false);
    assert.equal(result.assessment.installationComplete, false);
    const journal = JSON.parse(readFileSync(f.journal.location, "utf8"));
    assert.equal(journal.records.length, 7);
    assert.ok(journal.records.every(r => r.outcome === "CONFIRMED"));
  } finally {f.cleanup();}
});
test("unknown side effect stops, persists uncertainty, and cannot replay", async () => {
  const f = setup(); try {
    let count=0;
    const result = await simulateProvisionTransaction(f.plan, f.journal,
      adapter(async () => ({outcome: ++count === 2 ? "UNKNOWN" : "CONFIRMED"})));
    assert.equal(count, 2);
    assert.equal(result.state, "SIMULATION_STOPPED_RECONCILIATION_REQUIRED");
    const journal = JSON.parse(readFileSync(f.journal.location,"utf8"));
    assert.equal(journal.records[1].outcome,"UNKNOWN");
    await assert.rejects(simulateProvisionTransaction(f.plan, f.journal,
      adapter(async ()=>({outcome:"CONFIRMED"}))),/replayed transaction/);
  } finally {f.cleanup();}
});
test("thrown adapter leaves UNKNOWN after durable intent and no retry", async () => {
  const f=setup(); try {
    let calls=0;
    await simulateProvisionTransaction(f.plan,f.journal,adapter(async ()=>{
      calls++; throw new Error("unobserved helper exit"); }));
    assert.equal(calls, 1);
    const journal=f.journal.read();
    assert.equal(journal.records.length,1);
    assert.equal(journal.records[0].outcome,"UNKNOWN");
    assert.equal(journal.status,"AMBIGUOUS_RECONCILIATION_REQUIRED");
    assert.equal(inspectTransactionJournal(f.plan,f.journal).canRollbackAutomatically,false);
  } finally {f.cleanup();}
});
test("failed step stops and cannot be interpreted as successful provisioning", async () => {
  const f=setup(); try {
    let calls=0;
    const out=await simulateProvisionTransaction(f.plan,f.journal,adapter(async ()=>{
      calls++;return {outcome: "FAILED"}; }));
    assert.equal(calls,1);
    assert.equal(out.installed,false);
    assert.equal(out.assessment.state,"STOP_MANUAL_RECONCILIATION_REQUIRED");
  } finally {f.cleanup();}
});
test("untrusted transport and forged activation-ready plan are refused", async () => {
  const f=setup(); try {
    await assert.rejects(simulateProvisionTransaction(f.plan,f.journal,
      {kind:"WINDOWS_PRIVILEGED_EXECUTION",execute:async()=>({outcome:"CONFIRMED"})}),/only in-memory/);
    assert.equal(f.journal.read().records.length,0);
    assert.throws(()=>createTransactionJournal(join(f.base,"forged"),
      {...f.plan,installationPermitted:true}),/driver-install blocker/);
  } finally {f.cleanup();}
});
test("journal directory is exclusive and never deletes a prior receipt", () => {
  const f=setup();try {
    assert.throws(()=>createTransactionJournal(f.path,f.plan),/already exists/);
    assert.ok(existsSync(f.journal.location));
  }finally{f.cleanup();}
});
test("tampered journal is rejected before any transport invocation", async () => {
  const f=setup();try {
    const j=f.journal.read();
    j.providerSelected=true;
    f.journal.replace(j);
    assert.fail("Tampered receipt unexpectedly accepted");
  } catch(error) {
    assert.match(error.message,/journal claims privileged effects|invalid durable journal/);
  }finally{f.cleanup();}
});

test("journal binds to the exact plan and rejects a changed artifact hash", () => {
  const f=setup();try {
    const switched=plan();
    switched.installSteps[1].sha256 = "f".repeat(64);
    assert.throws(()=>inspectTransactionJournal(switched,f.journal),/invalid durable journal identity/);
  } finally {f.cleanup();}
});
test("wrong upstream revision or wrong step order cannot start simulation", async () => {
  const f=setup();try {
    await assert.rejects(simulateProvisionTransaction(
      {...f.plan,upstreamCommit:"0".repeat(40)},f.journal,
      adapter(async()=>({outcome:"CONFIRMED"}))),/driver-install blocker/);
    const bad=plan();bad.installSteps[0].action="DISABLE_SECURITY";
    assert.throws(()=>createTransactionJournal(join(f.base,"untrusted"),bad),/driver-install blocker/);
  }finally {f.cleanup();}
});
