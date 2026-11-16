param([string]$Label = ('rust-decode-' + [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfff')), [switch]$Supervisor, [switch]$MissingRuntime, [string]$Source = 'synthetic-ball-tone-12s.webm')
$ErrorActionPreference = 'Stop'
$probeRoot = Split-Path -Parent $PSCommandPath
$probeRuntime = Join-Path $probeRoot 'minimal-runtime'
$probeExe = Join-Path $probeRoot 'rust-probe/target/debug/ferrostage-media-research-probe.exe'
$start = [System.Diagnostics.ProcessStartInfo]::new()
if ($Supervisor) {
    if ($MissingRuntime) { $probeRuntime = Join-Path $probeRoot 'missing-runtime' }
    $start.ArgumentList.Add($probeRuntime)
    $start.ArgumentList.Add($probeExe)
    $probeExe = Join-Path $probeRoot 'rust-probe/target/debug/supervisor.exe'
}
$start.FileName = $probeExe
$start.ArgumentList.Add((Join-Path $probeRoot $Source))
$start.WorkingDirectory = $probeRoot
$start.UseShellExecute = $false
$start.CreateNoWindow = $true
$start.RedirectStandardOutput = $true
$start.RedirectStandardError = $true
$start.Environment['PATH'] = (Join-Path $probeRuntime 'bin') + ';' + (Join-Path $env:WINDIR 'System32') + ';' + $env:WINDIR
$start.Environment['GST_PLUGIN_PATH_1_0'] = Join-Path $probeRuntime 'lib/gstreamer-1.0'
$start.Environment['GST_PLUGIN_SYSTEM_PATH_1_0'] = Join-Path $probeRuntime 'lib/gstreamer-1.0'
$start.Environment['GST_PLUGIN_SCANNER_1_0'] = Join-Path $probeRuntime 'libexec/gstreamer-1.0/gst-plugin-scanner.exe'
$start.Environment['GST_REGISTRY_1_0'] = Join-Path $probeRuntime 'research-registry.bin'
$process = [System.Diagnostics.Process]::Start($start)
$stdout = $process.StandardOutput.ReadToEndAsync()
$stderr = $process.StandardError.ReadToEndAsync()
$finished = $process.WaitForExit(30000)
if (-not $finished) { $process.Kill($true); $process.WaitForExit() }
$result = [pscustomobject]@{utc=[DateTime]::UtcNow.ToString('o');pid=$process.Id;timeout=(-not $finished);exit_code=$process.ExitCode;exe_sha256=(Get-FileHash $probeExe).Hash;stdout=$stdout.GetAwaiter().GetResult();stderr=$stderr.GetAwaiter().GetResult()}
$result | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $probeRoot "$Label-receipt.json")
$result | ConvertTo-Json
$process.Dispose()
