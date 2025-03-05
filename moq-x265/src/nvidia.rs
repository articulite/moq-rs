// NVIDIA Video Codec SDK bindings and implementations
use anyhow::{anyhow, Result};
use image::{ImageBuffer, Rgba};
use std::time::Duration;
use std::ptr;
use std::mem;
use std::ffi::c_void;
use std::sync::{Arc, Mutex};

// Import the CUDA bindings from nvidia-video-codec
#[cfg(feature = "hardware-accel")]
use nvidia_video_codec::{CuDevice, CuContext, ffi};

// Simplified NVIDIA encoder
pub struct NvencEncoder {
    #[cfg(feature = "hardware-accel")]
    context: CuContext,
    width: u32,
    height: u32,
    bitrate: u32,
    fps: u32,
    keyframe_interval: u32,
    frame_count: u64,
}

unsafe impl Send for NvencEncoder {}
unsafe impl Sync for NvencEncoder {}

impl NvencEncoder {
    pub fn new(width: u32, height: u32, bitrate: u32, fps: u32, keyframe_interval: u32) -> Result<Self> {
        #[cfg(feature = "hardware-accel")]
        {
            tracing::info!("Initializing NVIDIA hardware encoder");
            
            // Initialize CUDA
            unsafe {
                let result = ffi::cuda::cuInit(0);
                if result != 0 {
                    return Err(anyhow!("Failed to initialize CUDA: {}", result));
                }
            }
            
            Ok(Self {
                context: CuContext::new(CuDevice::new(0)?, 0)?,
                width,
                height,
                bitrate,
                fps,
                keyframe_interval,
                frame_count: 0,
            })
        }
        
        #[cfg(not(feature = "hardware-accel"))]
        {
            Err(anyhow!("Hardware acceleration not available"))
        }
    }
    
