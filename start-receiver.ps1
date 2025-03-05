# Start MoQ Receiver
# This script starts the MoQ receiver with the correct environment variables and parameters
# Hardware acceleration enabled through environment variables

$ErrorActionPreference = "Stop"

# Set environment variables for x265
$env:X265_DIR = "$PSScriptRoot\moq-x265\x265"
$env:LIB = "$env:LIB;$PSScriptRoot\moq-receiver"
$env:PATH = "$env:X265_DIR\bin\x64;$env:PATH"

# Set environment variables for NVIDIA hardware acceleration
$nvcodecDir = "$PSScriptRoot\moq-x265\temp"
if (Test-Path "$nvcodecDir\Interface\nvEncodeAPI.h") {
    $env:NVIDIA_VIDEO_CODEC_SDK_DIR = $nvcodecDir
    Write-Host "Set NVIDIA_VIDEO_CODEC_SDK_DIR to $nvcodecDir" -ForegroundColor Green
    
    # Add NVIDIA libraries to PATH
    $nvLibPath = "$nvcodecDir\Lib\x64"
    if (Test-Path $nvLibPath) {
        $env:PATH = "$nvLibPath;$env:PATH"
        Write-Host "Added $nvLibPath to PATH" -ForegroundColor Green
    }
}

# Check for CUDA Toolkit
$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.8"
if (Test-Path $cudaPath) {
    # Add CUDA bin to PATH
    $cudaBinPath = "$cudaPath\bin"
    $env:PATH = "$cudaBinPath;$env:PATH"
    Write-Host "Added CUDA Toolkit bin directory to PATH: $cudaBinPath" -ForegroundColor Green
}

# Change to the receiver directory
Set-Location -Path "$PSScriptRoot\moq-receiver"

# Run the receiver with hardware acceleration enabled via environment variables
Write-Host "Starting MoQ Receiver with hardware acceleration..." -ForegroundColor Cyan
Write-Host "X265_DIR: $env:X265_DIR"
Write-Host "NVIDIA_VIDEO_CODEC_SDK_DIR: $env:NVIDIA_VIDEO_CODEC_SDK_DIR"
cargo run -- --server https://127.0.0.1:4443 --name desktop --width 640 --height 480 --latency 500 --disable-verify 