#requires -Version 7.0
param(
    [Parameter(Mandatory=$true)][string]$Label,
    [string]$Pdf = ''
)
$ErrorActionPreference = 'Stop'
if ($Label -notmatch '^[a-z0-9-]+$') { throw 'Invalid evidence label' }
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../../../../..')).Path
$exe = Join-Path $repo 'target/debug/ferrostage.exe'
$config = Join-Path $PSScriptRoot "$Label.toml"
$dll = Join-Path $repo 'target/debug/pdfium.dll'
$sourceFiles = @(
    'DESIGN.md',
    'crates/ferrostage-ui/src/presenter.rs',
    'crates/ferrostage-ui/src/presenter/home_view.rs',
    'crates/ferrostage-ui/src/theme.rs',
    'crates/ferrostage-ui/src/shell.rs',
    'assets/locales/zh-CN/app.toml',
    'assets/locales/en-US/app.toml'
) | ForEach-Object { Join-Path $repo $_ }
$inputs = @($exe, $config, $dll) + $sourceFiles
if ($Pdf) { $inputs += (Resolve-Path -LiteralPath $Pdf).Path }
$info = [Diagnostics.ProcessStartInfo]::new()
$info.FileName = $exe
$info.WorkingDirectory = $repo
$info.UseShellExecute = $false
$info.CreateNoWindow = $true
$info.RedirectStandardOutput = $true
$info.RedirectStandardError = $true
$info.Environment.Remove('RUST_LOG') | Out-Null
foreach ($argument in @('--config', $config, '--pdfium-library', $dll)) {
    $info.ArgumentList.Add($argument)
}
if ($Pdf) { $info.ArgumentList.Add((Resolve-Path -LiteralPath $Pdf).Path) }
$hashes = $inputs | ForEach-Object { Get-FileHash -LiteralPath $_ }
$process = [Diagnostics.Process]::Start($info)
$stdout = $process.StandardOutput.ReadToEndAsync()
$stderr = $process.StandardError.ReadToEndAsync()
@{
    pid = $process.Id
    started_utc = [DateTime]::UtcNow.ToString('o')
    source_head = (git -C $repo rev-parse HEAD)
    branch = (git -C $repo branch --show-current)
    inputs = $hashes
} | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $PSScriptRoot "$Label-launch.json")
Write-Output "Native $Label PID $($process.Id)"
$process.WaitForExit()
$stdout.GetAwaiter().GetResult() | Set-Content (Join-Path $PSScriptRoot "$Label-stdout.log")
$stderr.GetAwaiter().GetResult() | Set-Content (Join-Path $PSScriptRoot "$Label-stderr.log")
@{
    exit_code = $process.ExitCode
    ended_utc = [DateTime]::UtcNow.ToString('o')
} | ConvertTo-Json | Set-Content (Join-Path $PSScriptRoot "$Label-exit.json")
Write-Output "Native $Label exited $($process.ExitCode)"
