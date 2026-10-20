# Local developer preparation only. Never registers, stops or loads a service.
[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][ValidateSet('Prepare','EnableTestMode','RestoreTestMode','Inspect')][string]$Stage,
    [Parameter(Mandatory=$true)][string]$SessionDirectory,
    [string]$CandidateDirectory,
    [string]$BuildReport,
    [string]$SignTool = 'C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe'
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../..'))
$session = [IO.Path]::GetFullPath($SessionDirectory)
$allowed = [IO.Path]::GetFullPath((Join-Path $repo 'workspace/owned-tests')) + '\'
if (-not $session.StartsWith($allowed, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Session must be a new child of workspace/owned-tests.'
}
function Assert-OrdinaryPath([string]$Path) {
    $node = Get-Item -LiteralPath $Path -Force
    while ($null -ne $node) {
        if ($node.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Reparse points refused.' }
        $parent = Split-Path -Parent $node.FullName
        if (-not $parent -or $parent -eq $node.FullName) { break }
        $node = Get-Item -LiteralPath $parent -Force
    }
}
function Hash([string]$Path) { (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant() }
function Save($Data) {
    $temp = Join-Path $session 'session.tmp'
    [IO.File]::WriteAllText($temp, ($Data | ConvertTo-Json -Depth 12), [Text.UTF8Encoding]::new($false))
    Move-Item -LiteralPath $temp -Destination (Join-Path $session 'session.json') -Force
}
function Native([string]$Exe, [string[]]$Arguments) {
    $psi = [Diagnostics.ProcessStartInfo]::new()
    $psi.FileName = $Exe
    # All arguments are fixed switches, hashes or canonical paths without quotes.
    $psi.Arguments = ($Arguments | ForEach-Object {
        if ($_ -match '["\r\n]') { throw 'Unsafe native argument.' }
        '"' + $_ + '"'
    }) -join ' '
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $p = [Diagnostics.Process]::Start($psi)
    $stdout = $p.StandardOutput.ReadToEndAsync()
    $stderr = $p.StandardError.ReadToEndAsync()
    if (-not $p.WaitForExit(60000)) { $p.Kill(); throw 'Native operation timed out after 60 seconds.' }
    $result = [ordered]@{exitCode=$p.ExitCode; stdout=$stdout.Result; stderr=$stderr.Result}
    $p.Dispose()
    if ($result.exitCode -ne 0) { throw ('Native operation failed: ' + $Exe + ' exit=' + $result.exitCode + ' ' + $result.stderr + $result.stdout) }
    return $result.stdout
}
function Assert-Administrator {
    $principal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'Run in an elevated Windows PowerShell session (normal UAC).' }
}
function BootSnapshot {
    $text = Native "$env:SystemRoot\System32\bcdedit.exe" @('/enum','{current}','/v')
    if ($text -notmatch '(?im)^identifier\s+(\{[^}]+\})\s*$') { throw 'Cannot identify current Windows boot entry.' }
    $identifier = $Matches[1]
    $mode = 'ABSENT'
    if ($text -match '(?im)^testsigning\s+(\S+)\s*$') {
        if ($Matches[1] -notin @('Yes','No')) { throw 'Unrecognized TESTSIGNING value; do not guess.' }
        $mode = $Matches[1]
    }
    [ordered]@{identifier=$identifier; testSigning=$mode; raw=$text}
}
function Assert-TestSecurity {
    if (Confirm-SecureBootUEFI) { throw 'Secure Boot is enabled; no firmware changes are performed.' }
    $guard = Get-CimInstance -Namespace root/Microsoft/Windows/DeviceGuard -ClassName Win32_DeviceGuard
    if (@($guard.SecurityServicesRunning) -contains 2) { throw 'HVCI is running; this test preparation does not change it.' }
    $volume = Get-BitLockerVolume -MountPoint $env:SystemDrive
    if ([string]$volume.ProtectionStatus -ne 'Off') { throw 'BitLocker protection is enabled; no suspension is performed.' }
}
if ($Stage -eq 'Prepare') {
    if (Test-Path -LiteralPath $session) { throw 'Session exists; use a fresh directory.' }
    New-Item -ItemType Directory -Path $session -Force | Out-Null
}
Assert-OrdinaryPath $session
$lock = [IO.File]::Open((Join-Path $session 'operation.lock'), 'OpenOrCreate', 'ReadWrite', 'None')
try {
    Assert-Administrator
    if ((Native 'git.exe' @('-C',$repo,'branch','--show-current')).Trim() -cne 'Local') { throw 'Local branch required.' }
    if ($Stage -eq 'Prepare') {
        if (-not $CandidateDirectory -or -not $BuildReport) { throw 'Candidate and matching build report required.' }
        $candidate = (Resolve-Path -LiteralPath $CandidateDirectory).Path
        Assert-OrdinaryPath $candidate
        Assert-OrdinaryPath $BuildReport
        $manifestPath = Join-Path $candidate 'engine-provenance.json'
        $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
        $build = Get-Content -LiteralPath $BuildReport -Raw | ConvertFrom-Json
        if ($manifest.state -cne 'MIVUBI_V1186_CORE_STAGED_NOT_INSTALLABLE' -or
            $manifest.sourceVersion -cne 'v1.18.6' -or $manifest.artifacts.Count -ne 25 -or
            $manifest.buildReportSha256 -cne (Hash $BuildReport) -or
            $build.state -cne 'NATIVE_BUILD_ARTIFACTS_CAPTURED_NOT_INSTALLABLE' -or
            $build.repoHead -cne $manifest.ownedRepoHead -or $build.ownedTree -cne $manifest.ownedTree -or
            @($build.buildStages | Where-Object { $_.exitCode -ne 0 }).Count -ne 0 -or $build.buildStages.Count -ne 3) {
            throw 'Candidate/build provenance mismatch.'
        }
        $tree = Native 'git.exe' @('-C',$repo,'rev-parse',($manifest.ownedRepoHead + ':engine/isolation-owned'))
        if ($tree.Trim() -cne $manifest.ownedTree) { throw 'Build revision owned source tree mismatch.' }
        $paths = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
        foreach ($row in $manifest.artifacts) {
            if ($row.path -cnotmatch '^[A-Za-z0-9._/-]+$' -or $row.path.Split('/') -contains '..' -or
                $row.path.StartsWith('/') -or -not $paths.Add($row.path)) { throw 'Unsafe/duplicate artifact path.' }
            $file = Join-Path $candidate $row.path
            Assert-OrdinaryPath $file
            if ((Hash $file) -cne $row.sha256 -or (Get-Item -LiteralPath $file).Length -ne $row.sizeBytes) { throw 'Candidate artifact changed.' }
        }
        $all = @(Get-ChildItem -LiteralPath $candidate -File -Recurse -Force)
        if ($all.Count -ne 27) { throw 'Candidate inventory must be 25 artifacts plus manifest and NOT-INSTALLABLE marker.' }
        Assert-TestSecurity
        $boot = BootSnapshot
        $head = (Native 'git.exe' @('-C',$repo,'rev-parse','HEAD')).Trim()
        $data = [ordered]@{schema=1; state='PREPARING'; sourceRevision=$head; buildRevision=$manifest.ownedRepoHead;
            candidate=$candidate; manifestSha256=(Hash $manifestPath); boot=$boot;
            lastBoot=(Get-CimInstance Win32_OperatingSystem).LastBootUpTime.ToUniversalTime().ToString('o');
            certificateThumbprint=$null; trustedStores=@(); signedArtifacts=@();
            servicesChanged=$false; driverLoaded=$false; gameplayVerified=$false; productionInstallPermitted=$false}
        Save $data
        # Snapshot only. Never stop/delete the installed official services here.
        foreach ($name in @('SbieSvc','SbieDrv')) {
            $key = "HKLM:\SYSTEM\CurrentControlSet\Services\$name"
            if (-not (Test-Path -LiteralPath $key)) { throw 'Expected official service is absent; review host before continuing.' }
            $image = (Get-ItemProperty -LiteralPath $key).ImagePath
            $image = $image -replace '^\\\?\?\\',''
            $image = $image.Trim('"')
            if ($image -notmatch '^[A-Za-z]:\\' -or $image -notmatch '\\Sandboxie-Plus\\(SbieSvc\.exe|SbieDrv\.sys)$') { throw 'Unexpected official service image.' }
            $sig = Get-AuthenticodeSignature -LiteralPath $image
            if ($sig.Status -ne 'Valid') { throw 'Official service signature is not trusted.' }
            Native "$env:SystemRoot\System32\reg.exe" @('export',("HKLM\SYSTEM\CurrentControlSet\Services\$name"),(Join-Path $session "$name.reg"),'/y') | Out-Null
            [IO.File]::WriteAllText((Join-Path $session "$name-image.json"), (@{path=$image; sha256=(Hash $image); signer=$sig.SignerCertificate.Thumbprint} | ConvertTo-Json))
        }
        $payload = Join-Path $session 'payload'
        New-Item -ItemType Directory -Path $payload | Out-Null
        foreach ($row in $manifest.artifacts) {
            $target = Join-Path $payload $row.path
            New-Item -ItemType Directory -Path (Split-Path -Parent $target) -Force | Out-Null
            Copy-Item -LiteralPath (Join-Path $candidate $row.path) -Destination $target
            if ((Hash $target) -cne $row.sha256) { throw 'Artifact changed during copy.' }
        }
        # Explicit test-only copy. The unsigned candidate is never modified.
        $cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject ('CN=MIVUBI Local Developer Test ' + [guid]::NewGuid().ToString('N')) -CertStoreLocation 'Cert:\CurrentUser\My' -KeyExportPolicy NonExportable -HashAlgorithm SHA256 -NotAfter (Get-Date).AddMonths(3)
        $data.certificateThumbprint = $cert.Thumbprint
        Save $data
        $publicCert = Join-Path $session 'test-public.cer'
        Export-Certificate -Cert $cert -FilePath $publicCert | Out-Null
        foreach ($store in @('Cert:\LocalMachine\Root','Cert:\LocalMachine\TrustedPublisher')) {
            $data.trustedStores += $store
            Save $data
            Import-Certificate -FilePath $publicCert -CertStoreLocation $store | Out-Null
        }
        $binaries = @($manifest.artifacts | Where-Object { $_.path -match '\.(exe|dll|sys)$' })
        if ($binaries.Count -ne 16) { throw 'Expected exactly 16 native binaries.' }
        foreach ($row in $manifest.artifacts) {
            $target = Join-Path $payload $row.path
            if ($row.path -match '\.(exe|dll|sys)$') {
                Native $SignTool @('sign','/s','My','/sha1',$cert.Thumbprint,'/fd','SHA256',$target) | Out-Null
                $sig = Get-AuthenticodeSignature -LiteralPath $target
                if ($sig.Status -ne 'Valid' -or $sig.SignerCertificate.Thumbprint -cne $cert.Thumbprint) { throw 'Test signature verification failed.' }
            }
            $data.signedArtifacts += @{path=$row.path; sha256=(Hash $target)}
            Save $data
        }
        Copy-Item -LiteralPath $manifestPath -Destination (Join-Path $session 'unsigned-engine-provenance.json')
        Copy-Item -LiteralPath $BuildReport -Destination (Join-Path $session 'unsigned-build-result.json')
        [IO.File]::WriteAllText((Join-Path $payload 'TEST-ONLY-NOT-FOR-DISTRIBUTION.txt'), 'Local development test signing only. Not a production release. See ../session.json. Never mix with official Sandboxie.')
        $data.state = 'SIGNED_TEST_COPY_READY_BOOT_UNCHANGED'
        Save $data
    } else {
        $data = Get-Content -LiteralPath (Join-Path $session 'session.json') -Raw | ConvertFrom-Json
        if ($data.schema -ne 1 -or $data.productionInstallPermitted -ne $false -or $data.servicesChanged -ne $false) { throw 'Unrecognized preparation receipt.' }
        if ($Stage -eq 'EnableTestMode') {
            if ($data.state -cne 'SIGNED_TEST_COPY_READY_BOOT_UNCHANGED') { throw 'Test copy is not ready.' }
            Assert-TestSecurity
            $now = BootSnapshot
            if ($now.identifier -cne $data.boot.identifier -or $now.testSigning -cne $data.boot.testSigning) { throw 'Boot entry changed since preparation.' }
            foreach ($row in $data.signedArtifacts) {
                if ((Hash (Join-Path $session ('payload/' + $row.path))) -cne $row.sha256) { throw 'Signed artifact changed.' }
            }
            Native "$env:SystemRoot\System32\bcdedit.exe" @('/set',$data.boot.identifier,'testsigning','on') | Out-Null
            if ((BootSnapshot).testSigning -cne 'Yes') { throw 'TESTSIGNING readback failed.' }
            $data.state = 'REBOOT_REQUIRED_NO_SERVICE_CHANGES'
            Save $data
        } elseif ($Stage -eq 'RestoreTestMode') {
            # Only restore the exact entry recorded by this preparation; do not guess.
            $now = BootSnapshot
            if ($now.identifier -cne $data.boot.identifier) { throw 'Boot entry differs; manual review required.' }
            if ($data.boot.testSigning -eq 'ABSENT') {
                if ($now.testSigning -ne 'ABSENT') { Native "$env:SystemRoot\System32\bcdedit.exe" @('/deletevalue',$data.boot.identifier,'testsigning') | Out-Null }
            } else {
                Native "$env:SystemRoot\System32\bcdedit.exe" @('/set',$data.boot.identifier,'testsigning',$data.boot.testSigning) | Out-Null
            }
            if ((BootSnapshot).testSigning -cne $data.boot.testSigning) { throw 'Original boot setting readback failed.' }
            foreach ($store in $data.trustedStores) {
                if ($store -notin @('Cert:\LocalMachine\Root','Cert:\LocalMachine\TrustedPublisher')) { throw 'Unexpected trust store.' }
                $path = Join-Path $store $data.certificateThumbprint
                if (Test-Path -LiteralPath $path) {
                    if ((Get-Item -LiteralPath $path).Subject -cnotmatch '^CN=MIVUBI Local Developer Test [a-f0-9]{32}$') { throw 'Certificate identity mismatch.' }
                    Remove-Item -LiteralPath $path
                }
            }
            $private = Join-Path 'Cert:\CurrentUser\My' $data.certificateThumbprint
            if (Test-Path -LiteralPath $private) {
                if ((Get-Item -LiteralPath $private).Subject -cnotmatch '^CN=MIVUBI Local Developer Test [a-f0-9]{32}$') { throw 'Private certificate identity mismatch.' }
                Remove-Item -LiteralPath $private -DeleteKey
            }
            $data.state = 'BOOT_RESTORED_CERTIFICATE_REMOVED_REBOOT_REQUIRED'
            Save $data
        }
    }
    [ordered]@{state=$data.state; artifacts=@($data.signedArtifacts).Count; servicesChanged=$data.servicesChanged; gameplayVerified=$false} | ConvertTo-Json -Compress
} finally { $lock.Dispose() }
