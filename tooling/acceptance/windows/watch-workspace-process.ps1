# Read-only, bounded lifetime observation. This never grants provider ownership.
[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidateRange(1,2147483647)][int]$ProcessId,
    [Parameter(Mandatory)][ValidatePattern('^[0-9]+$')][string]$CreatedFileTime,
    [ValidateRange(1,60000)][int]$DurationMilliseconds=60000
)
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$process=Get-Process -Id $ProcessId -ErrorAction Stop
try {
    # Retain the original process handle so an exited/reused PID cannot redirect the wait.
    $handle=$process.Handle
    if($process.StartTime.ToUniversalTime().ToFileTimeUtc().ToString() -cne $CreatedFileTime){throw 'Process identity changed; no observation started.'}
    $started=[DateTimeOffset]::UtcNow
    $timer=[Diagnostics.Stopwatch]::StartNew()
    $exited=$process.WaitForExit($DurationMilliseconds)
    $timer.Stop()
    $code=if($exited){$process.ExitCode}else{$null}
    [pscustomobject]@{
        schema=1;scope='PROCESS_LIFETIME_ONLY';startedAt=$started.ToString('o')
        endedAt=[DateTimeOffset]::UtcNow.ToString('o');elapsedMilliseconds=$timer.ElapsedMilliseconds
        exited=$exited;exitCode=$code
    }|ConvertTo-Json -Compress
} finally {$process.Dispose()}
