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

# Start the relay server in a new PowerShell window
Write-Host "Starting MoQ Relay Server..."
Start-Process powershell -ArgumentList "-NoExit -File $PSScriptRoot\start-relay.ps1"

# Wait for the relay server to start
Write-Host "Waiting for relay server to start..."
Start-Sleep -Seconds 5

# Start the streamer in a new PowerShell window
Write-Host "Starting MoQ Streamer..."
Start-Process powershell -ArgumentList "-NoExit -File $PSScriptRoot\start-streamer.ps1"

# Wait for the streamer to start
Write-Host "Waiting for streamer to start..."
Start-Sleep -Seconds 5

# Start the receiver in a new PowerShell window
Write-Host "Starting MoQ Receiver..."
Start-Process powershell -ArgumentList "-NoExit -File $PSScriptRoot\start-receiver.ps1"

Write-Host "All components started. Check the individual windows for status." 