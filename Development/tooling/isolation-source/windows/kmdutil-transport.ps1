# MIVUBI disposable isolation KmdUtil transport -- source implementation.
# Probe mode is read-only. Apply mode is intentionally sealed until a
# reviewed lab certificate is pinned and Windows disposal/ownership evidence exists.
# NEVER invoke from the active Minecraft 02 / Tauri runtime.
[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$SpecPath,
    [ValidateSet('Probe','Apply')][string]$Mode = 'Probe',
    [string]$LabAttestationPath
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT') { throw 'Windows required' }

# Empty until an independently approved lab certificate and private signing
# workflow exist. A CLI argument cannot supply or override this trust anchor.
$PinnedLabAttestationSignerThumbprint = ''
$Pin = 'abd4c240031cfb46666daa74ca47ed4e48b561e4'
$MaxSpecBytes = 16384

function Assert-ReadOnlyFile([string]$Path) {
    $item = Get-Item -LiteralPath $Path -Force -ErrorAction Stop
    if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw 'Engine binary is not an ordinary file'
    }
    return $item
}
function Get-BinaryObservation([string]$Path) {
    try {
        $item = Assert-ReadOnlyFile $Path
        $signature = Get-AuthenticodeSignature -LiteralPath $Path -ErrorAction Stop
        return [pscustomobject]@{
            exists = $true
            sha256 = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
            signature = [string]$signature.Status
            signerThumbprint = if ($null -eq $signature.SignerCertificate) { '' }
                else { [string]$signature.SignerCertificate.Thumbprint }
            byteLength = [long]$item.Length
        }
    } catch {
        return [pscustomobject]@{
            exists = $false; sha256 = ''; signature = 'Unknown'
            signerThumbprint = ''; byteLength = 0
        }
    }
}
function Get-Registration([string]$Name, [string]$CimClass) {
    try {
        $records = @(Get-CimInstance -ClassName $CimClass -Filter ("Name='" + $Name + "'") -ErrorAction Stop)
        if ($records.Count -eq 0) { return @{ state='ABSENT'; path='' } }
        if ($records.Count -eq 1 -and [string]$records[0].Name -ceq $Name) {
            return @{ state='PRESENT'; path= [string]$records[0].PathName }
        }
    } catch { }
    return @{ state='UNKNOWN'; path='' }
}
function Assert-Spec {
    param($Spec)
    if ($Spec.schema -ne 1 -or $Spec.kind -cne 'PINNED_KMDUTIL_TRANSPORT_SPEC' -or
        $Spec.upstreamCommit -cne $Pin -or $Spec.executionAllowed -ne $false -or
        $Spec.driverSigningVerified -ne $false -or $Spec.disposableHostAttested -ne $false -or
        $Spec.rollbackAuthorized -ne $false -or
        $Spec.driverMinifilterAltitude -cne '86900') {
        throw 'Unexpected or enabled transport specification'
    }
    $supported = @('01','02','03','04','05','06','07','R1','R2','R3','R4','R5')
    if ($supported -cnotcontains [string]$Spec.stepId) { throw 'Unsupported transport operation' }
    $component = if ([string]$Spec.stepId -in @('01','02','03','07','R3','R4','R5')) { 'SbieDrv' } else { 'SbieSvc' }
    if ([string]$Spec.component -cne $component -or
        [string]$Spec.expectedScmServiceName -cne $component) {
        throw 'Service identity mismatch'
    }
    $helper = [string]$Spec.executable
    if ($helper -notmatch '^[A-Za-z]:\\' -or
        $helper -match '["\r\n]' -or
        $helper -notmatch '(?i)\\kmdutil\.exe$' -or
        $helper -match '(?i)\\Sandboxie-Plus\\') { throw 'Invalid helper path' }
    $root = Split-Path -Parent $helper
    $imageName = if ($component -eq 'SbieDrv') { 'SbieDrv.sys' } else { 'SbieSvc.exe' }
    $image = Join-Path $root $imageName
    if ([IO.Path]::GetFullPath([string]$Spec.requiredImagePath) -cne
        [IO.Path]::GetFullPath($image)) { throw 'Image path does not belong to transport root' }
    if ([string]$Spec.requiredImageSha256 -cnotmatch '^[a-f0-9]{64}$') {
        throw 'Invalid required image SHA-256'
    }
    # The actual argv is independently derived from the verified image root.
    $message = 'msgfile=' + (Join-Path $root 'SbieMsg.dll')
    $expectedArgs = switch ([string]$Spec.stepId) {
        '03' { @('install','SbieDrv',$image,'type=kernel','start=demand',$message,'altitude=86900') }
        '05' { @('install','SbieSvc',('"' + $image + '"'),'type=own','start=auto','display=MIVUBI Isolation Lab Service','group=UIGroup',$message) }
        '06' { @('start','SbieSvc') }
        'R1' { @('stop','SbieSvc') }
        'R2' { @('delete','SbieSvc') }
        'R3' { @('stop','SbieDrv') }
        'R4' { @('delete','SbieDrv') }
        default { @() }
    }
    $actualArgs = if ($null -eq $Spec.argv) { @() } else { @($Spec.argv) }
    if ($actualArgs.Count -ne $expectedArgs.Count) { throw 'Unexpected helper argv count' }
    for ($i=0; $i -lt $expectedArgs.Count; $i++) {
        if ([string]$actualArgs[$i] -cne [string]$expectedArgs[$i]) {
            throw 'Unapproved helper argv'
        }
    }
    return @{ root=$root; image=$image; helper=$helper; argv=$expectedArgs
        component=$component }
}
$specItem = Assert-ReadOnlyFile $SpecPath
if ($specItem.Length -gt $MaxSpecBytes) { throw 'Oversized transport specification' }
$spec = Get-Content -LiteralPath $SpecPath -Raw -Encoding UTF8 | ConvertFrom-Json
$validated = Assert-Spec -Spec $spec
$service = Get-Registration 'SbieSvc' 'Win32_Service'
$driver = Get-Registration 'SbieDrv' 'Win32_SystemDriver'
$imageInfo = Get-BinaryObservation $validated.image
$helperInfo = Get-BinaryObservation $validated.helper
$matchedSha = $imageInfo.exists -and $imageInfo.sha256 -ceq [string]$spec.requiredImageSha256
$signaturesObserved = ($imageInfo.signature -eq 'Valid' -and $helperInfo.signature -eq 'Valid')
# Exact read-only SCM ImagePath corroboration. A matching name/path is
# still NOT cryptographic ownership and never authorizes rollback.
$registered = if ($validated.component -eq 'SbieSvc') { $service } else { $driver }
$scmImagePath = [string]$registered.path
$scmPathMatches = ($registered.state -eq 'PRESENT') -and
    (($scmImagePath -ceq $validated.image) -or
     ($scmImagePath -ceq ('"' + $validated.image + '"')))
