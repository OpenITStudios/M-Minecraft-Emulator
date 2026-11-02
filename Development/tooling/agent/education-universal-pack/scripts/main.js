// Lazy Tester Universal Diagnostics 1.2.0 — Minecraft Education.
// Team-aligned, OPTIONAL generic observations; not [ANALYTICS] or [GAME_RESULTS].
// No raw player identity, location, inventory, chat, commands or world writes.
// The canonical analytics/scoring contracts remain owned by the team skills.
import {system, world} from "@minecraft/server";

const MARKER = "[LT_EDU_DIAG]";
const COMMAND = "lazytester:diagnostics"; // Retained team / old-world command
const SIMPLE_COMMAND = "lt:diag";          // Short operator alias
const HELP_LINES = [
  "[Lazy Tester] on = mulai | off = selesai | status = cek",
  "[Lazy Tester] more = detail 30 detik | help = bantuan. Contoh: /scriptevent lt:diag on"
];
const SUMMARY_TICKS = 200;       // Nominally 10 seconds at 20 TPS
const FOCUS_TICKS = 600;         // Noisy sources expire after 30 game seconds
const MAX_COUNT = 1000000;
const CORE = ["firstSpawns","respawns","playerLeaves","dimensionChanges","playerDeaths"];
const FOCUSED = ["otherDeaths","entitySpawns","damageEvents","damageWholeUnits"];
const COUNTER_KEYS = [...CORE,...FOCUSED];
const FILTER_PLAYER = {entityTypes:["minecraft:player"]};

let active = false;
let sequence = 0;
let captureNumber = 0;
let lastReportTick = 0;
let lastHeartbeatTick = 0;
let saturated = 0;
const bootToken = Math.floor(Math.random()*0xffffff).toString(36).padStart(5,"0");
// A local correlation token, NEVER a verified world/player or MC01-MC05 ID.
const session = "lt-"+bootToken+"-"+system.currentTick.toString(36);
const counters = Object.create(null);
const subscriptions = {core:[],combat:[],entities:[]};
const unsupported = [];
const focus = {combat:false,entities:false};
const focusGeneration = {combat:0,entities:0};

function tick(){return Number.isSafeInteger(system.currentTick)?system.currentTick:0;}
function resetCounts(){for(const key of COUNTER_KEYS)counters[key]=0;}
resetCounts();

