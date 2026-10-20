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
    expect(app.scripts["build:app"]).toContain(" -- --locked");
  });

  it("prevents shipping the native pilot inside the existing VMware NSIS bundle", () => {
    const entry = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
    const packageBuilder = readFileSync(new URL("../scripts/prepare-package-binaries.mjs", import.meta.url), "utf8");
    const policy = readFileSync(new URL("../../../engine/distribution/package-contract.json", import.meta.url), "utf8");
    expect(entry).toContain("UnifiedManagerApp");
    expect(packageBuilder).toContain("PACKAGE_BLOCKED_NATIVE_FEASIBILITY");
    expect(packageBuilder).toContain("const LEGACY_PACKAGE_RELEASE_APPROVED = false;");
    expect(packageBuilder).toContain("if (!LEGACY_PACKAGE_RELEASE_APPROVED)");
    expect(packageBuilder.indexOf("PACKAGE_BLOCKED_NATIVE_FEASIBILITY"))
      .toBeLessThan(packageBuilder.indexOf("const binaries ="));
    expect(policy).toContain("guest/windows/virtual-guest-agent.exe");
    expect(app.scripts["build:app"]).toContain("npm run prepare:package-binaries");
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