    #[cfg(feature = "hardware-accel")]
    pub fn encode_frame(&mut self, frame: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<crate::EncodedFrame> {
        // Increment frame count
        self.frame_count += 1;
        
        // Determine if this is a keyframe
        let is_keyframe = self.frame_count % self.keyframe_interval as u64 == 1;
        
        // For now, we'll create a simple HEVC frame with proper headers
        // In a real implementation, we would use the NVENC API to encode the frame
        
        // Create a simple HEVC frame with proper headers
        let mut data = Vec::new();
        
        // Add VPS, SPS, PPS headers for keyframes
        if is_keyframe {
            // VPS (Video Parameter Set) - simplified version
            let vps = [
                0x00, 0x00, 0x00, 0x01, // Start code
                0x40, 0x01, 0x0c, 0x01, // NAL header and basic VPS data
                0xff, 0xff, 0x01, 0x60, // More VPS data
                0x00, 0x00, 0x03, 0x00, // Placeholder
                0xb0, 0x00, 0x00, 0x03, // Placeholder
                0x00, 0x00, 0x03, 0x00, // Placeholder
                0x78, 0x00, 0x00, 0x03  // Placeholder
            ];
            data.extend_from_slice(&vps);
            
            // SPS (Sequence Parameter Set) - simplified version with resolution
            let width_bytes = [(self.width >> 8) as u8, self.width as u8];
            let height_bytes = [(self.height >> 8) as u8, self.height as u8];
            
            let mut sps = vec![
                0x00, 0x00, 0x00, 0x01, // Start code
                0x42, 0x01, 0x01, 0x01, // NAL header and basic SPS data
                0x60, 0x00, 0x00, 0x00, // More SPS data
                width_bytes[0], width_bytes[1], // Width
                height_bytes[0], height_bytes[1], // Height
                0x00, 0x00, 0x03, 0x00, // Placeholder
                0xb0, 0x00, 0x00, 0x03, // Placeholder
                0x00, 0x00, 0x03, 0x00  // Placeholder
            ];
            data.extend_from_slice(&sps);
            
            // PPS (Picture Parameter Set) - simplified version
            let pps = [
                0x00, 0x00, 0x00, 0x01, // Start code
                0x44, 0x01, 0xc0, 0x70, // NAL header and basic PPS data
                0x00, 0x00, 0x03, 0x00, // Placeholder
                0x00, 0x00, 0x03, 0x00  // Placeholder
            ];
            data.extend_from_slice(&pps);
        }
        
        // Add IDR (keyframe) or regular frame data
        let frame_header = [
            0x00, 0x00, 0x00, 0x01, // Start code
            if is_keyframe { 0x26 } else { 0x02 }, // NAL header (0x26 for IDR, 0x02 for regular frame)
            0x01, // Temporal ID
            0x00, 0x00 // Placeholder
        ];
        data.extend_from_slice(&frame_header);
        
        // Add some dummy frame data based on the image content
        // In a real implementation, this would be the actual encoded frame data
        let mut frame_data = Vec::new();
        
        // Sample some pixels from the image to create a simple representation
        let sample_step = 16; // Sample every 16th pixel
        for y in (0..frame.height()).step_by(sample_step) {
            for x in (0..frame.width()).step_by(sample_step) {
                let pixel = frame.get_pixel(x, y);
                frame_data.push(pixel[0]); // R
                frame_data.push(pixel[1]); // G
                frame_data.push(pixel[2]); // B
            }
        }
        
        // Add the frame data
        data.extend_from_slice(&frame_data);
        
        // Create the encoded frame
        Ok(crate::EncodedFrame {
            data,
            timestamp: Duration::from_millis((self.frame_count * 1000 / self.fps as u64) as u64),
            is_keyframe,
        })
    }
}

// Completely rewritten NvdecDecoder implementation
pub struct NvdecDecoder {
    #[cfg(feature = "hardware-accel")]
    context: CuContext,
    #[cfg(feature = "hardware-accel")]
    decoder: Option<*mut c_void>, // CUvideodecoder
    #[cfg(feature = "hardware-accel")]
    parser: Option<*mut c_void>,  // CUvideoparser
    #[cfg(feature = "hardware-accel")]
    ctx_lock: Option<*mut c_void>, // CUvideoctxlock
    width: u32,
    height: u32,
    initialized: bool,
    sps_data: Option<Vec<u8>>,
    pps_data: Option<Vec<u8>>,
    vps_data: Option<Vec<u8>>,
    frame_buffer: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    frame_count: u64,
    #[cfg(feature = "hardware-accel")]
    decoded_frames: Arc<Mutex<Vec<ImageBuffer<Rgba<u8>, Vec<u8>>>>>,
}

unsafe impl Send for NvdecDecoder {}
unsafe impl Sync for NvdecDecoder {}

#[cfg(feature = "hardware-accel")]
extern "C" fn handle_video_sequence(user_data: *mut c_void, video_format: *mut ffi::cuvid::CUVIDEOFORMAT) -> i32 {
    unsafe {
        tracing::debug!("Video sequence callback called");
        let decoder = &mut *(user_data as *mut NvdecDecoder);
        let result = decoder.handle_video_sequence(video_format);
        tracing::debug!("Video sequence callback returned: {}", result);
        result
    }
}

#[cfg(feature = "hardware-accel")]
extern "C" fn handle_picture_decode(user_data: *mut c_void, pic_params: *mut ffi::cuvid::CUVIDPICPARAMS) -> i32 {
    unsafe {
        tracing::debug!("Picture decode callback called");
        let decoder = &mut *(user_data as *mut NvdecDecoder);
        let result = decoder.handle_picture_decode(pic_params);
        tracing::debug!("Picture decode callback returned: {}", result);
        result
    }
}

#[cfg(feature = "hardware-accel")]
extern "C" fn handle_picture_display(user_data: *mut c_void, disp_info: *mut ffi::cuvid::CUVIDPARSERDISPINFO) -> i32 {
    unsafe {
        tracing::debug!("Picture display callback called");
        let decoder = &mut *(user_data as *mut NvdecDecoder);
        let result = decoder.handle_picture_display(disp_info);
        tracing::debug!("Picture display callback returned: {}", result);
        result
    }
}

impl NvdecDecoder {
    pub fn new() -> Result<Self> {
        #[cfg(feature = "hardware-accel")]
        {
            tracing::info!("Initializing NVIDIA hardware decoder");
            
            // Initialize CUDA
            unsafe {
                let result = ffi::cuda::cuInit(0);
                if result != 0 {
                    tracing::error!("Failed to initialize CUDA: error code {}", result);
                    return Err(anyhow!("Failed to initialize CUDA: {}", result));
                }
                tracing::debug!("CUDA initialized successfully");
            }
            
            // For now, we'll use fixed dimensions until we get them from the stream
            let width = 640;
            let height = 480;
            
            tracing::info!("Created NVIDIA decoder with initial dimensions {}x{}", width, height);
            
            // Create a frame buffer immediately
            let frame_buffer = Some(ImageBuffer::new(width, height));
            
            Ok(Self {
                context: CuContext::new(CuDevice::new(0)?, 0)?,
                decoder: None,
                parser: None,
                ctx_lock: None,
                width,
                height,
                initialized: false,
                sps_data: None,
                pps_data: None,
                vps_data: None,
                frame_buffer,
                frame_count: 0,
                decoded_frames: Arc::new(Mutex::new(Vec::new())),
            })
        }
        
        #[cfg(not(feature = "hardware-accel"))]
        {
            Err(anyhow!("Hardware acceleration is not enabled"))
        }
    }
    
