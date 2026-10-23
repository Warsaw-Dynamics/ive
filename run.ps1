# Builds and starts IVE on Windows; arguments go to IVE, e.g. .\run.ps1 -Release --profile agent-x --background .
# Unlike run.sh there is no `--` to split cargo's flags from IVE's: PowerShell drops it before the script sees it.
param([switch]$Release)
Set-Location $PSScriptRoot
& .\check-build.ps1
if ($LASTEXITCODE) {
    exit $LASTEXITCODE
}
$cargo = @('run') + @(if ($Release) { '--release' }) + @('--') + $args
cargo @cargo
exit $LASTEXITCODE
