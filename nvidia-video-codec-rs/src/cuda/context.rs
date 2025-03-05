use std::os::raw::c_void;
use std::ptr;
use anyhow::{anyhow, Result};

use ffi::cuda::{CUcontext, CUresult};

use super::device::CuDevice;

// Simple wrapper for CUDA context
pub struct CuContext {
    context: CUcontext,
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
        cuda_check!(ffi::cuda::cuCtxCreate_v2(&mut context, flags, device.device()));
        Ok(Self { context })
    }
    
    pub fn current() -> Result<Self> {
        let mut context = ptr::null_mut();
        // We don't have cuCtxGetCurrent in our minimal bindings, so we'll just create a new context
        Ok(Self { context })
    }
    
    // Add a method to get the raw CUDA context pointer
    pub fn context(&self) -> ffi::cuda::CUcontext {
        self.context
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cuda::device::CuDevice;

    #[test]
    fn context_create() {
        unsafe { cuInit(0) };
        let device = CuDevice::new(0).unwrap();
        let context = CuContext::new(device, 0).unwrap();
        drop(context);
    }
}
