param(
    [Parameter(Mandatory)][ValidateSet('v8-light','v8-dark','v8-system','v8-en','v8-future')][string]$Profile,
    [Parameter(Mandatory)][ValidatePattern('^[a-z0-9-]+$')][string]$RunTag,
    [Parameter(Mandatory)][ValidatePattern('^product-native-freeze-v[0-9]+\.json$')][string]$Freeze,
    [string]$Pdf
)
$ErrorActionPreference = 'Stop'
$researchDir = Split-Path $PSScriptRoot
$freezePath = Join-Path $researchDir $Freeze
$frozen = Get-Content -LiteralPath $freezePath -Raw | ConvertFrom-Json
$appExe = $frozen.binary.path
$appDll = $frozen.pdfium.path
$appConfig = Join-Path $PSScriptRoot "$Profile.toml"
if ((Get-FileHash -LiteralPath $appExe).Hash.ToLowerInvariant() -ne $frozen.binary.sha256) { throw 'Executable differs from freeze' }
if ((Get-FileHash -LiteralPath $appDll).Hash.ToLowerInvariant() -ne $frozen.pdfium.sha256) { throw 'PDFium differs from freeze' }
foreach ($source in $frozen.source_files.PSObject.Properties) {
    if ((Get-FileHash -LiteralPath $source.Name).Hash.ToLowerInvariant() -ne $source.Value) { throw "Source differs from freeze: $($source.Name)" }
}
if (Get-Process -Name ferrostage -ErrorAction SilentlyContinue) { throw 'An existing FerroStage process is still running' }
$stdout = Join-Path $PSScriptRoot "$RunTag.stdout.log"
$stderr = Join-Path $PSScriptRoot "$RunTag.stderr.log"
if ((Test-Path -LiteralPath $stdout) -or (Test-Path -LiteralPath $stderr)) { throw 'Use a new run tag; preserve earlier logs' }
$appArguments = @('--config', $appConfig, '--pdfium-library', $appDll)
if ($Pdf) { $appArguments += (Resolve-Path -LiteralPath $Pdf).Path }
$quotedArguments = $appArguments | ForEach-Object { '"{0}"' -f $_ }
# One live runner owns the desktop acceptance session. An interrupted UI close
# must not allow the next profile to start before this process actually exits.
$nativeRunLease = [System.IO.File]::Open((Join-Path $PSScriptRoot 'native-run.lock'), 'OpenOrCreate', 'ReadWrite', 'None')
try {
    $configBefore = [System.IO.File]::ReadAllBytes($appConfig)
    [System.IO.File]::WriteAllBytes((Join-Path $PSScriptRoot "$RunTag.config-before.toml"), $configBefore)
    $configBeforeHash = [Convert]::ToHexString([System.Security.Cryptography.SHA256]::HashData($configBefore))
    $nativeProcess = Start-Process -FilePath $appExe -ArgumentList $quotedArguments -WorkingDirectory (Get-Location).Path -WindowStyle Normal -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
    @{ utc=[DateTime]::UtcNow.ToString('o'); pid=$nativeProcess.Id; exe=$appExe; exe_sha256=$frozen.binary.sha256; source_sha256=$frozen.source_snapshot_sha256; freeze=$freezePath; config=$appConfig; config_sha256=$configBeforeHash; argv=$appArguments; window_style='Normal' } | ConvertTo-Json | Tee-Object -FilePath (Join-Path $PSScriptRoot "$RunTag.launch.json")
    $nativeProcess.WaitForExit()
    Copy-Item -LiteralPath $appConfig -Destination (Join-Path $PSScriptRoot "$RunTag.config-after.toml")
    @{ utc=[DateTime]::UtcNow.ToString('o'); pid=$nativeProcess.Id; exit_code=$nativeProcess.ExitCode } | ConvertTo-Json | Tee-Object -FilePath (Join-Path $PSScriptRoot "$RunTag.exit.json")
    $nativeExitCode = $nativeProcess.ExitCode
} finally {
    $nativeRunLease.Dispose()
}
exit $nativeExitCode
