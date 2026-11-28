# Resolve Windows cmdlets from the Windows PowerShell installation, not inherited PS7 modules.
foreach($name in @('Utility','Security','Management')) {
    Import-Module (Join-Path $PSHOME ('Modules/Microsoft.PowerShell.'+$name+'/Microsoft.PowerShell.'+$name+'.psd1')) -ErrorAction Stop
}
# Observe the existing Windows Desktop updater only. No task creation, changes,
# invocation, installer downloads or elevation. State is a point-in-time hint;
# the original image pin/registration checks still own executable admission.
function Get-ManagerEducationUpdaterState {
    try {
        $scheduler=New-Object -ComObject 'Schedule.Service' -ErrorAction Stop
        $scheduler.Connect()
        $tasks=$scheduler.GetFolder('\').GetTasks(1)
        $found=$false
        $ready=$false
        $unknown=$false
        foreach($task in $tasks) {
            if($task.Name -notin @('Minecraft Education Automatic Updater','Minecraft Education Edition Automatic Updater')){continue}
            $found=$true
            switch([int]$task.State) {
                1 {}
                2 {return 'Busy'}
                3 {$ready=$true}
                4 {return 'Busy'}
                default {$unknown=$true}
            }
        }
        if($unknown){return 'Unknown'}
        if($ready){return 'Ready'}
        if($found){return 'Disabled'}
        return 'Missing'
    } catch {return 'Unknown'}
}
# One installation resolver for native and owned manager operations.
function Resolve-ManagerEducationTarget {
    param([switch]$RegisteredOnly,[switch]$RequireUpdaterIdle)
    $registered=@(foreach($view in @([Microsoft.Win32.RegistryView]::Registry32,[Microsoft.Win32.RegistryView]::Registry64)){
        $key=[Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::LocalMachine,$view)
        try {$entry=$key.OpenSubKey('SOFTWARE\Microsoft\Microsoft Studios\Minecraft Education Edition');if($entry){try{$path=[string]$entry.GetValue('Path','');if($path){Join-Path $path 'Minecraft.Windows.exe'}}finally{$entry.Dispose()}}}finally{$key.Dispose()}
    })
    if($registered.Count -ne 1){throw 'Exactly one registered Minecraft Education Desktop installation is required.'}
    $expected=[IO.Path]::GetFullPath($registered[0])
    $selected=$expected
    if(-not $RegisteredOnly) {
        # Legacy developer recovery retains its old preference behavior.
        # Native preview never reads or trusts the retired NativePilot file.
        $settingsPath=Join-Path $env:LOCALAPPDATA 'MinecraftInstanceManager\NativePilot\manager-settings.json'
        if(Test-Path -LiteralPath $settingsPath){
            $settings=Get-Content -LiteralPath $settingsPath -Raw|ConvertFrom-Json
            if($settings.minecraftExecutable){$selected=[IO.Path]::GetFullPath($settings.minecraftExecutable)}
        }
        if($selected -ine $expected){throw 'Selected Minecraft executable does not match the registered Desktop installation. Update Settings.'}
    }
    $signature=Get-AuthenticodeSignature -LiteralPath $selected
    if($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch '^CN=Microsoft Corporation,'){throw 'Registered Minecraft Education executable has an invalid Microsoft signature.'}
    if($RequireUpdaterIdle) {
        $state=Get-ManagerEducationUpdaterState
        if($state -eq 'Busy'){throw 'Minecraft Education updater is queued or running. Wait for the update to finish before launching.'}
        if($state -eq 'Unknown'){throw 'Minecraft Education updater state could not be checked. Refresh after checking Windows Task Scheduler.'}
    }
    return $selected
}
function Assert-ManagerRegisteredAppId([string]$appId,[string]$selectedTarget) {
    if($appId -notmatch '^(\{[0-9A-Fa-f-]{36}\})\\(.+)$'){throw 'Registered Desktop app cannot be matched to the selected executable.'}
    $folder=[Guid]$Matches[1];$relative=$Matches[2]
    if(-not ('MivubiKnownFolder' -as [type])) {
        Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public static class MivubiKnownFolder {
 [DllImport("shell32.dll")] static extern int SHGetKnownFolderPath(ref Guid id,uint flags,IntPtr token,out IntPtr path);
 public static string Read(Guid id) { IntPtr p=IntPtr.Zero; try { int status=SHGetKnownFolderPath(ref id,0,IntPtr.Zero,out p); if(status!=0) Marshal.ThrowExceptionForHR(status); return Marshal.PtrToStringUni(p); } finally { if(p!=IntPtr.Zero) Marshal.FreeCoTaskMem(p); } }
}
"@
    }
    $actual=[IO.Path]::GetFullPath((Join-Path ([MivubiKnownFolder]::Read($folder)) $relative))
    if($actual -ine $selectedTarget){throw 'Registered Windows launcher points to a different Minecraft installation.'}
}
