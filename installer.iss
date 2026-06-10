; FlowForge Installer Script for Inno Setup 6
; Generates FlowForge-v1.0.0-Setup.exe

#define MyAppName "FlowForge"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "FlowForge"
#define MyAppExeName "flowforge.exe"

[Setup]
AppId={{FLOWFORGE-2026-06-10}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
OutputDir=C:\Users\haozi\dev\flowforge
OutputBaseFilename=FlowForge-v{#MyAppVersion}-Setup
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "C:\Users\haozi\dev\flowforge\dist\flowforge.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "C:\Users\haozi\dev\flowforge\dist\flutter_windows.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "C:\Users\haozi\dev\flowforge\dist\data\*"; DestDir: "{app}\data"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "C:\Users\haozi\dev\flowforge\dist\backend\flowforge.exe"; DestDir: "{app}\backend"; Flags: ignoreversion

[Icons]
Name: "{group}\FlowForge"; Filename: "{app}\FlowForgeLauncher.bat"; IconFilename: "{app}\flowforge.exe"
Name: "{group}\Uninstall FlowForge"; Filename: "{uninstallexe}"
Name: "{autodesktop}\FlowForge"; Filename: "{app}\FlowForgeLauncher.bat"; IconFilename: "{app}\flowforge.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\FlowForgeLauncher.bat"; Description: "Launch FlowForge"; Flags: postinstall shellexec skipifsilent

[Code]
procedure CurStepChanged(CurStep: TSetupStep);
var
  LauncherPath, AppDir, Content: AnsiString;
begin
  if CurStep = ssPostInstall then
  begin
    AppDir := ExpandConstant('{app}');
    LauncherPath := AppDir + '\FlowForgeLauncher.bat';
    Content := '@echo off' + #13#10 +
      'title FlowForge v1.0.0' + #13#10 +
      'echo.' + #13#10 +
      'echo  FlowForge v1.0.0' + #13#10 +
      'echo  Starting...' + #13#10 +
      'echo.' + #13#10 +
      'start /B "" "' + AppDir + '\backend\flowforge.exe"' + #13#10 +
      'timeout /t 2 /nobreak >nul' + #13#10 +
      'set SERVER_URL=http://127.0.0.1:19529' + #13#10 +
      'start "" "' + AppDir + '\flowforge.exe"' + #13#10 +
      'exit';
    SaveStringToFile(LauncherPath, Content, False);
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  ResultCode: Integer;
begin
  if CurUninstallStep = usUninstall then
  begin
    Exec('taskkill', '/IM flowforge.exe /F', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
    DelTree(ExpandConstant('{app}'), True, True, True);
  end;
end;
