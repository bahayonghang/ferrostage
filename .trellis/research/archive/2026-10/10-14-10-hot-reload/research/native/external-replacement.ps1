param(
    [Parameter(Mandatory)]
    [ValidateSet('Initialize','Chunked','Rename','Truncate','DeleteRecreate')]
    [string]$Mode,
    [ValidateRange(10,250)][int]$ChunkDelayMs = 50
)
$ErrorActionPreference = 'Stop'
# Test producer only: the watched file is always this script's own fixture copy.
# This is not an application PDF-write path and never accepts a user target path.
$reloadRun = Join-Path $PSScriptRoot 'runtime'
New-Item -ItemType Directory -Path $reloadRun -Force | Out-Null
if ((Get-Item -LiteralPath $reloadRun).Attributes -band [IO.FileAttributes]::ReparsePoint) {
    throw 'Refusing a linked test runtime directory.'
}
$reloadTarget = Join-Path $reloadRun 'watched.pdf'
if ((Test-Path -LiteralPath $reloadTarget) -and ((Get-Item -LiteralPath $reloadTarget).Attributes -band [IO.FileAttributes]::ReparsePoint)) {
    throw 'Refusing a linked watched fixture.'
}
$reloadSource = Join-Path $PSScriptRoot ('inputs/' + $(switch ($Mode) {
    'Initialize' { 'reload-blue.pdf' }
    'Chunked' { 'reload-green-chunked.pdf' }
    default { 'reload-orange.pdf' }
}))
$reloadBytes = [IO.File]::ReadAllBytes($reloadSource)
$reloadStarted = [DateTime]::UtcNow.ToString('o')
switch ($Mode) {
    'Initialize' { [IO.File]::WriteAllBytes($reloadTarget, $reloadBytes) }
    'Chunked' {
        $reloadStream = [IO.FileStream]::new($reloadTarget, [IO.FileMode]::Create, [IO.FileAccess]::Write, [IO.FileShare]::ReadWrite)
        try {
            for ($offset = 0; $offset -lt $reloadBytes.Length; $offset += 65536) {
                $reloadStream.Write($reloadBytes, $offset, [Math]::Min(65536, $reloadBytes.Length - $offset))
                $reloadStream.Flush($true)
                Start-Sleep -Milliseconds $ChunkDelayMs
            }
        } finally { $reloadStream.Dispose() }
    }
    'Rename' {
        $reloadSibling = Join-Path $reloadRun ('producer-' + [guid]::NewGuid().ToString('N') + '.tmp')
        [IO.File]::WriteAllBytes($reloadSibling, $reloadBytes)
        [IO.File]::Move($reloadSibling, $reloadTarget, $true)
    }
    'Truncate' { [IO.File]::WriteAllBytes($reloadTarget, $reloadBytes[0..([int]($reloadBytes.Length / 2))]) }
    'DeleteRecreate' {
        [IO.File]::Delete($reloadTarget)
        Start-Sleep -Milliseconds 500
        [IO.File]::WriteAllBytes($reloadTarget, $reloadBytes)
    }
}
$reloadReceipt = [ordered]@{
    mode = $Mode
    startedUtc = $reloadStarted
    completedUtc = [DateTime]::UtcNow.ToString('o')
    target = $reloadTarget
    source = $reloadSource
    bytes = (Get-Item -LiteralPath $reloadTarget).Length
    chunk_delay_ms = if ($Mode -eq 'Chunked') { $ChunkDelayMs } else { $null }
    targetSha256 = (Get-FileHash -LiteralPath $reloadTarget -Algorithm SHA256).Hash
    scope = 'External fixture producer only; no application acceptance inferred.'
}
$reloadReceipt | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $reloadRun ('producer-' + [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfff') + '.json')) -Encoding utf8
$reloadReceipt | ConvertTo-Json
