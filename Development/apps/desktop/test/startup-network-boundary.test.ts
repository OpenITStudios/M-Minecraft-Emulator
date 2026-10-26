import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";

describe("Active native-first startup boundary", () => {
  it("mounts Lazy Tester without querying VMware, Guest Agent or update APIs", () => {
    const entry = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
    const active = readFileSync(new URL("../src/LazyTesterApp.svelte", import.meta.url), "utf8");
    const bootstrap = readFileSync(new URL("../src-tauri/src/app_bootstrap.rs", import.meta.url), "utf8");
    expect(entry).toContain('import LazyTesterApp from "./LazyTesterApp.svelte"');
    expect(entry).toContain("mount(LazyTesterApp, { target });");
    expect(active).toContain("lazyTester.inspect()");
    expect(active).not.toContain("OwnedManagerPanel");
    expect(active).not.toContain("ownedProvider.execute(");
    expect(active).not.toContain("backend.");
    expect(active).toContain("if (!document.hidden && binding?.configured) void refresh()");
    expect(active).not.toContain("onMount(() => lazyTester.launch");
    expect(active).not.toContain("checkUpdateOnce");
    expect(bootstrap).not.toContain("commands::virtual_clients::virtual_clients_");
    expect(bootstrap).not.toContain("commands::setup_tools::setup_");
    expect(bootstrap).not.toContain("commands::window_arrangement::window_");
  });

});
