; Inno Setup script for the Windows installer, built by scripts/build-windows.sh --native:
;   ISCC /DAppVersion=0.2.0 /DNumVersion=0.2.0.0 /DSourceDir=<stage> /DIconFile=<ico>
;        /DOutputDir=<dist> /DOutputBase=Varstotas-0.2.0-windows-x86_64-setup varstotas.iss
; Installs per user (no admin) or for all users (the wizard asks), with a Start menu entry,
; an optional desktop icon and the .vrst file association.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef NumVersion
  #define NumVersion "0.0.0.0"
#endif

[Setup]
AppId={{2BD864DB-C4B3-457C-9A3F-9CBD48469539}
AppName=Varstotas
AppVersion={#AppVersion}
AppVerName=Varstotas {#AppVersion}
AppPublisher=Varstotas
AppPublisherURL=https://github.com/dirondin/varstotas
AppSupportURL=https://github.com/dirondin/varstotas/issues
AppUpdatesURL=https://github.com/dirondin/varstotas/releases
VersionInfoVersion={#NumVersion}
VersionInfoProductName=Varstotas
VersionInfoDescription=Varstotas setup
DefaultDirName={autopf}\Varstotas
DefaultGroupName=Varstotas
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
LicenseFile={#SourceDir}\LICENSE.txt
SetupIconFile={#IconFile}
UninstallDisplayIcon={app}\varstotas.exe
UninstallDisplayName=Varstotas
OutputDir={#OutputDir}
OutputBaseFilename={#OutputBase}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ChangesAssociations=yes
CloseApplications=yes

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "ru"; MessagesFile: "compiler:Languages\Russian.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "assoc"; Description: "{cm:AssocFileExtension,Varstotas,.vrst}"

[Files]
Source: "{#SourceDir}\varstotas.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\LICENSE.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\THIRD-PARTY-LICENSES.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Varstotas"; Filename: "{app}\varstotas.exe"
Name: "{autodesktop}\Varstotas"; Filename: "{app}\varstotas.exe"; Tasks: desktopicon

[Registry]
Root: HKA; Subkey: "Software\Classes\.vrst"; ValueType: string; ValueName: ""; ValueData: "Varstotas.Document"; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\.vrst\OpenWithProgids"; ValueType: string; ValueName: "Varstotas.Document"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Varstotas.Document"; ValueType: string; ValueName: ""; ValueData: "Varstotas document"; Flags: uninsdeletekey; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Varstotas.Document\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\varstotas.exe,0"; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Varstotas.Document\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\varstotas.exe"" ""%1"""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\varstotas.exe\SupportedTypes"; ValueType: string; ValueName: ".vrst"; ValueData: ""; Flags: uninsdeletekey; Tasks: assoc

[Run]
Filename: "{app}\varstotas.exe"; Description: "{cm:LaunchProgram,Varstotas}"; Flags: nowait postinstall skipifsilent
