@echo off
setlocal
if not exist "%~dp0Lazy Tester.exe" (
  echo The root Lazy Tester.exe launcher is not prepared yet.
  echo Build the local candidate with DEV.cmd package.
  pause
  exit /b 1
)
start "" /D "%~dp0" "%~dp0Lazy Tester.exe"
