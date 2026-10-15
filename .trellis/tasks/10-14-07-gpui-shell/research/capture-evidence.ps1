$ErrorActionPreference = 'Stop'
$workspace = (Resolve-Path '.').Path
$research = Join-Path $workspace '.trellis/tasks/10-14-07-gpui-shell/research'
$baselinePath = '.trellis/tasks/10-14-06-beamer-document-model/research/independent-check-manifest.json'
$baseline = Get-Content -LiteralPath $baselinePath -Raw | ConvertFrom-Json -AsHashtable

function Get-HashRecord([string] $path) {
    $item = Get-Item -LiteralPath $path
    [ordered]@{
        path = $item.FullName.Replace('\', '/')
        bytes = $item.Length
        sha256 = (Get-FileHash -LiteralPath $item.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    }
}

$comparison = foreach ($entry in $baseline.source_files.GetEnumerator() | Sort-Object Key) {
    $exists = Test-Path -LiteralPath $entry.Key -PathType Leaf
    $current = if ($exists) { (Get-FileHash -LiteralPath $entry.Key -Algorithm SHA256).Hash.ToLowerInvariant() } else { $null }
    [ordered]@{
        path = $entry.Key
        product_input = $entry.Key -match '^(crates/|Cargo\.(toml|lock)$|deny\.toml$|justfile$|rust-toolchain\.toml$|\.github/workflows/ci\.yml$)'
        baseline_sha256 = $entry.Value
        current_sha256 = $current
        result = if (-not $exists) { 'MISSING' } elseif ($current -eq $entry.Value) { 'UNCHANGED' } else { 'DIFFERENT' }
    }
}
$productInputs = @($comparison | Where-Object product_input)
$comparisonReport = [ordered]@{
    captured_utc = [DateTime]::UtcNow.ToString('o')
    baseline = Get-HashRecord $baselinePath
    baseline_head = $baseline.head
    total_files = @($comparison).Count
    unchanged = @($comparison | Where-Object result -eq 'UNCHANGED').Count
    different_or_missing = @($comparison | Where-Object result -ne 'UNCHANGED').Count
    product_input_count = $productInputs.Count
    product_inputs_unchanged = @($productInputs | Where-Object result -eq 'UNCHANGED').Count
    unlisted_crate_files = @(rg --files crates | ForEach-Object { $_.Replace('\', '/') } | Where-Object { -not $baseline.source_files.ContainsKey($_) })
    files = @($comparison)
}
$comparisonReport | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $research 'product-input-comparison.json') -Encoding utf8NoBOM

$routeRecords = foreach ($route in @('gpui-route-spike', 'gpui-kit-route-spike', 'gpui-pre-minimal-route')) {
    $original = Join-Path $workspace "cache/$route"
    $retained = Join-Path $research "route-fixtures/$route"
    New-Item -ItemType Directory -Path (Join-Path $retained 'src') -Force | Out-Null
    $copies = foreach ($relative in @('Cargo.toml', 'Cargo.lock', 'src/main.rs')) {
        $sourcePath = Join-Path $original $relative
        $copyPath = Join-Path $retained $relative
        Copy-Item -LiteralPath $sourcePath -Destination $copyPath -Force
        [ordered]@{ original = Get-HashRecord $sourcePath; retained = Get-HashRecord $copyPath }
    }
    $binaryPath = Join-Path $original "target/debug/ferrostage-$route.exe"
    [ordered]@{
        name = $route
        files = @($copies)
        binary = if (Test-Path -LiteralPath $binaryPath) { Get-HashRecord $binaryPath } else { $null }
        binary_executed = $false
        compiled = $route -ne 'gpui-pre-minimal-route'
    }
}

$tagged = 'C:/Users/lyh/.cargo/git/checkouts/zed-a70e2ad075855582/96a1b1b'
$registry = 'C:/Users/lyh/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f'
$advisories = 'C:/Users/lyh/.cargo/advisory-dbs/advisory-db-3157b0e258782691'
$sourcePaths = @(
    "$tagged/Cargo.toml",
    "$tagged/crates/gpui/Cargo.toml",
    "$tagged/crates/gpui_platform/Cargo.toml",
    "$tagged/crates/gpui_windows/Cargo.toml",
    "$tagged/crates/gpui_apple/Cargo.toml",
    "$tagged/crates/gpui_platform/src/gpui_platform.rs",
    "$tagged/crates/gpui_windows/src/display.rs",
    "$tagged/crates/gpui_windows/src/window.rs",
    "$tagged/crates/gpui/src/platform.rs",
    "$tagged/crates/gpui/src/window.rs",
    "$tagged/crates/gpui/src/app.rs",
    "$tagged/crates/gpui/src/keymap.rs",
    "$tagged/crates/gpui/src/platform/keystroke.rs",
    "$tagged/crates/gpui/src/svg_renderer.rs",
    "$tagged/crates/gpui/examples/a11y.rs",
    "$tagged/crates/gpui/examples/tab_stop.rs",
    "$registry/gpui-pre-0.3.9/src/svg_renderer.rs",
    "$registry/gpui-pre-windows-0.3.9/src/display.rs"
)
foreach ($package in @('gpui-kit-0.7.3', 'gpui-component-0.7.3', 'gpui-base-0.7.3', 'gpui-kit-assets-0.7.3', 'gpui-pre-0.3.9', 'gpui-pre-platform-0.3.9', 'gpui-pre-windows-0.3.9', 'gpui-pre-apple-0.3.9', 'gpui-pre-wgpu-0.3.9', 'gpui-pre-reqwest-0.12.15', 'zed-font-kit-0.14.1-zed', 'resvg-0.46.0', 'usvg-0.46.0', 'image-0.25.10', 'exr-1.74.2', 'pulp-0.22.3')) {
    $sourcePaths += "$registry/$package/Cargo.toml"
}
foreach ($package in @('arrayref-0.3.9', 'libbz2-rs-sys-0.2.5', 'cbindgen-0.28.0', 'dwrote-0.11.5', 'enum-iterator-2.3.0', 'enum-iterator-derive-1.5.0', 'option-ext-0.2.0')) {
    $sourcePaths += "$registry/$package/Cargo.toml"
    $sourcePaths += @(Get-ChildItem -LiteralPath "$registry/$package" -File | Where-Object Name -Like 'LICENSE*' | ForEach-Object FullName)
}
foreach ($entry in @('paste/RUSTSEC-2024-0436.md', 'rustybuzz/RUSTSEC-2026-0206.md', 'ttf-parser/RUSTSEC-2026-0192.md', 'instant/RUSTSEC-2024-0384.md', 'rustls-pemfile/RUSTSEC-2025-0134.md')) {
    $sourcePaths += "$advisories/crates/$entry"
}
$evidence = [ordered]@{
    captured_utc = [DateTime]::UtcNow.ToString('o')
    result = 'BLOCKED at I-2; isolated A/B native compile PASS; A/B/C unchanged policy FAIL; no product/UI delivery'
    tagged_revision = '96a1b1b5389efff473fc2306f7f8c690f2e1a417'
    pre_upstream_revision = '93d745668805029476e6d7fbb19652116fed51de'
    advisory_db_revision = (Get-Content -LiteralPath "$advisories/.git/refs/heads/main" -Raw).Trim()
    routes = @($routeRecords)
    sources = @($sourcePaths | Sort-Object -Unique | ForEach-Object { Get-HashRecord $_ })
    product_policy = Get-HashRecord (Join-Path $workspace 'deny.toml')
    task_checkpoint = Get-HashRecord (Join-Path $research '../implement.md')
    evidence_files = @(Get-ChildItem -LiteralPath $research -File | Where-Object Name -ne 'evidence-manifest.json' | Sort-Object Name | ForEach-Object { Get-HashRecord $_.FullName })
}
$evidence | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $research 'evidence-manifest.json') -Encoding utf8NoBOM
[ordered]@{
    total_files = $comparisonReport.total_files
    unchanged = $comparisonReport.unchanged
    product_inputs = $comparisonReport.product_input_count
    product_inputs_unchanged = $comparisonReport.product_inputs_unchanged
    differences = @($comparison | Where-Object result -ne 'UNCHANGED')
    routes = @($routeRecords | ForEach-Object { [ordered]@{ name = $_.name; compiled = $_.compiled; binary = $_.binary } })
    source_files_hashed = @($evidence.sources).Count
} | ConvertTo-Json -Depth 6
