[CmdletBinding()]
param(
  [ValidateSet('VMWARE', 'APP_SANDBOX_GPU_PV')]
  [string]$Provider = 'VMWARE',
  [ValidateSet('ENVIRONMENT', 'INTERACTIVE')]
  [string]$Stage = 'ENVIRONMENT',
  [ValidateSet('YES', 'NO', 'UNKNOWN')]
  [string]$MenuRendered = 'UNKNOWN',
  [ValidateSet('YES', 'NO', 'UNKNOWN')]
  [string]$WorldRendered = 'UNKNOWN',
  [ValidateSet('YES', 'NO', 'UNKNOWN')]
  [string]$InputResponsive = 'UNKNOWN',
  [ValidateSet('YES', 'NO', 'UNKNOWN')]
  [string]$BlockingGraphicsError = 'UNKNOWN',
  [ValidateSet('YES', 'NO', 'UNKNOWN')]
  [string]$VmDesktopOpened = 'UNKNOWN',
  [ValidateSet('YES', 'NO', 'UNKNOWN')]
  [string]$EducationInstalled = 'UNKNOWN',
  [ValidateSet('YES', 'NO', 'UNKNOWN')]
  [string]$EducationSignedIn = 'UNKNOWN',
  [ValidateSet('YES', 'NO', 'UNKNOWN')]
  [string]$CrashOrFreezeObserved = 'UNKNOWN',
  [ValidateRange(0, 1440)]
  [int]$ObservedMinutes = 0,
  [ValidatePattern('^(?:$|[a-zA-Z0-9_.-]{1,80})$')]
  [string]$EvidenceReference = '',
  [string]$OutputDirectory = ''
)

$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT') {
  throw 'Client feasibility must be collected inside the interactive Windows guest.'
}
if ($Stage -eq 'ENVIRONMENT' -and (
  $MenuRendered -ne 'UNKNOWN' -or $WorldRendered -ne 'UNKNOWN' -or
  $InputResponsive -ne 'UNKNOWN' -or $BlockingGraphicsError -ne 'UNKNOWN' -or
  $VmDesktopOpened -ne 'UNKNOWN' -or $EducationInstalled -ne 'UNKNOWN' -or
  $EducationSignedIn -ne 'UNKNOWN' -or $CrashOrFreezeObserved -ne 'UNKNOWN' -or
  $ObservedMinutes -ne 0 -or $EvidenceReference
)) {
  throw 'Operator game observations belong to INTERACTIVE, not ENVIRONMENT.'
}

if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
  $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
  $OutputDirectory = Join-Path $env:TEMP ("MinecraftEducation-Guest-Feasibility-$stamp-" + [guid]::NewGuid().ToString('N').Substring(0, 8))
}
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null

