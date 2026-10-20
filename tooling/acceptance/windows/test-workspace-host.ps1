# Synthetic cross-process probe only; never targets Minecraft or existing windows.
param([switch]$IncludeUnsafeDirectCrash)
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../..'))
$workspace = Join-Path $root 'workspace'
if ((Get-Item -LiteralPath $workspace -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Workspace must not be a reparse point' }
$output = Join-Path $workspace ('workspace-host-' + [Guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($output) | Out-Null
$compiler = Join-Path $env:WINDIR 'Microsoft.NET/Framework64/v4.0.30319/csc.exe'
if (-not (Test-Path -LiteralPath $compiler -PathType Leaf)) { throw 'Windows x64 .NET Framework compiler unavailable' }
$binary = Join-Path $output 'WorkspaceHostFixture.exe'
& $compiler /nologo /target:winexe /platform:x64 /reference:System.Windows.Forms.dll /reference:System.Drawing.dll "/out:$binary" (Join-Path $PSScriptRoot 'workspace-host-fixture.cs')
if ($LASTEXITCODE -ne 0) { throw 'Workspace fixture compilation failed' }
$modes = @('normal', 'partial-failure', 'protected-manager-crash')
if ($IncludeUnsafeDirectCrash) { $modes += 'crash-recovery' }
foreach ($mode in $modes) {
    $result = Join-Path $output ($mode + '.json')
    $start = New-Object Diagnostics.ProcessStartInfo
    $start.FileName = $binary
    $start.Arguments = $mode + ' "' + $result + '"'
    $start.UseShellExecute = $false
    $process = [Diagnostics.Process]::Start($start)
    try {
        if (-not $process.WaitForExit(15000)) { throw 'Synthetic workspace probe timed out; evidence retained' }
        if (-not (Test-Path -LiteralPath $result -PathType Leaf)) { throw "Workspace probe produced no verdict: $mode; evidence retained in $output" }
        $proof = Get-Content -LiteralPath $result -Raw | ConvertFrom-Json
        Write-Output ("{0}: {1}" -f $mode, ($proof | ConvertTo-Json -Compress))
        if ($process.ExitCode -ne 0) { throw "Workspace recovery FAILED: $mode; evidence retained in $output" }
        if ($proof.result -ne 'PASS' -or -not $proof.restored -or -not $proof.normalClose) { throw 'Workspace proof incomplete' }
        if ($mode -eq 'crash-recovery') {
            if (-not $proof.survivedHostCrash) { throw 'Synthetic client did not survive host crash' }
        } elseif ($mode -eq 'protected-manager-crash') {
            if (-not $proof.survivedManagerCrash) { throw 'Synthetic client did not survive UI crash' }
        } elseif ($proof.partialFailureObserved -ne ($mode -eq 'partial-failure')) { throw 'Partial-failure evidence mismatch' }
    } finally {
        # Only the exact synthetic host started above. Never targets external windows.
        # Its self-created synthetic child has its own 20-second lifetime.
        if (-not $process.HasExited) {
            $process.CloseMainWindow() | Out-Null
            if (-not $process.WaitForExit(2000)) { $process.Kill(); $process.WaitForExit() }
        }
        $process.Dispose()
    }
}
Write-Output 'Synthetic windows only: game rendering/input, mixed DPI and presentation-owner failure remain NOT_EXECUTED by this protected case. Direct manager parenting is unsafe; -IncludeUnsafeDirectCrash retains its failing recovery verdict.'
