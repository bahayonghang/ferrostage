# Stage the pinned PDFium runtime from cache/pdfium-7881-win-x64.
# Fetch runs only when bin/pdfium.dll is missing or its SHA256 differs.
param(
    [string]$Destination,
    [switch]$DllOnly,
    [switch]$CargoBin
)
$ErrorActionPreference = 'Stop'

function Get-Sha256Hex([string]$Path) {
    # Just launches Windows PowerShell with a module path that can hide Get-FileHash.
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
    if (-not (Test-Path -LiteralPath $Path)) {
        return $false
    }
    return (Get-Sha256Hex $Path).Equals($Expected, [StringComparison]::OrdinalIgnoreCase)
}

$repoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$manifest = Get-Content -Raw (Join-Path $repoRoot 'docs/pdfium-runtime.json') | ConvertFrom-Json
$cache = Join-Path $repoRoot 'cache/pdfium-7881-win-x64'
$cachedDll = Join-Path $cache 'bin/pdfium.dll'

if (-not (Test-Sha256Hex $cachedDll $manifest.dll_sha256)) {
    & (Join-Path $PSScriptRoot 'fetch-pdfium.ps1') -Destination $cache
}
if (-not (Test-Sha256Hex $cachedDll $manifest.dll_sha256)) {
    throw 'PDFium DLL checksum mismatch'
}

if ($CargoBin) {
    $cargoHome = $env:CARGO_HOME
    if ([string]::IsNullOrWhiteSpace($cargoHome)) {
        $cargoHome = Join-Path $env:USERPROFILE '.cargo'
    }
    $Destination = Join-Path $cargoHome 'bin'
} elseif ([string]::IsNullOrWhiteSpace($Destination)) {
    throw '-Destination is required unless -CargoBin is set'
}

$destinationPath = [IO.Path]::GetFullPath($Destination)
New-Item -ItemType Directory -Force -Path $destinationPath | Out-Null

$stagedDll = Join-Path $destinationPath 'pdfium.dll'
Copy-Item -LiteralPath $cachedDll -Destination $stagedDll -Force
if (-not (Test-Sha256Hex $stagedDll $manifest.dll_sha256)) {
    throw 'PDFium DLL checksum mismatch at destination'
}

if (-not $DllOnly) {
    Copy-Item -LiteralPath (Join-Path $cache 'LICENSE') -Destination (Join-Path $destinationPath 'LICENSE') -Force
    $licensesDestination = Join-Path $destinationPath 'licenses'
    New-Item -ItemType Directory -Force -Path $licensesDestination | Out-Null
    Copy-Item -Path (Join-Path $cache 'licenses/*') -Destination $licensesDestination -Recurse -Force
    foreach ($entry in $manifest.notices.PSObject.Properties) {
        $noticePath = Join-Path $destinationPath $entry.Name
        if (-not (Test-Sha256Hex $noticePath $entry.Value.sha256)) {
            throw "PDFium notice checksum mismatch: $($entry.Name)"
        }
    }
}

Write-Output $stagedDll
