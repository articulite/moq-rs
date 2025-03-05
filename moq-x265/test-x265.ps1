# Test script for x265 encoder and decoder
# This script tests both software and hardware acceleration

# Create output directory if it doesn't exist
$outputDir = Join-Path $PSScriptRoot "output"
if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir | Out-Null
    Write-Host "Created output directory: $outputDir" -ForegroundColor Green
}

# Test software encoding/decoding
Write-Host "Testing x265 software encoding/decoding..." -ForegroundColor Cyan
cargo run --example test_x265

# Check if hardware acceleration is available
Write-Host "Checking for NVIDIA hardware acceleration..." -ForegroundColor Cyan

# Set NVIDIA_VIDEO_CODEC_SDK_DIR if not already set
if (-not $env:NVIDIA_VIDEO_CODEC_SDK_DIR) {
    $env:NVIDIA_VIDEO_CODEC_SDK_DIR = Join-Path $PSScriptRoot "temp"
    Write-Host "NVIDIA_VIDEO_CODEC_SDK_DIR set to $env:NVIDIA_VIDEO_CODEC_SDK_DIR" -ForegroundColor Yellow
}

# Check if the NVIDIA Video Codec SDK exists
$nvencHeader = Join-Path $env:NVIDIA_VIDEO_CODEC_SDK_DIR "Interface\nvEncodeAPI.h"
$nvcuvidHeader = Join-Path $env:NVIDIA_VIDEO_CODEC_SDK_DIR "Interface\nvcuvid.h"

if ((Test-Path $nvencHeader) -and (Test-Path $nvcuvidHeader)) {
    Write-Host "NVIDIA Video Codec SDK found at $env:NVIDIA_VIDEO_CODEC_SDK_DIR" -ForegroundColor Green
    
    # Add NVIDIA libraries to PATH
    $nvLibPath = Join-Path $env:NVIDIA_VIDEO_CODEC_SDK_DIR "Lib\x64"
    $env:PATH = "$nvLibPath;$env:PATH"
    Write-Host "Added $nvLibPath to PATH" -ForegroundColor Green
    
    # Test hardware acceleration
    Write-Host "Testing NVIDIA hardware acceleration..." -ForegroundColor Cyan
    cargo run --features hardware-accel --example test_nvidia
    
    # Compare results
    Write-Host "Comparing software vs hardware results..." -ForegroundColor Cyan
    
    # Check if encoded files exist
    $swEncodedFile = Join-Path $outputDir "encoded.h265"
    $hwEncodedFile = Join-Path $outputDir "nvidia_encoded.h265"
    
    if ((Test-Path $swEncodedFile) -and (Test-Path $hwEncodedFile)) {
        $swSize = (Get-Item $swEncodedFile).Length
        $hwSize = (Get-Item $hwEncodedFile).Length
        
        Write-Host "Software encoded file size: $($swSize/1KB) KB" -ForegroundColor Yellow
        Write-Host "Hardware encoded file size: $($hwSize/1KB) KB" -ForegroundColor Yellow
        
        $ratio = [math]::Round(($hwSize / $swSize) * 100, 2)
        Write-Host "Hardware/Software size ratio: $ratio%" -ForegroundColor Yellow
    } else {
        Write-Host "Encoded files not found for comparison" -ForegroundColor Red
    }
    
    # Check if decoded images exist
    $swDecodedFile = Join-Path $outputDir "decoded_0.png"
    $hwDecodedFile = Join-Path $outputDir "nvidia_decoded_0.png"
    
    if ((Test-Path $swDecodedFile) -and (Test-Path $hwDecodedFile)) {
        Write-Host "Decoded images available for visual comparison:" -ForegroundColor Yellow
        Write-Host "Software decoded: $swDecodedFile" -ForegroundColor Yellow
        Write-Host "Hardware decoded: $hwDecodedFile" -ForegroundColor Yellow
    } else {
        Write-Host "Decoded images not found for comparison" -ForegroundColor Red
    }
} else {
    Write-Host "NVIDIA Video Codec SDK not found at $env:NVIDIA_VIDEO_CODEC_SDK_DIR" -ForegroundColor Red
    Write-Host "Hardware acceleration test skipped" -ForegroundColor Yellow
    Write-Host "Please download the NVIDIA Video Codec SDK from https://developer.nvidia.com/nvidia-video-codec-sdk/download" -ForegroundColor Yellow
    Write-Host "and extract it to $env:NVIDIA_VIDEO_CODEC_SDK_DIR" -ForegroundColor Yellow
}

Write-Host "All tests completed!" -ForegroundColor Green 