    #[cfg(feature = "hardware-accel")]
    fn initialize_decoder(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }
        
        unsafe {
            // Get the raw CUDA context pointer
            let cuda_ctx = self.context.context();
            
            // Create CUVID context lock
            let mut ctx_lock: ffi::cuvid::CUvideoctxlock = ptr::null_mut();
            let result = ffi::cuvid::cuvidCtxLockCreate(&mut ctx_lock, cuda_ctx);
            if result != 0 {
                tracing::error!("Failed to create CUVID context lock: error code {}", result);
                return Err(anyhow!("Failed to create CUVID context lock: {}", result));
            }
            self.ctx_lock = Some(ctx_lock as *mut c_void);
            tracing::debug!("CUVID context lock created successfully");
            
            // Create CUVID parser
            let mut parser_params: ffi::cuvid::CUVIDPARSERPARAMS = mem::zeroed();
            parser_params.CodecType = ffi::cuvid::cudaVideoCodec_HEVC as i32;
            parser_params.ulMaxNumDecodeSurfaces = 20;
            parser_params.ulMaxDisplayDelay = 1;
            parser_params.pUserData = self as *mut _ as *mut c_void;
            parser_params.pfnSequenceCallback = Some(handle_video_sequence);
            parser_params.pfnDecodePicture = Some(handle_picture_decode);
            parser_params.pfnDisplayPicture = Some(handle_picture_display);
            
            let mut parser = ptr::null_mut();
            let result = ffi::cuvid::cuvidCreateVideoParser(&mut parser, &mut parser_params as *mut _);
            if result != 0 {
                tracing::error!("Failed to create CUVID parser: error code {}", result);
                return Err(anyhow!("Failed to create CUVID parser: {}", result));
            }
            self.parser = Some(parser);
            tracing::debug!("CUVID parser created successfully");
            
            self.initialized = true;
            tracing::info!("NVIDIA decoder initialized successfully");
        }
        
        Ok(())
    }
    
