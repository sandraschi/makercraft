; MakerCraft current-user installer (no admin UAC required)
!include "MUI2.nsh"
Name "MakerCraft"
OutFile "..\dist\makercraft-0.1.0-windows-x64-setup.exe"
InstallDir "$LOCALAPPDATA\Programs\MakerCraft"
RequestExecutionLevel user
ShowInstDetails show

Section "Binaries" SEC_BIN
  SetOutPath "$INSTDIR\bin"
  File "..\target\release\makercraft.exe"
  File "..\target\release\makercraft-mcp.exe"
  WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\bin\makercraft.exe"
  Delete "$INSTDIR\bin\makercraft-mcp.exe"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR\bin"
  RMDir "$INSTDIR"
SectionEnd
