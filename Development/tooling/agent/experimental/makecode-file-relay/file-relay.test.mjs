// Authored source-only future tests. DO NOT execute until operator approval.
import test from 'node:test';
import assert from 'node:assert/strict';
import {
  FILE_ACTIONS,mailboxPaths,parseRequest,parseReply,renderMakeCodeTemplate,
} from './file-relay.mjs';

test('MC01 protected in hybrid; MC01 allowed as AI in five-ai',()=>{
  assert.throws(()=>mailboxPaths('C:\\AgentData',1,'human-plus-four-ai'),/AI/);
  assert.equal(mailboxPaths('C:\\AgentData',2,'human-plus-four-ai').slot,2);
  assert.equal(mailboxPaths('C:\\AgentData',1,'five-ai').slot,1);
  assert.throws(()=>mailboxPaths('\\\\host\\share',2,'five-ai'),/UNC/);
  assert.throws(()=>mailboxPaths('relative',2,'five-ai'),/absolute/);
});
test('short, explicit mailbox protocol never interprets executable commands',()=>{
  assert.deepEqual(parseRequest('LT1|12|STEP_NORTH'),{sequence:12,code:'STEP_NORTH'});
  assert.deepEqual(parseReply('LT1|12|STEP_OK'),{sequence:12,code:'STEP_OK'});
  assert.equal(FILE_ACTIONS.includes('PING'),true);
  for (const s of ['LT1|12|EXEC','LT1|12|STEP_NORTH\n', 'LT1|01|PING',
      'LT1|12|/op @a','LT1|12|PING|more']) assert.throws(()=>parseRequest(s));
});
test('MakeCode generation requires exact paths and has no global input',()=>{
  const out=renderMakeCodeTemplate(
    'let inFile=__INPUT_FILE__;let ackFile=__ACK_FILE__;',
    'C:\\AgentData','human-plus-four-ai',2);
  assert.match(out,/MC02\.in\.txt/);
  assert.match(out,/MC02\.ack\.txt/);
  assert.doesNotMatch(out,/__INPUT_FILE__|__ACK_FILE__/);
});
