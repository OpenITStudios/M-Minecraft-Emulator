!macro VC_REQUIRE_DESKTOP_CLOSED
  FindWindow $0 "" "M-Bedrock Virtual Clients"
  ${If} $0 != 0
    MessageBox MB_OK|MB_ICONEXCLAMATION "M-Bedrock Virtual Clients is currently open. Close the app, then run this installer again." /SD IDOK
    Abort
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREINSTALL
  !insertmacro VC_REQUIRE_DESKTOP_CLOSED
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro VC_REQUIRE_DESKTOP_CLOSED
!macroend
