# Short-lived, read-only process: release all vendor DLL/device handles on exit.
[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$DllPath,
      [Parameter(Mandatory=$true)][ValidatePattern('^[a-f0-9]{64}$')][string]$Sha256,
      [ValidateRange(0,2147483647)][int]$GameProcessId=0)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ((Get-FileHash -LiteralPath $DllPath).Hash.ToLowerInvariant() -cne $Sha256 -or
    (Get-AuthenticodeSignature -LiteralPath $DllPath).Status -ne 'Valid') { throw 'Probe DLL integrity/signature refused.' }
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class MivubiRuntimeProbe {
  [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
  static extern IntPtr LoadLibraryExW(string path, IntPtr reserved, uint flags);
  [DllImport("kernel32.dll", CharSet=CharSet.Ansi, ExactSpelling=true)]
  static extern IntPtr GetProcAddress(IntPtr module, string name);
  [DllImport("kernel32.dll")] static extern bool FreeLibrary(IntPtr module);
  [DllImport("ntdll.dll")] static extern int NtQuerySystemInformation(int kind, ref CI info, int length, out int returned);
  [StructLayout(LayoutKind.Sequential)] struct CI { public uint Length; public uint Options; }
  [UnmanagedFunctionPointer(CallingConvention.StdCall, CharSet=CharSet.Unicode)]
  delegate int Version([Out,MarshalAs(UnmanagedType.LPArray,SizeConst=16)] char[] version, out uint abi);
  [UnmanagedFunctionPointer(CallingConvention.StdCall)]
  delegate int EnumAll(IntPtr box, [MarshalAs(UnmanagedType.U1)] bool allSessions, uint session,
    [Out,MarshalAs(UnmanagedType.LPArray)] uint[] pids, ref uint count);
  [UnmanagedFunctionPointer(CallingConvention.StdCall,CharSet=CharSet.Unicode)]
  delegate int QueryProcess(IntPtr pid,[Out,MarshalAs(UnmanagedType.LPArray,SizeConst=34)] char[] box,IntPtr image,IntPtr sid,out uint session);
  public sealed class GameIdentity {
    public int Pid; public string ProcessStartUtc; public string Box;
    public int WindowsSession; public bool IdentityStable; public bool WindowPresent; public bool Responding;
  }
  public static GameIdentity Game(string path,int pid,long expectedStart) {
    using(var process=System.Diagnostics.Process.GetProcessById(pid)) {
      DateTime started=process.StartTime.ToUniversalTime(); int session=process.SessionId;
      if(started.Ticks!=expectedStart) throw new Exception("PID reused before attribution");
      IntPtr module=LoadLibraryExW(path,IntPtr.Zero,0x1100);
      if(module==IntPtr.Zero) throw new Exception("Query DLL load failed");
      try {
        IntPtr address=GetProcAddress(module,"SbieApi_QueryProcess");
        if(address==IntPtr.Zero) throw new Exception("Process attribution API unavailable");
        QueryProcess query=(QueryProcess)Marshal.GetDelegateForFunctionPointer(address,typeof(QueryProcess));
        char[] first=new char[34],second=new char[34]; uint one,two;
        if(query(new IntPtr(pid),first,IntPtr.Zero,IntPtr.Zero,out one)!=0 ||
           query(new IntPtr(pid),second,IntPtr.Zero,IntPtr.Zero,out two)!=0 || one!=two || one!=session ||
           new string(first).TrimEnd('\0')!="MivubiEdu02" || new string(second)!=new string(first))
          throw new Exception("Owned Client 02 profile/session attribution failed");
        process.Refresh();
        if(process.HasExited || process.StartTime.ToUniversalTime()!=started) throw new Exception("Process identity lost during attribution");
        using(var current=System.Diagnostics.Process.GetProcessById(pid)) {
          if(current.StartTime.ToUniversalTime()!=started) throw new Exception("PID reused after attribution");
        }
        return new GameIdentity {Pid=pid,ProcessStartUtc=started.ToString("o"),Box="MivubiEdu02",WindowsSession=session,
          IdentityStable=true,WindowPresent=process.MainWindowHandle!=IntPtr.Zero,Responding=process.Responding};
      } finally {FreeLibrary(module);}
    }
  }
  public static string Read(string path) {
    CI ci = new CI(); ci.Length=8; int returned;
    if(NtQuerySystemInformation(103,ref ci,8,out returned)!=0 || returned!=8) throw new Exception("Code integrity state unavailable");
    IntPtr module=LoadLibraryExW(path,IntPtr.Zero,0x1100);
    if(module==IntPtr.Zero) throw new Exception("Cannot load verified query DLL");
    try {
      IntPtr vp=GetProcAddress(module,"SbieApi_GetVersionEx"), ep=GetProcAddress(module,"SbieApi_EnumProcessEx");
      if(vp==IntPtr.Zero || ep==IntPtr.Zero) throw new Exception("Missing read-only query exports");
      Version ver=(Version)Marshal.GetDelegateForFunctionPointer(vp,typeof(Version));
      EnumAll query=(EnumAll)Marshal.GetDelegateForFunctionPointer(ep,typeof(EnumAll));
      char[] text=new char[16]; uint abi;
      if(ver(text,out abi)!=0) throw new Exception("Driver version query failed");
      // Null box + allSessions=true enumerates ALL boxed processes, including helpers.
      uint first=0, second=0;
      if(query(IntPtr.Zero,true,0,null,ref first)!=0 || query(IntPtr.Zero,true,0,null,ref second)!=0 || first!=second)
        throw new Exception("All-session process count unavailable or unstable");
      string version=new string(text).TrimEnd('\0');
      if(!System.Text.RegularExpressions.Regex.IsMatch(version,@"^[A-Za-z0-9.\-]{1,15}$")) throw new Exception("Unexpected driver identity");
      return "{\"schema\":1,\"version\":\""+version+"\",\"abi\":"+abi+",\"allSessionBoxedProcesses\":"+first+",\"testSigningActive\":"+(((ci.Options&2)!=0)?"true":"false")+"}";
    } finally { FreeLibrary(module); }
  }
}
'@
$resolved=(Resolve-Path -LiteralPath $DllPath).Path
$result=[MivubiRuntimeProbe]::Read($resolved) | ConvertFrom-Json
if ($GameProcessId -ne 0) {
    if ($result.version -cne '5.73.6-MIVU' -or $result.abi -ne 0x57230) { throw 'Private driver required for owned game attribution.' }
    $game=Get-Process -Id $GameProcessId
    $started=$game.StartTime.ToUniversalTime().Ticks
    if ($game.Path -cne 'C:\Program Files\Microsoft Studios\Minecraft Education\Minecraft.Windows.exe') { throw 'Unexpected game image path.' }
    $signature=Get-AuthenticodeSignature -LiteralPath $game.Path
    if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch '^CN=Microsoft Corporation,') { throw 'Official Education image required.' }
    $identity=[MivubiRuntimeProbe]::Game($resolved,$GameProcessId,$started)
    $result | Add-Member -NotePropertyName game -NotePropertyValue $identity
}
$result | ConvertTo-Json -Depth 4 -Compress
