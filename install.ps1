# Installs IVE for the current user on Windows: builds it, writes the launcher %LOCALAPPDATA%\ive\ive.ps1, adds an
# "IVE" shortcut to the Start menu and starts IVE. IVE runs as a profile named after the user, with the Restart
# button: it rebuilds this checkout and restarts IVE. Run it again to rebuild and reinstall.
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
$launcher = Join-Path $env:LOCALAPPDATA 'ive\ive.ps1'
$shortcut = Join-Path ([Environment]::GetFolderPath('Programs')) 'IVE.lnk'
# the id src/main.rs gives IVE's windows: the task bar groups them under the shortcut, and pinning one pins it
$appUserModelId = 'WarsawDynamics.IVE'

# `--profile` takes letters, digits, `-` and `_`, up to 64
function Get-ProfileName {
    $name = ($env:USERNAME.ToLowerInvariant() -replace '[^a-z0-9_-]', '-')
    if ($name.Length -gt 64) {
        $name = $name.Substring(0, 64)
    }
    if ($name) { $name } else { 'user' }
}

function Format-Quoted([string]$text) {
    "'" + ($text -replace "'", "''") + "'"
}

# a running exe cannot be overwritten, only renamed: as restart.ps1 does, the running ones move aside and a copy takes
# each one's place, so the terminals of the running IVE still find ive-cli.exe
function Move-RunningExesAside([string]$release) {
    $stamp = Get-Date -Format 'yyyyMMddHHmmss'
    foreach ($name in 'ive.exe', 'ive-cli.exe') {
        $running = Join-Path $release $name
        Get-ChildItem "$running*.old" -ErrorAction SilentlyContinue | Remove-Item -ErrorAction SilentlyContinue
        if (Test-Path $running) {
            Move-Item $running "$running.$stamp.old"
            Copy-Item "$running.$stamp.old" $running
        }
    }
}

# a Start menu start has no shell profile, so the launcher sets the toolchain this install was built with
function Write-Launcher([string]$profileName) {
    $toolchain = foreach ($name in 'CARGO_HOME', 'RUSTUP_HOME', 'CARGO_TARGET_DIR') {
        $value = [Environment]::GetEnvironmentVariable($name)
        if ($value) { "`$env:$name = $(Format-Quoted $value)" }
    }
    $cargoBin = Split-Path (Get-Command cargo).Source
    New-Item -ItemType Directory -Force (Split-Path $launcher) | Out-Null
    Set-Content -Encoding UTF8 $launcher @"
# IVE with profile $profileName and the Restart button, written by $PSCommandPath.
# The Start menu shortcut IVE runs it; the Restart button runs it with --replace PID (see restart.ps1).
$($toolchain -join "`n")
`$env:Path = $(Format-Quoted "$cargoBin;") + `$env:Path
# IVE started from a Claude session would hand this to every terminal, and a Claude started there saves no transcript
Remove-Item Env:CLAUDE_CODE_CHILD_SESSION -ErrorAction SilentlyContinue
`$IveLauncher = `$PSCommandPath
`$IveFlags = @('--profile', '$profileName')
. $(Format-Quoted (Join-Path $PSScriptRoot 'restart.ps1')) @args
"@
}

Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.ComTypes;

public static class ShortcutAppId {
    [ComImport, Guid("00021401-0000-0000-C000-000000000046")]
    class ShellLink {}

    [ComImport, Guid("886D8EEB-8CF2-4446-8D02-CDBA1DBDCF99"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface IPropertyStore {
        void GetCount(out uint count);
        void GetAt(uint index, out PropertyKey key);
        void GetValue(ref PropertyKey key, IntPtr value);
        void SetValue(ref PropertyKey key, ref PropVariant value);
        void Commit();
    }

    [StructLayout(LayoutKind.Sequential, Pack = 4)]
    struct PropertyKey { public Guid FormatId; public uint PropertyId; }

    [StructLayout(LayoutKind.Explicit)]
    struct PropVariant { [FieldOffset(0)] public ushort Type; [FieldOffset(8)] public IntPtr Text; }

    // PKEY_AppUserModel_ID, a VT_LPWSTR
    public static void Set(string path, string id) {
        var link = (IPersistFile)new ShellLink();
        link.Load(path, 2);
        var key = new PropertyKey { FormatId = new Guid("9F4C2855-9F79-4B39-A8D0-E1D42DE1D5F3"), PropertyId = 5 };
        var value = new PropVariant { Type = 31, Text = Marshal.StringToCoTaskMemUni(id) };
        try {
            var store = (IPropertyStore)link;
            store.SetValue(ref key, ref value);
            store.Commit();
        } finally {
            Marshal.FreeCoTaskMem(value.Text);
        }
        link.Save(path, true);
    }
}
'@

function Write-Shortcut {
    $link = (New-Object -ComObject WScript.Shell).CreateShortcut($shortcut)
    $link.TargetPath = (Get-Command powershell).Source
    $link.Arguments = "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File `"$launcher`""
    $link.IconLocation = Join-Path $PSScriptRoot 'desktop\icons\ive.ico'
    $link.Description = 'IVE'
    $link.Save()
    [ShortcutAppId]::Set($shortcut, $appUserModelId)
}

function Start-Ive([string]$profileName) {
    $running = Get-CimInstance Win32_Process -Filter "Name = 'ive.exe'" | Where-Object { $_.CommandLine -match "--profile $profileName( |$)" }
    if ($running) {
        Write-Output "IVE already runs as profile ${profileName}: press its Restart button to switch to the new build"
        return
    }
    Start-Process powershell -WindowStyle Hidden -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "`"$launcher`""
    Write-Output "started IVE (profile $profileName)"
}

& .\check-build.ps1
if ($LASTEXITCODE) {
    exit $LASTEXITCODE
}
$target = (cargo metadata --format-version 1 --no-deps | ConvertFrom-Json).target_directory
Move-RunningExesAside (Join-Path $target 'release')
cargo build --release
if ($LASTEXITCODE) {
    exit $LASTEXITCODE
}
$profileName = Get-ProfileName
Write-Launcher $profileName
Write-Shortcut
Start-Ive $profileName
Write-Output "installed $launcher; find IVE in the Start menu"
