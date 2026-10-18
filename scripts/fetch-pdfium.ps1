param([Parameter(Mandatory=$true)][string]$Destination)
$ErrorActionPreference = 'Stop'

function Get-Sha256Hex([string]$Path) {
    # Windows PowerShell under just can fail to see Get-FileHash.
    $sha256 = [System.Security.Cryptography.SHA256]::Create()
    try {
        $stream = [System.IO.File]::Open(
            $Path,
            [System.IO.FileMode]::Open,
            [System.IO.FileAccess]::Read,
            [System.IO.FileShare]::ReadWrite
        )
        try {
            return ([System.BitConverter]::ToString($sha256.ComputeHash($stream))).Replace('-', '')
        } finally {
            $stream.Dispose()
        }
    } finally {
        $sha256.Dispose()
    }
}

function Test-Sha256Hex([string]$Path, [string]$Expected) {
    return (Get-Sha256Hex $Path).Equals($Expected, [StringComparison]::OrdinalIgnoreCase)
}

$manifest = Get-Content (Join-Path $PSScriptRoot '../docs/pdfium-runtime.json') -Raw | ConvertFrom-Json
$destinationPath = [IO.Path]::GetFullPath($Destination)
New-Item -ItemType Directory -Force -Path $destinationPath | Out-Null
$archivePath = Join-Path $destinationPath 'pdfium-win-x64.tgz'
if (-not (Test-Path -LiteralPath $archivePath)) {
    Invoke-WebRequest -Uri $manifest.archive_url -OutFile $archivePath
}
if (-not (Test-Sha256Hex $archivePath $manifest.archive_sha256)) {
    throw 'PDFium archive checksum mismatch'
}
tar -xf $archivePath -C $destinationPath
if ($LASTEXITCODE -ne 0) { throw 'PDFium extraction failed' }
$libraryPath = Join-Path $destinationPath 'bin/pdfium.dll'
if (-not (Test-Sha256Hex $libraryPath $manifest.dll_sha256)) {
    throw 'PDFium DLL checksum mismatch'
}
foreach ($entry in $manifest.notices.PSObject.Properties) {
    $noticePath = Join-Path $destinationPath $entry.Name
    if (-not (Test-Sha256Hex $noticePath $entry.Value.sha256)) {
        throw "PDFium notice checksum mismatch: $($entry.Name)"
    }
}
Write-Output $libraryPath
