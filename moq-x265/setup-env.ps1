param([switch]$permanent)

# Setup environment script for moq-rs project
# This script sets the necessary environment variables for x265 and NVIDIA hardware acceleration

# Set x265 environment variable
$x265Dir = "$PSScriptRoot\x265"
if (Test-Path $x265Dir) {
    $env:X265_DIR = $x265Dir
    Write-Host "Set X265_DIR to $x265Dir" -ForegroundColor Green
    
    # Add x265 bin directory to PATH
    $binDir = "$x265Dir\bin\x64"
    if (Test-Path $binDir) {
        $env:PATH = "$binDir;$env:PATH"
        Write-Host "Added $binDir to PATH" -ForegroundColor Green
    } else {
        Write-Host "Warning: x265 bin directory not found at $binDir" -ForegroundColor Yellow
    }
} else {
    Write-Host "Warning: x265 directory not found at $x265Dir" -ForegroundColor Yellow
    Write-Host "Run setup-x265.ps1 to download and install x265" -ForegroundColor Yellow
}

# Set NVIDIA Video Codec SDK environment variable
$nvcodecDir = "$PSScriptRoot\temp"
if (Test-Path "$nvcodecDir\Interface\nvEncodeAPI.h") {
    $env:NVIDIA_VIDEO_CODEC_SDK_DIR = $nvcodecDir
    Write-Host "Set NVIDIA_VIDEO_CODEC_SDK_DIR to $nvcodecDir" -ForegroundColor Green
    
    # Add NVIDIA libraries to PATH
    $nvLibPath = "$nvcodecDir\Lib\x64"
    if (Test-Path $nvLibPath) {
        $env:PATH = "$nvLibPath;$env:PATH"
        Write-Host "Added $nvLibPath to PATH" -ForegroundColor Green
    }
} else {
    Write-Host "Warning: NVIDIA Video Codec SDK not found at $nvcodecDir" -ForegroundColor Yellow
    Write-Host "Run setup-nvidia.ps1 or download the SDK manually" -ForegroundColor Yellow
}

# Check for CUDA Toolkit
$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.8"
if (Test-Path $cudaPath) {
    # Add CUDA bin to PATH
    $cudaBinPath = "$cudaPath\bin"
    $env:PATH = "$cudaBinPath;$env:PATH"
    Write-Host "Added CUDA Toolkit bin directory to PATH: $cudaBinPath" -ForegroundColor Green
} else {
    Write-Host "Warning: CUDA Toolkit not found at $cudaPath" -ForegroundColor Yellow
}

# Print summary
Write-Host "`nEnvironment Setup Summary:" -ForegroundColor Cyan
Write-Host "------------------------" -ForegroundColor Cyan
Write-Host "X265_DIR = $env:X265_DIR"
Write-Host "NVIDIA_VIDEO_CODEC_SDK_DIR = $env:NVIDIA_VIDEO_CODEC_SDK_DIR"
Write-Host "PATH includes required directories: $(if ($env:PATH -like "*$binDir*" -and $env:PATH -like "*$nvLibPath*") { 'Yes' } else { 'No' })"

Write-Host "`nTo use these settings in a new terminal, run this script again." -ForegroundColor Cyan
Write-Host "To set these variables permanently, use the -permanent flag:"
Write-Host "    .\setup-env.ps1 -permanent" -ForegroundColor Yellow

# Check if the user wants to set the variables permanently
if ($permanent) {
    Write-Host "`nSetting environment variables permanently..." -ForegroundColor Cyan
    
    if (Test-Path $x265Dir) {
        [System.Environment]::SetEnvironmentVariable("X265_DIR", $x265Dir, [System.EnvironmentVariableTarget]::User)
        Write-Host "Permanently set X265_DIR environment variable" -ForegroundColor Green
    }
    
    if (Test-Path "$nvcodecDir\Interface\nvEncodeAPI.h") {
        [System.Environment]::SetEnvironmentVariable("NVIDIA_VIDEO_CODEC_SDK_DIR", $nvcodecDir, [System.EnvironmentVariableTarget]::User)
        Write-Host "Permanently set NVIDIA_VIDEO_CODEC_SDK_DIR environment variable" -ForegroundColor Green
    }
    
    Write-Host "Environment variables set permanently for user" -ForegroundColor Green
} 