; HomeLumen installer script, for NSIS (https://nsis.sourceforge.io/).
;
; Unlike Inno Setup, NSIS's compiler (makensis) runs on Linux as well as
; Windows, so this installer is built the same way the binary itself is:
; from source, on whichever machine is doing the building. See
; packaging/windows/build.sh for the one command that does both.

!include "MUI2.nsh"

!define APP_NAME "Home Lumen"
!define APP_VERSION "0.1.0"
!define APP_PUBLISHER "The HomeLumen contributors"
!define APP_EXE "homelumen.exe"
!define APP_URL "https://github.com/Victor-root/HomeLumen"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\HomeLumen"

Name "${APP_NAME}"
OutFile "..\..\target\installer\HomeLumen-Setup.exe"
InstallDir "$PROGRAMFILES64\${APP_NAME}"
InstallDirRegKey HKLM "Software\HomeLumen" "InstallDir"
RequestExecutionLevel admin
SetCompressor /SOLID lzma

!define MUI_ABORTWARNING
!define MUI_ICON "..\..\assets\icon\icon.ico"
!define MUI_UNICON "..\..\assets\icon\icon.ico"
!define MUI_FINISHPAGE_RUN "$INSTDIR\${APP_EXE}"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "French"

; NSIS has no direct equivalent of Inno Setup's automatic "detect the
; system's language" behaviour: without this, a multi-language installer
; just opens on the first `MUI_LANGUAGE` declared (English) and asks nothing.
; The block below reads Windows' own UI language at start-up and pre-selects
; French only when it matches, silently, before the picker would show.
Function .onInit
  System::Call "kernel32::GetUserDefaultUILanguage() i .r0"
  StrCmp $0 "1036" 0 +2
    StrCpy $LANGUAGE ${LANG_FRENCH}
FunctionEnd

Section "!Home Lumen" SecMain
  SectionIn RO
  SetOutPath "$INSTDIR"
  File "..\..\target\x86_64-pc-windows-gnu\release\${APP_EXE}"
  File "..\..\LICENSE"

  CreateDirectory "$SMPROGRAMS\${APP_NAME}"
  CreateShortcut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}"
  CreateShortcut "$SMPROGRAMS\${APP_NAME}\Uninstall.lnk" "$INSTDIR\uninstall.exe"

  WriteRegStr HKLM "Software\HomeLumen" "InstallDir" "$INSTDIR"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "Publisher" "${APP_PUBLISHER}"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "URLInfoAbout" "${APP_URL}"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\${APP_EXE}"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKLM "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoRepair" 1
  WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Desktop shortcut" SecDesktop
  CreateShortcut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\${APP_EXE}"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"

  Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
  Delete "$SMPROGRAMS\${APP_NAME}\Uninstall.lnk"
  RMDir "$SMPROGRAMS\${APP_NAME}"
  Delete "$DESKTOP\${APP_NAME}.lnk"

  DeleteRegKey HKLM "${UNINSTALL_KEY}"
  DeleteRegKey HKLM "Software\HomeLumen"
SectionEnd
