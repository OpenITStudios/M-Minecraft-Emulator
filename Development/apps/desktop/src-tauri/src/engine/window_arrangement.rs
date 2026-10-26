use serde::{Deserialize, Serialize};
use std::{collections::HashSet, io};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WindowLayout {
    Grid,
    Focus,
    Columns,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowLayoutRequest {
    pub layout: WindowLayout,
    pub display_index: usize,
    pub main_window: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayInfo {
    pub index: usize,
    pub primary: bool,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowArrangementResult {
    pub schema: u32,
    pub layout: WindowLayout,
    pub display_index: usize,
    pub arranged: Vec<String>,
    pub missing: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlay_applied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlay_warning: Option<String>,
}

fn validate_request(request: &WindowLayoutRequest) -> io::Result<()> {
    if let Some(main) = request.main_window.as_deref() {
        if !matches!(main, "Native" | "Virtual-01" | "Virtual-02" | "Virtual-03") {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "main window is not a known client"));
        }
    }
    if matches!(request.layout, WindowLayout::Focus) && request.main_window.is_none() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Focus layout requires a main window"));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

fn calculate_slots(layout: &WindowLayout, area: Rect, count: usize) -> Vec<Rect> {
    if count == 0 { return Vec::new(); }
    if count == 1 { return vec![area]; }

    match layout {
        WindowLayout::Columns => {
            let width = area.width / count as i32;
            (0..count).map(|index| {
                let x = area.x + index as i32 * width;
                Rect {
                    x,
                    y: area.y,
                    width: if index + 1 == count { area.x + area.width - x } else { width },
                    height: area.height,
                }
            }).collect()
        }
        WindowLayout::Focus => {
            let main_width = (area.width as f64 * 0.68).floor() as i32;
            let side_width = area.width - main_width;
            let side_count = count - 1;
            let side_height = area.height / side_count as i32;
            let mut slots = vec![Rect { x: area.x, y: area.y, width: main_width, height: area.height }];
            slots.extend((0..side_count).map(|index| {
                let y = area.y + index as i32 * side_height;
                Rect {
                    x: area.x + main_width,
                    y,
                    width: side_width,
                    height: if index + 1 == side_count { area.y + area.height - y } else { side_height },
                }
            }));
            slots
        }
        WindowLayout::Grid if count == 2 => {
            let width = area.width / 2;
            vec![
                Rect { x: area.x, y: area.y, width, height: area.height },
                Rect { x: area.x + width, y: area.y, width: area.width - width, height: area.height },
            ]
        }
        WindowLayout::Grid if count == 3 => {
            let width = area.width / 2;
            let height = area.height / 2;
            vec![
                Rect { x: area.x, y: area.y, width, height },
                Rect { x: area.x + width, y: area.y, width: area.width - width, height },
                Rect { x: area.x, y: area.y + height, width: area.width, height: area.height - height },
            ]
        }
        WindowLayout::Grid => {
            let width = area.width / 2;
            let height = area.height / 2;
            (0..count.min(4)).map(|index| {
                let column = index % 2;
                let row = index / 2;
                let x = area.x + column as i32 * width;
                let y = area.y + row as i32 * height;
                Rect {
                    x,
                    y,
                    width: if column == 1 { area.x + area.width - x } else { width },
                    height: if row == 1 { area.y + area.height - y } else { height },
                }
            }).collect()
        }
    }
}

#[cfg(target_os = "windows")]
pub fn displays() -> io::Result<Vec<DisplayInfo>> {
    use std::{path::Path, time::Duration};
    const SCRIPT: &str = r#"
Add-Type -AssemblyName System.Windows.Forms
$index = 0
@([System.Windows.Forms.Screen]::AllScreens | ForEach-Object {
  $area = $_.WorkingArea
  [PSCustomObject]@{
    index = $index
    primary = $_.Primary
    width = $area.Width
    height = $area.Height
  }
  $index++
}) | ConvertTo-Json -Compress
"#;
    let output = virtual_clients_core::bounded_command_output(
        Path::new("powershell.exe"),
        ["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", SCRIPT],
        Duration::from_secs(12),
    ).map_err(|error| io::Error::new(error.kind(), format!("display discovery failed: {error}")))?;
    let value: serde_json::Value = serde_json::from_str(&output)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, format!("display discovery returned invalid data: {error}")))?;
    let normalized = if value.is_array() { value } else { serde_json::Value::Array(vec![value]) };
    serde_json::from_value(normalized)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, format!("display discovery returned invalid data: {error}")))
}

