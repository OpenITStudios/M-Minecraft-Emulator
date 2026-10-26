import { existsSync, statSync } from "node:fs";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";

const root = resolve(import.meta.dirname, "..");
const source = resolve(root, "src-tauri", "app-icon.svg");
const outputs = [
  resolve(root, "src-tauri", "icons", "32x32.png"),
  resolve(root, "src-tauri", "icons", "128x128.png"),
  resolve(root, "src-tauri", "icons", "128x128@2x.png"),
  resolve(root, "src-tauri", "icons", "icon.ico")
];
const sourceMtime = statSync(source).mtimeMs;
if (outputs.every((path) => existsSync(path) && statSync(path).mtimeMs >= sourceMtime)) process.exit(0);
const cli = createRequire(import.meta.url).resolve("@tauri-apps/cli/tauri.js");
const result = spawnSync(process.execPath, [cli, "icon", source], {
  cwd: root, stdio: "inherit"
});
if (result.error) throw result.error;
process.exit(result.status ?? 1);
