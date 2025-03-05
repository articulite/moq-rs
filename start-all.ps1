# Start All MoQ Components
# This script starts all MoQ components (relay, streamer, and receiver) in the correct order

$ErrorActionPreference = "Stop"

# Function to check if a process is running
function Test-ProcessRunning {
    param (
        [Parameter(Mandatory=$true)]
        [string]$ProcessName
    )
    
    $process = Get-Process -Name $ProcessName -ErrorAction SilentlyContinue
    return ($null -ne $process)
}

# Check if NVIDIA hardware acceleration is available
$useHardwareAccel = $false
if (Test-Path "$PSScriptRoot\moq-x265\temp\Interface\nvEncodeAPI.h") {
    Write-Host "NVIDIA Video Codec SDK found, enabling hardware acceleration" -ForegroundColor Green
    $useHardwareAccel = $true
    
    # Set environment variables for hardware acceleration
    $env:NVIDIA_VIDEO_CODEC_SDK_DIR = "$PSScriptRoot\moq-x265\temp"
    
    # Add NVIDIA libraries to PATH
    $nvLibPath = Join-Path $env:NVIDIA_VIDEO_CODEC_SDK_DIR "Lib\x64"
    $env:PATH = "$nvLibPath;$env:PATH"
    
    # Add CUDA Toolkit bin directory to PATH if it exists
    $cudaBinPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.8\bin"
    if (Test-Path $cudaBinPath) {
        $env:PATH = "$cudaBinPath;$env:PATH"
        Write-Host "Added CUDA Toolkit to PATH: $cudaBinPath" -ForegroundColor Green
    } else {
        Write-Host "CUDA Toolkit not found at $cudaBinPath, hardware acceleration may not work properly" -ForegroundColor Yellow
    }
} else {
    Write-Host "NVIDIA Video Codec SDK not found, using software encoding/decoding" -ForegroundColor Yellow
}

# Set common environment variables
$env:X265_DIR = "$PSScriptRoot\moq-x265\x265"
$env:LIB = "$env:LIB;$PSScriptRoot\moq-receiver"
$env:PATH = "$env:X265_DIR\bin\x64;$env:PATH"

# Start the relay server in a new PowerShell window
Write-Host "Starting MoQ Relay Server..." -ForegroundColor Cyan
Start-Process powershell -ArgumentList "-NoExit -File $PSScriptRoot\start-relay.ps1"

# Wait for the relay server to start
Write-Host "Waiting for relay server to start..." -ForegroundColor Cyan
Start-Sleep -Seconds 5

# Start the streamer in a new PowerShell window
Write-Host "Starting MoQ Streamer..." -ForegroundColor Cyan
if ($useHardwareAccel) {
    Write-Host "Using hardware acceleration for encoding" -ForegroundColor Green
    # Run the setup-nvidia script first to ensure proper environment setup
    Start-Process powershell -ArgumentList "-NoExit -Command `"cd $PSScriptRoot\moq-streamer; .\setup-nvidia.ps1; cargo run -- --server https://127.0.0.1:4443 --name desktop --width 640 --height 480 --bitrate 2000 --fps 30 --screen 0 --tls-disable-verify`""
} else {
    Start-Process powershell -ArgumentList "-NoExit -File $PSScriptRoot\start-streamer.ps1"
}

# Wait for the streamer to start
Write-Host "Waiting for streamer to start..." -ForegroundColor Cyan
Start-Sleep -Seconds 5

# Start the receiver in a new PowerShell window
Write-Host "Starting MoQ Receiver..." -ForegroundColor Cyan
if ($useHardwareAccel) {
    Write-Host "Using hardware acceleration for decoding" -ForegroundColor Green
    # Run the setup-nvidia script first to ensure proper environment setup
    Start-Process powershell -ArgumentList "-NoExit -Command `"cd $PSScriptRoot\moq-receiver; .\setup-nvidia.ps1; cargo run -- --server https://127.0.0.1:4443 --name desktop --width 640 --height 480 --latency 500 --disable-verify`""
} else {
    Start-Process powershell -ArgumentList "-NoExit -File $PSScriptRoot\start-receiver.ps1"
}

Write-Host "All components started. Check the individual windows for status." -ForegroundColor Green
Write-Host "Press Ctrl+C in each window to stop the components when done." -ForegroundColor Yellow 