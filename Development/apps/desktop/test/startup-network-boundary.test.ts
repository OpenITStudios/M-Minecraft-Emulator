import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";

describe("Active native-first startup boundary", () => {
  it("starts capture probes only after native backend admission and avoids late startup publication", () => {
    const active = readFileSync(new URL("../src/LazyTesterApp.svelte", import.meta.url), "utf8");
    const startup = active.split("async function initializeLaunchers() {")[1]!.split("async function checkBinding(")[0]!;
    expect(startup.indexOf("nativeBackend === 'Unsupported'")).toBeLessThan(startup.indexOf("lazyTester.screenshotAvailable()"));
    expect(startup.indexOf("nativeBackend === 'Unsupported'")).toBeLessThan(startup.indexOf("lazyTester.recordingAvailable()"));
    expect(active.match(/lazyTester\.screenshotAvailable\(/g)).toHaveLength(1);
    expect(active.match(/lazyTester\.recordingAvailable\(/g)).toHaveLength(1);
    expect(startup).toContain("if (disposed) return;\n        nativeBackend = backend;");
    expect(startup).toContain("if (!disposed) void lazyTester.reportStartup(");
    expect(active).toContain("if (!disposed) { binding = result; bindingIssue = ''; }");
    const mount = active.split("onMount(() => {")[1]!.split("</script>")[0]!;
    expect(mount.indexOf("lazy-tester-close-review")).toBeLessThan(mount.indexOf("void initializeLaunchers()"));
  });

  it("does not start a presentation hook in the diagnostic and validates existing legacy binding bytes once", () => {
    const bootstrap = readFileSync(new URL("../src-tauri/src/app_bootstrap.rs", import.meta.url), "utf8");
    const diagnostic = bootstrap.split("fn check_webview_binding(")[1]!.split("fn create_main_window(")[0]!;
    expect(diagnostic).not.toContain("workspace_foreground::start(");
    expect(diagnostic).not.toContain("workspace_foreground::shutdown(");
    expect(bootstrap).toContain("workspace_foreground::start(window.as_ref().window())?");
    expect(bootstrap).toContain("workspace_foreground::shutdown();");
    expect(bootstrap).toContain("native_clients_session::CONTROLLER_GATE_NAME");
    const binding = readFileSync(new URL("../src-tauri/src/engine/independent_bootstrap/profile_binding.rs", import.meta.url), "utf8");
    expect(binding).toContain("Ok(bytes) => return Ok(binding_status(Ok(bytes), validate))");
    expect(binding).toContain("existing_controller_gate(CONTROLLER_GATE_NAME)");
  });

  it("routes a failed window destroy through existing post-release exit recovery", () => {
    const bootstrap = readFileSync(new URL("../src-tauri/src/app_bootstrap.rs", import.meta.url), "utf8");
    const close = bootstrap.split("let result=commands::native_clients::close_all_managed()")[1]!.split("CONFIRMING.store(false,Ordering::SeqCst);")[0]!;
    expect(close).toContain(".and_then(|()| window.destroy().map_err(");
    expect(close).toContain("(true, std::io::Error::other(");
    expect(close).toContain("if release_attempted");
    expect(close).toContain("workspace_surface::cancel_close()");
    expect(close).toContain('window.emit("lazy-tester-close-error"');
    expect(close).toContain('window.emit("lazy-tester-close-rejected"');
    const shutdown = bootstrap.split("pub fn shutdown() {")[1]!.split("#[test]")[0]!;
    expect(shutdown).toContain("WORKER.lock().ok().and_then(|mut owner| owner.take())");
    expect(shutdown).toContain("join.is_finished() ||");
    expect(shutdown).toContain("join.join().is_err()");
  });

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
    expect(active).toContain("sessionObservationEnabled = mounted && !disposed && managerVisible");
    expect(active).toContain("nativeBackend === 'WindowsNative' && !closeReviewPending && !!binding?.configured");
    expect(active).toContain("!selectiveOpen && storageReview===null");
    expect(active).not.toContain("onMount(() => lazyTester.launch");
    expect(active).not.toContain("checkUpdateOnce");
    expect(bootstrap).not.toContain("commands::virtual_clients::virtual_clients_");
    expect(bootstrap).not.toContain("commands::setup_tools::setup_");
    expect(bootstrap).not.toContain("commands::window_arrangement::window_");
  });

});
