// Modern Rust supports unions with Copy and ManuallyDrop fields without requiring untagged_unions feature

use std::os::raw::{c_char, c_int, c_uint, c_void, c_ulong, c_ulonglong};

// CUDA module
pub mod cuda {
    use std::os::raw::{c_int, c_uint, c_void};
    
    pub type CUdevice = c_int;
    pub type CUresult = c_int;
    pub type CUcontext = *mut c_void;
    pub type CUdeviceptr = u64;
    
    // Basic CUDA functions
    extern "C" {
        pub fn cuInit(flags: c_uint) -> CUresult;
        pub fn cuDeviceGet(device: *mut CUdevice, ordinal: c_int) -> CUresult;
        pub fn cuDeviceGetCount(count: *mut c_int) -> CUresult;
        pub fn cuCtxCreate_v2(pctx: *mut CUcontext, flags: c_uint, dev: CUdevice) -> CUresult;
        pub fn cuCtxDestroy_v2(ctx: CUcontext) -> CUresult;
    }
}

// CUVID module (minimal)
pub mod cuvid {
    // Placeholder for CUVID functions
}

// NVENC module (minimal)
pub mod nvenc {
    // Placeholder for NVENC functions
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cuda_init() {
        unsafe {
            let result = cuda::cuInit(0);
            println!("CUDA init result: {}", result);
            
            let mut version = 0;
            let result = cuda::cuDeviceGetCount(&mut version);
            println!("CUDA device count: {} (result: {})", version, result);
        }
    }
}
