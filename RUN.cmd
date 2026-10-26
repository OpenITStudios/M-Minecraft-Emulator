@echo off
setlocal
if not exist "%~dp0App\lazy-tester.exe" (
  echo Lazy Tester is not prepared in the App folder yet.
  echo Build the local candidate with npm run package:windows.
  pause
  exit /b 1
)
start "" /D "%~dp0App" "%~dp0App\lazy-tester.exe"
