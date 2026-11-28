import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";

describe("Active native-first startup boundary", () => {
  it("uses one native capability request and reports DOM only in diagnostic mode", () => {
    const active = readFileSync(new URL("../src/LazyTesterApp.svelte", import.meta.url), "utf8");
    const startup = active.split("async function initializeLaunchers() {")[1]!.split("async function checkBinding(")[0]!;
    expect(active.match(/lazyTester\.startupCapabilities\(/g)).toHaveLength(1);
    expect(active).not.toMatch(/lazyTester\.(?:screenshotAvailable|recordingAvailable|backendSupport)\(/);
    expect(startup).toContain("if (disposed) return;\n        nativeBackend = capabilities.backend;");
    expect(startup.indexOf("nativeBackend === 'Unsupported'")).toBeLessThan(startup.indexOf("if(recordingAvailable) void refreshRecordingStatus()"));
    expect(startup).toContain("if (diagnostic) {\n          await tick();\n          if (!disposed) void lazyTester.reportStartup(");
    expect(startup.match(/await loadBinding\(\)/g)).toHaveLength(1);
    expect(startup.match(/await refresh\(\)/g)).toHaveLength(1);
    expect(startup).not.toMatch(/setTimeout|for\s*\(let attempt/);
    expect(active).toContain("if (!binding?.configured) await loadBinding();");
    expect(active).toContain("Retry connection");
    const bootstrap = readFileSync(new URL("../src-tauri/src/app_bootstrap.rs", import.meta.url), "utf8");
    const normal = bootstrap.split(".invoke_handler(tauri::generate_handler![")[1]!.split("])")[0]!;
    const diagnostic = bootstrap.split(".invoke_handler(tauri::generate_handler![")[2]!.split("])")[0]!;
    expect(normal).not.toContain("startup_diagnostic_result");
    expect(diagnostic).toContain("startup_diagnostic_result");
    expect(normal).not.toMatch(/lazy_tester_(?:screenshot|recording)_available/);
    expect(bootstrap).toContain('if !startup_diagnostic_mode() { return Err("Startup diagnostic is not active".into()); }');
    expect(active).toContain("if (!disposed) { binding = result; bindingIssue = result.issue ?? ''; }");
    const mount = active.split("onMount(() => {")[1]!.split("</script>")[0]!;
    expect(mount.indexOf("lazy-tester-close-review")).toBeLessThan(mount.indexOf("void initializeLaunchers()"));
  });

  it("does not start a presentation hook in the diagnostic or admit historical profile data", () => {
    const bootstrap = readFileSync(new URL("../src-tauri/src/app_bootstrap.rs", import.meta.url), "utf8");
    const diagnostic = bootstrap.split("fn check_webview_binding(")[1]!.split("fn create_main_window(")[0]!;
    expect(diagnostic).not.toContain("workspace_foreground::start(");
    expect(diagnostic).not.toContain("workspace_foreground::shutdown(");
    expect(bootstrap).toContain("workspace_foreground::start(window.as_ref().window())?");
    expect(bootstrap).toContain("workspace_foreground::shutdown();");
    expect(bootstrap).toContain("native_clients_session::CONTROLLER_GATE_NAME");
    const binding = readFileSync(new URL("../src-tauri/src/engine/independent_bootstrap/profile_binding.rs", import.meta.url), "utf8");
    const inspection = binding.split("fn inspect_existing() -> ProfileBinding {")[1]!.split("pub(crate) fn run(")[0]!;
    expect(inspection).toContain("product_profiles::inspect_existing()");
    expect(inspection).not.toMatch(/migration_sources\(|discover_reviewed\(|publish\(/);
    expect(binding).toContain("existing_controller_gate(CONTROLLER_GATE_NAME)");
  });

  it("keeps startup diagnostic read-only and admits only the canonical catalog outside migration", () => {
    const native = readFileSync(new URL("../src-tauri/src/engine/native_clients.rs", import.meta.url), "utf8");
    const binding = native.split("pub fn profile_binding(import: bool)")[1]!.split("#[cfg(")[0]!;
    expect(binding.indexOf("startup_diagnostic_mode()")).toBeLessThan(binding.indexOf("profile_binding::run(import)"));
    expect(binding).toContain("profile_binding::inspect_existing()");
    expect(binding).toContain("PermissionDenied");
    expect(native).toContain("product_profiles::location(action, !crate::app_bootstrap::startup_diagnostic_mode())");
    const profiles = readFileSync(new URL("../src-tauri/src/engine/independent_bootstrap/product_profiles.rs", import.meta.url), "utf8");
    const location = profiles.split("pub(crate) fn location(")[1]!.split("#[cfg(test)]")[0]!;
    expect(location).toContain("if !persist_location && !matches!(action, DataLocationAction::Inspect)");
    expect(location).toContain("open_mode(&home, persist_location)?");
    expect(location).toContain("None => (None, Vec::new())");
    expect(location).not.toMatch(/registered_(?:replay_)?slots|migration_sources/);
    const controller = readFileSync(new URL("../src-tauri/src/engine/independent_bootstrap/session_controller.rs", import.meta.url), "utf8");
    const prepare = controller.split("fn prepare() -> io::Result<Self> {")[1]!.split("fn reap(")[0]!;
    expect(prepare).toContain('!cfg!(feature = "lazy-tester-package")');
    expect(prepare).toContain("product_profiles::open()?");
    expect(prepare).not.toMatch(/registered_(?:replay_)?slots|migration_sources/);
    expect(profiles).toContain("open_catalog(logical, root, 2)");
    const migration = profiles.split("fn migrate_reviewed() -> io::Result<usize> {")[1]!.split("fn publish_migrated_location(")[0]!;
    expect(migration).toContain("open_catalog(logical, &selected.0, 1)?");
    expect(migration).toContain("profile_binding::migration_sources()?");
    expect(migration.indexOf("acquire_startup_gate(")).toBeLessThan(migration.indexOf("migrate_sources(&target"));
    expect(migration.indexOf("migrate_sources(&target")).toBeLessThan(migration.indexOf("publish_migrated_location("));
    const publish = profiles.split("fn publish_migrated_location(")[1]!.split("fn migrate_sources(")[0]!;
    expect(publish.indexOf("let _stores = open_at(logical, target)?;")).toBeGreaterThanOrEqual(0);
    expect(publish.indexOf("let _stores = open_at(logical, target)?;")).toBeLessThan(publish.indexOf("write_json("));
    expect(migration).toContain("Ok(_) => return Ok(0)");
    expect(migration).toContain("Ok(sources.len())");
    expect(profiles).not.toContain("replay_roots()?");
    expect(profiles).toContain("logical_appdata_root()?");
    const handles = readFileSync(new URL("../src-tauri/src/engine/independent_bootstrap/profile_storage_handles.rs", import.meta.url), "utf8");
    expect(handles).toContain("#[cfg(test)]\npub(super) fn create_private_slot(");
    expect(handles).toContain("#[cfg(test)]\npub(super) fn create_private_slot_at(");
    expect(handles).toContain("fn logical_appdata_root()");
    const bootstrap = readFileSync(new URL("../src-tauri/src/engine/independent_bootstrap.rs", import.meta.url), "utf8");
    expect(bootstrap).toContain('"migratedProfiles":migrated_profiles');
    expect(bootstrap).not.toContain('"migratedProfiles":5');
    const bindSource = readFileSync(new URL("../src-tauri/src/engine/independent_bootstrap/profile_binding.rs", import.meta.url), "utf8");
    const normal = bindSource.split("pub(crate) fn run(")[1]!.split("// Explicit prior-launcher selection")[0]!;
    expect(normal).toContain("if has_legacy_source()? { return Ok(migration_required()); }");
    expect(normal).not.toMatch(/migration_sources\(|discover_reviewed\(/);
    const replaySource = readFileSync(new URL("../src-tauri/src/engine/independent_bootstrap/reviewed_profile_replay.rs", import.meta.url), "utf8");
    expect(replaySource).toContain("#[cfg(test)]\npub(super) fn registered_replay_slots");
    const mounted = readFileSync(new URL("../src/LazyTesterApp.svelte", import.meta.url), "utf8");
    const locationUi = mounted.split("async function loadLocation(")[1]!.split("async function openFolder(")[0]!;
    expect(locationUi).toContain("if((action==='Locate'||action==='Choose')&&dataLocation.path!==null)await checkBinding();");
  });

  it("refuses retired multi-child startup modes before native dispatch, retaining the controller journal", () => {
    const bootstrap = readFileSync(new URL("../src-tauri/src/engine/independent_bootstrap.rs", import.meta.url), "utf8");
    const entry = bootstrap.split("pub(super) fn maybe_run_native_entrypoint() -> bool {")[1]!.split("fn acquire_startup_gate")[0]!;
    const refusal = entry.indexOf("LEGACY_STARTUP_RETIRED");
    expect(refusal).toBeGreaterThanOrEqual(0);
    for (const flag of ["--education-startup-observation-once", "--education-five-startup-observation-once", "--education-prepare-five-profiles-once"])
      expect(entry.slice(0, refusal)).toContain(flag);
    expect(entry.indexOf("std::process::exit(2);")).toBeLessThan(entry.indexOf("product_profiles::migrate_reviewed"));
    expect(entry).toContain("session_controller::maybe_run()");
    expect(entry).not.toContain("legacy_startup_observation::maybe_run()");
    const journal = readFileSync(new URL("../src-tauri/src/engine/independent_bootstrap/legacy_startup_observation.rs", import.meta.url), "utf8");
    expect(journal).not.toContain("fn run_startup_observation(");
    expect(journal).toContain("#[cfg(test)]\npub(super) fn observe_until_exit");
    expect(journal).toContain("#[cfg(test)]\npub(super) fn prepare_registered_enrolled_for");
    expect(journal).toContain("pub(super) fn save_after_exit");
    const replay = readFileSync(new URL("../src-tauri/src/engine/independent_bootstrap/reviewed_profile_replay.rs", import.meta.url), "utf8");
    expect(replay).not.toContain("fn prepare_five_profiles(");
    const controller = readFileSync(new URL("../src-tauri/src/engine/independent_bootstrap/session_controller.rs", import.meta.url), "utf8");
    expect(controller).toContain("started.save_after_exit(1)");
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
    expect(active).toContain("supportsManagedSessions(nativeBackend) && !closeReviewPending && !!binding?.configured");
    expect(active).toContain("!selectiveOpen && storageReview===null");
    expect(active).not.toContain("onMount(() => lazyTester.launch");
    expect(active).not.toContain("checkUpdateOnce");
    expect(bootstrap).not.toContain("commands::virtual_clients::virtual_clients_");
    expect(bootstrap).not.toContain("commands::setup_tools::setup_");
    expect(bootstrap).not.toContain("commands::window_arrangement::window_");
  });

});
