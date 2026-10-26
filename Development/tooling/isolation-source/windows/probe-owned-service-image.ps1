# Read-only Windows SCM image evidence for a disposable lab.
# Never stores raw machine UUID, account IDs, tokens, or changes a service.
[CmdletBinding()]
param(
  [Parameter(Mandatory=$true)][ValidateSet('SbieSvc','SbieDrv')][string]$ServiceName,
  [Parameter(Mandatory=$true)][string]$ExpectedImagePath
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT') { throw 'Windows required' }

$original = [IO.Path]::GetFullPath($ExpectedImagePath)
if (-not [IO.Path]::IsPathRooted($original) -or
    $original -notmatch '^[A-Za-z]:\\' -or
    $original -match '(?i)\\Sandboxie-Plus\\' -or
    $original -match '(?i)^[A-Za-z]:\\Windows\\') {
    throw 'Expected binary path must be separate from official Sandboxie and Windows'
}
$expectedName = if ($ServiceName -eq 'SbieDrv') { 'SbieDrv.sys' } else { 'SbieSvc.exe' }
if ([IO.Path]::GetFileName($original) -cne $expectedName) {
    throw 'Image filename does not match service identity'
}
function Test-OrdinaryFile([string]$FilePath) {
    try {
        $file = Get-Item -LiteralPath $FilePath -Force -ErrorAction Stop
        if ($file.PSIsContainer -or ($file.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
            return $false
        }
        return $true
    } catch { return $false }
}
$class = if ($ServiceName -eq 'SbieDrv') { 'Win32_SystemDriver' } else { 'Win32_Service' }
$registration = 'UNKNOWN'
$scmPath = ''
try {
    $found = @(Get-CimInstance -ClassName $class -Filter ("Name='" + $ServiceName + "'") -ErrorAction Stop)
    if ($found.Count -eq 0) { $registration = 'ABSENT' }
    elseif ($found.Count -eq 1 -and [string]$found[0].Name -ceq $ServiceName) {
        $registration = 'PRESENT'
        $scmPath = [string]$found[0].PathName
    }
} catch { $registration = 'UNKNOWN' }
# Fail closed on extra args, NT object paths, environment variable expansion,
# unquoted whitespace or aliases; even valid service installations can differ.
$pathMatches = ($registration -eq 'PRESENT') -and (
    ($scmPath -ceq $original) -or ($scmPath -ceq ('"' + $original + '"'))
)
$sha = ''
$sig = 'Unknown'
if (Test-OrdinaryFile $original) {
    try {
        $sha = (Get-FileHash -LiteralPath $original -Algorithm SHA256).Hash.ToLowerInvariant()
        $sig = [string](Get-AuthenticodeSignature -LiteralPath $original -ErrorAction Stop).Status
    } catch { $sha = ''; $sig = 'Unknown' }
}
$fingerprint = ''
try {
    $uuid = [string](Get-CimInstance -ClassName Win32_ComputerSystemProduct -ErrorAction Stop).UUID
    if ($uuid -match '^[0-9A-Fa-f-]{36}$' -and $uuid -notmatch '^0{8}-') {
        $digest = [Security.Cryptography.SHA256]::Create()
        try {
            $bytes = [Text.Encoding]::UTF8.GetBytes('MIVUBI_ISOLATION_LAB_V1|' + $uuid.ToUpperInvariant())
            $fingerprint = ([BitConverter]::ToString($digest.ComputeHash($bytes))).Replace('-', '').ToLowerInvariant()
        } finally { $digest.Dispose() }
    }
} catch { $fingerprint = '' }
# Unknown host identity remains an empty fingerprint; never invent a match.
[pscustomobject]@{
    schema = 1
    evidenceClass = 'WINDOWS_READ_ONLY_SCM_IMAGE_PROBE'
    serviceName = $ServiceName
    registration = $registration
    imagePath = $scmPath
    expectedImagePath = $original
    registeredImagePathMatches = [bool]$pathMatches
    imageSha256 = $sha
    imageSignatureStatus = $sig
    hostFingerprintSha256 = $fingerprint
} | ConvertTo-Json -Depth 3 -Compress
