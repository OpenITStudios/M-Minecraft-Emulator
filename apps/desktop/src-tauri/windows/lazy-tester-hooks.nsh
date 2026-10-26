; Native installer only. No legacy hooks, process termination, profile migration
; or recursive data cleanup. Keep the same controller gate across build identities.
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
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro LAZY_TESTER_REQUIRE_IDLE
  ; Preserve application data even if the template's delete-data option was checked.
  StrCpy $DeleteAppDataCheckboxState 0
!macroend

