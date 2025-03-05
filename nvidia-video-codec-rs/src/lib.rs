// Simplified NVIDIA Video Codec SDK bindings
pub extern crate nvidia_video_codec_sys as ffi;

// Modules
pub mod cuda;
pub mod macros;

// Re-exports
pub use cuda::device::CuDevice;
pub use cuda::context::CuContext;

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cuda_init() {
        unsafe {
            let result = ffi::cuda::cuInit(0);
            assert_eq!(result, 0);
            
            let mut count = 0;
            let result = ffi::cuda::cuDeviceGetCount(&mut count);
            assert_eq!(result, 0);
            
            println!("Found {} CUDA device(s)", count);
        }
    }
}