function replyToOperator(event,text){
  // Only after an explicit command: no periodic chat, no player names.
  // scriptEventReceive may have no player source (e.g., command block);
  // when no player is available this is a world-chat announcement.
  try{
    const target=event?.initiator ?? event?.sourceEntity;
    if(target?.typeId==="minecraft:player"&&
       typeof target.sendMessage==="function"){
      target.sendMessage(text);return;
    }
    world.sendMessage(text);
  }catch{/* Chat feedback is optional; telemetry must not fail. */}
}
function showHelp(event){
  for(const line of HELP_LINES)replyToOperator(event,line);
}
function log(kind,data={}){
  // Fixed fields and counters only, never arbitrary event objects or text.
  try{
    console.warn(MARKER+JSON.stringify({
      schema:1,session,seq:++sequence,tick:tick(),kind,data
    }));
  }catch{/* Never let logging interrupt gameplay. */}
}
function count(key,amount=1){
  if(!active||counters[key]===undefined||!Number.isFinite(amount)||amount<0)return;
  const value=counters[key]+Math.floor(amount);
  if(value>MAX_COUNT){
    counters[key]=MAX_COUNT;
    saturated=Math.min(MAX_COUNT,saturated+1);
  }else counters[key]=value;
}
function subscribe(group,signal,handler,name,options){
  try{
    if(!signal||typeof signal.subscribe!=="function"){
      unsupported.push(name);return;
    }
    if(options)signal.subscribe(handler,options);
    else signal.subscribe(handler);
    subscriptions[group].push([signal,handler]);
  }catch{unsupported.push(name);}
}
function unsubscribe(group){
  for(const [signal,handler] of subscriptions[group]){
    try{signal.unsubscribe(handler);}catch{}
  }
  subscriptions[group].length=0;
}
function stopFocus(group,note){
  const wasOn=focus[group];
  focus[group]=false;
  focusGeneration[group]++;
  unsubscribe(group);
  if(wasOn&&active)log("status",{capture:captureNumber,active:true,
    combat:focus.combat,entities:focus.entities,note});
}
function flush(reason,forced=false){
  if(!active)return;
  if(!forced&&!COUNTER_KEYS.some(k=>counters[k]>0)&&saturated===0)return;
  const counts={};
  for(const k of COUNTER_KEYS)counts[k]=counters[k];
  log("summary",{capture:captureNumber,reason,counts,saturated});
  lastReportTick=tick();
  saturated=0;
  resetCounts();
}
function corePlayerDeath(e){
  try{if(e.deadEntity?.typeId==="minecraft:player")count("playerDeaths");}
  catch{/* A lost read is not evidence of zero deaths. */}
}
function onHurt(e){
  try{
    if(e.hurtEntity?.typeId!=="minecraft:player")return;
    count("damageEvents");
    const damage=e.damage;
    if(typeof damage==="number"&&Number.isFinite(damage)&&damage>0)
      count("damageWholeUnits",Math.floor(damage));
  }catch{/* Ignore unsupported event payloads. */}
}
function onOtherDeath(e){
  try{if(e.deadEntity?.typeId!=="minecraft:player")count("otherDeaths");}
  catch{}
}
function start(){
  if(active){
    log("status",{capture:captureNumber,active:true,note:"ALREADY_RUNNING"});
    return;
  }
  captureNumber++;
  active=true;
  saturated=0;
  unsupported.length=0;
  resetCounts();
  lastReportTick=tick();
  lastHeartbeatTick=tick();
  // Tier 1: low-frequency lifecycle only, aligned with team analytics tiers.
  subscribe("core",world.afterEvents.playerSpawn,
    e=>count(e.initialSpawn?"firstSpawns":"respawns"),"playerSpawn");
  subscribe("core",world.afterEvents.playerLeave,
    ()=>count("playerLeaves"),"playerLeave");
  subscribe("core",world.afterEvents.playerDimensionChange,
    ()=>count("dimensionChanges"),"playerDimensionChange");
  subscribe("core",world.afterEvents.entityDie,
    corePlayerDeath,"entityDiePlayer",FILTER_PLAYER);
  log("started",{capture:captureNumber,scope:"CORE_LIFECYCLE_ONLY",
    unsupported,worldIdentity:"UNVERIFIED"});
}
// Explicit short-lived focus, no universal default world-wide entity scans.
function enableFocus(group,quiet=false){
  if(!active){
    log("status",{active:false,note:"START_REQUIRED"});return;
  }
  if(focus[group]){
    log("status",{capture:captureNumber,active:true,
      note:"FOCUS_ALREADY_ENABLED"});return;
  }
  focus[group]=true;
  if(group==="combat"){
    subscribe("combat",world.afterEvents.entityHurt,
      onHurt,"entityHurtPlayer",FILTER_PLAYER);
  }else{
    subscribe("entities",world.afterEvents.entitySpawn,
      ()=>count("entitySpawns"),"entitySpawn");
    subscribe("entities",world.afterEvents.entityDie,
      onOtherDeath,"entityDieOthers");
  }
  const generation=++focusGeneration[group];
  if(!quiet)log("status",{capture:captureNumber,active:true,
    combat:focus.combat,entities:focus.entities,note:"FOCUS_STARTED"});
  // Generation fence prevents an earlier timeout from stopping a newer test.
  system.runTimeout(()=>{
    if(active&&focus[group]&&focusGeneration[group]===generation)
      stopFocus(group,"FOCUS_AUTO_EXPIRED");
  },FOCUS_TICKS);
}
function status(){
  log("status",{capture:captureNumber,active,
    combat:focus.combat,entities:focus.entities,
    worldIdentity:"UNVERIFIED"});
}
function stop(){
  if(!active){
    log("status",{active:false,note:"ALREADY_STOPPED"});return;
  }
  flush("stop",true);
  active=false;
  stopFocus("combat","STOPPED");
  stopFocus("entities","STOPPED");
  unsubscribe("core");
  log("stopped",{capture:captureNumber});
}
// Two IDs, ONE execution path. The original command is preserved for
// developer scripts and older worlds; the short alias is operator-friendly.
// Only an explicit help/status/control command sends chat feedback.
try{
  system.afterEvents.scriptEventReceive.subscribe(e=>{
    if(e.id!==COMMAND&&e.id!==SIMPLE_COMMAND)return;
    const command=String(e.message??"").trim().toLowerCase();
    if(command==="help"||command==="?"){
      showHelp(e);
    }else if(command==="on"||command==="start"){
      const already=active;
      start();
      replyToOperator(e,already?
        "[Lazy Tester] Diagnostics sudah aktif.":
        "[Lazy Tester] Diagnostics aktif. Ketik /scriptevent lt:diag off setelah selesai.");
    }else if(command==="off"||command==="stop"){
      const already=!active;
      stop();
      replyToOperator(e,already?
        "[Lazy Tester] Diagnostics sudah nonaktif.":
        "[Lazy Tester] Diagnostics dihentikan. Ringkasan terakhir dicatat lokal melalui jalur log jika tersedia.");
    }else if(command==="status"){
      status();
      replyToOperator(e,active?
        "[Lazy Tester] AKTIF. Detail sementara: "+
          (focus.combat||focus.entities?"ON":"OFF")+
          ". Ketik /scriptevent lt:diag help untuk bantuan.":
        "[Lazy Tester] NONAKTIF. Mulai: /scriptevent lt:diag on");
    }else if(command==="more"){
      if(!active){
        replyToOperator(e,"[Lazy Tester] Mulai dulu: /scriptevent lt:diag on");
      }else{
        const already=focus.combat&&focus.entities;
        if(!focus.combat)enableFocus("combat",true);
        if(!focus.entities)enableFocus("entities",true);
        if(!already)log("status",{capture:captureNumber,active:true,
          combat:focus.combat,entities:focus.entities,
          note:"FOCUS_SHORT_STARTED"});
        replyToOperator(e,already?
          "[Lazy Tester] Detail masih aktif; akan berhenti otomatis.":
          "[Lazy Tester] Detail aktif sekitar 30 detik waktu game, lalu otomatis nonaktif.");
      }
    }else if(command==="flush"){
      flush("manual",true);
      replyToOperator(e,active?
        "[Lazy Tester] Ringkasan dicatat ke log jika tersedia.":
        "[Lazy Tester] Diagnostics nonaktif, tidak ada ringkasan baru.");
    }else if(command==="combat on")enableFocus("combat");
    else if(command==="combat off")stopFocus("combat","FOCUS_DISABLED");
    else if(command==="entities on")enableFocus("entities");
    else if(command==="entities off")stopFocus("entities","FOCUS_DISABLED");
    else {
      log("status",{active,note:"INVALID_SUBCOMMAND"});
      showHelp(e);
    }
  });
  system.run(()=>log("ready",{scope:"CORE_LIFECYCLE_ONLY",
    defaultActive:false,command:COMMAND,compatibility:"NOT_VERIFIED"}));
}catch{
  system.run(()=>log("unavailable",{code:"SCRIPT_EVENT_COMMAND_UNAVAILABLE"}));
}
// Single low-frequency heartbeat/summary timer, no polling of game entities.
system.runInterval(()=>{
  if(!active)return;
  const now=tick();
  if(now-lastReportTick>=SUMMARY_TICKS){
    flush("interval");
    lastReportTick=now;
  }
  if(now-lastHeartbeatTick>=3*SUMMARY_TICKS){
    log("heartbeat",{capture:captureNumber,active:true});
    lastHeartbeatTick=now;
  }
},SUMMARY_TICKS);
