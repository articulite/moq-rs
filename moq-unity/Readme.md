# MoQ Unity

MoQ Unity is a low-latency video streaming solution that enables streaming desktop content to Unity applications using the Media over QUIC (MoQ) protocol.

## Components

This project contains two main components:

1. **Unity Receiver Plugin**: A native plugin for Unity that receives and displays MoQ video streams
2. **Desktop Streamer App**: A Rust application that captures desktop content, encodes it with H.264, and streams it via MoQ

## Requirements

### Unity Plugin Requirements
- Unity 2020.3 or newer
- CMake 3.12 or newer
- A compatible C++ compiler:
  - Windows: Visual Studio 2019 or newer
  - macOS: Xcode 12 or newer
  - Linux: GCC 9 or newer
  - Android (Quest): Android NDK r21 or newer

### Desktop Streamer Requirements
- Rust 1.64.0 or newer
- x264 libraries for video encoding
- Screen capture dependencies:
  - Windows: No additional dependencies
  - macOS: Core Graphics framework
  - Linux: X11 development libraries (`libx11-dev`, `libxext-dev`, `libxtst-dev`)

## Building the Unity Plugin

1. Navigate to the `NativePlugin` directory:

```bash
cd moq-unity/NativePlugin
```

2. Run the appropriate build script:

```bash
# Windows
./build.bat

# macOS/Linux
chmod +x build.sh
./build.sh
```

3. Open the Unity project in the `moq-unity` directory

## Building the Desktop Streamer

1. Navigate to the streamer directory:

```bash
cd moq-streamer
```

2. Build the application with Cargo:

```bash
cargo build --release
```

## Usage

### Running the MoQ Relay Server

Before you can stream video, you need a MoQ relay server:

```bash
cargo run -p moq-relay -- --bind 0.0.0.0:4443
```

### Streaming Your Desktop

Run the desktop streamer application:

```bash
cd moq-streamer
cargo run --release -- --server https://localhost:4443 --name desktop --width 1280 --height 720 --bitrate 3000
```

Command-line options:
- `--server`: The MoQ relay server URL (default: `https://localhost:4443`)
- `--name`: The stream name (default: `desktop`)
- `--width`: Output resolution width (default: 1920)
- `--height`: Output resolution height (default: 1080)
- `--bitrate`: Target bitrate in kbps (default: 5000)
- `--fps`: Target frame rate (default: 30)
- `--screen`: Screen number to capture (default: 0, the primary screen)

### Receiving Video in Unity

1. Add the `MoqVideoReceiver.cs` component to a GameObject in your scene (typically a quad)
2. Configure the component in the Inspector:
   - Server URL: The MoQ relay server address (e.g., `https://localhost:4443`)
   - Stream Path: The stream name (should match the `--name` parameter from the streamer)
   - Initial Width/Height: The expected video dimensions
   - Target Material: The material to apply the video texture to (optional if the component is on a renderer)
   - Target Latency: Latency target in milliseconds

3. Run your Unity application

## For Oculus Quest Users

For Quest deployment, you'll need to:

1. Build the native plugin for Android ARM64:

```bash
cd moq-unity/NativePlugin
mkdir -p build-android
cd build-android
cmake .. -DCMAKE_TOOLCHAIN_FILE=$ANDROID_NDK/build/cmake/android.toolchain.cmake -DANDROID_ABI=arm64-v8a
cmake --build .
```

2. Ensure the Android plugin is properly set up in Unity (placed in Assets/Plugins/Android/arm64-v8a/)

3. Configure Unity for Android development with the appropriate Oculus SDK

## Troubleshooting

- **Connection Issues**: Ensure the relay server is running and accessible from both the streamer and Unity application
- **Performance Problems**: Lower the resolution, bitrate, or framerate in the streamer
- **Black Screen in Unity**: Check if the frame queue is empty or if Unity is receiving frames but not displaying them
- **Build Errors**: Ensure all dependencies are properly installed

## Advanced Configuration

### Adjusting Video Quality

The desktop streamer uses x264 with the "superfast" and "zerolatency" presets for low-latency encoding. You can modify the encoding parameters in `encoder.rs` to trade quality for lower latency.

### Reducing Latency

For minimum latency:
- Reduce the resolution and bitrate
- Set a lower target latency in the Unity receiver
- Consider using an Ethernet connection instead of Wi-Fi
- Place the relay server closer to both the publisher and subscriber

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Setup Notes

### Building the Native Plugin on Windows

When building the native plugin on Windows, you may encounter issues with the Rust libraries. Here's how to resolve them:

1. **Build the required Rust libraries first**:
   ```powershell
   cargo build --release --package moq-transfork
   cargo build --release --package moq-karp
   cargo build --release --package moq-native
   ```

2. **Modify the CMakeLists.txt** to use the .rlib files instead of .a files:
   - Update the MOQ_LIB_DIR path to point to the target/release directory:
     ```cmake
     if(WIN32)
         set(MOQ_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release")
     ```
   - Use .rlib extension for Windows builds:
     ```cmake
     if(WIN32)
         target_link_libraries(MoqNativePlugin
             ${MOQ_LIB_DIR}/libmoq_transfork.rlib
             ${MOQ_LIB_DIR}/libmoq_karp.rlib
             ${MOQ_LIB_DIR}/libmoq_native.rlib
         )
     else()
         target_link_libraries(MoqNativePlugin
             ${MOQ_LIB_DIR}/libmoq_transfork.a
             ${MOQ_LIB_DIR}/libmoq_karp.a
             ${MOQ_LIB_DIR}/libmoq_native.a
         )
     endif()
     ```

