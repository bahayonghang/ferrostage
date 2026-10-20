# One Windows desktop payload for build, Setup, portable release and installation.
param(
    [ValidateSet('build', 'release', 'install', 'uninstall', 'verify')]
    [string]$Action = 'build',
    [switch]$Silent,
    [string]$PayloadPath,
    [switch]$LibraryOnly
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$script:DistributionRepo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$script:UninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\FerroStage'

function Get-DistributionHash([string]$Path) {
    $algorithm = [Security.Cryptography.SHA256]::Create()
    try {
        $stream = [IO.File]::OpenRead($Path)
        try { return ([BitConverter]::ToString($algorithm.ComputeHash($stream))).Replace('-', '').ToLowerInvariant() }
        finally { $stream.Dispose() }
    } finally { $algorithm.Dispose() }
}

function Invoke-DistributionTool([string]$File, [string[]]$Arguments, [string]$Stage) {
    $timer = [Diagnostics.Stopwatch]::StartNew()
    Write-Host "[START] $Stage"
    $process = [Diagnostics.Process]::new()
    try {
        # All arguments are controlled by this script; quote paths containing spaces.
        $quoted = @($Arguments | ForEach-Object {
            # NSIS requires bare /S and consumes its final _?= path as raw text.
            if ($_.StartsWith('_?=', [StringComparison]::Ordinal) -or $_ -notmatch '[\s"]') { $_ }
            else { '"' + $_.Replace('"', '\"') + '"' }
        })
        $process.StartInfo.FileName = $File
        $process.StartInfo.Arguments = $quoted -join ' '
        $process.StartInfo.WorkingDirectory = $script:DistributionRepo
        $process.StartInfo.UseShellExecute = $false
        $process.StartInfo.CreateNoWindow = $true
        $process.StartInfo.RedirectStandardOutput = $true
        $process.StartInfo.RedirectStandardError = $true
        $process.StartInfo.StandardOutputEncoding = [Text.Encoding]::UTF8
        $process.StartInfo.StandardErrorEncoding = [Text.Encoding]::UTF8
        $process.Start() | Out-Null
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        $heartbeat = 0
        while (-not $process.WaitForExit(1000)) {
            if ($timer.Elapsed.TotalSeconds -ge $heartbeat + 20) {
                $heartbeat = [int]$timer.Elapsed.TotalSeconds
                Write-Host "[RUNNING] $Stage ($heartbeat s)"
            }
        }
        $process.WaitForExit()
        $errors = $stderr.Result
        if ($errors) { Write-Host $errors.TrimEnd() }
        $output = $stdout.Result
        if ($process.ExitCode -ne 0) {
            if ($output) { Write-Host $output.TrimEnd() }
            throw "$Stage failed (exit $($process.ExitCode))"
        }
        Write-Host "[PASS] $Stage ($([math]::Round($timer.Elapsed.TotalSeconds, 1)) s)"
        return $output
    } finally {
        $process.Dispose()
    }
}

function Get-DistributionMetadata {
    $metadata = (Invoke-DistributionTool 'cargo' @('metadata', '--no-deps', '--format-version', '1', '--locked') 'Cargo metadata') | ConvertFrom-Json
    $app = @($metadata.packages | Where-Object name -eq 'ferrostage-app')
    if ($app.Count -ne 1 -or $app[0].version -notmatch '^\d+\.\d+\.\d+$') { throw 'A stable numeric app version is required for Windows Setup' }
    $version = [version]$app[0].version
    if ($version.Major -gt 65535 -or $version.Minor -gt 65535 -or $version.Build -gt 65535) { throw 'Version exceeds Windows version fields' }
    return [pscustomobject]@{ Version = $app[0].version; PackageId = $app[0].id; Dist = (Join-Path $metadata.target_directory 'dist') }
}

function Reset-DistributionDirectory([string]$Path, [string]$Root) {
    $rootPath = [IO.Path]::GetFullPath($Root).TrimEnd('\')
    $fullPath = [IO.Path]::GetFullPath($Path)
    if (-not $fullPath.StartsWith($rootPath + '\', [StringComparison]::OrdinalIgnoreCase)) { throw "Refusing cleanup outside distribution root: $fullPath" }
    # Cargo may own a target symlink. Reparse points below its dist directory are not ours.
    $cursor = $fullPath
    while ($cursor.Length -ge $rootPath.Length) {
        if (Test-Path -LiteralPath $cursor) {
            if ((Get-Item -LiteralPath $cursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Refusing distribution reparse point: $cursor" }
        }
        if ($cursor -eq $rootPath) { break }
        $cursor = [IO.Path]::GetDirectoryName($cursor)
    }
    if (Test-Path -LiteralPath $fullPath) {
        if (@(Get-ChildItem -LiteralPath $fullPath -Recurse -Force | Where-Object { $_.Attributes -band [IO.FileAttributes]::ReparsePoint }).Count) { throw "Refusing cleanup containing reparse points: $fullPath" }
        Remove-Item -LiteralPath $fullPath -Recurse -Force
    }
    New-Item -ItemType Directory -Path $fullPath -Force | Out-Null
}

function Get-DistributionFiles {
    $runtime = Get-Content -LiteralPath (Join-Path $script:DistributionRepo 'docs/pdfium-runtime.json') -Raw | ConvertFrom-Json
    $toolchain = Get-Content -LiteralPath (Join-Path $script:DistributionRepo 'packaging/windows/nsis-toolchain.json') -Raw | ConvertFrom-Json
    $files = @('ferrostage.exe', 'pdfium.dll', 'FERROSTAGE-LICENSE.txt', 'FERROSTAGE-LICENSE-MIT.txt', 'FERROSTAGE-LICENSE-APACHE.txt')
    $files += @($runtime.notices.PSObject.Properties.Name)
    $files += @($toolchain.notices.PSObject.Properties | ForEach-Object { 'installer-licenses/' + [IO.Path]::GetFileName($_.Name) })
    return @($files | Sort-Object -Unique)
}

function Test-DistributionPayload([string]$Path, [switch]$Installed) {
    $inventory = Get-Content -LiteralPath (Join-Path $Path 'inventory.json') -Raw | ConvertFrom-Json
    if ($inventory.schema -ne 1 -or $inventory.version -notmatch '^\d+\.\d+\.\d+$') { throw 'Invalid payload inventory' }
    $expected = @(Get-DistributionFiles)
    $actual = @(Get-ChildItem -LiteralPath $Path -File -Recurse | ForEach-Object { $_.FullName.Substring($Path.TrimEnd('\').Length + 1).Replace('\', '/') } | Where-Object { $_ -ne 'inventory.json' -and (-not $Installed -or $_ -ne 'Uninstall.exe') } | Sort-Object)
    if ($Installed) {
        # Unrelated user files are allowed in an installation and preserved by uninstall.
        $actual = @($actual | Where-Object { $_ -in $expected })
        if (-not (Test-Path -LiteralPath (Join-Path $Path 'Uninstall.exe'))) { throw 'Native uninstaller is missing' }
    }
    if (@(Compare-Object $expected $actual).Count -ne 0) { throw 'Payload file set differs from the approved inventory' }
    $entries = @($inventory.files)
    if (@(Compare-Object $expected @($entries.path | Sort-Object)).Count -ne 0) { throw 'Inventory does not describe the complete payload' }
    foreach ($entry in $entries) {
        if ($entry.sha256 -ne (Get-DistributionHash (Join-Path $Path $entry.path))) { throw "Payload checksum mismatch: $($entry.path)" }
    }
    $runtime = Get-Content -LiteralPath (Join-Path $script:DistributionRepo 'docs/pdfium-runtime.json') -Raw | ConvertFrom-Json
    if ((Get-DistributionHash (Join-Path $Path 'pdfium.dll')) -ne $runtime.dll_sha256) { throw 'Pinned PDFium DLL mismatch' }
    foreach ($entry in $runtime.notices.PSObject.Properties) {
        if ((Get-DistributionHash (Join-Path $Path $entry.Name)) -ne $entry.Value.sha256) { throw "Pinned PDFium notice mismatch: $($entry.Name)" }
    }
    foreach ($name in @('LICENSE', 'LICENSE-MIT', 'LICENSE-APACHE')) {
        if ((Get-DistributionHash (Join-Path $Path "FERROSTAGE-$name.txt")) -ne (Get-DistributionHash (Join-Path $script:DistributionRepo $name))) { throw "Application license mismatch: $name" }
    }
    $toolchain = Get-Content -LiteralPath (Join-Path $script:DistributionRepo 'packaging/windows/nsis-toolchain.json') -Raw | ConvertFrom-Json
    foreach ($entry in $toolchain.notices.PSObject.Properties) {
        if ((Get-DistributionHash (Join-Path $Path ('installer-licenses/' + [IO.Path]::GetFileName($entry.Name)))) -ne $entry.Value) { throw "Installer license mismatch: $($entry.Name)" }
    }
    return $inventory
}

function Get-DistributionNsis {
    $lock = Get-Content -LiteralPath (Join-Path $script:DistributionRepo 'packaging/windows/nsis-toolchain.json') -Raw | ConvertFrom-Json
    $cache = Join-Path $script:DistributionRepo "cache/nsis-$($lock.version)"
    $archive = Join-Path $script:DistributionRepo "cache/nsis-$($lock.version).zip"
    New-Item -ItemType Directory -Force -Path $cache | Out-Null
    if (-not (Test-Path -LiteralPath $archive)) { Invoke-WebRequest -Uri $lock.archive_url -OutFile $archive -UseBasicParsing }
    if ((Get-DistributionHash $archive) -ne $lock.archive_sha256) { throw 'NSIS compiler archive checksum mismatch' }
    $compilerRoot = Join-Path $script:DistributionRepo ('cache/' + $lock.directory)
    # Re-extract the verified archive to prevent altered cached tools being trusted.
    Expand-Archive -LiteralPath $archive -DestinationPath (Join-Path $script:DistributionRepo 'cache') -Force
    foreach ($entry in $lock.notices.PSObject.Properties) {
        if ((Get-DistributionHash (Join-Path $compilerRoot $entry.Name)) -ne $entry.Value) { throw "NSIS notice checksum mismatch: $($entry.Name)" }
    }
    return $compilerRoot
}

function New-DistributionPayload($Metadata, [string]$CompilerRoot) {
    $output = Invoke-DistributionTool 'cargo' @('build', '-p', 'ferrostage-app', '--release', '--features', 'gpui,pdfium', '--locked', '--message-format=json-render-diagnostics') 'Desktop release build'
    $artifacts = @($output -split '\r?\n' | Where-Object { $_.StartsWith('{') } | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.package_id -eq $Metadata.PackageId -and $_.target.name -eq 'ferrostage' -and $_.executable })
    if ($artifacts.Count -ne 1) { throw 'Cargo did not report exactly one FerroStage executable' }
    $executable = $artifacts[0].executable
    if ($executable -notlike '*\x86_64-pc-windows-msvc\*' -and $env:CARGO_BUILD_TARGET -and $env:CARGO_BUILD_TARGET -ne 'x86_64-pc-windows-msvc') { throw 'Only Windows x64 MSVC desktop packages are supported' }
    # Read the PE machine field as the final authority, including .cargo/config targets.
    $bytes = [IO.File]::ReadAllBytes($executable)
    $pe = [BitConverter]::ToInt32($bytes, 60)
    if ([BitConverter]::ToUInt16($bytes, $pe + 4) -ne 0x8664) { throw 'Desktop executable is not Windows x64' }
    if ([BitConverter]::ToUInt16($bytes, $pe + 24 + 68) -ne 2) { throw 'Release executable does not use the Windows GUI subsystem' }
    & (Join-Path $PSScriptRoot 'stage-pdfium.ps1') -Destination ([IO.Path]::GetDirectoryName($executable)) | Out-Host
    $payload = Join-Path $Metadata.Dist "payload-$($Metadata.Version)"
    Reset-DistributionDirectory $payload $Metadata.Dist
    foreach ($relative in @(Get-DistributionFiles)) {
        $destination = Join-Path $payload $relative
        New-Item -ItemType Directory -Path ([IO.Path]::GetDirectoryName($destination)) -Force | Out-Null
        if ($relative -like 'FERROSTAGE-*') { $source = Join-Path $script:DistributionRepo ($relative.Substring(11).Replace('.txt', '')) }
        elseif ($relative -like 'installer-licenses/*') { $source = Join-Path $CompilerRoot ([IO.Path]::GetFileName($relative)) }
        else { $source = Join-Path ([IO.Path]::GetDirectoryName($executable)) $relative }
        Copy-Item -LiteralPath $source -Destination $destination
    }
    $inventory = [ordered]@{ schema = 1; version = $Metadata.Version; platform = 'windows-x64'; files = @() }
    $inventory.files = @(Get-DistributionFiles | ForEach-Object { [ordered]@{ path = $_; sha256 = (Get-DistributionHash (Join-Path $payload $_)) } })
    $inventory | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $payload 'inventory.json') -Encoding UTF8
    Test-DistributionPayload $payload | Out-Null
    return $payload
}

function ConvertTo-NsisPath([string]$Text) { return $Text.Replace('$', '$$').Replace('"', '$\"').Replace('/', '\') }

function New-DistributionSetup([string]$Payload, [string]$Version, [string]$CompilerRoot, [string]$Dist, [string]$InstallerTemplate = (Join-Path $script:DistributionRepo 'packaging/windows/installer.nsi')) {
    $inventory = Test-DistributionPayload $Payload
    if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw 'Setup version must be numeric' }
    if ($inventory.version -ne $Version) { throw 'Setup and payload versions differ' }
    $work = Join-Path $Dist "setup-$Version"
    Reset-DistributionDirectory $work $Dist
    $files = @((Get-DistributionFiles) + 'inventory.json')
    $include = [Collections.Generic.List[string]]::new()
    $include.Add('!macro PayloadExtract')
    foreach ($file in $files) {
        $parent = [IO.Path]::GetDirectoryName($file.Replace('/', '\'))
        $include.Add('SetOutPath "$PLUGINSDIR\payload' + $(if ($parent) { '\' + $parent }) + '"')
        $include.Add('File "' + (ConvertTo-NsisPath (Join-Path $Payload $file)) + '"')
    }
    $include.Add('!macroend')
    $directories = @($files | ForEach-Object { [IO.Path]::GetDirectoryName($_.Replace('/', '\')) } | Where-Object { $_ } | Sort-Object -Unique)
    $include.Add('!macro CheckDirectories PREFIX')
    foreach ($dir in @('') + $directories) {
        $include.Add('Push "$INSTDIR' + $(if ($dir) { '\' + $dir }) + '"')
        $include.Add('Call ${PREFIX}CheckDirectory')
    }
    $include.Add('!macroend')
    $include.Add('!macro CheckFiles PREFIX')
    foreach ($file in $files + 'Uninstall.exe') {
        $include.Add('Push "$INSTDIR\' + $file.Replace('/', '\') + '"')
        $include.Add('Call ${PREFIX}CheckFile')
    }
    $include.Add('!macroend')
    foreach ($operation in @('Backup', 'Install', 'Restore', 'Delete')) {
        $include.Add("!macro Payload$operation")
        foreach ($file in $files + 'Uninstall.exe') {
            $relative = $file.Replace('/', '\')
            $parent = [IO.Path]::GetDirectoryName($relative)
            if ($operation -eq 'Backup') {
                $include.Add('CreateDirectory "$PLUGINSDIR\backup' + $(if ($parent) { '\' + $parent }) + '"')
                $include.Add('IfFileExists "$INSTDIR\' + $relative + '" 0 +4')
                $include.Add('ClearErrors')
                $include.Add('CopyFiles /SILENT "$INSTDIR\' + $relative + '" "$PLUGINSDIR\backup\' + $relative + '"')
                $include.Add('IfErrors backup_failed')
            } elseif ($operation -eq 'Install') {
                if ($file -eq 'Uninstall.exe') { continue }
                $include.Add('CreateDirectory "$INSTDIR' + $(if ($parent) { '\' + $parent }) + '"')
                $include.Add('ClearErrors')
                $include.Add('CopyFiles /SILENT "$PLUGINSDIR\payload\' + $relative + '" "$INSTDIR\' + $relative + '"')
                $include.Add('IfErrors install_failed')
            } elseif ($operation -eq 'Restore') {
                $include.Add('Delete "$INSTDIR\' + $relative + '"')
                $include.Add('IfFileExists "$PLUGINSDIR\backup\' + $relative + '" 0 +2')
                $include.Add('CopyFiles /SILENT "$PLUGINSDIR\backup\' + $relative + '" "$INSTDIR\' + $relative + '"')
            } else {
                if ($file -eq 'Uninstall.exe') { continue }
                $include.Add('IfFileExists "$INSTDIR\' + $relative + '" 0 +2')
                $include.Add('Delete "$INSTDIR\' + $relative + '"')
            }
        }
        $include.Add('!macroend')
    }
    $include.Add('!macro RemoveEmptyDirectories')
    foreach ($dir in @($directories | Sort-Object -Descending)) { $include.Add('RMDir "$INSTDIR\' + $dir + '"') }
    $include.Add('RMDir "$INSTDIR"')
    $include.Add('!macroend')
    [IO.File]::WriteAllLines((Join-Path $work 'payload.nsh'), $include, [Text.UTF8Encoding]::new($true))
    $setup = Join-Path $Dist "FerroStage-$Version-windows-x64-Setup.exe"
    if (Test-Path -LiteralPath $setup) { Remove-Item -LiteralPath $setup -Force }
    $size = [int][math]::Ceiling((Get-ChildItem -LiteralPath $Payload -File -Recurse | Measure-Object -Property Length -Sum).Sum / 1024)
    Invoke-DistributionTool (Join-Path $CompilerRoot 'makensis.exe') @('/WX', '/V2', "/DAPP_VERSION=$Version", "/DAPP_SIZE_KIB=$size", "/DOUTPUT_FILE=$setup", "/DPAYLOAD_INCLUDE=$(Join-Path $work 'payload.nsh')", "/DAPP_ICON=$(Join-Path $script:DistributionRepo 'assets/icons/ferrostage.ico')", "/DAPP_LICENSE=$(Join-Path $Payload 'FERROSTAGE-LICENSE.txt')", $InstallerTemplate) 'NSIS Setup' | Out-Host
    if (-not (Test-Path -LiteralPath $setup)) { throw 'NSIS did not produce Setup' }
    return $setup
}

function Test-DistributionChecksums([string]$Dist, [string]$Manifest) {
    $lines = @(Get-Content -LiteralPath $Manifest)
    if ($lines.Count -ne 2 -or @($lines | Sort-Object -Unique).Count -ne 2) { throw 'Release checksum manifest must describe exactly Setup and portable ZIP' }
    foreach ($line in $lines) {
        if ($line -notmatch '^([0-9a-f]{64})  ([A-Za-z0-9_.-]+)$') { throw 'Invalid release checksum line' }
        if ((Get-DistributionHash (Join-Path $Dist $Matches[2])) -ne $Matches[1]) { throw "Release checksum mismatch: $($Matches[2])" }
    }
}

function Test-DistributionRelease($Metadata, $Receipt) {
    if ($Receipt.version -ne $Metadata.Version) { throw 'Release receipt version does not match Cargo metadata' }
    $prefix = "FerroStage-$($Metadata.Version)-windows-x64-"
    $expected = @{
        payload = (Join-Path $Metadata.Dist "payload-$($Metadata.Version)")
        setup = (Join-Path $Metadata.Dist ($prefix + 'Setup.exe'))
        portable = (Join-Path $Metadata.Dist ($prefix + 'portable.zip'))
        checksums = (Join-Path $Metadata.Dist ($prefix + 'SHA256SUMS.txt'))
    }
    foreach ($name in $expected.Keys) {
        if ($Receipt.$name -ne $expected[$name]) { throw "Unexpected release receipt path: $name" }
    }
    foreach ($name in @('setup', 'portable')) {
        if ((Get-DistributionHash $Receipt.$name) -ne $Receipt.($name + '_sha256')) { throw "Release receipt hash mismatch: $name" }
    }
    Test-DistributionChecksums $Metadata.Dist $Receipt.checksums
    $manifestNames = @(Get-Content -LiteralPath $Receipt.checksums | ForEach-Object { ($_ -split '  ', 2)[1] } | Sort-Object)
    if (@(Compare-Object $manifestNames @([IO.Path]::GetFileName($Receipt.setup), [IO.Path]::GetFileName($Receipt.portable) | Sort-Object)).Count) { throw 'Release checksum manifest names do not match receipt artifacts' }
    $inventory = Test-DistributionPayload $Receipt.payload
    if ($inventory.version -ne $Metadata.Version) { throw 'Release payload version mismatch' }
    $zipCheck = Join-Path $Metadata.Dist 'zip-verification'
    Reset-DistributionDirectory $zipCheck $Metadata.Dist
    Expand-Archive -LiteralPath $Receipt.portable -DestinationPath $zipCheck
    $portableInventory = Test-DistributionPayload $zipCheck
    if ($portableInventory.version -ne $Metadata.Version -or (Get-DistributionHash (Join-Path $zipCheck 'inventory.json')) -ne (Get-DistributionHash (Join-Path $Receipt.payload 'inventory.json'))) { throw 'Portable ZIP and Setup payload inventories differ' }
}

function Test-DistributionInstalledPayload([string]$Payload, [string]$Installed) {
    Test-DistributionPayload $Payload | Out-Null
    if ((Get-DistributionHash (Join-Path $Installed 'inventory.json')) -ne (Get-DistributionHash (Join-Path $Payload 'inventory.json'))) { throw 'Installed inventory differs from the trusted release payload' }
    Test-DistributionPayload $Installed -Installed | Out-Null
}

function Test-DistributionInstallation([string]$Payload, [string]$Version) {
    $installed = Join-Path $env:LOCALAPPDATA 'Programs\FerroStage'
    Test-DistributionInstalledPayload $Payload $installed
    $registration = Get-ItemProperty $script:UninstallKey
    $uninstaller = '"' + (Join-Path $installed 'Uninstall.exe') + '"'
    $expected = @{
        DisplayName = 'FerroStage'; DisplayVersion = $Version; InstallLocation = $installed
        DisplayIcon = "$installed\ferrostage.exe,0"; Publisher = 'FerroStage contributors'
        UninstallString = $uninstaller; QuietUninstallString = "$uninstaller /S"; NoModify = 1; NoRepair = 1
    }
    foreach ($name in $expected.Keys) {
        if ($registration.$name -ne $expected[$name]) { throw "Installed registration mismatch: $name" }
    }
    $shell = New-Object -ComObject WScript.Shell
    foreach ($folder in @([Environment]::GetFolderPath('Desktop'), [Environment]::GetFolderPath('Programs'))) {
        $path = Join-Path $folder 'FerroStage.lnk'
        if (-not (Test-Path -LiteralPath $path)) { throw "Installed shortcut missing: $path" }
        $shortcut = $shell.CreateShortcut($path)
        if ($shortcut.TargetPath -ne (Join-Path $installed 'ferrostage.exe') -or $shortcut.WorkingDirectory -ne $installed -or $shortcut.IconLocation -ne "$installed\ferrostage.exe,0") { throw "Installed shortcut metadata mismatch: $path" }
    }
}

function Invoke-DistributionUninstall([switch]$Quiet) {
    if (-not (Test-Path $script:UninstallKey)) { Write-Host 'FerroStage is not installed for this user.'; return }
    $registration = Get-ItemProperty $script:UninstallKey
    $expected = Join-Path $env:LOCALAPPDATA 'Programs\FerroStage'
    if ($registration.InstallLocation -ne $expected) { throw 'Refusing an unexpected registered installation path' }
    $uninstaller = Join-Path $expected 'Uninstall.exe'
    if ($registration.UninstallString -ne ('"' + $uninstaller + '"')) { throw 'Refusing an unexpected uninstall command' }
    $arguments = @()
    if ($Quiet) { $arguments += '/S' }
    # _?= prevents NSIS detaching a second process, so the exit code can be checked.
    # Copy the uninstaller first so its owned original is removable during execution.
    $temporary = Join-Path ([IO.Path]::GetTempPath()) ('FerroStage-uninstall-' + [guid]::NewGuid().ToString('N') + '.exe')
    Copy-Item -LiteralPath $uninstaller -Destination $temporary
    try {
        Invoke-DistributionTool $temporary ($arguments + "_?=$expected") 'Uninstall FerroStage' | Out-Host
        if (Test-Path $script:UninstallKey) { throw 'Uninstall did not remove application registration (possibly cancelled)' }
    } finally { Remove-Item -LiteralPath $temporary -Force -ErrorAction SilentlyContinue }
}

if ($LibraryOnly) { return }
if ($env:OS -ne 'Windows_NT') { throw 'Windows desktop distribution requires Windows' }
try {
    if ($Action -eq 'uninstall') { Invoke-DistributionUninstall -Quiet:$Silent; return }
    if ($Action -eq 'verify') {
        if ($PayloadPath) { Test-DistributionPayload ([IO.Path]::GetFullPath($PayloadPath)) | Out-Host }
        else {
            $metadata = Get-DistributionMetadata
            $receipt = Get-Content -LiteralPath (Join-Path $metadata.Dist 'release.json') -Raw | ConvertFrom-Json
            Test-DistributionRelease $metadata $receipt
            Write-Host '[PASS] Release payload, Setup, portable ZIP and checksums verified'
        }
        return
    }
    $metadata = Get-DistributionMetadata
    $receiptPath = Join-Path $metadata.Dist 'release.json'
    if (Test-Path -LiteralPath $receiptPath) { Remove-Item -LiteralPath $receiptPath -Force }
    $compiler = Get-DistributionNsis
    $payload = New-DistributionPayload $metadata $compiler
    Write-Host "[PASS] Verified desktop payload: $payload"
    if ($Action -eq 'build') { return }
    $setup = New-DistributionSetup $payload $metadata.Version $compiler $metadata.Dist
    $zip = Join-Path $metadata.Dist "FerroStage-$($metadata.Version)-windows-x64-portable.zip"
    if (Test-Path -LiteralPath $zip) { Remove-Item -LiteralPath $zip -Force }
    Compress-Archive -Path (Join-Path $payload '*') -DestinationPath $zip -CompressionLevel Optimal
    $checksums = Join-Path $metadata.Dist "FerroStage-$($metadata.Version)-windows-x64-SHA256SUMS.txt"
    @($setup, $zip) | ForEach-Object { (Get-DistributionHash $_) + '  ' + [IO.Path]::GetFileName($_) } | Set-Content -LiteralPath $checksums -Encoding ASCII
    $receipt = [ordered]@{ version = $metadata.Version; payload = $payload; setup = $setup; portable = $zip; checksums = $checksums; setup_sha256 = (Get-DistributionHash $setup); portable_sha256 = (Get-DistributionHash $zip) }
    Test-DistributionRelease $metadata ([pscustomobject]$receipt)
    $receipt | ConvertTo-Json | Set-Content -LiteralPath $receiptPath -Encoding UTF8
    Write-Host "[PASS] Release verified: $receiptPath"
    if ($Action -eq 'install') {
        $arguments = @()
        if ($Silent) { $arguments += '/S' }
        Invoke-DistributionTool $setup $arguments 'Install FerroStage' | Out-Host
        Test-DistributionInstallation $payload $metadata.Version
        Write-Host "[PASS] Installed FerroStage $($metadata.Version): $(Join-Path $env:LOCALAPPDATA 'Programs\FerroStage')"
    }
} catch {
    Write-Error $_
    exit 1
}
