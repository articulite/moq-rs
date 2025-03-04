use anyhow::{Context, Result};
use image::{ImageBuffer, Rgba};
use std::time::Instant;
use tracing::debug;
use x264::{Encoder, Picture, ColorSpace, nal::UnitType};

pub struct VideoEncoder {
    encoder: Encoder,
    last_keyframe: Instant,
    keyframe_interval: std::time::Duration,
    frame_count: u64,
}

pub struct EncodedFrame {
    pub data: Vec<u8>,
    pub timestamp: std::time::Duration,
    pub is_keyframe: bool,
}

impl VideoEncoder {
    pub fn new(width: u32, height: u32, bitrate: u32, fps: u32) -> Result<Self> {
        let mut param = x264::Param::default_preset("superfast", "zerolatency")
            .context("Failed to create encoder preset")?;
        
        // Set encoder parameters
        param.set_dimension(width as i32, height as i32);
        param.set_bitrate(bitrate as i32 / 1000);
        param.set_framerate(fps as i32, 1);
        param.set_keyint(fps as i32 * 2); // Keyframe every 2 seconds
        param.set_min_keyint(fps as i32); // Minimum keyframe interval
        param.set_repeat_headers(1); // Add SPS/PPS to each keyframe
        param.apply_profile("baseline")
            .context("Failed to apply encoder profile")?;
        
        // Create encoder
        let encoder = Encoder::open(&param)
            .context("Failed to create H264 encoder")?;
        
        Ok(Self {
            encoder,
            last_keyframe: Instant::now(),
            keyframe_interval: std::time::Duration::from_secs(2),
            frame_count: 0,
        })
    }
    
    pub fn encode_frame(&mut self, frame: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<EncodedFrame> {
        let width = frame.width() as i32;
        let height = frame.height() as i32;
        
        // Create H264 picture
        let mut pic = Picture::from_param(self.encoder.param())?;
        
        // Set timestamp in microseconds
        let timestamp = self.frame_count * 1_000_000 / self.encoder.param().framerate().1 as u64;
        pic.set_timestamp(timestamp as i64);
        
        // Convert RGBA to I420 (YUV)
        self.convert_to_yuv(frame, &mut pic);
        
        // Force keyframe if needed
        let force_keyframe = self.last_keyframe.elapsed() >= self.keyframe_interval;
        if force_keyframe {
            pic.set_keyframe(true);
            self.last_keyframe = Instant::now();
        }
        
        // Encode frame
        let nal_units = self.encoder.encode(&pic)
            .context("Failed to encode frame")?;
        
        // Check if this is a keyframe
        let is_keyframe = nal_units.iter().any(|nal| {
            matches!(nal.unit_type(), UnitType::SliceIDR)
        });
        
        if is_keyframe {
            debug!("Encoded keyframe at frame {}", self.frame_count);
        }
        
        // Combine NAL units into a single buffer
        let mut encoded_data = Vec::new();
        for nal in nal_units {
            encoded_data.extend_from_slice(nal.payload());
        }
        
        self.frame_count += 1;
        
        Ok(EncodedFrame {
            data: encoded_data,
            timestamp: std::time::Duration::from_micros(timestamp),
            is_keyframe,
        })
    }
    
    fn convert_to_yuv(&self, frame: &ImageBuffer<Rgba<u8>, Vec<u8>>, pic: &mut Picture) {
        let width = frame.width() as usize;
        let height = frame.height() as usize;
        
        let y_plane = pic.plane_mut(0);
        let u_plane = pic.plane_mut(1);
        let v_plane = pic.plane_mut(2);
        
        let y_stride = pic.stride(0) as usize;
        let u_stride = pic.stride(1) as usize;
        let v_stride = pic.stride(2) as usize;
        
        // Convert RGBA to YUV420
        for y in 0..height {
            for x in 0..width {
                let pixel = frame.get_pixel(x as u32, y as u32);
                
                // RGB to Y
                let y_value = (0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32) as u8;
                y_plane[y * y_stride + x] = y_value;
                
                // Downsample chroma (U and V)
                if y % 2 == 0 && x % 2 == 0 {
                    let u_x = x / 2;
                    let u_y = y / 2;
                    
                    // RGB to U
                    let u_value = (128.0 - 0.168736 * pixel[0] as f32 - 0.331264 * pixel[1] as f32 + 0.5 * pixel[2] as f32) as u8;
                    u_plane[u_y * u_stride + u_x] = u_value;
                    
                    // RGB to V
                    let v_value = (128.0 + 0.5 * pixel[0] as f32 - 0.418688 * pixel[1] as f32 - 0.081312 * pixel[2] as f32) as u8;
                    v_plane[u_y * v_stride + u_x] = v_value;
                }
            }
        }
    }
} 