param([switch]$Clippy)
$ErrorActionPreference = 'Stop'
$probeRoot = Split-Path -Parent $PSCommandPath
$probeLibraries = Join-Path $probeRoot 'import-libs'
$probeDeps = @{
    GLIB_2_0 = 'glib-2.0'
    GOBJECT_2_0 = 'gobject-2.0'
    GIO_2_0 = 'gio-2.0'
    GSTREAMER_1_0 = 'gstreamer-1.0'
    GSTREAMER_APP_1_0 = 'gstapp-1.0'
    GSTREAMER_BASE_1_0 = 'gstbase-1.0'
    GSTREAMER_VIDEO_1_0 = 'gstvideo-1.0'
}
foreach ($item in $probeDeps.GetEnumerator()) {
    Set-Item -LiteralPath "Env:SYSTEM_DEPS_$($item.Key)_NO_PKG_CONFIG" -Value '1'
    Set-Item -LiteralPath "Env:SYSTEM_DEPS_$($item.Key)_LIB" -Value $item.Value
    Set-Item -LiteralPath "Env:SYSTEM_DEPS_$($item.Key)_SEARCH_NATIVE" -Value $probeLibraries
}
$probeManifest = Join-Path $probeRoot 'rust-probe/Cargo.toml'
if ($Clippy) {
    & cargo clippy --manifest-path $probeManifest --offline --all-targets -- -D warnings 2>&1 | Tee-Object -FilePath (Join-Path $probeRoot 'rust-probe-clippy.log')
} else {
    & cargo build --manifest-path $probeManifest --offline 2>&1 | Tee-Object -FilePath (Join-Path $probeRoot 'rust-probe-build.log')
}
exit $LASTEXITCODE
