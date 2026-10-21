# Explicit synthetic-only native Windows proof; no game/account/service operations.
[CmdletBinding()]
param([switch]$VerifyNegativeControl)
$ErrorActionPreference='Stop'
if($env:OS -ne 'Windows_NT'){throw 'Windows AppContainer fixture requires Windows.'}
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../..'))
$workspace=Join-Path $root 'workspace'
if((Get-Item -LiteralPath $workspace -Force).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Fixture workspace cannot be a reparse point.'}
$output=Join-Path $workspace ('appcontainer-probe-'+[Guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($output)|Out-Null
$locator=Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
if(-not(Test-Path -LiteralPath $locator -PathType Leaf)){throw 'Installed MSVC Build Tools locator is unavailable.'}
$installation=& $locator -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if($LASTEXITCODE -ne 0 -or -not $installation){throw 'Installed x64 MSVC Build Tools unavailable.'}
$developer=Join-Path $installation 'Common7/Tools/VsDevCmd.bat'
$source=Join-Path $PSScriptRoot 'appcontainer-probe.cpp'
$binary=Join-Path $output 'AppContainerProbe.exe'
$object=Join-Path $output 'probe.obj'
foreach($path in @($developer,$source,$binary,$object)){if($path -match '["%&|<>^!\r\n]'){throw 'Compiler paths contain unsupported shell characters.'}}
$build=Join-Path $output 'build.cmd'
$lines=@('@echo off',('call "'+$developer+'" -arch=amd64 -host_arch=amd64'),'if errorlevel 1 exit /b 1',('cl /nologo /std:c++20 /EHsc /MT /W4 /WX /DUNICODE /D_UNICODE "'+$source+'" /Fe:"'+$binary+'" /Fo:"'+$object+'" /link userenv.lib advapi32.lib ole32.lib shell32.lib'))
[IO.File]::WriteAllLines($build,$lines,[Text.Encoding]::ASCII)
& $env:ComSpec /d /c $build *> (Join-Path $output 'compile.log')
if($LASTEXITCODE -ne 0){throw "Synthetic compilation failed; inspect $output\compile.log"}
function Invoke-SyntheticProbe([string]$Mode,[string]$Directory){
    $start=New-Object Diagnostics.ProcessStartInfo
    $start.FileName=$binary
    $start.Arguments=$Mode+' "'+$Directory+'"'
    $start.UseShellExecute=$false
    $start.CreateNoWindow=$true
    $start.RedirectStandardOutput=$true
    $start.RedirectStandardError=$true
    $process=[Diagnostics.Process]::Start($start)
    try{
    if(-not $process.WaitForExit(30000)){ $process.Kill();throw "Synthetic probe timed out. Evidence retained in $output; fixture children expire after 20 seconds." }
    $text=$process.StandardOutput.ReadToEnd()
    [IO.File]::WriteAllText((Join-Path $Directory 'result.json'),$text,[Text.UTF8Encoding]::new($false))
    $proof=$text|ConvertFrom-Json
    $proof|ConvertTo-Json -Compress
    if($Mode -eq '--negative-control'){
        if($process.ExitCode -ne 1 -or $proof.result -ne 'FAIL' -or $proof.stage -ne 'actual-distinct-container-tokens' -or -not $proof.profilesRemoved){throw "Negative control did not fail safely at its expected identity check; inspect $Directory"}
    }elseif($process.ExitCode -ne 0 -or $proof.result -ne 'PASS'){throw "Native AppContainer proof failed; evidence retained in $Directory"}
    if($Mode -eq '--run'){
        if(-not $proof.syntheticOnly -or -not $proof.syntheticPrivateRoamingWritten -or
           -not $proof.syntheticPrivateRoamingPeerDenied -or
           -not $proof.syntheticPrivateRoamingPersistsAfterRestart -or
           $proof.minecraftRoamingCompatibility -ne 'NOT_EXECUTED' -or
           $proof.minecraftAcceptance -ne 'NOT_EXECUTED'){
            throw "Synthetic roaming contract missing or overclaimed; inspect $Directory"
        }
    }
    }finally{$process.Dispose()}
}
Invoke-SyntheticProbe '--run' $output
if($VerifyNegativeControl){
    $negative=Join-Path $output 'negative-control'
    [IO.Directory]::CreateDirectory($negative)|Out-Null
    Invoke-SyntheticProbe '--negative-control' $negative
}
