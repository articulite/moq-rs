# PowerShell script to download and set up SDL2 for Windows
$sdlVersion = "2.28.5"
$downloadUrl = "https://github.com/libsdl-org/SDL/releases/download/release-$sdlVersion/SDL2-devel-$sdlVersion-VC.zip"
$tempZipFile = "SDL2-devel.zip"
$extractDir = "SDL2-devel"

# Create directories if they don't exist
New-Item -ItemType Directory -Force -Path $extractDir | Out-Null

# Download SDL2 development libraries
Write-Host "Downloading SDL2 development libraries..."
Invoke-WebRequest -Uri $downloadUrl -OutFile $tempZipFile

# Extract the ZIP file
Write-Host "Extracting SDL2 development libraries..."
Expand-Archive -Path $tempZipFile -DestinationPath $extractDir -Force

# Copy SDL2.lib to the current directory
Write-Host "Copying SDL2.lib to the current directory..."
Copy-Item -Path "$extractDir\SDL2-$sdlVersion\lib\x64\SDL2.lib" -Destination "." -Force

# Copy SDL2.dll to the current directory
Write-Host "Copying SDL2.dll to the current directory..."
Copy-Item -Path "$extractDir\SDL2-$sdlVersion\lib\x64\SDL2.dll" -Destination "." -Force

# Copy SDL2main.lib to the current directory (might be needed for some applications)
Write-Host "Copying SDL2main.lib to the current directory..."
Copy-Item -Path "$extractDir\SDL2-$sdlVersion\lib\x64\SDL2main.lib" -Destination "." -Force

Write-Host "SDL2 setup complete. SDL2.lib, SDL2main.lib, and SDL2.dll have been copied to the current directory."
Write-Host "You can now build the moq-receiver application with: cargo build"
Write-Host "If you still encounter linking issues, try: `$env:LIB += `";`$(Get-Location)`"; cargo build" 