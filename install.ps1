# git-notes universal installer for Windows PowerShell
$ErrorActionPreference = 'Stop'
$version = 'v0.1.0'
$repo = 'isaim0011/git-notes'
$binary = 'git-notes-windows-x86_64.exe'
$url = "https://github.com/$repo/releases/download/$version/$binary"

$installDir = Join-Path $HOME ".local\bin"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

$dest = Join-Path $installDir "git-notes.exe"
Write-Host "==> Downloading git-notes ($version) for Windows x86_64..." -ForegroundColor Cyan
Invoke-WebRequest -Uri $url -OutFile $dest

# Unblock file to clear SmartScreen Mark of the Web
try {
    Unblock-File -Path $dest
} catch {}

# Add to user PATH if not present
$userPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($userPath -split ';' -notcontains $installDir) {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", [EnvironmentVariableTarget]::User)
    Write-Host "==> Added $installDir to user PATH" -ForegroundColor Green
}

Write-Host "==> Installed git-notes to $dest" -ForegroundColor Green
Write-Host "==> Run 'git-notes --help' to get started!" -ForegroundColor Green