#[cfg(not(target_os = "windows"))]
pub fn displays() -> io::Result<Vec<DisplayInfo>> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "Window Layout is currently available on Windows only"))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscoveredWindow {
    id: String,
    handle: u64,
    process_id: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WindowDiscovery {
    area_x: i32,
    area_y: i32,
    area_width: i32,
    area_height: i32,
    windows: Vec<DiscoveredWindow>,
    missing: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ArrangedSlot {
    pub id: String,
    pub rect: Rect,
}

#[derive(Debug)]
pub struct ArrangementExecution {
    pub result: WindowArrangementResult,
    pub slots: Vec<ArrangedSlot>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WindowMove {
    id: String,
    handle: u64,
    process_id: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

// A VM tab is not a separate top-level window. Never assign one HWND to more
// than one client or move an HWND whose identity cannot be established.
fn validate_discovery(discovery: &WindowDiscovery) -> io::Result<()> {
    if discovery.area_width <= 0 || discovery.area_height <= 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "window discovery returned an invalid display area"));
    }
    let mut ids = HashSet::new();
    let mut handles = HashSet::new();
    for window in &discovery.windows {
        if window.handle == 0 || window.process_id == 0
            || !ids.insert(window.id.as_str()) || !handles.insert(window.handle)
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "window discovery contained a duplicate or invalid window identity"));
        }
    }
    let missing: HashSet<&str> = discovery.missing.iter().map(String::as_str).collect();
    if missing.len() != discovery.missing.len()
        || ["Native", "Virtual-01", "Virtual-02", "Virtual-03"]
            .into_iter()
            .any(|id| ids.contains(id) == missing.contains(id))
        || ids.len() + missing.len() != 4
    {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "window discovery returned an inconsistent client inventory"));
    }
    if discovery.windows.is_empty() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No distinct Minecraft Education or VMware client windows were found"));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn discover_windows(display_index: usize) -> io::Result<WindowDiscovery> {
    use std::{path::Path, time::Duration};
    const SCRIPT: &str = r#"
param([int]$DisplayIndex)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -TypeDefinition @"
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public sealed class VirtualClientsTopLevelWindow {
    public ulong Handle { get; set; }
    public uint ProcessId { get; set; }
    public string Title { get; set; }
    public string ClassName { get; set; }
}
public static class VirtualClientsWindowDiscovery {
    private delegate bool EnumCallback(IntPtr hwnd, IntPtr lparam);
    [DllImport("user32.dll")] private static extern bool EnumWindows(EnumCallback callback, IntPtr lparam);
    [DllImport("user32.dll")] private static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")] private static extern IntPtr GetWindow(IntPtr hwnd, uint command);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] private static extern int GetWindowText(IntPtr hwnd, StringBuilder buffer, int length);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] private static extern int GetClassName(IntPtr hwnd, StringBuilder buffer, int length);
    [DllImport("user32.dll")] private static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
    public static List<VirtualClientsTopLevelWindow> Enumerate() {
        var result = new List<VirtualClientsTopLevelWindow>();
        EnumWindows((hwnd, _) => {
            if (!IsWindowVisible(hwnd) || GetWindow(hwnd, 4) != IntPtr.Zero) return true;
            var title = new StringBuilder(512);
            if (GetWindowText(hwnd, title, title.Capacity) <= 0) return true;
            var windowClass = new StringBuilder(256);
            GetClassName(hwnd, windowClass, windowClass.Capacity);
            uint pid;
            GetWindowThreadProcessId(hwnd, out pid);
            if (pid != 0) result.Add(new VirtualClientsTopLevelWindow {
                Handle = unchecked((ulong)hwnd.ToInt64()), ProcessId = pid,
                Title = title.ToString(), ClassName = windowClass.ToString()
            });
            return true;
        }, IntPtr.Zero);
        return result;
    }
}
"@
$screens = @([System.Windows.Forms.Screen]::AllScreens)
if ($DisplayIndex -lt 0 -or $DisplayIndex -ge $screens.Count) { throw "Selected display is unavailable." }
$area = $screens[$DisplayIndex].WorkingArea
$targets = @('Native', 'Virtual-01', 'Virtual-02', 'Virtual-03')
$deadline = [DateTime]::UtcNow.AddSeconds(5)
$found = @{}
do {
  $candidates = @{}
  foreach ($id in $targets) { $candidates[$id] = @() }
  foreach ($window in [VirtualClientsWindowDiscovery]::Enumerate()) {
    foreach ($id in $targets) {
      $matchesTitle = if ($id -eq 'Native') {
        $window.Title -match '(?i)(?<![a-z0-9])Minecraft Education(?![a-z0-9])'
      } else {
        $window.Title -match "(?i)(?<![a-z0-9])$([regex]::Escape($id))(?![a-z0-9])"
      }
      if (-not $matchesTitle) { continue }
      $process = Get-Process -Id ([int]$window.ProcessId) -ErrorAction SilentlyContinue
      if (-not $process) { continue }
      $name = $process.ProcessName.ToLowerInvariant()
      if ($id -eq 'Native') {
        $nativeExe = $name -like '*minecraft*' -or $name -like '*education*'
        $storeFrame = $name -eq 'applicationframehost' -and $window.ClassName -eq 'ApplicationFrameWindow'
        if (-not ($nativeExe -or $storeFrame)) { continue }
      } elseif ($name -notin @('vmware', 'vmplayer')) {
        continue
      }
      $candidates[$id] += $window
    }
  }
  $found = @{}
  foreach ($id in $targets) {
    $matches = @($candidates[$id] | Sort-Object -Property Handle -Unique)
    if ($matches.Count -gt 1) { throw "Multiple windows match $id; select distinct client windows before arranging." }
    if ($matches.Count -eq 1) { $found[$id] = $matches[0] }
  }
  if ($found.Count -eq 4) { break }
  Start-Sleep -Milliseconds 200
} while ([DateTime]::UtcNow -lt $deadline)
$windows = @()
$missing = @()
foreach ($id in $targets) {
  if ($found.ContainsKey($id)) {
    $win = $found[$id]
    $windows += [PSCustomObject]@{ id=$id; handle=$win.Handle; processId=$win.ProcessId }
  } else {
    $missing += $id
  }
}
[PSCustomObject]@{
  areaX=$area.Left; areaY=$area.Top; areaWidth=$area.Width; areaHeight=$area.Height
  windows=@($windows); missing=@($missing)
} | ConvertTo-Json -Depth 4 -Compress
"#;
    let output = virtual_clients_core::bounded_command_output(
        Path::new("powershell.exe"),
        [
            "-NoLogo", "-NoProfile", "-NonInteractive", "-Command", SCRIPT,
            "-DisplayIndex", &display_index.to_string(),
        ],
        Duration::from_secs(12),
    ).map_err(|error| io::Error::new(error.kind(), format!("window discovery failed: {error}")))?;
    let discovery: WindowDiscovery = serde_json::from_str(&output).map_err(|error| io::Error::new(
        io::ErrorKind::InvalidData,
        format!("window discovery returned invalid data: {error}"),
    ))?;
    validate_discovery(&discovery)?;
    Ok(discovery)
}

