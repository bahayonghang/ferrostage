param([Parameter(Mandatory=$true)][string]$Binary)
$ErrorActionPreference='Stop'
$mediaVswhere=Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$mediaVs=& $mediaVswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
$mediaTools=Get-ChildItem (Join-Path $mediaVs 'VC/Tools/MSVC') -Directory|Sort-Object Name -Descending|Select-Object -First 1
$imports=& (Join-Path $mediaTools.FullName 'bin/Hostx64/x64/dumpbin.exe') /imports $Binary
if($LASTEXITCODE -ne 0){throw 'GUI PE inspection failed'}
if($imports -match '(?i)(gstreamer|gstapp|gstbase|gstvideo|glib-2|gobject-2|gio-2).*\.dll'){throw 'Native media DLL import leaked into GUI'}
Write-Output "PASS GUI media imports: $((Get-FileHash -Algorithm SHA256 -LiteralPath $Binary).Hash)"
