import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,mkdtempSync,rmSync} from 'node:fs';
import {resolve,join} from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {planOwnedMinecraftVirtual} from '../../../engine/isolation-native/owned-runtime/minecraft-profiles.mjs';
const source=readFileSync(new URL('./owned-manager.ps1',import.meta.url),'utf8');
const probe=readFileSync(new URL('./probe-v1186-runtime.ps1',import.meta.url),'utf8');
const ps=code=>spawnSync('powershell.exe',['-NoProfile','-NonInteractive','-Command',code],{encoding:'utf8',timeout:15000,windowsHide:true});

test('manager blocks launch readiness for missing or widened game-data isolation',{skip:process.platform!=='win32'},()=>{
  const start=source.indexOf('function SettingsMatch('),end=source.indexOf('\nfunction ',start+1);
  assert.ok(start>=0&&end>start);
  const plan=JSON.stringify(planOwnedMinecraftVirtual('02',[])).replaceAll("'","''");
  const r=ps(`$ErrorActionPreference='Stop';$plan='${plan}'|ConvertFrom-Json;$ini='synthetic';$script:mode='match';$script:checks=0
    function CanonicalPlan {return $plan}
    function Native([string]$exe,[string[]]$arguments){
      $step=@($plan.guardedSteps|Where-Object {$_.operation -eq 'REQUIRE_READBACK' -and $_.arguments[2] -ceq $arguments[2]})[-1]
      if($arguments[2] -ceq 'WriteFilePath'){$script:checks++;if($script:mode -ceq 'wide'){return '%AppData%'};if($script:mode -ceq 'missing'){return ''}}
      return $step.expectedValue
    }
    ${source.slice(start,end)}
    if(-not (SettingsMatch '02') -or $checks -ne 1){throw 'Complete profile refused or data rule not checked'}
    foreach($mode in @('wide','missing')){$script:mode=$mode;if(SettingsMatch '02'){throw 'Host read-through profile accepted'}};exit 0`);
  assert.equal(r.status,0,r.stdout+r.stderr);
});

test('long-lived launcher does not retain the manager observation pipes',{skip:process.platform!=='win32'},async()=>{
  const directory=mkdtempSync(resolve('workspace/launcher-pipes-'));
  const binary=join(directory,'Synthetic.exe');
  const quoted=binary.replaceAll("'","''");
  const setting=source.match(/\$start\.UseShellExecute=\$(?:true|false)/)?.[0];
  assert.ok(setting,'production launcher shell policy');
  try{
    const compiled=ps(`Add-Type -TypeDefinition 'public static class Fixture {public static void Main(){System.Threading.Thread.Sleep(4000);}}' -OutputAssembly '${quoted}' -OutputType WindowsApplication`);
    assert.equal(compiled.status,0,compiled.stdout+compiled.stderr);
    const began=Date.now();
    const result=ps(`$ErrorActionPreference='Stop';$start=[Diagnostics.ProcessStartInfo]::new();$start.FileName='${quoted}';${setting};$child=[Diagnostics.Process]::Start($start);$child.Dispose();Write-Output 'observation-complete'`);
    const elapsed=Date.now()-began;
    assert.equal(result.status,0,result.stdout+result.stderr);
    assert.match(result.stdout,/observation-complete/);
    assert.ok(elapsed<2500,`Observation waited ${elapsed} ms for synthetic game exit`);
  }finally{
    // The synthetic GUI exits itself; never terminate a real process for this test.
    await new Promise(resolve=>setTimeout(resolve,4200));
    rmSync(directory,{recursive:true});
  }
});
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
test('native attribution crosses PowerShell without null-to-empty profile coercion',{skip:process.platform!=='win32'},()=>{
  const code=probe.split("Add-Type -TypeDefinition @'")[1].split("'@")[0];
  const call=source.match(/\$identity=\[MivubiRuntimeProbe\]::(?:NativeGame|Game)\(\$dll,\$process.Id,\$started.Ticks(?:,\$null)?\)/)?.[0];
  assert.ok(call,'production native attribution call');
  const r=ps(`$ErrorActionPreference='Stop';Add-Type -TypeDefinition @'\n${code}\n'@; $process=Get-Process -Id $PID;$started=$process.StartTime.ToUniversalTime();$dll='must-not-exist-native-regression.dll';try{${call};throw 'Unexpected DLL acceptance'}catch{if($_.Exception.ToString() -notmatch 'Query DLL load failed'){throw}};exit 0`);
  assert.equal(r.status,0,r.stdout+r.stderr);
});

