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

$javacCommand = Get-Command javac -ErrorAction SilentlyContinue
if (-not $javacCommand) {
    throw "A Java 21+ JDK is required so javac can compile the JAR launcher."
}

Push-Location $repoRoot
try {
    & $cargo build --release --locked --package pumpkin --bin lithum-server
    if ($LASTEXITCODE -ne 0) {
        throw "The Lithum Rust server build failed with exit code $LASTEXITCODE."
    }

    $outputDirectory = Join-Path $repoRoot "dist"
    $stage = Join-Path $targetDirectory ("jar-stage-" + [Guid]::NewGuid().ToString("N"))
    $classes = $stage
    $native = Join-Path $stage "native"
    New-Item -ItemType Directory -Path $classes, $native -Force | Out-Null

    & $javacCommand.Source -d $classes (Join-Path $repoRoot "jar-launcher\LithumLauncher.java")
    if ($LASTEXITCODE -ne 0) {
        throw "The Lithum JAR launcher compilation failed with exit code $LASTEXITCODE."
    }

    Copy-Item -LiteralPath (Join-Path $targetDirectory "release\lithum-server.exe") `
        -Destination (Join-Path $native "lithum-server.exe")
    $metaInf = Join-Path $stage "META-INF"
    New-Item -ItemType Directory -Path $metaInf -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $repoRoot "jar-launcher\MANIFEST.MF") `
        -Destination (Join-Path $metaInf "MANIFEST.MF")

    New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
    $jarPath = Join-Path $outputDirectory "LithumServer.jar"
    if (Test-Path -LiteralPath $jarPath) {
        Remove-Item -LiteralPath $jarPath -Force
    }

    Add-Type -AssemblyName System.IO.Compression.FileSystem
    [System.IO.Compression.ZipFile]::CreateFromDirectory(
        $stage,
        $jarPath,
        [System.IO.Compression.CompressionLevel]::Optimal,
        $false
    )
    Copy-Item -LiteralPath (Join-Path $targetDirectory "release\lithum-server.exe") `
        -Destination (Join-Path $outputDirectory "lithum-server.exe") -Force

    Write-Host "Built $jarPath"
    Write-Host "Built $(Join-Path $outputDirectory 'lithum-server.exe')"
} finally {
    Pop-Location
}
