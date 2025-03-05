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

# Build and run the test example
Write-Host "Building and running the test example..." -ForegroundColor Cyan;
cargo run --example test_x265;

# Check the exit code
if ($LASTEXITCODE -eq 0) {
    Write-Host "Test completed successfully!" -ForegroundColor Green;
} else {
    Write-Host "Test failed with exit code $LASTEXITCODE" -ForegroundColor Red;
} 