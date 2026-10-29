@echo off
setlocal
set "PROJECT_ROOT=%~dp0"
if "%~1"=="" goto help
if /I "%~1"=="help" goto help
if /I "%~1"=="bootstrap" (
  powershell.exe -NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%PROJECT_ROOT%Development\tooling\windows-toolchain\bootstrap.ps1"
  exit /b %ERRORLEVEL%
)
set "LOCAL_PWSH=%PROJECT_ROOT%workspace\toolchains\powershell\pwsh.exe"
if not exist "%LOCAL_PWSH%" (
  echo Portable developer environment is missing. Run DEV.cmd bootstrap first.
  exit /b 1
)
"%LOCAL_PWSH%" -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%PROJECT_ROOT%Development\tooling\windows-toolchain\dev.ps1" %*
exit /b %ERRORLEVEL%

:help
echo M-Minecraft-Emulator ^(Windows x64, one local checkout^)
echo   DEV.cmd bootstrap   - download portable tools into workspace
echo   DEV.cmd doctor      - show and verify all local toolchain paths
echo   DEV.cmd storage report - read-only developer disk inventory
echo   DEV.cmd storage explain - identify largest measured categories
echo   DEV.cmd preflight   - full source checks, no game/installer
echo   DEV.cmd dev         - launch the desktop in developer mode
echo All project downloads and toolchains belong inside workspace.
exit /b 0
