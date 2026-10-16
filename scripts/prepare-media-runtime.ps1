param([string]$Destination=(Join-Path $PSScriptRoot '../cache/media-runtime'),[string]$Archives=(Join-Path $PSScriptRoot '../cache/media-archives'),[string]$ImportLibraries=(Join-Path $PSScriptRoot '../cache/media-import-libs'),[switch]$Offline)
$ErrorActionPreference='Stop'
$mediaManifest=Get-Content -Raw (Join-Path $PSScriptRoot '../packaging/media/runtime.json')|ConvertFrom-Json
New-Item -ItemType Directory -Force -Path $Archives,$ImportLibraries|Out-Null
foreach($archive in $mediaManifest.archives){
 $path=Join-Path $Archives $archive.file
 if(-not(Test-Path -LiteralPath $path)){
  if($Offline){throw "Missing reviewed archive $($archive.file)"}
  Invoke-WebRequest -Uri $archive.url -OutFile $path
 }
 if((Get-Item -LiteralPath $path).Length -ne $archive.bytes -or (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash -ne $archive.sha256){throw "Archive hash mismatch: $($archive.file)"}
}
& python (Join-Path $PSScriptRoot 'stage-media-runtime.py') --archives $Archives --output $Destination
if($LASTEXITCODE -ne 0){throw 'Selective runtime staging failed'}
$mediaVswhere=Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$mediaVs=& $mediaVswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if(-not $mediaVs){throw 'MSVC x64 build tools are required to generate safe import libraries'}
$mediaTools=Get-ChildItem (Join-Path $mediaVs 'VC/Tools/MSVC') -Directory|Sort-Object Name -Descending|Select-Object -First 1
$mediaBin=Join-Path $mediaTools.FullName 'bin/Hostx64/x64'
$mediaMap=@{'glib-2.0'='glib-2.0-0.dll';'gobject-2.0'='gobject-2.0-0.dll';'gio-2.0'='gio-2.0-0.dll';'gstreamer-1.0'='gstreamer-1.0-0.dll';'gstapp-1.0'='gstapp-1.0-0.dll';'gstbase-1.0'='gstbase-1.0-0.dll';'gstvideo-1.0'='gstvideo-1.0-0.dll'}
foreach($item in $mediaMap.GetEnumerator()){
 $exports=& (Join-Path $mediaBin 'dumpbin.exe') /exports (Join-Path $Destination "bin/$($item.Value)")
 if($LASTEXITCODE -ne 0){throw 'Export inspection failed'}
 $names=@($exports|ForEach-Object {if($_ -match '^\s+\d+\s+[0-9A-Fa-f]+\s+[0-9A-Fa-f]+\s+(\S+)'){$Matches[1]}})
 if($names.Count -eq 0){throw 'No native exports found'}
 $definition=Join-Path $ImportLibraries "$($item.Key).def"
 @("LIBRARY $($item.Value)",'EXPORTS')+$names|Set-Content -Encoding ascii -LiteralPath $definition
 & (Join-Path $mediaBin 'lib.exe') /nologo /machine:x64 "/def:$definition" "/out:$(Join-Path $ImportLibraries "$($item.Key).lib")"
 if($LASTEXITCODE -ne 0){throw 'Import library generation failed'}
}
Write-Output "Prepared selected runtime: $Destination; import libraries: $ImportLibraries"
