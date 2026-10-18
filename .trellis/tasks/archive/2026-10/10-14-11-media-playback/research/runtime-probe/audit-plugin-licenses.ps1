$ErrorActionPreference = 'Stop'
$auditRoot = Split-Path -Parent $PSCommandPath
$auditRuntime = Join-Path $auditRoot 'minimal-runtime'
$results = @()
foreach ($plugin in Get-ChildItem -LiteralPath (Join-Path $auditRuntime 'lib/gstreamer-1.0') -Filter '*.dll') {
    $start = [System.Diagnostics.ProcessStartInfo]::new()
    $start.FileName = Join-Path $auditRuntime 'bin/gst-inspect-1.0.exe'
    $start.ArgumentList.Add($plugin.FullName)
    $start.WorkingDirectory = $auditRoot
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.Environment['PATH'] = (Join-Path $auditRuntime 'bin') + ';' + (Join-Path $env:WINDIR 'System32') + ';' + $env:WINDIR
    $start.Environment['GST_PLUGIN_PATH_1_0'] = Join-Path $auditRuntime 'lib/gstreamer-1.0'
    $start.Environment['GST_PLUGIN_SYSTEM_PATH_1_0'] = Join-Path $auditRuntime 'lib/gstreamer-1.0'
    $start.Environment['GST_PLUGIN_SCANNER_1_0'] = Join-Path $auditRuntime 'libexec/gstreamer-1.0/gst-plugin-scanner.exe'
    $start.Environment['GST_REGISTRY_1_0'] = Join-Path $auditRuntime 'research-registry.bin'
    $process = [System.Diagnostics.Process]::Start($start)
    $stdout = $process.StandardOutput.ReadToEndAsync()
    $stderr = $process.StandardError.ReadToEndAsync()
    $finished = $process.WaitForExit(5000)
    if (-not $finished) { $process.Kill($true); $process.WaitForExit() }
    $out = $stdout.GetAwaiter().GetResult()
    $result = [pscustomobject]@{file=$plugin.Name;sha256=(Get-FileHash -LiteralPath $plugin.FullName).Hash.ToLowerInvariant();pid=$process.Id;timeout=(-not $finished);exit_code=$process.ExitCode;stdout=$out;stderr=$stderr.GetAwaiter().GetResult();license=(@($out -split "`n" | Where-Object { $_ -match '^\s+License\s' }) -join ' ').Trim()}
    $process.Dispose()
    $results += $result
}
$results | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $auditRoot 'all-selected-plugin-licenses.json')
$results | Select-Object file,exit_code,timeout,license | Format-Table -AutoSize
if (@($results | Where-Object { $_.exit_code -ne 0 -or $_.timeout -or $_.license -notmatch '^License\s+LGPL\s*$' }).Count -gt 0) { throw 'Selected plugin audit failed; inspect original receipt.' }
