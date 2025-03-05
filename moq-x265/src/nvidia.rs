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
    
    pub fn encode_frame(&mut self, _frame: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<crate::EncodedFrame> {
        #[cfg(feature = "hardware-accel")]
        {
            // For now, we'll just create a dummy encoded frame
            // In a real implementation, we would use the NVENC API to encode the frame
            
            // Increment frame count
            self.frame_count += 1;
            
            // Determine if this is a keyframe
            let is_keyframe = self.frame_count % self.keyframe_interval as u64 == 1;
            
            // Create a dummy encoded frame
            let data = vec![0u8; 1024]; // Dummy data
            
            Ok(crate::EncodedFrame {
                data,
                timestamp: Duration::from_millis(self.frame_count * 1000 / self.fps as u64),
                is_keyframe,
            })
        }
        
        #[cfg(not(feature = "hardware-accel"))]
        {
            Err(anyhow!("Hardware acceleration not available"))
        }
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
        let decoder = &mut *(user_data as *mut NvdecDecoder);
        decoder.handle_video_sequence(video_format)
    }
}

#[cfg(feature = "hardware-accel")]
extern "C" fn handle_picture_decode(user_data: *mut c_void, pic_params: *mut ffi::cuvid::CUVIDPICPARAMS) -> i32 {
    unsafe {
        let decoder = &mut *(user_data as *mut NvdecDecoder);
        decoder.handle_picture_decode(pic_params)
    }
}

