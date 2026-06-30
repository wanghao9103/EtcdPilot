param(
    [string]$Version = "dev",
    [string]$Runtime = "windows-x64"
)

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Resolve-Path (Join-Path $scriptDir "..\..")
$sourceDir = Join-Path (Join-Path (Join-Path $repoRoot "build") "package") "EtcdPilot-$Version-$Runtime"
$outputDir = Join-Path $repoRoot "artifacts"
$issPath = Join-Path $scriptDir "EtcdPilot.iss"

if (-not (Test-Path $sourceDir)) {
    throw "Portable package stage directory not found: $sourceDir"
}

$iscc = Get-Command iscc.exe -ErrorAction SilentlyContinue
$isccPath = if ($iscc) { $iscc.Source } else { "" }
if (-not $iscc) {
    $candidates = @(
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "${env:ProgramFiles}\Inno Setup 6\ISCC.exe",
        "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe"
    )
    foreach ($candidate in $candidates) {
        if (Test-Path $candidate) {
            $isccPath = (Get-Item $candidate).FullName
            break
        }
    }
}
if (-not $isccPath) {
    throw "Inno Setup compiler not found. Install Inno Setup 6 or ensure iscc.exe is on PATH."
}

New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
& $isccPath "/DAppVersion=$Version" "/DSourceDir=$sourceDir" "/DOutputDir=$outputDir" $issPath
