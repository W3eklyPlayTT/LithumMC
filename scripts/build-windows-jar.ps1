$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$targetDirectory = if ($env:CARGO_TARGET_DIR) {
    [System.IO.Path]::GetFullPath($env:CARGO_TARGET_DIR)
} else {
    Join-Path $repoRoot "target"
}
$env:CARGO_TARGET_DIR = $targetDirectory
$cargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
if ($cargoCommand) {
    $cargo = $cargoCommand.Source
} else {
    $cargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
}

if (-not (Test-Path -LiteralPath $cargo -PathType Leaf)) {
    throw "Rust Cargo was not found. Install Rust 1.97 or newer and run this script again."
}

Push-Location $repoRoot
try {
    & $cargo build --release --locked --package pumpkin --bin lithum-server
    if ($LASTEXITCODE -ne 0) {
        throw "The Lithum Windows server build failed with exit code $LASTEXITCODE."
    }

    $outputDirectory = Join-Path $repoRoot "dist"
    New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $targetDirectory "release\lithum-server.exe") `
        -Destination (Join-Path $outputDirectory "lithum-server.exe") -Force

    & (Join-Path $PSScriptRoot "package-jar.ps1")
    if ($LASTEXITCODE -ne 0) {
        throw "The cross-platform JAR packaging failed with exit code $LASTEXITCODE."
    }
} finally {
    Pop-Location
}
