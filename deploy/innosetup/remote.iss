; AudioMultiverse Remote - Inno Setup Script
; Erstellt Windows Installer (.exe)

#define AppName "AudioMultiverse Remote"
#define AppVersion "0.1.0"
#define AppPublisher "AudioMultiverse"
#define AppURL "https://github.com/yourusername/audiomultiverse"
#define AppExeName "audiomultiverse-remote.exe"

[Setup]
; Eindeutige App-ID (GUID generieren für Produktion!)
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}
AppUpdatesURL={#AppURL}
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
AllowNoIcons=yes
; Output-Verzeichnis wird per Kommandozeile überschrieben
OutputDir=..\..\dist\windows
OutputBaseFilename=AudioMultiverse-Remote-Setup-{#AppVersion}
SetupIconFile=..\..\app\src-tauri\icons\icon.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
; Für Windows 10/11
MinVersion=10.0.17763
; 64-bit only
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
; Admin-Rechte nicht benötigt für User-Install
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

[Languages]
Name: "german"; MessagesFile: "compiler:Languages\German.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "autostart"; Description: "Mit Windows starten"; GroupDescription: "Zusätzliche Optionen:"; Flags: unchecked

[Files]
; Tauri Build Output
Source: "..\..\remote\src-tauri\target\release\{#AppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\remote\src-tauri\target\release\*.dll"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

; WebView2 Bootstrapper (falls nicht installiert)
Source: "MicrosoftEdgeWebview2Setup.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall skipifsourcedoesntexist

; Zusätzliche Dateien
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExeName}"
Name: "{group}\{cm:UninstallProgram,{#AppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Registry]
; Autostart
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "AudioMultiverseRemote"; ValueData: """{app}\{#AppExeName}"""; Flags: uninsdeletevalue; Tasks: autostart

[Run]
; WebView2 installieren falls nicht vorhanden
Filename: "{tmp}\MicrosoftEdgeWebview2Setup.exe"; Parameters: "/silent /install"; StatusMsg: "WebView2 Runtime wird installiert..."; Flags: waituntilterminated skipifdoesntexist

; App nach Installation starten
Filename: "{app}\{#AppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(AppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[Code]
// Prüfen ob WebView2 bereits installiert ist
function IsWebView2Installed: Boolean;
var
  ResultCode: Integer;
begin
  Result := RegKeyExists(HKEY_LOCAL_MACHINE, 'SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}') or
            RegKeyExists(HKEY_CURRENT_USER, 'Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}');
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssInstall then
  begin
    // Vor Installation
  end;
end;

function InitializeSetup: Boolean;
begin
  Result := True;
  
  // Hinweis wenn WebView2 fehlt
  if not IsWebView2Installed then
  begin
    if MsgBox('Microsoft Edge WebView2 Runtime wird benötigt. ' +
              'Soll es jetzt installiert werden?', mbConfirmation, MB_YESNO) = IDNO then
    begin
      // Warnung aber trotzdem fortfahren
      MsgBox('Die Anwendung funktioniert möglicherweise nicht ohne WebView2.', mbInformation, MB_OK);
    end;
  end;
end;

[UninstallDelete]
Type: filesandordirs; Name: "{app}\logs"
Type: filesandordirs; Name: "{app}\config"
