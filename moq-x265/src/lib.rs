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
    yuv_buffer: Vec<u8>,
    headers: Vec<u8>,  // Store the headers for later use
}

unsafe impl Send for X265Encoder {}
unsafe impl Sync for X265Encoder {}

impl X265Encoder {
    pub fn new(width: u32, height: u32, bitrate: u32, fps: u32, keyframe_interval: u32) -> Result<Self> {
        unsafe {
            // Create parameter set
            let param = x265_param_alloc();
            if param.is_null() {
                return Err(anyhow!(X265Error::EncoderInitFailed("Failed to allocate parameters".into())));
            }
            
            // Initialize with default values
            x265_param_default(param);
            
            // Set parameters
            (*param).internalCsp = X265_CSP_I420 as i32;
            (*param).sourceWidth = width as i32;
            (*param).sourceHeight = height as i32;
            (*param).fpsNum = fps;
            (*param).fpsDenom = 1;
            (*param).rc.rateControlMode = X265_RC_METHODS_X265_RC_ABR as i32;
            (*param).rc.bitrate = bitrate as i32;
            
            // Set keyframe interval
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
            
            // Create a buffer for YUV data
            let yuv_size = (width * height * 3 / 2) as usize;
            let yuv_buffer = vec![0u8; yuv_size];
            
            // Create the encoder instance
            let mut encoder_instance = Self {
                encoder,
                param,
                pic_in,
                width,
                height,
                frame_count: 0,
                fps,
                last_keyframe_count: 0,
                keyframe_interval: keyframe_interval as u64,
                yuv_buffer,
                headers: Vec::new(),
            };
            
            // Write headers (SPS, PPS, VPS)
            let mut nal: *mut x265_nal = ptr::null_mut();
            let mut num_nal: u32 = 0;
            let headers_size = x265_encoder_headers(encoder, &mut nal, &mut num_nal);
            println!("Headers size: {} bytes, {} NALs", headers_size, num_nal);
            
            if headers_size > 0 && num_nal > 0 {
                println!("Successfully wrote headers");
                for i in 0..num_nal as usize {
                    let nal_unit = &*nal.add(i);
                    println!("Header NAL unit {}: type={}, size={} bytes", i, nal_unit.type_, nal_unit.sizeBytes);
                    encoder_instance.headers.extend_from_slice(std::slice::from_raw_parts(nal_unit.payload, nal_unit.sizeBytes as usize));
                }
            } else {
                println!("Failed to write headers");
            }
            
            Ok(encoder_instance)
        }
    }
    
