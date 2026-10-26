param(
    [Parameter(Mandatory=$true)][string]$Root,
    [switch]$PlanOnly
)
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSHOME 'Modules/Microsoft.PowerShell.Utility') -ErrorAction Stop
$checkout = (Resolve-Path -LiteralPath $Root).Path
$appFolder = Join-Path $checkout 'App'
$executable = Join-Path $appFolder 'lazy-tester.exe'
$launcher = Join-Path $checkout 'Lazy Tester.exe'
foreach ($itemPath in @($checkout,$appFolder,$executable)) {
    $item = Get-Item -LiteralPath $itemPath
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'Shortcut target must not be a junction or symlink.' }
}
$receipt = Get-Content -LiteralPath (Join-Path $appFolder 'build-info.json') -Raw | ConvertFrom-Json
if ($receipt.sourceCommit -notmatch '^[a-f0-9]{40}$') { throw 'App build receipt is invalid.' }
$entry = @($receipt.files | Where-Object { $_.name -eq 'lazy-tester.exe' })
$algorithm = [Security.Cryptography.SHA256]::Create()
$stream = [IO.File]::OpenRead($executable)
try { $executableHash = [BitConverter]::ToString($algorithm.ComputeHash($stream)).Replace('-','') }
finally { $stream.Dispose(); $algorithm.Dispose() }
if ($entry.Count -ne 1 -or $executableHash -ine $entry[0].sha256) { throw 'App executable does not match its build receipt.' }
if (-not $receipt.launcher -or $receipt.launcher.name -ne 'Lazy Tester.exe' -or (Get-FileHash -LiteralPath $launcher -Algorithm SHA256).Hash -ine $receipt.launcher.sha256) { throw 'Root launcher does not match the App receipt.' }
$desktop = [Environment]::GetFolderPath('DesktopDirectory')
$programs = [Environment]::GetFolderPath('Programs')
if (-not $desktop -or -not $programs) { throw 'Windows Desktop or Start Menu location is unavailable.' }
$menu = Join-Path $programs 'Lazy Tester'
$paths = @((Join-Path $desktop 'Lazy Tester.lnk'),(Join-Path $menu 'Lazy Tester.lnk'))
$shell = New-Object -ComObject WScript.Shell
try {
    # Check both destinations before changing either; preserve unrelated user shortcuts.
    foreach ($path in $paths) {
        if (Test-Path -LiteralPath $path) {
            $item = Get-Item -LiteralPath $path
            if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw "Unsafe shortcut destination: $path" }
            $existing = $shell.CreateShortcut($path)
            if ($existing.TargetPath -ine $executable -and $existing.TargetPath -ine $launcher) { throw "An unrelated shortcut already exists: $path" }
        }
    }
    if (-not $PlanOnly) {
        if (-not (Test-Path -LiteralPath $menu)) { $null = New-Item -ItemType Directory -Path $menu }
        foreach ($path in $paths) {
            $shortcut = $shell.CreateShortcut($path)
            $shortcut.TargetPath = $launcher
            $shortcut.WorkingDirectory = $checkout
            $shortcut.Arguments = ''
            $shortcut.IconLocation = "$launcher,0"
            $shortcut.Description = 'Lazy Tester - Minecraft Education'
            $shortcut.Save()
            $check = $shell.CreateShortcut($path)
            if ($check.TargetPath -ine $launcher -or $check.WorkingDirectory -ine $checkout -or $check.Arguments -or $check.IconLocation -ine "$launcher,0") { throw "Shortcut verification failed: $path" }
        }
    }
    [PSCustomObject]@{target=$launcher;workingDirectory=$checkout;shortcuts=$paths;created=(-not $PlanOnly)} | ConvertTo-Json
} finally { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($shell) }
