#requires -Version 5.1
[CmdletBinding()]
param([switch]$Record, [switch]$SelfTest)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-MC01StartupBlocker {
    param([object[]]$Processes)
    if (@($Processes).Count -gt 0) { return 'EXISTING_GAME_OR_HELPER_PRESERVED' }
    return $null
}

function Get-MC01ObservationBlocker {
    param([object[]]$Processes, [string]$RegisteredImage, [int]$OperatorSession, [string]$FirstIdentity)
    $games = @($Processes | Where-Object { $_.name -eq 'Minecraft.Windows' })
    if ($games.Count -gt 1) { return 'MULTIPLE_GAME_PROCESSES' }
    foreach ($game in $games) {
        if (-not $game.readable) { return 'GAME_IDENTITY_UNKNOWN' }
        if ($game.path -ine $RegisteredImage) { return 'REGISTERED_IMAGE_MISMATCH' }
        if ($game.session -ne $OperatorSession) { return 'OTHER_INTERACTIVE_SESSION' }
        if ($FirstIdentity -and $game.identity -ne $FirstIdentity) { return 'GAME_LIFETIME_CHANGED' }
    }
    return $null
}

function Get-MC01StartupProcesses {
    # Inspect globally by basename; inaccessible identities still block a new recording.
    foreach ($process in @(Get-Process -ErrorAction Stop)) {
        try {
            if ($process.ProcessName -notin @('Minecraft.Windows', 'Minecraft.AdalServer')) { continue }
            $item = [ordered]@{ name=$process.ProcessName; pid=$process.Id; readable=$false
                session=$null; creationFileTime=$null; path=$null; identity=$null }
            try {
                $item.session = $process.SessionId
                $item.creationFileTime = $process.StartTime.ToUniversalTime().ToFileTimeUtc().ToString()
                $item.path = $process.Path
                if (-not $item.path) { throw 'Process image unavailable.' }
                $item.identity = $item.pid.ToString() + ':' + $item.creationFileTime
                $process.Refresh()
                if ($process.HasExited -or
                    $item.creationFileTime -ne $process.StartTime.ToUniversalTime().ToFileTimeUtc().ToString()) {
                    throw 'Process lifetime changed during observation.'
                }
                $item.readable = $true
            } catch { $item.readable = $false }
            [pscustomobject]$item
        } finally { $process.Dispose() }
    }
}

function Invoke-MC01Wpr {
    param([string[]]$Arguments)
    # Native stderr is evidence, not a terminating PowerShell error that bypasses cleanup.
    $previousPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        $output = (& $wpr @Arguments 2>&1 | Out-String)
        [pscustomobject]@{output=$output; exitCode=$LASTEXITCODE}
    } finally { $ErrorActionPreference = $previousPreference }
}

function Save-MC01StartupReceipt {
    param($Receipt, [string]$Path)
    $temporary = $Path + '.new'
    [IO.File]::WriteAllText($temporary, ($Receipt | ConvertTo-Json -Depth 8), [Text.UTF8Encoding]::new($false))
    Move-Item -LiteralPath $temporary -Destination $Path -Force
}

if ($SelfTest) {
    if ($Record) { throw 'SelfTest cannot record.' }
    $sample = [pscustomobject]@{name='Minecraft.Windows'; pid=7; readable=$true; session=1
        creationFileTime='100'; path='C:\fixture\Minecraft.Windows.exe'; identity='7:100'}
    $cases = @(
        @((Get-MC01StartupBlocker @()), $null),
        @((Get-MC01StartupBlocker @($sample)), 'EXISTING_GAME_OR_HELPER_PRESERVED'),
        @((Get-MC01StartupBlocker @([pscustomobject]@{name='Minecraft.AdalServer';readable=$false})),
            'EXISTING_GAME_OR_HELPER_PRESERVED'),
        @((Get-MC01ObservationBlocker @($sample) $sample.path 1 ''), $null),
        @((Get-MC01ObservationBlocker @($sample,$sample) $sample.path 1 ''), 'MULTIPLE_GAME_PROCESSES'),
        @((Get-MC01ObservationBlocker @($sample) 'C:\other.exe' 1 ''), 'REGISTERED_IMAGE_MISMATCH'),
        @((Get-MC01ObservationBlocker @($sample) $sample.path 2 ''), 'OTHER_INTERACTIVE_SESSION'),
        @((Get-MC01ObservationBlocker @($sample) $sample.path 1 '7:99'), 'GAME_LIFETIME_CHANGED')
    )
    $sample.readable = $false
    $cases += ,@((Get-MC01ObservationBlocker @($sample) $sample.path 1 ''), 'GAME_IDENTITY_UNKNOWN')
    foreach ($case in $cases) {
        if ($case[0] -cne $case[1]) { throw ('Guard self-check failed: ' + $case[1]) }
    }
    $wpr = [Diagnostics.Process]::GetCurrentProcess().MainModule.FileName
    $fixtureCommand = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes(
        '[Console]::Error.WriteLine("MC01_NATIVE_ERROR_FIXTURE"); exit 5'))
    $nativeFailure = Invoke-MC01Wpr @('-NoProfile', '-EncodedCommand', $fixtureCommand)
    if ($nativeFailure.exitCode -ne 5 -or
        $nativeFailure.output -notmatch 'MC01_NATIVE_ERROR_FIXTURE' -or $ErrorActionPreference -ne 'Stop') {
        throw 'Native failure preservation self-check failed.'
    }
    [pscustomobject]@{selfChecks=($cases.Count + 1); result='PASS'; recording='NOT_EXECUTED'} | ConvertTo-Json
    return
}

