@echo off
setlocal
set "PROJECT_ROOT=%~dp0"
if /I "%~1"=="bootstrap" (
  powershell.exe -NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%PROJECT_ROOT%tooling\windows-toolchain\bootstrap.ps1"
  exit /b %ERRORLEVEL%
)
set "LOCAL_PWSH=%PROJECT_ROOT%workspace\toolchains\powershell\pwsh.exe"
if not exist "%LOCAL_PWSH%" (
  echo Portable developer environment is missing. Run DEV.cmd bootstrap first.
  exit /b 1
)
"%LOCAL_PWSH%" -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%PROJECT_ROOT%tooling\windows-toolchain\dev.ps1" %*
exit /b %ERRORLEVEL%
