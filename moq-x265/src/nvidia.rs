// NVIDIA Video Codec SDK bindings and implementations
use anyhow::{anyhow, Result};
use image::{ImageBuffer, Rgba};
use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_char, c_int, c_uint, c_void, c_ulong, c_ulonglong};
use std::ptr;
use std::time::Duration;
use std::sync::Once;
use lazy_static::lazy_static;
use std::sync::Mutex;

#[cfg(feature = "hardware-accel")]
use nvidia_video_codec::{
    cuda::{device::CuDevice, context::CuContext},
    nvenc::{NvEncoder, EncoderConfig, EncoderPreset, EncodeProfile, EncodeFormat, EncodeBuffer},
    cuvid::{CuvidDecoder, DecoderConfig, DecoderFormat}
};

// FFI bindings for NVIDIA Video Codec SDK
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod ffi {
    use std::os::raw::{c_char, c_int, c_uint, c_void, c_ulong, c_ulonglong};
    
    // Common types
    pub type NVENCSTATUS = c_int;
    pub type CUdeviceptr = c_ulonglong;
    pub type CUcontext = *mut c_void;
    pub type CUresult = c_int;
    
    // NVENC constants
    pub const NV_ENC_SUCCESS: NVENCSTATUS = 0;
    pub const NV_ENC_ERR_NO_ENCODE_DEVICE: NVENCSTATUS = -1;
    pub const NV_ENC_ERR_UNSUPPORTED_DEVICE: NVENCSTATUS = -2;
    pub const NV_ENC_ERR_INVALID_ENCODERDEVICE: NVENCSTATUS = -3;
    pub const NV_ENC_ERR_DEVICE_NOT_EXIST: NVENCSTATUS = -4;
    
    // CUDA constants
    pub const CUDA_SUCCESS: CUresult = 0;
    
    // NVENC structures
    #[repr(C)]
    pub struct NV_ENCODE_API_FUNCTION_LIST {
        pub version: c_uint,
        pub reserved: c_uint,
        pub nvEncOpenEncodeSession: extern "C" fn(
            device: *mut c_void,
            deviceType: c_uint,
            encoder: *mut *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncGetEncodeGUIDCount: extern "C" fn(
            encoder: *mut c_void,
            guidCount: *mut c_uint,
        ) -> NVENCSTATUS,
        pub nvEncGetEncodeGUIDs: extern "C" fn(
            encoder: *mut c_void,
            guids: *mut c_void,
            guidArraySize: c_uint,
            guidCount: *mut c_uint,
        ) -> NVENCSTATUS,
        pub nvEncGetEncodeProfileGUIDCount: extern "C" fn(
            encoder: *mut c_void,
            encodeGUID: c_void,
            guidCount: *mut c_uint,
        ) -> NVENCSTATUS,
        pub nvEncGetEncodeProfileGUIDs: extern "C" fn(
            encoder: *mut c_void,
            encodeGUID: c_void,
            profileGUIDs: *mut c_void,
            guidArraySize: c_uint,
            guidCount: *mut c_uint,
        ) -> NVENCSTATUS,
        pub nvEncGetInputFormats: extern "C" fn(
            encoder: *mut c_void,
            encodeGUID: c_void,
            inputFmts: *mut c_uint,
            inputFmtArraySize: c_uint,
            inputFmtCount: *mut c_uint,
        ) -> NVENCSTATUS,
        pub nvEncGetEncodePresetCount: extern "C" fn(
            encoder: *mut c_void,
            encodeGUID: c_void,
            presetCount: *mut c_uint,
        ) -> NVENCSTATUS,
        pub nvEncGetEncodePresetGUIDs: extern "C" fn(
            encoder: *mut c_void,
            encodeGUID: c_void,
            presetGUIDs: *mut c_void,
            guidArraySize: c_uint,
            guidCount: *mut c_uint,
        ) -> NVENCSTATUS,
        pub nvEncGetEncodePresetConfig: extern "C" fn(
            encoder: *mut c_void,
            encodeGUID: c_void,
            presetGUID: c_void,
            presetConfig: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncGetEncodeCapabilities: extern "C" fn(
            encoder: *mut c_void,
            encodeGUID: c_void,
            capsParam: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncGetInputFormatCaps: extern "C" fn(
            encoder: *mut c_void,
            encodeGUID: c_void,
            inputFmt: c_uint,
            capsParam: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncGetEncodeCaps: extern "C" fn(
            encoder: *mut c_void,
            encodeGUID: c_void,
            capsParam: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncInitializeEncoder: extern "C" fn(
            encoder: *mut c_void,
            createEncodeParams: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncCreateInputBuffer: extern "C" fn(
            encoder: *mut c_void,
            createInputBufferParams: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncDestroyInputBuffer: extern "C" fn(
            encoder: *mut c_void,
            inputBuffer: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncCreateBitstreamBuffer: extern "C" fn(
            encoder: *mut c_void,
            createBitstreamBufferParams: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncDestroyBitstreamBuffer: extern "C" fn(
            encoder: *mut c_void,
            bitstreamBuffer: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncEncodePicture: extern "C" fn(
            encoder: *mut c_void,
            encodePicParams: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncLockBitstream: extern "C" fn(
            encoder: *mut c_void,
            lockBitstreamBufferParams: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncUnlockBitstream: extern "C" fn(
            encoder: *mut c_void,
            bitstreamBuffer: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncLockInputBuffer: extern "C" fn(
            encoder: *mut c_void,
            lockInputBufferParams: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncUnlockInputBuffer: extern "C" fn(
            encoder: *mut c_void,
            inputBuffer: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncGetEncodeStats: extern "C" fn(
            encoder: *mut c_void,
            encodeStats: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncGetSequenceParams: extern "C" fn(
            encoder: *mut c_void,
            sequenceParamPayload: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncRegisterAsyncEvent: extern "C" fn(
            encoder: *mut c_void,
            eventParams: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncUnregisterAsyncEvent: extern "C" fn(
            encoder: *mut c_void,
            eventParams: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncMapInputResource: extern "C" fn(
            encoder: *mut c_void,
            mapInputResParams: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncUnmapInputResource: extern "C" fn(
            encoder: *mut c_void,
            mappedInputBuffer: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncDestroyEncoder: extern "C" fn(
            encoder: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncInvalidateRefFrames: extern "C" fn(
            encoder: *mut c_void,
            invalidRefFrameTimeStamp: c_ulonglong,
        ) -> NVENCSTATUS,
        pub nvEncOpenEncodeSessionEx: extern "C" fn(
            openSessionExParams: *mut c_void,
            encoder: *mut *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncRegisterResource: extern "C" fn(
            encoder: *mut c_void,
            registerResParams: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncUnregisterResource: extern "C" fn(
            encoder: *mut c_void,
            registeredRes: *mut c_void,
        ) -> NVENCSTATUS,
        pub nvEncReconfigureEncoder: extern "C" fn(
            encoder: *mut c_void,
            reInitEncodeParams: *mut c_void,
        ) -> NVENCSTATUS,
    }
    
    // NVDEC constants
    pub const CUVIDDECODECREATEINFO_BITFIELD_ARRAY_SIZE: usize = 4;
    
    // NVDEC structures
    #[repr(C)]
    pub struct CUVIDDECODECREATEINFO {
        pub ulWidth: c_uint,
        pub ulHeight: c_uint,
        pub ulNumDecodeSurfaces: c_uint,
        pub CodecType: c_uint,
        pub ChromaFormat: c_uint,
        pub ulCreationFlags: c_uint,
        pub Reserved1: [c_uint; CUVIDDECODECREATEINFO_BITFIELD_ARRAY_SIZE],
        pub display_area: CUVIDDECODECREATEINFO_display_area,
        pub OutputFormat: c_uint,
        pub DeinterlaceMode: c_uint,
        pub ulTargetWidth: c_uint,
        pub ulTargetHeight: c_uint,
        pub ulNumOutputSurfaces: c_uint,
        pub vidLock: *mut c_void,
        pub target_rect: CUVIDDECODECREATEINFO_target_rect,
        pub Reserved2: [c_uint; CUVIDDECODECREATEINFO_BITFIELD_ARRAY_SIZE],
    }
    
    #[repr(C)]
    pub struct CUVIDDECODECREATEINFO_display_area {
        pub left: c_int,
        pub top: c_int,
        pub right: c_int,
        pub bottom: c_int,
    }
    
    #[repr(C)]
    pub struct CUVIDDECODECREATEINFO_target_rect {
        pub left: c_int,
        pub top: c_int,
        pub right: c_int,
        pub bottom: c_int,
    }
    
    #[repr(C)]
    pub struct CUVIDPICPARAMS {
        pub PicWidthInMbs: c_int,
        pub FrameHeightInMbs: c_int,
        pub CurrPicIdx: c_int,
        pub field_pic_flag: c_int,
        pub bottom_field_flag: c_int,
        pub second_field: c_int,
        pub nBitstreamDataLen: c_uint,
        pub pBitstreamData: *const c_uchar,
        pub nNumSlices: c_uint,
        pub pSliceDataOffsets: *const c_uint,
    }
    
    pub type c_uchar = u8;
    
    // External function declarations
    extern "C" {
        // NVENC functions
        pub fn NvEncodeAPICreateInstance(
            functionList: *mut NV_ENCODE_API_FUNCTION_LIST,
        ) -> NVENCSTATUS;
        
        // CUDA functions
        pub fn cuInit(flags: c_uint) -> CUresult;
        pub fn cuDeviceGet(device: *mut CUdevice, ordinal: c_int) -> CUresult;
        pub fn cuCtxCreate_v2(pctx: *mut CUcontext, flags: c_uint, dev: CUdevice) -> CUresult;
        pub fn cuCtxDestroy_v2(ctx: CUcontext) -> CUresult;
        pub fn cuMemAlloc_v2(dptr: *mut CUdeviceptr, bytesize: usize) -> CUresult;
        pub fn cuMemFree_v2(dptr: CUdeviceptr) -> CUresult;
        pub fn cuMemcpy(dst: CUdeviceptr, src: *const c_void, bytesize: usize) -> CUresult;
        
        // NVDEC functions
        pub fn cuvidCreateDecoder(
            decoder: *mut *mut c_void,
            createInfo: *const CUVIDDECODECREATEINFO,
        ) -> CUresult;
        
        pub fn cuvidDestroyDecoder(
            decoder: *mut c_void,
        ) -> CUresult;
        
        pub fn cuvidDecodePicture(
            decoder: *mut c_void,
            picParams: *const CUVIDPICPARAMS,
        ) -> CUresult;
        
        pub fn cuvidMapVideoFrame(
            decoder: *mut c_void,
            picIdx: c_int,
            mappedFrame: *mut CUdeviceptr,
            pPitch: *mut c_uint,
            pParams: *const c_void,
        ) -> CUresult;
        
        pub fn cuvidUnmapVideoFrame(
            decoder: *mut c_void,
            mappedFrame: CUdeviceptr,
        ) -> CUresult;
    }
    
    pub type CUdevice = c_int;
}

// Initialize CUDA and NVENC once
static INIT: Once = Once::new();
lazy_static! {
    static ref INIT_MUTEX: Mutex<bool> = Mutex::new(false);
}

// NVIDIA encoder implementation
pub struct NvencEncoder {
    #[cfg(feature = "hardware-accel")]
    encoder: NvEncoder,
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
        // Check if NVIDIA hardware is available
        if !is_nvidia_hardware_available() {
            return Err(anyhow!("NVIDIA hardware acceleration is not available"));
        }
        
        #[cfg(feature = "hardware-accel")]
        {
            // Initialize CUDA
            let device = CuDevice::new(0)?;
            let context = CuContext::new(device, 0)?;
            
            // Create encoder configuration
            let mut config = EncoderConfig::new(width, height);
            config.set_framerate(fps, 1);
            config.set_rate_control_mode(bitrate as i32, 0, 0);
            config.set_gop_length(keyframe_interval);
            
            // Create encoder
            let encoder = NvEncoder::new(
                &context,
                width,
                height,
                EncodeFormat::Hevc,
                EncodeProfile::Main,
                EncoderPreset::LowLatencyHp,
                config
            )?;
            
            Ok(NvencEncoder {
                encoder,
                context,
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
            Err(anyhow!("NVIDIA hardware acceleration is not enabled. Recompile with --features hardware-accel"))
        }
    }
    
    pub fn encode_frame(&mut self, frame: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<crate::EncodedFrame> {
        // Increment frame count
        self.frame_count += 1;
        
        // Determine if this is a keyframe
        let is_keyframe = self.frame_count % self.keyframe_interval as u64 == 1;
        
        #[cfg(feature = "hardware-accel")]
        {
            // Convert RGBA to NV12 format for NVENC
            let yuv_data = rgba_to_nv12(frame, self.width, self.height)?;
            
            // Create input buffer
            let mut input_buffer = EncodeBuffer::new_input_buffer(
                &self.encoder,
                &yuv_data,
                self.width,
                self.height
            )?;
            
            // Encode frame
            let output_buffer = self.encoder.encode(&mut input_buffer, is_keyframe)?;
            
            // Get encoded data
            let encoded_data = output_buffer.get_bitstream_data()?;
            
            Ok(crate::EncodedFrame {
                data: encoded_data,
                timestamp: Duration::from_millis((self.frame_count * 1000 / self.fps as u64) as u64),
                is_keyframe,
            })
        }
        
        #[cfg(not(feature = "hardware-accel"))]
        {
            Err(anyhow!("NVIDIA hardware acceleration is not enabled. Recompile with --features hardware-accel"))
        }
    }
}

// NVIDIA decoder implementation
pub struct NvdecDecoder {
    #[cfg(feature = "hardware-accel")]
    decoder: Option<CuvidDecoder>,
    #[cfg(feature = "hardware-accel")]
    context: CuContext,
    width: u32,
    height: u32,
    sps_data: Option<Vec<u8>>,
    pps_data: Option<Vec<u8>>,
    vps_data: Option<Vec<u8>>,
    initialized: bool,
}

unsafe impl Send for NvdecDecoder {}
unsafe impl Sync for NvdecDecoder {}

impl NvdecDecoder {
    pub fn new() -> Result<Self> {
        // Check if NVIDIA hardware is available
        if !is_nvidia_hardware_available() {
            return Err(anyhow!("NVIDIA hardware acceleration is not available"));
        }
        
        #[cfg(feature = "hardware-accel")]
        {
            // Initialize CUDA
            let device = CuDevice::new(0)?;
            let context = CuContext::new(device, 0)?;
            
            Ok(NvdecDecoder {
                decoder: None,
                context,
                width: 0,
                height: 0,
                sps_data: None,
                pps_data: None,
                vps_data: None,
                initialized: false,
            })
        }
        
        #[cfg(not(feature = "hardware-accel"))]
        {
            Err(anyhow!("NVIDIA hardware acceleration is not enabled. Recompile with --features hardware-accel"))
        }
    }
    
    pub fn decode_frame(&mut self, data: &[u8]) -> Result<Option<ImageBuffer<Rgba<u8>, Vec<u8>>>> {
        // Parse NAL units to extract resolution information
        let nal_units = crate::parse_nal_units(data);
        
        // Process NAL units to extract SPS, PPS, VPS data
        for nal_unit in nal_units {
            if nal_unit.len() > 0 {
                let nal_type = (nal_unit[0] >> 1) & 0x3F; // HEVC NAL type
                
                match nal_type {
                    32 => { // VPS
                        self.vps_data = Some(nal_unit.to_vec());
                        tracing::debug!("Found VPS NAL unit: {} bytes", nal_unit.len());
                    },
                    33 => { // SPS
                        self.sps_data = Some(nal_unit.to_vec());
                        tracing::debug!("Found SPS NAL unit: {} bytes", nal_unit.len());
                        
                        // Try to extract resolution from SPS
                        if let Some((w, h)) = crate::extract_resolution_from_sps(nal_unit) {
                            self.width = w;
                            self.height = h;
                            tracing::debug!("Extracted resolution from SPS: {}x{}", w, h);
                        }
                    },
                    34 => { // PPS
                        self.pps_data = Some(nal_unit.to_vec());
                        tracing::debug!("Found PPS NAL unit: {} bytes", nal_unit.len());
                    },
                    _ => {
                        tracing::trace!("Found NAL unit type {}: {} bytes", nal_type, nal_unit.len());
                    }
                }
            }
        }
        
        #[cfg(feature = "hardware-accel")]
        {
            // If we have resolution information, create a decoder if not already created
            if self.width > 0 && self.height > 0 && !self.initialized {
                // Create decoder configuration
                let config = DecoderConfig::new(self.width, self.height, DecoderFormat::Hevc);
                
                // Create decoder
                self.decoder = Some(CuvidDecoder::new(&self.context, config)?);
                self.initialized = true;
                
                tracing::debug!("Created NVDEC decoder for {}x{}", self.width, self.height);
            }
            
            // If decoder is initialized, decode the frame
            if self.initialized {
                if let Some(decoder) = &mut self.decoder {
                    // Decode frame
                    let frame = decoder.decode(data)?;
                    
                    if let Some(frame) = frame {
                        // Convert to RGBA
                        let mut img = ImageBuffer::new(self.width, self.height);
                        
                        // Get frame data
                        let frame_data = frame.get_frame_data()?;
                        
                        // Convert NV12 to RGBA
                        nv12_to_rgba(&frame_data, &mut img, self.width, self.height)?;
                        
                        return Ok(Some(img));
                    }
                }
            }
        }
        
        #[cfg(not(feature = "hardware-accel"))]
        {
            return Err(anyhow!("NVIDIA hardware acceleration is not enabled. Recompile with --features hardware-accel"));
        }
        
        Ok(None)
    }
}

// Helper functions
fn is_nvidia_hardware_available() -> bool {
    #[cfg(feature = "hardware-accel")]
    {
        // Try to initialize CUDA
        match CuDevice::new(0) {
            Ok(_) => {
                tracing::debug!("NVIDIA hardware is available");
                true
            },
            Err(e) => {
                tracing::debug!("NVIDIA hardware is not available: {}", e);
                false
            }
        }
    }
    
    #[cfg(not(feature = "hardware-accel"))]
    {
        false
    }
}

fn rgba_to_nv12(frame: &ImageBuffer<Rgba<u8>, Vec<u8>>, width: u32, height: u32) -> Result<Vec<u8>> {
    let width = width as usize;
    let height = height as usize;
    
    // Calculate plane sizes for NV12 format
    let y_size = width * height;
    let uv_size = width * height / 2;
    
    // Allocate buffer for NV12 data
    let mut nv12_data = vec![0u8; y_size + uv_size];
    
    // Split the buffer into Y and UV planes
    let (y_plane, uv_plane) = nv12_data.split_at_mut(y_size);
    
    // Convert RGBA to NV12
    for y in 0..height {
        for x in 0..width {
            let rgba = frame.get_pixel(x as u32, y as u32);
            
            // Convert RGB to Y
            let y_value = (0.299 * rgba[0] as f32 + 0.587 * rgba[1] as f32 + 0.114 * rgba[2] as f32) as u8;
            
            // Store Y
            y_plane[y * width + x] = y_value;
            
            // Downsample and convert to UV (4:2:0)
            if y % 2 == 0 && x % 2 == 0 {
                let u_value = (128.0 - 0.168736 * rgba[0] as f32 - 0.331264 * rgba[1] as f32 + 0.5 * rgba[2] as f32) as u8;
                let v_value = (128.0 + 0.5 * rgba[0] as f32 - 0.418688 * rgba[1] as f32 - 0.081312 * rgba[2] as f32) as u8;
                
                let uv_index = (y / 2) * width + x;
                if uv_index + 1 < uv_plane.len() {
                    uv_plane[uv_index] = u_value;
                    uv_plane[uv_index + 1] = v_value;
                }
            }
        }
    }
    
    Ok(nv12_data)
}

fn nv12_to_rgba(nv12_data: &[u8], rgba_image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, width: u32, height: u32) -> Result<()> {
    let width = width as usize;
    let height = height as usize;
    
    // Calculate plane sizes for NV12 format
    let y_size = width * height;
    
    // Split the buffer into Y and UV planes
    let (y_plane, uv_plane) = nv12_data.split_at(y_size);
    
    // Convert NV12 to RGBA
    for y in 0..height {
        for x in 0..width {
            let y_value = y_plane[y * width + x] as f32;
            
            // Get UV values (4:2:0 format)
            let uv_index = (y / 2) * width + (x / 2) * 2;
            let u_value = if uv_index < uv_plane.len() { uv_plane[uv_index] as f32 } else { 128.0 };
            let v_value = if uv_index + 1 < uv_plane.len() { uv_plane[uv_index + 1] as f32 } else { 128.0 };
            
            // Convert YUV to RGB
            let c = y_value - 16.0;
            let d = u_value - 128.0;
            let e = v_value - 128.0;
            
            let r = (298.0 * c + 409.0 * e + 128.0) / 256.0;
            let g = (298.0 * c - 100.0 * d - 208.0 * e + 128.0) / 256.0;
            let b = (298.0 * c + 516.0 * d + 128.0) / 256.0;
            
            // Clamp values to 0-255 range
            let r = r.max(0.0).min(255.0) as u8;
            let g = g.max(0.0).min(255.0) as u8;
            let b = b.max(0.0).min(255.0) as u8;
            
            // Set RGBA pixel
            *rgba_image.get_pixel_mut(x as u32, y as u32) = Rgba([r, g, b, 255]);
        }
    }
    
    Ok(())
}

// Public API for hardware acceleration
pub fn create_hardware_encoder(width: u32, height: u32, bitrate: u32, fps: u32, keyframe_interval: u32) -> Result<NvencEncoder> {
    NvencEncoder::new(width, height, bitrate, fps, keyframe_interval)
}

pub fn create_hardware_decoder() -> Result<NvdecDecoder> {
    NvdecDecoder::new()
} 