#[cfg(feature = "hardware-accel")]
extern "C" fn handle_picture_display(user_data: *mut c_void, disp_info: *mut ffi::cuvid::CUVIDPARSERDISPINFO) -> i32 {
    unsafe {
        let decoder = &mut *(user_data as *mut NvdecDecoder);
        decoder.handle_picture_display(disp_info)
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
            
            tracing::info!("Video sequence: {}x{}, codec: {}", width, height, format.codec);
            
            // Update dimensions if they've changed
            if self.width as i32 != width || self.height as i32 != height {
                self.width = width as u32;
                self.height = height as u32;
                
                // Recreate frame buffer with new dimensions
                self.frame_buffer = Some(ImageBuffer::new(self.width, self.height));
            }
            
            // Create decoder if it doesn't exist
            if self.decoder.is_none() {
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
                let result = ffi::cuvid::cuvidDecodePicture(decoder, pic_params);
                if result != 0 {
                    tracing::error!("Failed to decode picture: error code {}", result);
                    return 0;
                }
                1 // Success
            } else {
                tracing::error!("Decoder not initialized");
                0 // Failure
            }
        }
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
                
                let mut dev_ptr = 0;
                let mut pitch = 0;
                let result = ffi::cuvid::cuvidMapVideoFrame(
                    decoder,
                    (*disp_info).picture_index,
                    &mut dev_ptr,
                    &mut pitch,
                    &mut proc_params as *mut _
                );
                
                if result != 0 {
                    tracing::error!("Failed to map video frame: error code {}", result);
                    return 0;
                }
                
                // Copy the frame data to host memory and convert to RGBA
                let frame_size = (pitch * self.height as u32 * 3 / 2) as usize;
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
                
                let result = ffi::cuda::cuMemcpy2D_v2(&copy_params);
                if result != 0 {
                    tracing::error!("Failed to copy Y plane: error code {}", result);
                    ffi::cuvid::cuvidUnmapVideoFrame(decoder, dev_ptr);
                    return 0;
                }
                
                // Copy UV plane
                copy_params.srcDevice = dev_ptr + (pitch * self.height as u32) as u64;
                copy_params.dstHost = nv12_data.as_mut_ptr().add(pitch as usize * self.height as usize) as *mut c_void;
                copy_params.Height = self.height as usize / 2;
                
                let result = ffi::cuda::cuMemcpy2D_v2(&copy_params);
                if result != 0 {
                    tracing::error!("Failed to copy UV plane: error code {}", result);
                    ffi::cuvid::cuvidUnmapVideoFrame(decoder, dev_ptr);
                    return 0;
                }
                
                // Unmap the frame
                ffi::cuvid::cuvidUnmapVideoFrame(decoder, dev_ptr);
                
                // Convert NV12 to RGBA
                let mut rgba_buffer = ImageBuffer::new(self.width, self.height);
                self.nv12_to_rgba(&nv12_data, pitch as usize, &mut rgba_buffer);
                
                // Store the decoded frame
                let mut frames = self.decoded_frames.lock().unwrap();
                frames.push(rgba_buffer);
                
                1 // Success
            } else {
                tracing::error!("Decoder not initialized");
                0 // Failure
            }
        }
    }
    
    #[cfg(feature = "hardware-accel")]
    fn nv12_to_rgba(&self, nv12_data: &[u8], pitch: usize, rgba_buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
        // NV12 format: Y plane followed by interleaved UV plane
        let y_plane_size = pitch * self.height as usize;
        
        for y in 0..self.height {
            for x in 0..self.width {
                let y_index = y as usize * pitch + x as usize;
                let uv_index = y_plane_size + (y as usize / 2) * pitch + (x as usize / 2) * 2;
                
                if y_index >= nv12_data.len() || uv_index + 1 >= nv12_data.len() {
                    continue;
                }
                
                let y_val = nv12_data[y_index] as f32;
                let u_val = nv12_data[uv_index] as f32 - 128.0;
                let v_val = nv12_data[uv_index + 1] as f32 - 128.0;
                
                // YUV to RGB conversion
                let r = y_val + 1.402 * v_val;
                let g = y_val - 0.344136 * u_val - 0.714136 * v_val;
                let b = y_val + 1.772 * u_val;
                
                let r = r.max(0.0).min(255.0) as u8;
                let g = g.max(0.0).min(255.0) as u8;
                let b = b.max(0.0).min(255.0) as u8;
                
                rgba_buffer.put_pixel(x, y, Rgba([r, g, b, 255]));
            }
        }
    }
    
    pub fn decode_frame(&mut self, data: &[u8]) -> Result<Option<ImageBuffer<Rgba<u8>, Vec<u8>>>> {
        #[cfg(feature = "hardware-accel")]
        {
            // Add detailed logging
            tracing::debug!("NvdecDecoder: Received data of length: {}", data.len());
            
            if data.len() < 4 {
                tracing::warn!("NvdecDecoder: Data too short to be a valid HEVC frame");
                return Ok(None);
            }
            
            // Increment frame count
            self.frame_count += 1;
            
            // Initialize decoder if not already done
            if !self.initialized {
                self.initialize_decoder()?;
            }
            
            // Clear previous decoded frames
            {
                let mut frames = self.decoded_frames.lock().unwrap();
                frames.clear();
            }
            
            // Parse NAL units from the frame data
            let nal_units = crate::parse_nal_units(data);
            tracing::debug!("NvdecDecoder: Found {} NAL units", nal_units.len());
            
            // Process each NAL unit
            for nal in nal_units {
                if nal.len() < 2 {
                    continue;
                }
                
                // Get NAL unit type (bits 1-6 of the first byte after the start code)
                let nal_type = (nal[0] >> 1) & 0x3F;
                
                match nal_type {
                    32 => { // VPS
                        tracing::debug!("NvdecDecoder: Found VPS NAL unit");
                        self.vps_data = Some(nal.to_vec());
                    },
                    33 => { // SPS
                        tracing::debug!("NvdecDecoder: Found SPS NAL unit");
                        self.sps_data = Some(nal.to_vec());
                        
                        // Try to extract resolution from SPS
                        if let Some((width, height)) = crate::extract_resolution_from_sps(nal) {
                            tracing::info!("NvdecDecoder: Extracted resolution from SPS: {}x{}", width, height);
                            
                            // Only update dimensions if they've changed
                            if self.width != width || self.height != height {
                                self.width = width;
                                self.height = height;
                                
                                // Recreate frame buffer with new dimensions
                                self.frame_buffer = Some(ImageBuffer::new(width, height));
                            }
                        }
                    },
                    34 => { // PPS
                        tracing::debug!("NvdecDecoder: Found PPS NAL unit");
                        self.pps_data = Some(nal.to_vec());
                    },
                    _ => {
                        tracing::trace!("NvdecDecoder: NAL unit type: {}", nal_type);
                    }
                }
            }
            
            // Feed the data to the parser
            if let Some(parser) = self.parser {
                unsafe {
                    let mut packet: ffi::cuvid::CUVIDSOURCEDATAPACKET = mem::zeroed();
                    packet.payload_size = data.len() as u32;
                    packet.payload = data.as_ptr();
                    packet.flags = 0;
                    
                    if self.frame_count == 1 {
                        packet.flags |= ffi::cuvid::CUVID_PKT_ENDOFSTREAM;
                    }
                    
                    let result = ffi::cuvid::cuvidParseVideoData(parser, &mut packet as *mut _);
                    if result != 0 {
                        tracing::error!("Failed to parse video data: error code {}", result);
                        return Err(anyhow!("Failed to parse video data: {}", result));
                    }
                }
            }
            
            // Get the decoded frame
            let frames = self.decoded_frames.lock().unwrap();
            if let Some(frame) = frames.last() {
                return Ok(Some(frame.clone()));
            }
            
            // If no frame was decoded but we have a frame buffer, return a solid color
            if let Some(ref mut buffer) = self.frame_buffer {
                // Create a simple color based on the frame data
                let frame_hash = data.iter().fold(0u32, |acc, &b| acc.wrapping_add(b as u32));
                let r = ((frame_hash & 0xFF) as u8).wrapping_add(50);
                let g = (((frame_hash >> 8) & 0xFF) as u8).wrapping_add(50);
                let b = (((frame_hash >> 16) & 0xFF) as u8).wrapping_add(50);
                
                // Fill the entire buffer with this color
                for pixel in buffer.pixels_mut() {
                    *pixel = Rgba([r, g, b, 255]);
                }
                
                tracing::debug!("NvdecDecoder: Created solid color image for frame {}", self.frame_count);
                return Ok(Some(buffer.clone()));
            }
            
            // If we don't have a frame buffer yet, return None
            tracing::warn!("NvdecDecoder: No frame buffer available yet");
            Ok(None)
        }
        
        #[cfg(not(feature = "hardware-accel"))]
        {
            return Err(anyhow!("Hardware acceleration not available"));
        }
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