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
test('shared Education resolver rejects another launcher target without launching',{skip:process.platform!=='win32'},()=>{
  const file=fileURLToPath(new URL('./education-target.ps1',import.meta.url)).replaceAll("'","''");
  const r=ps(`$ErrorActionPreference='Stop';. '${file}';try{Assert-ManagerRegisteredAppId '{6D809377-6AF0-444B-8957-A3773F02200E}\\Minecraft.Windows.exe' 'C:\\wrong\\Minecraft.Windows.exe';throw 'foreign launcher accepted'}catch{if($_.Exception.ToString() -notmatch 'different Minecraft installation'){throw}};exit 0`);
  assert.equal(r.status,0,r.stdout+r.stderr);
});
test('launch observation waits for an attributed window instead of a PID alone',{skip:process.platform!=='win32'},()=>{
  const loop=source.match(/do \{\$after=Snapshot 'Launch'[^\n]+while\(\[DateTime\]::UtcNow -lt \$deadline\)/)?.[0];
  assert.ok(loop,'production launch observation loop');
  const r=ps(`$ErrorActionPreference='Stop';$script:count=0;$request=@{clientId='02'};$exitCode=0;$deadline=[DateTime]::UtcNow.AddSeconds(3);function Snapshot($operation,$exitCode){$script:count++;return @{clients=@(@{clientId='02';status=$(if($script:count -eq 1){'STARTING_OR_CLOSING'}else{'RUNNING'});processes=@(@{pid=1})})}};${loop};if($count -ne 2 -or $found.status -ne 'RUNNING'){throw 'Stopped observation before window readiness'};exit 0`);
  assert.equal(r.status,0,r.stdout+r.stderr);
});
test('a launched game is explicitly presented and focus refusal preserves launch outcome',{skip:process.platform!=='win32'},()=>{
  const file=fileURLToPath(new URL('./owned-manager.ps1',import.meta.url)).replaceAll("'","''");
  const r=ps(`$ErrorActionPreference='Stop';$t=$null;$e=$null;$ast=[Management.Automation.Language.Parser]::ParseFile('${file}',[ref]$t,[ref]$e);$f=$ast.Find({param($n)$n -is [Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -eq 'Show-LaunchedWindow'},$true);if(-not $f){throw 'Launch never presents the game window'};. ([ScriptBlock]::Create($f.Extent.Text));Add-Type 'public static class MivubiManagerWindows {public static int Calls;public static bool Refuse;public static void Act(int pid,long time,long hwnd,string operation){if(pid!=77||time!=123||hwnd!=456||operation!="Focus")throw new System.Exception("wrong identity");Calls++;if(Refuse)throw new System.Exception("focus refused");}}';$slot=@{status='RUNNING';settingsVerified=$true;processes=@(@{pid=77;createdFileTime='123';windowHandle='0x1c8'})};$result=Show-LaunchedWindow $slot;if($null -ne $result -or [MivubiManagerWindows]::Calls -ne 1){throw 'No automatic focus'};[MivubiManagerWindows]::Refuse=$true;$result=Show-LaunchedWindow $slot;if($result -notmatch 'focus refused'){throw 'Lost focus refusal'};exit 0`);
  assert.equal(r.status,0,r.stdout+r.stderr);
});
