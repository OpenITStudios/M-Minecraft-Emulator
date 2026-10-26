// Review an existing disposable simulation journal: no Windows mutations,
// no KmdUtil, no service stop/delete, no Tauri route, no account inspection.
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { openTransactionJournal } from "../execute-disposable.mjs";
import {
  appendServiceObservation, inspectOwnershipReview, recordOperatorReviewIntent,
} from "./ownership-ledger.mjs";

function boundedJson(path) {
  const raw = readFileSync(resolve(path));
  if (raw.length > 64 * 1024) throw new Error("Owned review input exceeds size bound");
  return JSON.parse(raw.toString("utf8").replace(/^\uFEFF/, ""));
}
export function processOwnedReviewArgs(args) {
  if (!Array.isArray(args) || args.length < 3) {
    throw new Error("Usage: review-ownership.mjs inspect|acknowledge|observe <plan.json> <journal-directory> [transactionId + phrase | serviceName + read-only-SCM.json]");
  }
  const [operation, planFile, journalDirectory, ...rest] = args;
  const plan = boundedJson(planFile);
  const journal = openTransactionJournal(journalDirectory, plan);
  if (operation === "inspect" && rest.length === 0) {
    return inspectOwnershipReview({plan, journal});
  }
  if (operation === "acknowledge" && rest.length === 2) {
    const [transactionId, acknowledgment] = rest;
    return recordOperatorReviewIntent({plan,journal,request:{
      transactionId,
      planFingerprint: journal.read().planFingerprint,
      action:"REVIEW_OWNED_SERVICE_RECOVERY",
      acknowledgment,
    }});
  }
  if (operation === "observe" && rest.length === 2 &&
    ["SbieDrv","SbieSvc"].includes(rest[0])) {
    return appendServiceObservation({plan,journal,component:rest[0],
      observation:boundedJson(rest[1])});
  }
  throw new Error("Unsupported owned-service review operation or argument count");
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    process.stdout.write(JSON.stringify(processOwnedReviewArgs(process.argv.slice(2)),null,2)+"\n");
  } catch (error) {
    process.stderr.write(error.message+"\n");
    process.exitCode = 1;
  }
}
