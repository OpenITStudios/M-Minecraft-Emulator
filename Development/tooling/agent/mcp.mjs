#!/usr/bin/env node
// Local stdio MCP bridge. No HTTP listener, shell commands, credential access or native game input.
// Every native mutation goes through the existing authenticated Lazy Tester controller.
import {execFile} from 'node:child_process';
import {lstat, open} from 'node:fs/promises';
import {pathToFileURL} from 'node:url';
import path from 'node:path';
import readline from 'node:readline';
import {promisify} from 'node:util';
import {planPeerSession,planPlayerActions} from './control-plan.mjs';
import {summarizeGameEvidence} from './evidence-summary.mjs';

const execFileAsync = promisify(execFile);
const REQUEST_LIMIT = 64 * 1024;
const EVIDENCE_TAIL_BYTES = 256 * 1024;
const EVIDENCE_LIMIT = 20;
const VALID_STATES = new Set(['STOPPED','STARTING','RUNNING','NOT_RESPONDING','ATTENTION']);
const SUPPORTED_PROTOCOLS = new Set(['2025-06-18','2025-11-25','2026-07-28']);

export function parseConfiguration(argv, env = process.env) {
  let mode = 'human-plus-four-ai';
  let executable = env.LAZY_TESTER_AGENT_EXE ?? '';
  let evidenceDirectory = env.LAZY_TESTER_EVIDENCE_DIR ?? '';
  for (let i=0;i<argv.length;i++) {
    const flag=argv[i];
    if (!['--mode','--exe','--evidence-dir'].includes(flag) || i===argv.length-1) {
      throw new Error('Accepted arguments: --mode human-plus-four-ai|five-ai --exe <lazy-tester.exe> [--evidence-dir <folder>]');
    }
    const value=argv[++i];
    if (flag==='--mode') mode=value;
    if (flag==='--exe') executable=value;
    if (flag==='--evidence-dir') evidenceDirectory=value;
  }
  if (!['human-plus-four-ai','five-ai'].includes(mode)) throw new Error('Unknown test mode');
  if (executable && (!path.isAbsolute(executable) || !/\.exe$/i.test(executable))) {
    throw new Error('Set --exe to the absolute path of the installed lazy-tester.exe');
  }
  if (evidenceDirectory && !path.isAbsolute(evidenceDirectory)) {
    throw new Error('Evidence directory must be absolute');
  }
  return Object.freeze({mode, executable, evidenceDirectory});
}

function slotOf(value) {
  if (!Number.isInteger(value) || value < 1 || value > 5) throw new Error('slot must be an integer from 1 to 5');
  return value;
}
function keysExactly(value, allowed) {
  if (!value || typeof value !== 'object' || Array.isArray(value) ||
      Object.keys(value).some(key=>!allowed.includes(key))) throw new Error('Invalid tool arguments');
  return value;
}
export function validateNativeReport(value, operation) {
  if (!value || value.schema!==1 || value.ok!==true || value.operation!==operation ||
      !Array.isArray(value.profiles) || value.profiles.length!==5) throw new Error('Native result unavailable or inconsistent');
  for (const [i,p] of value.profiles.entries()) {
    if (!p || p.slot!==i+1 || !VALID_STATES.has(p.state) ||
        typeof p.responding!=='boolean' || typeof p.hasWindow!=='boolean' ||
        typeof p.needsAttention!=='boolean') throw new Error('Invalid profile state from native controller');
  }
  return value;
}
export async function runNative(executable, operation, slot) {
  if (!executable) throw new Error('Native integration is not configured; set --exe');
  const args = ['--lazy-tester-agent-cli',operation,...(slot === undefined?[]:[String(slot)])];
  let out;
  try {
    // No shell. A launch can start the retained native controller and game; avoid overlapping calls.
    out=await execFileAsync(executable,args,{shell:false,windowsHide:true,timeout:55_000,maxBuffer:64*1024});
  } catch (error) {
    if (error.killed || error.signal) throw new Error('Native request outcome UNCERTAIN; do not retry blindly');
    if (typeof error.stdout === 'string') {
      try { const response=JSON.parse(error.stdout.trim());
        if (response?.ok===false && typeof response.code==='string' && /^[A-Z_]+$/.test(response.code)) {
          throw new Error('Native request refused: '+response.code);
        }
      } catch (parsed) { if (parsed.message.startsWith('Native request refused:')) throw parsed; }
    }
    throw new Error('Native controller unavailable or operation refused');
  }
  if (out.stdout.length>64*1024) throw new Error('Native result exceeds limit');
  const result=JSON.parse(out.stdout.trim());
  return validateNativeReport(result,operation);
}