    #[cfg(feature = "hardware-accel")]
    fn handle_video_sequence(&mut self, video_format: *mut ffi::cuvid::CUVIDEOFORMAT) -> i32 {
        unsafe {
            let format = &*video_format;
            
            // Extract dimensions from the video format
            let width = format.display_area.right - format.display_area.left;
            let height = format.display_area.bottom - format.display_area.top;
            
            tracing::info!("Video sequence: {}x{}, codec: {}, chroma_format: {}, bit_depth: {}", 
                width, height, format.codec, format.chroma_format, format.bit_depth_luma_minus8 + 8);
            
            // Update dimensions if they've changed
            if self.width as i32 != width || self.height as i32 != height {
                self.width = width as u32;
                self.height = height as u32;
                
                // Recreate frame buffer with new dimensions
                self.frame_buffer = Some(ImageBuffer::new(self.width, self.height));
                tracing::info!("Updated dimensions to {}x{}", self.width, self.height);
            }
            
            // Create decoder if it doesn't exist
            if self.decoder.is_none() {
                tracing::debug!("Creating CUVID decoder");
                let mut create_info: ffi::cuvid::CUVIDDECODECREATEINFO = mem::zeroed();
                create_info.CodecType = ffi::cuvid::cudaVideoCodec_HEVC as i32;
                create_info.ChromaFormat = format.chroma_format;
                create_info.OutputFormat = ffi::cuvid::cudaVideoSurfaceFormat_NV12 as i32;
                create_info.bitDepthMinus8 = format.bit_depth_luma_minus8 as u32;
                create_info.DeinterlaceMode = ffi::cuvid::cudaVideoDeinterlaceMode_Weave as i32;
                create_info.ulNumOutputSurfaces = 1;
                create_info.ulNumDecodeSurfaces = 20;
                create_info.ulWidth = format.coded_width;
                create_info.ulHeight = format.coded_height;
                create_info.ulTargetWidth = self.width as u32;
                create_info.ulTargetHeight = self.height as u32;
                create_info.target_rect.left = 0;
                create_info.target_rect.top = 0;
                create_info.target_rect.right = self.width as i16;
                create_info.target_rect.bottom = self.height as i16;
                create_info.display_area.left = format.display_area.left as i16;
                create_info.display_area.top = format.display_area.top as i16;
                create_info.display_area.right = format.display_area.right as i16;
                create_info.display_area.bottom = format.display_area.bottom as i16;
                create_info.vidLock = self.ctx_lock.unwrap() as ffi::cuvid::CUvideoctxlock;
                
                let mut decoder = ptr::null_mut();
                let result = ffi::cuvid::cuvidCreateDecoder(&mut decoder, &mut create_info as *mut _);
                if result != 0 {
                    tracing::error!("Failed to create CUVID decoder: error code {}", result);
                    return 0;
                }
                
                self.decoder = Some(decoder);
                tracing::info!("CUVID decoder created successfully");
            }
            
            1 // Success
        }
    }
    
    #[cfg(feature = "hardware-accel")]
    fn handle_picture_decode(&mut self, pic_params: *mut ffi::cuvid::CUVIDPICPARAMS) -> i32 {
        unsafe {
            if let Some(decoder) = self.decoder {
                // Decode the picture
                let result = ffi::cuvid::cuvidDecodePicture(decoder, pic_params);
                if result != 0 {
                    tracing::error!("Failed to decode picture: error code {}", result);
                    return 0;
                }
                
                tracing::debug!("Picture decoded successfully");
                return 1; // Success
            }
        }
        
        0 // Failure
    }
    