# This is a bounded probe. It creates no VM, installs no drivers and never
# reads browser/Microsoft tokens, account identifiers, world data or logs.
function Get-OpenGlObservation {
  $result = [ordered]@{
    status = 'UNKNOWN'
    version = $null
    renderer = $null
    vendor = $null
    reasonCode = 'PROBE_NOT_EXECUTED'
  }

  # A VMX mks.enable3d setting is not proof of a working OpenGL context.
  # Probe the Windows guest's current interactive display driver instead.
  if ([System.Diagnostics.Process]::GetCurrentProcess().SessionId -eq 0) {
    $result.reasonCode = 'NON_INTERACTIVE_SESSION'
    return $result
  }
  if (-not [Environment]::Is64BitProcess) {
    $result.reasonCode = 'REQUIRES_64_BIT_POWERSHELL'
    return $result
  }

  $source = @'
using System;
using System.Runtime.InteropServices;

public static class MinecraftGuestOpenGlProbe {
    [StructLayout(LayoutKind.Sequential)]
    private struct PixelFormatDescriptor {
        public ushort nSize;
        public ushort nVersion;
        public uint dwFlags;
        public byte iPixelType;
        public byte cColorBits, cRedBits, cRedShift, cGreenBits, cGreenShift;
        public byte cBlueBits, cBlueShift, cAlphaBits, cAlphaShift;
        public byte cAccumBits, cAccumRedBits, cAccumGreenBits, cAccumBlueBits;
        public byte cDepthBits, cStencilBits, cAuxBuffers, iLayerType, bReserved;
        public uint dwLayerMask, dwVisibleMask, dwDamageMask;
    }

    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern IntPtr CreateWindowEx(uint exStyle, string className, string windowName,
        uint style, int x, int y, int width, int height, IntPtr parent, IntPtr menu, IntPtr instance, IntPtr parameter);
    [DllImport("user32.dll")] private static extern bool DestroyWindow(IntPtr window);
    [DllImport("user32.dll")] private static extern IntPtr GetDC(IntPtr window);
    [DllImport("user32.dll")] private static extern int ReleaseDC(IntPtr window, IntPtr dc);
    [DllImport("gdi32.dll")] private static extern int ChoosePixelFormat(IntPtr dc, ref PixelFormatDescriptor pfd);
    [DllImport("gdi32.dll")] private static extern bool SetPixelFormat(IntPtr dc, int format, ref PixelFormatDescriptor pfd);
    [DllImport("opengl32.dll")] private static extern IntPtr wglCreateContext(IntPtr dc);
    [DllImport("opengl32.dll")] private static extern bool wglMakeCurrent(IntPtr dc, IntPtr context);
    [DllImport("opengl32.dll")] private static extern bool wglDeleteContext(IntPtr context);
    [DllImport("opengl32.dll")] private static extern IntPtr glGetString(uint name);

    public static string[] Observe() {
        IntPtr window = IntPtr.Zero, dc = IntPtr.Zero, context = IntPtr.Zero;
        try {
            window = CreateWindowEx(0, "STATIC", "Minecraft Guest Graphics Probe", 0,
                0, 0, 1, 1, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero);
            if (window == IntPtr.Zero) throw new InvalidOperationException("WINDOW_CREATION_FAILED");
            dc = GetDC(window);
            if (dc == IntPtr.Zero) throw new InvalidOperationException("DEVICE_CONTEXT_UNAVAILABLE");
            PixelFormatDescriptor pfd = new PixelFormatDescriptor {
                nSize = (ushort)Marshal.SizeOf(typeof(PixelFormatDescriptor)),
                nVersion = 1, dwFlags = 0x00000025, iPixelType = 0,
                cColorBits = 24, cDepthBits = 24
            };
            int format = ChoosePixelFormat(dc, ref pfd);
            if (format <= 0 || !SetPixelFormat(dc, format, ref pfd))
                throw new InvalidOperationException("OPENGL_PIXEL_FORMAT_UNAVAILABLE");
            context = wglCreateContext(dc);
            if (context == IntPtr.Zero || !wglMakeCurrent(dc, context))
                throw new InvalidOperationException("OPENGL_CONTEXT_UNAVAILABLE");
            IntPtr version = glGetString(0x1F02);
            IntPtr renderer = glGetString(0x1F01);
            IntPtr vendor = glGetString(0x1F00);
            if (version == IntPtr.Zero) throw new InvalidOperationException("OPENGL_VERSION_UNAVAILABLE");
            return new string[] {
                Marshal.PtrToStringAnsi(version),
                renderer == IntPtr.Zero ? null : Marshal.PtrToStringAnsi(renderer),
                vendor == IntPtr.Zero ? null : Marshal.PtrToStringAnsi(vendor)
            };
        } finally {
            if (context != IntPtr.Zero) {
                wglMakeCurrent(IntPtr.Zero, IntPtr.Zero);
                wglDeleteContext(context);
            }
            if (dc != IntPtr.Zero && window != IntPtr.Zero) ReleaseDC(window, dc);
            if (window != IntPtr.Zero) DestroyWindow(window);
        }
    }
}
'@
  try {
    Add-Type -TypeDefinition $source -Language CSharp -ErrorAction Stop
    $observed = [MinecraftGuestOpenGlProbe]::Observe()
    if ($observed.Count -ne 3 -or [string]::IsNullOrWhiteSpace($observed[0])) {
      throw 'OPENGL_VERSION_UNAVAILABLE'
    }
    $result.status = 'MEASURED'
    $result.version = [string]$observed[0]
    $result.renderer = [string]$observed[1]
    $result.vendor = [string]$observed[2]
    $result.reasonCode = $null
  } catch {
    # Never substitute the VMware configuration flag or DirectX capability
    # for an actual OpenGL measurement. Fail to UNKNOWN, not PASS/FAIL.
    $result.status = 'UNKNOWN'
    $result.version = $null
    $result.renderer = $null
    $result.vendor = $null
    $result.reasonCode = 'OPENGL_CONTEXT_PROBE_FAILED'
  }
  return $result
}

