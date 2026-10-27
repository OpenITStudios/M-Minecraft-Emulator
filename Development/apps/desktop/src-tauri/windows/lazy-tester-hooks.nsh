; Native installer only. No legacy hooks, process termination, profile migration
; or recursive data cleanup. Keep the same controller gate across build identities.
!include "${__FILEDIR__}\..\package-resources\lazy-tester\process-role-links.nsh"

; Remove only a fixed role link whose exact volume/file ID equals the main image.
; Handle disposition removes that link, preserving the main image and all Data.
!macro LAZY_TESTER_REMOVE_ROLE_FUNCTION PREFIX
Function ${PREFIX}LazyTesterRemoveRoleLink
  Exch $0
  Push $1
  Push $2
  Push $3
  Push $4
  Push $5
  Push $6
  Push $7
  Push $8
  Push $9
  Push $R0
  Push $R1
  Push $R2
  Push $R3
  Push $R4
  Push $R5
  StrCpy $R0 0
  StrCpy $R1 -1
  StrCpy $R2 -1
  StrCpy $R3 0
  StrCpy $R4 0
  StrCpy $R5 0
  IfFileExists "$INSTDIR\$0" 0 role_link_done
  System::Call 'kernel32::CreateFileW(w "$INSTDIR\lazy-tester.exe", i 0x80, i 5, p 0, i 3, i 0x00200000, p 0) p.rR1'
  System::Call 'kernel32::CreateFileW(w "$INSTDIR\$0", i 0x10080, i 5, p 0, i 3, i 0x00200000, p 0) p.rR2'
  ${If} $R1 P= -1
  ${OrIf} $R2 P= -1
    Goto role_link_error
  ${EndIf}
  System::Alloc 52
  Pop $R3
  System::Alloc 52
  Pop $R4
  System::Call 'kernel32::GetFileInformationByHandle(p rR1, p rR3) i.r9'
  ${If} $9 = 0
    Goto role_link_error
  ${EndIf}
  System::Call 'kernel32::GetFileInformationByHandle(p rR2, p rR4) i.r9'
  ${If} $9 = 0
    Goto role_link_error
  ${EndIf}
  System::Call '*$R3(i .r4, i, i, i, i, i, i, i .r1, i, i, i, i .r2, i .r3)'
  System::Call '*$R4(i .r8, i, i, i, i, i, i, i .r5, i, i, i, i .r6, i .r7)'
  IntOp $4 $4 & 0x410
  IntOp $8 $8 & 0x410
  ${If} $4 != 0
  ${OrIf} $8 != 0
  ${OrIf} $1 != $5
  ${OrIf} $2 != $6
  ${OrIf} $3 != $7
    Goto role_link_error
  ${EndIf}
  System::Alloc 4
  Pop $R5
  System::Call '*$R5(i 1)'
  System::Call 'kernel32::SetFileInformationByHandle(p rR2, i 4, p rR5, i 4) i.r9'
  ${If} $9 = 0
    Goto role_link_error
  ${EndIf}
  Goto role_link_done
  role_link_error:
  StrCpy $R0 1
  role_link_done:
  ${If} $R2 P<> -1
    System::Call 'kernel32::CloseHandle(p rR2)'
  ${EndIf}
  ${If} $R1 P<> -1
    System::Call 'kernel32::CloseHandle(p rR1)'
  ${EndIf}
  System::Free $R3
  System::Free $R4
  System::Free $R5
  ${If} $R0 != 0
    MessageBox MB_OK|MB_ICONEXCLAMATION "Could not verify or update a Lazy Tester helper link. Installation stopped; review the existing App files."
    Abort
  ${EndIf}
  Pop $R5
  Pop $R4
  Pop $R3
  Pop $R2
  Pop $R1
  Pop $R0
  Pop $9
  Pop $8
  Pop $7
  Pop $6
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd
!macroend
!insertmacro LAZY_TESTER_REMOVE_ROLE_FUNCTION ""
!insertmacro LAZY_TESTER_REMOVE_ROLE_FUNCTION "un."

Function LazyTesterCreateRoleLink
  Exch $0
  Push $1
  Push $0
  Call LazyTesterRemoveRoleLink
  System::Call 'kernel32::CreateHardLinkW(w "$INSTDIR\$0", w "$INSTDIR\lazy-tester.exe", p 0) i.r1'
  ${If} $1 = 0
    MessageBox MB_OK|MB_ICONEXCLAMATION "The selected App location could not create verified helper links. Installation stopped."
    Abort
  ${EndIf}
  Pop $1
  Pop $0
FunctionEnd
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
  !insertmacro LAZY_TESTER_ROLE_FILES LazyTesterRemoveRoleLink
!macroend

!macro NSIS_HOOK_POSTINSTALL
  !insertmacro LAZY_TESTER_ROLE_FILES LazyTesterCreateRoleLink
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro LAZY_TESTER_REQUIRE_IDLE
  !insertmacro LAZY_TESTER_ROLE_FILES un.LazyTesterRemoveRoleLink
  ; Preserve application data even if the template's delete-data option was checked.
  StrCpy $DeleteAppDataCheckboxState 0
!macroend

