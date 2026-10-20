import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
const script = new URL('./prepare-owned-v1186-test.ps1', import.meta.url);
const source = readFileSync(script, 'utf8').replaceAll('\r\n', '\n');
const windows = process.platform === 'win32';
function ps(command) {
  return spawnSync('powershell.exe', ['-NoProfile','-NonInteractive','-Command',command],
    {encoding:'utf8', timeout:15000, windowsHide:true});
}
test('preparation remains separate from unsigned build and service activation', () => {
  assert.match(source, /SIGNED_TEST_COPY_READY_BOOT_UNCHANGED/);
  assert.match(source, /REBOOT_REQUIRED_NO_SERVICE_CHANGES/);
  assert.match(source, /NonExportable/);
  assert.match(source, /productionInstallPermitted=\$false/);
  assert.doesNotMatch(source, /\b(?:Start-Service|Stop-Service|New-Service|Restart-Computer|shutdown\.exe|sc\.exe|kmdutil\.exe)\b/i);
  assert.match(source, /manifest\.artifacts\.Count -ne 25/);
  assert.match(source, /binaries\.Count -ne 16/);
  assert.match(source, /Artifact changed during copy/);
  assert.ok(source.indexOf('Assert-TestSecurity\n') < source.indexOf('$cert = New-SelfSignedCertificate'));
  assert.match(source, /\/enum','\{current\}','\/v/);
  assert.match(source, /Boot entry changed since preparation/);
  assert.match(source, /Signed artifact changed/);
  assert.match(source, /Original boot setting readback failed/);
  assert.match(source, /Certificate identity mismatch/);
  assert.match(source, /Native operation timed out after 60 seconds/);
});
test('PowerShell parses the full preparation script without execution', {skip:!windows}, () => {
  const path = decodeURIComponent(script.pathname).replace(/^\//,'').replaceAll("'", "''");
  const result = ps(`$t=$null;$e=$null;[Management.Automation.Language.Parser]::ParseFile('${path}',[ref]$t,[ref]$e)|Out-Null;if($e.Count){$e|Out-String|Write-Error;exit 1}`);
  assert.equal(result.status, 0, result.stderr + result.stdout);
});
test('bounded native runner preserves spaced arguments and rejects failure and quotes', {skip:!windows}, () => {
  const native = source.slice(source.indexOf('function Native('), source.indexOf('function Assert-Administrator'));
  const result = ps(`${native}
    $ErrorActionPreference='Stop'
    $exe=Join-Path $PSHOME 'powershell.exe'
    $out=Native $exe @('-NoProfile','-Command','Write-Output spaced_argument_value')
    if($out.Trim() -cne 'spaced_argument_value'){throw 'output lost'}
    try { Native $exe @('-NoProfile','-Command','exit 7'); throw 'accepted failure' }
    catch { if($_.Exception.Message -notmatch 'exit=7'){throw} }
    try { Native $exe @('bad"argument'); throw 'accepted quote' }
    catch { if($_.Exception.Message -notmatch 'Unsafe native argument'){throw} }
    exit 0
  `);
  assert.equal(result.status, 0, result.stderr + result.stdout);
});
