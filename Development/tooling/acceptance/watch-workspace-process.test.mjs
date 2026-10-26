import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

test('lifetime collector pins identity, reports timeout without exit and preserves exit code', {skip:process.platform!=='win32'},()=>{
  const collector=fileURLToPath(new URL('./windows/watch-workspace-process.ps1',import.meta.url)).replaceAll("'","''");
  const script=`
    $ErrorActionPreference='Stop'
    $child=Start-Process -FilePath (Join-Path $PSHOME 'powershell.exe') -ArgumentList '-NoProfile -NonInteractive -Command "Start-Sleep -Milliseconds 1800; exit 17"' -WindowStyle Hidden -PassThru
    try {
      $handle=$child.Handle
      $birth=$child.StartTime.ToUniversalTime().ToFileTimeUtc().ToString()
      $rejected=$false
      try {& '${collector}' -ProcessId $child.Id -CreatedFileTime '1' -DurationMilliseconds 1|Out-Null} catch {$rejected=$_.Exception.Message -like '*identity changed*'}
      if(-not $rejected){throw 'Changed process identity was accepted'}
      $idle=& '${collector}' -ProcessId $child.Id -CreatedFileTime $birth -DurationMilliseconds 1|ConvertFrom-Json
      if($idle.exited -or $null -ne $idle.exitCode){throw 'Timeout was reported as exit'}
      $done=& '${collector}' -ProcessId $child.Id -CreatedFileTime $birth -DurationMilliseconds 5000|ConvertFrom-Json
      if(-not $done.exited -or $done.exitCode -ne 17){throw 'Original process exit code was lost'}
      @{identityRejected=$rejected;idle=$idle;done=$done}|ConvertTo-Json -Depth 4 -Compress
    } finally {
      if(-not $child.WaitForExit(5000)){$child.Kill()}
      $child.Dispose()
    }
  `;
  const result=spawnSync('powershell.exe',['-NoProfile','-NonInteractive','-Command',script],{encoding:'utf8',windowsHide:true,timeout:15000});
  assert.equal(result.status,0,result.stderr||String(result.error));
  const observed=JSON.parse(result.stdout.trim());
  assert.equal(observed.identityRejected,true);
  assert.equal(observed.idle.exited,false);
  assert.equal(observed.done.exitCode,17);
  assert.equal(observed.done.scope,'PROCESS_LIFETIME_ONLY');
  assert.equal('pid' in observed.done,false);
});
