import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,mkdtempSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
const source=readFileSync(new URL('./test-owned-v1186-runtime.ps1',import.meta.url),'utf8').replaceAll('\r\n','\n');
const probe=readFileSync(new URL('./probe-v1186-runtime.ps1',import.meta.url),'utf8');
function ps(command) { return spawnSync('powershell.exe',['-NoProfile','-NonInteractive','-Command',command],{encoding:'utf8',timeout:15000,windowsHide:true}); }
test('test takeover preserves existing service registrations and authenticates rollback images',()=>{
  assert.match(source,/ChangeServiceConfig\(service,0xffffffff,0xffffffff,0xffffffff,path,null,IntPtr.Zero,null,null,null,null\)/);
  assert.doesNotMatch(source,/\b(?:DeleteService|CreateService|Remove-Item|reg\.exe|Restart-Computer)\b/);
  assert.match(source,/Foreign SCM path; automatic restoration refused/);
  assert.match(source,/SCM ownership changed before redirect/);
  assert.match(source,/ACTIVATION_FAILED_RESTORE_REQUIRED/);
  assert.match(source,/Global\\MIVUBI.OwnedV1186.TestRuntime/);
  assert.match(source,/Manual reboot has not occurred/);
  assert.match(source,/\.abi -ne 0x57230/);
  assert.match(source,/5\.73\.6-MIVU/);
  assert.match(source,/Lifecycle helper hash changed/);
  assert.ok(source.indexOf("Helper $officialRoot 'stop' 'SbieDrv'")<source.indexOf('[MivubiTestScm]::Image($service.name,$service.ownedImage)'));
  assert.match(probe,/query\(IntPtr.Zero,true,0,null,ref first\)/);
  assert.match(probe,/first!=second/);
  assert.match(probe,/NtQuerySystemInformation\(103/);
  assert.match(probe,/finally \{ FreeLibrary\(module\); \}/);
});
test('all-session busy/unknown results stop execution; image hashes and signer mismatch fail closed',{skip:process.platform!=='win32'},()=>{
  const verify=source.slice(source.indexOf('function FileFromImage'),source.indexOf('function Probe'));
  const idle=source.slice(source.indexOf('function Idle'),source.indexOf('function Helper'));
  const result=ps(`${verify}\n${idle}
    $ErrorActionPreference='Stop'
    function Ordinary($path) {}
    function Digest($path) { 'expected' }
    function Get-AuthenticodeSignature { [pscustomobject]@{Status='Valid';SignerCertificate=[pscustomobject]@{Thumbprint='signer'}} }
    VerifyImage '"C:\\Program Files\\Sandboxie-Plus\\SbieSvc.exe"' 'expected' 'signer'
    if((FileFromImage '\\??\\C:\\driver.sys') -cne 'C:\\driver.sys'){throw 'kernel path normalization wrong'}
    try { VerifyImage 'C:\\driver.sys' 'wrong' 'signer';throw 'accepted changed image' } catch {if($_.Exception.Message -notmatch 'hash mismatch'){throw}}
    try { VerifyImage 'C:\\driver.sys' 'expected' 'wrong';throw 'accepted foreign signer' } catch {if($_.Exception.Message -notmatch 'signature mismatch'){throw}}
    function Probe($dll) { [pscustomobject]@{allSessionBoxedProcesses=1} }
    try { Idle 'test';throw 'accepted active sandbox' } catch {if($_.Exception.Message -notmatch 'Sandbox processes are active'){throw}}
    function Probe($dll) { throw 'inventory unavailable' }
    try { Idle 'test';throw 'accepted unknown inventory' } catch {if($_.Exception.Message -notmatch 'inventory unavailable'){throw}}
    exit 0`);
  assert.equal(result.status,0,result.stdout+result.stderr);
});
test('runtime and read-only probe scripts parse without privileged execution',{skip:process.platform!=='win32'},()=>{
  for(const name of ['test-owned-v1186-runtime.ps1','probe-v1186-runtime.ps1']){
    const path=decodeURIComponent(new URL(name,import.meta.url).pathname).replace(/^\//,'').replaceAll("'","''");
    const r=ps(`$t=$null;$e=$null;[Management.Automation.Language.Parser]::ParseFile('${path}',[ref]$t,[ref]$e)|Out-Null;if($e.Count){throw ($e|Out-String)}`);
    assert.equal(r.status,0,r.stdout+r.stderr);
  }
});
test('failure before SCM redirection restores without unloading the original driver',{skip:process.platform!=='win32'},()=>{
  const start=source.indexOf('        if ($ownedLive) {');
  const end=source.indexOf("        Helper $officialRoot 'start' 'SbieDrv'",start);
  assert.ok(start>0 && end>start);
  const result=ps(`$ErrorActionPreference='Stop';$ownedLive=$false
    function Helper { throw 'must not stop original driver' }
    function Image { throw 'must not redirect original images' }
    ${source.slice(start,end)}
    exit 0`);
  assert.equal(result.status,0,result.stdout+result.stderr);
  assert.match(source,/runtimeHistory \+= \$receipt.runtime/);
  assert.match(source,/Exit the official Sandboxie manager normally/);
  assert.ok(source.indexOf('Exit the official Sandboxie manager normally')<source.indexOf("Helper $officialRoot 'stop' 'SbieSvc'"));
});
test('first-client adapter refuses occupied profiles and uses canonical guarded readbacks',()=>{
  const dir=mkdtempSync(join(tmpdir(),'owned-profile-'));
  try {
    const input=join(dir,'inventory.json');
    const cli=fileURLToPath(new URL('./client02-profile-plan.mjs',import.meta.url));
    writeFileSync(input,JSON.stringify({schema:1,boxes:[]}));
    const result=spawnSync(process.execPath,[cli,input],{encoding:'utf8',timeout:10000,windowsHide:true});
    assert.equal(result.status,0,result.stderr);
    const plan=JSON.parse(result.stdout);
    assert.equal(plan.profiles[0].box,'MivubiEdu02');
    assert.equal(plan.guardedSteps[0].arguments.join(' '),'set MivubiEdu02 Enabled n');
    assert.equal(plan.guardedSteps.at(-1).operation,'REQUIRE_READBACK');
    writeFileSync(input,JSON.stringify({schema:1,boxes:['mivubiedu02']}));
    const occupied=spawnSync(process.execPath,[cli,input],{encoding:'utf8',timeout:10000,windowsHide:true});
    assert.notEqual(occupied.status,0);
    assert.match(occupied.stderr,/already exists/);
  } finally {rmSync(dir,{recursive:true,force:true});}
  assert.match(source,/A profile attempt already exists/);
  assert.match(source,/PARTIAL_REQUIRES_REVIEW/);
  assert.match(source,/Launch the game from a normal unelevated/);
  assert.match(source,/Profile changed since configuration/);
});
test('game attribution rejects PID reuse before touching a vendor DLL',{skip:process.platform!=='win32'},()=>{
  const code=probe.split("Add-Type -TypeDefinition @'")[1].split("'@")[0];
  const result=ps(`$ErrorActionPreference='Stop';Add-Type -TypeDefinition @'\n${code}\n'@
    try {[MivubiRuntimeProbe]::Game('must-not-load.dll',$PID,0);throw 'accepted wrong start time'}
    catch {if($_.Exception.ToString() -notmatch 'PID reused before attribution'){throw}}
    exit 0`);
  assert.equal(result.status,0,result.stdout+result.stderr);
  assert.match(probe,/one!=two \|\| one!=session/);
  assert.match(probe,/PID reused after attribution/);
  assert.match(probe,/Private driver required for owned game attribution/);
});
