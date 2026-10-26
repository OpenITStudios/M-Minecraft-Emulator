param(
  [string]$VirtualClients = 'virtual-clients.exe',
  [string]$OutputDirectory = '',
  [switch]$VerifyIdentities
)

$ErrorActionPreference = 'Stop'

if (!$OutputDirectory) {
  $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
  $OutputDirectory = Join-Path $env:TEMP "M-Bedrock-VirtualClients-Acceptance-$stamp"
}
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null

function Invoke-VirtualClientsJson {
  param(
    [Parameter(Mandatory = $true)] [string]$Name,
    [Parameter(Mandatory = $true)] [string[]]$Arguments
  )

  $stdout = Join-Path $OutputDirectory "$Name.json"
  $stderr = Join-Path $OutputDirectory "$Name.stderr.txt"
  $process = Start-Process -FilePath $VirtualClients -ArgumentList $Arguments -NoNewWindow -Wait -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
  if ($process.ExitCode -ne 0) {
    $detail = if (Test-Path -LiteralPath $stderr) { (Get-Content -LiteralPath $stderr -Raw).Trim() } else { '' }
    throw "$Name failed with exit code $($process.ExitCode): $detail"
  }

  try {
    $envelope = Get-Content -LiteralPath $stdout -Raw | ConvertFrom-Json -ErrorAction Stop
  } catch {
    throw "$Name did not return valid JSON."
  }
  if ($null -eq $envelope -or $envelope.schema -ne 1 -or $null -eq $envelope.data) {
    throw "$Name returned an invalid CLI success envelope; expected schema 1 and non-null data."
  }
  return $envelope
}

$doctor = (Invoke-VirtualClientsJson -Name 'doctor' -Arguments @('doctor')).data
$status = (Invoke-VirtualClientsJson -Name 'status' -Arguments @('status')).data

$identityProof = $null
$diagnostics = (Invoke-VirtualClientsJson -Name 'diagnostics' -Arguments @('diagnostics')).data
# resources 3 is only valid after three clients are provisioned. Preserve an
# early HOST/baseline report instead of failing the entire campaign.
$resources = $null

$virtuals = @($status.clients | Where-Object { $_.native -eq $false })
if ($virtuals.Count -ne 3) {
  throw "Expected exactly three Virtual client status slots, found $($virtuals.Count)."
}
$provisionedCount = @($virtuals | Where-Object { $_.state -ne 'NOT_PROVISIONED' -and $_.state -ne 'ERROR' }).Count
if ($VerifyIdentities) {
  $runningCount = @($virtuals | Where-Object { $_.state -eq 'RUNNING' }).Count
  if ($provisionedCount -ne 3 -or $runningCount -ne 3) {
    throw 'Identity verification requires all three Virtual clients to be provisioned and RUNNING. Do not run it during HOST/baseline collection.'
  }
  $identityProof = (Invoke-VirtualClientsJson -Name 'verify-identities' -Arguments @('verify-identities')).data
  $doctor = (Invoke-VirtualClientsJson -Name 'doctor-after-verify' -Arguments @('doctor')).data
  $status = (Invoke-VirtualClientsJson -Name 'status-after-verify' -Arguments @('status')).data
  $virtuals = @($status.clients | Where-Object { $_.native -eq $false })
}
if ($provisionedCount -eq 3) {
  $resources = (Invoke-VirtualClientsJson -Name 'resources-3' -Arguments @('resources', '3')).data
}

if ($VerifyIdentities) {
  foreach ($virtual in $virtuals) {
    if ([string]$virtual.vmIdentity -ne 'UNIQUE' -or [string]$virtual.windowsIdentity -ne 'UNIQUE') {
      throw "$($virtual.id) did not remain identity-unique after verify-identities."
    }
  }
}

$summary = [ordered]@{
  schema = 1
  capturedAt = (Get-Date).ToString('o')
  appVersion = $diagnostics.appVersion
  platform = $doctor.platform
  provider = $doctor.provider
  baseState = $doctor.baseState
  nextSetupAction = [string]$doctor.nextSetupAction
  provisionedVirtuals = [int]$provisionedCount
  resourcesCollected = ($null -ne $resources)
  runtimeProfileParity = [string]$doctor.runtimeProfile.parity
  pressure = [string]$status.pressure.level
  virtuals = @(
    $virtuals | ForEach-Object {
      [ordered]@{
        id = $_.id
        state = [string]$_.state
        readySnapshot = $_.readySnapshot
        guestToolsReady = $_.guestToolsReady
        guestAgentReady = $_.guestAgentReady
        guestAgentVersion = $_.guestAgentVersion
        connectionHealth = $_.connectionHealth
        interactiveLauncherReady = $_.interactiveLauncherReady
        minecraftRunning = $_.minecraftRunning
        minecraftVersion = $_.minecraftVersion
        lineageParity = [string]$_.lineageParity
        versionParity = [string]$_.versionParity
        vmIdentity = [string]$_.vmIdentity
        windowsIdentity = [string]$_.windowsIdentity
      }
    }
  )
  virtualHardware = @(
    $diagnostics.virtualHardware | ForEach-Object {
      [ordered]@{
        id = $_.id
        networkMode = $_.networkMode
        graphics3dEnabled = $_.graphics3dEnabled
      }
    }
  )
  host = [ordered]@{
    logicalCpus = $diagnostics.host.logicalCpus
    totalMemoryMb = $diagnostics.host.totalMemoryMb
    availableMemoryMb = $diagnostics.host.availableMemoryMb
    hypervisorPresent = $diagnostics.host.hypervisorPresent
    vbsStatus = $diagnostics.host.vbsStatus
    graphics = @($diagnostics.host.graphics)
  }
  resources = [ordered]@{
    requestedVirtualClients = if ($null -ne $resources) { $resources.requestedVirtualClients } else { $null }
    runningVirtualClients = if ($null -ne $resources) { $resources.runningVirtualClients } else { $null }
    suspendedVirtualClients = if ($null -ne $resources) { $resources.suspendedVirtualClients } else { $null }
    stoppedVirtualClients = if ($null -ne $resources) { $resources.stoppedVirtualClients } else { $null }
    virtualMemoryLimitMb = if ($null -ne $resources) { $resources.virtualMemoryLimitMb } else { $null }
    observedWorkingSetMb = if ($null -ne $resources) { $resources.observedWorkingSetMb } else { $null }
    observedWorkingSetInstances = if ($null -ne $resources) { $resources.observedWorkingSetInstances } else { $null }
  }
  identityVerificationRequested = [bool]$VerifyIdentities
  identityVerificationCaptured = ($null -ne $identityProof)
}

$summaryPath = Join-Path $OutputDirectory 'summary.json'
$summary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $summaryPath -Encoding utf8

[ordered]@{
  outputDirectory = $OutputDirectory
  summary = $summaryPath
  files = @(Get-ChildItem -LiteralPath $OutputDirectory -File | Sort-Object Name | Select-Object -ExpandProperty Name)
} | ConvertTo-Json -Depth 5
