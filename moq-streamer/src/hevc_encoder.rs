use anyhow::Result;
use image::{ImageBuffer, Rgba};
use moq_x265::{X265Encoder, EncodedFrame, Encoder};

// Initialize x265
pub fn init_x265() -> Result<()> {
    // x265 doesn't require explicit initialization
    Ok(())
}

// Shutdown x265
pub fn shutdown_x265() -> Result<()> {
    // x265 doesn't require explicit shutdown
    Ok(())
}

pub struct HEVCEncoder {
    width: u32,
    height: u32,
    bitrate: u32,
    fps: u32,
    encoder: Box<dyn Encoder>,
    using_hardware: bool,
}

impl HEVCEncoder {
    pub fn new(width: u32, height: u32, bitrate: u32, fps: u32) -> Result<Self> {
        // Use a shorter keyframe interval (1 second) for better compatibility
        let keyframe_interval = fps;
        
        // Try to create a hardware encoder first
        let (encoder, using_hardware) = if moq_x265::is_hardware_acceleration_available() {
            match moq_x265::create_hardware_encoder(width, height, bitrate, fps, keyframe_interval) {
                Ok(hw_encoder) => {
                    tracing::info!("Using NVIDIA hardware encoder");
                    (hw_encoder, true)
                },
                Err(e) => {
                    tracing::warn!("Failed to create hardware encoder: {}, falling back to software", e);
                    let sw_encoder = moq_x265::X265Encoder::new(width, height, bitrate, fps, keyframe_interval)?;
                    (Box::new(sw_encoder) as Box<dyn Encoder>, false)
                }
            }
        } else {
            tracing::info!("Hardware acceleration not available, using software encoder");
            let sw_encoder = moq_x265::X265Encoder::new(width, height, bitrate, fps, keyframe_interval)?;
            (Box::new(sw_encoder) as Box<dyn Encoder>, false)
        };
        
        Ok(Self {
            width,
            height,
            bitrate,
            fps,
            encoder,
            using_hardware,
        })
    }
    
    pub fn encode_frame(&mut self, frame: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<EncodedFrame> {
        // Log the frame dimensions
        tracing::info!("Encoding frame of size: {}x{}", frame.width(), frame.height());
        
        // Encode the frame
        let encoded_frame = self.encoder.encode(frame)?;
        
        // Log the encoded frame details
        tracing::info!(
            "Encoded frame: size={} bytes, keyframe={}, timestamp={}ms",
            encoded_frame.data.len(),
            encoded_frame.is_keyframe,
            encoded_frame.timestamp.as_millis()
        );
        
        // Print the first few bytes of the encoded frame for debugging
        if encoded_frame.data.len() >= 16 {
            let header = &encoded_frame.data[0..16];
            tracing::info!("Encoded frame header: {:02X?}", header);
        }
        
        Ok(encoded_frame)
    }
    
    pub fn is_using_hardware(&self) -> bool {
        self.using_hardware
    }
}

impl Drop for HEVCEncoder {
    fn drop(&mut self) {
        // The encoder will clean up resources in its own Drop implementation
    }
} 