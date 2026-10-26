param([Parameter(Mandatory=$true)][string]$Root, [switch]$KeepOpen)
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSHOME 'Modules/Microsoft.PowerShell.Utility') -ErrorAction Stop
Import-Module (Join-Path $PSHOME 'Modules/CimCmdlets') -ErrorAction Stop
$checkout = (Resolve-Path -LiteralPath $Root).Path
$appFolder = Join-Path $checkout 'App'
$executable = Join-Path $appFolder 'lazy-tester.exe'
$launcher = Join-Path $checkout 'Lazy Tester.exe'
$receipt = Get-Content -LiteralPath (Join-Path $appFolder 'build-info.json') -Raw | ConvertFrom-Json
$entry = @($receipt.files | Where-Object name -eq 'lazy-tester.exe')
if ($entry.Count -ne 1 -or (Get-FileHash -LiteralPath $executable -Algorithm SHA256).Hash -ine $entry[0].sha256) {
    throw 'App does not match its build receipt.'
}
if (@(Get-Process -Name lazy-tester -ErrorAction SilentlyContinue).Count) {
    throw 'Close Lazy Tester normally before verifying Desktop startup.'
}
$shortcutPath = Join-Path ([Environment]::GetFolderPath('DesktopDirectory')) 'Lazy Tester.lnk'
$links = New-Object -ComObject WScript.Shell
$link = $links.CreateShortcut($shortcutPath)
if ($link.TargetPath -ine $launcher -or $link.WorkingDirectory -ine $checkout -or $link.Arguments) {
    throw 'Desktop shortcut does not target this App directly.'
}
[void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($links)
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$shell = New-Object -ComObject Shell.Application
$handle = 0
# The desktop's Application dispatches from Explorer. Shell.Application alone can
# start the executable in the build tool's virtualized registry/AppData context.
$desktop = $shell.Windows().FindWindowSW(0, 0, 8, [ref]$handle, 1)
if (-not $desktop) { throw 'Interactive Windows Explorer desktop is unavailable.' }
$explorer = @(Get-Process -Name explorer | Where-Object SessionId -eq (Get-Process -Id $PID).SessionId)
$manager = $null
$observation = $null
try {
    $desktop.Document.Application.ShellExecute($shortcutPath, '', '', 'open', 1)
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    while ([DateTime]::UtcNow -lt $deadline) {
        $candidates = @(Get-Process -Name lazy-tester -ErrorAction SilentlyContinue | Where-Object { $_.Path -ieq $executable -and $_.MainWindowHandle -ne 0 })
        if ($candidates.Count -gt 1) { throw 'Ambiguous manager processes; no process was closed.' }
        if ($candidates.Count -eq 1) {
            $manager = $candidates[0]
            $process = Get-CimInstance Win32_Process -Filter "ProcessId=$($manager.Id)"
            $parent = Get-CimInstance Win32_Process -Filter "ProcessId=$($process.ParentProcessId)"
            if ($parent.ExecutablePath -ine $launcher -or $parent.ParentProcessId -notin $explorer.Id) { throw 'Root launcher was not launched by Windows Explorer.' }
            if (-not $receipt.launcher -or (Get-FileHash -LiteralPath $launcher -Algorithm SHA256).Hash -ine $receipt.launcher.sha256) { throw 'Root launcher does not match the receipt.' }
            $window = [System.Windows.Automation.AutomationElement]::FromHandle($manager.MainWindowHandle)
            $elements = @($window.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition))
            $names = @($elements | ForEach-Object { $_.Current.Name })
            $cards = @($elements | Where-Object { $_.Current.Name -match '^Launch Minecraft - [1-5]$' -and $_.Current.ControlType -eq [System.Windows.Automation.ControlType]::Button })
            $enabled = @($cards | Where-Object { $_.Current.IsEnabled }).Count
            $issue = @($names | Where-Object { $_ -match 'saved registry store cannot be opened|connection could not be|launcher connection was found' })
            $cacheInsideData = $false
            $cache = [IO.Path]::GetFullPath((Join-Path $checkout 'Data/Cache/WebView'))
            foreach ($browser in @(Get-CimInstance Win32_Process -Filter "ParentProcessId=$($manager.Id) AND Name='msedgewebview2.exe'")) {
                if ($browser.CommandLine -match '--user-data-dir=(?:"([^"]+)"|([^\s]+))') {
                    $folder = if ($Matches[1]) { $Matches[1] } else { $Matches[2] }
                    $folder = [IO.Path]::GetFullPath($folder)
                    $cacheInsideData = $folder -ieq $cache -or $folder.StartsWith(($cache+'\'),[StringComparison]::OrdinalIgnoreCase)
                    if ($cacheInsideData) { break }
                }
            }
            $observation = [PSCustomObject]@{event='desktop-startup-observation';sourceCommit=$receipt.sourceCommit;processId=$manager.Id;parentProcessId=$process.ParentProcessId;explorerParent=$true;connected=('Connected' -in $names);cards=$cards.Count;enabled=$enabled;issue=($issue -join ' ');cacheInsideData=$cacheInsideData}
            if ($observation.connected -and $cards.Count -eq 5 -and $enabled -eq 5 -and -not $observation.issue -and $cacheInsideData) { break }
        }
        Start-Sleep -Milliseconds 250
    }
    if (-not $observation -or -not $observation.connected -or $observation.cards -ne 5 -or $observation.enabled -ne 5 -or $observation.issue -or -not $observation.cacheInsideData) {
        if ($observation) { $observation | ConvertTo-Json -Compress | Write-Output }
        throw 'Actual Desktop startup failed. Build-tool or hidden WebView success cannot replace this check.'
    }
    $observation | ConvertTo-Json -Compress | Write-Output
} finally {
    # Only this verifier's Explorer-launched manager, and only when no game exists.
    if ($manager -and -not $KeepOpen -and -not $manager.HasExited) {
        $games = @(Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.ProcessName -match '^Minecraft' -and $_.ProcessName -notmatch 'Updater' })
        if (-not $games.Count) {
            [void]$manager.CloseMainWindow()
            if (-not $manager.WaitForExit(8000)) { Write-Warning 'Verified manager did not close normally; it was left running.' }
        }
    }
    [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($shell)
}
