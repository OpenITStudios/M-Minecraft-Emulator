import { createHash } from "node:crypto";
import { existsSync, lstatSync, realpathSync, readFileSync } from "node:fs";
import { open } from "node:fs/promises";
import { isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { verifyReport } from "./validate-baseline.mjs";

function demand(ok, message) {
  if (!ok) throw new Error("Artifact verification refused: " + message);
}
function inside(root, target) {
  const rel = relative(root, target);
  return rel !== ".." && !rel.startsWith(".." + sep) && !isAbsolute(rel);
}
function checkPath(root, rel) {
  const target = resolve(root, ...rel.split("/"));
  demand(inside(root, target), "artifact escapes source root");
  let current = root;
  for (const part of rel.split("/")) {
    current = join(current, part);
    // existsSync() follows symlinks and reports dangling links as absent.
    // lstatSync() identifies the directory entry even if its target vanished.
    let entry;
    try {
      entry = lstatSync(current);
    } catch (error) {
      if (error.code === "ENOENT") break;
      throw error;
    }
    demand(!entry.isSymbolicLink(), "symlinked component: " + rel);
  }
  if (existsSync(target)) {
    demand(inside(root, realpathSync(target)), "resolved artifact escapes source root");
  }
  return target;
}

// Verify actual bytes; a self-reported SHA-256 string alone is not file evidence.
export async function verifyArtifactFiles(manifest, report, sourceRoot) {
  verifyReport(manifest, report);
  demand(typeof sourceRoot === "string" && sourceRoot.length > 0, "missing source root");
  const root = realpathSync(sourceRoot);
  demand(lstatSync(root).isDirectory(), "source root not a directory");
  let checked = 0;
  let absent = 0;
  for (const item of report.artifacts) {
    const path = checkPath(root, item.path);
    if (item.state === "MISSING") {
      demand(!existsSync(path), "reported missing artifact exists: " + item.path);
      absent++;
      continue;
    }
    demand(existsSync(path) && lstatSync(path).isFile(), "reported present artifact missing: " + item.path);
    const handle = await open(path, "r");
    try {
      const before = await handle.stat({ bigint: true });
      demand(before.isFile() && before.size === BigInt(item.sizeBytes), "file size differs: " + item.path);
      const hash = createHash("sha256");
      for await (const chunk of handle.createReadStream({ autoClose: false })) hash.update(chunk);
      const after = await handle.stat({ bigint: true });
      demand(before.dev === after.dev && before.ino === after.ino &&
        before.size === after.size && before.mtimeNs === after.mtimeNs,
        "file changed during verification: " + item.path);
      demand(hash.digest("hex") === item.sha256, "SHA-256 differs: " + item.path);
      checked++;
    } finally {
      await handle.close();
    }
  }
  return {
    verification: "FILE_CONTENTS_MATCH_REPORTED_HASHES",
    checkedPresent: checked, checkedAbsent: absent,
    driverLoadVerified: false, gameplayVerified: false, cutoverApproved: false,
  };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2);
    demand(args.length === 3, "usage: node verify-artifact-files.mjs <manifest.json> <report.json> <source-root>");
    const parse = path => JSON.parse(readFileSync(resolve(path), "utf8").replace(/^\uFEFF/, ""));
    const result = await verifyArtifactFiles(parse(args[0]), parse(args[1]), args[2]);
    process.stdout.write(JSON.stringify(result, null, 2) + "\n");
  } catch (error) {
    process.stderr.write(error.message + "\n");
    process.exitCode = 1;
  }
}