if ($PSVersionTable.PSEdition -ne 'Desktop' -or -not [Environment]::Is64BitProcess) {
    throw 'Use 64-bit Windows PowerShell 5.1; no automatic relaunch or elevation is performed.'
}
$before = @(Get-MC01StartupProcesses)
$blocker = Get-MC01StartupBlocker $before
if ($blocker) {
    [pscustomobject]@{mode='PREFLIGHT'; status=$blocker; processCount=$before.Count
        recording='NOT_EXECUTED'; gameAction='NONE'} | ConvertTo-Json
    if ($Record) { throw 'Recording refused; keep the existing game/helper session untouched.' }
    return
}

. (Join-Path $PSScriptRoot '../../isolation-source/windows/education-target.ps1')
$registeredImage = Resolve-ManagerEducationTarget -RegisteredOnly
$imageHash = (Get-FileHash -LiteralPath $registeredImage -Algorithm SHA256).Hash
$profile = Join-Path $PSScriptRoot 'mc01-startup.wprp'
if ((Get-FileHash -LiteralPath $profile -Algorithm SHA256).Hash -ne
    '3AAAC488F25D093D16DB6B40B1C5AE22CE3DA03ABAD1D4B11FB7A6640567D724') {
    throw 'Recording profile differs from the reviewed Handle/Loader-only profile.'
}
if (-not $Record) {
    [pscustomobject]@{mode='PREFLIGHT'; status='NO_EXISTING_TARGET_OBSERVED'; registeredImageVerified=$true
        durationSeconds=30; bufferMiB=32; recording='NOT_EXECUTED'; gameAction='NONE'
        creatorBinding='UNKNOWN'; operatorLaunchRequired=$true} | ConvertTo-Json
    return
}
$principal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'Explicit administrator recording is required; this script never elevates itself.'
}