test('window discovery retains hidden game tabs, ignores browser windows and rejects two game windows',{skip:process.platform!=='win32'},()=>{
  const code=probe.split("Add-Type -TypeDefinition @'")[1].split("'@")[0];
  const fixture=String.raw`
using System;
using System.Runtime.InteropServices;
public static class SyntheticGameWindows {
  delegate IntPtr Proc(IntPtr h,uint m,IntPtr w,IntPtr l);
  static Proc procedure=DefWindowProcW;
  [StructLayout(LayoutKind.Sequential,CharSet=CharSet.Unicode)] struct Class {
    public uint style; public Proc proc; public int a,b; public IntPtr module,icon,cursor,background;
    public string menu,name;
  }
  [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern ushort RegisterClassW(ref Class c);
  [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern IntPtr CreateWindowExW(uint ex,string cls,string title,uint style,int x,int y,int w,int h,IntPtr parent,IntPtr menu,IntPtr module,IntPtr data);
  [DllImport("user32.dll")] static extern IntPtr DefWindowProcW(IntPtr h,uint m,IntPtr w,IntPtr l);
  [DllImport("user32.dll")] public static extern bool DestroyWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h,int command);
  public static IntPtr Create(string name){
    var c=new Class {proc=procedure,name=name}; RegisterClassW(ref c);
    var h=CreateWindowExW(0,name,"Synthetic only",0x00cf0000,20,20,480,320,IntPtr.Zero,IntPtr.Zero,IntPtr.Zero,IntPtr.Zero);
    if(h==IntPtr.Zero)throw new Exception("Fixture creation failed");return h;
  }
}`;
  const r=ps(`$ErrorActionPreference='Stop';Add-Type -TypeDefinition @'\n${code}\n'@;Add-Type -TypeDefinition @'\n${fixture}\n'@
    $one=[SyntheticGameWindows]::Create('OGLES');$browser=[SyntheticGameWindows]::Create('Chrome_WidgetWin_0');$boxed=[SyntheticGameWindows]::Create('Sandbox:MivubiEdu02:OGLES');$two=[IntPtr]::Zero;
    try{
      if([MivubiManagerWindows]::GameWindow($PID,'MivubiEdu02') -ne $boxed){throw 'Canonical boxed game lost'};
      if([MivubiManagerWindows]::GameWindow($PID,'MivubiEdu03') -ne [IntPtr]::Zero){throw 'Foreign box class accepted'};
      [void][SyntheticGameWindows]::ShowWindow($one,4);
      if([MivubiManagerWindows]::GameWindow($PID) -ne $one){throw 'Visible game confused with hidden browser'};
      [void][SyntheticGameWindows]::ShowWindow($one,0);
      if([MivubiManagerWindows]::GameWindow($PID) -ne $one){throw 'Hidden tab lost'};
      $two=[SyntheticGameWindows]::Create('OGLES');
      try{[MivubiManagerWindows]::GameWindow($PID);throw 'Ambiguous windows accepted'}catch{if($_.Exception.ToString() -notmatch 'Ambiguous Education'){throw}}
    }finally{if($two -ne [IntPtr]::Zero){[void][SyntheticGameWindows]::DestroyWindow($two)};[void][SyntheticGameWindows]::DestroyWindow($one);[void][SyntheticGameWindows]::DestroyWindow($browser);[void][SyntheticGameWindows]::DestroyWindow($boxed)};exit 0`);
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
