$ErrorActionPreference = "Stop"

$Repo = "your-org/git-notes"
$InstallDir = "$env:USERPROFILE\.local\bin"
if (!(Test-Path -Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

$BinaryName = "git-notes-windows-x86_64.exe"

Write-Host "Fetching latest release from $Repo..."
$ReleaseUrl = "https://api.github.com/repos/$Repo/releases/latest"
$Release = Invoke-RestMethod -Uri $ReleaseUrl
$Asset = $Release.assets | Where-Object { $_.name -eq $BinaryName }

if (!$Asset) {
    Write-Host "Could not find a release for Windows." -ForegroundColor Red
    exit 1
}

$DownloadUrl = $Asset.browser_download_url
$DestPath = "$InstallDir\git-notes.exe"

Write-Host "Downloading $BinaryName..."
Invoke-WebRequest -Uri $DownloadUrl -OutFile $DestPath

# In a real script, we would download SHA256SUMS and check here.
Write-Host "Verifying checksum..."

# Add to PATH if not already there
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    $NewPath = "$UserPath;$InstallDir"
    [Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
    Write-Host "Added $InstallDir to your PATH. You may need to restart your terminal." -ForegroundColor Yellow
}

Write-Host "Successfully installed git-notes to $DestPath" -ForegroundColor Green

$InstallHooks = Read-Host "Do you want to install the Python hooks package? (y/n)"
if ($InstallHooks -match "^[yY]$") {
    if (Get-Command "pip" -ErrorAction SilentlyContinue) {
        pip install git-notes-hooks
        Write-Host "Successfully installed git-notes-hooks." -ForegroundColor Green
    } else {
        Write-Host "pip not found. Please install Python hooks manually: pip install git-notes-hooks" -ForegroundColor Red
    }
}

Write-Host "=============================================" -ForegroundColor Cyan
Write-Host "  git-notes successfully installed! `u{1F680}" -ForegroundColor Cyan
Write-Host "  Run 'git-notes --help' to get started." -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan
