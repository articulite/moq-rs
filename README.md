<p align="center">
	<img height="128px" src="https://github.com/kixelated/moq-rs/blob/main/.github/logo.svg" alt="Media over QUIC">
</p>

Media over QUIC (MoQ) is a live media delivery protocol utilizing QUIC.
It's a client-server model that is designed to scale to enormous viewership via clustered relay servers (aka a CDN).
The application determines the trade-off between latency and quality, potentially on a per viewer-basis.

See [quic.video](https://quic.video) for more information.
Note: this project is a [fork of the IETF draft](https://quic.video/blog/transfork) to speed up development.
If you're curious about the protocol, check out the current [specification](https://github.com/kixelated/moq-drafts).

The project is split into a few crates:

-   [moq-relay](moq-relay): A server that forwards content from publishers to any interested subscribers. It can optionally be clustered, allowing N servers to transfer between themselves.
- [moq-web](moq-web): A web client utilizing Rust and WASM. Supports both consuming and publishing media.
-   [moq-transfork](moq-transfork): The underlying network protocol. It can be used by live applications that need real-time and scale, even if they're not media.
- [moq-karp](moq-karp): The underlying media protocol powered by moq-transfork. It includes a CLI for importing/exporting to other formats, for example integrating with ffmpeg.
-   [moq-clock](moq-clock): A dumb clock client/server just to prove MoQ can be used for more than media.
-   [moq-native](moq-native): Helpers to configure the native MoQ tools.
-   [moq-streamer](moq-streamer): A desktop application that captures the screen and streams it using HEVC encoding. Supports hardware acceleration with NVIDIA GPUs.
-   [moq-receiver](moq-receiver): A desktop application that receives and displays video streams. Supports hardware acceleration with NVIDIA GPUs.
-   [moq-x265](moq-x265): Rust bindings for the x265 HEVC encoder/decoder library. Includes support for NVIDIA hardware acceleration.



# Usage
## Requirements
- [Rustup](https://www.rust-lang.org/tools/install)
- [Just](https://github.com/casey/just?tab=readme-ov-file#installation)
- [Node + NPM](https://nodejs.org/)
- [x265](https://www.videolan.org/developers/x265.html) (for HEVC encoding/decoding)
- [NVIDIA Video Codec SDK](https://developer.nvidia.com/nvidia-video-codec-sdk) (optional, for hardware acceleration)

## Setup
We use `just` to simplify the development process.
Check out the [Justfile](justfile) or run `just` to see the available commands.

Install any other required tools:
```sh
just setup
```

### x265 Setup
For the HEVC encoding/decoding functionality, you need to install the x265 library:

#### Windows
Run the provided setup script:
```powershell
cd moq-x265
.\setup-x265.ps1
```

#### Linux
Install using your package manager:
```sh
# Ubuntu/Debian
sudo apt-get install libx265-dev

# Fedora
sudo dnf install libx265-devel

# Arch Linux
sudo pacman -S x265
```

#### macOS
Install using Homebrew:
```sh
brew install x265
```

### Hardware Acceleration Setup (Optional)
For NVIDIA hardware acceleration, follow the instructions in [HARDWARE_ACCELERATION.md](HARDWARE_ACCELERATION.md).

## Development

```sh
# Run the relay, a demo movie, and web server:
just all

# Or run each individually in separate terminals:
just relay
just bbb
just web
```

Then, visit [https://localhost:8080](localhost:8080) to watch the simple demo.

When you're ready to submit a PR, make sure the tests pass or face the wrath of CI:
```sh
just check
just test
```

# Components
## moq-relay

[moq-relay](moq-relay) is a server that forwards subscriptions from publishers to subscribers, caching and deduplicating along the way.
It's designed to be run in a datacenter, relaying media across multiple hops to deduplicate and improve QoS.

This listens for WebTransport connections on `UDP https://localhost:4443` by default.
You need a client to connect to that address, to both publish and consume media.

## moq-web

[moq-web](moq-web) is a web client that can consume media (and soon publish).
It's available [on NPM](https://www.npmjs.com/package/@kixelated/moq) as both a JS library and web component.

For example:

```html
<script type="module">
	import '@kixelated/moq/watch'
</script>

<moq-watch url="https://relay.quic.video/demo/bbb"></moq-watch>
```


See the [moq-web README](moq-web/README.md) for more information.

## moq-streamer

[moq-streamer](moq-streamer) is a desktop application that captures the screen and streams it to a MoQ relay server using HEVC encoding. It uses the x265 library for high-quality, efficient video encoding and supports NVIDIA hardware acceleration for improved performance.

```bash
# Build and run the streamer
cd moq-streamer
cargo run -- --server https://localhost:4443 --name desktop

# With hardware acceleration (Windows)
.\setup-nvidia.ps1
cargo run -- --server https://localhost:4443 --name desktop
```

## moq-receiver

[moq-receiver](moq-receiver) is a desktop application that can connect to a MoQ relay server and display video streams. It uses SDL2 for rendering and window management, and the x265 library for HEVC decoding. It also supports NVIDIA hardware acceleration for improved performance.

```bash
# Build and run the receiver
cd moq-receiver
cargo run -- --server https://localhost:4443 --name desktop

# With hardware acceleration (Windows)
.\setup-nvidia.ps1
cargo run -- --server https://localhost:4443 --name desktop
```

See the [moq-receiver README](moq-receiver/README.md) for more information on setup and usage, including SDL2 installation instructions.

## moq-x265

[moq-x265](moq-x265) provides Rust bindings for the x265 HEVC encoder/decoder library. It's used by both moq-streamer and moq-receiver for efficient video encoding and decoding. It also includes support for NVIDIA hardware acceleration through the NVENC and NVDEC APIs.

```bash
# Test hardware acceleration
cd moq-x265
.\setup-nvidia.ps1
cargo run --example test_nvidia
```

See the [moq-x265 README](moq-x265/README.md) for more information on setup and usage.

## moq-karp

[moq-karp](moq-karp) is a simple media layer on top of MoQ.
The crate includes a binary that accepts fMP4 with a few restrictions:

-   `separate_moof`: Each fragment must contain a single track.
-   `frag_keyframe`: A keyframe must be at the start of each keyframe.
-   `fragment_per_frame`: (optional) Each frame should be a separate fragment to minimize latency.

This can be used in conjunction with ffmpeg to publish media to a MoQ relay.
See the [Justfile](./justfile) for the required ffmpeg flags.

Alternatively, see [moq-gst](https://github.com/kixelated/moq-gst) for a gstreamer plugin.

## moq-transfork

A media-agnostic library used by [moq-relay](moq-relay) and [moq-karp](moq-karp) to serve the underlying subscriptions.
It has caching/deduplication built-in, so your application is oblivious to the number of connections under the hood.

See the published [crate](https://crates.io/crates/moq-transfork) and [documentation](https://docs.rs/moq-transfork/latest/moq_transfork/).

## moq-clock

[moq-clock](moq-clock) is a simple client that can publish or subscribe to the current time.
It's meant to demonstate that [moq-transfork](moq-transfork) can be used for more than just media.

## nix/nixos

moq also has nix support see [`nix/README.md`](nix/README.md)


# License

Licensed under either:

-   Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
-   MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

# MoQ Streaming System

This repository contains a Media over QUIC (MoQ) implementation for streaming video.

## Components

- **MoQ Relay**: A server that relays media streams between publishers and subscribers
- **MoQ Streamer**: A client that captures the screen and publishes it to the relay server
- **MoQ Receiver**: A client that subscribes to streams from the relay server and displays them

## Prerequisites

- Windows 10 or later
- Rust and Cargo installed
- x265 library (included in the repository)
- SDL2 library (included in the repository)
- FFmpeg (for MP4 container creation)
- NVIDIA GPU with NVENC/NVDEC support (optional, for hardware acceleration)

## Getting Started

### Using the PowerShell Scripts

The repository includes several PowerShell scripts to simplify running the components:

1. **start-relay.ps1**: Starts the MoQ relay server
2. **start-streamer.ps1**: Starts the MoQ streamer (publisher)
3. **start-receiver.ps1**: Starts the MoQ receiver (subscriber)
4. **start-all.ps1**: Starts all components in the correct order

To run the entire system at once:

```powershell
.\start-all.ps1
```

This will open three PowerShell windows, one for each component.

### Running Components Individually

If you prefer to run components individually:

1. First, start the relay server:
```powershell
.\start-relay.ps1
```

2. Then, start the streamer:
```powershell
.\start-streamer.ps1
```

3. Finally, start the receiver:
```powershell
.\start-receiver.ps1
```

### Hardware Acceleration

For improved performance with NVIDIA GPUs, use the hardware acceleration setup scripts:

```powershell
# For the streamer
cd moq-streamer
.\setup-nvidia.ps1

# For the receiver
cd moq-receiver
.\setup-nvidia.ps1
```

See [HARDWARE_ACCELERATION.md](HARDWARE_ACCELERATION.md) for detailed instructions.

## Configuration

The scripts use default configuration values. If you need to modify them:

- Edit the PowerShell scripts directly to change parameters like resolution, bitrate, etc.
- The default resolution is 640x480
- The default bitrate is 2000 kbps
- The default frame rate is 30 fps

## Troubleshooting

If you encounter issues:

1. Make sure all environment variables are set correctly
2. Check that the x265 library is properly installed
3. Verify that SDL2 is available
4. Ensure that the relay server is running before starting the streamer and receiver

## License

This project is licensed under the MIT License - see the LICENSE file for details.
