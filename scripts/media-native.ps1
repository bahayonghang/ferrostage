param([ValidateSet('check','build','clippy','test')][string]$Action='build',[string]$ImportLibraries=(Join-Path $PSScriptRoot '../cache/media-import-libs'))
$ErrorActionPreference='Stop'
$mediaRoot=Split-Path -Parent $PSScriptRoot
$mediaLibraries=(Resolve-Path -LiteralPath $ImportLibraries).Path
if (-not (Test-Path -LiteralPath $mediaLibraries)) { throw 'Prepare the reviewed runtime import libraries first; see docs/packaging.md.' }
$mediaDeps=@{GLIB_2_0='glib-2.0';GOBJECT_2_0='gobject-2.0';GIO_2_0='gio-2.0';GSTREAMER_1_0='gstreamer-1.0';GSTREAMER_APP_1_0='gstapp-1.0';GSTREAMER_BASE_1_0='gstbase-1.0';GSTREAMER_VIDEO_1_0='gstvideo-1.0'}
foreach($item in $mediaDeps.GetEnumerator()) {
 Set-Item -LiteralPath "Env:SYSTEM_DEPS_$($item.Key)_NO_PKG_CONFIG" -Value '1'
 Set-Item -LiteralPath "Env:SYSTEM_DEPS_$($item.Key)_LIB" -Value $item.Value
 Set-Item -LiteralPath "Env:SYSTEM_DEPS_$($item.Key)_SEARCH_NATIVE" -Value $mediaLibraries
}
$mediaArgs=@($Action,'-p','ferrostage-media-helper','--features','native','--all-targets','--locked')
if($Action -eq 'clippy'){$mediaArgs+=@('--','-D','warnings')}
& cargo @mediaArgs
exit $LASTEXITCODE