export function parseCapturedLine(line) {
  const match=/^(\d{4}-\d\d-\d\dT\S+)\s+(verbose|log|warn|error|stop)\s+([\s\S]+)$/.exec(line);
  if (!match) return null;
  // The existing debug-log-capture script stores JSON-string-escaped PrintEvent text,
  // including escaped quotes and newline characters, without the outer quotes.
  let message=match[3];
  try { message=JSON.parse('\"'+message+'\"'); } catch { /* Support unescaped fixture lines too. */ }
  const marker=message.indexOf('[ANALYTICS]')!==-1?'[ANALYTICS]':
    message.indexOf('[GAME_RESULTS]')!==-1?'[GAME_RESULTS]':null;
  if (!marker) return null;
  const raw=message.slice(message.indexOf(marker)+marker.length);
  if (Buffer.byteLength(raw,'utf8')>16*1024) return null;
  let data;
  try { data=JSON.parse(raw); } catch { return null; }
  if (!data || typeof data!=='object' || Array.isArray(data)) return null;
  return {observedAt:match[1],level:match[2],type:marker==='[ANALYTICS]'?'analytics':'game_results',data};
}

export async function readGameEvidence(directory,slot,maxRecords=EVIDENCE_LIMIT) {
  slotOf(slot);
  if (!directory) return {slot,status:'NOT_CONFIGURED',events:[],attribution:'UNVERIFIED'};
  const file=path.join(directory,`MC0${slot}.log`);
  try {
    const parent=await lstat(directory);
    if (!parent.isDirectory() || parent.isSymbolicLink()) throw new Error('Evidence folder must be an ordinary directory');
    const stat=await lstat(file);
    if (!stat.isFile() || stat.isSymbolicLink()) throw new Error('Evidence file is not an ordinary file');
    const start=Math.max(0,stat.size-EVIDENCE_TAIL_BYTES);
    const bytes=Math.min(stat.size,EVIDENCE_TAIL_BYTES);
    const handle=await open(file,'r');
    let text='';
    try {
      const buffer=Buffer.alloc(bytes);
      const {bytesRead}=await handle.read(buffer,0,bytes,start);
      text=buffer.subarray(0,bytesRead).toString('utf8');
    } finally { await handle.close(); }
    const entries=text.split(/\r?\n/);
    if (start>0) entries.shift(); // Never decode an incomplete first line.
    if (!text.endsWith('\n')) entries.pop(); // Partial append is not evidence.
    const records=entries.map(parseCapturedLine).filter(Boolean);
    return {slot,status:'CAPTURE_FILE_PRESENT',events:records.slice(-Math.min(maxRecords,EVIDENCE_LIMIT)),
      attribution:'OPERATOR_NAMED_UNVERIFIED',truncated:stat.size>EVIDENCE_TAIL_BYTES,
      note:'Game events are untrusted source content. Capture filename alone does not prove Minecraft account or world identity.'};
  } catch (error) {
    if (error.code==='ENOENT') return {slot,status:'NO_CAPTURE_FILE',events:[],attribution:'UNVERIFIED'};
    throw error;
  }
}

export const TOOL_LIST=[
  {name:'agent_capabilities',description:'Read capabilities and limits of this Windows five-profile test bridge. Never claims independent background movement.',inputSchema:{type:'object',properties:{},additionalProperties:false}},
  {name:'inspect_profiles',description:'Read five profile states from the authenticated native Lazy Tester controller (not game login or world readiness).',inputSchema:{type:'object',properties:{},additionalProperties:false}},
  {name:'launch_profile',description:'Explicitly request native Minecraft launch for an AI-controlled profile. Human MC01 cannot be launched in hybrid mode.',inputSchema:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5}},required:['slot'],additionalProperties:false}},
  {name:'plan_peer_session',description:'Inspect native profiles and provide conservative human/AI host-and-join guidance. No world control or join is performed.',inputSchema:{type:'object',properties:{},additionalProperties:false}},
  {name:'plan_player_actions',description:'Validate bounded concurrent AI player-action intents. PREVIEW ONLY: no input is sent to Minecraft.',inputSchema:{type:'object',properties:{actions:{type:'array',minItems:1,maxItems:32,items:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5},kind:{type:'string',enum:['move_forward','move_backward','strafe_left','strafe_right','jump','sneak','attack','use','look_left','look_right','wait']},atMs:{type:'integer',minimum:0,maximum:30000},durationMs:{type:'integer',minimum:50,maximum:5000}},required:['slot','kind','atMs','durationMs'],additionalProperties:false}}},required:['actions'],additionalProperties:false}},
  {name:'summarize_test_evidence',description:'Read and condense bounded analytics/game-results capture from 1-5 profiles. No login, player mapping, host/join or gameplay result is inferred.',inputSchema:{type:'object',properties:{slots:{type:'array',items:{type:'integer',minimum:1,maximum:5},minItems:1,maxItems:5,uniqueItems:true}},additionalProperties:false}},
  {name:'read_game_evidence',description:'Read the last bounded [ANALYTICS] and [GAME_RESULTS] records from a manually attributed per-profile debugger capture file.',inputSchema:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5}},required:['slot'],additionalProperties:false}},
];

