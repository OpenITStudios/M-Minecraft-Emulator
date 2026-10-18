import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { parseOwnedCandidateInspection, parseOwnedHostInspection } from "../src/app/bridge/nativeFeasibilityApi.js";

const candidate = () => ({
  schema: 1,
  state: "CANDIDATE_INTEGRITY_VERIFIED_NOT_INSTALLABLE",
  upstreamCommit: "abd4c240031cfb46666daa74ca47ed4e48b561e4",
  verifiedArtifactCount: 24,
  requiredArtifactCount: 24,
  providerSelected: false,
  driverLoadVerified: false,
  gameplayVerified: false,
  installPermitted: false,
});

describe("Owned isolation candidate — strictly read-only", () => {
  it("accepts staged artifact integrity without claiming readiness", () => {
    expect(parseOwnedCandidateInspection(candidate())).toEqual(candidate());
  });

  it("rejects inflated readiness, altered source and incomplete assets", () => {
    const overrides = [
      { providerSelected: true }, { driverLoadVerified: true },
      { gameplayVerified: true }, { installPermitted: true },
      { verifiedArtifactCount: 23 }, { verifiedArtifactCount: 49 },
      { requiredArtifactCount: 23 },
      { requiredArtifactCount: 16 }, { schema: 2 },
      { state: "INSTALLED_AND_READY" }, { upstreamCommit: "0".repeat(40) },
      { surpriseEmail: "private@example.invalid" },
    ];
    for (const override of overrides) {
      expect(() => parseOwnedCandidateInspection({ ...candidate(), ...override })).toThrow();
    }
  });

  it("exposes one read-only desktop route without switching active backend", () => {
    const engine = readFileSync(new URL("../src-tauri/src/engine/native_clients.rs", import.meta.url), "utf8");
    const owned = readFileSync(new URL("../src-tauri/src/engine/owned_isolation.rs", import.meta.url), "utf8");
    const bootstrap = readFileSync(new URL("../src-tauri/src/app_bootstrap.rs", import.meta.url), "utf8");
    const commands = readFileSync(new URL("../src-tauri/src/commands/native_clients.rs", import.meta.url), "utf8");
    const screen = readFileSync(new URL("../src/CompactManagerApp.svelte", import.meta.url), "utf8");
    expect(engine).toContain("const ACTIVE_ISOLATION_BACKEND: OfficialSandboxieBackend");
    expect(engine).toContain("inspect_owned_engine_candidate(");
    expect(owned).toContain("CANDIDATE_INTEGRITY_VERIFIED_NOT_INSTALLABLE");
    expect(owned).toContain("driver_load_verified: false");
    expect(owned).toContain("install_permitted: false");
    expect(owned).toContain("LICENSES/SandboxiePlus/QSbieAPI/LICENSE");
    expect(owned).toContain("expected_source_for_payload");
    expect(owned).toContain("expected_not_installable_marker");
    expect(owned).toContain("verify_payload_inventory(root, &declared)?");
    expect(owned).toContain("FILE_ATTRIBUTE_REPARSE_POINT");
    expect(owned).toContain("owned-engine payload entry cap exceeded");
    expect(owned).toContain("owned-engine metadata changed while it was read");
    expect(bootstrap).toContain("commands::native_clients::desktop_inspect_owned_engine_candidate,");
    expect(bootstrap).toContain("commands::native_clients::desktop_select_owned_engine_candidate_directory,");
    expect(commands).toContain("fn desktop_inspect_owned_engine_candidate(");
    expect(screen).toContain("Inspect engine candidate");
    expect(screen).toContain("No engine was activated.");
    expect(screen).toContain("Engine package ");
    expect(screen).toContain("checksums match");
    expect(screen).toContain("Files could change after inspection.");
  });
});

describe("Owned engine host service registration", () => {
  const host = () => ({
    schema: 1,
    interactiveSession: true,
    state: "BLOCKED_SERVICE_NAMESPACE_IN_USE",
    serviceRegistration: "PRESENT",
    driverRegistration: "PRESENT",
    serviceNamespaceAvailable: false,
    providerSelected: false,
    trustedDriverLoaded: false,
    gameplayVerified: false,
    activationAllowed: false,
  });
  it("reports a service collision without authorizing launch", () => {
    expect(parseOwnedHostInspection(host())).toEqual(host());
  });
  it("accepts free names only when still blocked for unverified driver", () => {
    const empty = { ...host(), state: "BLOCKED_UNVERIFIED_OWNED_DRIVER",
      serviceRegistration: "ABSENT", driverRegistration: "ABSENT",
      serviceNamespaceAvailable: true };
    expect(parseOwnedHostInspection(empty)).toEqual(empty);
    expect(() => parseOwnedHostInspection({ ...empty, activationAllowed: true })).toThrow();
  });
  it("accepts a noninteractive result only with matching evidence", () => {
    const blocked = { ...host(), interactiveSession: false,
      state: "BLOCKED_NON_INTERACTIVE_SESSION" };
    expect(parseOwnedHostInspection(blocked)).toEqual(blocked);
  });
  it("rejects contradictory and private runtime claims", () => {
    for (const change of [
      { serviceNamespaceAvailable: true },
      { serviceRegistration: "RUNNING" },
      { trustedDriverLoaded: true },
      { providerSelected: true },
      { gameplayVerified: true },
      { state: "ACTIVE" },
      { state: "BLOCKED_UNVERIFIED_OWNED_DRIVER" },
      { interactiveSession: false },
      { userAccount: "private@example.invalid" },
    ]) {
      expect(() => parseOwnedHostInspection({ ...host(), ...change })).toThrow();
    }
  });
});
