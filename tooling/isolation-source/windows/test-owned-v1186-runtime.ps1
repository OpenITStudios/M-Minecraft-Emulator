# Explicit local developer test. Reuses SCM registrations; never deletes services/data.
[CmdletBinding()]
param([Parameter(Mandatory=$true)][ValidateSet('Inspect','Activate','Restore')][string]$Stage,
      [Parameter(Mandatory=$true)][string]$SessionDirectory)
Set-StrictMode -Version Latest
$ErrorActionPreference='Stop'
$repo=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../..'))
$session=[IO.Path]::GetFullPath($SessionDirectory)
if (-not $session.StartsWith((Join-Path $repo 'workspace/owned-tests/'),[StringComparison]::OrdinalIgnoreCase)) { throw 'Session outside owned-tests.' }
function Ordinary([string]$path) {
    $node=Get-Item -LiteralPath $path -Force
    while ($null -ne $node) {
        if ($node.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Reparse point refused.' }
        $parent=Split-Path -Parent $node.FullName
        if (-not $parent -or $parent -eq $node.FullName) { break }
        $node=Get-Item -LiteralPath $parent -Force
    }
}
function Digest([string]$path) { (Get-FileHash -LiteralPath $path).Hash.ToLowerInvariant() }
function Native([string]$exe,[string[]]$arguments) {
    $pinfo=[Diagnostics.ProcessStartInfo]::new()
    $pinfo.FileName=$exe
    $pinfo.Arguments=($arguments | ForEach-Object {
        if ($_ -match '["\r\n]') { throw 'Unsafe argument.' }
        '"'+$_+'"'
    }) -join ' '
    $pinfo.UseShellExecute=$false; $pinfo.CreateNoWindow=$true
    $pinfo.RedirectStandardOutput=$true; $pinfo.RedirectStandardError=$true
    $p=[Diagnostics.Process]::Start($pinfo)
    $output=$p.StandardOutput.ReadToEndAsync(); $errorText=$p.StandardError.ReadToEndAsync()
    if (-not $p.WaitForExit(60000)) { $p.Kill(); throw 'Operation timeout; inspect journal before retry.' }
    $code=$p.ExitCode; $text=$output.Result; $err=$errorText.Result; $p.Dispose()
    if ($code -ne 0) { throw "Native operation failed: exit=$code $text $err" }
    $text
}
function Save {
    [IO.File]::WriteAllText((Join-Path $session 'runtime.tmp'),($script:receipt | ConvertTo-Json -Depth 15),[Text.UTF8Encoding]::new($false))
    Move-Item -LiteralPath (Join-Path $session 'runtime.tmp') -Destination (Join-Path $session 'session.json') -Force
}
function Journal([string]$event) {
    $script:receipt.runtime.events += [pscustomobject]@{at=[DateTime]::UtcNow.ToString('o'); action=$event}
    Save
}
function Image([string]$name) {
    [string](Get-ItemProperty -LiteralPath "HKLM:\SYSTEM\CurrentControlSet\Services\$name").ImagePath
}
function FileFromImage([string]$image) { ($image -replace '^\\\?\?\\','').Trim('"') }
function VerifyImage([string]$image,[string]$hash,[string]$thumbprint) {
    $file=FileFromImage $image
    Ordinary $file
    if ((Digest $file) -cne $hash) { throw 'Service binary hash mismatch.' }
    $sig=Get-AuthenticodeSignature -LiteralPath $file
    if ($sig.Status -ne 'Valid' -or $sig.SignerCertificate.Thumbprint -cne $thumbprint) { throw 'Service signature mismatch.' }
}
function Probe([string]$dll) {
    (Native (Join-Path $PSHOME 'powershell.exe') @('-NoProfile','-NonInteractive','-File',
        (Join-Path $PSScriptRoot 'probe-v1186-runtime.ps1'),'-DllPath',$dll,'-Sha256',(Digest $dll))) | ConvertFrom-Json
}
function Idle([string]$dll) {
    $probe=Probe $dll
    if ($probe.allSessionBoxedProcesses -ne 0) { throw 'Sandbox processes are active; close them normally before continuing.' }
    $probe
}
function Helper([string]$root,[string]$operation,[string]$name) {
    $helper=Join-Path $root 'kmdutil.exe'
    $sig=Get-AuthenticodeSignature -LiteralPath $helper
    if ($sig.Status -ne 'Valid') { throw 'Lifecycle helper signature invalid.' }
    $expectedHash=if ($root -ceq (Join-Path $session 'payload')) {
        ($script:receipt.signedArtifacts | Where-Object path -ceq 'kmdutil.exe').sha256
    } else { $script:receipt.runtime.officialHelperHash }
    if ((Digest $helper) -cne $expectedHash) { throw 'Lifecycle helper hash changed.' }
    Journal ($operation+':'+$name)
    Native $helper @($operation,$name) | Out-Null
    $expected=if ($operation -eq 'start') { 'Running' } else { 'Stopped' }
    $limit=[DateTime]::UtcNow.AddSeconds(20)
    do {
        $service=Get-Service -Name $name
        if ([string]$service.Status -eq $expected) { return }
        Start-Sleep -Milliseconds 200
    } while ([DateTime]::UtcNow -lt $limit)
    throw 'Service state readback timed out.'
}
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class MivubiTestScm {
  [DllImport("advapi32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr OpenSCManager(string machine,string database,uint access);
  [DllImport("advapi32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr OpenService(IntPtr manager,string name,uint access);
  [DllImport("advapi32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool ChangeServiceConfig(IntPtr service,uint type,uint start,uint error,string path,string group,IntPtr tag,string dependencies,string account,string password,string display);
  [DllImport("advapi32.dll")] static extern bool CloseServiceHandle(IntPtr handle);
  public static void Image(string name,string path) {
    if(name!="SbieSvc" && name!="SbieDrv") throw new Exception("Unexpected service");
    IntPtr manager=OpenSCManager(null,null,1);
    if(manager==IntPtr.Zero) throw new System.ComponentModel.Win32Exception();
    try {
      IntPtr service=OpenService(manager,name,2);
      if(service==IntPtr.Zero) throw new System.ComponentModel.Win32Exception();
      try { if(!ChangeServiceConfig(service,0xffffffff,0xffffffff,0xffffffff,path,null,IntPtr.Zero,null,null,null,null)) throw new System.ComponentModel.Win32Exception(); }
      finally { CloseServiceHandle(service); }
    } finally { CloseServiceHandle(manager); }
  }
}
'@
Ordinary $session
$principal=[Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'Normal elevated Windows PowerShell required.' }
$mutex=[Threading.Mutex]::new($false,'Global\MIVUBI.OwnedV1186.TestRuntime')
$acquired=$false; $fileLock=$null
try {
    $acquired=$mutex.WaitOne(0)
    if (-not $acquired) { throw 'Another owned runtime operation is active.' }
    $fileLock=[IO.File]::Open((Join-Path $session 'operation.lock'),'OpenOrCreate','ReadWrite','None')
    if ((Native 'git.exe' @('-C',$repo,'branch','--show-current')).Trim() -cne 'Local') { throw 'Local branch required.' }
    $script:receipt=Get-Content -LiteralPath (Join-Path $session 'session.json') -Raw | ConvertFrom-Json
    if ($receipt.schema -ne 1 -or $receipt.productionInstallPermitted -ne $false -or $receipt.certificateThumbprint -notmatch '^[A-F0-9]{40}$') { throw 'Unexpected test preparation receipt.' }
    $payload=Join-Path $session 'payload'
    if (@($receipt.signedArtifacts).Count -ne 25) { throw 'Incomplete test copy.' }
    foreach ($row in $receipt.signedArtifacts) {
        if ($row.path -cnotmatch '^[A-Za-z0-9._/-]+$' -or $row.path.Split('/') -contains '..' -or $row.path.StartsWith('/')) { throw 'Unsafe payload path.' }
        $file=Join-Path $payload $row.path
        Ordinary $file
        if ((Digest $file) -cne $row.sha256) { throw 'Test payload changed.' }
        if ($row.path -match '\.(exe|dll|sys)$') { VerifyImage $file $row.sha256 $receipt.certificateThumbprint }
    }
    $officialRoot='C:\Program Files\Sandboxie-Plus'
    $bootTime=(Get-CimInstance Win32_OperatingSystem).LastBootUpTime.ToUniversalTime()
    if ($bootTime -le [DateTime]::Parse($receipt.lastBoot).ToUniversalTime()) { throw 'Manual reboot has not occurred.' }
    if ($Stage -eq 'Inspect') {
        $expectedOwned=$receipt.servicesChanged
        $dll=if ($expectedOwned) { Join-Path $payload 'SbieDll.dll' } else { Join-Path $officialRoot 'SbieDll.dll' }
        $probe=Probe $dll
        [ordered]@{schema=1; state=$receipt.state; rebootObserved=$true; driver=$probe; servicesChanged=$receipt.servicesChanged; gameplayVerified=$false} | ConvertTo-Json -Depth 5 -Compress
        return
    }
    if ($Stage -eq 'Activate') {
        if ($receipt.state -cne 'REBOOT_REQUIRED_NO_SERVICE_CHANGES' -or $receipt.servicesChanged -ne $false -or $receipt.PSObject.Properties.Name -contains 'runtime') { throw 'Activation is not fresh; inspect or restore the existing transaction.' }
        $officialSigner=(Get-Content -LiteralPath (Join-Path $session 'SbieSvc-image.json') -Raw | ConvertFrom-Json).signer
        foreach ($file in @('kmdutil.exe','SbieDll.dll')) {
            $sig=Get-AuthenticodeSignature -LiteralPath (Join-Path $officialRoot $file)
            if ($sig.Status -ne 'Valid' -or $sig.SignerCertificate.Thumbprint -cne $officialSigner) { throw 'Official query/helper identity mismatch.' }
        }
        $instances=Get-ItemProperty -LiteralPath 'HKLM:\SYSTEM\CurrentControlSet\Services\SbieDrv\Instances'
        if ($instances.DefaultInstance -cne 'SbieDrv Instance' -or
            (Get-ItemProperty -LiteralPath 'HKLM:\SYSTEM\CurrentControlSet\Services\SbieDrv\Instances\SbieDrv Instance').Altitude -cne '86900') {
            throw 'Existing driver minifilter registration differs; do not rewrite it.'
        }
        $probe=Idle (Join-Path $officialRoot 'SbieDll.dll')
        if (-not $probe.testSigningActive -or $probe.version -like '*MIVU*') { throw 'Actual Test Mode/official-driver preflight failed.' }
        $snapshots=@()
        foreach ($name in @('SbieSvc','SbieDrv')) {
            $old=Get-Content -LiteralPath (Join-Path $session "$name-image.json") -Raw | ConvertFrom-Json
            $raw=Image $name
            if ((FileFromImage $raw) -cne $old.path) { throw 'Official SCM path changed since preparation.' }
            VerifyImage $raw $old.sha256 $old.signer
            $ownFile=if ($name -eq 'SbieDrv') { 'SbieDrv.sys' } else { 'SbieSvc.exe' }
            $ownPath=Join-Path $payload $ownFile
            $ownImage=if ($name -eq 'SbieDrv') { '\??\'+$ownPath } else { '"'+$ownPath+'"' }
            $snapshots += [pscustomobject]@{name=$name; originalImage=$raw; originalHash=$old.sha256; originalSigner=$old.signer;
                originalState=[string](Get-Service $name).Status; ownedImage=$ownImage; ownedHash=(Digest $ownPath)}
        }
        if (@($snapshots | Where-Object originalState -ne 'Running').Count -ne 0) { throw 'Expected original services running; do not guess restore semantics.' }
        $receipt | Add-Member -NotePropertyName runtime -NotePropertyValue ([pscustomobject]@{
            sourceRevision=(Native 'git.exe' @('-C',$repo,'rev-parse','HEAD')).Trim(); services=$snapshots;
            officialHelperHash=(Digest (Join-Path $officialRoot 'kmdutil.exe')); events=@(); error=$null})
        $receipt.state='ACTIVATING_TEST_ENGINE'; Save
        try {
            # Recheck global idle immediately before asking the official helper to stop.
            Idle (Join-Path $officialRoot 'SbieDll.dll') | Out-Null
            Helper $officialRoot 'stop' 'SbieSvc'
            Helper $officialRoot 'stop' 'SbieDrv'
            foreach ($service in $snapshots) {
                if ((Image $service.name) -cne $service.originalImage) { throw 'SCM ownership changed before redirect.' }
                VerifyImage $service.originalImage $service.originalHash $service.originalSigner
                $receipt.servicesChanged=$true; Journal ('redirect:'+ $service.name)
                [MivubiTestScm]::Image($service.name,$service.ownedImage)
                if ((Image $service.name) -cne $service.ownedImage) { throw 'Owned SCM path readback mismatch.' }
            }
            Helper $payload 'start' 'SbieDrv'
            Helper $payload 'start' 'SbieSvc'
            $owned=Idle (Join-Path $payload 'SbieDll.dll')
            if ($owned.version -cne '5.73.6-MIVU' -or $owned.abi -ne 0x57230 -or -not $owned.testSigningActive) { throw 'Private driver identity/ABI mismatch.' }
            $receipt.state='OWNED_TEST_DRIVER_ACTIVE_NO_GAME'; $receipt.driverLoaded=$true; Journal 'private-abi-verified'
        } catch {
            $receipt.runtime.error=$_.Exception.Message; $receipt.state='ACTIVATION_FAILED_RESTORE_REQUIRED'; Save
            throw
        }
    } else {
        if ($receipt.PSObject.Properties.Name -notcontains 'runtime' -or $receipt.runtime.services.Count -ne 2) { throw 'No runtime snapshot to restore.' }
        # Restore only this exact transaction's images; never stop an unknown provider.
        $ownedLive=$false
        foreach ($service in $receipt.runtime.services) {
            $live=Image $service.name
            if ($live -ceq $service.ownedImage) {
                VerifyImage $live $service.ownedHash $receipt.certificateThumbprint
                $ownedLive=$true
            } elseif ($live -ceq $service.originalImage) {
                VerifyImage $live $service.originalHash $service.originalSigner
            } else { throw 'Foreign SCM path; automatic restoration refused.' }
        }
        $driverRunning=[string](Get-Service SbieDrv).Status -eq 'Running'
        if ($driverRunning) {
            $driver=($receipt.runtime.services | Where-Object name -eq 'SbieDrv')
            $dll=if ((Image 'SbieDrv') -ceq $driver.ownedImage) { Join-Path $payload 'SbieDll.dll' } else { Join-Path $officialRoot 'SbieDll.dll' }
            Idle $dll | Out-Null
        }
        foreach ($name in @('SbieSvc','SbieDrv')) {
            if ([string](Get-Service $name).Status -ne 'Stopped') {
                Helper $(if ($ownedLive) { $payload } else { $officialRoot }) 'stop' $name
            }
        }
        foreach ($service in $receipt.runtime.services) {
            $live=Image $service.name
            if ($live -cne $service.originalImage -and $live -cne $service.ownedImage) { throw 'SCM identity changed during restoration.' }
            VerifyImage $service.originalImage $service.originalHash $service.originalSigner
            Journal ('restore-image:'+ $service.name)
            [MivubiTestScm]::Image($service.name,$service.originalImage)
            if ((Image $service.name) -cne $service.originalImage) { throw 'Original SCM path readback mismatch.' }
        }
        Helper $officialRoot 'start' 'SbieDrv'
        Helper $officialRoot 'start' 'SbieSvc'
        $original=Probe (Join-Path $officialRoot 'SbieDll.dll')
        if ($original.version -like '*MIVU*') { throw 'Original driver identity was not restored.' }
        $receipt.servicesChanged=$false; $receipt.driverLoaded=$false; $receipt.state='OFFICIAL_SERVICES_RESTORED_TEST_BOOT_REMAINS'; Journal 'official-services-restored'
    }
    [ordered]@{state=$receipt.state; servicesChanged=$receipt.servicesChanged; driverLoaded=$receipt.driverLoaded; gameplayVerified=$false} | ConvertTo-Json -Compress
} finally {
    if ($null -ne $fileLock) { $fileLock.Dispose() }
    if ($acquired) { $mutex.ReleaseMutex() }
    $mutex.Dispose()
}
