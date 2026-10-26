param(
  [Parameter(Mandatory = $true)]
  [ValidateSet('CLIENT_FEASIBILITY','HOST','SINGLE_VM','GUEST','MINECRAFT','ACCOUNT','THREE_VMS','FOUR_MINECRAFT','FOUR_ACCOUNTS','MULTIPLAYER','RECOVERY','SOAK')]
  [string]$Stage,
  [Parameter(Mandatory = $true)]
  [ValidateSet('NOT_RUN','OBSERVED','PASS','FAIL','BLOCKED')]
  [string]$Result,
  [string]$Evidence,
  [string]$Note
)

$ErrorActionPreference = 'Stop'

if ($Result -eq 'PASS' -and [string]::IsNullOrWhiteSpace($Evidence)) {
  throw 'PASS requires an evidence reference. Do not promote an unproven stage.'
}
if ($Result -eq 'NOT_RUN' -and ![string]::IsNullOrWhiteSpace($Evidence)) {
  throw 'NOT_RUN cannot carry runtime evidence.'
}

$record = [ordered]@{
  schema = 1
  recordedAtUtc = [DateTime]::UtcNow.ToString('o')
  stage = $Stage
  result = $Result
  evidence = if ([string]::IsNullOrWhiteSpace($Evidence)) { $null } else { $Evidence }
  note = if ([string]::IsNullOrWhiteSpace($Note)) { $null } else { $Note }
}

$record | ConvertTo-Json -Depth 4
