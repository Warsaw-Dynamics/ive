# Checks that IVE can be built here on Windows: cargo, and the Windows SDK whose fxc.exe compiles gpui's shaders.
# The MSVC linker is not checked: cargo's own error names it.
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    $version = (Get-Content (Join-Path $PSScriptRoot '.tool-versions') | Select-String '^rust ').Line -replace '^rust ', ''
    Write-Error "cargo not found: install Rust $version with rustup (https://rustup.rs), MSVC toolchain"
    exit 1
}
if ($env:GPUI_FXC_PATH) {
    exit 0
}
$sdk = (Get-ItemProperty 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Microsoft SDKs\Windows\v10.0' -ErrorAction SilentlyContinue).InstallationFolder
if (-not ($sdk -and (Get-ChildItem (Join-Path $sdk 'bin\*\x64\fxc.exe') -ErrorAction SilentlyContinue))) {
    Write-Error 'Windows SDK not found (gpui needs its fxc.exe): install Visual Studio 2022 Build Tools with "Desktop development with C++" and a Windows 10/11 SDK, or set GPUI_FXC_PATH'
    exit 1
}
exit 0
