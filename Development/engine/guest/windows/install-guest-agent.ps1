param(
  [Parameter(Mandatory = $true)]
  [string]$AgentSource
)

$ErrorActionPreference = 'Stop'

if (!(Test-Path -LiteralPath $AgentSource -PathType Leaf)) {
  throw "Guest Agent binary not found: $AgentSource"
}

$vmtoolsd = Join-Path $env:ProgramFiles 'VMware\VMware Tools\vmtoolsd.exe'
if (!(Test-Path -LiteralPath $vmtoolsd -PathType Leaf)) {
  throw "VMware Tools is required before installing the Guest Agent: $vmtoolsd"
}

$root = Join-Path $env:ProgramData 'M-Bedrock\VirtualClients'
$agent = Join-Path $root 'virtual-guest-agent.exe'
$rootItem = Get-Item -LiteralPath $root -Force -ErrorAction SilentlyContinue
if ($rootItem -and ($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
  throw 'Guest Agent installation root must not be a reparse point.'
}
$taskName = 'M-Bedrock Virtual Guest Agent'
New-Item -ItemType Directory -Path $root -Force | Out-Null

# Repair/upgrade is idempotent. Stop the existing SYSTEM task before replacing
# its executable so a running process cannot keep an old binary alive.
$existingTask = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
if ($existingTask) {
  Stop-ScheduledTask -InputObject $existingTask -ErrorAction SilentlyContinue
}
Get-Process -Name 'virtual-guest-agent' -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Copy-Item -LiteralPath $AgentSource -Destination $agent -Force

$action = New-ScheduledTaskAction -Execute $agent
$trigger = New-ScheduledTaskTrigger -AtStartup
$principal = New-ScheduledTaskPrincipal -UserId 'SYSTEM' -LogonType ServiceAccount -RunLevel Highest
$settings = New-ScheduledTaskSettingsSet -StartWhenAvailable -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1) -ExecutionTimeLimit ([TimeSpan]::Zero)
Register-ScheduledTask -TaskName $taskName -Action $action -Trigger $trigger -Principal $principal -Settings $settings -Force | Out-Null
Start-ScheduledTask -TaskName $taskName

$publicDesktop = [Environment]::GetFolderPath('CommonDesktopDirectory')
$interactiveSetup = Join-Path $publicDesktop 'Enable Virtual Clients Launcher.cmd'
$batchAgent = $agent.Replace('%', '%%')
@"
@echo off
call "$batchAgent" --register-interactive-launcher
if errorlevel 1 (
  echo.
  echo Virtual Clients launcher setup failed.
  pause
  exit /b 1
)
echo.
echo Virtual Clients launcher is enabled for this Windows user.
echo You can continue account setup now.
pause
"@ | Set-Content -LiteralPath $interactiveSetup -Encoding ASCII

$ruleName = 'M-Bedrock Virtual Guest Agent'
$rule = Get-NetFirewallRule -DisplayName $ruleName -ErrorAction SilentlyContinue
if ($rule) {
  $rule | Remove-NetFirewallRule
}
New-NetFirewallRule -DisplayName $ruleName -Direction Inbound -Action Allow -Program $agent -Protocol TCP -LocalPort 47831 -RemoteAddress LocalSubnet -Profile Any | Out-Null
$installedRule = Get-NetFirewallRule -DisplayName $ruleName -ErrorAction Stop
$portFilter = $installedRule | Get-NetFirewallPortFilter
$addressFilter = $installedRule | Get-NetFirewallAddressFilter
$appFilter = $installedRule | Get-NetFirewallApplicationFilter
if ($installedRule.Direction -ne 'Inbound' -or $installedRule.Action -ne 'Allow' -or $portFilter.Protocol -ne 'TCP' -or $portFilter.LocalPort -ne '47831' -or $addressFilter.RemoteAddress -ne 'LocalSubnet' -or $appFilter.Program -ne $agent) {
  throw 'Guest Agent firewall rule does not match the bounded transport policy.'
}

$installed = Get-ScheduledTask -TaskName $taskName -ErrorAction Stop
$installedAction = @($installed.Actions)
if ($installedAction.Count -ne 1 -or $installedAction[0].Execute -ne $agent -or ![string]::IsNullOrWhiteSpace($installedAction[0].Arguments)) {
  throw 'Guest Agent scheduled task does not point to the exact installed binary.'
}
if ($installed.Principal.UserId -ne 'SYSTEM' -or $installed.Principal.RunLevel -ne 'Highest') {
  throw 'Guest Agent scheduled task principal does not match the SYSTEM policy.'
}
if ($installed.State -eq 'Disabled') {
  throw 'Guest Agent scheduled task was registered but is disabled.'
}
if (!(Test-Path -LiteralPath $agent -PathType Leaf)) {
  throw 'Guest Agent executable is missing after installation.'
}

