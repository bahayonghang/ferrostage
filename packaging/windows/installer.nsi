Unicode true
RequestExecutionLevel user
ManifestDPIAware true
SetCompressor /SOLID zlib
!include "MUI2.nsh"
!include "WordFunc.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
!include "WinVer.nsh"
!include "${PAYLOAD_INCLUDE}"

!define APP_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\FerroStage"
Name "FerroStage"
OutFile "${OUTPUT_FILE}"
InstallDir "$LOCALAPPDATA\Programs\FerroStage"
Icon "${APP_ICON}"
UninstallIcon "${APP_ICON}"
VIProductVersion "${APP_VERSION}.0"
VIAddVersionKey /LANG=1033 "ProductName" "FerroStage Setup"
VIAddVersionKey /LANG=1033 "FileDescription" "FerroStage current-user installer"
VIAddVersionKey /LANG=1033 "FileVersion" "${APP_VERSION}"
VIAddVersionKey /LANG=1033 "LegalCopyright" "FerroStage contributors"
ShowInstDetails show
ShowUninstDetails show
!define MUI_ABORTWARNING
!define MUI_ICON "${APP_ICON}"
!define MUI_UNICON "${APP_ICON}"
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "${APP_LICENSE}"
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH
!insertmacro MUI_LANGUAGE "English"

Var InstallMutex
Var PreviousVersion
Var PreviousSize

!macro Init FUNCTION PREFIX
Function ${FUNCTION}
  SetShellVarContext current
  StrCpy $INSTDIR "$LOCALAPPDATA\Programs\FerroStage"
  ${IfNot} ${RunningX64}
    SetErrorLevel 10
    Abort "FerroStage requires Windows x64."
  ${EndIf}
  System::Call 'kernel32::CreateMutexW(p 0, i 0, w "Local\FerroStage.Setup") p.r0 ?e'
  Pop $1
  StrCpy $InstallMutex $0
  ${If} $1 == 183
    SetErrorLevel 2
    Abort "Another FerroStage installer is already running."
  ${EndIf}
  !insertmacro CheckDirectories "${PREFIX}"
FunctionEnd
!macroend
!insertmacro Init ".onInit" ""
!insertmacro Init "un.onInit" "un."

!macro RestoreShortcuts
  Delete "$DESKTOP\FerroStage.lnk"
  Delete "$SMPROGRAMS\FerroStage.lnk"
  IfFileExists "$PLUGINSDIR\desktop.lnk" 0 +2
  CopyFiles /SILENT "$PLUGINSDIR\desktop.lnk" "$DESKTOP\FerroStage.lnk"
  IfFileExists "$PLUGINSDIR\start.lnk" 0 +2
  CopyFiles /SILENT "$PLUGINSDIR\start.lnk" "$SMPROGRAMS\FerroStage.lnk"
!macroend

!macro CheckShortcuts PREFIX
  Push "$DESKTOP\FerroStage.lnk"
  Call ${PREFIX}CheckFile
  Push "$SMPROGRAMS\FerroStage.lnk"
  Call ${PREFIX}CheckFile
!macroend

!macro Preflight PREFIX
Function ${PREFIX}CheckDirectory
  Exch $0
  System::Call 'kernel32::GetFileAttributesW(w r0) i.r1'
  ${If} $1 != -1
    IntOp $1 $1 & 0x400
    ${If} $1 != 0
      SetErrorLevel 3
      Abort "The installation path contains a link. No files were changed."
    ${EndIf}
  ${EndIf}
  Pop $0
FunctionEnd

Function ${PREFIX}CheckFile
  Exch $0
  IfFileExists "$0" 0 done
  System::Call 'kernel32::GetFileAttributesW(w r0) i.r1'
  IntOp $1 $1 & 0x400
  ${If} $1 != 0
    SetErrorLevel 3
    Abort "An owned installation file is a link. No files were changed."
  ${EndIf}
  ; Write access with no sharing catches running executable/DLL and locked notices.
  System::Call 'kernel32::CreateFileW(w r0, i 0xC0000000, i 0, p 0, i 3, i 0, p 0) p.r1'
  ${If} $1 == -1
    IfSilent locked
    MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "Close FerroStage and any program using its files, then choose Retry." IDRETRY retry
    locked:
    SetErrorLevel 4
    Abort "Installation files are in use. No files were changed."
    retry:
    Push $0
    Call ${PREFIX}CheckFile
  ${Else}
    System::Call 'kernel32::CloseHandle(p r1)'
  ${EndIf}
  done:
  Pop $0
FunctionEnd
!macroend
!insertmacro Preflight ""
!insertmacro Preflight "un."

