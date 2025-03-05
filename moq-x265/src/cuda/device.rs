use std::os::raw::c_int;
use anyhow::{anyhow, Result};

#[cfg(feature = "hardware-accel")]
use nvidia_video_codec_sys as ffi;

// Simple wrapper for CUDA device
pub struct CuDevice {
    device: c_int,
}

// Macro for handling CUDA errors
macro_rules! cuda_check {
    ($expr:expr) => {
        {
            let result = unsafe { $expr };
            if result != 0 {
                return Err(anyhow!("CUDA error: {}", result));
            }
            result
        }
    };
}

impl CuDevice {
    pub fn new(ordinal: i32) -> Result<Self> {
        let mut device = 0;
        cuda_check!(ffi::cuda::cuDeviceGet(&mut device, ordinal));
        Ok(Self { device })
    }
}

// Get the number of available CUDA devices
pub fn get_count() -> Result<i32> {
    let mut count = 0;
    cuda_check!(ffi::cuda::cuDeviceGetCount(&mut count));
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_enum() {
        // Initialize CUDA
        unsafe { ffi::cuda::cuInit(0) };
        
        // Get device count
        let count = get_count().unwrap();
        println!("Found {} CUDA device(s)", count);
        
        if count > 0 {
            // Create device
            let device = CuDevice::new(0).unwrap();
            assert!(device.device >= 0);
        }
    }
} 