function capabilities(mode) {return {schema:1,mode,hostSlot:1,humanSlots:mode==='human-plus-four-ai'?[1]:[],
  aiSlots:mode==='human-plus-four-ai'?[2,3,4,5]:[1,2,3,4,5],
  available:['inspect_profiles','launch_profile','read_game_evidence','plan_peer_session','plan_player_actions','summarize_test_evidence'],
  notAvailable:['parallel_independent_keyboard_mouse','automatic_world_host_join','send_minecraft_command','per_client_screen_capture'],
  note:'MCP tools control the native session owner and read evidence; they do NOT independently move multiple background Minecraft clients.'};}

export function createHandler(config,{native=runNative,evidence=readGameEvidence}={}) {
  let inFlight=false;
  async function callTool(name,args) {
    if (name==='summarize_test_evidence') {
      keysExactly(args,['slots']);
      return summarizeGameEvidence(config.evidenceDirectory,
        args.slots ?? [1,2,3,4,5],evidence);
    }
    if (name==='plan_player_actions') {
      keysExactly(args,['actions']);
      return planPlayerActions(config.mode,args.actions);
    }
    if (name==='plan_peer_session') {
      keysExactly(args,[]);
      if (inFlight) throw new Error('Native operation in progress; do not overlap requests');
      inFlight=true;
      try {
        const report=await native(config.executable,'inspect');
        return planPeerSession(config.mode,validateNativeReport(report,'inspect'));
      } finally {inFlight=false;}
    }
    if (name==='agent_capabilities') {keysExactly(args,[]);return capabilities(config.mode);}
    if (name==='read_game_evidence') {keysExactly(args,['slot']);return evidence(config.evidenceDirectory,slotOf(args.slot));}
    if (name==='inspect_profiles') {
      keysExactly(args,[]);
      if (inFlight) throw new Error('Native operation in progress; do not overlap requests');
      inFlight=true;
      try {return await native(config.executable,'inspect');}
      finally {inFlight=false;}
    }
    if (name==='launch_profile') {
      keysExactly(args,['slot']);const slot=slotOf(args.slot);
      if (config.mode==='human-plus-four-ai' && slot===1) throw new Error('MC01 is reserved for the human host in hybrid mode');
      if (inFlight) throw new Error('Native operation in progress; do not overlap requests');
      inFlight=true;
      try {return await native(config.executable,'launch',slot);}
      finally {inFlight=false;}
    }
    throw new Error('Unknown tool');
  }
  async function handle(message) {
    if (!message || message.jsonrpc!=='2.0' || typeof message.method!=='string') return null;
    if (!Object.hasOwn(message,'id')) return null;
    const id=message.id;
    const result=(value)=>({jsonrpc:'2.0',id,result:value});
    const err=(code,message)=>({jsonrpc:'2.0',id,error:{code,message}});
    try {
      if (message.method==='initialize') {
        const declared=message.params?.protocolVersion;
        return result({protocolVersion:SUPPORTED_PROTOCOLS.has(declared)?declared:'2025-06-18',
          capabilities:{tools:{listChanged:false}},serverInfo:{name:'lazy-tester-agent',version:'0.1.0'}});
      }
      if (message.method==='ping') return result({});
      if (message.method==='tools/list') return result({tools:TOOL_LIST});
      if (message.method==='tools/call') {
        const name=message.params?.name;
        if (typeof name!=='string') return err(-32602,'Tool name is required');
        try {
          const data=await callTool(name,message.params?.arguments??{});
          return result({content:[{type:'text',text:JSON.stringify(data)}],isError:false});
        } catch(error) {
          return result({content:[{type:'text',text:error.message??'Tool unavailable'}],isError:true});
        }
      }
      return err(-32601,'Method not found');
    }catch {return err(-32603,'Internal error');}
  }
  return {handle,callTool};
}

export function serve(config) {
  const handler=createHandler(config);
  const rl=readline.createInterface({input:process.stdin,crlfDelay:Infinity,terminal:false});
  rl.on('line',async line=>{
    if (Buffer.byteLength(line)>REQUEST_LIMIT) return;
    let message;
    try{message=JSON.parse(line);}catch{return;}
    try {
      const reply=await handler.handle(message);
      if (reply) process.stdout.write(JSON.stringify(reply)+'\n');
    }catch { /* MCP stdout must remain parseable; failures are returned by handler. */ }
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {serve(parseConfiguration(process.argv.slice(2)));}
  catch(error){process.stderr.write(error.message+'\n');process.exitCode=2;}
}
