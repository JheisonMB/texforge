# install.ps1 — download and install texforge on Windows
# tectonic (LaTeX engine) is installed automatically on first build
# Usage: irm https://raw.githubusercontent.com/JheisonMB/texforge/main/install.ps1 | iex
#
# Options (set as env vars before running):
#   $env:VERSION    = "0.1.0"           # pin a specific version
#   $env:INSTALL_DIR = "C:\my\bin"      # custom install directory

$ErrorActionPreference = "Stop"

$Repo       = "JheisonMB/texforge"
$Binary     = "texforge.exe"
$Target     = "x86_64-pc-windows-msvc"
$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { "$env:USERPROFILE\.local\bin" }

function Info($label, $msg) {
    Write-Host "  " -NoNewline
    Write-Host $label -ForegroundColor Blue -NoNewline
    Write-Host " $msg"
}

function Fail($msg) {
    Write-Host "  error: $msg" -ForegroundColor Red
    exit 1
}

# --- resolve version ---
if ($env:VERSION) {
    $Tag = "v$($env:VERSION)"
    Info "version" "$Tag (pinned)"
} else {
    $latest = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
    $Tag = $latest.tag_name
    if (-not $Tag) { Fail "Could not resolve latest release tag" }
    Info "version" "$Tag (latest)"
}

# --- download ---
$Archive = "texforge-$Tag-$Target.zip"
$Url     = "https://github.com/$Repo/releases/download/$Tag/$Archive"
$Tmp     = Join-Path $env:TEMP "texforge-install"
New-Item -ItemType Directory -Force -Path $Tmp | Out-Null

Info "download" $Url
try {
    Invoke-WebRequest -Uri $Url -OutFile "$Tmp\$Archive" -UseBasicParsing
} catch {
    Fail "Download failed: $_`nURL: $Url"
}

# --- extract ---
Expand-Archive -Path "$Tmp\$Archive" -DestinationPath $Tmp -Force
$extracted = Join-Path $Tmp $Binary
if (-not (Test-Path $extracted)) { Fail "Binary not found in archive" }

# --- install ---
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item $extracted "$InstallDir\$Binary" -Force
Info "installed" "$InstallDir\$Binary"

# --- ensure PATH ---
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$InstallDir;$userPath", "User")
    $env:PATH = "$InstallDir;$env:PATH"
    Info "updated" "User PATH"
}

# --- cleanup ---
Remove-Item $Tmp -Recurse -Force

# --- verify ---
$ver = & "$InstallDir\$Binary" --version 2>$null
Info "done" $ver
Write-Host ""
Info "ready" "Run 'texforge new my-project' to get started!"
Info "note"  "tectonic (LaTeX engine) will be installed automatically on first build"
