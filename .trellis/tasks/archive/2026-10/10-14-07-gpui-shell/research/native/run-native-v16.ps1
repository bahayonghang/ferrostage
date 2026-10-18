param(
    [Parameter(Mandatory)][ValidatePattern('^v16-[a-z0-9-]+$')][string]$RunTag
)
$ErrorActionPreference = 'Stop'
$researchDir = Split-Path $PSScriptRoot
$freezePath = Join-Path $researchDir 'product-native-freeze-v16.json'
$frozen = Get-Content -LiteralPath $freezePath -Raw | ConvertFrom-Json
$appConfig = Join-Path $PSScriptRoot 'v16-core.toml'
foreach ($source in $frozen.source_files.PSObject.Properties) {
    if ((Get-FileHash -LiteralPath $source.Name).Hash.ToLowerInvariant() -ne $source.Value) { throw "Source differs from freeze: $($source.Name)" }
}
foreach ($item in @($frozen.binary, $frozen.pdfium, $frozen.fixture)) {
    if ((Get-FileHash -LiteralPath $item.path).Hash.ToLowerInvariant() -ne $item.sha256) { throw "Artifact differs from freeze: $($item.path)" }
}
if (Get-Process -Name ferrostage,cargo,rustc -ErrorAction SilentlyContinue) { throw 'App or build process is still running' }
$stdout = Join-Path $PSScriptRoot "$RunTag.stdout.log"
$stderr = Join-Path $PSScriptRoot "$RunTag.stderr.log"
if ((Test-Path -LiteralPath $stdout) -or (Test-Path -LiteralPath $stderr)) { throw 'Use a fresh run tag' }
$appArguments = @('--config', $appConfig, '--pdfium-library', $frozen.pdfium.path, $frozen.fixture.path)
$quotedArguments = $appArguments | ForEach-Object { '"{0}"' -f $_ }
$nativeRunLease = [System.IO.File]::Open((Join-Path $PSScriptRoot 'native-run.lock'), 'OpenOrCreate', 'ReadWrite', 'None')
$env:RUST_BACKTRACE = 'full'
$env:RUST_LOG = 'info,ferrostage_ui=debug'
try {
    Copy-Item -LiteralPath $appConfig -Destination (Join-Path $PSScriptRoot "$RunTag.config-before.toml")
    # Visible application is explicitly required for this authorized native test.
    $nativeProcess = Start-Process -FilePath $frozen.binary.path -ArgumentList $quotedArguments -WorkingDirectory (Get-Location).Path -WindowStyle Normal -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
    @{ utc=[DateTime]::UtcNow.ToString('o'); pid=$nativeProcess.Id; exe=$frozen.binary.path; exe_sha256=$frozen.binary.sha256; source_sha256=$frozen.source_snapshot_sha256; freeze=$freezePath; freeze_sha256=(Get-FileHash $freezePath).Hash; config=$appConfig; config_sha256=(Get-FileHash $appConfig).Hash; pdf_sha256=$frozen.fixture.sha256; dll_sha256=$frozen.pdfium.sha256; argv=$appArguments; rust_backtrace=$env:RUST_BACKTRACE; rust_log=$env:RUST_LOG } | ConvertTo-Json | Tee-Object -FilePath (Join-Path $PSScriptRoot "$RunTag.launch.json")
    $nativeProcess.WaitForExit()
    Copy-Item -LiteralPath $appConfig -Destination (Join-Path $PSScriptRoot "$RunTag.config-after.toml")
    @{ utc=[DateTime]::UtcNow.ToString('o'); pid=$nativeProcess.Id; exit_code=$nativeProcess.ExitCode } | ConvertTo-Json | Tee-Object -FilePath (Join-Path $PSScriptRoot "$RunTag.exit.json")
    $nativeExitCode = $nativeProcess.ExitCode
} finally {
    $nativeRunLease.Dispose()
}
exit $nativeExitCode
