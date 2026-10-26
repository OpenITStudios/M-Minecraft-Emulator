$ErrorActionPreference = 'Stop'

$taskName = 'M-Bedrock Virtual Guest Agent'
$task = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
if ($task) {
  Stop-ScheduledTask -InputObject $task -ErrorAction SilentlyContinue
  Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue
}
Get-Process -Name 'virtual-guest-agent' -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Remove-NetFirewallRule -DisplayName 'M-Bedrock Virtual Guest Agent' -ErrorAction SilentlyContinue

$publicDesktop = [Environment]::GetFolderPath('CommonDesktopDirectory')
$interactiveSetup = Join-Path $publicDesktop 'Enable Virtual Clients Launcher.cmd'
Remove-Item -LiteralPath $interactiveSetup -Force -ErrorAction SilentlyContinue

# Interactive launcher registration is per Windows user and lives in that
# user's Startup folder. Remove registrations for local profiles without
# deleting any account data.
Get-ChildItem -LiteralPath (Join-Path $env:SystemDrive 'Users') -Directory -ErrorAction SilentlyContinue | ForEach-Object {
  $launcher = Join-Path $_.FullName 'AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup\M-Bedrock Virtual Interactive Launcher.cmd'
  Remove-Item -LiteralPath $launcher -Force -ErrorAction SilentlyContinue
}

$root = Join-Path $env:ProgramData 'M-Bedrock\VirtualClients'
$rootItem = Get-Item -LiteralPath $root -Force -ErrorAction SilentlyContinue
if ($rootItem -and ($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
  throw 'Guest Agent uninstall refused a redirected installation directory.'
}
# Only the Guest Agent executable is owned by this uninstaller.
# Preserve Base finalization tools and other guest state in the same folder.
$agent = Join-Path $root 'virtual-guest-agent.exe'
Remove-Item -LiteralPath $agent -Force -ErrorAction SilentlyContinue
