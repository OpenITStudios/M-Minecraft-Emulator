// SOURCE-ONLY candidate for the original team-owned in-game Script API route.
// The canonical coords/display module, not this file, owns event subscription,
// per-player visibility state and Minecraft UI rendering. No listener is added.
// A /scriptevent may originate from a keyboard macro, chat or another system;
// even a Player source is NOT proof of a physical F3 key press.
import {formatPlayerDebugSnapshot} from './format-player-debug.mjs';

const EVENT_ID = /^[a-z0-9_.-]+:[a-z0-9_./-]+$/;

export function planPlayerSourcedDebugToggle(event, {eventId, currentlyVisible} = {}) {
  if (typeof eventId !== 'string' || !EVENT_ID.test(eventId) ||
      typeof currentlyVisible !== 'boolean') {
    throw new TypeError('Invalid original team debug-toggle contract');
  }
  try {
    const player = event?.sourceEntity;
    // Do not borrow a player from an NPC, block, server command, roster,
    // Windows process, display name or unqualified event payload.
    // These property getters can throw if Minecraft retired the Entity.
    if (event?.id !== eventId || event?.message !== 'toggle' ||
        event?.sourceType !== 'Entity' || !player ||
        player.typeId !== 'minecraft:player' ||
        event.sourceBlock !== undefined || event.initiator !== undefined) {
      return null;
    }
    if (currentlyVisible) {
      // Hide must not depend on obtaining a fresh Player snapshot. This is
      // only a proposed UI intent; the existing team HUD owner must clear it.
      return Object.freeze({kind:'hide',player,nextVisible:false});
    }
    // All values come from this exact original Player, not world.getPlayers.
    const snapshot = formatPlayerDebugSnapshot({
      location:player.location,
      rotation:player.getRotation(),
      dimensionId:player.dimension?.id,
    });
    // This is prepared text, NOT a successful clipboard write.
    return Object.freeze({
      kind:'show',player,nextVisible:true,
      lines:snapshot.lines,copyCoordinatesText:snapshot.copyCoordinatesText,
    });
  } catch {
    // Missing/replaced player, invalid dimension/rotation/coords and API
    // read errors fail closed without exposing private native messages.
    return null;
  }
}
