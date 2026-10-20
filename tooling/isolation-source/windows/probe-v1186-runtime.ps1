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
    return Game(path,pid,expectedStart,"MivubiEdu02");
  }
  public static GameIdentity Game(string path,int pid,long expectedStart,string expectedBox) {
    if(expectedBox!=null && expectedBox!="MivubiEdu02" && expectedBox!="MivubiEdu03" && expectedBox!="MivubiEdu04") throw new Exception("Unknown owned profile");
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
        int a=query(new IntPtr(pid),first,IntPtr.Zero,IntPtr.Zero,out one);
        int b=query(new IntPtr(pid),second,IntPtr.Zero,IntPtr.Zero,out two);
        bool valid=expectedBox==null ? a==unchecked((int)0xc000000b) && b==a :
           a==0 && b==0 && one==two && one==session && new string(first).TrimEnd('\0')==expectedBox && new string(second)==new string(first);
        if(!valid)
          throw new Exception("Owned Client 02 profile/session attribution failed");
        process.Refresh();
        if(process.HasExited || process.StartTime.ToUniversalTime()!=started) throw new Exception("Process identity lost during attribution");
        using(var current=System.Diagnostics.Process.GetProcessById(pid)) {
          if(current.StartTime.ToUniversalTime()!=started) throw new Exception("PID reused after attribution");
        }
        return new GameIdentity {Pid=pid,ProcessStartUtc=started.ToString("o"),Box=expectedBox??"NATIVE",WindowsSession=session,
          IdentityStable=true,WindowPresent=process.MainWindowHandle!=IntPtr.Zero,Responding=process.Responding};
      } finally {FreeLibrary(module);}
    }
  }
  public static uint[] BoxPids(string path,string box) {
    if(box!="MivubiEdu02" && box!="MivubiEdu03" && box!="MivubiEdu04") throw new Exception("Unknown owned profile");
    IntPtr module=LoadLibraryExW(path,IntPtr.Zero,0x1100), name=IntPtr.Zero;
    if(module==IntPtr.Zero) throw new Exception("Cannot load query DLL");
    try {
      IntPtr address=GetProcAddress(module,"SbieApi_EnumProcessEx");
      if(address==IntPtr.Zero) throw new Exception("Process inventory API unavailable");
      EnumAll query=(EnumAll)Marshal.GetDelegateForFunctionPointer(address,typeof(EnumAll));
      char[] padded=new char[34]; box.CopyTo(0,padded,0,box.Length);
      name=Marshal.AllocHGlobal(68); Marshal.Copy(padded,0,name,34);
      uint count=0;
      if(query(name,true,0,null,ref count)!=0 || count>256) throw new Exception("Box inventory unavailable/too large");
      uint[] first=new uint[512],second=new uint[512]; uint one=512,two=512;
      if(query(name,true,0,first,ref one)!=0 || query(name,true,0,second,ref two)!=0 || one!=count || two!=count)
        throw new Exception("Box inventory changed");
      Array.Resize(ref first,(int)count);Array.Resize(ref second,(int)count);Array.Sort(first);Array.Sort(second);
      for(int i=0;i<first.Length;i++) if(first[i]==0 || first[i]!=second[i] || (i>0 && first[i]==first[i-1])) throw new Exception("Box identity changed");
      return first;
    } finally {if(name!=IntPtr.Zero)Marshal.FreeHGlobal(name);FreeLibrary(module);}
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
public static class MivubiManagerWindows {
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr window,out uint pid);
  [DllImport("user32.dll")] static extern bool IsWindow(IntPtr window);
  [DllImport("user32.dll")] static extern bool IsIconic(IntPtr window);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr window);
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern bool ShowWindow(IntPtr window,int command);
  [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll",SetLastError=true)] static extern bool PostMessage(IntPtr window,uint message,IntPtr w,IntPtr l);
  [DllImport("user32.dll")] static extern IntPtr MonitorFromWindow(IntPtr window,uint flags);
  [StructLayout(LayoutKind.Sequential)] struct Rect {public int Left,Top,Right,Bottom;}
  [StructLayout(LayoutKind.Sequential)] struct Monitor {public uint Size;public Rect Bounds,Work;public uint Flags;}
  [DllImport("user32.dll")] static extern bool GetMonitorInfo(IntPtr monitor,ref Monitor info);
  [DllImport("user32.dll")] static extern bool SetWindowPos(IntPtr window,IntPtr after,int x,int y,int width,int height,uint flags);
  public static void Act(int pid,long fileTime,long handle,string operation) {
    using(var process=System.Diagnostics.Process.GetProcessById(pid)) {
      IntPtr window=new IntPtr(handle);uint actual;
      if(process.HasExited || process.StartTime.ToUniversalTime().ToFileTimeUtc()!=fileTime || !IsWindow(window) ||
         GetWindowThreadProcessId(window,out actual)==0 || actual!=pid) throw new Exception("Stale process/window identity");
      if(operation=="Close") {if(!PostMessage(window,0x10,IntPtr.Zero,IntPtr.Zero))throw new Exception("Windows refused normal close");return;}
      if(IsIconic(window) || !IsWindowVisible(window)) ShowWindow(window,9);
      if(!IsWindow(window) || GetWindowThreadProcessId(window,out actual)==0 || actual!=pid || process.HasExited) throw new Exception("Window identity changed after restore");
      if(operation=="Focus") {if(!IsWindowVisible(window) || !SetForegroundWindow(window) || GetForegroundWindow()!=window)throw new Exception("Windows refused focus; the game remains open in the background. Click Focus in the manager.");return;}
      if(operation!="PlaceLeft" && operation!="PlaceRight")throw new Exception("Unknown window operation");
      Monitor info=new Monitor();info.Size=(uint)Marshal.SizeOf(typeof(Monitor));
      if(!GetMonitorInfo(MonitorFromWindow(window,2),ref info))throw new Exception("Monitor work area unavailable");
      int width=(info.Work.Right-info.Work.Left)/2;
      if(width<=0 || !SetWindowPos(window,IntPtr.Zero,info.Work.Left+(operation=="PlaceRight"?width:0),info.Work.Top,width,info.Work.Bottom-info.Work.Top,0x14))throw new Exception("Windows refused placement");
    }
  }
}
'@
$resolved=(Resolve-Path -LiteralPath $DllPath).Path
$result=[MivubiRuntimeProbe]::Read($resolved) | ConvertFrom-Json
if ($GameProcessId -ne 0) {
    if ($result.version -cne '5.73.6-MIVU' -or $result.abi -ne 0x57230) { throw 'Private driver required for owned game attribution.' }
    $game=Get-Process -Id $GameProcessId
    $started=$game.StartTime.ToUniversalTime().Ticks
    . (Join-Path $PSScriptRoot 'education-target.ps1')
    if ($game.Path -ine (Resolve-ManagerEducationTarget)) { throw 'Unexpected game image path.' }
    $signature=Get-AuthenticodeSignature -LiteralPath $game.Path
    if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch '^CN=Microsoft Corporation,') { throw 'Official Education image required.' }
    $identity=[MivubiRuntimeProbe]::Game($resolved,$GameProcessId,$started)
    $result | Add-Member -NotePropertyName game -NotePropertyValue $identity
}
$result | ConvertTo-Json -Depth 4 -Compress
