# Optional offline/read-only metadata for the single registered and
# Microsoft-signed Minecraft Education Desktop client. No game execution,
# installation, input, rendering, DLL injection, account or world operations.
# Not invoked by LazyTester, CI or a schedule. Explicit developer use only.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'education-target.ps1')

function Read-F3PeBytes {
    param([IO.BinaryReader]$Reader, [long]$Offset, [int]$Count, [long]$FileLength)
    if ($Offset -lt 0 -or $Count -lt 0 -or $Count -gt 4096 -or
        $Offset -gt $FileLength -or $Count -gt $FileLength - $Offset) {
        throw 'Out-of-bounds PE evidence read.'
    }
    $Reader.BaseStream.Position=$Offset
    $data=$Reader.ReadBytes($Count)
    if ($data.Length -ne $Count) {throw 'Truncated PE metadata.'}
    return ,$data
}
function Convert-F3PeRva {
    param([uint32]$Rva, [uint32]$Size, [object[]]$Sections, [long]$FileLength)
    foreach($sec in $Sections) {
        $delta=[long]$Rva - [long]$sec.Rva
        if ($delta -lt 0 -or $delta -ge [long]$sec.RawSize -or
            $Size -gt [long]$sec.RawSize - $delta) {continue}
        $offset=[long]$sec.Pointer + $delta
        if ($offset -ge 0 -and $offset -le $FileLength-[long]$Size) {return $offset}
    }
    return $null
}
function Read-F3PeMetadata {
    param([string]$VerifiedPath)
    $file=Get-Item -LiteralPath $VerifiedPath -ErrorAction Stop
    if ($file.PSIsContainer -or $file.Length -lt 4096 -or
        $file.Length -gt 8589934592) {throw 'Client PE size outside read budget.'}
    $length=[long]$file.Length
    $stream=[IO.File]::Open($VerifiedPath,[IO.FileMode]::Open,
        [IO.FileAccess]::Read,[IO.FileShare]::Read)
    $reader=[IO.BinaryReader]::new($stream)
    try {
        $dos=Read-F3PeBytes $reader 0 64 $length
        if ([BitConverter]::ToUInt16($dos,0) -ne 0x5a4d) {throw 'Missing MZ header.'}
        $at=[long][BitConverter]::ToUInt32($dos,0x3c)
        if ($at -lt 64 -or $at -gt 1048576) {throw 'Invalid PE header offset.'}
        $coff=Read-F3PeBytes $reader $at 24 $length
        if ([BitConverter]::ToUInt32($coff,0) -ne 0x00004550 -or
            [BitConverter]::ToUInt16($coff,4) -ne 0x8664) {
            throw 'Expected AMD64 PE executable.'
        }
        $n=[int][BitConverter]::ToUInt16($coff,6)
        $optSize=[int][BitConverter]::ToUInt16($coff,20)
        if ($n -lt 1 -or $n -gt 96 -or $optSize -lt 176 -or
            $optSize -gt 4096) {throw 'Invalid PE section layout.'}
        $opt=Read-F3PeBytes $reader ($at+24) $optSize $length
        if ([BitConverter]::ToUInt16($opt,0) -ne 0x20b) {throw 'Expected PE32+.'}
        $sizeOfImage=[BitConverter]::ToUInt32($opt,56)
        $directories=[BitConverter]::ToUInt32($opt,108)
        if ($sizeOfImage -lt 4096 -or $directories -gt 16) {
            throw 'Invalid PE optional directory.'
        }
        $headers=Read-F3PeBytes $reader ($at+24+$optSize) ($n*40) $length
        $sections=@()
        for($i=0;$i -lt $n;$i++){
            $p=$i*40
            $size=[BitConverter]::ToUInt32($headers,$p+16)
            $ptr=[BitConverter]::ToUInt32($headers,$p+20)
            if ($size -gt 0 -and
                ([long]$ptr -gt $length -or
                 [long]$size -gt $length-[long]$ptr)) {
                throw 'Unbounded PE raw section.'
            }
            $sections += [pscustomobject]@{
                Rva=[BitConverter]::ToUInt32($headers,$p+12)
                RawSize=$size
                Pointer=$ptr
            }
        }
        $cv=[ordered]@{status='NOT_PRESENT';kind='NONE'}
        # Data directory index 6 = IMAGE_DIRECTORY_ENTRY_DEBUG.
        # The RSDS record is only a local PDB REFERENCE, not symbol access.
        if ($directories -gt 6) {
            $rva=[BitConverter]::ToUInt32($opt,160)
            $bytes=[BitConverter]::ToUInt32($opt,164)
            if ($rva -ne 0 -and $bytes -ge 28 -and $bytes -le 4096 -and
                ($bytes % 28) -eq 0) {
                $offset=Convert-F3PeRva $rva $bytes $sections $length
                if ($null -eq $offset) {
                    $cv=[ordered]@{status='UNMAPPABLE_DEBUG_DIRECTORY';kind='UNKNOWN'}
                } else {
                    $table=Read-F3PeBytes $reader $offset ([int]$bytes) $length
                    for($i=0;$i -lt $table.Length;$i+=28) {
                        if ([BitConverter]::ToUInt32($table,$i+12) -ne 2) {continue}
                        $size=[BitConverter]::ToUInt32($table,$i+16)
                        $ptr=[BitConverter]::ToUInt32($table,$i+24)
                        if ($size -lt 25 -or $size -gt 4096) {continue}
                        $data=Read-F3PeBytes $reader ([long]$ptr) ([int]$size) $length
                        if ([Text.Encoding]::ASCII.GetString($data,0,4) -ne 'RSDS') {continue}
                        $tail=[byte[]]$data[24..($data.Length-1)]
                        $nullAt=[Array]::IndexOf($tail,[byte]0)
                        if ($nullAt -lt 1) {continue}
                        $rawName=[Text.Encoding]::ASCII.GetString($tail,0,$nullAt)
                        $name=($rawName -split '[\\/]' | Select-Object -Last 1)
                        # Strip ALL build-machine/source paths before output.
                        if ($name -cnotmatch '^[A-Za-z0-9._-]{1,100}\.pdb$') {
                            $name='UNAVAILABLE'
                        }
                        $id=($data[4..19] | ForEach-Object {$_.ToString('x2')}) -join ''
                        $cv=[ordered]@{
                            status='REFERENCE_ONLY_NOT_RESOLVED'
                            kind='RSDS'; pdbFileName=$name; guidBytesHex=$id
                            age=[BitConverter]::ToUInt32($data,20)
                        }
                        break
                    }
                }
            }
        }
        return [ordered]@{
            machine='AMD64'; sections=$n; sizeOfImage=$sizeOfImage
            timestampUnix=[BitConverter]::ToUInt32($coff,8)
            codeView=$cv
        }
    } finally {$reader.Dispose()}
}
# One original registered Desktop binary. No arbitrary file paths and no
# fallback to an unregistered/Store executable or old NativePilot override.
$target=Resolve-ManagerEducationTarget -RegisteredOnly
$before=Get-Item -LiteralPath $target -ErrorAction Stop
$pe=Read-F3PeMetadata -VerifiedPath $target
$hash=(Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant()
$after=Get-Item -LiteralPath $target -ErrorAction Stop
$signature=Get-AuthenticodeSignature -LiteralPath $target
if ($before.Length -ne $after.Length -or
    $before.LastWriteTimeUtc -ne $after.LastWriteTimeUtc -or
    $signature.Status -ne 'Valid' -or
    $signature.SignerCertificate.Subject -notmatch '^CN=Microsoft Corporation,') {
    throw 'Verified Education image changed or lost valid signature.'
}
# Do not reveal install paths, PDB source paths, binary or account data.
# A CodeView reference DOES NOT establish keyboard/Player/HUD ABI.
[ordered]@{
    schema=1;status='CLIENT_IMAGE_METADATA_ONLY'
    target='REGISTERED_SIGNED_EDUCATION_DESKTOP'
    fileName='Minecraft.Windows.exe'
    fileVersion=[string]$after.VersionInfo.FileVersion
    productVersion=[string]$after.VersionInfo.ProductVersion
    sha256=$hash;length=[long]$after.Length;pe=$pe
    f3InputAbi='NOT_ESTABLISHED'
    nativePlayerCoordinatesAbi='NOT_ESTABLISHED'
    inGameHudRendererAbi='NOT_ESTABLISHED'
    copyCoordinates='NOT_ESTABLISHED'
    executedGame=$false;modifiedGame=$false
    inspectedWorld=$false;proofCeiling='PE_METADATA_NOT_GAMEPLAY'
} | ConvertTo-Json -Depth 8 -Compress
