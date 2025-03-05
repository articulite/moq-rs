// Modern Rust supports unions with Copy and ManuallyDrop fields
// No need for the untagged_unions feature anymore

pub mod cuda;
pub mod cuvid;
pub mod nvenc;

#[cfg(test)]
mod tests {
    use super::cuda::*;
    #[test]
    fn init_and_version() {
        let ret = unsafe { cuInit(0) };
        println!("{:?}", ret);

        let ver = unsafe {
            let mut ver = 0;
            cuDriverGetVersion(&mut ver as *mut i32);
            ver
        };

        println!("Version {}", ver);
    }
}