Section "Install"
  ${IfNot} ${AtLeastWin10}
  ${OrIfNot} ${AtLeastBuild} 18362
    SetErrorLevel 10
    Abort "FerroStage requires Windows 10 version 1903 (build 18362) or later, x64."
  ${EndIf}
  ${DisableX64FSRedirection}
  IfFileExists "$WINDIR\System32\VCRUNTIME140.dll" runtime_found
  ${EnableX64FSRedirection}
  IfSilent +2
  MessageBox MB_OK|MB_ICONSTOP "Install the Microsoft Visual C++ x64 Redistributable, then run Setup again:$\r$\nhttps://aka.ms/vc14/vc_redist.x64.exe"
  SetErrorLevel 11
  Abort "Microsoft Visual C++ x64 Redistributable is required."
  runtime_found:
  ${EnableX64FSRedirection}
  ReadRegStr $0 HKCU "${APP_KEY}" "DisplayVersion"
  StrCpy $PreviousVersion $0
  ReadRegDWORD $PreviousSize HKCU "${APP_KEY}" "EstimatedSize"
  ${If} $0 != ""
    ${VersionCompare} "$0" "${APP_VERSION}" $1
    ${If} $1 == 1
      SetErrorLevel 5
      Abort "A newer FerroStage version is installed. Downgrade is not supported."
    ${EndIf}
  ${EndIf}
  InitPluginsDir
  ClearErrors
  !insertmacro PayloadExtract
  IfErrors extraction_failed
  !insertmacro CheckFiles ""
  !insertmacro CheckShortcuts ""
  !insertmacro PayloadBackup
  ClearErrors
  IfFileExists "$DESKTOP\FerroStage.lnk" 0 +2
  CopyFiles /SILENT "$DESKTOP\FerroStage.lnk" "$PLUGINSDIR\desktop.lnk"
  IfErrors backup_failed
  IfFileExists "$SMPROGRAMS\FerroStage.lnk" 0 +2
  CopyFiles /SILENT "$SMPROGRAMS\FerroStage.lnk" "$PLUGINSDIR\start.lnk"
  IfErrors backup_failed
  !insertmacro PayloadInstall
  ClearErrors
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  IfErrors install_failed
  SetOutPath "$INSTDIR"
  CreateShortcut "$DESKTOP\FerroStage.lnk" "$INSTDIR\ferrostage.exe" "" "$INSTDIR\ferrostage.exe" 0
  IfErrors install_failed
  CreateShortcut "$SMPROGRAMS\FerroStage.lnk" "$INSTDIR\ferrostage.exe" "" "$INSTDIR\ferrostage.exe" 0
  IfErrors install_failed
  WriteRegStr HKCU "${APP_KEY}" "DisplayName" "FerroStage"
  WriteRegStr HKCU "${APP_KEY}" "Publisher" "FerroStage contributors"
  WriteRegStr HKCU "${APP_KEY}" "DisplayIcon" "$INSTDIR\ferrostage.exe,0"
  WriteRegStr HKCU "${APP_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${APP_KEY}" "UninstallString" '$\"$INSTDIR\Uninstall.exe$\"'
  WriteRegStr HKCU "${APP_KEY}" "QuietUninstallString" '$\"$INSTDIR\Uninstall.exe$\" /S'
  WriteRegDWORD HKCU "${APP_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${APP_KEY}" "NoRepair" 1
  ; EstimatedSize is in KiB and includes the complete embedded payload.
  WriteRegDWORD HKCU "${APP_KEY}" "EstimatedSize" ${APP_SIZE_KIB}
  IfErrors registration_failed
  WriteRegStr HKCU "${APP_KEY}" "DisplayVersion" "${APP_VERSION}"
  IfErrors registration_failed
  SetErrorLevel 0
  Goto complete
  extraction_failed:
    SetErrorLevel 6
    Abort "Setup could not unpack its payload. The installed version was not changed."
  backup_failed:
    SetErrorLevel 7
    Abort "Setup could not back up the installed files. No files were replaced."
  install_failed:
    !insertmacro PayloadRestore
    !insertmacro RestoreShortcuts
    SetErrorLevel 8
    Abort "Setup could not finish installation. Previous payload files were restored."
  registration_failed:
    ; Do not advertise a successful new version after registration failure.
    ${If} $PreviousVersion == ""
      DeleteRegKey HKCU "${APP_KEY}"
    ${Else}
      WriteRegStr HKCU "${APP_KEY}" "DisplayVersion" "$PreviousVersion"
      WriteRegDWORD HKCU "${APP_KEY}" "EstimatedSize" $PreviousSize
    ${EndIf}
    !insertmacro PayloadRestore
    !insertmacro RestoreShortcuts
    SetErrorLevel 9
    Abort "Setup could not register FerroStage. Previous payload files were restored."
  complete:
SectionEnd

Section "Uninstall"
  !insertmacro CheckFiles "un."
  !insertmacro CheckShortcuts "un."
  ClearErrors
  !insertmacro PayloadDelete
  IfErrors uninstall_failed
  IfFileExists "$DESKTOP\FerroStage.lnk" 0 +2
  Delete "$DESKTOP\FerroStage.lnk"
  IfErrors uninstall_failed
  IfFileExists "$SMPROGRAMS\FerroStage.lnk" 0 +2
  Delete "$SMPROGRAMS\FerroStage.lnk"
  IfErrors uninstall_failed
  DeleteRegKey HKCU "${APP_KEY}"
  IfErrors uninstall_failed
  ; Keep the registered recovery executable until all earlier cleanup succeeded.
  ClearErrors
  Delete "$INSTDIR\Uninstall.exe"
  IfErrors final_cleanup_failed
  !insertmacro RemoveEmptyDirectories
  SetErrorLevel 0
  Goto uninstall_complete
  uninstall_failed:
    SetErrorLevel 12
    Abort "Some files could not be removed. Registration was preserved; close programs and retry."
  final_cleanup_failed:
    SetErrorLevel 13
    Abort "FerroStage was removed, but its uninstaller file could not be deleted. Close programs and run Setup to repair or retry cleanup."
  uninstall_complete:
SectionEnd
