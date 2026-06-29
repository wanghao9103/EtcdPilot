#define AppName "EtcdPilot"
#ifndef AppVersion
#define AppVersion "dev"
#endif
#ifndef SourceDir
#define SourceDir "..\..\build\package\EtcdPilot-dev-windows-x64"
#endif
#ifndef OutputDir
#define OutputDir "..\..\artifacts"
#endif

[Setup]
AppId={{6E8A7B56-F89A-4D34-A180-78D9F84B6C5E}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=EtcdPilot
DefaultDirName={autopf}\EtcdPilot
DefaultGroupName=EtcdPilot
DisableProgramGroupPage=yes
OutputDir={#OutputDir}
OutputBaseFilename=EtcdPilot-Setup-{#AppVersion}-windows-x64
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64

[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\EtcdPilot"; Filename: "powershell.exe"; Parameters: "-ExecutionPolicy Bypass -File ""{app}\start.ps1"""; WorkingDir: "{app}"
Name: "{commondesktop}\EtcdPilot"; Filename: "powershell.exe"; Parameters: "-ExecutionPolicy Bypass -File ""{app}\start.ps1"""; WorkingDir: "{app}"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Run]
Filename: "powershell.exe"; Parameters: "-ExecutionPolicy Bypass -File ""{app}\start.ps1"""; Description: "Launch EtcdPilot"; Flags: postinstall nowait skipifsilent
