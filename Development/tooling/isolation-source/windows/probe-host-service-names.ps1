# Read-only check for Windows service/driver NAME collisions.
# Neither service presence nor absence proves executable provenance or driver trust.
# Never inspect Sandboxie accounts, launch a game, or make SCM changes.
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Get-RegistrationStatus {
    param([string]$ClassName, [string]$ServiceName)
    try {
        $registrations = @(Get-CimInstance -ClassName $ClassName -Filter ("Name='" + $ServiceName + "'") -ErrorAction Stop)
        if ($registrations.Count -eq 0) { return 'ABSENT' }
        if ($registrations.Count -ne 1) { return 'UNKNOWN' }
        if ([string]$registrations[0].Name -cne $ServiceName) { return 'UNKNOWN' }
        return 'PRESENT'
    } catch {
        return 'UNKNOWN'
    }
}
$session = (Get-Process -Id $PID -ErrorAction Stop).SessionId
$service = Get-RegistrationStatus -ClassName 'Win32_Service' -ServiceName 'SbieSvc'
$driver = Get-RegistrationStatus -ClassName 'Win32_SystemDriver' -ServiceName 'SbieDrv'
$quality = if ($service -eq 'UNKNOWN' -or $driver -eq 'UNKNOWN') {
    'PARTIAL_UNKNOWN'
} else { 'OBSERVED' }
[pscustomobject]@{
    schema = 1
    evidenceClass = 'WINDOWS_SERVICE_NAME_ONLY_NO_OWNER_PROOF'
    interactiveSession = ($session -ne 0)
    inspectionStatus = $quality
    serviceRegistration = $service
    driverRegistration = $driver
    providerSelected = $false
    trustedDriverLoaded = $false
    gameplayVerified = $false
} | ConvertTo-Json -Depth 3 -Compress
