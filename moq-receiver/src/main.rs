use anyhow::Context as _;
use clap::Parser;
use moq_karp::BroadcastConsumer;
use moq_native::{log, quic, tls};
use moq_transfork::Session;
use sdl2::pixels::PixelFormatEnum;
use tracing::{info, error, debug, warn};
use url::Url;
use std::time::Duration;

/// MoQ Receiver - A simple application to receive and display MoQ video streams
#[derive(Parser, Clone, Default)]
struct LogArgs {
	/// Verbose logging
	#[clap(long, short)]
	verbose: bool,
}

impl From<LogArgs> for log::Args {
	fn from(args: LogArgs) -> Self {
		let mut log_args = log::Args::default();
		if args.verbose {
			log_args.verbose = 1; // DEBUG level
		} else {
			log_args.verbose = 0;
		}
		log_args
	}
}

#[derive(Parser, Clone, Default)]
struct TlsArgs {
	/// Use the certificates at this path, encoded as PEM.
	#[clap(long)]
	cert: Option<String>,

	/// Use the private key at this path, encoded as PEM.
	#[clap(long)]
	key: Option<String>,

	/// Use the TLS root at this path, encoded as PEM.
	#[clap(long)]
	root: Option<String>,

	/// Danger: Disable TLS certificate verification.
	#[clap(long)]
	disable_verify: bool,
}

impl From<TlsArgs> for tls::Args {
	fn from(args: TlsArgs) -> Self {
		let mut tls_args = tls::Args::default();
		
		if let Some(cert) = args.cert {
			tls_args.cert = vec![cert.into()];
		}
		
		if let Some(key) = args.key {
			tls_args.key = vec![key.into()];
		}
		
		if let Some(root) = args.root {
			tls_args.root = vec![root.into()];
		}
		
		if args.disable_verify {
			tls_args.disable_verify = true;
		}
		
		tls_args
	}
}

#[derive(Parser)]
struct Args {
	/// URL of the MoQ relay server
	#[clap(long, short, default_value = "https://localhost:4443")]
	server: Url,

	/// Stream name to subscribe to
	#[clap(long, short, default_value = "desktop")]
	name: String,

	/// Target latency in milliseconds
	#[clap(long, default_value = "500")]
	latency: u64,

	/// Initial window width
	#[clap(long, default_value = "640")]
	width: u32,

	/// Initial window height
	#[clap(long, default_value = "480")]
	height: u32,

	/// Log configuration
	#[clap(flatten)]
	log: LogArgs,
	
	/// TLS configuration
	#[clap(flatten)]
	tls: TlsArgs,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	// Parse command line arguments
	let args = Args::parse();
	
	// Initialize logging
	let log_args: log::Args = args.log.into();
	log_args.init();
	
	info!("Starting MoQ Receiver");
	info!("Server: {}", args.server);
	info!("Stream: {}", args.name);
	info!("Target latency: {} ms", args.latency);
	
	// Initialize SDL2
	let sdl_context = sdl2::init()
		.map_err(|e| anyhow::anyhow!("Failed to initialize SDL: {}", e))?;
	let video_subsystem = sdl_context
		.video()
		.map_err(|e| anyhow::anyhow!("Failed to initialize video subsystem: {}", e))?;
	
	let window = video_subsystem.window("MoQ Receiver", args.width, args.height)
		.position_centered()
		.opengl()
		.build()
		.map_err(|e| anyhow::anyhow!("Failed to create window: {}", e))?;
	
	let mut canvas = window.into_canvas().build().map_err(|e| anyhow::anyhow!("Failed to create canvas: {}", e))?;
	let texture_creator = canvas.texture_creator();
	
	let mut texture = texture_creator
		.create_texture_streaming(PixelFormatEnum::RGB24, args.width, args.height)
		.map_err(|e| anyhow::anyhow!("Failed to create texture: {}", e))?;
	
	// Initialize QUIC
	let quic_args = quic::Args {
		bind: "0.0.0.0:0".parse().unwrap(),
		tls: args.tls.into(),
	};
	