function Get-Direct3D11Observation {
  $result = [ordered]@{
    status = 'UNKNOWN'
    featureLevel = $null
    driverType = $null
    reasonCode = 'PROBE_NOT_EXECUTED'
  }
  if ([System.Diagnostics.Process]::GetCurrentProcess().SessionId -eq 0) {
    $result.reasonCode = 'NON_INTERACTIVE_SESSION'
    return $result
  }
  if (-not [Environment]::Is64BitProcess) {
    $result.reasonCode = 'REQUIRES_64_BIT_POWERSHELL'
    return $result
  }
  # D3D_DRIVER_TYPE_HARDWARE=1, D3D_FEATURE_LEVEL_11_0=0xB000,
  # D3D11_SDK_VERSION=7. No WARP/REFERENCE/SOFTWARE fallback.
  # Microsoft D3D11CreateDevice: learn.microsoft.com/en-us/windows/win32/api/d3d11/nf-d3d11-d3d11createdevice
  $source = @'
using System;
using System.Runtime.InteropServices;
public static class MinecraftGuestD3D11HardwareProbe {
    [DllImport("d3d11.dll", ExactSpelling = true)]
    private static extern int D3D11CreateDevice(
        IntPtr adapter, int driverType, IntPtr software, uint flags,
        [In, MarshalAs(UnmanagedType.LPArray)] uint[] levels, uint count,
        uint sdkVersion, out IntPtr device, out uint selected, out IntPtr context);

    public static bool SupportsHardware11_0() {
        IntPtr device = IntPtr.Zero, context = IntPtr.Zero;
        try {
            uint selected;
            int hr = D3D11CreateDevice(IntPtr.Zero, 1, IntPtr.Zero, 0,
                new uint[] { 0xB000 }, 1, 7, out device, out selected, out context);
            return hr >= 0 && selected == 0xB000 && device != IntPtr.Zero;
        } finally {
            if (context != IntPtr.Zero) Marshal.Release(context);
            if (device != IntPtr.Zero) Marshal.Release(device);
        }
    }
}
'@
  try {
    Add-Type -TypeDefinition $source -Language CSharp -ErrorAction Stop
    if ([MinecraftGuestD3D11HardwareProbe]::SupportsHardware11_0()) {
      $result.status = 'MEASURED'
      $result.featureLevel = '11_0'
      $result.driverType = 'HARDWARE'
      $result.reasonCode = $null
    } else {
      $result.reasonCode = 'D3D11_HARDWARE_FL11_0_UNAVAILABLE'
    }
  } catch {
    $result.reasonCode = 'D3D11_HARDWARE_PROBE_FAILED'
  }
  return $result
}

function Get-SafeWindowsObservation {
  $osInfo = $null
  try { $osInfo = Get-CimInstance Win32_OperatingSystem -ErrorAction Stop } catch {}
  $video = @()
  try {
    $video = @(Get-CimInstance Win32_VideoController -ErrorAction Stop | Select-Object -First 8 |
      ForEach-Object {
        [ordered]@{
          name = [string]$_.Name
          driverVersion = [string]$_.DriverVersion
          status = [string]$_.Status
        }
      })
  } catch {}
  $vmwareTools = $null
  try {
    $service = Get-Service -Name 'VMTools' -ErrorAction Stop
    $vmwareTools = ($service.Status -eq 'Running')
  } catch {}
  return [ordered]@{
    name = if ($null -ne $osInfo) { [string]$osInfo.Caption } else { $null }
    version = if ($null -ne $osInfo) { [string]$osInfo.Version } else { $null }
    architecture = if ($null -ne $osInfo) { [string]$osInfo.OSArchitecture } else { $null }
    totalMemoryMb = if ($null -ne $osInfo) { [long][math]::Floor([double]$osInfo.TotalVisibleMemorySize / 1024) } else { $null }
    interactiveSession = ([System.Diagnostics.Process]::GetCurrentProcess().SessionId -ne 0)
    vmwareToolsRunning = $vmwareTools
    displayAdapters = $video
  }
}

