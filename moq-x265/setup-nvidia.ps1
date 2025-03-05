# Setup script for NVIDIA hardware acceleration
# This script sets up the environment for using NVIDIA hardware acceleration with moq-x265

# Check if NVIDIA_VIDEO_CODEC_SDK_DIR is set
if (-not $env:NVIDIA_VIDEO_CODEC_SDK_DIR) {
    # Set it to the temp directory in the project
    $env:NVIDIA_VIDEO_CODEC_SDK_DIR = Join-Path $PSScriptRoot "temp"
    Write-Host "NVIDIA_VIDEO_CODEC_SDK_DIR set to $env:NVIDIA_VIDEO_CODEC_SDK_DIR"
}

# Check if the NVIDIA Video Codec SDK exists
$nvencHeader = Join-Path $env:NVIDIA_VIDEO_CODEC_SDK_DIR "Interface\nvEncodeAPI.h"
$nvcuvidHeader = Join-Path $env:NVIDIA_VIDEO_CODEC_SDK_DIR "Interface\nvcuvid.h"

if (-not (Test-Path $nvencHeader) -or -not (Test-Path $nvcuvidHeader)) {
    Write-Host "NVIDIA Video Codec SDK not found at $env:NVIDIA_VIDEO_CODEC_SDK_DIR" -ForegroundColor Red
    Write-Host "Please download the NVIDIA Video Codec SDK from https://developer.nvidia.com/nvidia-video-codec-sdk/download" -ForegroundColor Yellow
    Write-Host "and extract it to $env:NVIDIA_VIDEO_CODEC_SDK_DIR" -ForegroundColor Yellow
    exit 1
}

Write-Host "Found NVIDIA Video Codec SDK at $env:NVIDIA_VIDEO_CODEC_SDK_DIR" -ForegroundColor Green

# Add NVIDIA libraries to PATH
$nvLibPath = Join-Path $env:NVIDIA_VIDEO_CODEC_SDK_DIR "Lib\x64"
$env:PATH = "$nvLibPath;$env:PATH"
Write-Host "Added $nvLibPath to PATH" -ForegroundColor Green

# Build with hardware acceleration
Write-Host "Building moq-x265 with hardware acceleration..." -ForegroundColor Cyan
cargo build --features hardware-accel

# Run the NVIDIA test example
Write-Host "Running NVIDIA hardware acceleration test..." -ForegroundColor Cyan
cargo run --features hardware-accel --example test_nvidia 