; HomeLumen installer script, for Inno Setup 6 (https://jrsoftware.org/isinfo.php).
;
; Build with `cargo build --release` first, then compile this file (open it
; in the Inno Setup Compiler and press Build, or run `iscc homelumen.iss`
; from this folder). The result lands in target\installer\HomeLumen-Setup.exe.
;
; AppId is a fixed, random GUID: it is what lets a later installer recognise
; an existing HomeLumen and upgrade it in place rather than install a second
; copy beside it. Keep it exactly as it is across every version.

; MyAppVersion is not read from Cargo.toml: bump it by hand alongside
; `workspace.package.version` there.
#define MyAppName "Home Lumen"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "The HomeLumen contributors"
#define MyAppExeName "homelumen.exe"
#define MyAppURL "https://github.com/Victor-root/HomeLumen"

[Setup]
AppId={{9B5BF86F-4F3F-4346-B3BD-FE219C8AEE2D}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
UninstallDisplayIcon={app}\{#MyAppExeName}
OutputDir=..\..\target\installer
OutputBaseFilename=HomeLumen-Setup
Compression=lzma
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
SetupIconFile=..\..\assets\icon\icon.ico
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: checkedonce

[Files]
Source: "..\..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#MyAppName}}"; Flags: nowait postinstall skipifsilent
