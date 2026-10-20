import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";

describe("Active native-first startup boundary", () => {
  it("mounts the Unified Manager without querying VMware, Guest Agent or update APIs", () => {
    const entry = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
    const active = readFileSync(new URL("../src/UnifiedManagerApp.svelte", import.meta.url), "utf8");
    const bootstrap = readFileSync(new URL("../src-tauri/src/app_bootstrap.rs", import.meta.url), "utf8");
    expect(entry).toContain('import UnifiedManagerApp from "./UnifiedManagerApp.svelte"');
    expect(entry).toContain("mount(UnifiedManagerApp, { target });");
    expect(active).toContain("managerConfiguration.load()");
    expect(active).toContain("OwnedManagerPanel");
    expect(readFileSync(new URL('../src/OwnedManagerPanel.svelte', import.meta.url), 'utf8')).toContain("ownedProvider.execute(");
    expect(active).not.toContain("backend.");
    expect(active).not.toContain("setInterval(");
    expect(active).not.toContain("checkUpdateOnce");
    expect(bootstrap).not.toContain("commands::virtual_clients::virtual_clients_");
    expect(bootstrap).not.toContain("commands::setup_tools::setup_");
    expect(bootstrap).not.toContain("commands::window_arrangement::window_");
  });

  it("keeps archived VM diagnostics from polling update checks in refresh", () => {
    const source = readFileSync(new URL("../src/App.svelte", import.meta.url), "utf8");
    const refresh = source.slice(source.indexOf("async function refresh"), source.indexOf("async function checkUpdateOnce"));
    expect(refresh).not.toContain("backend.checkUpdate()");
  });
});
