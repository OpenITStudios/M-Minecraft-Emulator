// Lazy Tester — Team Commands Help 2.0.0 (Minecraft Education)
// ONE optional help overlay, NOT another diagnostics/analytics engine.
// The original team modules remain the authoritative implementations:
// [ANALYTICS] from mivubi:get_analytics and [GAME_RESULTS] from daigon:get_all_scoring.
// No logging, recording, polling, scoreboard, world or player-state mutation.
import {Player, system} from "@minecraft/server";

const HELP_EVENT = "lazytester:help";
const HELP_MESSAGE = "commands";
// These IDs come from the operator-supplied team module source snapshots.
// Availability depends on whether the target map actually installed/wired
// the team modules; Skills Probe-only commands are not universal.
const HELP_LINES = Object.freeze([
  "§b[Lazy Tester] DAFTAR COMMAND TIM§r",
  "§eAnalytics (perlu modul analytics di map):§r",
  "/scriptevent mivubi:get_analytics",
  "  Export data analytics ke Content Log, bukan chat.",
  "§eGame Results (perlu modul game-results di map):§r",
  "/scriptevent daigon:get_all_scoring",
  "  Export skor/hasil permainan ke Content Log.",
  "§eHanya di world Skills Probe (admin):§r",
  "/scriptevent probe:list",
  "  Lihat daftar probe; hasil daftar ada di Content Log.",
  "/scriptevent probe:run auto",
  "  Jalankan tes otomatis. Gunakan world uji saja.",
  "/scriptevent probe:report",
  "  Lihat hasil tes tersimpan di Content Log.",
  "§7Perintah hanya bekerja jika modul/probe tim sudah terpasang.§r"
]);

function showCommandHelp(player) {
  for (const line of HELP_LINES) {
    try { player.sendMessage(line); }
    catch { return; } // No world-chat fallback; never broadcast.
  }
}

// Exactly one event listener. Only a player explicitly requesting the
// fixed help command can receive text. No per-tick work or hidden export.
try {
  system.afterEvents.scriptEventReceive.subscribe(event => {
    if (event.id !== HELP_EVENT) return;
    const player = event.sourceEntity;
    if (!(player instanceof Player) || !player.isValid) return;
    if (String(event.message ?? "").trim().toLowerCase() !== HELP_MESSAGE) {
      player.sendMessage("Gunakan: /scriptevent lazytester:help commands");
      return;
    }
    showCommandHelp(player);
  }, {namespaces:["lazytester"]});
} catch {
  // This helper is optional. Unsupported Script API must not break
  // another team pack or mutate a game state.
}
