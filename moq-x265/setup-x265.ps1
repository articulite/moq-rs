# Setup script for x265 on Windows
# This script downloads and extracts the x265 library for Windows

# Configuration
$x265Version = "3.5"
$x265Url = "https://github.com/ShiftMediaProject/x265/releases/download/3.5/libx265_3.5_msvc16.zip"
$x265Dir = "$PSScriptRoot\x265"

# Create the x265 directory if it doesn't exist
if (-not (Test-Path $x265Dir)) {
    Write-Host "Creating x265 directory..."
    New-Item -ItemType Directory -Path $x265Dir | Out-Null
}

# Download x265
$zipFile = "$env:TEMP\x265.zip"
Write-Host "Downloading x265 $x265Version..."
Invoke-WebRequest -Uri $x265Url -OutFile $zipFile

# Extract the zip file
Write-Host "Extracting x265..."
Expand-Archive -Path $zipFile -DestinationPath $x265Dir -Force

# Set the X265_DIR environment variable for the current session
$env:X265_DIR = $x265Dir
Write-Host "Set X265_DIR environment variable to $x265Dir for the current session"

# Add X265_DIR to the user's environment variables permanently
[System.Environment]::SetEnvironmentVariable("X265_DIR", $x265Dir, [System.EnvironmentVariableTarget]::User)
Write-Host "Added X265_DIR to user environment variables"

# Add the x265 bin directory to the PATH for the current session
$binDir = "$x265Dir\bin\x64"
$env:PATH = "$binDir;$env:PATH"
Write-Host "Added $binDir to PATH for the current session"

# Check if the x265 library is available
if (Test-Path "$binDir\x265.dll") {
    Write-Host "x265 library installed successfully!"
} else {
    Write-Host "Error: x265 library not found after installation" -ForegroundColor Red
    Write-Host "Looking for x265.dll in $binDir" -ForegroundColor Yellow
    Get-ChildItem -Path $binDir -Recurse | ForEach-Object { Write-Host $_.FullName }
    exit 1
}

Write-Host "x265 setup complete!" -ForegroundColor Green
Write-Host "You can now build and run the moq-streamer and moq-receiver applications." 