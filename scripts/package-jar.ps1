$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$outputDirectory = Join-Path $repoRoot "dist"
$windowsServer = Join-Path $outputDirectory "lithummc-server.exe"
$linuxServer = Join-Path $outputDirectory "lithummc-server-linux-x86_64"
$javacCommand = Get-Command javac -ErrorAction SilentlyContinue

if (-not $javacCommand) {
    throw "A Java 21+ JDK is required to compile and package the JAR."
}
if (-not (Test-Path -LiteralPath $windowsServer -PathType Leaf)) {
    throw "The Windows server executable is missing at $windowsServer. Build it first."
}
if (-not (Test-Path -LiteralPath $linuxServer -PathType Leaf)) {
    throw "The Linux server executable is missing at $linuxServer. Download the LithumMC-linux-x86_64 Actions artifact first."
}

$stage = Join-Path (Join-Path $repoRoot "target") ("jar-stage-" + [Guid]::NewGuid().ToString("N"))
$classes = $stage
$native = Join-Path $stage "native"
New-Item -ItemType Directory -Path $native -Force | Out-Null

Push-Location $repoRoot
try {
    & $javacCommand.Source -d $classes (Join-Path $repoRoot "jar-launcher\LithumMCLauncher.java")
    if ($LASTEXITCODE -ne 0) {
        throw "The LithumMC JAR launcher compilation failed with exit code $LASTEXITCODE."
    }

    Copy-Item -LiteralPath $windowsServer -Destination (Join-Path $native "lithummc-server.exe")
    Copy-Item -LiteralPath $linuxServer -Destination (Join-Path $native "lithummc-server-linux-x86_64")

    $metaInf = Join-Path $stage "META-INF"
    New-Item -ItemType Directory -Path $metaInf -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $repoRoot "jar-launcher\MANIFEST.MF") `
        -Destination (Join-Path $metaInf "MANIFEST.MF")

    $jarPath = Join-Path $outputDirectory "LithumMC.jar"
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

    $hash = (Get-FileHash -LiteralPath $jarPath -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  LithumMC.jar" | Set-Content -LiteralPath "$jarPath.sha256" -Encoding ascii
    Write-Host "Built $jarPath with Windows x86_64 and Linux x86_64 server runtimes."
} finally {
    Pop-Location
}