$wpr = Join-Path ([Environment]::GetFolderPath([Environment+SpecialFolder]::Windows)) 'System32/wpr.exe'
if (-not (Test-Path -LiteralPath $wpr -PathType Leaf)) { throw 'System WPR is unavailable.' }
$runId = [Guid]::NewGuid().ToString('N')
$instance = 'MivubiMC01Startup_' + $runId
$destination = Join-Path ([IO.Path]::GetTempPath()) $instance
$operatorSession = [Diagnostics.Process]::GetCurrentProcess().SessionId
New-Item -ItemType Directory -Path $destination -ErrorAction Stop | Out-Null
$receiptPath = Join-Path $destination 'receipt.json'
$eventIds = @(('MC01StartupStart_' + $runId), ('MC01StartupStop_' + $runId))
$observations = [Collections.Generic.List[object]]::new()
$lifetimes = @{}
$firstIdentity = ''
$started = $false
$stopSucceeded = $false
$receipt = [ordered]@{
    schema=1; instance=$instance; durationSeconds=30; bufferMiB=32
    scriptSha256=(Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash
    profileSha256=(Get-FileHash -LiteralPath $profile -Algorithm SHA256).Hash
    registeredImageSha256=$imageHash; operatorSession=$operatorSession
    includes=@('Handle','Loader'); excludes=@('ProcessThread','FileIO','Registry','Network','Heap','MemoryPages')
    state='PREPARING'; readyUtc=$null; endedUtc=$null; error=$null
    exactCreator='UNKNOWN'; parentEvidence='NAME_FILTERED_CANDIDATE_ONLY'
    startupCoverage='UNKNOWN'; singletonDecision='UNKNOWN'; gameObserved=$false; providerReady=$false
    startExitCode=$null; stopExitCode=$null; cancelExitCode=$null
    traceFiles=@(); processEvents=@(); sampledLifetimes=@()
}
try {
    foreach ($kind in @('Start','Stop')) {
        $index = if ($kind -eq 'Start') { 0 } else { 1 }
        $query = "SELECT * FROM Win32_Process${kind}Trace WHERE ProcessName='Minecraft.Windows.exe' OR ProcessName='Minecraft.AdalServer.exe'"
        Register-WmiEvent -Query $query -SourceIdentifier $eventIds[$index] | Out-Null
    }
    if (Get-MC01StartupBlocker @(Get-MC01StartupProcesses)) {
        throw 'A game/helper appeared before recording; no launch or recording is permitted.'
    }
    $startResult = Invoke-MC01Wpr @('-start', ($profile + '!MC01Startup'), '-recordtempto', $destination, '-instancename', $instance)
    $receipt.startOutput = $startResult.output
    $receipt.startExitCode = $startResult.exitCode
    if ($startResult.exitCode -ne 0) { throw 'WPR start failed; no ready signal was issued.' }
    $started = $true
    $timer = [Diagnostics.Stopwatch]::StartNew()
    $receipt.readyUtc = [DateTime]::UtcNow.ToString('o')
    $receipt.state = 'RECORDING'
    Save-MC01StartupReceipt $receipt $receiptPath
    Write-Output 'MC01_STARTUP_READY_30_SECONDS: manually open MC01 once using the existing manager official launch route.'
    while ($timer.Elapsed.TotalSeconds -lt 30) {
        $current = @(Get-MC01StartupProcesses)
        $reason = Get-MC01ObservationBlocker $current $registeredImage $operatorSession $firstIdentity
        if ($reason) { throw $reason }
        foreach ($item in $current) {
            if ($item.readable -and -not $lifetimes.ContainsKey($item.identity)) {
                $lifetimes[$item.identity] = $item
                if ($item.name -eq 'Minecraft.Windows' -and -not $firstIdentity) {
                    if ([long]$item.creationFileTime -lt ([DateTime]::Parse($receipt.readyUtc).ToFileTimeUtc())) {
                        throw 'Game started before the ready signal; startup coverage is unknown.'
                    }
                    $firstIdentity = $item.identity
                }
            }
        }
        foreach ($eventId in $eventIds) {
            foreach ($event in @(Get-Event -SourceIdentifier $eventId -ErrorAction SilentlyContinue)) {
                try {
                    $metadata = $event.SourceEventArgs.NewEvent
                    $observations.Add([pscustomobject]@{
                        kind=$(if ($eventId -eq $eventIds[0]) {'start'} else {'stop'})
                        name=[string]$metadata.ProcessName; pid=[uint32]$metadata.ProcessID
                        parentPid=[uint32]$metadata.ParentProcessID; session=[uint32]$metadata.SessionID
                        eventFileTime=[string]$metadata.TIME_CREATED
                        attribution='NAME_CANDIDATE_ONLY'
                    })
                } finally { Remove-Event -EventIdentifier $event.EventIdentifier }
            }
        }
        Start-Sleep -Milliseconds 100
    }
    $receipt.state = 'WINDOW_OBSERVED'
} catch {
    $receipt.state = 'INCOMPLETE'
    $receipt.error = $_.Exception.Message
} finally {
    if ($started) {
        $receipt.statusBeforeStop = Invoke-MC01Wpr @('-status', 'collectors', '-details', '-instancename', $instance)
        $stopResult = Invoke-MC01Wpr @('-stop', (Join-Path $destination 'startup.etl'), '-skipPdbGen', '-instancename', $instance)
        $receipt.stopOutput = $stopResult.output
        $receipt.stopExitCode = $stopResult.exitCode
        $stopSucceeded = $stopResult.exitCode -eq 0
        if (-not $stopSucceeded) {
            # Only this successfully started GUID instance is eligible for cancellation.
            $cancelResult = Invoke-MC01Wpr @('-cancel', '-instancename', $instance)
            $receipt.cancelOutput = $cancelResult.output
            $receipt.cancelExitCode = $cancelResult.exitCode
            $receipt.state = 'INCOMPLETE'
        }
        $receipt.finalStatus = Invoke-MC01Wpr @('-status', '-instancename', $instance)
    }
    foreach ($eventId in $eventIds) {
        Unregister-Event -SourceIdentifier $eventId -ErrorAction SilentlyContinue
        Get-Event -SourceIdentifier $eventId -ErrorAction SilentlyContinue | Remove-Event -ErrorAction SilentlyContinue
    }
    # SkipMerge may produce primary + rundown names instead of startup.etl.
    $receipt.traceFiles = @(Get-ChildItem -LiteralPath $destination -Filter '*.etl' -File | ForEach-Object {
        [pscustomobject]@{name=$_.Name; bytes=$_.Length; sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash}
    })
    if (-not $stopSucceeded -or $receipt.traceFiles.Count -eq 0) { $receipt.state = 'INCOMPLETE' }
    $receipt.processEvents = @($observations.ToArray())
    $receipt.sampledLifetimes = @($lifetimes.Values)
    $receipt.gameObserved = [bool]$firstIdentity
    $receipt.endedUtc = [DateTime]::UtcNow.ToString('o')
    Save-MC01StartupReceipt $receipt $receiptPath
}
Write-Output ('Local receipt: ' + $receiptPath)
if ($receipt.state -eq 'INCOMPLETE') { throw 'Capture incomplete; retain the local receipt and raw files for review.' }
