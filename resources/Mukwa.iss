#define AppName "Mukwa"
#define AppId "{{db71d5de-df75-49a2-a950-655d772281cb}"

[Setup]
AppId={#AppId}
AppName={#AppName}
AppVersion={#AppVersion}
LicenseFile={#ResourceDir}/LICENSE
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
UninstallDisplayName={#AppName}
PrivilegesRequiredOverridesAllowed=dialog
PrivilegesRequired=lowest
AppCopyright=Copyright (C) 2026 Wakunguma Kalimukwa
AppPublisher=Wakunguma Kalimukwa
AppPublisherURL=https://www.github.com/snubwoody/mukwa
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
;TODO: compile for arm and x86_64

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\mukwa.exe"

[UninstallDelete]
Type: filesandordirs; Name: "{app}"

[Files]
Source: "{#ResourceDir}\mukwa.exe"; DestDir: "{app}"; Flags: ignoreversion
