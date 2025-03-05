macro_rules! wrap {
    ($val:ident, $res:ident) => (
        if $res == CUDA_SUCCESS {
            Ok($val)
        } else {
            Err($res)
        }
    )
}

// Simplified macros for error handling
#[macro_export]
macro_rules! cuda_check {
    ($expr:expr) => {
        {
            let result = unsafe { $expr };
            if result != 0 {
                return Err(anyhow::anyhow!("CUDA error: {}", result));
            }
            result
        }
    };
}


