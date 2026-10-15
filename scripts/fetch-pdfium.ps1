param([Parameter(Mandatory=$true)][string]$Destination)
$ErrorActionPreference = 'Stop'
$manifest = Get-Content (Join-Path $PSScriptRoot '../docs/pdfium-runtime.json') -Raw | ConvertFrom-Json
$destinationPath = [IO.Path]::GetFullPath($Destination)
New-Item -ItemType Directory -Force -Path $destinationPath | Out-Null
$archivePath = Join-Path $destinationPath 'pdfium-win-x64.tgz'
if (-not (Test-Path -LiteralPath $archivePath)) {
    Invoke-WebRequest -Uri $manifest.archive_url -OutFile $archivePath
}
if ((Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash -ne $manifest.archive_sha256) {
    throw 'PDFium archive checksum mismatch'
}
tar -xf $archivePath -C $destinationPath
if ($LASTEXITCODE -ne 0) { throw 'PDFium extraction failed' }
$libraryPath = Join-Path $destinationPath 'bin/pdfium.dll'
if ((Get-FileHash -LiteralPath $libraryPath -Algorithm SHA256).Hash -ne $manifest.dll_sha256) {
    throw 'PDFium DLL checksum mismatch'
}
foreach ($entry in $manifest.notices.PSObject.Properties) {
    $noticePath = Join-Path $destinationPath $entry.Name
    if ((Get-FileHash -LiteralPath $noticePath -Algorithm SHA256).Hash -ne $entry.Value.sha256) {
        throw "PDFium notice checksum mismatch: $($entry.Name)"
    }
}
Write-Output $libraryPath
