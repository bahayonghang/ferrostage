param([Parameter(Mandatory=$true)][string]$Label)
$ErrorActionPreference = 'Stop'
if ($Label -notmatch '^[a-z0-9-]+$') { throw 'Invalid label' }
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../../../../..')).Path
$exe = Join-Path $repo 'target/debug/ferrostage.exe'
$pdf = Join-Path $repo 'fixtures/pdf/perf-100.pdf'
$dll = Join-Path $repo 'target/debug/pdfium.dll'
$config = Join-Path $PSScriptRoot "$Label.toml"
$info = [Diagnostics.ProcessStartInfo]::new()
$info.FileName = $exe
$info.WorkingDirectory = $repo
$info.UseShellExecute = $false
$info.CreateNoWindow = $true
$info.RedirectStandardOutput = $true
$info.RedirectStandardError = $true
$info.Environment.Remove('RUST_LOG') | Out-Null
foreach ($arg in @('--config',$config,'--pdfium-library',$dll,$pdf)) { $info.ArgumentList.Add($arg) }
$hashes = @($exe,$pdf,$dll,$config) | ForEach-Object { Get-FileHash -LiteralPath $_ }
$process = [Diagnostics.Process]::Start($info)
$stdout = $process.StandardOutput.ReadToEndAsync()
$stderr = $process.StandardError.ReadToEndAsync()
@{pid=$process.Id;started=[DateTime]::UtcNow.ToString('o');inputs=$hashes} | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $PSScriptRoot "$Label-launch.json")
Write-Output "Started $Label PID $($process.Id)"
$process.WaitForExit()
$stdout.GetAwaiter().GetResult() | Set-Content (Join-Path $PSScriptRoot "$Label-stdout.log")
$stderr.GetAwaiter().GetResult() | Set-Content (Join-Path $PSScriptRoot "$Label-stderr.log")
@{exit_code=$process.ExitCode;ended=[DateTime]::UtcNow.ToString('o')} | ConvertTo-Json | Set-Content (Join-Path $PSScriptRoot "$Label-exit.json")
Write-Output "Exited $($process.ExitCode)"
