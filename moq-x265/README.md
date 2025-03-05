# moq-x265

Rust bindings for the x265 HEVC encoder/decoder library for the MoQ project.

## Prerequisites

To use this library, you need to have the x265 library installed on your system.

### Windows

On Windows, you can install x265 using one of the following methods:

1. **Use the provided setup script**:
   ```powershell
   # Run the setup script
   .\setup-x265.ps1
   ```
   
   This script will download and extract the x265 library to the `x265` directory and set up the necessary environment variables.

2. **Use the pre-downloaded files**:
   If you already have the x265 files in the `x265` directory, you can run the test script to verify that everything is working correctly:
   ```powershell
   # Run the test script
   .\test-x265.ps1
   ```

3. **Manual setup**:
   - Download the latest x265 binaries from [the official website](https://www.videolan.org/developers/x265.html)
   - Extract the files to a directory of your choice
   - Set the `X265_DIR` environment variable to point to this directory:
     ```powershell
     $env:X265_DIR = "C:\path\to\x265"
     ```
   - Add the bin directory to your PATH:
     ```powershell
     $env:PATH = "$env:X265_DIR\bin\x64;$env:PATH"
     ```

4. **Using vcpkg**:
   ```
   vcpkg install x265:x64-windows
   ```

### Linux

On Linux, you can install x265 using your package manager:

#### Ubuntu/Debian:
```
sudo apt-get install libx265-dev
```

#### Fedora:
```
sudo dnf install libx265-devel
```

#### Arch Linux:
```
sudo pacman -S x265
```

### macOS

On macOS, you can install x265 using Homebrew:
```
brew install x265
```

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
moq-x265 = { path = "../moq-x265" }
```

## Example

```rust
use moq_x265::{X265Encoder, X265Decoder};
use image::{ImageBuffer, Rgba};

// Create an encoder
let mut encoder = X265Encoder::new(1920, 1080, 5000, 30)?;

// Encode a frame
let frame: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(1920, 1080);
let encoded_frame = encoder.encode_frame(&frame)?;

// Create a decoder
let mut decoder = X265Decoder::new();

// Decode the frame
let decoded_image = decoder.decode_frame(&encoded_frame.data)?;
```

## Testing

You can run the test example to verify that the x265 library is working correctly:

```
cargo run --example test_x265
```

Or use the provided PowerShell script on Windows:

```powershell
.\test-x265.ps1
```

## Troubleshooting

### Windows

If you encounter issues with the x265 library not being found, make sure:

1. The `X265_DIR` environment variable is set correctly
2. The x265 DLL is in the `bin\x64` directory
3. The x265 library is in the `lib\x64` directory
4. The x265 headers are in the `include` directory

You can check the directory structure with:

```powershell
Get-ChildItem -Path $env:X265_DIR -Recurse
```

### Linux/macOS

If you encounter issues with the x265 library not being found, make sure:

1. The x265 development package is installed
2. The pkg-config can find the x265 library:
   ```
   pkg-config --libs --cflags x265
   ```

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option. 