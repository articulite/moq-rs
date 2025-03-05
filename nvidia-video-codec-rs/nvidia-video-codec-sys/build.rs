extern crate bindgen;

use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

fn format_write(builder: bindgen::Builder, output: &str) {
    let s = builder.generate()
        .unwrap()
        .to_string()
        .replace("/**", "/*")
        .replace("/*!", "/*");

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(output)
        .unwrap();

    let _ = file.write(s.as_bytes());
}

fn common_builder() -> bindgen::Builder {
    bindgen::builder()
        .raw_line("#![allow(dead_code)]")
        .raw_line("#![allow(non_camel_case_types)]")
        .raw_line("#![allow(non_snake_case)]")
        .raw_line("#![allow(non_upper_case_globals)]")
}

fn find_dir(default: &'static str, env_key: &'static str) -> PathBuf {
    match env::var_os(env_key) {
        Some(val) => PathBuf::from(&val),
        _ => PathBuf::from(default),
    }
}

fn main() {
    // For Windows, we'll use the NVIDIA Video Codec SDK headers directly
    // Get the SDK path from environment variable or use the default
    let sdk_path = env::var("NVIDIA_VIDEO_CODEC_SDK_DIR")
        .unwrap_or_else(|_| "C:\\gitprojects\\moq-rs\\moq-x265\\temp".to_string());
    
    let sdk_path = PathBuf::from(sdk_path);
    let interface_path = sdk_path.join("Interface");
    
    println!("cargo:warning=Using NVIDIA Video Codec SDK from: {}", sdk_path.display());
    println!("cargo:warning=Interface path: {}", interface_path.display());
    
    // Check if we're on Windows
    let is_windows = env::var("CARGO_CFG_TARGET_OS").map(|s| s == "windows").unwrap_or(false);
    
    // Set library names based on OS
    if is_windows {
        // On Windows, we need to link to the CUDA libraries
        println!("cargo:rustc-link-search=native={}\\Lib\\x64", sdk_path.display());
        
        // Add CUDA Toolkit path for Windows
        let cuda_toolkit_path = "C:\\Program Files\\NVIDIA GPU Computing Toolkit\\CUDA\\v12.8\\lib\\x64";
        println!("cargo:rustc-link-search=native={}", cuda_toolkit_path);
        println!("cargo:warning=Using CUDA Toolkit from: {}", cuda_toolkit_path);
        
        println!("cargo:rustc-link-lib=dylib=cuda");
        println!("cargo:rustc-link-lib=dylib=nvcuvid");
        println!("cargo:rustc-link-lib=dylib=nvencodeapi");
        println!("cargo:warning=Windows build: Linking to NVIDIA libraries from {}", sdk_path.display());
    } else {
        println!("cargo:rustc-link-lib=dylib={}", "cuda");
        println!("cargo:rustc-link-lib=dylib={}", "nvcuvid");
        println!("cargo:rustc-link-lib=dylib={}", "nvidia-encode");
    }

    // Create stub files for the bindings
    // We'll implement the necessary functionality in Rust directly
    
    // Create cuda.rs with minimal bindings
    let cuda_rs = r#"
    #![allow(dead_code)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(non_upper_case_globals)]
    
    use std::os::raw::{c_int, c_void, c_uint, c_ulonglong};
    
    pub type CUresult = c_int;
    pub type CUdevice = c_int;
    pub type CUcontext = *mut c_void;
    pub type CUdeviceptr = c_ulonglong;
    
    pub const CUDA_SUCCESS: CUresult = 0;
    "#;
    
    let mut cuda_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("src/cuda.rs")
        .unwrap();
    
    let _ = cuda_file.write(cuda_rs.as_bytes());
    
    // Check if the NVENC header exists
    let nvenc_header = interface_path.join("nvEncodeAPI.h");
    
    if !nvenc_header.exists() {
        panic!("NVENC header not found at: {}", nvenc_header.display());
    }
    
    println!("cargo:warning=Found NVENC header at: {}", nvenc_header.display());
    
    // Generate bindings for NVENC
    let nvenc_builder = common_builder()
        .clang_arg(format!("-I{}", interface_path.to_string_lossy()))
        .header(nvenc_header.to_string_lossy());
    
    format_write(nvenc_builder, "src/nvenc.rs");
    
    // Create cuvid.rs with minimal bindings
    let cuvid_rs = r#"
    #![allow(dead_code)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(non_upper_case_globals)]
    
    use std::os::raw::{c_int, c_void, c_uint, c_uchar};
    use crate::cuda::{CUcontext, CUdeviceptr};
    
    pub type CUvideodecoder = *mut c_void;
    pub type CUvideoparser = *mut c_void;
    
    pub const CUVIDDECODECREATEINFO_BITFIELD_ARRAY_SIZE: usize = 4;
    
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
    "#;
    
    let mut cuvid_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("src/cuvid.rs")
        .unwrap();
    
    let _ = cuvid_file.write(cuvid_rs.as_bytes());
    
    println!("cargo:warning=Generated minimal bindings for CUDA and CUVID");
}
