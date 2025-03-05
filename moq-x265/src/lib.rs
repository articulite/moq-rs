#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use anyhow::{anyhow, Result};
use image::{ImageBuffer, Rgba};
use std::ffi::CString;
use std::ptr;
use std::time::Duration;
use thiserror::Error;

// Include the generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[derive(Error, Debug)]
pub enum X265Error {
    #[error("Failed to initialize encoder: {0}")]
    EncoderInitFailed(String),
    
    #[error("Failed to encode frame: {0}")]
    EncodeFailed(String),
    
    #[error("Failed to initialize decoder: {0}")]
    DecoderInitFailed(String),
    
    #[error("Failed to decode frame: {0}")]
    DecodeFailed(String),
}

pub struct EncodedFrame {
    pub data: Vec<u8>,
    pub timestamp: Duration,
    pub is_keyframe: bool,
}

pub struct X265Encoder {
    encoder: *mut x265_encoder,
    param: *mut x265_param,
    pic_in: *mut x265_picture,
    width: u32,
    height: u32,
    frame_count: u64,
    fps: u32,
    last_keyframe_count: u64,
    keyframe_interval: u64,
}

unsafe impl Send for X265Encoder {}
unsafe impl Sync for X265Encoder {}

impl X265Encoder {
    pub fn new(width: u32, height: u32, bitrate: u32, fps: u32) -> Result<Self> {
        unsafe {
            // Create parameter set
            let param = x265_param_alloc();
            if param.is_null() {
                return Err(anyhow!(X265Error::EncoderInitFailed("Failed to allocate parameters".into())));
            }
            
            // Initialize with default values
            x265_param_default(param);
            
            // Set parameters
            (*param).bRepeatHeaders = 1;
            (*param).bAnnexB = 1;
            (*param).internalCsp = X265_CSP_I420 as i32;
            (*param).sourceWidth = width as i32;
            (*param).sourceHeight = height as i32;
            (*param).fpsNum = fps;
            (*param).fpsDenom = 1;
            (*param).rc.rateControlMode = X265_RC_METHODS_X265_RC_ABR as i32;
            (*param).rc.bitrate = bitrate as i32;
            
            // Set keyframe interval (2 seconds)
            let keyframe_interval = fps * 2;
            (*param).keyframeMax = keyframe_interval as i32;
            (*param).keyframeMin = keyframe_interval as i32;
            
            // Set other parameters
            (*param).bRepeatHeaders = 1; // Repeat SPS/PPS headers
            (*param).bAnnexB = 1; // Use Annex B format (start codes)
            
            // Create encoder
            // Get the API struct
            let api = x265_api_get_199(8); // 8-bit depth
            if api.is_null() {
                x265_param_free(param);
                return Err(anyhow!(X265Error::EncoderInitFailed("Failed to get x265 API".into())));
            }
            
            // Use the encoder_open function from the API
            let encoder = ((*api).encoder_open.unwrap())(param);
            if encoder.is_null() {
                x265_param_free(param);
                return Err(anyhow!(X265Error::EncoderInitFailed("Failed to open encoder".into())));
            }
            
            // Allocate picture
            let pic_in = x265_picture_alloc();
            if pic_in.is_null() {
                x265_encoder_close(encoder);
                x265_param_free(param);
                return Err(anyhow!(X265Error::EncoderInitFailed("Failed to allocate picture".into())));
            }
            
            // Initialize picture
            x265_picture_init(param, pic_in);
            
            Ok(Self {
                encoder,
                param,
                pic_in,
                width,
                height,
                frame_count: 0,
                fps,
                last_keyframe_count: 0,
                keyframe_interval: keyframe_interval as u64,
            })
        }
    }
    
