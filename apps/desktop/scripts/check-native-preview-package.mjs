// Source-level package safety check; does NOT build, sign, install or test
// Minecraft. Tauri applies config overlays using RFC 7396 JSON Merge Patch.
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const appRoot = resolve(fileURLToPath(new URL("..", import.meta.url)));
const source = (path) => readFileSync(resolve(appRoot, path), "utf8");

export function mergePatch(document, patch) {
  if (patch === null || typeof patch !== "object" || Array.isArray(patch)) {
    return structuredClone(patch);
  }
  const merged =
    document && typeof document === "object" && !Array.isArray(document)
      ? structuredClone(document) : {};
  for (const [key, value] of Object.entries(patch)) {
    if (value === null) delete merged[key];
    else merged[key] = mergePatch(merged[key], value);
  }
  return merged;
}

export function verifyNativePreviewPackage() {
  const base = JSON.parse(source("src-tauri/tauri.conf.json"));
  const native = JSON.parse(source("src-tauri/tauri.native-preview.conf.json"));
  const current = mergePatch(base, native);
  const scripts = JSON.parse(source("package.json")).scripts;
  const cargo = source("src-tauri/Cargo.toml");
  const build = source("src-tauri/build.rs");
  const surface = source("src-tauri/src/engine/workspace_surface.rs");
  const legacy = source("scripts/prepare-package-binaries.mjs");
  const fail = (condition, reason) => {
    if (!condition) throw new Error("NATIVE_PREVIEW_PACKAGE_REFUSED: " + reason);
  };

  fail(base.identifier === "com.mivubi.mbedrock.virtualclients",
    "The legacy installed app identity changed unexpectedly.");
  fail(base.identifier !== current.identifier,
    "Preview must never replace or overwrite the legacy app identity.");
  fail(current.identifier === "com.mivubi.minecraft.instancemanager.preview",
    "Native preview identifier must be fixed and independently reviewable.");
  fail(current.productName === "Minecraft Instance Manager Preview",
    "Native preview product name is unexpected.");
  fail(current.mainBinaryName === "minecraft-instance-manager-preview",
    "Native preview must have an executable filename distinct from the old VM binary.");
  fail(current.bundle?.windows?.nsis?.installMode === "currentUser",
    "Preview must install only for the current user.");
  fail(!Object.hasOwn(current.bundle.windows.nsis, "installerHooks"),
    "Inherited VMware installer hooks are forbidden.");
  fail(Object.keys(current.bundle?.resources ?? {}).length === 0,
    "VMware, Guest Agent, acceptance and virtual-client payloads must not be packaged.");
  fail(Array.isArray(current.bundle?.externalBin) && current.bundle.externalBin.length === 0,
    "Preview must not bundle the legacy virtual-clients sidecar.");
  fail(Object.keys(native.bundle?.resources ?? {}).length === Object.keys(base.bundle?.resources ?? {}).length &&
    Object.keys(base.bundle.resources).every(key => native.bundle.resources[key] === null),
    "Every inherited VMware resource must be explicitly removed.");
  fail(native.bundle.windows.nsis.installerHooks === null,
    "Preview configuration must explicitly delete legacy installer hooks.");
  fail(!Object.hasOwn(native, "plugins") && current.app?.security?.csp === base.app?.security?.csp,
    "Preview must not silently broaden desktop security permissions.");
  fail(scripts["build:app"].includes("prepare:package-binaries") &&
      legacy.includes("PACKAGE_BLOCKED_NATIVE_FEASIBILITY") &&
      legacy.includes("LEGACY_PACKAGE_RELEASE_APPROVED = false"),
    "Original legacy distribution safety hold must remain enabled.");
  const nativeCommand = scripts["build:preview"];
  fail(typeof nativeCommand === "string" &&
      nativeCommand.includes("check:preview-package") &&
      nativeCommand.includes("--bundles nsis") &&
      nativeCommand.includes("--config src-tauri/tauri.native-preview.conf.json") &&
      nativeCommand.includes("--features native-workspace-preview") &&
      nativeCommand.includes("--target x86_64-pc-windows-msvc") &&
      nativeCommand.includes("-- --locked") &&
      !nativeCommand.includes("prepare:package-binaries"),
    "Preview command must use the dedicated configuration, feature and locked build.");
  fail(cargo.includes("native-workspace-preview = []") &&
      !cargo.includes('default = ["custom-protocol", "native-workspace-preview"]'),
    "Preview helper feature must be opt-in, never default.");
  fail(build.includes('CARGO_FEATURE_NATIVE_WORKSPACE_PREVIEW') &&
      build.includes('Some("com.mivubi.minecraft.instancemanager.preview")') &&
      build.includes('Some("minecraft-instance-manager-preview")') &&
      build.includes('patch.pointer("/bundle/resources")') &&
      build.includes('patch.pointer("/bundle/windows/nsis/installerHooks")') &&
      build.includes('Legacy Windows release packaging is blocked'),
    "Rust build-time preview identity and legacy packaging gates must remain bound.");
  fail(surface.includes('cfg!(feature = "native-workspace-preview")') &&
      surface.includes("native::trusted_parent()") &&
      surface.includes('if !workspace_enabled()'),
    "Release window helper must be enabled only for the isolated native preview and validate its parent.");
  fail(!JSON.stringify(current).includes("virtual-guest-agent") &&
      !JSON.stringify(current).includes("guest/windows"),
    "Resolved native bundle still references retired Guest Agent resources.");

  return { identifier: current.identifier, resources: 0, sidecars: 0,
    installMode: current.bundle.windows.nsis.installMode, config: current };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const result = verifyNativePreviewPackage();
  process.stdout.write(
    "Native-only preview package source contract verified: " +
    result.identifier + " (no VMware resources, guest binaries or legacy installer hooks)\n",
  );
}
