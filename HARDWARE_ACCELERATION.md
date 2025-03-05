# Hardware Acceleration for moq-rs

This document explains how to set up and use NVIDIA hardware acceleration for encoding and decoding in the moq-rs project.

## Prerequisites

- NVIDIA GPU with support for NVENC and NVDEC
- NVIDIA Graphics Driver (latest recommended)
- CUDA Toolkit (v11.0 or later recommended)
- NVIDIA Video Codec SDK (v12.0 or later recommended)

## Setup

### 1. Install NVIDIA Video Codec SDK

1. Download the NVIDIA Video Codec SDK from [NVIDIA Developer website](https://developer.nvidia.com/nvidia-video-codec-sdk)
2. Extract the SDK to a location on your system
3. Set the environment variable `NVIDIA_VIDEO_CODEC_SDK_DIR` to the path where you extracted the SDK

### 2. Set up moq-x265 with Hardware Acceleration

1. Navigate to the `moq-x265` directory
2. Run the setup script:
   ```
   .\setup-nvidia.ps1
   ```
   This script will:
   - Check for the NVIDIA Video Codec SDK
   - Add necessary paths to the environment
   - Build the moq-x265 library with hardware acceleration support

3. Test the hardware acceleration:
   ```
   cargo run --example test_nvidia
   ```

### 3. Set up moq-streamer with Hardware Acceleration

1. Navigate to the `moq-streamer` directory
2. Run the setup script:
   ```
   .\setup-nvidia.ps1
   ```
   This script will:
   - Check for the NVIDIA Video Codec SDK
   - Add necessary paths to the environment
   - Build the moq-streamer with hardware acceleration support

3. Run the streamer:
   ```
   cargo run -- --server https://localhost:4443 --name desktop
   ```

### 4. Set up moq-receiver with Hardware Acceleration

1. Navigate to the `moq-receiver` directory
2. Run the setup script:
   ```
   .\setup-nvidia.ps1
   ```
   This script will:
   - Check for the NVIDIA Video Codec SDK
   - Add necessary paths to the environment
   - Build the moq-receiver with hardware acceleration support

3. Run the receiver:
   ```
   cargo run -- --server https://localhost:4443 --name receiver
   ```

## How Hardware Acceleration Works

The moq-rs project uses hardware acceleration in the following ways:

1. **Encoding (moq-streamer)**: When hardware acceleration is available, the streamer uses NVIDIA's NVENC encoder to encode video frames. This significantly reduces CPU usage and improves performance.

2. **Decoding (moq-receiver)**: When hardware acceleration is available, the receiver uses NVIDIA's NVDEC decoder to decode video frames. This also reduces CPU usage and improves performance.

3. **Fallback Mechanism**: If hardware acceleration is not available or fails, the system automatically falls back to software encoding/decoding using x265.

## Troubleshooting

### Common Issues

1. **"NVIDIA Video Codec SDK not found"**
   - Make sure the `NVIDIA_VIDEO_CODEC_SDK_DIR` environment variable is set correctly
   - Verify that the SDK is extracted to the correct location

2. **"CUDA Toolkit bin directory not found"**
   - Make sure CUDA Toolkit is installed
   - The default path is `C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.8\bin`
   - If your CUDA installation is in a different location, update the path in the setup script

3. **Linking Errors**
   - Make sure the NVIDIA libraries are in your PATH
   - Run the setup script again to ensure all paths are set correctly

4. **Performance Issues**
   - Update to the latest NVIDIA drivers
   - Check GPU utilization to ensure hardware acceleration is being used

### Checking if Hardware Acceleration is Being Used

The applications log whether they are using hardware or software encoding/decoding. Look for messages like:
- "Using hardware acceleration for encoding"
- "Using hardware acceleration for decoding"
- "Hardware acceleration not available, falling back to software encoding/decoding"

## Performance Considerations

Hardware acceleration can significantly improve performance, especially for high-resolution video or when running on systems with limited CPU resources. However, there are some considerations:

1. **Startup Time**: Hardware-accelerated encoding/decoding may take slightly longer to initialize.
2. **Memory Usage**: Hardware acceleration may use more GPU memory.
3. **Quality vs. Performance**: Hardware encoding may have different quality characteristics compared to software encoding at the same bitrate.

## Contributing

If you encounter issues with hardware acceleration or have suggestions for improvements, please open an issue or submit a pull request on the moq-rs GitHub repository. 