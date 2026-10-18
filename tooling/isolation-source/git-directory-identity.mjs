// Compare physical Git directory identity when Git-for-Windows reports an
// NTFS 8.3 alias instead of Node's absolute path spelling.
import { lstatSync, statSync } from "node:fs";
import { resolve } from "node:path";

export function sameGitDirectory(first, second) {
  if (typeof first !== "string" || typeof second !== "string" ||
      !first.trim() || !second.trim()) return false;
  const a = resolve(first), b = resolve(second);
  try {
    const left = lstatSync(a), right = lstatSync(b);
    if (!left.isDirectory() || !right.isDirectory() ||
        left.isSymbolicLink() || right.isSymbolicLink()) return false;
    if (a === b || (process.platform === "win32" &&
        a.toLowerCase() === b.toLowerCase())) return true;
    if (process.platform !== "win32") return false;
    const l = statSync(a), r = statSync(b);
    return l.dev === r.dev && l.ino !== 0 && l.ino === r.ino;
  } catch {
    return false;
  }
}
