# Read-only preflight for a future disposable Windows isolation engine executor.
# NO service creation, driver load, signature bypass, profile access, or game launch.
[CmdletBinding()]
param([Parameter(Mandatory = $true)][string]$PayloadRoot)
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

if ($env:OS -ne "Windows_NT") { throw "Windows required" }
$root = (Resolve-Path -LiteralPath $PayloadRoot -ErrorAction Stop).ProviderPath
$entry = Get-Item -LiteralPath $root -Force -ErrorAction Stop
if (-not $entry.PSIsContainer -or ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
    throw "Payload must be an ordinary directory, not a redirected link"
}
function Get-NameRegistration([string]$class, [string]$name) {
    try {
        $found = @(Get-CimInstance -ClassName $class -Filter ("Name='" + $name + "'") -ErrorAction Stop)
        if ($found.Count -eq 0) { return "ABSENT" }
        if ($found.Count -eq 1 -and [string]$found[0].Name -ceq $name) { return "PRESENT" }
    } catch { }
    return "UNKNOWN"
}
function Get-SignedBinaryStatus([string]$relative) {
    $path = Join-Path $root $relative
    try {
        $file = Get-Item -LiteralPath $path -Force -ErrorAction Stop
        if ($file.PSIsContainer -or ($file.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
            return "UNKNOWN"
        }
        $signature = Get-AuthenticodeSignature -LiteralPath $path -ErrorAction Stop
        if ($signature.Status -ne "Valid" -or $null -eq $signature.SignerCertificate) {
            return "NOT_VALID"
        }
        # Signature validity is NOT an assertion that Windows will load a
        # custom kernel driver or that the signer is accepted for this product.
        return "VALID_SIGNATURE_ONLY"
    } catch { return "UNKNOWN" }
}
$service = Get-NameRegistration "Win32_Service" "SbieSvc"
$driver = Get-NameRegistration "Win32_SystemDriver" "SbieDrv"
$checks = @("SbieDrv.sys", "SbieSvc.exe", "SbieDll.dll", "Start.exe",
    "SbieIni.exe", "kmdutil.exe")
$signatures = @($checks | ForEach-Object {
    [pscustomobject]@{
        file = $_
        signatureStatus = (Get-SignedBinaryStatus $_)
    }
})
try {
    $machine = Get-CimInstance -ClassName Win32_ComputerSystem -ErrorAction Stop
    $manufacturer = [string]$machine.Manufacturer
    $model = [string]$machine.Model
    $virtual = (
        $manufacturer -match '(?i)(VMware|Microsoft|QEMU|Xen|innotek|Oracle)' -or
        $model -match '(?i)(virtual|VMware|KVM|Hyper-V|VirtualBox|HVM domU)'
    )
} catch { $virtual = $false }
$signerReady = @($signatures | Where-Object { $_.signatureStatus -ne "VALID_SIGNATURE_ONLY" }).Count -eq 0
$namesEmpty = $service -eq "ABSENT" -and $driver -eq "ABSENT"
$state = if (-not $namesEmpty) { "BLOCKED_SERVICE_NAMESPACE" }
    elseif (-not $signerReady) { "BLOCKED_BINARY_SIGNATURES" }
    else { "BLOCKED_DISPOSABLE_HOST_AND_DRIVER_LOAD_NOT_ATTESTED" }
[pscustomobject]@{
    schema = 1
    evidenceClass = "WINDOWS_READ_ONLY_SCOPED_PROVISIONING_PREFLIGHT"
    upstreamCommit = "abd4c240031cfb46666daa74ca47ed4e48b561e4"
    serviceRegistration = $service
    driverRegistration = $driver
    signatures = $signatures
    virtualizationHeuristic = [bool]$virtual
    virtualizationAttested = $false
    ownerIdentityVerified = $false
    windowsKernelDriverLoadVerified = $false
    disposableEnvironmentVerified = $false
    installationPermitted = $false
    rollbackPermitted = $false
    state = $state
} | ConvertTo-Json -Compress -Depth 5
