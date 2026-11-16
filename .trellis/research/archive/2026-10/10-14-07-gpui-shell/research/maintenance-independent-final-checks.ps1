$ErrorActionPreference = 'Continue'
$reviewResearch = '.trellis/tasks/10-14-07-gpui-shell/research'
$reviewCandidate = 'cache/gpui-maintenance-candidate'
$reviewTarget = "$reviewCandidate/target"
$reviewJobs = @(
    @{ Name='final-root-lock'; Args=@('update','--manifest-path',"$reviewCandidate/Cargo.toml",'-p','harfrust@0.12.0','--offline') },
    @{ Name='final-gpui-lock'; Args=@('update','--manifest-path',"$reviewCandidate/vendor/gpui/Cargo.toml",'-p','harfrust@0.12.0','--offline') },
    @{ Name='final-wgpu-lock'; Args=@('update','--manifest-path',"$reviewCandidate/validation/wgpu-consumer/Cargo.toml",'-p','harfrust@0.12.0','--offline') },
    @{ Name='final-harfrust-tests'; Args=@('test','--manifest-path',"$reviewCandidate/vendor/harfrust/Cargo.toml",'--target-dir',$reviewTarget,'--lib','--locked','--offline') },
    @{ Name='final-gpui-tests'; Args=@('test','--manifest-path',"$reviewCandidate/vendor/gpui/Cargo.toml",'--target-dir',$reviewTarget,'--no-default-features','--lib','svg_renderer','--locked','--offline') },
    @{ Name='final-root-tests'; Args=@('test','--manifest-path',"$reviewCandidate/Cargo.toml",'--tests','--locked','--offline') },
    @{ Name='final-harfrust-clippy'; Args=@('clippy','--manifest-path',"$reviewCandidate/vendor/harfrust/Cargo.toml",'--target-dir',$reviewTarget,'--lib','--locked','--offline','--','-D','warnings') },
    @{ Name='final-gpui-clippy'; Args=@('clippy','--manifest-path',"$reviewCandidate/vendor/gpui/Cargo.toml",'--target-dir',$reviewTarget,'--no-default-features','--lib','--locked','--offline','--','-D','warnings') },
    @{ Name='final-root-clippy'; Args=@('clippy','--manifest-path',"$reviewCandidate/Cargo.toml",'--all-targets','--locked','--offline','--','-D','warnings') },
    @{ Name='final-root-typecheck'; Args=@('check','--manifest-path',"$reviewCandidate/Cargo.toml",'--all-targets','--locked','--offline') },
    @{ Name='final-wgpu-typecheck'; Args=@('check','--manifest-path',"$reviewCandidate/validation/wgpu-consumer/Cargo.toml",'--target-dir',$reviewTarget,'--locked','--offline') },
    @{ Name='final-root-build'; Args=@('build','--manifest-path',"$reviewCandidate/Cargo.toml",'--locked','--offline') }
)
$reviewResults = @()
foreach ($reviewJob in $reviewJobs) {
    $reviewCommand = 'cargo ' + ($reviewJob.Args -join ' ')
    $reviewLog = "$reviewResearch/maintenance-independent-$($reviewJob.Name).log"
    Add-Content "$reviewResearch/maintenance-independent-status.txt" "RUNNING $(Get-Date -Format o): $reviewCommand"
    & cargo @($reviewJob.Args) *> $reviewLog
    $reviewExit = $LASTEXITCODE
    Add-Content "$reviewResearch/maintenance-independent-status.txt" "EXIT $reviewExit $(Get-Date -Format o): $reviewCommand"
    $reviewResults += @{ name=$reviewJob.Name; command=$reviewCommand; exit=$reviewExit; log=$reviewLog }
    $reviewResults | ConvertTo-Json -Depth 5 | Set-Content "$reviewResearch/maintenance-independent-final-command-results.json"
    Write-Output "$($reviewJob.Name): exit $reviewExit"
}
