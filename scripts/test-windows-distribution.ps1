# Focused artifact regressions. -Lifecycle explicitly exercises this user's installation.
param([switch]$Lifecycle)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'windows-distribution.ps1') -LibraryOnly

function Assert-Distribution([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Assert-DistributionFailure([scriptblock]$Operation, [string]$Message) {
    $failed = $false
    try { & $Operation | Out-Null } catch { $failed = $true; Write-Host "[PASS] $Message ($($_.Exception.Message))" }
    Assert-Distribution $failed "Expected failure: $Message"
}

$metadata = Get-DistributionMetadata
$receipt = Get-Content -LiteralPath (Join-Path $metadata.Dist 'release.json') -Raw | ConvertFrom-Json
$payload = $receipt.payload
Test-DistributionPayload $payload | Out-Null
Test-DistributionChecksums $metadata.Dist $receipt.checksums
$work = Join-Path $metadata.Dist 'distribution-tests'
Reset-DistributionDirectory $work $metadata.Dist
$copy = Join-Path $work 'payload'
Copy-Item -LiteralPath $payload -Destination $copy -Recurse

Remove-Item -LiteralPath (Join-Path $copy 'licenses/pdfium.txt')
Assert-DistributionFailure { Test-DistributionPayload $copy } 'Missing runtime notice is rejected'
Copy-Item -LiteralPath (Join-Path $payload 'licenses/pdfium.txt') -Destination (Join-Path $copy 'licenses/pdfium.txt')
[IO.File]::AppendAllText((Join-Path $copy 'pdfium.dll'), 'tamper')
Assert-DistributionFailure { Test-DistributionPayload $copy } 'Corrupt runtime is rejected'
$inventory = Get-Content -LiteralPath (Join-Path $copy 'inventory.json') -Raw | ConvertFrom-Json
($inventory.files | Where-Object path -eq 'pdfium.dll').sha256 = Get-DistributionHash (Join-Path $copy 'pdfium.dll')
$inventory | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $copy 'inventory.json') -Encoding UTF8
Assert-DistributionFailure { Test-DistributionPayload $copy } 'Modified inventory cannot override the pinned runtime hash'
Copy-Item -LiteralPath (Join-Path $payload 'pdfium.dll'), (Join-Path $payload 'inventory.json') -Destination $copy -Force
Set-Content -LiteralPath (Join-Path $copy 'unrelated.exe') -Value 'unrelated'
Assert-DistributionFailure { Test-DistributionPayload $copy } 'Stale or unrelated payload file is rejected'
Remove-Item -LiteralPath (Join-Path $copy 'unrelated.exe')
Set-Content -LiteralPath (Join-Path $copy 'FERROSTAGE-LICENSE-MIT.txt') -Value 'wrong license'
Assert-DistributionFailure { Test-DistributionPayload $copy } 'Corrupt application license is rejected'
Copy-Item -LiteralPath (Join-Path $payload 'FERROSTAGE-LICENSE-MIT.txt') -Destination $copy -Force
Test-DistributionPayload $copy | Out-Null
[IO.File]::AppendAllText((Join-Path $copy 'ferrostage.exe'), 'same-version stale executable')
$inventory = Get-Content -LiteralPath (Join-Path $copy 'inventory.json') -Raw | ConvertFrom-Json
($inventory.files | Where-Object path -eq 'ferrostage.exe').sha256 = Get-DistributionHash (Join-Path $copy 'ferrostage.exe')
$inventory | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $copy 'inventory.json') -Encoding UTF8
Test-DistributionPayload $copy | Out-Null
Assert-DistributionFailure { Test-DistributionInstalledPayload $payload $copy } 'Same-version different executable cannot pass installed verification'
Copy-Item -LiteralPath (Join-Path $payload 'ferrostage.exe'), (Join-Path $payload 'inventory.json') -Destination $copy -Force

