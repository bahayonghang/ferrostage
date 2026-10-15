param(
    [Parameter(Mandatory)][ValidateSet('a-dark','a-conflicting-keys')][string]$Profile,
    [Parameter(Mandatory)][ValidatePattern('^[a-z0-9-]+$')][string]$RunTag
)
$ErrorActionPreference = 'Stop'
$settingsPath = Join-Path $PSScriptRoot "$Profile.toml"
$releasePath = Join-Path $PSScriptRoot "$RunTag.release"
$receiptPath = Join-Path $PSScriptRoot "$RunTag.lock.json"
if ((Test-Path -LiteralPath $releasePath) -or (Test-Path -LiteralPath $receiptPath)) {
    throw 'Use a new run tag; preserve earlier evidence'
}
# Only this task's isolated config is locked. Reads remain available, while
# Windows replacement fails until the handle is released. No file bytes change.
$configHash = (Get-FileHash -LiteralPath $settingsPath).Hash
$heldSettings = [System.IO.File]::Open($settingsPath, 'Open', 'Read', 'Read')
try {
    @{ utc=[DateTime]::UtcNow.ToString('o'); config=$settingsPath; sha256=$configHash; status='LOCK_HELD'; release_file=$releasePath } |
        ConvertTo-Json | Tee-Object -FilePath $receiptPath
    $lockDeadline = [DateTime]::UtcNow.AddMinutes(10)
    while (!(Test-Path -LiteralPath $releasePath) -and [DateTime]::UtcNow -lt $lockDeadline) {
        Start-Sleep -Milliseconds 200
    }
    $lockResult = if (Test-Path -LiteralPath $releasePath) { 'RELEASE_REQUESTED' } else { 'TIMEOUT_RELEASED' }
} finally {
    $heldSettings.Dispose()
}
@{ utc=[DateTime]::UtcNow.ToString('o'); config=$settingsPath; status=$lockResult } |
    ConvertTo-Json | Tee-Object -FilePath (Join-Path $PSScriptRoot "$RunTag.unlock.json")
