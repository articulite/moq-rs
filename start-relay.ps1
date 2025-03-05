# Start MoQ Relay Server
# This script starts the MoQ relay server with the correct parameters

$ErrorActionPreference = "Stop"

# Change to the relay directory
Set-Location -Path "$PSScriptRoot\moq-relay"

# Run the relay server
Write-Host "Starting MoQ Relay Server..."
cargo run -- --bind 127.0.0.1:4443 --tls-cert ../certs/cert.pem --tls-key ../certs/key.pem --tls-disable-verify 