$checksumCopy = Join-Path $work 'SHA256SUMS.txt'
$badChecksum = ([IO.File]::ReadAllText($receipt.checksums)) -replace '^[0-9a-f]{64}', ('0' * 64)
[IO.File]::WriteAllText($checksumCopy, $badChecksum)
Assert-DistributionFailure { Test-DistributionChecksums $metadata.Dist $checksumCopy } 'Release checksum tampering is rejected'
Assert-DistributionFailure { Invoke-DistributionTool 'powershell.exe' @('-NoProfile', '-Command', 'exit 17') 'Expected subprocess failure' } 'Native failure propagates'
$savedTarget = $env:CARGO_TARGET_DIR
try {
    $env:CARGO_TARGET_DIR = Join-Path $work 'custom-target'
    $custom = Get-DistributionMetadata
    Assert-Distribution ($custom.Dist -eq (Join-Path $env:CARGO_TARGET_DIR 'dist')) 'Cargo target override was ignored'
    Write-Host '[PASS] Cargo target-directory override is resolved from metadata'
} finally { $env:CARGO_TARGET_DIR = $savedTarget }
Assert-DistributionFailure { Reset-DistributionDirectory $metadata.Dist $metadata.Dist } 'Cleanup cannot remove distribution root'
Assert-DistributionFailure { Reset-DistributionDirectory (Join-Path $metadata.Dist '../not-owned') $metadata.Dist } 'Cleanup cannot escape distribution root'

