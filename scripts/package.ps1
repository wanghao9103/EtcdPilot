param(
    [string]$Version = "",
    [string]$Runtime = "",
    [ValidateSet("zip", "tar.gz")]
    [string]$ArchiveFormat = "",
    [string]$OutputDir = "",
    [switch]$SkipFrontendInstall,
    [switch]$SkipFrontendBuild,
    [switch]$SkipBackendBuild
)

$utf8 = New-Object System.Text.UTF8Encoding($false)
[Console]::OutputEncoding = $utf8
[Console]::InputEncoding = $utf8

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Resolve-Path (Join-Path $scriptDir "..")
$frontendDir = Join-Path $repoRoot "frontend"
$backendDir = Join-Path $repoRoot "backend"
$isWindowsHost = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
    [System.Runtime.InteropServices.OSPlatform]::Windows
)
$isMacHost = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
    [System.Runtime.InteropServices.OSPlatform]::OSX
)

if ([string]::IsNullOrWhiteSpace($Version)) {
    $Version = (git -C $repoRoot describe --tags --always 2>$null)
    if ([string]::IsNullOrWhiteSpace($Version)) {
        $Version = "dev"
    }
}

if ([string]::IsNullOrWhiteSpace($Runtime)) {
    if ($isWindowsHost) {
        $Runtime = "windows-x64"
    } elseif ($isMacHost) {
        $arch = (uname -m)
        $Runtime = if ($arch -eq "arm64") { "macos-arm64" } else { "macos-x64" }
    } else {
        $Runtime = "linux-x64"
    }
}

if ([string]::IsNullOrWhiteSpace($ArchiveFormat)) {
    $ArchiveFormat = if ($Runtime.StartsWith("windows")) { "zip" } else { "tar.gz" }
}

if ([string]::IsNullOrWhiteSpace($OutputDir)) {
    $OutputDir = Join-Path $repoRoot "artifacts"
}

$packageName = "EtcdPilot-$Version-$Runtime"
$buildRoot = Join-Path (Join-Path $repoRoot "build") "package"
$stageDir = Join-Path $buildRoot $packageName

Write-Host "Packaging $packageName"
Write-Host "repo: $repoRoot"
Write-Host "archive: $ArchiveFormat"

if (-not $SkipFrontendBuild) {
    Push-Location $frontendDir
    try {
        if (-not $SkipFrontendInstall) {
            if (Test-Path (Join-Path $frontendDir "package-lock.json")) {
                npm ci
            } else {
                npm install
            }
        }
        npm run build
    } finally {
        Pop-Location
    }
}

if (-not $SkipBackendBuild) {
    Push-Location $backendDir
    try {
        cargo build --release
    } finally {
        Pop-Location
    }
}

if (Test-Path $stageDir) {
    Remove-Item -Recurse -Force $stageDir
}
New-Item -ItemType Directory -Force -Path $stageDir | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path (Join-Path $stageDir "web") "dist") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $stageDir "config") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $stageDir "data") | Out-Null
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

$binaryName = if ($Runtime.StartsWith("windows")) { "etcdpilot-backend.exe" } else { "etcdpilot-backend" }
$binarySource = Join-Path (Join-Path (Join-Path $backendDir "target") "release") $binaryName
if (-not (Test-Path $binarySource)) {
    throw "Release binary not found: $binarySource"
}

$binaryTarget = if ($Runtime.StartsWith("windows")) { "etcdpilot.exe" } else { "etcdpilot" }
Copy-Item -Force $binarySource (Join-Path $stageDir $binaryTarget)
if (-not $Runtime.StartsWith("windows")) {
    chmod +x (Join-Path $stageDir $binaryTarget)
}

Copy-Item -Recurse -Force (Join-Path (Join-Path $frontendDir "dist") "*") (Join-Path (Join-Path $stageDir "web") "dist")
Copy-Item -Recurse -Force (Join-Path (Join-Path $repoRoot "config") "*") (Join-Path $stageDir "config")
Copy-Item -Force (Join-Path $repoRoot "LICENSE") (Join-Path $stageDir "LICENSE")
Copy-Item -Force (Join-Path $repoRoot "README.en.md") (Join-Path $stageDir "README.md")
Copy-Item -Force (Join-Path $repoRoot "README.zh-CN.md") (Join-Path $stageDir "README.zh-CN.md")

@"
`$env:ETCD_MANAGER_CONFIG = Join-Path `$PSScriptRoot "config/config.test.toml"
`$env:ETCD_MANAGER_SESSION_SECRET = if (`$env:ETCD_MANAGER_SESSION_SECRET) { `$env:ETCD_MANAGER_SESSION_SECRET } else { "etcdpilot-local-session-secret" }
Set-Location `$PSScriptRoot
./$binaryTarget
"@ | Set-Content -Encoding UTF8 (Join-Path $stageDir "start.ps1")

@"
#!/usr/bin/env sh
set -eu
cd "`$(dirname "`$0")"
export ETCD_MANAGER_CONFIG="`$PWD/config/config.test.toml"
export ETCD_MANAGER_SESSION_SECRET="`${ETCD_MANAGER_SESSION_SECRET:-etcdpilot-local-session-secret}"
./$binaryTarget
"@ | Set-Content -Encoding UTF8 (Join-Path $stageDir "start.sh")

if (-not $Runtime.StartsWith("windows")) {
    chmod +x (Join-Path $stageDir "start.sh")
}

@"
EtcdPilot portable package

Start:
- Windows PowerShell: ./start.ps1
- Linux/macOS: ./start.sh
- Or run ./$binaryTarget from this directory.

Default URL:
http://127.0.0.1:8080

Configuration:
- Test/local: config/config.test.toml
- Production template: config/config.prod.toml
- Override with ETCD_MANAGER_CONFIG.

Set ETCD_MANAGER_SESSION_SECRET before shared or production use.
"@ | Set-Content -Encoding UTF8 (Join-Path $stageDir "RUNNING.md")

$archivePath = if ($ArchiveFormat -eq "zip") {
    Join-Path $OutputDir "$packageName.zip"
} else {
    Join-Path $OutputDir "$packageName.tar.gz"
}

if (Test-Path $archivePath) {
    Remove-Item -Force $archivePath
}

if ($ArchiveFormat -eq "zip") {
    Compress-Archive -Path (Join-Path $stageDir "*") -DestinationPath $archivePath -Force
} else {
    Push-Location $buildRoot
    try {
        tar -czf $archivePath $packageName
    } finally {
        Pop-Location
    }
}

Write-Host "Created package: $archivePath"
