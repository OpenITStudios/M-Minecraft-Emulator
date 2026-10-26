; Native installer only. No legacy hooks, process termination, profile migration
; or recursive data cleanup. Keep the same controller gate across build identities.
; Set a visible default after Tauri's .onInit, before showing the directory page.
; Existing registered installations and explicit /D locations remain unchanged.
!define MUI_CUSTOMFUNCTION_GUIINIT LazyTesterVisibleInstallDefault
Function LazyTesterVisibleInstallDefault
  Push $0
  IfFileExists "$INSTDIR\data-location.json" lazy_tester_visible_default_done 0
  ReadRegStr $0 HKCU "Software\MIVUBI-STD\Lazy Tester" ""
  ${If} $0 == ""
  ${AndIf} $INSTDIR == "$LOCALAPPDATA\Lazy Tester"
    StrCpy $INSTDIR "$PROFILE\Lazy Tester\App"
  ${EndIf}
  lazy_tester_visible_default_done:
  Pop $0
FunctionEnd

!macro LAZY_TESTER_APP_LAYOUT
  Push $0
  ; An upgrade may have removed the old executable before this hook runs.
  ; A retained location pointer still identifies an existing flat installation.
  IfFileExists "$INSTDIR\lazy-tester.exe" lazy_tester_layout_done 0
  IfFileExists "$INSTDIR\data-location.json" lazy_tester_layout_done 0
  ReadRegStr $0 HKCU "Software\MIVUBI-STD\Lazy Tester" ""
  ${If} $0 == ""
  ${AndIf} $INSTDIR == "$LOCALAPPDATA\Lazy Tester"
    StrCpy $INSTDIR "$PROFILE\Lazy Tester\App"
  ${EndIf}
  ; Preserve old flat installs. New selected roots place binaries under App.
  ${GetFileName} "$INSTDIR" $0
  ${If} $0 != "App"
    StrCpy $INSTDIR "$INSTDIR\App"
  ${EndIf}
  lazy_tester_layout_done:
  SetOutPath $INSTDIR
  Pop $0
!macroend

!macro LAZY_TESTER_REQUIRE_IDLE
  Push $0
  Push $1
  System::Call 'kernel32::OpenMutexW(i 0x00100000, i 0, w "Local\MivubiEducationStartupObservation") p.r0'
  System::Call 'kernel32::GetLastError() i.r1'
  ${If} $0 P<> 0
    System::Call 'kernel32::CloseHandle(p r0)'
    Pop $1
    Pop $0
    MessageBox MB_OK|MB_ICONEXCLAMATION "Save your worlds and close all Minecraft sessions normally before installing or removing Lazy Tester."
    Abort
  ${ElseIf} $1 != 2
    Pop $1
    Pop $0
    MessageBox MB_OK|MB_ICONEXCLAMATION "Lazy Tester could not verify that its session controller is stopped. Existing sessions were preserved."
    Abort
  ${EndIf}
  Pop $1
  Pop $0
!macroend

!macro NSIS_HOOK_PREINSTALL
  !insertmacro LAZY_TESTER_REQUIRE_IDLE
  !insertmacro LAZY_TESTER_APP_LAYOUT
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro LAZY_TESTER_REQUIRE_IDLE
  ; Preserve application data even if the template's delete-data option was checked.
  StrCpy $DeleteAppDataCheckboxState 0
!macroend