3. **Update the build.bat script** to copy the DLL from the correct location:
   ```batch
   copy /Y bin\Windows\Release\MoqNativePlugin.dll ..\Assets\Plugins\
   ```

These changes ensure that the build process correctly finds and links the Rust libraries, and that the resulting DLL is properly copied to the Unity Plugins directory.

### Building the Desktop Streamer

When building the desktop streamer, you might encounter a workspace-related error:

```
error: current package believes it's in a workspace when it's not:
current:   C:\gitprojects\moq-rs\moq-streamer\Cargo.toml
workspace: C:\gitprojects\moq-rs\Cargo.toml
```

To fix this issue, you have two options:

1. **Add an empty workspace table to moq-streamer's Cargo.toml**:
   ```toml
   [package]
   name = "moq-streamer"
   version = "0.1.0"
   edition = "2021"
   
   [workspace]
   
   [dependencies]
   # ... dependencies ...
   ```

   This keeps the moq-streamer package out of the main workspace and allows it to be built independently.

2. **Build from the root directory** instead of the moq-streamer directory:
   ```powershell
   # From the root directory
   cargo build --release -p moq-streamer
   ```

Additionally, you may encounter issues with dependencies:

1. **Missing display-capture crate**: The moq-streamer depends on a crate called `display-capture` which may not be available. You can comment out this dependency in the Cargo.toml file since the streamer already uses the `scrap` crate for screen capture.

2. **x264 library**: The streamer requires the x264 library for video encoding. You'll need to install it:
   - **Windows**: Install pkg-config and x264 using MSYS2 or vcpkg
   - **macOS**: `brew install x264 pkg-config`
   - **Linux**: `sudo apt-get install libx264-dev pkg-config`

   After installing, make sure pkg-config is in your PATH environment variable.

### Installing x264 on Windows with MSYS2

If you're using Windows, here are detailed steps for installing the required x264 library using MSYS2:

1. **Install MSYS2** from https://www.msys2.org/ if you don't have it already

2. **Open the MSYS2 console** and run the following commands:
   ```bash
   # Update package database (if you haven't recently)
   pacman -Syu
   
   # Install pkg-config and x264
   pacman -S mingw64/mingw-w64-x86_64-pkg-config mingw64/mingw-w64-x86_64-x264
   ```

3. **Add MSYS2 binaries to your PATH** in PowerShell:
   ```powershell
   $env:PATH += ";C:\msys64\mingw64\bin"
   ```
   
   For a permanent solution, add this path to your system environment variables.

4. **Build the streamer**:
   ```powershell
   cargo build
   ```

### Known Issues with moq-streamer

The moq-streamer currently has API compatibility issues with the latest version of moq-karp. Here are the specific problems and solutions:

#### 1. VideoTrackProducer Type Missing

The moq-streamer expects a `VideoTrackProducer` type from moq-karp, but this type doesn't exist in the current version. Instead, moq-karp only has a generic `TrackProducer` that's used for all track types.

**Solution:**
Modify `moq-streamer/src/publisher.rs` to use the correct type:

```rust
// Change this:
video_track: Option<moq_karp::VideoTrackProducer>,

// To this:
video_track: Option<moq_karp::TrackProducer>,
```

#### 2. Codec String vs Enum Type Mismatch

The moq-streamer is passing a string for the codec, but moq-karp expects a `VideoCodec` enum:

```rust
// Current code in moq-streamer:
codec: "avc1.42001e".to_string(), // H.264 baseline profile

// Should be changed to:
codec: moq_karp::H264 {
    profile: 0x42,
    constraints: 0x00,
    level: 0x1e,
}.into(),
```

#### 3. x264 Library API Changes

The x264 crate used by moq-streamer may have API differences with the version expected. The specific issues include:

- Missing `ColorSpace` and `nal` imports
- Missing `Param` type
- Method signature changes for `Encoder` and `Picture` structs

**Solution:**
Check the version of the x264 crate in your Cargo.toml and update the code to match the API of that version. You may need to:

```rust
// Update imports to match the current x264 crate structure
use x264::{Encoder, Picture};
// Import specific types from the correct modules
use x264::colorspace::ColorSpace;
use x264::nal::UnitType;
```

#### 4. Debug Implementation Missing

There's an error about `moq_native::log::Args` not implementing `std::fmt::Debug`. This is likely due to a version mismatch between the moq-native crate used by moq-streamer and the one expected.

**Solution:**
Update the moq-native dependency to a compatible version or add a manual Debug implementation for the Args struct.

#### Recommended Approach

The simplest solution is to pin the dependencies to specific versions that are known to work together:

1. In moq-streamer/Cargo.toml, specify exact versions for all dependencies:
   ```toml
   moq-karp = { path = "../moq-karp", version = "=0.14.1" }
   moq-transfork = { path = "../moq-transfork", version = "=0.12.0" }
   moq-native = { path = "../moq-native", version = "=0.6.0" }
   x264 = "=0.5.0"
   ```

2. Alternatively, create a fork of moq-karp that includes a VideoTrackProducer type that wraps TrackProducer for backward compatibility.

These issues indicate that the moq-streamer was developed against an older version of the moq-karp API, and the API has since changed significantly.
