param([switch]$Mp4, [switch]$AudioOnly)
$ErrorActionPreference = 'Stop'
$clipRoot = Split-Path -Parent $PSCommandPath
$clipRuntime = Join-Path $clipRoot 'minimal-runtime'
$clipExtension = if ($AudioOnly) { 'ogg' } elseif ($Mp4) { 'mp4' } else { 'webm' }
$clipPath = Join-Path $clipRoot "synthetic-ball-tone-12s.$clipExtension"
$clipPipelinePath = $clipPath.Replace('\', '/')
$clipArgs = @('-e','videotestsrc','num-buffers=360','pattern=ball','!',
    'video/x-raw,width=320,height=180,framerate=30/1','!','vp8enc','deadline=1','!','queue','!','mux.',
    'audiotestsrc','num-buffers=1200','samplesperbuffer=480','wave=sine','volume=0.05','!',
    'audio/x-raw,rate=48000,channels=1','!','audioconvert','!','vorbisenc','!','queue','!','mux.',
    'matroskamux','name=mux','!','filesink',"location=$clipPipelinePath")
if ($Mp4) {
    $clipArgs = @('-e','videotestsrc','num-buffers=360','pattern=ball','!',
        'video/x-raw,width=320,height=180,framerate=30/1','!','videoconvertscale','!',
        'video/x-raw,format=NV12','!','mfh264device5enc','!','h264parse','!','queue','!','mux.',
        'audiotestsrc','num-buffers=1200','samplesperbuffer=480','wave=sine','volume=0.05','!',
        'audio/x-raw,rate=48000,channels=2','!','audioconvert','!','mfaacenc','!','queue','!','mux.',
        'mp4mux','name=mux','!','filesink',"location=$clipPipelinePath")
}
if ($AudioOnly) {
    $clipArgs = @('-e','audiotestsrc','num-buffers=1200','samplesperbuffer=480','wave=sine','volume=0.05','!',
        'audio/x-raw,rate=48000,channels=1','!','audioconvert','!','vorbisenc','!','oggmux','!','filesink',"location=$clipPipelinePath")
}
$start = [System.Diagnostics.ProcessStartInfo]::new()
$start.FileName = Join-Path $clipRuntime 'bin/gst-launch-1.0.exe'
foreach ($arg in $clipArgs) { $start.ArgumentList.Add($arg) }
$start.WorkingDirectory = Join-Path $clipRuntime 'bin'
$start.UseShellExecute = $false
$start.CreateNoWindow = $true
$start.RedirectStandardOutput = $true
$start.RedirectStandardError = $true
$start.Environment['PATH'] = (Join-Path $clipRuntime 'bin') + ';' + (Join-Path $env:WINDIR 'System32') + ';' + $env:WINDIR
$start.Environment['GST_PLUGIN_PATH_1_0'] = Join-Path $clipRuntime 'lib/gstreamer-1.0'
$start.Environment['GST_PLUGIN_SYSTEM_PATH_1_0'] = Join-Path $clipRuntime 'lib/gstreamer-1.0'
$start.Environment['GST_PLUGIN_SCANNER_1_0'] = Join-Path $clipRuntime 'libexec/gstreamer-1.0/gst-plugin-scanner.exe'
$start.Environment['GST_REGISTRY_1_0'] = Join-Path $clipRuntime 'research-registry.bin'
$process = [System.Diagnostics.Process]::Start($start)
$stdout = $process.StandardOutput.ReadToEndAsync()
$stderr = $process.StandardError.ReadToEndAsync()
$finished = $process.WaitForExit(30000)
if (-not $finished) { $process.Kill($true); $process.WaitForExit() }
$result = [pscustomobject]@{pid=$process.Id;arguments=$clipArgs;timeout=(-not $finished);exit_code=$process.ExitCode;stdout=$stdout.GetAwaiter().GetResult();stderr=$stderr.GetAwaiter().GetResult()}
$result | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $clipRoot "generate-$clipExtension-clip-receipt.json")
$process.Dispose()
if ($result.exit_code -ne 0 -or $result.timeout) { throw ($result | ConvertTo-Json) }
$clipIdentity = [pscustomobject]@{path=$clipPath;bytes=(Get-Item -LiteralPath $clipPath).Length;sha256=(Get-FileHash -LiteralPath $clipPath -Algorithm SHA256).Hash.ToLowerInvariant()}
$clipIdentity | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $clipRoot "synthetic-$clipExtension-clip-identity.json")
$start.ArgumentList.Clear()
$decodeArgs = @('-v','filesrc',"location=$clipPipelinePath",'!','decodebin','name=d',
    'd.','!','queue','!','videoconvertscale','!','video/x-raw,format=BGRA','!','fakesink','sync=false',
    'd.','!','queue','!','audioconvert','!','fakesink','sync=false')
if ($AudioOnly) {
    $decodeArgs = @('-v','filesrc',"location=$clipPipelinePath",'!','decodebin','!','audioconvert','!','fakesink','sync=false')
}
foreach ($arg in $decodeArgs) { $start.ArgumentList.Add($arg) }
$process = [System.Diagnostics.Process]::Start($start)
$stdout = $process.StandardOutput.ReadToEndAsync()
$stderr = $process.StandardError.ReadToEndAsync()
$finished = $process.WaitForExit(15000)
if (-not $finished) { $process.Kill($true); $process.WaitForExit() }
$result = [pscustomobject]@{pid=$process.Id;arguments=$decodeArgs;timeout=(-not $finished);exit_code=$process.ExitCode;stdout=$stdout.GetAwaiter().GetResult();stderr=$stderr.GetAwaiter().GetResult()}
$result | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $clipRoot "decode-$clipExtension-clip-receipt.json")
$process.Dispose()
if ($result.exit_code -ne 0 -or $result.timeout -or ((-not $AudioOnly) -and $result.stdout -notmatch 'format=\(string\)BGRA') -or $result.stdout -notmatch 'Got EOS') { throw ($result | ConvertTo-Json) }
$clipIdentity | ConvertTo-Json
Write-Output 'Native finite clip decode reached EOS; audio routed only to fakesink, with no video sink for AudioOnly.'
