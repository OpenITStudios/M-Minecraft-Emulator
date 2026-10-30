@echo off
setlocal
set "PROJECT_ROOT=%~dp0"
if "%~1"=="" goto help
if /I "%~1"=="help" goto help
if /I "%~1"=="bootstrap" (
  powershell.exe -NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%PROJECT_ROOT%Development\tooling\windows-toolchain\bootstrap.ps1"
  exit /b %ERRORLEVEL%
)
rem AI-facing control operations use only the reviewed workspace-local Node.
if /I "%~1"=="inspect" goto controller
if /I "%~1"=="catalog" goto controller
if /I "%~1"=="plan" goto controller
if /I "%~1"=="run" goto controller
rem Existing manual developer routes remain available, unchanged.
set "LOCAL_PWSH=%PROJECT_ROOT%workspace\toolchains\powershell\pwsh.exe"
if not exist "%LOCAL_PWSH%" (
  echo Portable developer environment is missing. Run DEV.cmd bootstrap first.
  exit /b 1
)
"%LOCAL_PWSH%" -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%PROJECT_ROOT%Development\tooling\windows-toolchain\dev.ps1" %*
exit /b %ERRORLEVEL%

:controller
set "LOCAL_NODE=%PROJECT_ROOT%workspace\toolchains\node\node.exe"
if not exist "%LOCAL_NODE%" (
  echo Portable developer Node is missing. Run DEV.cmd bootstrap first.
  exit /b 1
)
"%LOCAL_NODE%" "%PROJECT_ROOT%Development\tooling\developer\controller.mjs" %*
exit /b %ERRORLEVEL%

:help
echo M-Minecraft-Emulator ^(Windows x64, single developer entry^)
echo   DEV.cmd bootstrap                      - reviewed opt-in toolchain setup
echo   DEV.cmd inspect --json                - source state, changes, tool availability
echo   DEV.cmd catalog --json                - safe, versioned task capabilities
echo   DEV.cmd plan --json                   - recommended validation from changes
echo   DEV.cmd run ^<task-id^> --expect-sha ^<sha^> - execute registered task only
echo   Additional legacy developer routes remain supported.
echo   No UI, cleanup, CI, or game/session automation is started by this launcher.
exit /b 0
