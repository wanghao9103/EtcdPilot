param(
    [string]$FrontendDir = "",
    [string]$BackendWebDist = "",
    [switch]$SkipInstall
)

# Keep terminal encoding stable for mixed environments
$utf8 = New-Object System.Text.UTF8Encoding($false)
[Console]::OutputEncoding = $utf8
[Console]::InputEncoding = $utf8

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Resolve-Path (Join-Path $scriptDir "..")

if ([string]::IsNullOrWhiteSpace($FrontendDir)) {
    $FrontendDir = Join-Path $repoRoot "frontend"
}
if ([string]::IsNullOrWhiteSpace($BackendWebDist)) {
    $BackendWebDist = Join-Path $repoRoot "backend\web\dist"
}

Write-Host "EtcdPilot local web sync start"
Write-Host "frontend: $FrontendDir"
Write-Host "backend dist: $BackendWebDist"

if (-not (Test-Path $FrontendDir)) {
    throw "Frontend directory does not exist: $FrontendDir"
}

if (-not $SkipInstall) {
    Write-Host "Installing frontend dependencies..."
    Push-Location $FrontendDir
    npm install
    Pop-Location
}

Write-Host "Building frontend..."
Push-Location $FrontendDir
npm run build
Pop-Location

if (Test-Path $BackendWebDist) {
    Remove-Item -Recurse -Force $BackendWebDist
}
New-Item -ItemType Directory -Force -Path $BackendWebDist | Out-Null

Write-Host "Sync dist to backend..."
Copy-Item -Recurse -Force (Join-Path $FrontendDir "dist\*") $BackendWebDist

Write-Host "Synced successfully."
