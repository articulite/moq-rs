mod capture;
mod hevc_encoder;
mod publisher;

use anyhow::{Context, Result};
use clap::Parser;
use tracing::info;
use url::Url;

// Create a wrapper struct for moq_native::log::Args to implement Debug
#[derive(Parser, Debug)]
struct LogArgs {
    /// Verbose logging
    #[clap(long, short)]
    verbose: bool,
    
    /// Quiet logging
    #[clap(long, short)]
    quiet: bool,
}

impl From<LogArgs> for moq_native::log::Args {
    fn from(args: LogArgs) -> Self {
        moq_native::log::Args {
            verbose: if args.verbose { 1 } else { 0 },
            quiet: if args.quiet { 1 } else { 0 },
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about = "MoQ Desktop Streamer")]
struct Args {
    /// URL of the MoQ relay server
    #[clap(long, short, default_value = "https://localhost:4443")]
    server: String,

    /// Stream name
    #[clap(long, short, default_value = "desktop")]
    name: String,

    /// Width of the output stream
    #[clap(long, default_value = "1920")]
    width: u32,

    /// Height of the output stream
    #[clap(long, default_value = "1080")]
    height: u32,

    /// Target bitrate in kbps
    #[clap(long, default_value = "5000")]
    bitrate: u32,

    /// Target framerate
    #[clap(long, default_value = "30")]
    fps: u32,

    /// Screen number to capture (0-based index)
    #[clap(long, default_value = "0")]
    screen: usize,

    /// Log configuration
    #[clap(flatten)]
    log: LogArgs,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Initialize logging
    let log_args: moq_native::log::Args = args.log.into();
    log_args.init();
    
    // Parse the server URL
    let server_url = Url::parse(&args.server)
        .context("Invalid server URL")?;
    
    info!("Starting MoQ Desktop Streamer");
    info!("Server: {}", server_url);
    info!("Stream: {}", args.name);
    info!("Resolution: {}x{}", args.width, args.height);
    info!("Bitrate: {} kbps", args.bitrate);
    info!("FPS: {}", args.fps);
    info!("Codec: HEVC (H.265)");
    
    // Initialize screen capture
    let mut capturer = capture::ScreenCapture::new(
        args.screen,
        args.width,
        args.height,
        args.fps,
    )?;
    
    // Initialize publisher
    let mut publisher = publisher::MoqPublisher::new(
        server_url,
        args.name,
        args.width,
        args.height,
        args.bitrate,
    ).await?;
    
    // Initialize HEVC encoder
    let mut hevc_encoder = hevc_encoder::HEVCEncoder::new(
        args.width,
        args.height,
        args.bitrate,
        args.fps,
    )?;
    
    info!("Started streaming. Press Ctrl+C to stop.");
    
    // Main capture and encoding loop
    loop {
        // Capture frame
        let frame = capturer.capture_frame().await?;
        
        // Encode frame with HEVC
        let encoded = hevc_encoder.encode_frame(&frame)?;
        
        // Publish frame
        publisher.publish_frame(encoded).await?;
    }
} 