#[cfg(target_os = "windows")]
fn apply_window_moves(moves: &[WindowMove]) -> io::Result<()> {
    use std::{path::Path, time::Duration};
    let plan = serde_json::to_string(moves)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))?;
    const SCRIPT: &str = r#"
param([string]$PlanJson)
$ErrorActionPreference = 'Stop'
Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class VirtualClientsWindowNative {
  [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)]
  public static extern int GetWindowText(IntPtr hwnd, StringBuilder title, int length);
  [DllImport("user32.dll", SetLastError=true)]
  public static extern bool MoveWindow(IntPtr hwnd, int x, int y, int width, int height, bool repaint);
}
"@
$moves = @($PlanJson | ConvertFrom-Json)
# Validate every planned HWND before moving any window. HWNDs can be closed
# or reused between the discovery call and placement.
foreach ($move in $moves) {
  $handle = [IntPtr][uint64]$move.handle
  if (-not [VirtualClientsWindowNative]::IsWindow($handle)) { throw "Window for $($move.id) no longer exists." }
  $currentPid = [uint32]0
  [void][VirtualClientsWindowNative]::GetWindowThreadProcessId($handle, [ref]$currentPid)
  if ($currentPid -ne [uint32]$move.processId) { throw "Window ownership changed for $($move.id)." }
  $title = New-Object System.Text.StringBuilder 512
  [void][VirtualClientsWindowNative]::GetWindowText($handle, $title, $title.Capacity)
  $sameClient = if ($move.id -eq 'Native') {
    $title.ToString() -match '(?i)(?<![a-z0-9])Minecraft Education(?![a-z0-9])'
  } else {
    $title.ToString() -match "(?i)(?<![a-z0-9])$([regex]::Escape($move.id))(?![a-z0-9])"
  }
  if (-not $sameClient) { throw "Window title changed for $($move.id)." }
}
foreach ($move in $moves) {
  if (-not [VirtualClientsWindowNative]::MoveWindow(
    [IntPtr][uint64]$move.handle,
    [int]$move.x, [int]$move.y, [int]$move.width, [int]$move.height, $true
  )) { throw "Windows rejected the window placement for $($move.id)." }
}
"#;
    virtual_clients_core::bounded_command_output(
        Path::new("powershell.exe"),
        ["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", SCRIPT, "-PlanJson", &plan],
        Duration::from_secs(12),
    ).map(|_| ()).map_err(|error| {
        io::Error::new(error.kind(), format!("window placement failed: {error}"))
    })
}

