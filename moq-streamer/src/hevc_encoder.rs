use anyhow::Result;
use image::{ImageBuffer, Rgba};
use moq_x265::{X265Encoder, EncodedFrame};

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
    encoder: X265Encoder,
}

impl HEVCEncoder {
    pub fn new(width: u32, height: u32, bitrate: u32, fps: u32) -> Result<Self> {
        // Default keyframe interval to 2 seconds (2 * fps)
        let keyframe_interval = 2 * fps;
        let encoder = X265Encoder::new(width, height, bitrate, fps, keyframe_interval)?;
        
        Ok(Self {
            width,
            height,
            bitrate,
            fps,
            encoder,
        })
    }
    
    pub fn encode_frame(&mut self, frame: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<EncodedFrame> {
        // Encode the frame using x265
        self.encoder.encode_frame(frame)
    }
}

impl Drop for HEVCEncoder {
    fn drop(&mut self) {
        // The X265Encoder will clean up resources in its own Drop implementation
    }
} 