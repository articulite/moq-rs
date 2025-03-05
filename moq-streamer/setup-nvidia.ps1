# Setup script for NVIDIA hardware acceleration in moq-streamer
# This script sets up the environment for using NVIDIA hardware acceleration

# Check if NVIDIA_VIDEO_CODEC_SDK_DIR is set
if (-not $env:NVIDIA_VIDEO_CODEC_SDK_DIR) {
    # Set it to the temp directory in the moq-x265 project
    $env:NVIDIA_VIDEO_CODEC_SDK_DIR = Join-Path (Split-Path -Parent $PSScriptRoot) "moq-x265\temp"
    Write-Host "NVIDIA_VIDEO_CODEC_SDK_DIR set to $env:NVIDIA_VIDEO_CODEC_SDK_DIR"
}

# Check if the NVIDIA Video Codec SDK exists
$nvencHeader = Join-Path $env:NVIDIA_VIDEO_CODEC_SDK_DIR "Interface\nvEncodeAPI.h"
$nvcuvidHeader = Join-Path $env:NVIDIA_VIDEO_CODEC_SDK_DIR "Interface\nvcuvid.h"

if (-not (Test-Path $nvencHeader) -or -not (Test-Path $nvcuvidHeader)) {
    Write-Host "NVIDIA Video Codec SDK not found at $env:NVIDIA_VIDEO_CODEC_SDK_DIR" -ForegroundColor Red
    Write-Host "Please run the setup-nvidia.ps1 script in the moq-x265 directory first" -ForegroundColor Yellow
    exit 1
}

Write-Host "Found NVIDIA Video Codec SDK at $env:NVIDIA_VIDEO_CODEC_SDK_DIR" -ForegroundColor Green

# Add NVIDIA libraries to PATH
$nvLibPath = Join-Path $env:NVIDIA_VIDEO_CODEC_SDK_DIR "Lib\x64"
$env:PATH = "$nvLibPath;$env:PATH"
Write-Host "Added $nvLibPath to PATH" -ForegroundColor Green

# Add CUDA Toolkit bin directory to PATH
$cudaBinPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.8\bin"
if (Test-Path $cudaBinPath) {
    $env:PATH = "$cudaBinPath;$env:PATH"
    Write-Host "Added CUDA Toolkit bin directory to PATH: $cudaBinPath" -ForegroundColor Green
} else {
    Write-Host "CUDA Toolkit bin directory not found at $cudaBinPath" -ForegroundColor Yellow
    Write-Host "Please make sure CUDA Toolkit is installed" -ForegroundColor Yellow
}

# Add x265 to PATH
$x265Path = Join-Path (Split-Path -Parent $PSScriptRoot) "moq-x265\x265\bin\x64"
if (Test-Path $x265Path) {
    $env:PATH = "$x265Path;$env:PATH"
    Write-Host "Added x265 bin directory to PATH: $x265Path" -ForegroundColor Green
} else {
    Write-Host "x265 bin directory not found at $x265Path" -ForegroundColor Yellow
    Write-Host "Please run the setup-x265.ps1 script in the moq-x265 directory first" -ForegroundColor Yellow
}

# Build with hardware acceleration
Write-Host "Building moq-streamer with hardware acceleration..." -ForegroundColor Cyan
cargo build

# Run the streamer
Write-Host "You can now run the streamer with: cargo run -- --server https://localhost:4443 --name desktop" -ForegroundColor Green 