#[cfg(target_os = "windows")]
pub fn arrange_with_slots(request: WindowLayoutRequest) -> io::Result<ArrangementExecution> {
    validate_request(&request)?;
    let mut discovery = discover_windows(request.display_index)?;
    if matches!(request.layout, WindowLayout::Focus) {
        let main = request.main_window.as_deref().expect("validated Focus main window");
        discovery.windows.sort_by_key(|window| if window.id == main { 0 } else { 1 });
    }

    let area = Rect {
        x: discovery.area_x,
        y: discovery.area_y,
        width: discovery.area_width,
        height: discovery.area_height,
    };
    let slots = calculate_slots(&request.layout, area, discovery.windows.len());
    let moves: Vec<WindowMove> = discovery.windows.iter().zip(slots.iter()).map(|(window, slot)| WindowMove {
        id: window.id.clone(),
        handle: window.handle,
        process_id: window.process_id,
        x: slot.x,
        y: slot.y,
        width: slot.width,
        height: slot.height,
    }).collect();
    apply_window_moves(&moves)?;

    let arranged: Vec<String> = discovery.windows.iter().map(|window| window.id.clone()).collect();
    let arranged_slots = discovery.windows.iter().zip(slots.iter()).map(|(window, rect)| ArrangedSlot {
        id: window.id.clone(),
        rect: *rect,
    }).collect();
    Ok(ArrangementExecution {
        result: WindowArrangementResult {
            schema: 2,
            layout: request.layout,
            display_index: request.display_index,
            arranged,
            missing: discovery.missing,
            overlay_applied: None,
            overlay_warning: None,
        },
        slots: arranged_slots,
    })
}

#[cfg(not(target_os = "windows"))]
pub fn arrange_with_slots(request: WindowLayoutRequest) -> io::Result<ArrangementExecution> {
    validate_request(&request)?;
    Err(io::Error::new(io::ErrorKind::Unsupported, "Window Layout is currently available on Windows only"))
}

#[cfg(test)]
mod tests {
    use super::{
        calculate_slots, validate_discovery, validate_request, DiscoveredWindow, Rect,
        WindowDiscovery, WindowLayout, WindowLayoutRequest,
    };

    #[test]
    fn layout_geometry_is_adaptive_and_gap_free() {
        let area = Rect { x: 0, y: 0, width: 1200, height: 800 };
        let grid = calculate_slots(&WindowLayout::Grid, area, 3);
        assert_eq!(grid.len(), 3);
        assert_eq!(grid[2], Rect { x: 0, y: 400, width: 1200, height: 400 });

        let focus = calculate_slots(&WindowLayout::Focus, area, 4);
        assert_eq!(focus.len(), 4);
        assert_eq!(focus[0].height, 800);
        assert_eq!(focus.iter().skip(1).map(|slot| slot.height).sum::<i32>(), 800);

        let columns = calculate_slots(&WindowLayout::Columns, area, 4);
        assert_eq!(columns.len(), 4);
        assert_eq!(columns.iter().map(|slot| slot.width).sum::<i32>(), 1200);
    }

    #[test]
    fn discovered_windows_require_unique_client_ids_and_handles() {
        let mut discovery = WindowDiscovery {
            area_x: 0, area_y: 0, area_width: 1200, area_height: 800,
            windows: vec![
                DiscoveredWindow { id: "Native".into(), handle: 11, process_id: 101 },
                DiscoveredWindow { id: "Virtual-01".into(), handle: 22, process_id: 202 },
            ],
            missing: vec!["Virtual-02".into(), "Virtual-03".into()],
        };
        assert!(validate_discovery(&discovery).is_ok());

        discovery.windows[1].handle = 11;
        assert!(validate_discovery(&discovery).is_err());
        discovery.windows[1].handle = 22;

        discovery.windows[1].id = "Native".into();
        assert!(validate_discovery(&discovery).is_err());
        discovery.windows[1].id = "Virtual-01".into();

        discovery.windows[1].process_id = 0;
        assert!(validate_discovery(&discovery).is_err());
        discovery.windows[1].process_id = 202;

        discovery.missing = vec!["Virtual-01".into(), "Virtual-03".into()];
        assert!(validate_discovery(&discovery).is_err());
    }

    #[test]
    fn focus_requires_known_main_window() {
        assert!(validate_request(&WindowLayoutRequest {
            layout: WindowLayout::Focus,
            display_index: 0,
            main_window: Some("Native".into()),
        }).is_ok());
        assert!(validate_request(&WindowLayoutRequest {
            layout: WindowLayout::Focus,
            display_index: 0,
            main_window: None,
        }).is_err());
        assert!(validate_request(&WindowLayoutRequest {
            layout: WindowLayout::Grid,
            display_index: 0,
            main_window: Some("Unknown".into()),
        }).is_err());
    }
}
