param(
    [string]$FilePath = "target\debug\git-notes.exe"
)

$cert = Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert | Where-Object { $_.Subject -match "CN=Bimo" } | Select-Object -First 1
if (-not $cert) {
    Write-Host "Creating Code Signing Certificate for Bimo in CurrentUser\My..." -ForegroundColor Cyan
    $cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject "CN=Bimo" -CertStoreLocation "Cert:\CurrentUser\My" -NotAfter (Get-Date).AddYears(5)
}

if (Test-Path $FilePath) {
    Write-Host "Signing $FilePath with Bimo certificate..." -ForegroundColor Cyan
    $sig = Set-AuthenticodeSignature -FilePath $FilePath -Certificate $cert
    Write-Host "Signature Status: $($sig.Status)" -ForegroundColor Green
    Write-Host "Signer Subject:   $($sig.SignerCertificate.Subject)" -ForegroundColor Green
} else {
    Write-Host "File not found: $FilePath" -ForegroundColor Yellow
}
