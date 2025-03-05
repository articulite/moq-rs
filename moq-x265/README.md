# moq-x265

Rust bindings for the x265 HEVC encoder/decoder library with optional NVIDIA hardware acceleration.

## Features

- HEVC (H.265) encoding using the x265 library
- Basic HEVC decoding capabilities
- Optional NVIDIA hardware acceleration for both encoding and decoding

## Requirements

- Rust and Cargo
- x265 library (included in the repository for Windows)
- For hardware acceleration:
  - NVIDIA GPU with NVENC/NVDEC support
  - NVIDIA Video Codec SDK (included in the repository)
  - CUDA Toolkit (optional, but recommended)

## Installation

### Windows

The x265 library is included in the repository for Windows. No additional installation is required.

### Linux

Install the x265 library using your package manager:

```bash
# Ubuntu/Debian
sudo apt-get install libx265-dev

# Fedora
sudo dnf install libx265-devel

# Arch Linux
sudo pacman -S x265
```

### macOS

Install the x265 library using Homebrew:

```bash
brew install x265
```

## Hardware Acceleration

To enable NVIDIA hardware acceleration, you need to:

1. Have a compatible NVIDIA GPU with NVENC/NVDEC support
2. Have the NVIDIA Video Codec SDK (included in the repository)
3. Optionally, install the CUDA Toolkit for better performance
4. Build with the `hardware-accel` feature flag

### Windows

Use the provided setup script:

```powershell
.\setup-nvidia.ps1
```

This script will:
- Set the `NVIDIA_VIDEO_CODEC_SDK_DIR` environment variable
- Check for the NVIDIA Video Codec SDK
- Add the NVIDIA libraries to the system PATH
- Check for CUDA installation
- Build the project with hardware acceleration
- Run the NVIDIA test example

### Linux/macOS

Set the environment variable and build with the feature flag:

```bash
export NVIDIA_VIDEO_CODEC_SDK_DIR=/path/to/nvidia/video/codec/sdk
cargo build --features hardware-accel
```

## Usage

### Basic Encoding

```rust
use moq_x265::{X265Encoder, EncodedFrame};
use image::{ImageBuffer, Rgba};

// Create an encoder
let mut encoder = X265Encoder::new(640, 480, 5000, 30, 60)?;

// Create a frame
let img = ImageBuffer::new(640, 480);
// Fill the image with data...

// Encode the frame
let encoded_frame = encoder.encode_frame(&img)?;
```

### Hardware-Accelerated Encoding

```rust
use moq_x265::{HardwareEncoder, EncodedFrame};
use image::{ImageBuffer, Rgba};

// Create a hardware encoder
let mut encoder = HardwareEncoder::new(640, 480, 5000, 30, 60)?;

// Create a frame
let img = ImageBuffer::new(640, 480);
// Fill the image with data...

// Encode the frame
let encoded_frame = encoder.encode_frame(&img)?;
```

### Basic Decoding

```rust
use moq_x265::X265Decoder;

// Create a decoder
let mut decoder = X265Decoder::new();

// Decode a frame
let decoded_image = decoder.decode_frame(&encoded_data)?;
```

### Hardware-Accelerated Decoding

```rust
use moq_x265::HardwareDecoder;

// Create a hardware decoder
let mut decoder = HardwareDecoder::new()?;

// Decode a frame
let decoded_image = decoder.decode_frame(&encoded_data)?;
```

### Checking for Hardware Acceleration

```rust
use moq_x265::is_hardware_acceleration_available;

if is_hardware_acceleration_available() {
    println!("Hardware acceleration is available");
    // Use hardware acceleration
} else {
    println!("Hardware acceleration is not available");
    // Fall back to software implementation
}
```

## Examples

The repository includes several examples:

- `test_x265.rs`: Tests the x265 encoder and decoder
- `test_decoder.rs`: Tests the decoder with a pre-encoded file
- `test_nvidia.rs`: Tests the NVIDIA hardware acceleration

To run an example:

```bash
# Run the x265 test
cargo run --example test_x265

# Run the decoder test
cargo run --example test_decoder

# Run the NVIDIA test (requires hardware acceleration)
cargo run --features hardware-accel --example test_nvidia
```

## Performance Considerations

- Hardware acceleration can provide significant performance improvements, especially for real-time encoding and decoding.
- The NVIDIA NVENC encoder is optimized for speed rather than quality, so the x265 software encoder may produce better quality at the same bitrate.
- For the best performance with hardware acceleration, use the CUDA Toolkit and ensure your GPU drivers are up to date.

## Troubleshooting

If you encounter issues with hardware acceleration:

1. Make sure your NVIDIA GPU supports NVENC/NVDEC (most modern NVIDIA GPUs do)
2. Update your GPU drivers to the latest version
3. Install the CUDA Toolkit
4. Check that the NVIDIA Video Codec SDK is properly installed
5. Run the `test_nvidia.rs` example to verify your setup

## License

This project is licensed under the MIT License or Apache License 2.0, at your option. 