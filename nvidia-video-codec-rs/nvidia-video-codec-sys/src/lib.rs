#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]

// Include the generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Re-export modules for better organization
pub mod cuda {
    pub use super::{
        CUresult, CUdevice, CUcontext, CUdeviceptr, CUmemorytype, CUDA_MEMCPY2D,
        cuInit, cuDeviceGet, cuDeviceGetCount, cuCtxCreate_v2, cuCtxDestroy_v2, cuMemcpy2D_v2,
    };
    
    // Export the memory type enum values as constants
    pub const CUmemorytype_enum_CU_MEMORYTYPE_HOST: CUmemorytype = 1;
    pub const CUmemorytype_enum_CU_MEMORYTYPE_DEVICE: CUmemorytype = 2;
}

pub mod cuvid {
    // Export CUVID types
    pub use super::{
        CUVIDEOFORMAT, CUVIDPICPARAMS, CUVIDPARSERDISPINFO, CUVIDPARSERPARAMS,
        CUVIDDECODECREATEINFO, CUVIDPROCPARAMS, CUVIDSOURCEDATAPACKET,
        CUvideodecoder, CUvideoparser, CUvideoctxlock,
    };
    
    // Export CUVID functions
    pub use super::{
        cuvidCreateDecoder, cuvidDestroyDecoder, cuvidCreateVideoParser, 
        cuvidDestroyVideoParser, cuvidDecodePicture, 
        cuvidCtxLockCreate, cuvidCtxLockDestroy, cuvidParseVideoData,
    };
    
    // Export CUVID constants
    pub use super::{
        cudaVideoCodec_HEVC, cudaVideoSurfaceFormat_NV12, 
        cudaVideoDeinterlaceMode_Weave,
    };
    
    // Define missing functions
    extern "C" {
        pub fn cuvidMapVideoFrame(
            decoder: CUvideodecoder,
            picture_index: i32,
            device_ptr: *mut u64,
            pitch: *mut u32,
            params: *mut CUVIDPROCPARAMS,
        ) -> i32;
        
        pub fn cuvidUnmapVideoFrame(
            decoder: CUvideodecoder,
            device_ptr: u64,
        ) -> i32;
    }
    
    // Define the missing constant
    pub const CUVID_PKT_ENDOFSTREAM: u32 = 0x01;
}

pub mod nvenc {
    // Re-export NVENC types and functions
    pub use super::{
        NV_ENC_INITIALIZE_PARAMS, NV_ENC_CONFIG,
        NvEncodeAPICreateInstance, NvEncodeAPIGetMaxSupportedVersion,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cuda_init() {
        unsafe {
            let result = cuda::cuInit(0);
            println!("CUDA init result: {}", result);
            
            let mut count = 0;
            let result = cuda::cuDeviceGetCount(&mut count);
            println!("CUDA device count: {}, result: {}", count, result);
        }
    }
}
