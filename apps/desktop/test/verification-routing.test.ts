import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";

const root = JSON.parse(readFileSync(new URL("../../../package.json", import.meta.url), "utf8"));
const app = JSON.parse(readFileSync(new URL("../package.json", import.meta.url), "utf8"));
const router = readFileSync(new URL("../../../tooling/windows-toolchain/dev.ps1", import.meta.url), "utf8");

describe("Emulator source verification routing", () => {
  it("has exactly one repository-to-desktop-to-runtime verification route", () => {
    expect(root.scripts["verify:source"].split(" && ")).toEqual([
      "npm run verify:repository -- --g0",
      "npm run check:structure",
      "npm run check:dependencies",
      "npm run test:repository",
      "npm run test:localtest",
      "npm run test:feasibility",
      "npm run test:isolation-source",
      "npm run test:native-feasibility",
      "npm run test:sandboxie-feasibility",
      "npm run test:native-performance",
      "npm run test:presentmon",
      "npm run check:desktop",
      "npm run check:runtime",
      "npm run test:runtime",
      "npm run format:runtime",
    ]);
    expect(root.scripts["check:desktop"]).toBe("npm --prefix apps/desktop run verify:source");
    expect(root.scripts["check:runtime"]).toContain("engine/runtime-core/Cargo.toml");
    expect(root.scripts["test:runtime"]).toContain("engine/runtime-core/Cargo.toml");
  });

  it("runs desktop tests and source compilation checks without packaging", () => {
    expect(app.scripts["verify:source"]).toBe(
      "npm run typecheck && npm run test && npm run build:frontend && npm run check:rust && npm run test:rust && npm run format:rust",
    );
    expect(app.scripts["check:rust"]).toContain("npm run prepare:icons && cargo check");
    expect(app.scripts["test:rust"]).toContain("cargo test --all-targets");
    expect(app.scripts["format:rust"]).toBe("cargo fmt --all --check --manifest-path src-tauri/Cargo.toml");
    expect(root.scripts["verify:source"]).not.toMatch(/build:app|dev:app|vmrun/);
    expect(app.scripts["check:rust"]).toContain(" --locked ");
    expect(app.scripts["test:rust"]).toContain(" --locked ");
    expect(app.scripts["build:app"]).toBe("npm run build:lazy-tester");
    expect(root.scripts["package:windows"]).toBe("node apps/desktop/scripts/package-lazy-tester.mjs");
    expect(app.scripts["build:lazy-tester"]).toContain(" -- --locked");
  });

  it("uses the product route for normal development and keeps private preview explicit", () => {
    expect(root.scripts.dev).toBe("npm --prefix apps/desktop run dev:app");
    expect(app.scripts["dev:app"]).toBe("npm run dev:lazy-tester");
    expect(app.scripts["dev:lazy-tester"]).toBe(app.scripts["build:lazy-tester"].replace(
      "tauri build --target x86_64-pc-windows-msvc --bundles nsis",
      "tauri dev --target x86_64-pc-windows-msvc",
    ));
    expect(app.scripts["dev:preview"]).toContain("--config src-tauri/tauri.native-preview.conf.json --features native-workspace-preview -- --locked");
    expect(app.scripts["dev:preview"]).not.toContain("--features lazy-tester-package");
  });

  it("prevents shipping the native pilot inside the existing VMware NSIS bundle", () => {
    const entry = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
    const packageBuilder = readFileSync(new URL("../scripts/prepare-package-binaries.mjs", import.meta.url), "utf8");
    const policy = readFileSync(new URL("../../../engine/distribution/package-contract.json", import.meta.url), "utf8");
    expect(entry).toContain("LazyTesterApp");
    expect(packageBuilder).toContain("PACKAGE_BLOCKED_NATIVE_FEASIBILITY");
    expect(packageBuilder).toContain("const LEGACY_PACKAGE_RELEASE_APPROVED = false;");
    expect(packageBuilder).toContain("if (!LEGACY_PACKAGE_RELEASE_APPROVED)");
    expect(packageBuilder.indexOf("PACKAGE_BLOCKED_NATIVE_FEASIBILITY"))
      .toBeLessThan(packageBuilder.indexOf("const binaries ="));
    expect(policy).toContain("guest/windows/virtual-guest-agent.exe");
    expect(app.scripts["build:lazy-tester"]).not.toContain("prepare:package-binaries");
    expect(app.scripts["build:lazy-tester"]).toContain("--config src-tauri/tauri.lazy-tester.conf.json");
  });

  it("native preflight rejects unsupported Windows hosts and does not package legacy VM resources", () => {
    const preflight = router.split("function Invoke-NativeSourcePreflight {")[1]
      ?.split("# One opt-in source/package checkpoint.")[0] ?? "";
    expect(router).toContain("function Assert-NativePilotHost");
    expect(preflight).toContain("Assert-NativePilotHost");
    expect(preflight).toContain("Assert-CleanSourceAtHead -ExpectedSha $sha");
    expect(preflight).toContain('Invoke-RepositoryNpm -NpmArguments @("run", "verify:source")');
    expect(preflight).not.toContain('Invoke-RepositoryNpm -NpmArguments @("run", "package:windows")');
    expect(router).toContain('"preflight" { Invoke-NativeSourcePreflight }');
  });

  it("routes the optional Windows entrypoint through source checks", () => {
    expect(router).toContain("Set-LocalDeveloperEnvironment -Root $Root");
    expect(router).toContain("& $LocalNpm @NpmArguments");
    expect(router).not.toContain("& npm @NpmArguments");
    expect(router).toMatch(/"check"\s*\{ Invoke-RepositoryNpm -NpmArguments @\("run", "verify:source"\) \}/);
    expect(router).toMatch(/"setup"\s*\{ Invoke-RepositoryNpm -NpmArguments @\("run", "setup"\) \}/);
    expect(router).not.toMatch(/virtual-clients:verify|M-Bedrock-Dev/);
  });
});


