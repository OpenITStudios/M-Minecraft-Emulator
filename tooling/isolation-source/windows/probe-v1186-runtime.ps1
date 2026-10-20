# Short-lived, read-only process: release all vendor DLL/device handles on exit.
[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$DllPath,
      [Parameter(Mandatory=$true)][ValidatePattern('^[a-f0-9]{64}$')][string]$Sha256)
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
[MivubiRuntimeProbe]::Read((Resolve-Path -LiteralPath $DllPath).Path)
