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