describe("Native-only Windows preview package", () => {
  it("isolates preview identity and payload from the retired VMware installer", async () => {
    const {verifyNativePreviewPackage} = await import("../scripts/check-native-preview-package.mjs");
    const actual = verifyNativePreviewPackage();
    expect(actual.identifier).toBe("com.mivubi.minecraft.instancemanager.preview");
    expect(actual.config.mainBinaryName).toBe("minecraft-instance-manager-preview");
    expect(actual.installMode).toBe("currentUser");
    expect(actual.config.bundle.resources).toEqual({});
    expect(actual.config.bundle.externalBin).toEqual([]);
    expect(actual.config.bundle.windows.nsis).not.toHaveProperty("installerHooks");
    expect(actual.config.bundle.windows.nsis.startMenuFolder).toBe("Minecraft Instance Manager Preview");
    expect(actual.config.app.security.csp).toContain("object-src 'none'");
    expect(actual.config.app.windows[0].title).toBe("Lazy Tester");
  });
  it("uses RFC 7396 deletions and preserves the original package hold", async () => {
    const {mergePatch} = await import("../scripts/check-native-preview-package.mjs");
    expect(mergePatch({a:{keep:"yes",old:"VMware"},b:[1,2]},
      {a:{old:null},b:[]})).toEqual({a:{keep:"yes"},b:[]});
    const legacy=readFileSync(new URL("../scripts/prepare-package-binaries.mjs",import.meta.url),"utf8");
    expect(legacy).toContain("PACKAGE_BLOCKED_NATIVE_FEASIBILITY");
    expect(legacy).toContain("LEGACY_PACKAGE_RELEASE_APPROVED = false");
    const build=readFileSync(new URL("../src-tauri/build.rs",import.meta.url),"utf8");
    expect(build).toContain("CARGO_FEATURE_NATIVE_WORKSPACE_PREVIEW");
    expect(build).toContain("Legacy Windows release packaging is blocked");
    const helper=readFileSync(new URL("../src-tauri/src/engine/workspace_surface.rs",import.meta.url),"utf8");
    expect(helper).toContain('cfg!(feature = "native-workspace-preview")');
    expect(helper).toContain("native::trusted_parent()");
    expect(helper).toContain("if !workspace_enabled()");
  });
});