	let quic_config = quic_args.load().context("Failed to load QUIC config")?;
	let quic = quic::Endpoint::new(quic_config).context("Failed to create QUIC endpoint")?;
	
	// Connect to the server
	let server_url = args.server.clone();
	info!("Connecting to {}", server_url);
	
	let connection = quic.client.connect(server_url.clone())
		.await
		.context("Failed to connect to server")?;
	
	// Create a session
	let session = Session::connect(connection)
		.await
		.context("Failed to create session")?;
	
	// Create a broadcast consumer
	let mut broadcast = BroadcastConsumer::new(session, args.name.clone());
	
	// Wait for the video track to become available
	let mut video_track = None;
	for _ in 0..10 {
		let catalog_result = broadcast.next_catalog().await;
		
		if let Ok(Some(catalog)) = catalog_result {
			let mut video_tracks = Vec::new();
			
			for video in &catalog.video {
				if video.track.name == "video" {
					info!("Found video track");
					video_tracks.push(video.track.clone());
				}
			}
			
			for track in video_tracks {
				match broadcast.track(&track) {
					Ok(track_consumer) => {
						video_track = Some(track_consumer);
						break;
					}
					Err(e) => {
						error!("Failed to subscribe to video track: {}", e);
					}
				}
			}
		}
		
		if video_track.is_some() {
			break;
		}
		
		info!("Waiting for video track...");
		tokio::time::sleep(Duration::from_secs(1)).await;
	}
	
	let mut video_track = video_track.context("Failed to find video track")?;
	
	// Set latency
	if args.latency > 0 {
		video_track.set_latency(Duration::from_millis(args.latency));
	}
	
	// Create an event loop
	let mut event_pump = sdl_context.event_pump().map_err(|e| anyhow::anyhow!("Failed to create event pump: {}", e))?;
	
	'running: loop {
		// Process events
		for event in event_pump.poll_iter() {
			match event {
				sdl2::event::Event::Quit {..} => break 'running,
				_ => {}
			}
		}
		
		// Try to read a frame
		match video_track.read().await {
			Ok(Some(frame)) => {
				process_frame(&mut texture, &frame, args.width)
					.context("Failed to process frame")?;
				canvas.copy(&texture, None, None)
					.map_err(|e| anyhow::anyhow!("Failed to copy texture: {}", e))?;
				canvas.present();
			},
			Ok(None) => {
				info!("End of stream");
				break 'running;
			},
			Err(e) => {
				error!("Failed to read frame: {}", e);
				tokio::time::sleep(Duration::from_millis(10)).await;
			}
		}
	}
	
	info!("Exiting MoQ Receiver");
	Ok(())
}

fn process_frame(
	texture: &mut sdl2::render::Texture,
	frame: &moq_karp::Frame,
	width: u32,
) -> anyhow::Result<()> {
	// Frame data is directly accessible via the payload field
	let data = &frame.payload;
	
	// Log frame information
	tracing::debug!("Processing frame: {} bytes", data.len());
	
	// Skip empty frames
	if data.len() <= 4 {
		tracing::warn!("Skipping empty frame: {} bytes", data.len());
		return Ok(());
	}
	
	// Create a placeholder image (blue background)
	let pitch = width as usize * 3; // RGB = 3 bytes per pixel
	let height = texture.query().height;
	let mut rgb_data = vec![0u8; pitch * height as usize];
	
	// Fill with blue color (RGB format)
	for pixel in rgb_data.chunks_exact_mut(3) {
		pixel[0] = 0;    // R
		pixel[1] = 0;    // G
		pixel[2] = 255;  // B
	}
	
	// Add frame number as text (simple visualization)
	let frame_number = frame.timestamp.as_secs_f32() * 30.0; // Approximate frame number
	if frame.keyframe {
		tracing::info!("Keyframe received at timestamp: {:?}", frame.timestamp);
	}
	
	// Update the texture with our placeholder
	match texture.update(None, &rgb_data, pitch) {
		Ok(_) => Ok(()),
		Err(e) => {
			tracing::error!("Failed to update texture: {}", e);
			Err(anyhow::anyhow!("Failed to update texture: {}", e))
		}
	}
}
