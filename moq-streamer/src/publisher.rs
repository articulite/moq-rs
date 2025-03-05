use anyhow::{Context, Result};
use moq_karp::{BroadcastProducer, Dimensions, Video, Track, Frame, TrackProducer, H265};
use moq_native::quic;
use moq_transfork::{Session, coding::Bytes};
use tracing::{info, warn};
use url::Url;

use moq_x265::EncodedFrame;

pub struct MoqPublisher {
    broadcast: BroadcastProducer,
    video_track: Option<TrackProducer>,
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
        tls_args: moq_native::tls::Args,
    ) -> Result<Self> {
        // Initialize QUIC endpoint
        let quic_config = quic::Config {
            bind: "0.0.0.0:0".parse().unwrap(),
            tls: tls_args.load().context("Failed to load TLS config")?,
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
            // Get HEVC VPS/SPS/PPS from the first keyframe
            if !frame.is_keyframe {
                warn!("Waiting for keyframe to initialize video track");
                return Ok(());
            }
            
            info!("Creating video track with {}x{} @ {} bps", self.width, self.height, self.bitrate);
            
            // HEVC description (VPS/SPS/PPS)
            // Note: In a real implementation, you would extract these from the encoder
            // For now, use the first keyframe data as-is
            let description = Some(Bytes::copy_from_slice(&frame.data));
            
            // HEVC codec parameters (Main profile, Main tier, Level 4.1)
            let hevc_codec = H265 {
                profile_space: 0,
                profile_idc: 1, // Main profile
                profile_compatibility_flags: [0x60, 0, 0, 0], // Main profile compatibility
                tier_flag: false, // Main tier
                level_idc: 123, // Level 4.1
                constraint_flags: [0, 0, 0, 0, 0, 0],
            };
            
            // Video track info
            let video_info = Video {
                track: Track {
                    name: "video".to_string(),
                    priority: 2,
                },
                codec: hevc_codec.into(), // Use HEVC codec
                description,
                resolution: Dimensions {
                    width: self.width,
                    height: self.height,
                },
                bitrate: Some(self.bitrate as u64),
            };
            
            // Publish the video track
            self.video_track = Some(self.broadcast.publish_video(video_info)?);
            info!("Video track created successfully");
        }
        
        // Create MoQ frame
        let moq_frame = Frame {
            timestamp: frame.timestamp,
            keyframe: frame.is_keyframe,
            payload: Bytes::copy_from_slice(&frame.data),
        };
        
        // Write frame to track
        if let Some(track) = &mut self.video_track {
            track.write(moq_frame);
        }
        
        Ok(())
    }
} 