mod capture;
mod encoder;
mod publisher;

use anyhow::{Context, Result};
use clap::Parser;
use std::net;
use tracing::info;
use url::Url;

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
    log: moq_native::log::Args,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Initialize logging
    args.log.init();
    
    // Parse the server URL
    let server_url = Url::parse(&args.server)
        .context("Invalid server URL")?;
    
    info!("Starting MoQ Desktop Streamer");
    info!("Server: {}", server_url);
    info!("Stream: {}", args.name);
    info!("Resolution: {}x{}", args.width, args.height);
    info!("Bitrate: {} kbps", args.bitrate);
    info!("FPS: {}", args.fps);
    
    // Initialize screen capture
    let mut capturer = capture::ScreenCapture::new(
        args.screen,
        args.width,
        args.height,
        args.fps,
    )?;
    
    // Initialize encoder
    let mut encoder = encoder::VideoEncoder::new(
        args.width,
        args.height,
        args.bitrate * 1000, // Convert to bps
        args.fps,
    )?;
    
    // Initialize publisher
    let mut publisher = publisher::MoqPublisher::new(
        server_url,
        args.name,
        args.width,
        args.height,
        args.bitrate * 1000,
    ).await?;
    
    info!("Started streaming. Press Ctrl+C to stop.");
    
    // Main capture and encoding loop
    loop {
        // Capture frame
        let frame = capturer.capture_frame().await?;
        
        // Encode frame
        let encoded = encoder.encode_frame(&frame)?;
        
        // Publish frame
        publisher.publish_frame(encoded).await?;
    }
} 