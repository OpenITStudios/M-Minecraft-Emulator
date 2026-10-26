// Developer-only simulation CLI. Never launches privileged Windows utilities.
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  createTransactionJournal, simulateProvisionTransaction,
} from "./execute-disposable.mjs";

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2);
    if (args.length !== 4 || args[0] !== "--plan" || args[2] !== "--journal-directory") {
      throw new Error("Usage: node simulate-disposable.mjs --plan <plan.json> --journal-directory <new-empty-directory>");
    }
    const planBuffer = readFileSync(resolve(args[1]));
    if (planBuffer.length > 64 * 1024) throw new Error("Provisioning plan exceeds size limit");
    const plan = JSON.parse(planBuffer.toString("utf8").replace(/^\uFEFF/, ""));
    const journal = createTransactionJournal(args[3], plan);
    const adapter = {
      kind: "IN_MEMORY_SIMULATION",
      execute: async () => ({ outcome: "CONFIRMED" }),
    };
    const result = await simulateProvisionTransaction(plan, journal, adapter);
    process.stdout.write(JSON.stringify(result, null, 2) + "\n");
  } catch (error) {
    process.stderr.write(error.message + "\n");
    process.exitCode = 1;
  }
}
