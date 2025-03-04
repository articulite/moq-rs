#!/bin/bash
set -e

# Create build directory
mkdir -p build
cd build

# Configure and build
cmake ..
cmake --build . --config Release

# Copy the built libraries to the appropriate Unity Plugins directory
cd ..
mkdir -p ../Assets/Plugins

# Copy platform-specific libraries
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    cp bin/Darwin/libMoqNativePlugin.dylib ../Assets/Plugins/
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    cp bin/Linux/libMoqNativePlugin.so ../Assets/Plugins/
elif [[ "$OSTYPE" == "cygwin" || "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    # Windows
    cp bin/Windows/MoqNativePlugin.dll ../Assets/Plugins/
fi

echo "Native plugin built and copied to Unity Plugins directory" 