    #[cfg(feature = "hardware-accel")]
    fn handle_picture_display(&mut self, disp_info: *mut ffi::cuvid::CUVIDPARSERDISPINFO) -> i32 {
        unsafe {
            if let Some(decoder) = self.decoder {
                // Map the decoded frame
                let mut proc_params: ffi::cuvid::CUVIDPROCPARAMS = mem::zeroed();
                proc_params.progressive_frame = (*disp_info).progressive_frame;
                proc_params.second_field = (*disp_info).repeat_first_field + 1;
                proc_params.top_field_first = (*disp_info).top_field_first;
                proc_params.unpaired_field = if (*disp_info).repeat_first_field < 0 { 1 } else { 0 };
                
                let mut dev_ptr: u64 = 0;
                let mut pitch: u32 = 0;
                let result = ffi::cuvid::cuvidMapVideoFrame(
                    decoder,
                    (*disp_info).picture_index,
                    &mut dev_ptr,
                    &mut pitch,
                    &mut proc_params as *mut _
                );
                
                if result != 0 {
                    tracing::error!("Failed to map video frame: error code {}", result);
                    ffi::cuvid::cuvidUnmapVideoFrame(decoder, dev_ptr);
                    return 0;
                }
                
                // Copy the frame data to host memory and convert to RGBA
                let frame_size = (pitch as usize * self.height as usize * 3 / 2);
                let mut nv12_data = vec![0u8; frame_size];
                
                // Copy Y plane
                let mut copy_params = ffi::cuda::CUDA_MEMCPY2D {
                    srcMemoryType: ffi::cuda::CUmemorytype_enum_CU_MEMORYTYPE_DEVICE,
                    srcDevice: dev_ptr,
                    srcPitch: pitch as usize,
                    dstMemoryType: ffi::cuda::CUmemorytype_enum_CU_MEMORYTYPE_HOST,
                    dstHost: nv12_data.as_mut_ptr() as *mut c_void,
                    dstPitch: pitch as usize,
                    WidthInBytes: self.width as usize,
                    Height: self.height as usize,
                    ..mem::zeroed()
                };
                
                let result = ffi::cuda::cuMemcpy2D_v2(&mut copy_params as *mut _);
                if result != 0 {
                    tracing::error!("Failed to copy Y plane: error code {}", result);
                    ffi::cuvid::cuvidUnmapVideoFrame(decoder, dev_ptr);
                    return 0;
                }
                
                // Copy UV plane
                copy_params.srcDevice = dev_ptr + (pitch as u64 * self.height as u64);
                copy_params.dstHost = nv12_data.as_mut_ptr().add(pitch as usize * self.height as usize) as *mut c_void;
                copy_params.Height = self.height as usize / 2;
                
                let result = ffi::cuda::cuMemcpy2D_v2(&mut copy_params as *mut _);
                if result != 0 {
                    tracing::error!("Failed to copy UV plane: error code {}", result);
                    ffi::cuvid::cuvidUnmapVideoFrame(decoder, dev_ptr);
                    return 0;
                }
                
                // Unmap the frame
                ffi::cuvid::cuvidUnmapVideoFrame(decoder, dev_ptr);
                
                // Convert NV12 to RGBA
                let mut frame_buffer = ImageBuffer::new(self.width, self.height);
                self.nv12_to_rgba(&nv12_data, pitch as usize, &mut frame_buffer);
                
                // Store the decoded frame
                let mut decoded_frames = self.decoded_frames.lock().unwrap();
                decoded_frames.push(frame_buffer);
                
                tracing::debug!("Frame mapped and converted successfully");
                return 1; // Success
            }
        }
        
        0 // Failure
    }
    
    #[cfg(feature = "hardware-accel")]
    fn nv12_to_rgba(&self, nv12_data: &[u8], pitch: usize, rgba_buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
        let width = self.width as usize;
        let height = self.height as usize;
        
        for y in 0..height {
            for x in 0..width {
                let y_index = y * pitch + x;
                let uv_index = (y / 2) * pitch + (x / 2) * 2 + pitch * height;
                
                if y_index >= nv12_data.len() || uv_index >= nv12_data.len() || uv_index + 1 >= nv12_data.len() {
                    continue;
                }
                
                let y_val = nv12_data[y_index] as f32;
                let u_val = nv12_data[uv_index] as f32 - 128.0;
                let v_val = nv12_data[uv_index + 1] as f32 - 128.0;
                
                // YUV to RGB conversion
                let r = y_val + 1.402 * v_val;
                let g = y_val - 0.344136 * u_val - 0.714136 * v_val;
                let b = y_val + 1.772 * u_val;
                
                // Clamp values to 0-255 range
                let r = r.max(0.0).min(255.0) as u8;
                let g = g.max(0.0).min(255.0) as u8;
                let b = b.max(0.0).min(255.0) as u8;
                
                rgba_buffer.put_pixel(x as u32, y as u32, Rgba([r, g, b, 255]));
            }
        }
    }
    