$result = [ordered]@{
    schema = 1
    mode = $Mode
    stepId = [string]$spec.stepId
    upstreamCommit = $Pin
    component = $validated.component
    imageHashMatches = [bool]$matchedSha
    signaturesValid = [bool]$signaturesObserved
    serviceRegistration = $service.state
    driverRegistration = $driver.state
    registeredImagePathMatches = [bool]$scmPathMatches
    registeredImagePathObserved = $scmImagePath
    namespaceOwnershipVerified = $false
    disposableHostVerified = $false
    driverLoadVerified = $false
    installationPermitted = $false
    executionState = 'BLOCKED_PENDING_SIGNED_LAB_ATTESTATION'
}
if ($Mode -eq 'Probe') {
    [pscustomobject]$result | ConvertTo-Json -Compress -Depth 3
    exit 0
}

# Mutating execution is present in source but NOT enabled by an operator
# supplied flag. It requires a pinned signed lab attestator to be reviewed.
# No private signing key or trusted signer has been configured in this source.
if ([string]::IsNullOrWhiteSpace($PinnedLabAttestationSignerThumbprint)) {
    throw 'Mutating Windows transport disabled: no approved lab signer pinned'
}
# Future signed-lab approval must verify machine identity, disposable VM
# snapshot, exact plan fingerprint, journal receipt and service ownership.
# Until that implementation is reviewed, the following remains blocked.
throw 'Signed disposable-host attestation and owned-service receipt verification not yet implemented'
