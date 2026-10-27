# Start script for `ive --restart` on Windows. Run `.\restart.ps1` once from a terminal: it starts IVE with the Restart
# button. The button types `restart.ps1 --replace PID` into a terminal tab: it builds IVE and, after a good build,
# tells the IVE with PID it is replaced (it saves its tabs and quits; its terminals keep running) and starts the new
# one, which takes the terminals over. A failed build stays in that tab. To start IVE with other flags (--profile,
# --mcp, a folder), dot-source this script from a launcher that sets $IveLauncher to its own path and $IveFlags, as
# the one install.ps1 writes does: the Restart button then runs the launcher.
if (-not $IveLauncher) {
    $IveLauncher = $PSCommandPath
}
$flags = @("--restart=$IveLauncher") + @($IveFlags | Where-Object { $_ })
Set-Location $PSScriptRoot
$target = (cargo metadata --format-version 1 --no-deps | ConvertFrom-Json).target_directory
# without it there is nothing to build or start, and the IVE to replace must keep running
if (-not $target) {
    Write-Error 'restart.ps1: cannot find the build folder with cargo metadata; is cargo on PATH?'
    exit 1
}
$exe = Join-Path $target 'release\ive.exe'

if (-not $args) {
    if (-not (Test-Path $exe)) {
        cargo build --release
    }
    & $exe @flags
    exit $LASTEXITCODE
}
if ($args[0] -ne '--replace') {
    Write-Error "restart.ps1: unexpected argument $($args[0]); pass IVE's flags through a launcher (see the top of this script)"
    exit 2
}
$old = $args[1]
if (-not $old) {
    Write-Error 'restart.ps1: --replace needs the process id of the IVE to replace'
    exit 2
}
# a running exe cannot be overwritten, but it can be renamed: the running ones move aside under a name of their own
# (terminals of earlier builds keep running theirs), those no process runs any more go, and a copy takes each one's
# place, so terminals this IVE starts during the build, or after a failed one, still find ive-cli.exe
$stamp = Get-Date -Format 'yyyyMMddHHmmss'
foreach ($name in 'ive.exe', 'ive-cli.exe') {
    $running = Join-Path $target "release\$name"
    Get-ChildItem "$running*.old" -ErrorAction SilentlyContinue | Remove-Item -ErrorAction SilentlyContinue
    if (Test-Path $running) {
        Move-Item $running "$running.$stamp.old"
        Copy-Item "$running.$stamp.old" $running
    }
}
cargo build --release
if ($LASTEXITCODE) {
    exit $LASTEXITCODE
}
# make sure the process is still IVE before telling it to quit
$process = Get-Process -Id $old -ErrorAction SilentlyContinue
if ($process.ProcessName -ne 'ive') {
    Write-Error "restart.ps1: process $old is not IVE (any more); start IVE with .\restart.ps1"
    exit 1
}
# the old IVE may end this terminal as it quits, so a hidden PowerShell outlives it: it waits for the old IVE to
# save its tabs and quit, then starts the new one through ive-cli --detach, outside this terminal's job, whose end
# would otherwise end the new IVE too
$cli = Join-Path $target 'release\ive-cli.exe'
$quotedFlags = ($flags | ForEach-Object { "'" + ($_ -replace "'", "''") + "'" }) -join ' '
$finish = "Wait-Process -Id $old -Timeout 10 -ErrorAction SilentlyContinue; & '$cli' --detach '$exe' $quotedFlags"
# encoded, as Start-Process passes a plain -Command on without its quotes
$encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($finish))
Start-Process powershell -WindowStyle Hidden -ArgumentList '-NoProfile', '-EncodedCommand', $encoded
# the event IVE waits on stands in for SIGUSR1; an IVE built before it is asked to close its window, which with
# terminals running shows its quit prompt
try {
    [void][System.Threading.EventWaitHandle]::OpenExisting("Local\ive-replace-$old").Set()
} catch {
    if (-not $process.CloseMainWindow()) {
        Write-Error "restart.ps1: IVE $old has no window to close"
        exit 1
    }
}
