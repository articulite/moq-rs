# MoQ Receiver

A simple SDL2-based video player for MoQ streams.

## Overview

The MoQ Receiver is a desktop application that can connect to a MoQ relay server and display video streams. It uses SDL2 for rendering and window management.

## Building

### Prerequisites

- Rust 1.64.0 or newer
- SDL2 development libraries

### SDL2 Setup

#### Windows

On Windows, you need to install the SDL2 development libraries. There are several ways to do this:

1. **Using the provided PowerShell script**:

   ```powershell
   # Run from the moq-receiver directory
   .\setup-sdl2.ps1
   ```

   This script will:
   - Download the SDL2 development libraries
   - Extract them to a temporary directory
   - Copy SDL2.lib and SDL2.dll to the current directory
   - Clean up temporary files

2. **Manual installation**:

   - Download SDL2 development libraries from [SDL's website](https://github.com/libsdl-org/SDL/releases)
   - Extract the ZIP file
   - Copy the following files to your moq-receiver directory:
     - `SDL2-x.x.x\lib\x64\SDL2.lib`
     - `SDL2-x.x.x\lib\x64\SDL2.dll`

3. **Using the LIB environment variable**:

   If you have SDL2 installed elsewhere on your system, you can set the LIB environment variable to include the directory containing SDL2.lib:

   ```powershell
   $env:LIB += ";C:\path\to\sdl2\lib\x64"
   ```

#### macOS

On macOS, you can install SDL2 using Homebrew:

```bash
brew install sdl2
```

#### Linux

On Debian/Ubuntu:

```bash
sudo apt-get install libsdl2-dev
```

On Fedora:

```bash
sudo dnf install SDL2-devel
```

On Arch Linux:

```bash
sudo pacman -S sdl2
```

### Building the Application

Once SDL2 is set up, you can build the application:

```bash
cargo build --release
```

If you're on Windows and have SDL2.lib in the current directory but the linker still can't find it, you can set the LIB environment variable:

```powershell
$env:LIB += ";$(Get-Location)"; cargo build
```

## Usage

```bash
cargo run -- --server https://localhost:4443 --name desktop
```

### Command-line Options

- `--server`, `-s`: URL of the MoQ relay server (default: https://localhost:4443)
- `--name`, `-n`: Stream name to subscribe to (default: desktop)
- `--latency`: Target latency in milliseconds (default: 500)
- `--width`: Initial window width (default: 1280)
- `--height`: Initial window height (default: 720)
- `--verbose`, `-v`: Enable verbose logging
- `--tls-cert`: Path to TLS certificate file
- `--tls-key`: Path to TLS private key file
- `--tls-root`: Path to TLS root certificate file
- `--tls-disable-verify`: Disable TLS certificate verification (dangerous)

## Troubleshooting

### SDL2 Library Not Found

If you encounter an error like `cannot open input file 'SDL2.lib'` during the build process:

1. Make sure SDL2.lib is in the current directory or in a directory included in your LIB environment variable
2. Try setting the LIB environment variable explicitly:

   ```powershell
   $env:LIB += ";$(Get-Location)"; cargo build
   ```

3. Check that you have the correct version of SDL2 for your platform (32-bit vs 64-bit)

### Runtime Issues

If the application builds but crashes at runtime with an error about SDL2.dll not being found:

1. Make sure SDL2.dll is in the same directory as your executable or in your system PATH
2. For development, keep SDL2.dll in the same directory as your Cargo.toml file

## License

Licensed under either:

- Apache License, Version 2.0, ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT) 