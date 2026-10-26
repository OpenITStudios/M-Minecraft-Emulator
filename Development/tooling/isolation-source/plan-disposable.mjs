// Pure plan delivery for a future disposable-host provisioning executor.
// This CLI NEVER installs Windows services or touches a sandbox or game.
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createDisposableProvisionPlan } from "./provision-disposable.mjs";

function readBounded(path) {
  const raw = readFileSync(resolve(path));
  if (raw.length > 256 * 1024) {
    throw new Error("Owned isolation planner input exceeded bounded size");
  }
  return JSON.parse(raw.toString("utf8").replace(/^\uFEFF/, ""));
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2);
    if (args.length !== 6 || args[0] !== "--payload" ||
        args[2] !== "--host" || args[4] !== "--target") {
      throw new Error("Usage: node plan-disposable.mjs --payload <engine-provenance.json> --host <read-only-host.json> --target <disposable-target.json>");
    }
    const result = createDisposableProvisionPlan({
      payload: readBounded(args[1]),
      host: readBounded(args[3]),
      target: readBounded(args[5]),
    });
    process.stdout.write(JSON.stringify(result, null, 2) + "\n");
  } catch (error) {
    process.stderr.write(error.message + "\n");
    process.exitCode = 1;
  }
}
