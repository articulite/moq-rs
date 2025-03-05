# Start MoQ Streamer
# This script starts the MoQ streamer with the correct environment variables and parameters

$ErrorActionPreference = "Stop"

# Set environment variables
$env:X265_DIR = "$PSScriptRoot\moq-x265\x265"
$env:LIB = "$env:LIB;$PSScriptRoot\moq-receiver"
$env:PATH = "$env:X265_DIR\bin\x64;$env:PATH"

# Change to the streamer directory
Set-Location -Path "$PSScriptRoot\moq-streamer"

# Run the streamer
Write-Host "Starting MoQ Streamer..."
Write-Host "X265_DIR: $env:X265_DIR"
cargo run -- --server https://127.0.0.1:4443 --name desktop --width 640 --height 480 --bitrate 2000 --fps 30 --screen 0 --tls-disable-verify 