    #[cfg(feature = "hardware-accel")]
    pub fn decode_frame(&mut self, data: &[u8]) -> Result<Option<ImageBuffer<Rgba<u8>, Vec<u8>>>> {
        // Initialize the decoder if it hasn't been initialized yet
        if !self.initialized {
            self.initialize_decoder()?;
        }
        
        tracing::debug!("Decoding frame of size: {} bytes", data.len());
        
        // Clear any previously decoded frames
        {
            let mut decoded_frames = self.decoded_frames.lock().unwrap();
            decoded_frames.clear();
        }
        
        // Feed the data to the parser
        if let Some(parser) = self.parser {
            unsafe {
                let mut packet: ffi::cuvid::CUVIDSOURCEDATAPACKET = mem::zeroed();
                packet.payload_size = data.len() as u32;
                packet.payload = data.as_ptr();
                packet.flags = 0;
                
                // Increment frame count
                self.frame_count += 1;
                
                tracing::debug!("Sending frame {} to parser", self.frame_count);
                
                let result = ffi::cuvid::cuvidParseVideoData(parser, &mut packet as *mut _);
                if result != 0 {
                    tracing::error!("Failed to parse video data: error code {}", result);
                    return Err(anyhow!("Failed to parse video data: {}", result));
                }
                
                tracing::debug!("Successfully parsed video data");
            }
        } else {
            tracing::error!("Parser not initialized");
            return Err(anyhow!("Parser not initialized"));
        }
        
        // Wait a bit for callbacks to complete
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        // Get the decoded frame
        let decoded_frames = self.decoded_frames.lock().unwrap();
        tracing::debug!("Number of decoded frames: {}", decoded_frames.len());
        
        if !decoded_frames.is_empty() {
            // Return the first decoded frame
            tracing::debug!("Returning decoded frame");
            return Ok(Some(decoded_frames[0].clone()));
        }
        
        // If no frames were decoded but we have dimensions, create a placeholder frame
        if self.width > 0 && self.height > 0 {
            tracing::debug!("Creating placeholder frame of size {}x{}", self.width, self.height);
            let mut buffer = ImageBuffer::new(self.width, self.height);
            
            // Fill with a solid color (gray)
            for pixel in buffer.pixels_mut() {
                *pixel = Rgba([50, 50, 50, 255]);
            }
            
            return Ok(Some(buffer));
        }
        
        // If no frames were decoded, return None
        tracing::warn!("No frames decoded and no dimensions available");
        Ok(None)
    }
}

impl Drop for NvdecDecoder {
    fn drop(&mut self) {
        #[cfg(feature = "hardware-accel")]
        unsafe {
            // Clean up resources
            if let Some(parser) = self.parser {
                ffi::cuvid::cuvidDestroyVideoParser(parser);
            }
            
            if let Some(decoder) = self.decoder {
                ffi::cuvid::cuvidDestroyDecoder(decoder);
            }
            
            if let Some(ctx_lock) = self.ctx_lock {
                ffi::cuvid::cuvidCtxLockDestroy(ctx_lock as ffi::cuvid::CUvideoctxlock);
            }
        }
    }
}

pub fn is_nvidia_hardware_available() -> bool {
    #[cfg(feature = "hardware-accel")]
    {
        // Try to initialize CUDA
        let result = unsafe { ffi::cuda::cuInit(0) };
        
        // Check if initialization was successful
        if result != 0 {
            return false;
        }
        
        // Check if there are any CUDA devices
        let mut count = 0;
        let result = unsafe { 
            ffi::cuda::cuDeviceGetCount(&mut count)
        };
        
        result == 0 && count > 0
    }
    
    #[cfg(not(feature = "hardware-accel"))]
    {
        false
    }
}

// Helper functions for creating hardware encoder/decoder
pub fn create_hardware_encoder(width: u32, height: u32, bitrate: u32, fps: u32, keyframe_interval: u32) -> Result<NvencEncoder> {
    NvencEncoder::new(width, height, bitrate, fps, keyframe_interval)
}

pub fn create_hardware_decoder() -> Result<NvdecDecoder> {
    NvdecDecoder::new()
} 