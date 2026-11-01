// Minecraft Education Script API CAPABILITY PROBE (source only).
// Intended ONLY for explicit import on a test copy of a behavior pack.
// Does not mutate a world, read player data, open sockets, or run automatically.
// Stable @minecraft/server APIs still require Education runtime verification.
import {system} from '@minecraft/server';

const MARKER='[LT_EDU_PROBE]';
const RUN=/^[a-z][a-z0-9._-]{0,47}$/;

// Caller must opt in by importing this function and calling it ONCE.
// Output is a minimal marker to test whether a Script API console.warn
// reaches a separately configured local log capture; never gameplay proof.
export function emitEducationScriptLogProbe(run) {
  if(typeof run!=='string'||!RUN.test(run)) {
    throw new TypeError('Expected nonsecret probe run identifier');
  }
  const emit=(seq,phase)=>{
    console.warn(MARKER+JSON.stringify({
      schema:1,run,seq,tick:system.currentTick,
      kind:'script_probe',phase,
    }));
  };
  // Explicitly bounded: exactly two emits and one delayed callback.
  // No world scans, subscriptions, loops, player/account access, or commands.
  system.run(()=>{
    emit(1,'START');
    system.runTimeout(()=>emit(2,'DEFERRED'),20);
  });
}
