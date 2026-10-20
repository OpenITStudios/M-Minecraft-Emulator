import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const source=readFileSync(new URL('./owned-manager.ps1',import.meta.url),'utf8');
const probe=readFileSync(new URL('./probe-v1186-runtime.ps1',import.meta.url),'utf8');
const ps=code=>spawnSync('powershell.exe',['-NoProfile','-NonInteractive','-Command',code],{encoding:'utf8',timeout:15000,windowsHide:true});
test('launch cooldown compares UTC instants on non-UTC Windows',{skip:process.platform!=='win32'},()=>{
  const expression=source.match(/Demand \((\(\[DateTimeOffset\]::UtcNow-\[DateTimeOffset\]::Parse\(\$receipt.runtime.\$launchKey\)\).TotalSeconds -ge 30)\)/)?.[1];
  assert.ok(expression,'production cooldown expression');
  const r=ps(`$launchKey='managerLaunch02';$receipt=[pscustomobject]@{runtime=[pscustomobject]@{managerLaunch02=[DateTimeOffset]::UtcNow.AddSeconds(-31).ToOffset([TimeSpan]::FromHours(7)).ToString('o')}};if(-not (${expression})){throw 'elapsed cooldown rejected'};$receipt.runtime.managerLaunch02=[DateTimeOffset]::UtcNow.AddSeconds(-5).ToOffset([TimeSpan]::FromHours(7)).ToString('o');if(${expression}){throw 'recent launch admitted'}`);
  assert.equal(r.status,0,r.stdout+r.stderr);
});
test('owned manager confines mutations to profiles and attributed windows',()=>{
  assert.doesNotMatch(source,/ChangeServiceConfig|New-Service|Stop-Service|TerminateProcess|Remove-Item|bcdedit|Export-PfxCertificate/);
  for(const required of ['Private driver/ABI/Test Mode verification failed','Active service belongs to another provider','Recent launch request','Previous private launcher is still running','Profile exists; refusing','partial profile attempt','account-bearing profile data','Game belongs to another Windows session','processes.Count -eq 1','createdFileTime -ceq $target.createdFileTime'])assert.ok(source.includes(required),required);
  assert.match(source,/--owned-profile-create/);
  assert.match(source,/HelperOnly/);
  assert.doesNotMatch(source,/-File.*\$scriptPath/);
  assert.ok(source.indexOf("state='PARTIAL'")<source.indexOf('foreach ($step in $plan.guardedSteps)'));
});
test('manager script parses without executing any Windows operation',{skip:process.platform!=='win32'},()=>{
  const file=fileURLToPath(new URL('./owned-manager.ps1',import.meta.url)).replaceAll("'","''");
  const r=ps(`$t=$null;$e=$null;[Management.Automation.Language.Parser]::ParseFile('${file}',[ref]$t,[ref]$e)|Out-Null;if($e.Count){throw ($e|Out-String)}`);
  assert.equal(r.status,0,r.stdout+r.stderr);
});
test('window action and box query refuse stale/foreign identities before native effects',{skip:process.platform!=='win32'},()=>{
  const code=probe.split("Add-Type -TypeDefinition @'")[1].split("'@")[0];
  const r=ps(`$ErrorActionPreference='Stop';Add-Type -TypeDefinition @'\n${code}\n'@
    try{[MivubiManagerWindows]::Act($PID,0,0,'Close');throw 'unsafe close'}catch{if($_.Exception.ToString() -notmatch 'Stale process/window identity'){throw}}
    try{[MivubiRuntimeProbe]::BoxPids('must-not-load.dll','foreign');throw 'unsafe box'}catch{if($_.Exception.ToString() -notmatch 'Unknown owned profile'){throw}}
    exit 0`);
  assert.equal(r.status,0,r.stdout+r.stderr);
});
