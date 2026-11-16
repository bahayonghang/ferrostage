param(
    [Parameter(Mandatory=$true)][string]$ExpectedExeSha256,
    [Parameter(Mandatory=$true)][string]$Label,
    [Parameter(Mandatory=$true)][string]$Pdf,
    [ValidateSet('complete','absent','partial')][string]$Runtime = 'complete',
    [string]$ConfigName = 'settings.toml'
)
$ErrorActionPreference = 'Stop'
if ($Label -notmatch '^[a-z0-9-]+$' -or $ConfigName -notmatch '^[a-z0-9-]+\.toml$') { throw 'Invalid evidence/config name.' }
$smokeEvidence = Split-Path -Parent $PSCommandPath
$smokeRepo = (Resolve-Path (Join-Path $smokeEvidence '../../../../..')).Path
$smokeBin = Join-Path $smokeRepo "cache/consolidated-smoke/$Runtime"
$smokeExe = Join-Path $smokeBin 'ferrostage.exe'
$smokeHelper = Join-Path $smokeBin 'ferrostage-media-helper.exe'
$smokeDll = Join-Path $smokeRepo 'cache/pdfium-7881-win-x64/bin/pdfium.dll'
$smokeConfig = Join-Path $smokeEvidence $ConfigName
$smokeRegistryDirectory = Join-Path $smokeRepo 'cache/consolidated-smoke/registries'
New-Item -ItemType Directory -Force -Path $smokeRegistryDirectory | Out-Null
$smokeRegistry = Join-Path $smokeRegistryDirectory "$Label.bin"
if (Test-Path -LiteralPath $smokeRegistry) { throw 'Use a fresh run label for an isolated plugin registry.' }
$smokePdf = (Resolve-Path -LiteralPath $Pdf).Path
if (-not $smokePdf.StartsWith($smokeRepo + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Smoke input must be a repository-owned fixture.' }
if (Test-Path -LiteralPath (Join-Path $smokeEvidence "$Label-launch.json")) { throw 'Do not overwrite an existing run receipt.' }
$smokeIdentities = foreach ($path in @($smokeExe,$smokeHelper,$smokePdf,$smokeDll,$smokeConfig)) {
    if (Test-Path -LiteralPath $path) {
        [pscustomobject]@{path=$path;bytes=(Get-Item -LiteralPath $path).Length;sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()}
    }
}
if ($smokeIdentities[0].sha256 -ne $ExpectedExeSha256.ToLowerInvariant()) { throw 'Executable differs from the independently checked feature build.' }
if ((Get-FileHash -LiteralPath $smokeDll -Algorithm SHA256).Hash -ne '79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118') { throw 'PDFium identity changed.' }
$smokeRuntimeFiles = if (Test-Path -LiteralPath (Join-Path $smokeBin 'media-runtime')) {
    @(Get-ChildItem -LiteralPath (Join-Path $smokeBin 'media-runtime') -File -Recurse | ForEach-Object {
        [pscustomobject]@{path=[IO.Path]::GetRelativePath($smokeBin,$_.FullName);bytes=$_.Length;sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}
    })
} else { @() }
$smokeArgs = @('--config',$smokeConfig,'--pdfium-library',$smokeDll,$smokePdf)
$smokeStart = [System.Diagnostics.ProcessStartInfo]::new()
$smokeStart.FileName = $smokeExe
$smokeStart.WorkingDirectory = $smokeRepo
foreach ($arg in $smokeArgs) { $smokeStart.ArgumentList.Add($arg) }
$smokeStart.UseShellExecute = $false
$smokeStart.CreateNoWindow = $true
$smokeStart.RedirectStandardOutput = $true
$smokeStart.RedirectStandardError = $true
$smokeStart.Environment['RUST_BACKTRACE'] = 'full'
$smokeStart.Environment['RUST_LOG'] = 'info,ferrostage_ui=debug,ferrostage_platform=debug,ferrostage_media=debug'
$smokeStart.Environment['GST_REGISTRY_1_0'] = $smokeRegistry
$smokeStarted = [DateTime]::UtcNow.ToString('o')
$smokeProcess = [System.Diagnostics.Process]::Start($smokeStart)
$smokeOut = $smokeProcess.StandardOutput.ReadToEndAsync()
$smokeErr = $smokeProcess.StandardError.ReadToEndAsync()
$smokeLaunch = [pscustomobject]@{started_utc=$smokeStarted;pid=$smokeProcess.Id;exe=$smokeExe;arguments=$smokeArgs;inputs=$smokeIdentities;runtime=$Runtime;runtime_files=$smokeRuntimeFiles;plugin_registry=$smokeRegistry;plugin_registry_initially_absent=$true;audio_sink='product default, no silent-test environment'}
$smokeLaunch | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $smokeEvidence "$Label-launch.json")
[pscustomobject]@{pid=$smokeProcess.Id;exe=$smokeExe;pdf=$smokePdf;runtime=$Runtime;label=$Label} | ConvertTo-Json
$smokeProcess.WaitForExit()
$smokeOut.GetAwaiter().GetResult() | Set-Content -LiteralPath (Join-Path $smokeEvidence "$Label-stdout.log")
$smokeErr.GetAwaiter().GetResult() | Set-Content -LiteralPath (Join-Path $smokeEvidence "$Label-stderr.log")
$smokeExit = [pscustomobject]@{finished_utc=[DateTime]::UtcNow.ToString('o');pid=$smokeProcess.Id;exit_code=$smokeProcess.ExitCode;exe_sha256=(Get-FileHash -LiteralPath $smokeExe -Algorithm SHA256).Hash.ToLowerInvariant();pdf_sha256=(Get-FileHash -LiteralPath $smokePdf -Algorithm SHA256).Hash.ToLowerInvariant();config_sha256=(Get-FileHash -LiteralPath $smokeConfig -Algorithm SHA256).Hash.ToLowerInvariant()}
$smokeExit | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $smokeEvidence "$Label-exit.json")
$smokeExit | ConvertTo-Json
$smokeProcess.Dispose()