if ($Lifecycle) {
    $installRoot = Join-Path $env:LOCALAPPDATA 'Programs\FerroStage'
    $settingsRoot = Join-Path $env:APPDATA 'ferrostage'
    $canaryName = 'distribution-test-' + [guid]::NewGuid().ToString('N') + '.txt'
    $settingsCanary = Join-Path $settingsRoot $canaryName
    $settingsBefore = @{}
    if (Test-Path -LiteralPath $settingsRoot) {
        Get-ChildItem -LiteralPath $settingsRoot -Recurse -File | ForEach-Object { $settingsBefore[$_.FullName] = Get-DistributionHash $_.FullName }
    }
    New-Item -ItemType Directory -Path $settingsRoot -Force | Out-Null
    [IO.File]::WriteAllText($settingsCanary, 'preserve settings canary')
    $compiler = Get-DistributionNsis
    $next = [version]$metadata.Version
    $nextVersion = "$($next.Major).$($next.Minor).$($next.Build + 1)"
    $inventory = Get-Content -LiteralPath (Join-Path $copy 'inventory.json') -Raw | ConvertFrom-Json
    $inventory.version = $nextVersion
    $inventory | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $copy 'inventory.json') -Encoding UTF8
    $upgrade = New-DistributionSetup $copy $nextVersion $compiler $work
    # Compile an isolated fault fixture, injecting a failure after file replacement.
    # The production template and package never contain the injected instruction.
    $failureTemplate = Join-Path $work 'failure-installer.nsi'
    $template = [IO.File]::ReadAllText((Join-Path $script:DistributionRepo 'packaging/windows/installer.nsi'))
    [IO.File]::WriteAllText($failureTemplate, $template.Replace('!insertmacro PayloadInstall', "!insertmacro PayloadInstall`r`n  Goto install_failed"))
    $failureRoot = Join-Path $work 'fault-package'
    New-Item -ItemType Directory -Path $failureRoot -Force | Out-Null
    $failureSetup = New-DistributionSetup $copy $nextVersion $compiler $failureRoot $failureTemplate
    try {
        Invoke-DistributionUninstall -Quiet
        Assert-DistributionFailure { Invoke-DistributionTool $failureSetup @('/S') 'Expected fresh installation rollback' } 'Failed fresh installation does not register success'
        Assert-Distribution (-not (Test-Path $script:UninstallKey)) 'Failed fresh install registered an application'
        foreach ($file in @(Get-DistributionFiles) + @('inventory.json', 'Uninstall.exe')) {
            Assert-Distribution (-not (Test-Path -LiteralPath (Join-Path $installRoot $file))) "Failed fresh install left owned file: $file"
        }
        Invoke-DistributionTool $receipt.setup @('/S') 'Fresh native installation' | Out-Host
        Test-DistributionPayload $installRoot -Installed | Out-Null
        Invoke-DistributionTool $receipt.setup @('/S') 'Same-version reinstall' | Out-Host
        Assert-DistributionFailure { Invoke-DistributionTool $failureSetup @('/S') 'Expected upgrade rollback' } 'Mid-install failure restores previous payload'
        Test-DistributionPayload $installRoot -Installed | Out-Null
        Assert-Distribution ((Get-ItemProperty $script:UninstallKey).DisplayVersion -eq $metadata.Version) 'Failed upgrade changed registered version'
        Assert-Distribution ((Get-DistributionHash (Join-Path $installRoot 'inventory.json')) -eq (Get-DistributionHash (Join-Path $payload 'inventory.json'))) 'Failed upgrade did not restore previous inventory'
        $unknownFile = Join-Path $installRoot $canaryName
        $pdfCanary = Join-Path $installRoot ($canaryName + '.pdf')
        [IO.File]::WriteAllText($unknownFile, 'preserve unrelated file')
        [IO.File]::WriteAllText($pdfCanary, '%PDF-1.7 preserve user document %%EOF')
        $unknownHash = Get-DistributionHash $unknownFile
        $pdfHash = Get-DistributionHash $pdfCanary
        $lock = [IO.File]::Open((Join-Path $installRoot 'ferrostage.exe'), [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::None)
        try {
            Assert-DistributionFailure { Invoke-DistributionTool $receipt.setup @('/S') 'Expected locked-file installation failure' } 'Locked executable prevents installation'
            Assert-DistributionFailure { Invoke-DistributionUninstall -Quiet } 'Locked executable prevents uninstallation'
        } finally { $lock.Dispose() }
        $desktopShortcut = Join-Path ([Environment]::GetFolderPath('Desktop')) 'FerroStage.lnk'
        $lock = [IO.File]::Open($desktopShortcut, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::None)
        try {
            Assert-DistributionFailure { Invoke-DistributionTool $receipt.setup @('/S') 'Expected locked shortcut installation failure' } 'Locked shortcut prevents installation'
            Assert-DistributionFailure { Invoke-DistributionUninstall -Quiet } 'Locked shortcut prevents uninstallation before deleting payload'
        } finally { $lock.Dispose() }
        Test-DistributionInstallation $payload $metadata.Version
        Assert-Distribution ((Get-ItemProperty $script:UninstallKey).DisplayVersion -eq $metadata.Version) 'Lock failure changed registered version'
        Test-DistributionInstallation $payload $metadata.Version
        Invoke-DistributionTool $upgrade @('/S') 'Forward upgrade test package' | Out-Host
        Assert-Distribution ((Get-ItemProperty $script:UninstallKey).DisplayVersion -eq $nextVersion) 'Upgrade did not register test version'
        Assert-DistributionFailure { Invoke-DistributionTool $receipt.setup @('/S') 'Expected downgrade failure' } 'Downgrade is refused'
        Assert-Distribution ((Get-ItemProperty $script:UninstallKey).DisplayVersion -eq $nextVersion) 'Downgrade changed registered version'
        Invoke-DistributionUninstall -Quiet
        Assert-Distribution (-not (Test-Path $script:UninstallKey)) 'Native uninstall left its registry key'
        foreach ($file in @(Get-DistributionFiles) + @('inventory.json', 'Uninstall.exe')) {
            Assert-Distribution (-not (Test-Path -LiteralPath (Join-Path $installRoot $file))) "Uninstall left owned file: $file"
        }
        Assert-Distribution ((Get-DistributionHash $unknownFile) -eq $unknownHash) 'Uninstall changed unrelated file'
        Assert-Distribution ((Get-DistributionHash $pdfCanary) -eq $pdfHash) 'Uninstall changed user PDF'
        Assert-Distribution ([IO.File]::ReadAllText($settingsCanary) -eq 'preserve settings canary') 'Uninstall changed settings canary'
        Write-Host '[PASS] Uninstall removed owned payload and preserved settings, PDF and unrelated file'
        Remove-Item -LiteralPath $unknownFile, $pdfCanary -Force
    } finally {
        # Test package identities are synthetic. Always restore the actual release.
        if (Test-Path $script:UninstallKey) { Invoke-DistributionUninstall -Quiet }
        Invoke-DistributionTool $receipt.setup @('/S') 'Restore actual FerroStage release' | Out-Host
        Test-DistributionPayload $installRoot -Installed | Out-Null
        foreach ($path in $settingsBefore.Keys) { Assert-Distribution ((Get-DistributionHash $path) -eq $settingsBefore[$path]) "User settings changed: $path" }
        Remove-Item -LiteralPath $settingsCanary -Force
    }
    Test-DistributionInstallation $payload $metadata.Version
    Write-Host '[PASS] Lifecycle and shortcut/registration metadata; actual release is installed'
}
Write-Host '[PASS] Windows distribution regression checks'