    pub fn encode_frame(&mut self, frame: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<EncodedFrame> {
        unsafe {
            // Convert RGBA to I420
            let yuv_data = rgba_to_i420(frame, self.width, self.height)?;
            
            // Set picture data
            (*self.pic_in).planes[0] = yuv_data.as_ptr() as *mut libc::c_void; // Y
            (*self.pic_in).planes[1] = yuv_data.as_ptr().add((self.width * self.height) as usize) as *mut libc::c_void; // U
            (*self.pic_in).planes[2] = yuv_data.as_ptr().add((self.width * self.height + (self.width * self.height / 4)) as usize) as *mut libc::c_void; // V
            
            // Set stride
            (*self.pic_in).stride[0] = self.width as i32;
            (*self.pic_in).stride[1] = (self.width / 2) as i32;
            (*self.pic_in).stride[2] = (self.width / 2) as i32;
            
            // Set timestamp
            (*self.pic_in).pts = self.frame_count as i64;
            
            // Force keyframe if needed
            if self.frame_count % self.keyframe_interval == 0 {
                (*self.pic_in).sliceType = X265_TYPE_IDR as i32;
            } else {
                (*self.pic_in).sliceType = X265_TYPE_AUTO as i32;
            }
            
            // Encode frame
            let mut nal: *mut x265_nal = ptr::null_mut();
            let mut num_nal: u32 = 0;
            
            let frame_size = x265_encoder_encode(self.encoder, &mut nal, &mut num_nal, self.pic_in, ptr::null_mut());
            if frame_size < 0 {
                return Err(anyhow!(X265Error::EncodeFailed("Failed to encode frame".into())));
            }
            
            // Copy encoded data
            let mut data = Vec::new();
            for i in 0..num_nal as usize {
                let nal_unit = &*nal.add(i);
                data.extend_from_slice(std::slice::from_raw_parts(nal_unit.payload, nal_unit.sizeBytes as usize));
            }
            
            // Check if this is a keyframe
            let is_keyframe = self.frame_count % self.keyframe_interval == 0;
            if is_keyframe {
                self.last_keyframe_count = self.frame_count;
            }
            
            // Increment frame count
            self.frame_count += 1;
            
            Ok(EncodedFrame {
                data,
                timestamp: Duration::from_secs_f64(self.frame_count as f64 / self.fps as f64),
                is_keyframe,
            })
        }
    }
    
    pub fn flush(&mut self) -> Result<Option<EncodedFrame>> {
        unsafe {
            let mut nal: *mut x265_nal = ptr::null_mut();
            let mut num_nal: u32 = 0;
            
            let frame_size = x265_encoder_encode(self.encoder, &mut nal, &mut num_nal, ptr::null_mut(), ptr::null_mut());
            if frame_size < 0 {
                return Err(anyhow!(X265Error::EncodeFailed("Failed to flush encoder".into())));
            }
            
            if frame_size == 0 || num_nal == 0 {
                return Ok(None);
            }
            
            // Copy encoded data
            let mut data = Vec::new();
            for i in 0..num_nal as usize {
                let nal_unit = &*nal.add(i);
                data.extend_from_slice(std::slice::from_raw_parts(nal_unit.payload, nal_unit.sizeBytes as usize));
            }
            
            // Increment frame count
            self.frame_count += 1;
            
            Ok(Some(EncodedFrame {
                data,
                timestamp: Duration::from_secs_f64(self.frame_count as f64 / self.fps as f64),
                is_keyframe: false,
            }))
        }
    }
}

impl Drop for X265Encoder {
    fn drop(&mut self) {
        unsafe {
            if !self.encoder.is_null() {
                x265_encoder_close(self.encoder);
            }
            if !self.param.is_null() {
                x265_param_free(self.param);
            }
            if !self.pic_in.is_null() {
                x265_picture_free(self.pic_in);
            }
        }
    }
}

pub struct X265Decoder {
    // For x265, we'll use a third-party decoder since x265 itself is encoder-only
    // We'll implement a simple H.265 parser and decoder
    sps_data: Option<Vec<u8>>,
    pps_data: Option<Vec<u8>>,
    vps_data: Option<Vec<u8>>,
    width: u32,
    height: u32,
    frame_buffer: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
}

impl X265Decoder {
    pub fn new() -> Self {
        Self {
            sps_data: None,
            pps_data: None,
            vps_data: None,
            width: 0,
            height: 0,
            frame_buffer: None,
        }
    }
    
