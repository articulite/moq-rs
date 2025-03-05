use anyhow::Context as _;
use clap::Parser;
use moq_karp::BroadcastConsumer;
use moq_native::{log, quic, tls};
use moq_transfork::Session;
use sdl2::pixels::PixelFormatEnum;
use tracing::{info, error, debug, warn};
use url::Url;
use std::time::Duration;
use moq_x265::{X265Decoder, Decoder};
use std::sync::Mutex;

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

// Create a static decoder that persists between frames
lazy_static::lazy_static! {
	static ref DECODER: Mutex<Box<dyn Decoder>> = {
		// Try to create a hardware decoder first
		if moq_x265::is_hardware_acceleration_available() {
			match moq_x265::create_hardware_decoder() {
				Ok(hw_decoder) => {
					info!("Using NVIDIA hardware decoder");
					Mutex::new(hw_decoder)
				},
				Err(e) => {
					warn!("Failed to create hardware decoder: {}, falling back to software", e);
					Mutex::new(Box::new(X265Decoder::new()))
				}
			}
		} else {
			info!("Hardware acceleration not available, using software decoder");
			Mutex::new(Box::new(X265Decoder::new()))
		}
	};
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
	tracing::info!("Processing frame: {} bytes, keyframe: {}", data.len(), frame.keyframe);
	
	// Skip empty frames
	if data.len() <= 4 {
		tracing::warn!("Skipping empty frame: {} bytes", data.len());
		return Ok(());
	}
	
	// Try to decode the frame using our x265 decoder
	let mut decoder = DECODER.lock().unwrap();
	match decoder.decode(data) {
		Ok(Some(image)) => {
			// Log image details
			tracing::info!("Decoded image: {}x{}", image.width(), image.height());
			
			// Convert the image to RGB format for SDL
			let pitch = width as usize * 3; // RGB = 3 bytes per pixel
			let height = texture.query().height;
			let mut rgb_data = vec![0u8; pitch * height as usize];
			
			tracing::debug!("Texture dimensions: {}x{}, pitch: {}", width, height, pitch);
			
			// Copy the image data to the RGB buffer
			for y in 0..height {
				for x in 0..width {
					if x < image.width() && y < image.height() {
						let pixel = image.get_pixel(x, y);
						let offset = (y as usize * pitch) + (x as usize * 3);
						if offset + 2 < rgb_data.len() {
							rgb_data[offset] = pixel[0];     // R
							rgb_data[offset + 1] = pixel[1]; // G
							rgb_data[offset + 2] = pixel[2]; // B
						}
					}
				}
			}
			
			// Sample a few pixels to verify data
			if frame.keyframe {
				let center_x = image.width() / 2;
				let center_y = image.height() / 2;
				let center_pixel = image.get_pixel(center_x, center_y);
				tracing::info!("Center pixel RGBA: [{}, {}, {}, {}]", 
					center_pixel[0], center_pixel[1], center_pixel[2], center_pixel[3]);
			}
			
			// Update the texture with the decoded image
			match texture.update(None, &rgb_data, pitch) {
				Ok(_) => {
					if frame.keyframe {
						tracing::info!("Keyframe decoded at timestamp: {:?}", frame.timestamp);
					}
					Ok(())
				},
				Err(e) => {
					tracing::error!("Failed to update texture: {}", e);
					Err(anyhow::anyhow!("Failed to update texture: {}", e))
				}
			}
		},
		Ok(None) => {
			tracing::warn!("Decoder returned None for frame");
			// If decoding failed or no frame was produced, show a blue screen as fallback
			let pitch = width as usize * 3; // RGB = 3 bytes per pixel
			let height = texture.query().height;
			let mut rgb_data = vec![0u8; pitch * height as usize];
			
			// Fill with blue color (RGB format)
			for pixel in rgb_data.chunks_exact_mut(3) {
				pixel[0] = 0;    // R
				pixel[1] = 0;    // G
				pixel[2] = 255;  // B
			}
			
			// Update the texture with the blue screen
			match texture.update(None, &rgb_data, pitch) {
				Ok(_) => Ok(()),
				Err(e) => {
					tracing::error!("Failed to update texture with blue screen: {}", e);
					Err(anyhow::anyhow!("Failed to update texture: {}", e))
				}
			}
		},
		Err(e) => {
			tracing::error!("Failed to decode frame: {}", e);
			Err(anyhow::anyhow!("Failed to decode frame: {}", e))
		}
	}
}
