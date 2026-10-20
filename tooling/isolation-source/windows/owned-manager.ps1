# Developer-only adapter for the existing owned runtime; no service installation.
[CmdletBinding()]
param([string]$RequestJson,[string]$RepositoryRoot,[string]$ApplicationExecutable,[switch]$HelperOnly)
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
function Demand($condition,[string]$message) { if (-not $condition) { throw $message } }
function Ordinary([string]$path) {
    $node=Get-Item -LiteralPath $path -Force
    while ($null -ne $node) {
        Demand (-not ($node.Attributes -band [IO.FileAttributes]::ReparsePoint)) 'Reparse points refused.'
        $parent=Split-Path -Parent $node.FullName
        if (-not $parent -or $parent -eq $node.FullName) { break }
        $node=Get-Item -LiteralPath $parent -Force
    }
}
function Hash([string]$path) { (Get-FileHash -LiteralPath $path).Hash.ToLowerInvariant() }
function Native([string]$exe,[string[]]$arguments) {
    $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$exe
    $info.Arguments=($arguments|ForEach-Object {Demand ($_ -notmatch '["\r\n]') 'Unsafe native argument.';'"'+$_+'"'}) -join ' '
    $info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
    $p=[Diagnostics.Process]::Start($info);$output=$p.StandardOutput.ReadToEndAsync();$errorText=$p.StandardError.ReadToEndAsync()
    if (-not $p.WaitForExit(25000)) {$p.Kill();throw 'Helper timed out; inspect before retrying.'}
    $code=$p.ExitCode;$text=$output.Result;$err=$errorText.Result;$p.Dispose()
    Demand ($code -eq 0) ("Helper failed: $code $err $text")
    return $text
}
function SaveReceipt {
    [IO.File]::WriteAllText((Join-Path $session 'manager.tmp'),($receipt|ConvertTo-Json -Depth 20),[Text.UTF8Encoding]::new($false))
    Move-Item -LiteralPath (Join-Path $session 'manager.tmp') -Destination (Join-Path $session 'session.json') -Force
}
function CanonicalPlan([string]$client,[object[]]$boxes) {
    $inputPath=Join-Path $session ('manager-profile-input-'+$client+'.json')
    [IO.File]::WriteAllText($inputPath,(@{schema=1;boxes=@($boxes)}|ConvertTo-Json -Depth 3),[Text.UTF8Encoding]::new($false))
    (Native (Join-Path $repo 'workspace/toolchains/node/node.exe') @((Join-Path $repo 'tooling/isolation-source/windows/client02-profile-plan.mjs'),$inputPath,$client))|ConvertFrom-Json
}
function Sections {
    $names=@((Native $ini @('query','*')).Trim() -split '\r?\n')
    Demand ($names.Count -gt 0 -and $names.Count -le 512 -and $names[0] -ceq 'GlobalSettings') 'Unreliable profile inventory.'
    return @($names|Select-Object -Skip 1|Where-Object {$_ -notmatch '^(Template_|UserSettings_)' -and $_ -ne 'TemplateSettings'})
}
function OwnedProfile([string]$client) {
    if ($client -ceq '02' -and $receipt.runtime.PSObject.Properties.Name -contains 'client02') {
        return $receipt.runtime.client02.box -ceq 'MivubiEdu02' -and $receipt.runtime.client02.configurationState -ceq 'CONFIGURED'
    }
    if ($receipt.runtime.PSObject.Properties.Name -notcontains 'managerCreatedProfiles') { return $false }
    $entry=@($receipt.runtime.managerCreatedProfiles|Where-Object clientId -ceq $client)
    return $entry.Count -eq 1 -and $entry[0].state -ceq 'COMPLETE'
}
function SettingsMatch([string]$client) {
    $plan=CanonicalPlan $client @()
    foreach ($step in @($plan.guardedSteps|Where-Object operation -ceq 'REQUIRE_READBACK'|Select-Object -Skip 1)) {
        if ((Native $ini @($step.arguments)).Trim() -cne $step.expectedValue) {return $false}
    }
    return $true
}
function Show-LaunchedWindow($slot) {
    if($slot.status -cne 'RUNNING' -or -not $slot.settingsVerified -or $slot.processes.Count -ne 1){return $null}
    $game=$slot.processes[0]
    try {
        [MivubiManagerWindows]::Act([int]$game.pid,[long]$game.createdFileTime,[long][Convert]::ToInt64($game.windowHandle.Substring(2),16),'Focus')
        return $null
    } catch { return $_.Exception.GetBaseException().Message }
}
function Snapshot([string]$operation,$exitCode) {
    $sections=@(Sections);$clients=@();$ownedPids=[Collections.Generic.HashSet[int]]::new()
    foreach ($target in $minecraftTargets) {
        $client=$target.id;$box=$target.profile;$exists=$sections -icontains $box
        $owned=$exists -and (OwnedProfile $client)
        $ready=$owned -and (SettingsMatch $client)
        $pids=if($exists){@([MivubiRuntimeProbe]::BoxPids($dll,$box))}else{@()};$pids=@($pids);$games=@()
        foreach ($number in $pids) {
            [void]$ownedPids.Add([int]$number)
            $process=Get-Process -Id $number -ErrorAction Stop
            if ($process.ProcessName -ine 'Minecraft.Windows') {continue}
            Demand ($process.Path -ceq $gamePath) 'Untrusted Minecraft image in owned profile.'
            $started=$process.StartTime.ToUniversalTime()
            $identity=[MivubiRuntimeProbe]::Game($dll,$process.Id,$started.Ticks,$box)
            Demand ($identity.WindowsSession -eq $currentSession) 'Game belongs to another Windows session.'
            $process.Refresh()
            Demand (-not $process.HasExited -and $process.StartTime.ToUniversalTime() -eq $started) 'Game identity changed.'
            $games+=@{pid=$process.Id;createdFileTime=$started.ToFileTimeUtc().ToString();sessionId=$identity.WindowsSession;
                windowHandle=('0x'+$process.MainWindowHandle.ToInt64().ToString('x'));responding=$identity.Responding}
        }
        Demand ($games.Count -le 1) 'Multiple Education processes in one profile; ownership is ambiguous.'
        $status=if (-not $exists) {'ABSENT'} elseif (-not $ready) {'BLOCKED'} elseif ($games.Count -eq 1 -and $games[0].windowHandle -ne '0x0') {'RUNNING'} elseif ($pids.Count -gt 0) {'STARTING_OR_CLOSING'} else {'STOPPED'}
        $launchAllowed=$ready -and $pids.Count -eq 0
        $launchKey='managerLaunch'+$client;$launcherKey='managerLauncher'+$client
        if($receipt.runtime.PSObject.Properties.Name -contains $launchKey){$launchAllowed=$launchAllowed -and (([DateTimeOffset]::UtcNow-[DateTimeOffset]::Parse($receipt.runtime.$launchKey)).TotalSeconds -ge 30)}
        if($receipt.runtime.PSObject.Properties.Name -contains $launcherKey){$prior=$receipt.runtime.$launcherKey;$running=Get-Process -Id $prior.pid -ErrorAction SilentlyContinue;if($running -and $running.StartTime.ToUniversalTime().ToFileTimeUtc().ToString() -ceq $prior.createdFileTime){$launchAllowed=$false}}
        $clients+=@{clientId=$client;box=$box;profileExists=$exists;settingsVerified=$ready;boxedProcessCount=$pids.Count;status=$status;processes=@($games);launchAllowed=[bool]$launchAllowed}
    }
    $native=@()
    foreach($process in @(Get-Process -Name Minecraft.Windows -ErrorAction SilentlyContinue)) {
        if($ownedPids.Contains($process.Id)){continue}
        Demand ($process.Path -ceq $gamePath) 'Unknown Education image outside owned profiles.'
        $started=$process.StartTime.ToUniversalTime()
        $identity=[MivubiRuntimeProbe]::Game($dll,$process.Id,$started.Ticks,$null)
        Demand ($identity.WindowsSession -eq $currentSession) 'Native game belongs to another Windows session.'
        $native+=@{pid=$process.Id;createdFileTime=$started.ToFileTimeUtc().ToString();sessionId=$identity.WindowsSession;windowHandle=('0x'+$process.MainWindowHandle.ToInt64().ToString('x'));responding=$identity.Responding}
    }
    Demand ($native.Count -le 1) 'Multiple native Education processes; ownership is ambiguous.'
    return @{schema=1;provider='OWNED_V1186_LOCAL_TEST';operation=$operation;launcherExitCode=$exitCode;focusError=$null;clients=@($clients);nativeProcesses=@($native)}
}
$mutex=$null;$held=$false;$fileLock=$null
try {
    $repo=[IO.Path]::GetFullPath($RepositoryRoot)
    $request=$RequestJson|ConvertFrom-Json
    Demand ($request.operation -cin @('Inspect','Create','Launch','Focus','PlaceLeft','PlaceRight','Close')) 'Unknown owned operation.'
    Demand ($request.clientId -cin @('01','02','03','04')) 'Unknown client.'
    Demand (-not ($request.clientId -ceq '01' -and $request.operation -in @('Create','Launch'))) 'Native launch uses the existing registered Windows route.'
    $session=[IO.Path]::GetFullPath($request.sessionDirectory)
    Demand ($session.StartsWith(([IO.Path]::GetFullPath((Join-Path $repo 'workspace/owned-tests'))+'\'),[StringComparison]::OrdinalIgnoreCase)) 'Session outside developer test storage.'
    Ordinary $session
    $principal=[Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
    $currentSession=[Diagnostics.Process]::GetCurrentProcess().SessionId
    Demand ($currentSession -gt 0) 'An interactive Windows session is required.'
    if($HelperOnly){Demand ($request.operation -ceq 'Create' -and $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) 'Profile helper requires normal administrator elevation.'}
    if ($request.operation -ceq 'Create' -and -not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        $directory=Join-Path $session ('operations/'+[guid]::NewGuid().ToString('N'))
        New-Item -ItemType Directory -Path $directory -Force|Out-Null
        $inputPath=Join-Path $directory 'request.json'
        [IO.File]::WriteAllText($inputPath,$RequestJson,[Text.UTF8Encoding]::new($false))
        # The same application executes its embedded, fixed Create-only helper.
        # Never elevate a mutable .ps1 file from the checkout.
        Demand ($ApplicationExecutable -and (Test-Path -LiteralPath $ApplicationExecutable -PathType Leaf)) 'Application helper unavailable.'
        $child=Start-Process -FilePath $ApplicationExecutable -ArgumentList ('--owned-profile-create "'+$inputPath+'"') -Verb RunAs -WindowStyle Normal -PassThru
        Demand ($child.WaitForExit(100000)) 'Elevated creation is pending; inspect before retrying.'
        $answer=Get-Content -LiteralPath (Join-Path $directory 'response.json') -Raw|ConvertFrom-Json
        Demand ($answer.ok -eq $true) $answer.error
        $answer.result|ConvertTo-Json -Depth 12 -Compress
        return
    }
    Demand (-not ($request.operation -ceq 'Launch' -and $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator))) 'Launch Minecraft without elevation.'
    $mutex=[Threading.Mutex]::new($false,'Global\MIVUBI.OwnedV1186.TestRuntime');$held=$mutex.WaitOne(0)
    Demand $held 'Another owned runtime operation is running.'
    $fileLock=[IO.File]::Open((Join-Path $session 'operation.lock'),'OpenOrCreate','ReadWrite','None')
    $receipt=Get-Content -LiteralPath (Join-Path $session 'session.json') -Raw|ConvertFrom-Json
    Demand ($receipt.schema -eq 1 -and $receipt.servicesChanged -eq $true -and $receipt.driverLoaded -eq $true -and $receipt.productionInstallPermitted -eq $false) 'An activated local test session is required.'
    $payload=Join-Path $session 'payload';$dll=Join-Path $payload 'SbieDll.dll';$ini=Join-Path $payload 'SbieIni.exe'
    $manifestPath=Join-Path $session 'unsigned-engine-provenance.json';Ordinary $manifestPath
    Demand ((Hash $manifestPath) -ceq $receipt.manifestSha256) 'Original build manifest changed.'
    $manifest=Get-Content -LiteralPath $manifestPath -Raw|ConvertFrom-Json
    Demand ($manifest.state -ceq 'MIVUBI_V1186_CORE_STAGED_NOT_INSTALLABLE' -and $manifest.ownedTree -ceq '21dff59ba02a1a257beacd62a793399acb47da23' -and $manifest.artifacts.Count -eq 25) 'Unexpected native source provenance.'
    $seen=[Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    Demand (@($receipt.signedArtifacts).Count -eq 25) 'Incomplete test payload.'
    foreach ($row in $receipt.signedArtifacts) {
        Demand ($row.path -cmatch '^[A-Za-z0-9._/-]+$' -and -not $row.path.StartsWith('/') -and $row.path.Split('/') -notcontains '..' -and $seen.Add($row.path)) 'Unsafe/duplicate payload entry.'
        $path=Join-Path $payload $row.path;Ordinary $path
        Demand ((Hash $path) -ceq $row.sha256) 'Test payload changed.'
        if ($row.path -match '\.(exe|dll|sys)$') {
            $signature=Get-AuthenticodeSignature -LiteralPath $path
            Demand ($signature.Status -eq 'Valid' -and $signature.SignerCertificate.Thumbprint -ceq $receipt.certificateThumbprint) 'Untrusted test binary.'
        }
    }
    foreach($row in $manifest.artifacts){Demand ($seen.Contains($row.path)) 'Missing declared artifact.';if($row.path -notmatch '\.(exe|dll|sys)$'){Demand ((Hash (Join-Path $payload $row.path)) -ceq $row.sha256) 'Original configuration/license resource changed.'}}
    Ordinary (Join-Path $payload 'TEST-ONLY-NOT-FOR-DISTRIBUTION.txt')
    Demand (@(Get-ChildItem -LiteralPath $payload -Recurse -File -Force).Count -eq 26) 'Unexpected files in the test payload.'
    foreach ($name in @('SbieDrv','SbieSvc')) {
        $filename=if ($name -eq 'SbieDrv') {'SbieDrv.sys'} else {'SbieSvc.exe'}
        $live=([string](Get-ItemProperty -LiteralPath "HKLM:\SYSTEM\CurrentControlSet\Services\$name").ImagePath -replace '^\\\?\?\\','').Trim('"')
        Demand ($live -ceq (Join-Path $payload $filename) -and [string](Get-Service $name).Status -eq 'Running') 'Active service belongs to another provider or is stopped.'
    }
    # Rust embeds the canonical probe definition; elevated file mode reads it locally.
    if (-not ('MivubiRuntimeProbe' -as [type])) {
        $probeSource=Get-Content -LiteralPath (Join-Path $repo 'tooling/isolation-source/windows/probe-v1186-runtime.ps1') -Raw
        $code=($probeSource -split "Add-Type -TypeDefinition @'",2)[1] -split "'@",2
        Add-Type -TypeDefinition $code[0]
    }
    $driver=[MivubiRuntimeProbe]::Read($dll)|ConvertFrom-Json
    Demand ($driver.version -ceq '5.73.6-MIVU' -and $driver.abi -eq 0x57230 -and $driver.testSigningActive -eq $true) 'Private driver/ABI/Test Mode verification failed.'
    if (-not (Get-Command Resolve-ManagerEducationTarget -ErrorAction SilentlyContinue)) {
        . (Join-Path $repo 'tooling/isolation-source/windows/education-target.ps1')
    }
    $gamePath=Resolve-ManagerEducationTarget
    $minecraftTargets=@((Get-Content -LiteralPath (Join-Path $repo 'engine/isolation-native/owned-runtime/minecraft-targets.json') -Raw|ConvertFrom-Json).targets|Where-Object kind -ceq 'ISOLATED')
    $signature=Get-AuthenticodeSignature -LiteralPath $gamePath
    Demand ($signature.Status -eq 'Valid' -and $signature.SignerCertificate.Subject -match '^CN=Microsoft Corporation,') 'Official Education installation is untrusted.'
    $before=Snapshot $request.operation $null
    $slot=if($request.clientId -ceq '01'){@{settingsVerified=$true;processes=@($before.nativeProcesses)}}else{@($before.clients|Where-Object clientId -ceq $request.clientId)[0]}
    $exitCode=$null
    if ($request.operation -ceq 'Create') {
        Demand ($request.compatibilityConfirmed -eq $true) 'Explicit compatibility-profile confirmation required.'
        Demand (-not $slot.profileExists) 'Profile exists; refusing to overwrite it.'
        foreach($key in @('FileRootPath','BoxRootFolder','KeyRootPath','IpcRootPath')) {
            Demand ([string]::IsNullOrWhiteSpace((Native $ini @('query','GlobalSettings',$key)))) 'Custom global storage roots require an explicit ownership review.'
        }
        $dataRoot=Join-Path $env:SystemDrive ('Sandbox/'+$env:USERNAME+'/'+$slot.box)
        $hive='Registry::HKEY_USERS\Sandbox_'+$env:USERNAME+'_'+$slot.box
        Demand (-not (Test-Path -LiteralPath $dataRoot) -and -not (Test-Path -LiteralPath $hive)) 'Existing account-bearing profile data must not be adopted implicitly.'
        $plan=CanonicalPlan $request.clientId @(Sections)
        if ($receipt.runtime.PSObject.Properties.Name -notcontains 'managerCreatedProfiles') {$receipt.runtime|Add-Member -NotePropertyName managerCreatedProfiles -NotePropertyValue @()}
        Demand (@($receipt.runtime.managerCreatedProfiles|Where-Object clientId -ceq $request.clientId).Count -eq 0) 'A partial profile attempt requires review.'
        $entry=[pscustomobject]@{clientId=$request.clientId;state='PARTIAL';box=$slot.box}
        $receipt.runtime.managerCreatedProfiles+= $entry;SaveReceipt
        foreach ($step in $plan.guardedSteps) {
            $text=Native $ini @($step.arguments)
            if ($step.operation -ceq 'REQUIRE_READBACK') {Demand ($text.Trim() -ceq $step.expectedValue) 'Profile readback mismatch; partial profile preserved.'}
        }
        $entry.state='COMPLETE';SaveReceipt
    } elseif ($request.operation -ceq 'Launch') {
        Demand ($request.licensed -eq $true -and $slot.settingsVerified -eq $true) 'Confirm licensing and verified profile settings before launch.'
        Demand ($slot.boxedProcessCount -eq 0) 'Profile is not empty; another launch is refused.'
        $launchKey='managerLaunch'+$request.clientId
        if ($receipt.runtime.PSObject.Properties.Name -contains $launchKey) {
            Demand (([DateTimeOffset]::UtcNow-[DateTimeOffset]::Parse($receipt.runtime.$launchKey)).TotalSeconds -ge 30) 'Recent launch request; inspect before retrying.'
        }
        $launcherKey='managerLauncher'+$request.clientId
        if($receipt.runtime.PSObject.Properties.Name -contains $launcherKey){
            $prior=$receipt.runtime.$launcherKey
            $running=Get-Process -Id $prior.pid -ErrorAction SilentlyContinue
            Demand (-not ($running -and $running.StartTime.ToUniversalTime().ToFileTimeUtc().ToString() -ceq $prior.createdFileTime)) 'Previous private launcher is still running; inspect it before retrying.'
        }
        $receipt.runtime|Add-Member -Force -NotePropertyName $launchKey -NotePropertyValue ([DateTime]::UtcNow.ToString('o'));SaveReceipt
        $start=[Diagnostics.ProcessStartInfo]::new()
        $start.FileName=Join-Path $payload 'Start.exe';$start.Arguments='/mivubi_client:'+$request.clientId+' "'+$gamePath+'"'
        $start.WorkingDirectory=$payload;$start.UseShellExecute=$false
        $child=[Diagnostics.Process]::Start($start)
        $handle=$child.Handle
        try {
            $receipt.runtime|Add-Member -Force -NotePropertyName $launcherKey -NotePropertyValue (@{pid=$child.Id;createdFileTime=$child.StartTime.ToUniversalTime().ToFileTimeUtc().ToString()});SaveReceipt
        } catch {if(-not $child.HasExited){throw}}
        if ($child.WaitForExit(1200)) {$exitCode=$child.ExitCode}
        # Observe only: never issue a second launch to compensate for missing evidence.
        $deadline=[DateTime]::UtcNow.AddSeconds(12)
        do {$after=Snapshot 'Launch' $exitCode;$found=@($after.clients|Where-Object clientId -ceq $request.clientId)[0];if($found.status -ceq 'RUNNING'){break};Start-Sleep -Milliseconds 350} while([DateTime]::UtcNow -lt $deadline)
        if($child.HasExited){$exitCode=$child.ExitCode;$after.launcherExitCode=$exitCode}
        $after.focusError=Show-LaunchedWindow $found
        $receipt.runtime|Add-Member -Force -NotePropertyName ('managerLaunchEvidence'+$request.clientId) -NotePropertyValue (@{at=[DateTimeOffset]::UtcNow.ToString('o');launcherExitCode=$exitCode;launcherAlive=(-not $child.HasExited);boxedProcessCount=$found.boxedProcessCount;attributedGameCount=$found.processes.Count;status=$found.status;focusError=$after.focusError});SaveReceipt
        $result=$after
    } elseif ($request.operation -notin @('Inspect','Create')) {
        Demand ($slot.settingsVerified -and $slot.processes.Count -eq 1 -and $null -ne $request.target) 'One freshly attributed game/window is required.'
        $live=$slot.processes[0];$target=$request.target
        Demand ($live.pid -eq $target.pid -and $live.createdFileTime -ceq $target.createdFileTime -and $live.sessionId -eq $target.sessionId -and $live.windowHandle -ceq $target.windowHandle -and $live.windowHandle -ne '0x0') 'Process/window identity changed; refresh first.'
        [MivubiManagerWindows]::Act([int]$live.pid,[long]$live.createdFileTime,[long][Convert]::ToInt64($live.windowHandle.Substring(2),16),$request.operation)
    }
    if ($request.operation -ne 'Launch') {$result=Snapshot $request.operation $exitCode}
    $result|ConvertTo-Json -Depth 12 -Compress
} finally {
    if ($fileLock) {$fileLock.Dispose()};if($held){$mutex.ReleaseMutex()};if($mutex){$mutex.Dispose()}
}