    pub fn decode_frame(&mut self, data: &[u8]) -> Result<Option<ImageBuffer<Rgba<u8>, Vec<u8>>>> {
        // Parse NAL units
        let nal_units = parse_nal_units(data);
        
        // Process each NAL unit
        for nal in nal_units {
            if nal.is_empty() {
                continue;
            }
            
            let nal_type = (nal[0] >> 1) & 0x3F;
            
            match nal_type {
                32 => {
                    // VPS
                    self.vps_data = Some(nal.to_vec());
                    tracing::debug!("Found VPS NAL unit");
                }
                33 => {
                    // SPS
                    self.sps_data = Some(nal.to_vec());
                    tracing::debug!("Found SPS NAL unit");
                    
                    // Extract resolution from SPS (simplified)
                    if let Some((w, h)) = extract_resolution_from_sps(&nal) {
                        self.width = w;
                        self.height = h;
                        tracing::debug!("Resolution: {}x{}", w, h);
                    }
                }
                34 => {
                    // PPS
                    self.pps_data = Some(nal.to_vec());
                    tracing::debug!("Found PPS NAL unit");
                }
                _ => {
                    if nal_type <= 31 {
                        tracing::trace!("Found slice NAL unit type {}", nal_type);
                        
                        // This is a slice, we should decode it
                        // For now, just create a dummy image
                        if self.width > 0 && self.height > 0 {
                            if self.frame_buffer.is_none() {
                                self.frame_buffer = Some(ImageBuffer::new(self.width, self.height));
                            }
                            
                            // Return the frame buffer
                            return Ok(self.frame_buffer.clone());
                        }
                    }
                }
            }
        }
        
        Ok(None)
    }
}

fn c_str(s: &str) -> Result<CString> {
    Ok(CString::new(s)?)
}

fn rgba_to_i420(frame: &ImageBuffer<Rgba<u8>, Vec<u8>>, width: u32, height: u32) -> Result<Vec<u8>> {
    let width = width as usize;
    let height = height as usize;
    
    // Calculate plane sizes
    let y_size = width * height;
    let u_size = width * height / 4;
    let v_size = width * height / 4;
    
    // Allocate buffer for YUV data
    let mut yuv_data = vec![0u8; y_size + u_size + v_size];
    
    // Split the buffer into planes using split_at_mut
    let (y_part, uv_part) = yuv_data.split_at_mut(y_size);
    let (u_part, v_part) = uv_part.split_at_mut(u_size);
    
    // Convert RGBA to I420
    for y in 0..height {
        for x in 0..width {
            let rgba = frame.get_pixel(x as u32, y as u32);
            
            // Convert RGB to Y
            let y_value = (0.299 * rgba[0] as f32 + 0.587 * rgba[1] as f32 + 0.114 * rgba[2] as f32) as u8;
            
            // Store Y
            y_part[y * width + x] = y_value;
            
            // Downsample and convert to U and V (4:2:0)
            if y % 2 == 0 && x % 2 == 0 {
                let u_value = (128.0 - 0.168736 * rgba[0] as f32 - 0.331264 * rgba[1] as f32 + 0.5 * rgba[2] as f32) as u8;
                let v_value = (128.0 + 0.5 * rgba[0] as f32 - 0.418688 * rgba[1] as f32 - 0.081312 * rgba[2] as f32) as u8;
                
                let u_index = (y / 2) * (width / 2) + (x / 2);
                let v_index = (y / 2) * (width / 2) + (x / 2);
                
                u_part[u_index] = u_value;
                v_part[v_index] = v_value;
            }
        }
    }
    
    Ok(yuv_data)
}

fn parse_nal_units(data: &[u8]) -> Vec<&[u8]> {
    let mut nal_units = Vec::new();
    let mut start_idx = 0;
    
    // Find NAL unit boundaries (0x000001 or 0x00000001)
    for i in 0..data.len().saturating_sub(3) {
        if (i + 3 < data.len() && data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 1) ||
           (i + 4 < data.len() && data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 0 && data[i + 3] == 1) {
            
            // If we've already found a start code, add the NAL unit
            if start_idx > 0 {
                nal_units.push(&data[start_idx..i]);
            }
            
            // Set the new start index after the start code
            if data[i + 2] == 1 {
                start_idx = i + 3;
            } else {
                start_idx = i + 4;
            }
        }
    }
    
    // Add the last NAL unit
    if start_idx < data.len() {
        nal_units.push(&data[start_idx..]);
    }
    
    nal_units
}

fn extract_resolution_from_sps(_sps: &[u8]) -> Option<(u32, u32)> {
    // This is a simplified implementation
    // In a real implementation, you would parse the SPS to extract width and height
    // For now, we'll just return a default resolution
    Some((640, 480))
} 