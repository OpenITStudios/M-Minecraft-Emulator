// MIVUBI-owned Minecraft Education multi-client core control contract.
// This is functional configuration/launch planning for the PRIVATE fork,
// not an installer, signed driver, official Sandboxie adapter, or game proof.
// Real execution may be enabled only by a separately verified Windows lab.
import {readFileSync} from 'node:fs';
const targets = JSON.parse(readFileSync(new URL('./minecraft-targets.json', import.meta.url), 'utf8')).targets;
const PROFILES = Object.freeze(Object.fromEntries(targets.filter(t => t.kind === 'ISOLATED').map(t => [t.id, t.profile])));
// Exact Minecraft Education compatibility profile observed as usable in
// the authorized official Sandboxie 1.18.5 two-account Windows local test.
// App Compartment / OpenCOM reduces containment; this is NOT a hardened box.
const SETTINGS = Object.freeze([
  Object.freeze(["AutoDelete", "n"]),
  Object.freeze(["AutoRemove", "n"]),
  Object.freeze(["NoSecurityIsolation", "y"]),
  Object.freeze(["DropAdminRights", "y"]),
  Object.freeze(["Template", "OpenCOM"]),
  Object.freeze(["OpenCredentials", "n"]),
  Object.freeze(["OpenProtectedStorage", "n"]),
  // Enable LAST: never present a partially configured box as ready.
  Object.freeze(["Enabled", "y"]),
]);
// A new Sandboxie section might otherwise be enabled before its other
// settings are written. In the FUTURE exclusive-engine executor, disarm
// the target first and verify the readback before writing any other key.
const INITIAL_DISABLED = Object.freeze(["Enabled", "n"]);
const GAME_BINARIES = new Set([
  "minecraft.windows.exe", "minecrafteducation.exe",
  "minecrafteducationedition.exe",
]);
function requireProfile(client) {
  if (typeof client !== "string" ||
      !Object.prototype.hasOwnProperty.call(PROFILES, client)) {
    throw new Error("MIVUBI private launch requires exact Client 02, 03 or 04");
  }
  return PROFILES[client];
}
function requireGameExe(executable) {
  if (typeof executable !== "string" || executable.length > 1024 ||
      !/^[A-Za-z]:\\/.test(executable) ||
      /[\u0000-\u001f\u007f"'<>|*?]/.test(executable) ||
      // The native Start.cpp parser also rejects colon/forward slash after
      // the drive prefix. A syntactically accepted JS plan must never create
      // a raw command line that the private launcher itself refuses.
      /[:/]/.test(executable.slice(2)) ||
      executable.split("\\").some(part => part === ".." || part === "." || part.length === 0)) {
    throw new Error("MIVUBI private launch requires a canonical absolute Windows Education EXE");
  }
  const name = executable.split("\\").at(-1)?.toLowerCase();
  if (!GAME_BINARIES.has(name)) {
    throw new Error("MIVUBI private launch refuses non-Education executable");
  }
  return executable;
}

/** Deterministic, PRIVATE-only profile identity. Does not touch the host. */
export function ownedMinecraftProfiles() {
  return Object.entries(PROFILES).map(([client, box]) => ({
    client, box, persistent: true, accountAuthenticated: false,
    windowsDriverVerified: false,
  }));
}

/** Command arguments for creating three distinct persistent config sections.
 * An executor MUST first prove a disposable host with the owned SbieSvc/SbieDrv
 * namespace; an existing box must never be overwritten or mutated by this plan.
 * These settings reproduce the successful Education App Compartment pilot;
 * NoSecurityIsolation=y and Template=OpenCOM reduce process/COM containment.
 * OpenCredentials=n is NOT a guarantee that WAM/SSO state stays isolated.
 * An executor needs explicit operator approval for this compatibility mode
 * and must verify all settings via SbieIni query before any box is usable.
 */
function occupiedNames(existingBoxes) {
  if (!Array.isArray(existingBoxes) || existingBoxes.length > 256 ||
      existingBoxes.some(name =>
        typeof name !== "string" || !/^[A-Za-z0-9_]{1,32}$/.test(name))) {
    throw new Error("A bounded trustworthy existing box-name inventory is required");
  }
  const occupied = new Set(existingBoxes.map(name => name.toLowerCase()));
  if (occupied.size !== existingBoxes.length) {
    throw new Error("Ambiguous duplicate Sandboxie box identity");
  }
  return occupied;
}

function configurationPlan(profiles) {
  return {
    state: "PRIVATE_BOX_CONFIGURATION_PLAN_NOT_EXECUTED",
    source: "MIVUBI_PRIVATE_START_PROFILE_V1",
    compatibilityProfile: "MINECRAFT_EDUCATION_APP_COMPARTMENT_OPENCOM",
    reducedSecurityIsolation: true,
    requiresExplicitOperatorApproval: true,
    settingsReadbackVerified: false,
    profiles,
    // Historical write-only commands remain for source consumers. They are
    // NOT an authorized execution transaction. The future owned-engine
    // executor must process guardedSteps in order, stopping on the first
    // failed write/readback without ever deleting a box or account state.
    commands: profiles.flatMap(({client, box}) =>
      SETTINGS.map(([key, value]) => ({
        client, executable: "SbieIni.exe",
        arguments: ["set", box, key, value],
      }))),
    guardedSteps: profiles.flatMap(({client, box}) =>
      [INITIAL_DISABLED, ...SETTINGS].flatMap(([key, value]) => [
        {client, operation: "SET", executable: "SbieIni.exe",
          arguments: ["set", box, key, value]},
        {client, operation: "REQUIRE_READBACK", executable: "SbieIni.exe",
          arguments: ["query", box, key], expectedValue: value},
      ])),
    // A partial write can leave an existing configuration section. Without
    // proven ownership, never auto-rollback, erase or retry it.
    stopOnReadbackMismatch: true,
    partialConfigurationRequiresManualReview: true,
    automaticDeletionPermitted: false,
    hostChanged: false,
    accountIsolationVerified: false,
    driverInstalled: false,
  };
}

/** Compare read-only setting values against the ONE canonical source.
 * This is equality checking only: caller must independently authenticate a
 * signed, exclusively owned engine and fresh same-host query results.
 * It cannot prove provider identity, sign-in or gameplay by itself.
 * Fail closed on truncated, duplicated, reordered or foreign-box records.
 */
export function matchesOwnedMinecraftReadback(client, readings) {
  const box = requireProfile(client);
  if (!Array.isArray(readings) || readings.length !== SETTINGS.length) return false;
  return SETTINGS.every(([key, value], index) => {
    const row = readings[index];
    // Every required field must be OWNED by this record. An inherited
    // client/box/key/value with four unrelated own keys is not readback.
    const fields = ["client", "box", "key", "value"];
    return row !== null && typeof row === "object" && !Array.isArray(row) &&
      Object.keys(row).length === fields.length &&
      fields.every(field => Object.hasOwn(row, field)) &&
      row.client === client && row.box === box &&
      row.key === key && row.value === value;
  });
}

/** A SINGLE supported sandbox target, never an eager three-box creation.
 * The caller must explicitly pass independently observed occupied names through
 * its OWN verified engine, not the installed official Sandboxie pilot. Omitting
 * inventory must never silently mean an empty and safe host.
 * This is source-only planning: no executable or driver is invoked.
 */
export function planOwnedMinecraftVirtual(client, existingBoxes) {
  const box = requireProfile(client);
  const occupied = occupiedNames(existingBoxes);
  if (occupied.has(box.toLowerCase())) {
    throw new Error("MIVUBI-owned box already exists; refusing any overwrite");
  }
  const profile = ownedMinecraftProfiles().find(item => item.client === client);
  return configurationPlan([profile]);
}

/** Legacy bulk planning retained for existing tests/source consumers. It
 * shares the EXACT per-box profile and remains nonexecuting. The desktop
 * should use one-target planning for a single Create Virtual request.
 */
export function planOwnedMinecraftProfiles(existingBoxes) {
  const occupied = occupiedNames(existingBoxes);
  const profiles = ownedMinecraftProfiles();
  if (profiles.some(p => occupied.has(p.box.toLowerCase()))) {
    throw new Error("MIVUBI-owned box already exists; refusing any overwrite");
  }
  return configurationPlan(profiles);
}

/** Launch contract implemented by the MIVUBI-modified Start.exe parser.
 * Never replace this with /box:DefaultBox or the official installed Start.exe.
 * No shell, arguments, credentials, URL or auto-retry are supported.
 */
export function ownedMinecraftLaunch(client, executable) {
  const box = requireProfile(client);
  const game = requireGameExe(executable);
  return {
    state: "PRIVATE_START_LAUNCH_ARGUMENTS_NOT_EXECUTED",
    client, box,
    executable: "Start.exe",
    arguments: ["/mivubi_client:" + client, game],
    // For a future CreateProcessW adapter: the private Start parser requires
    // the target as one fully quoted path even if it has no spaces.
    rawCommandLine: "/mivubi_client:" + client + ' "' + game + '"',
    driverInstalled: false,
    gameplayVerified: false,
  };
}