function Get-SafeMinecraftObservation {
  $desktopVersion = $null
  $storeVersion = $null
  try {
    # Match the backend's /reg:32 Minecraft Education desktop-version lookup.
    $root = [Microsoft.Win32.RegistryKey]::OpenBaseKey(
      [Microsoft.Win32.RegistryHive]::LocalMachine, [Microsoft.Win32.RegistryView]::Registry32)
    try {
      $key = $root.OpenSubKey('SOFTWARE\Microsoft\Microsoft Studios\Minecraft Education Edition')
      if ($null -ne $key) {
        try {
          $value = $key.GetValue('Version')
          if ($null -ne $value) { $desktopVersion = [string]$value }
        } finally { $key.Dispose() }
      }
    } finally { $root.Dispose() }
  } catch {}
  try {
    $store = Get-AppxPackage -Name 'Microsoft.MinecraftEducationEdition' -ErrorAction Stop | Select-Object -First 1
    if ($null -ne $store) { $storeVersion = [string]$store.Version }
  } catch {}

  # This intentionally reuses the existing Guest Agent's broad discovery
  # heuristic. It is CANDIDATE evidence only, never proof of game rendering.
  $candidateCount = $null
  try {
    $candidateCount = @(Get-Process -ErrorAction Stop | Where-Object {
      $_.ProcessName -like '*Minecraft*' -or $_.ProcessName -like '*Education*'
    }).Count
  } catch {}
  return [ordered]@{
    desktopInstallVersion = $desktopVersion
    storeInstallVersion = $storeVersion
    candidateProcessCount = $candidateCount
    processEvidenceClass = 'CANDIDATE_ONLY'
    gameplayState = 'UNVERIFIED'
  }
}

$report = [ordered]@{
  schema = 1
  scope = 'CLIENT_FEASIBILITY'
  stage = $Stage
  provider = $Provider
  collectedAtUtc = [DateTime]::UtcNow.ToString('o')
  evidenceOrigin = 'WINDOWS_GUEST'
  automaticVerdict = 'NOT_PROVIDED'
  windows = Get-SafeWindowsObservation
  graphics = [ordered]@{
    openGl = Get-OpenGlObservation
    direct3D11 = Get-Direct3D11Observation
    publishedMinecraftOpenGlReference = '4.4'
    referenceIsNotAnAutomaticVerdict = $true
  }
  minecraft = Get-SafeMinecraftObservation
  operator = [ordered]@{
    menuRendered = $MenuRendered
    worldRendered = $WorldRendered
    inputResponsive = $InputResponsive
    blockingGraphicsError = $BlockingGraphicsError
    vmDesktopOpened = $VmDesktopOpened
    educationInstalled = $EducationInstalled
    educationSignedIn = $EducationSignedIn
    crashOrFreezeObserved = $CrashOrFreezeObserved
    observedMinutes = $ObservedMinutes
    evidenceReference = if ($EvidenceReference) { $EvidenceReference } else { $null }
    attribution = 'OPERATOR_DECLARED_NOT_MACHINE_VERIFIED'
  }
}

$path = Join-Path $OutputDirectory ("client-feasibility-" + $Stage.ToLowerInvariant() + ".json")
$json = $report | ConvertTo-Json -Depth 9
[System.IO.File]::WriteAllText($path, $json, ([System.Text.UTF8Encoding]::new($false)))
[ordered]@{
  report = $path
  stage = $Stage
  automaticVerdict = 'NOT_PROVIDED'
  next = if ($Stage -eq 'ENVIRONMENT') { 'INTERACTIVE_GAME_OBSERVATION_REQUIRED' } else { 'REVIEW_OPERATOR_AND_GRAPHICS_EVIDENCE' }
} | ConvertTo-Json -Depth 3
