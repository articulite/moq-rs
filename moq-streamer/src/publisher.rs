use anyhow::{Context, Result};
use moq_karp::{BroadcastProducer, Dimensions, Video, Track, Frame};
use moq_native::quic;
use moq_transfork::Session;
use std::time::Duration;
use tracing::{info, debug, warn};
use url::Url;

use crate::encoder::EncodedFrame;

pub struct MoqPublisher {
    broadcast: BroadcastProducer,
    video_track: Option<moq_karp::VideoTrackProducer>,
    width: u32,
    height: u32,
    bitrate: u32,
}

impl MoqPublisher {
    pub async fn new(
        server_url: Url,
        stream_name: String,
        width: u32,
        height: u32,
        bitrate: u32,
    ) -> Result<Self> {
        // Initialize QUIC endpoint
        let quic_config = quic::Config {
            bind: "[::]:0".parse().unwrap(),
            tls: moq_native::tls::Args::default().load().context("Failed to load TLS config")?,
        };
        
        let quic = quic::Endpoint::new(quic_config)
            .context("Failed to create QUIC endpoint")?;
        
        // Connect to server
        info!("Connecting to MoQ relay at {}", server_url);
        let transport_session = quic.client.connect(server_url).await
            .context("Failed to connect to relay server")?;
        
        // Create MoQ session
        let moq_session = Session::connect(transport_session).await
            .context("Failed to establish MoQ session")?;
        
        // Create broadcast
        let broadcast = BroadcastProducer::new(moq_session, stream_name)
            .context("Failed to create broadcast")?;
        
        Ok(Self {
            broadcast,
            video_track: None,
            width,
            height,
            bitrate,
        })
    }
    
    pub async fn publish_frame(&mut self, frame: EncodedFrame) -> Result<()> {
        // Create video track if not already created
        if self.video_track.is_none() {
            // Get H.264 SPS/PPS from the first keyframe
            if !frame.is_keyframe {
                warn!("Waiting for keyframe to initialize video track");
                return Ok(());
            }
            
            info!("Creating video track with {}x{} @ {} bps", self.width, self.height, self.bitrate);
            
            // H.264 description (SPS/PPS)
            // Note: In a real implementation, you would extract these from the encoder
            // For now, use the first keyframe data as-is
            let description = frame.data.clone();
            
            // Video track info
            let video_info = Video {
                track: Track {
                    name: "video".to_string(),
                    priority: 2,
                },
                codec: "avc1.42001e".to_string(), // H.264 baseline profile
                description,
                resolution: Dimensions {
                    width: self.width,
                    height: self.height,
                },
                bitrate: Some(self.bitrate),
            };
            
            // Publish the video track
            self.video_track = Some(self.broadcast.publish_video(video_info)?);
            info!("Video track created successfully");
        }
        
        // Create MoQ frame
        let moq_frame = Frame {
            timestamp: frame.timestamp,
            keyframe: frame.is_keyframe,
            payload: frame.data,
        };
        
        // Write frame to track
        if let Some(track) = self.video_track.as_mut() {
            debug!(
                "Publishing frame: timestamp={:?}, keyframe={}, size={} bytes",
                moq_frame.timestamp, moq_frame.keyframe, moq_frame.payload.len()
            );
            track.write(moq_frame);
        }
        
        Ok(())
    }
} 