    pub fn encode_frame(&mut self, frame: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<EncodedFrame> {
        unsafe {
            // Convert RGBA to I420 and store in our buffer
            rgba_to_i420_buffer(frame, self.width, self.height, &mut self.yuv_buffer)?;
            
            // Set picture data
            (*self.pic_in).planes[0] = self.yuv_buffer.as_ptr() as *mut libc::c_void; // Y
            (*self.pic_in).planes[1] = self.yuv_buffer.as_ptr().add((self.width * self.height) as usize) as *mut libc::c_void; // U
            (*self.pic_in).planes[2] = self.yuv_buffer.as_ptr().add((self.width * self.height + (self.width * self.height / 4)) as usize) as *mut libc::c_void; // V
            
            // Set stride
            (*self.pic_in).stride[0] = self.width as i32;
            (*self.pic_in).stride[1] = (self.width / 2) as i32;
            (*self.pic_in).stride[2] = (self.width / 2) as i32;
            
            // Set timestamp
            (*self.pic_in).pts = self.frame_count as i64;
            
            // Force keyframe if needed
            if self.frame_count % self.keyframe_interval == 0 {
                (*self.pic_in).sliceType = X265_TYPE_IDR as i32;
                println!("Setting slice type to IDR");
            } else {
                (*self.pic_in).sliceType = X265_TYPE_AUTO as i32;
                println!("Setting slice type to AUTO");
            }
            
            // Encode frame
            let mut nal: *mut x265_nal = ptr::null_mut();
            let mut num_nal: u32 = 0;
            
            println!("Calling x265_encoder_encode with pic_in: {:?}", self.pic_in);
            let frame_size = x265_encoder_encode(self.encoder, &mut nal, &mut num_nal, self.pic_in, ptr::null_mut());
            println!("x265_encoder_encode returned: {} bytes, {} NALs", frame_size, num_nal);
            
            if frame_size < 0 {
                return Err(anyhow!(X265Error::EncodeFailed("Failed to encode frame".into())));
            }
            
            // Copy encoded data
            let mut data = Vec::new();
            
            // For the first frame, include the headers
            if self.frame_count == 0 {
                data.extend_from_slice(&self.headers);
                println!("Including {} bytes of headers", self.headers.len());
            }
            
            // Add the frame data
            for i in 0..num_nal as usize {
                let nal_unit = &*nal.add(i);
                println!("NAL unit {}: type={}, size={} bytes", i, nal_unit.type_, nal_unit.sizeBytes);
                data.extend_from_slice(std::slice::from_raw_parts(nal_unit.payload, nal_unit.sizeBytes as usize));
            }
            
            // If we have no data but this is the first frame, just return the headers
            if data.is_empty() && self.frame_count == 0 && !self.headers.is_empty() {
                println!("No frame data, but returning headers");
                data = self.headers.clone();
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
    
    /// Set a parameter on the x265 encoder
    pub fn set_param(&mut self, name: &str, value: &str) -> Result<()> {
        let name_cstr = CString::new(name).map_err(|_| anyhow!("Invalid parameter name"))?;
        let value_cstr = CString::new(value).map_err(|_| anyhow!("Invalid parameter value"))?;
        
        unsafe {
            if self.param.is_null() {
                return Err(anyhow!("Encoder parameters not initialized"));
            }
            
            let result = x265_param_parse(self.param, name_cstr.as_ptr(), value_cstr.as_ptr());
            if result < 0 {
                return Err(anyhow!("Failed to set parameter {} to {}", name, value));
            }
            
            // Apply the parameter to the encoder
            if !self.encoder.is_null() {
                x265_encoder_reconfig(self.encoder, self.param);
            }
        }
        
        Ok(())
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
        // Skip empty data
        if data.is_empty() {
            return Ok(None);
        }
        
        // Parse NAL units
        let nal_units = parse_nal_units(data);
        let nal_count = nal_units.len();
        println!("Parsed {} NAL units", nal_count);
        
        // Process each NAL unit
        for nal in &nal_units {
            // Skip empty NAL units
            if nal.is_empty() {
                continue;
            }
            
            // Get NAL unit type (6 bits from the first byte after the header)
            if nal.len() < 1 {
                continue;
            }
            
            let nal_type = (nal[0] >> 1) & 0x3F;
            println!("Processing NAL unit type: {}", nal_type);
            
            match nal_type {
                32 => { // VPS
                    self.vps_data = Some(nal.to_vec());
                    println!("Found VPS, {} bytes", nal.len());
                },
                33 => { // SPS
                    self.sps_data = Some(nal.to_vec());
                    println!("Found SPS, {} bytes", nal.len());
                    
                    // Extract resolution from SPS
                    if let Some((width, height)) = extract_resolution_from_sps(nal) {
                        self.width = width;
                        self.height = height;
                        println!("Resolution: {}x{}", width, height);
                    }
                },
                34 => { // PPS
                    self.pps_data = Some(nal.to_vec());
                    println!("Found PPS, {} bytes", nal.len());
                },
                _ => {
                    // Other NAL types (slices, etc.)
                    println!("Found NAL type {}, {} bytes", nal_type, nal.len());
                }
            }
        }
        
        // For now, just return None since we don't have a full decoder implementation
        // In a real implementation, you would decode the frame and return the image
        println!("Decoder processed {} NAL units but doesn't have full decoding capability", nal_count);
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
    let mut found_start = false;
    
    // Find NAL unit boundaries (0x000001 or 0x00000001)
    for i in 0..data.len().saturating_sub(3) {
        if (i + 3 < data.len() && data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 1) ||
           (i + 4 < data.len() && data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 0 && data[i + 3] == 1) {
            
            // If we've already found a start code, add the NAL unit
            if found_start && start_idx < i {
                nal_units.push(&data[start_idx..i]);
            }
            
            // Set the new start index after the start code
            if i + 2 < data.len() && data[i + 2] == 1 {
                start_idx = i + 3;
            } else if i + 3 < data.len() {
                start_idx = i + 4;
            }
            
            found_start = true;
        }
    }
    
    // Add the last NAL unit
    if found_start && start_idx < data.len() {
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

fn rgba_to_i420_buffer(
    rgba: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    width: u32,
    height: u32,
    buffer: &mut Vec<u8>,
) -> Result<()> {
    let w = width as usize;
    let h = height as usize;
    
    // Ensure buffer is large enough
    let required_size = (width * height * 3 / 2) as usize;
    if buffer.len() < required_size {
        buffer.resize(required_size, 0);
    }
    
    // Split the buffer into Y, U, and V planes
    let (y_part, uv_part) = buffer.split_at_mut(w * h);
    let (u_part, v_part) = uv_part.split_at_mut(w * h / 4);
    
    // Fill with test pattern for debugging
    // Y plane (full brightness)
    for i in 0..y_part.len() {
        y_part[i] = 235; // Almost white in Y (studio range)
    }
    
    // U and V planes (neutral color)
    for i in 0..u_part.len() {
        u_part[i] = 128; // Neutral U
        v_part[i] = 128; // Neutral V
    }
    
    // Now convert the actual image
    for y in 0..h {
        for x in 0..w {
            let rgba = rgba.get_pixel(x as u32, y as u32);
            let r = rgba[0] as f32;
            let g = rgba[1] as f32;
            let b = rgba[2] as f32;
            
            // RGB to YUV conversion (BT.709)
            let y_val = (0.2126 * r + 0.7152 * g + 0.0722 * b) as u8;
            y_part[y * w + x] = y_val;
            
            // Downsample for U and V planes (4:2:0)
            if x % 2 == 0 && y % 2 == 0 {
                let u_x = x / 2;
                let u_y = y / 2;
                let u_idx = u_y * (w / 2) + u_x;
                
                let u_val = (-0.09991 * r - 0.33609 * g + 0.436 * b + 128.0) as u8;
                let v_val = (0.615 * r - 0.55861 * g - 0.05639 * b + 128.0) as u8;
                
                u_part[u_idx] = u_val;
                v_part[u_idx] = v_val;
            }
        }
    }
    
    Ok(())
} 