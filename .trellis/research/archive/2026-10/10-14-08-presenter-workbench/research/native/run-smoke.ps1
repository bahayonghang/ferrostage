param(
    [Parameter(Mandatory=$true)][string]$ExpectedExeSha256,
    [string]$Label = 'run01'
)
$ErrorActionPreference = 'Stop'
$smokeEvidence = Split-Path -Parent $PSCommandPath
$smokeRepo = (Resolve-Path (Join-Path $smokeEvidence '../../../../..')).Path
$smokeExe = Join-Path $smokeRepo 'target/debug/ferrostage.exe'
$smokePdf = Join-Path $smokeRepo 'fixtures/notes-functional/pympress-notes.pdf'
$smokeDll = Join-Path $smokeRepo 'cache/pdfium-7881-win-x64/bin/pdfium.dll'
$smokeConfig = Join-Path $smokeEvidence 'workbench-smoke.toml'
$smokeIdentities = foreach ($path in @($smokeExe,$smokePdf,$smokeDll,$smokeConfig)) {
    [pscustomobject]@{path=$path;bytes=(Get-Item -LiteralPath $path).Length;sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()}
}
if ($smokeIdentities[0].sha256 -ne $ExpectedExeSha256.ToLowerInvariant()) { throw 'Executable differs from the independently checked feature build.' }
if ($smokeIdentities[1].sha256 -ne 'e939e35f931c184fd93b2e9b95e560db73ba6a6f6dfccbb0963262f0b359dfeb') { throw 'PDF fixture identity changed.' }
if ($smokeIdentities[2].sha256 -ne '79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118') { throw 'PDFium identity changed.' }
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
$smokeStart.Environment['RUST_LOG'] = 'info'
$smokeStarted = [DateTime]::UtcNow.ToString('o')
$smokeProcess = [System.Diagnostics.Process]::Start($smokeStart)
$smokeOut = $smokeProcess.StandardOutput.ReadToEndAsync()
$smokeErr = $smokeProcess.StandardError.ReadToEndAsync()
$smokeLaunch = [pscustomobject]@{started_utc=$smokeStarted;pid=$smokeProcess.Id;exe=$smokeExe;arguments=$smokeArgs;inputs=$smokeIdentities}
$smokeLaunch | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $smokeEvidence "$Label-launch.json")
$smokeLaunch | ConvertTo-Json -Depth 5
$smokeProcess.WaitForExit()
$smokeOut.GetAwaiter().GetResult() | Set-Content -LiteralPath (Join-Path $smokeEvidence "$Label-stdout.log")
$smokeErr.GetAwaiter().GetResult() | Set-Content -LiteralPath (Join-Path $smokeEvidence "$Label-stderr.log")
$smokeExit = [pscustomobject]@{finished_utc=[DateTime]::UtcNow.ToString('o');pid=$smokeProcess.Id;exit_code=$smokeProcess.ExitCode;exe_sha256=(Get-FileHash -LiteralPath $smokeExe -Algorithm SHA256).Hash.ToLowerInvariant();pdf_sha256=(Get-FileHash -LiteralPath $smokePdf -Algorithm SHA256).Hash.ToLowerInvariant()}
$smokeExit | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $smokeEvidence "$Label-exit.json")
$smokeExit | ConvertTo-Json
$smokeProcess.Dispose()
