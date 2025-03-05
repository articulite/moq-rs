use std::os::raw::c_void;
use std::ptr;
use anyhow::{anyhow, Result};

#[cfg(feature = "hardware-accel")]
use nvidia_video_codec_sys as ffi;

use super::device::CuDevice;

// Simple wrapper for CUDA context
pub struct CuContext {
    context: *mut c_void,
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

impl CuContext {
    pub fn new(device: CuDevice, flags: u32) -> Result<Self> {
        let mut context = ptr::null_mut();
        cuda_check!(ffi::cuda::cuCtxCreate_v2(&mut context, flags, device.device));
        Ok(Self { context })
    }
}

impl Drop for CuContext {
    fn drop(&mut self) {
        if !self.context.is_null() {
            unsafe {
                ffi::cuda::cuCtxDestroy_v2(self.context);
            }
        }
    }
}

unsafe impl Send for CuContext {}
unsafe impl Sync for CuContext {} 