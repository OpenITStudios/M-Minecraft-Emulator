// EXPERIMENTAL SOURCE: local file mailbox for Minecraft Education MakeCode.
// No MCP tool calls this yet. No game input is injected into Windows.
// Requires the optional MakeCode File extension running in EACH AI client.
// No tests or Minecraft acceptance have been performed on this transport.
import {randomUUID} from 'node:crypto';
import {readFile, lstat, rename, writeFile, unlink} from 'node:fs/promises';
import path from 'node:path';
import {slotsForMode} from '../control-plan.mjs';

export const FILE_ACTIONS = Object.freeze([
  'PING','STEP_NORTH','STEP_SOUTH','STEP_EAST','STEP_WEST',
]);
const ACTIONS = new Set(FILE_ACTIONS);
const FILE_LIMIT = 80;
const SEQ_LIMIT = 999999;
const REQUEST = /^LT1\|(0|[1-9]\d{0,5})\|(IDLE|PING|STEP_NORTH|STEP_SOUTH|STEP_EAST|STEP_WEST)$/;
const REPLY = /^LT1\|(0|[1-9]\d{0,5})\|(READY|PING_ACK|STEP_OK|STEP_FAILED|UNSUPPORTED)$/;

function validSlot(mode,slot) {
  const owner=slotsForMode(mode);
  if (!Number.isInteger(slot) || !owner.aiSlots.includes(slot)) {
    throw new TypeError('Slot is not controlled by AI in this test mode');
  }
  return slot;
}
export function mailboxPaths(directory,slot,mode='human-plus-four-ai') {
  validSlot(mode,slot);
  if (typeof directory!=='string' || directory.length>240 ||
      !path.win32.isAbsolute(directory) || directory.startsWith('\\\\')) {
    throw new TypeError('Use a trusted absolute local Windows folder, not a UNC/network path');
  }
  const folder=path.win32.normalize(directory);
  if (!folder || folder.endsWith(':\\')) {
    throw new TypeError('A dedicated existing data folder is required');
  }
  const basename='MC0'+slot;
  return Object.freeze({
    slot, folder,
    inbox:path.win32.join(folder,basename+'.in.txt'),
    ack:path.win32.join(folder,basename+'.ack.txt'),
  });
}
function parseMessage(source,pattern) {
  if (typeof source!=='string' || source.length>FILE_LIMIT) {
    throw new TypeError('Invalid mailbox payload length');
  }
  const match=pattern.exec(source);
  if (!match) throw new TypeError('Malformed mailbox message');
  const sequence=Number(match[1]);
  if (!Number.isSafeInteger(sequence)||sequence<0||sequence>SEQ_LIMIT) {
    throw new TypeError('Invalid mailbox sequence');
  }
  return {sequence,code:match[2]};
}
export const parseRequest = value => parseMessage(value,REQUEST);
export const parseReply = value => parseMessage(value,REPLY);

async function ordinaryFolder(directory) {
  const stat=await lstat(directory);
  if (!stat.isDirectory() || stat.isSymbolicLink()) {
    throw new TypeError('Mailbox root must be an ordinary local folder');
  }
}
async function readBounded(file) {
  let stat;
  try {stat=await lstat(file);}
  catch(error) {if(error?.code==='ENOENT') return null;throw error;}
  if (!stat.isFile() || stat.isSymbolicLink() || stat.size>FILE_LIMIT) {
    throw new TypeError('Mailbox must be an ordinary bounded text file');
  }
  return readFile(file,'utf8');
}
async function atomicReplace(file,data) {
  // Never allow a partial new command to be interpreted by MakeCode. If
  // Windows denies replacement while the game reads, report failure and STOP.
  const temp=file+'.'+randomUUID()+'.tmp';
  await writeFile(temp,data,{flag:'wx',encoding:'utf8'});
  try {await rename(temp,file);}
  finally {await unlink(temp).catch(error => {
    if (error?.code!=='ENOENT') throw error;
  });}
}

/** Prepare a mailbox only if neither file exists. It never launches a game. */
export async function prepareMailbox(directory,mode,slot) {
  const p=mailboxPaths(directory,slot,mode);
  await ordinaryFolder(p.folder);
  if (await readBounded(p.inbox)!==null || await readBounded(p.ack)!==null) {
    throw new TypeError('Existing mailbox was preserved; cannot silently reinitialize');
  }
  // The MakeCode project must start after this IDLE record is installed.
  await atomicReplace(p.inbox,'LT1|0|IDLE');
  return {slot,prepared:true,ready:false,codeBuilderStarted:false};
}

/** An ACK proves only that a file was observed; never real player movement. */
export async function readMailboxStatus(directory,mode,slot) {
  const p=mailboxPaths(directory,slot,mode);
  await ordinaryFolder(p.folder);
  const input=await readBounded(p.inbox);
  const output=await readBounded(p.ack);
  const request=input===null?null:parseRequest(input);
  const reply=output===null?null:parseReply(output);
  return {
    slot,hasInbox:request!==null,hasAck:reply!==null,
    sequence:request?.sequence??null,
    acknowledged:request!==null && reply?.sequence===request.sequence,
    acknowledgement:reply?.code??null,
    proof:'FILE_ONLY_CLIENT_AND_GAMEPLAY_UNVERIFIED',
  };
}

/** Stage one fixed action only after the preceding request was ACKed. */
export async function stageMakeCodeAction(directory,mode,slot,action) {
  if (!ACTIONS.has(action)) throw new TypeError('Unsupported MakeCode experiment action');
  const p=mailboxPaths(directory,slot,mode);
  await ordinaryFolder(p.folder);
  const current=await readBounded(p.inbox);
  const response=await readBounded(p.ack);
  if (current===null||response===null) {
    throw new TypeError('MakeCode client is not READY; no action staged');
  }
  const request=parseRequest(current),ack=parseReply(response);
  if (ack.sequence!==request.sequence || request.sequence>=SEQ_LIMIT) {
    throw new TypeError('Pending or exhausted command sequence; no implicit retry');
  }
  const sequence=request.sequence+1;
  await atomicReplace(p.inbox,'LT1|'+sequence+'|'+action);
  return {
    slot,sequence,action,delivery:'LOCAL_MAILBOX_STAGED',
    gameExecuted:false,gameplayVerified:false,
    note:'Client project must be running; file ACK alone is not proof of movement.',
  };
}

/** Render a template for exactly one AI-operated profile and chosen folder. */
export function renderMakeCodeTemplate(template,directory,mode,slot) {
  const p=mailboxPaths(directory,slot,mode);
  if (typeof template!=='string' ||
      template.split('__INPUT_FILE__').length!==2 ||
      template.split('__ACK_FILE__').length!==2) {
    throw new TypeError('Exact MakeCode template markers required');
  }
  const input=p.inbox.replaceAll('\\','/');
  const ack=p.ack.replaceAll('\\','/');
  return template.replace('__INPUT_FILE__',JSON.stringify(input))
    .replace('__ACK_FILE__',JSON.stringify(ack));
}
