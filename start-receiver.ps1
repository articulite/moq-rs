# Start MoQ Receiver
# This script starts the MoQ receiver with the correct environment variables and parameters

$ErrorActionPreference = "Stop"

# Set environment variables
$env:X265_DIR = "$PSScriptRoot\moq-x265\x265"
$env:LIB = "$env:LIB;$PSScriptRoot\moq-receiver"
$env:PATH = "$env:X265_DIR\bin\x64;$env:PATH"

# Change to the receiver directory
Set-Location -Path "$PSScriptRoot\moq-receiver"

# Run the receiver
Write-Host "Starting MoQ Receiver..."
Write-Host "X265_DIR: $env:X265_DIR"
cargo run -- --server https://127.0.0.1:4443 --name desktop --width 640 --height 480 --latency 500 --disable-verify 