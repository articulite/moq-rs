# Test script for x265 on Windows
# This script runs the test example to verify that x265 is working correctly

# Set the X265_DIR environment variable for the current session
$env:X265_DIR = "$PSScriptRoot\x265";
Write-Host "Set X265_DIR environment variable to $env:X265_DIR for the current session";

# Add the x265 bin directory to the PATH for the current session
$binDir = "$env:X265_DIR\bin\x64";
$env:PATH = "$binDir;$env:PATH";
Write-Host "Added $binDir to PATH for the current session";

# Check if the x265 library is available
if (Test-Path "$binDir\x265.dll") {
    Write-Host "x265 library found at $binDir\x265.dll" -ForegroundColor Green;
} else {
    Write-Host "Error: x265 library not found at $binDir\x265.dll" -ForegroundColor Red;
    Write-Host "Looking for x265.dll in $binDir" -ForegroundColor Yellow;
    Get-ChildItem -Path $binDir -Recurse | ForEach-Object { Write-Host $_.FullName };
    exit 1;
}

# Check for FFmpeg
$ffmpegPath = $null
try {
    $ffmpegPath = (Get-Command ffmpeg -ErrorAction Stop).Source
    Write-Host "FFmpeg found at: $ffmpegPath" -ForegroundColor Green
} catch {
    Write-Host "FFmpeg not found in PATH" -ForegroundColor Yellow
    
    # Check common FFmpeg installation locations
    $commonPaths = @(
        "C:\ffmpeg\bin",
        "C:\Program Files\ffmpeg\bin",
        "C:\Program Files (x86)\ffmpeg\bin"
    )
    
    foreach ($path in $commonPaths) {
        if (Test-Path "$path\ffmpeg.exe") {
            Write-Host "FFmpeg found at $path\ffmpeg.exe" -ForegroundColor Green
            $ffmpegPath = $path
            break
        }
    }
    
    if ($ffmpegPath) {
        # Add FFmpeg to PATH for current session
        $env:PATH = "$ffmpegPath;$env:PATH"
        Write-Host "Added $ffmpegPath to PATH for the current session" -ForegroundColor Green
    } else {
        Write-Host "FFmpeg not found. MP4 creation will not work." -ForegroundColor Yellow
        Write-Host "The test will still run, but no MP4 file will be created." -ForegroundColor Yellow
    }
}

# Build and run the test example
Write-Host "Building and running the test example..." -ForegroundColor Cyan;
cargo run --example test_x265;

# Check the exit code
if ($LASTEXITCODE -eq 0) {
    Write-Host "Test completed successfully!" -ForegroundColor Green;
    
    # Check if MP4 file was created
    if (Test-Path "output/color_alternating.mp4") {
        Write-Host "MP4 file created successfully at output/color_alternating.mp4" -ForegroundColor Green
        Write-Host "You can play this file in any video player that supports HEVC/H.265"
    } else {
        Write-Host "MP4 file was not created." -ForegroundColor Yellow
        if ($ffmpegPath) {
            Write-Host "You can manually create an MP4 file with:" -ForegroundColor Yellow
            Write-Host "ffmpeg -f hevc -i output/all_frames.hevc -c:v copy output/color_alternating.mp4" -ForegroundColor Yellow
        } else {
            Write-Host "To create an MP4 file, install FFmpeg from https://ffmpeg.org/download.html" -ForegroundColor Yellow
        }
    }
} else {
    Write-Host "Test failed with exit code $LASTEXITCODE" -ForegroundColor Red;
} 