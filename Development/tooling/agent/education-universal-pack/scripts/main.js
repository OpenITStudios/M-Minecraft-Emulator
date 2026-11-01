// Lazy Tester Universal Diagnostics 1.0.0
// Minecraft Education target; stable @minecraft/server 2.8.0 candidate.
// Generic read-only, operator-controlled test telemetry.
// No identity, inventory, chat, coordinate, or world-data collection.
// No HTTP/network/file/world/dynamic-property writes.
import { system, world } from "@minecraft/server";

const MARKER = "[LT_EDU_DIAG]";
const COMMAND = "lazytester:diagnostics";
const COUNTER_KEYS = [
  "firstSpawns", "respawns", "playerLeaves",
  "playerDeaths", "otherDeaths", "entitySpawns",
  "damageEvents", "damageWholeUnits"
];
const SUMMARY_TICKS = 200; // 10 seconds at 20 ticks/s, not wall-clock time
const MAX_COUNT = 1000000;

let active = false;
let sequence = 0;
let lastReportTick = 0;
let lastHeartbeatTick = 0;
let captureNumber = 0;
// Correlation only, NOT a verified world/player/profile identity.
const bootToken = Math.floor(Math.random() * 0xFFFFFF).toString(36).padStart(5, "0");
const session = "lt-" + bootToken + "-" + system.currentTick.toString(36);
const totals = Object.create(null);
const counters = Object.create(null);
let saturated = 0;
const registered = [];
const unsupported = [];

function resetCounts() {
  for (const name of COUNTER_KEYS) counters[name] = 0;
}
resetCounts();

function tick() {
  return Number.isSafeInteger(system.currentTick) ? system.currentTick : 0;
}
function log(kind, detail = {}) {
  // Non-secret fixed keys. Never include raw game events or error objects.
  const record = {
    schema: 1, session, seq: ++sequence, tick: tick(),
    kind, data: detail
  };
  try { console.warn(MARKER + JSON.stringify(record)); } catch {}
}
function count(key, amount = 1) {
  if (!active) return;
  if (counters[key] === undefined) return;
  if (!Number.isFinite(amount) || amount < 0) return;
  const next = counters[key] + Math.floor(amount);
  if (next > MAX_COUNT) {
    saturated++;
    counters[key] = MAX_COUNT;
  } else {
    counters[key] = next;
  }
}
function flush(reason, forced = false) {
  if (!active) return;
  const hasData = COUNTER_KEYS.some(key => counters[key] > 0);
  if (!forced && !hasData) return;
  const values = {};
  for (const key of COUNTER_KEYS) values[key] = counters[key];
  log("summary", { capture: captureNumber, reason, counts: values,
    saturated: saturated > 0 ? saturated : 0 });
  lastReportTick = tick();
  saturated = 0;
  resetCounts();
}
function subscribe(signal, handler, name) {
  try {
    if (!signal || typeof signal.subscribe !== "function") {
      unsupported.push(name);
      return;
    }
    signal.subscribe(handler);
    registered.push([signal, handler]);
  } catch {
    unsupported.push(name);
  }
}
function onFirstSpawn(e) {
  count(e.initialSpawn ? "firstSpawns" : "respawns");
}
function onPlayerLeave() { count("playerLeaves"); }
function onEntityDeath(e) {
  let isPlayer = false;
  try { isPlayer = e.deadEntity.typeId === "minecraft:player"; } catch {}
  count(isPlayer ? "playerDeaths" : "otherDeaths");
}
function onEntitySpawn() { count("entitySpawns"); }
function onHurt(e) {
  count("damageEvents");
  const damage = Number(e.damage);
  if (Number.isFinite(damage) && damage > 0) {
    count("damageWholeUnits", Math.floor(damage));
  }
}
function begin() {
  if (active) {
    log("status", { active: true, capture: captureNumber,
      note: "ALREADY_RUNNING" });
    return;
  }
  captureNumber++;
  active = true;
  lastReportTick = tick();
  lastHeartbeatTick = tick();
  unsupported.length = 0;
  resetCounts();
  saturated = 0;
  subscribe(world.afterEvents.playerSpawn, onFirstSpawn, "playerSpawn");
  subscribe(world.afterEvents.playerLeave, onPlayerLeave, "playerLeave");
  subscribe(world.afterEvents.entityDie, onEntityDeath, "entityDie");
  subscribe(world.afterEvents.entitySpawn, onEntitySpawn, "entitySpawn");
  subscribe(world.afterEvents.entityHurt, onHurt, "entityHurt");
  log("started", { capture: captureNumber, scope: "GENERIC_EVENTS_ONLY",
    worldIdentity: "UNVERIFIED", unsupported });
}
function finish() {
  if (!active) {
    log("status", { active: false, note: "ALREADY_STOPPED" });
    return;
  }
  flush("stop", true);
  active = false;
  for (const [signal, handler] of registered) {
    try { signal.unsubscribe(handler); } catch {}
  }
  registered.length = 0;
  log("stopped", { capture: captureNumber });
}
function status() {
  log("status", { active, capture: captureNumber,
    scope: "GENERIC_EVENTS_ONLY", worldIdentity: "UNVERIFIED" });
}
// Only this namespaced operator command controls collection.
// No chat output, permissions, player data, game commands, or effects.
try {
  system.afterEvents.scriptEventReceive.subscribe(e => {
    if (e.id !== COMMAND) return;
    const action = String(e.message ?? "").trim().toLowerCase();
    if (action === "start") begin();
    else if (action === "stop") finish();
    else if (action === "status") status();
    else if (action === "flush") flush("manual", true);
    else log("status", { active, note: "USE_START_STOP_STATUS_FLUSH" });
  });
  system.run(() => log("ready", {
    scope: "GENERIC_EVENTS_ONLY", defaultActive: false,
    command: COMMAND, compatibility: "UNVERIFIED_EDUCATION_26_33"
  }));
} catch {
  system.run(() => log("unavailable", {
    code: "SCRIPT_EVENT_COMMAND_UNAVAILABLE" }));
}
// One scheduler; no polling players, entities, blocks or scoreboard.
system.runInterval(() => {
  if (!active) return;
  const now = tick();
  if (now - lastReportTick >= SUMMARY_TICKS) {
    flush("interval");
    lastReportTick = now;
  }
  // An occasional heartbeat distinguishes quiet testing from a dead logger.
  if (now - lastHeartbeatTick >= SUMMARY_TICKS * 3) {
    log("heartbeat", { capture: captureNumber, active: true });
    lastHeartbeatTick = now;
  }
}, SUMMARY_TICKS);
