#requires -Version 5.1
[CmdletBinding()]
param([switch]$Record, [switch]$SelfTest, [switch]$Reactivation, [switch]$LifecycleSelfTest)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if (($SelfTest -and ($Record -or $Reactivation -or $LifecycleSelfTest)) -or
    ($LifecycleSelfTest -and ($Record -or $Reactivation))) {
    throw 'Self-test modes cannot activate a game or record.'
}

function Get-MC01StartupBlocker {
    param([object[]]$Processes)
    if (@($Processes).Count -gt 0) { return 'EXISTING_GAME_OR_HELPER_PRESERVED' }
    return $null
}

function Get-MC01TraceBudgetBlocker {
    param([long]$Bytes)
    if ($Bytes -ge 268435456) { return 'TRACE_SIZE_THRESHOLD_REACHED' }
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


function Get-MC01ReactivationBlocker {
    param([object[]]$Processes, [string]$RegisteredImage, [int]$OperatorSession)
    if (@($Processes | Where-Object { $_.name -eq 'Minecraft.Windows' }).Count -ne 1) {
        return 'EXACTLY_ONE_EXISTING_GAME_REQUIRED'
    }
    return Get-MC01ObservationBlocker $Processes $RegisteredImage $OperatorSession ''
}
function Remove-MC01LifecycleObserver {
    param([string[]]$EventIds)
    foreach ($id in $EventIds) {
        Unregister-Event -SourceIdentifier $id -ErrorAction SilentlyContinue
        Get-Event -SourceIdentifier $id -ErrorAction SilentlyContinue | Remove-Event -ErrorAction SilentlyContinue
    }
    if (@(Get-EventSubscriber | Where-Object { $_.SourceIdentifier -in $EventIds }).Count -gt 0) {
        throw 'LIFECYCLE_CLEANUP_UNCONFIRMED'
    }
}
function Register-MC01LifecycleObserver {
    param([string[]]$EventIds, [bool]$IncludeGame)
    try {
        foreach ($kind in @('Start','Stop')) {
            $index = if ($kind -eq 'Start') { 0 } else { 1 }
            $filter = "TargetInstance.Name='powershell.exe'"
            if ($IncludeGame) { $filter += " OR TargetInstance.Name='Minecraft.Windows.exe' OR TargetInstance.Name='Minecraft.AdalServer.exe'" }
            $class = if ($kind -eq 'Start') { '__InstanceCreationEvent' } else { '__InstanceDeletionEvent' }
            $query = "SELECT * FROM $class WITHIN 1 WHERE TargetInstance ISA 'Win32_Process' AND (" + $filter + ')'
            Register-WmiEvent -Query $query -SourceIdentifier $EventIds[$index] | Out-Null
        }
    } catch {
        Remove-MC01LifecycleObserver $EventIds
        throw 'LIFECYCLE_SUBSCRIPTION_FAILED'
    }
}
function Read-MC01LifecycleEvents {
    param([string[]]$EventIds, [int]$ControlPid, [int]$OperatorSession, [string]$ControlCreated = '')
    foreach ($id in $EventIds) {
        foreach ($event in @(Get-Event -SourceIdentifier $id -ErrorAction SilentlyContinue)) {
            try {
                $notification = $event.SourceEventArgs.NewEvent
                $m = $notification.TargetInstance
                $created = $null
                try {
                    $created = [Management.ManagementDateTimeConverter]::ToDateTime([string]$m.CreationDate).ToUniversalTime().ToFileTimeUtc().ToString()
                } catch { $created = $null }
                $birthMatches = $created -and $ControlCreated -and [Math]::Abs(([long]$created - [long]$ControlCreated)) -lt 10
                $control = $birthMatches -and $ControlPid -gt 0 -and $m.Name -ieq 'powershell.exe' -and $m.ProcessID -eq $ControlPid -and $m.SessionID -eq $OperatorSession
                $game = $m.Name -in @('Minecraft.Windows.exe','Minecraft.AdalServer.exe')
                if (-not $control -and -not $game) { continue }
                [pscustomobject]@{kind=$(if ($id -eq $EventIds[0]) {'start'} else {'stop'})
                    control=$control; name=[string]$m.Name; pid=[uint32]$m.ProcessID
                    parentPid=[uint32]$m.ParentProcessID; session=[uint32]$m.SessionID; createdFileTime=$created
                    eventFileTime=[string]$notification.TIME_CREATED; receivedUtc=[DateTime]::UtcNow.ToString('o')
                    attribution='POLLED_INSTANCE_CANDIDATE_ONLY'}
            } finally { Remove-Event -EventIdentifier $event.EventIdentifier }
        }
    }
}
function Invoke-MC01LifecycleControl {
    param([string[]]$EventIds, [int]$OperatorSession)
    Start-Sleep -Milliseconds 1200 # Allow the one-second intrinsic observer to establish a baseline.
    $child = [Diagnostics.Process]::new()
    $child.StartInfo = [Diagnostics.ProcessStartInfo]::new()
    $child.StartInfo.FileName = Join-Path $PSHOME 'powershell.exe'
    $child.StartInfo.Arguments = '-NoLogo -NoProfile -NonInteractive -Command "[Console]::ReadLine() | Out-Null"'
    $child.StartInfo.UseShellExecute = $false
    $child.StartInfo.CreateNoWindow = $true
    $child.StartInfo.RedirectStandardInput = $true
    $started = $false; $startSeen = $false; $stopSeen = $false
    $gameEvents = [Collections.Generic.List[object]]::new()
    try {
        if (-not $child.Start()) { throw 'CONTROL_START_FAILED' }
        $started = $true
        $null = $child.Handle # Retain the original owned-process handle.
        $created = $child.StartTime.ToUniversalTime().ToFileTimeUtc().ToString()
        $timer = [Diagnostics.Stopwatch]::StartNew()
        while ($timer.Elapsed.TotalSeconds -lt 5 -and -not $startSeen) {
            foreach ($item in @(Read-MC01LifecycleEvents $EventIds $child.Id $OperatorSession $created)) {
                if ($item.control) { if ($item.kind -eq 'start') { $startSeen = $true } }
                else {
                    if ($gameEvents.Count -ge 256) { throw 'LIFECYCLE_EVENT_BUDGET' }
                    $gameEvents.Add($item)
                }
            }
            Start-Sleep -Milliseconds 50
        }
        $child.StandardInput.WriteLine('finish')
        $child.StandardInput.Close()
        if (-not $child.WaitForExit(5000) -or $child.ExitCode -ne 0) { throw 'CONTROL_EXIT_FAILED' }
        $timer.Restart()
        while ($timer.Elapsed.TotalSeconds -lt 5 -and -not $stopSeen) {
            foreach ($item in @(Read-MC01LifecycleEvents $EventIds $child.Id $OperatorSession $created)) {
                if ($item.control) {
                    if ($item.kind -eq 'start') { $startSeen = $true }
                    if ($item.kind -eq 'stop') { $stopSeen = $true }
                } else {
                    if ($gameEvents.Count -ge 256) { throw 'LIFECYCLE_EVENT_BUDGET' }
                    $gameEvents.Add($item)
                }
            }
            Start-Sleep -Milliseconds 50
        }
        if (-not $startSeen -or -not $stopSeen) { throw 'LIFECYCLE_CONTROL_NOT_OBSERVED' }
        [pscustomobject]@{startObserved=$startSeen; stopObserved=$stopSeen; gameEvents=$gameEvents.ToArray()}
    } finally {
        try {
            if ($started -and -not $child.HasExited) {
                $child.Kill() # Exact self-created control only; never a game/PID reopen.
                if (-not $child.WaitForExit(5000)) { throw 'CONTROL_CLEANUP_UNCONFIRMED' }
            }
        } finally { $child.Dispose() }
    }
}
function Get-MC01ReactivationApp {
    param([string]$RegisteredImage)
    if (@(Get-AppxPackage -Name 'Microsoft.MinecraftEducationEdition' -ErrorAction Stop).Count -gt 0) {
        throw 'MIXED_STORE_DESKTOP_REFUSED'
    }
    $apps = @(Get-StartApps -ErrorAction Stop | Where-Object {
        $_.Name -in @('Minecraft Education','Minecraft: Education Edition','Minecraft Education Edition')
    })
    if ($apps.Count -ne 1) { throw 'REGISTERED_APP_AMBIGUOUS' }
    $appId = [string]$apps[0].AppID
    if ($appId.Length -gt 256 -or $appId -notmatch '^\{[0-9A-Fa-f]{8}(?:-[0-9A-Fa-f]{4}){3}-[0-9A-Fa-f]{12}\}(?:\\[A-Za-z0-9_][A-Za-z0-9_. -]*)+$') {
        throw 'REGISTERED_DESKTOP_APP_INVALID'
    }
    Assert-ManagerRegisteredAppId $appId $RegisteredImage
    return $appId
}
function Invoke-MC01ReactivationObservation {
    param([bool]$Execute, [bool]$ControlOnly)
    $session = [Diagnostics.Process]::GetCurrentProcess().SessionId
    $ids = @(('MC01ReactivationStart_' + [Guid]::NewGuid().ToString('N')),
             ('MC01ReactivationStop_' + [Guid]::NewGuid().ToString('N')))
    if ($ControlOnly) {
        try {
            Register-MC01LifecycleObserver $ids $false
            $result = Invoke-MC01LifecycleControl $ids $session
            [pscustomobject]@{mode='LIFECYCLE_SELF_TEST'; startObserved=$result.startObserved
                stopObserved=$result.stopObserved; result='PASS'; gameAction='NONE'
                recording='NOT_EXECUTED'; observer='WMI_INTRINSIC_WITHIN_1_SECOND'
                lifecycleCompleteness='UNKNOWN'} | ConvertTo-Json
        } finally { Remove-MC01LifecycleObserver $ids }
        return
    }
    . (Join-Path $PSScriptRoot 'education-target.ps1')
    $registered = Resolve-ManagerEducationTarget -RegisteredOnly
    $hash = (Get-FileHash -LiteralPath $registered -Algorithm SHA256).Hash
    $appId = Get-MC01ReactivationApp $registered
    $current = @(Get-MC01StartupProcesses)
    $blocker = Get-MC01ReactivationBlocker $current $registered $session
    if ($blocker) { throw $blocker }
    $principal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
    if ($session -eq 0 -or $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        throw 'UNELEVATED_INTERACTIVE_CALLER_REQUIRED'
    }
    if (-not $Execute) {
        [pscustomobject]@{mode='REACTIVATION_PREFLIGHT'; existingGameVerified=$true
            activation='NOT_EXECUTED'; recording='NOT_EXECUTED'; gameAction='NONE'
            operatorApprovalRequired=$true; singletonDecision='UNKNOWN'; providerReady=$false} | ConvertTo-Json
        return
    }
    $first = @($current | Where-Object { $_.name -eq 'Minecraft.Windows' })[0]
    $original = [Diagnostics.Process]::GetProcessById($first.pid)
    $destination = Join-Path ([IO.Path]::GetTempPath()) ('MivubiMC01Reactivation_' + [Guid]::NewGuid().ToString('N'))
    $receipt = [ordered]@{schema=1; mode='REACTIVATION'; state='PREPARING'
        scriptSha256=(Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash
        registeredImageSha256=$hash; originalPid=$first.pid; originalCreated=$first.creationFileTime
        session=$session; originalPreserved=$null; lifecycleControl=$false
        activationAttempts=0; activationSubmitted=$false; requestStartedUtc=$null; requestEndedUtc=$null
        requestStartTicks=$null; requestEndTicks=$null; qpcFrequency=[Diagnostics.Stopwatch]::Frequency
        durationSeconds=30; endedUtc=$null; observer='WMI_INTRINSIC_WITHIN_1_SECOND'
        processEvents=@(); newLifetimes=@(); error=$null; errorStage=$null; kernelRecording='NOT_EXECUTED'
        activationTargetPid='UNKNOWN'; windowResult='UNKNOWN'; lifecycleCompleteness='UNKNOWN'
        singletonDecision='UNKNOWN'; authentication='NOT_EXECUTED'; providerReady=$false}
    $events = [Collections.Generic.List[object]]::new()
    $lifetimes = @{}; $receiptPath = $null; $boundOriginal = $false
    try {
        $null = $original.Handle
        if ($original.HasExited -or $original.StartTime.ToUniversalTime().ToFileTimeUtc().ToString() -ne $first.creationFileTime) {
            throw 'ORIGINAL_LIFETIME_CHANGED'
        }
        $boundOriginal = $true
        New-Item -ItemType Directory -Path $destination -ErrorAction Stop | Out-Null
        $receiptPath = Join-Path $destination 'receipt.json'
        Register-MC01LifecycleObserver $ids $true
        $receipt.state = 'CHECKING_LIFECYCLE_CONTROL'
        $control = Invoke-MC01LifecycleControl $ids $session
        $receipt.lifecycleControl = $control.startObserved -and $control.stopObserved
        if (-not $receipt.lifecycleControl) { throw 'LIFECYCLE_CONTROL_NOT_OBSERVED' }
        foreach ($item in $control.gameEvents) { $events.Add($item) }
        $receipt.state = 'RECHECKING_BINDING'
        if ((Resolve-ManagerEducationTarget -RegisteredOnly) -ine $registered -or
            (Get-FileHash -LiteralPath $registered -Algorithm SHA256).Hash -ne $hash -or
            (Get-MC01ReactivationApp $registered) -cne $appId -or $original.HasExited) {
            throw 'PREACTIVATION_BINDING_CHANGED'
        }
        $rechecked = @(Get-MC01StartupProcesses)
        $reason = Get-MC01ReactivationBlocker $rechecked $registered $session
        if (-not $reason) { $reason = Get-MC01ObservationBlocker $rechecked $registered $session $first.identity }
        if ($reason) { throw $reason }
        $explorer = Join-Path ([Environment]::GetFolderPath([Environment+SpecialFolder]::Windows)) 'explorer.exe'
        if (-not (Test-Path -LiteralPath $explorer -PathType Leaf)) { throw 'SYSTEM_EXPLORER_UNAVAILABLE' }
        $receipt.state = 'REQUESTING'; $receipt.activationAttempts = 1
        Save-MC01StartupReceipt $receipt $receiptPath
        $receipt.requestStartedUtc = [DateTime]::UtcNow.ToString('o')
        $receipt.requestStartTicks = [Diagnostics.Stopwatch]::GetTimestamp()
        try {
            Start-Process -FilePath $explorer -ArgumentList ('"shell:AppsFolder\' + $appId + '"') -ErrorAction Stop | Out-Null
            $receipt.activationSubmitted = $true
        } finally {
            $receipt.requestEndTicks = [Diagnostics.Stopwatch]::GetTimestamp()
            $receipt.requestEndedUtc = [DateTime]::UtcNow.ToString('o')
        }
        $receipt.state = 'OBSERVING'
        Save-MC01StartupReceipt $receipt $receiptPath
        $timer = [Diagnostics.Stopwatch]::StartNew()
        while ($timer.Elapsed.TotalSeconds -lt 30) {
            if ($original.HasExited) { throw 'ORIGINAL_EXITED_PRESERVED_NO_RETRY' }
            foreach ($item in @(Get-MC01StartupProcesses)) {
                if ($item.name -eq 'Minecraft.Windows' -and $item.identity -ne $first.identity) {
                    $key = if ($item.readable) { $item.identity } else { 'unreadable:' + $item.pid }
                    if ($lifetimes.Count -ge 256 -and -not $lifetimes.ContainsKey($key)) { throw 'LIFETIME_BUDGET' }
                    $lifetimes[$key] = [pscustomobject]@{pid=$item.pid; readable=$item.readable
                        created=$item.creationFileTime; session=$item.session
                        registeredImageMatches=($item.path -ieq $registered)}
                }
            }
            foreach ($item in @(Read-MC01LifecycleEvents $ids 0 $session)) {
                if ($events.Count -ge 256) { throw 'LIFECYCLE_EVENT_BUDGET' }
                $events.Add($item)
            }
            Start-Sleep -Milliseconds 100
        }
        $receipt.originalPreserved = -not $original.HasExited
        $receipt.state = 'OBSERVATION_COMPLETED' # Timer completed, not a window/singleton verdict.
    } catch {
        $receipt.errorStage = $receipt.state
        $receipt.state = 'INCOMPLETE'; $receipt.error = 'BINDING_CONTROL_OR_OBSERVATION_FAILED'
    } finally {
        try {
            Remove-MC01LifecycleObserver $ids
            if ($boundOriginal) { $receipt.originalPreserved = -not $original.HasExited }
            $receipt.processEvents = @($events.ToArray())
            $receipt.newLifetimes = @($lifetimes.Values)
            $receipt.endedUtc = [DateTime]::UtcNow.ToString('o')
            if ($receiptPath) { Save-MC01StartupReceipt $receipt $receiptPath }
        } finally { $original.Dispose() } # Never terminate a game, sign out or retry.
    }
    if ($receiptPath) { Write-Output ('Local receipt: ' + $receiptPath) }
    if ($receipt.state -ne 'OBSERVATION_COMPLETED') { throw 'Reactivation observation incomplete; do not retry automatically.' }
}

if ($SelfTest) {
    if ($Record) { throw 'SelfTest cannot record.' }
    $sample = [pscustomobject]@{name='Minecraft.Windows'; pid=7; readable=$true; session=1
        creationFileTime='100'; path='C:\fixture\Minecraft.Windows.exe'; identity='7:100'}
    $cases = @(
        @((Get-MC01StartupBlocker @()), $null),
        @((Get-MC01TraceBudgetBlocker 0), $null),
        @((Get-MC01TraceBudgetBlocker 268435455), $null),
        @((Get-MC01TraceBudgetBlocker 268435456), 'TRACE_SIZE_THRESHOLD_REACHED'),
        @((Get-MC01StartupBlocker @($sample)), 'EXISTING_GAME_OR_HELPER_PRESERVED'),
        @((Get-MC01StartupBlocker @([pscustomobject]@{name='Minecraft.AdalServer';readable=$false})),
            'EXISTING_GAME_OR_HELPER_PRESERVED'),
        @((Get-MC01ObservationBlocker @($sample) $sample.path 1 ''), $null),
        @((Get-MC01ObservationBlocker @($sample,$sample) $sample.path 1 ''), 'MULTIPLE_GAME_PROCESSES'),
        @((Get-MC01ObservationBlocker @($sample) 'C:\other.exe' 1 ''), 'REGISTERED_IMAGE_MISMATCH'),
        @((Get-MC01ObservationBlocker @($sample) $sample.path 2 ''), 'OTHER_INTERACTIVE_SESSION'),
        @((Get-MC01ObservationBlocker @($sample) $sample.path 1 '7:99'), 'GAME_LIFETIME_CHANGED')
    )
    $cases += ,@((Get-MC01ReactivationBlocker @($sample) $sample.path 1), $null)
    $cases += ,@((Get-MC01ReactivationBlocker @() $sample.path 1), 'EXACTLY_ONE_EXISTING_GAME_REQUIRED')
    $cases += ,@((Get-MC01ReactivationBlocker @($sample,$sample) $sample.path 1), 'EXACTLY_ONE_EXISTING_GAME_REQUIRED')
    $cases += ,@((Get-MC01ReactivationBlocker @($sample) 'C:\other.exe' 1), 'REGISTERED_IMAGE_MISMATCH')
    $cases += ,@((Get-MC01ReactivationBlocker @($sample) $sample.path 2), 'OTHER_INTERACTIVE_SESSION')
    $sample.readable = $false
    $cases += ,@((Get-MC01ReactivationBlocker @($sample) $sample.path 1), 'GAME_IDENTITY_UNKNOWN')
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
if ($Reactivation -or $LifecycleSelfTest) {
    Invoke-MC01ReactivationObservation -Execute ([bool]$Record) -ControlOnly ([bool]$LifecycleSelfTest)
    return
}
$before = @(Get-MC01StartupProcesses)
$blocker = Get-MC01StartupBlocker $before
if ($blocker) {
    [pscustomobject]@{mode='PREFLIGHT'; status=$blocker; processCount=$before.Count
        recording='NOT_EXECUTED'; gameAction='NONE'} | ConvertTo-Json
    if ($Record) { throw 'Recording refused; keep the existing game/helper session untouched.' }
    return
}

. (Join-Path $PSScriptRoot 'education-target.ps1')
$registeredImage = Resolve-ManagerEducationTarget -RegisteredOnly
$imageHash = (Get-FileHash -LiteralPath $registeredImage -Algorithm SHA256).Hash
$profile = Join-Path $PSScriptRoot 'mc01-startup.wprp'
if ((Get-FileHash -LiteralPath $profile -Algorithm SHA256).Hash -ne
    '6160EDF23C6BF4A9B8B812EF2027226BAC1F765712A9A9E12C3749020026DD17') {
    throw 'Recording profile differs from the reviewed Handle/Loader-only profile.'
}
if (-not $Record) {
    [pscustomobject]@{mode='PREFLIGHT'; status='NO_EXISTING_TARGET_OBSERVED'; registeredImageVerified=$true
        durationSeconds=30; bufferMiB=32; loggingMode='File'; traceStopThresholdMiB=256; recording='NOT_EXECUTED'; gameAction='NONE'
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
    schema=1; instance=$instance; durationSeconds=30; bufferMiB=32; loggingMode='File'; traceStopThresholdMiB=256
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
    $startResult = Invoke-MC01Wpr @('-start', ($profile + '!MC01Startup'), '-filemode', '-recordtempto', $destination, '-instancename', $instance)
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
        # Stop threshold, not a hard size cap: stop/rundown can write additional data.
        $rawBytes = [long]((Get-ChildItem -LiteralPath $destination -Filter '*.etl' -File |
            Measure-Object -Property Length -Sum).Sum)
        if (Get-MC01TraceBudgetBlocker $rawBytes) { throw 'TRACE_SIZE_THRESHOLD_REACHED' }
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
