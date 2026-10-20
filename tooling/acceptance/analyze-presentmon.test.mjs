import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync, symlinkSync, lstatSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { analyzePresentMonCsv } from './analyze-presentmon.mjs';

const header = 'Application,ProcessID,SwapChainAddress,MsBetweenPresents,Extra\n';
const row = (pid='1234', val='16.7', app='Minecraft.Windows.exe', chain='0x123A') =>
  app+','+pid+','+chain+','+val+',"Note, optional"\n';
const valid = () => header + Array.from({length:100}, (_,i) =>
  row('1234',i===99?'50':i===98?'34':'16.7')).join('');

test('100 selected intervals, quantiles, no raw process IDs or names emitted', () => {
  const result=analyzePresentMonCsv(valid(),'1234');
  assert.equal(result.observedIntervals,100);
  assert.equal(result.p50PresentIntervalMs,16.7);
  assert.equal(result.p95PresentIntervalMs,16.7);
  assert.equal(result.p99PresentIntervalMs,34);
  assert.equal(result.over33msIntervals,2);
  assert.equal(result.over50msIntervals,0);
  assert.equal(result.fpsVerified,false);
  assert.equal(result.accountIsolationVerified,false);
  assert.ok(!JSON.stringify(result).includes('1234'));
  assert.ok(!JSON.stringify(result).includes('Minecraft.Windows.exe'));
});

test('select one PID, excluding all other processes before interval statistics', () => {
  const s=header+Array.from({length:30},()=>row('1234')).join('')+
    Array.from({length:100},()=>row('9876','100','MinecraftEducation.exe')).join('');
  assert.equal(analyzePresentMonCsv(s,'1234').observedIntervals,30);
});

test('old lowercase PresentMon header and BOM accepted', () => {
  const s='\ufeff'+header.replace('MsBetweenPresents','msBetweenPresents')+
    Array.from({length:30},()=>row()).join('');
  assert.equal(analyzePresentMonCsv(s,'1234').p95PresentIntervalMs,16.7);
});

test('v2 FrameTime alone is not silently substituted as present interval', () => {
  const s=header.replace('MsBetweenPresents','FrameTime')+
    Array.from({length:30},()=>row()).join('');
  assert.throws(()=>analyzePresentMonCsv(s,'1234'), /MsBetweenPresents/);
});

test('multiple swapchains or PID reuse refuse contaminated game evidence', () => {
  const s=header+Array.from({length:30},()=>row()).join('')+
    row('1234','16.7','Minecraft.Windows.exe','0xFFFF');
  assert.throws(()=>analyzePresentMonCsv(s,'1234'), /multiple swap chains/);
  const t=header+Array.from({length:30},()=>row()).join('')+
    row('1234','16.7','SomeOtherApp.exe');
  assert.throws(()=>analyzePresentMonCsv(t,'1234'), /unexpected application/);
});

test('no process selection, invalid PID, invalid interval and malformed CSV fail closed', () => {
  for(const pid of ['0','001','-1','4294967296','12;drop','',1234])
    assert.throws(()=>analyzePresentMonCsv(valid(),pid));
  for(const bad of ['0','NaN','Infinity','-20','NAH','1e3','60001'])
    assert.throws(()=>analyzePresentMonCsv(
      header+Array.from({length:30},()=>row('1234',bad)).join(''),'1234'));
  assert.throws(()=>analyzePresentMonCsv(header+row(),'1234'),/insufficient/);
  assert.throws(()=>analyzePresentMonCsv(
    'Application,ProcessID,Application,SwapChainAddress,MsBetweenPresents\n','1234'), /duplicate/);
  assert.throws(()=>analyzePresentMonCsv(
    header+Array.from({length:30},()=>row('1234','NA')).join(''),'1234'));
  assert.throws(()=>analyzePresentMonCsv(
    header+Array.from({length:30},()=>row()).join('')+'"garbage\n','1234'), /unterminated/);
});

test('one missing interval retains coverage, but extensive missing data fails', () => {
  const s=header+Array.from({length:50},(_,i)=>row('1234',i===0?'NA':'16.7')).join('');
  const r=analyzePresentMonCsv(s,'1234');
  assert.equal(r.unavailableIntervals,1);
  assert.equal(r.observedIntervals,49);
  assert.equal(r.presentationCoveragePercent,98);
  assert.throws(()=>analyzePresentMonCsv(header+
    Array.from({length:50},(_,i)=>row('1234',i<3?'NA':'16.7')).join(''),'1234'),
    /too many missing/);
});

test('CLI analyzes existing CSV without raw PID output and rejects invalid input', () => {
  const dir=mkdtempSync(join(tmpdir(),'mivubi-frame-'));
  try {
    const csv=join(dir,'capture.csv');
    writeFileSync(csv, valid());
    const path=fileURLToPath(new URL('./analyze-presentmon.mjs',import.meta.url));
    const good=spawnSync(process.execPath,[path,'--csv',csv,'--process-id','1234'],
      {encoding:'utf8'});
    assert.equal(good.status,0,good.stderr);
    assert.equal(JSON.parse(good.stdout).observedIntervals,100);
    assert.ok(!good.stdout.includes('1234'));
    const bad=spawnSync(process.execPath,[path,'--csv',csv,'--process-id','wrong'],
      {encoding:'utf8'});
    assert.equal(bad.status,1);
    assert.ok(!bad.stderr.includes(csv));
  } finally {
    rmSync(dir,{recursive:true,force:true});
  }
});


test('CLI refuses a real symbolic link input, including an unprivileged Windows junction', t => {
  const dir=mkdtempSync(join(tmpdir(),'mivubi-frame-link-'));
  try {
    const csv=join(dir,'capture.csv');
    writeFileSync(csv, valid());
    const alias=join(dir,'alias.csv');
    try {
      symlinkSync(csv,alias,'file');
    } catch (error) {
      if (process.platform !== 'win32' || !['EPERM','EACCES'].includes(error.code)) throw error;
      // A junction exercises the same real link-input rejection without changing host policy.
      symlinkSync(dir,alias,'junction');
      t.diagnostic('File-symlink privilege unavailable; actual Windows directory junction tested.');
    }
    assert.equal(lstatSync(alias).isSymbolicLink(),true);
    const path=fileURLToPath(new URL('./analyze-presentmon.mjs',import.meta.url));
    const link=spawnSync(process.execPath,[path,'--csv',alias,'--process-id','1234'],{encoding:'utf8'});
    assert.equal(link.status,1);
    assert.match(link.stderr,/CSV must be an ordinary file/);
    assert.ok(!link.stderr.includes(alias));
  } finally {
    rmSync(dir,{recursive:true,force:true});
  }
});
