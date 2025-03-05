extern crate bindgen;

use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::fs::File;

fn format_write(builder: bindgen::Builder, output: &str) {
    let bindings = builder
        .generate()
        .unwrap_or_else(|_| panic!("Unable to generate bindings for {}", output));

    // Get the output directory
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // Create the output file
    let out_file = out_path.join(output);
    
    // Create parent directories if they don't exist
    if let Some(parent) = out_file.parent() {
        std::fs::create_dir_all(parent).unwrap_or_else(|_| panic!("Couldn't create directory for {}", output));
    }
    
    // Write the bindings to the file
    bindings
        .write_to_file(&out_file)
        .unwrap_or_else(|_| panic!("Couldn't write bindings for {}", output));
}

fn common_builder() -> bindgen::Builder {
    bindgen::Builder::default()
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++11")
        .derive_debug(true)
        .derive_default(true)
        .derive_copy(true)
        .derive_eq(true)
        .derive_hash(true)
        .derive_ord(true)
        .derive_partialeq(true)
        .derive_partialord(true)
        .allowlist_recursively(true)
        .prepend_enum_name(false)
        .size_t_is_usize(true)
}

fn find_dir(default: &'static str, env_key: &'static str) -> PathBuf {
    if let Ok(dir) = env::var(env_key) {
        PathBuf::from(dir)
    } else {
        PathBuf::from(default)
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Find the NVIDIA Video Codec SDK directory
    let sdk_dir = find_dir(".", "NVIDIA_VIDEO_CODEC_SDK_DIR");
    println!("cargo:warning=Using NVIDIA Video Codec SDK from: {}", sdk_dir.display());

    // Find the interface directory
    let interface_path = sdk_dir.join("Interface");
    println!("cargo:warning=Interface path: {}", interface_path.display());

    // Find the CUDA Toolkit directory
    let cuda_dir = if let Ok(dir) = env::var("CUDA_PATH") {
        PathBuf::from(dir)
    } else {
        PathBuf::from("C:/Program Files/NVIDIA GPU Computing Toolkit/CUDA/v12.8")
    };
    
    // Add CUDA include directory
    let cuda_include_dir = cuda_dir.join("include");
    println!("cargo:warning=Using CUDA include directory: {}", cuda_include_dir.display());

    // Add library paths
    let cuda_lib_dir = cuda_dir.join("lib/x64");
    println!("cargo:warning=Using CUDA Toolkit from: {}", cuda_lib_dir.display());

    // Link to required libraries
    println!("cargo:rustc-link-lib=dylib=cuda");
    println!("cargo:rustc-link-lib=dylib=nvcuvid");
    println!("cargo:rustc-link-lib=dylib=nvencodeapi");

    // Windows-specific setup
    if cfg!(target_os = "windows") {
        println!("cargo:warning=Windows build: Linking to NVIDIA libraries from {}", sdk_dir.display());
        
        // Add library paths
        println!("cargo:rustc-link-search=native={}", cuda_lib_dir.display());
        println!("cargo:rustc-link-search=native={}/Lib/x64", sdk_dir.display());
    }

    // Create a wrapper.h file that includes all necessary headers
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let wrapper_path = out_dir.join("wrapper.h");
    let mut wrapper_file = File::create(&wrapper_path).unwrap();
    
    // Write the wrapper file
    writeln!(wrapper_file, "#include <cuda.h>").unwrap();
    writeln!(wrapper_file, "#include <nvcuvid.h>").unwrap();
    writeln!(wrapper_file, "#include <nvEncodeAPI.h>").unwrap();
    
    // Set up include paths
    let mut clang_args = Vec::new();
    clang_args.push(format!("-I{}", cuda_include_dir.display()));
    clang_args.push(format!("-I{}", interface_path.display()));

    // Generate bindings
    let bindings = common_builder()
        .header(wrapper_path.to_str().unwrap())
        .clang_args(&clang_args)
        // CUDA types and functions
        .allowlist_type("CUresult")
        .allowlist_type("CUdevice")
        .allowlist_type("CUcontext")
        .allowlist_type("CUdeviceptr")
        .allowlist_type("CUmemorytype_enum")
        .allowlist_type("CUDA_MEMCPY2D")
        .allowlist_function("cuInit")
        .allowlist_function("cuDeviceGet")
        .allowlist_function("cuDeviceGetCount")
        .allowlist_function("cuCtxCreate_v2")
        .allowlist_function("cuCtxDestroy_v2")
        .allowlist_function("cuMemcpy2D_v2")
        // CUVID types and functions
        .allowlist_type("CUVIDEOFORMAT")
        .allowlist_type("CUVIDPICPARAMS")
        .allowlist_type("CUVIDPARSERDISPINFO")
        .allowlist_type("CUVIDPARSERPARAMS")
        .allowlist_type("CUVIDDECODECREATEINFO")
        .allowlist_type("CUVIDPROCPARAMS")
        .allowlist_type("CUVIDSOURCEDATAPACKET")
        .allowlist_type("CUvideodecoder")
        .allowlist_type("CUvideoparser")
        .allowlist_type("CUvideoctxlock")
        .allowlist_function("cuvidCreateDecoder")
        .allowlist_function("cuvidDestroyDecoder")
        .allowlist_function("cuvidCreateVideoParser")
        .allowlist_function("cuvidDestroyVideoParser")
        .allowlist_function("cuvidDecodePicture")
        .allowlist_function("cuvidMapVideoFrame")
        .allowlist_function("cuvidUnmapVideoFrame")
        .allowlist_function("cuvidCtxLockCreate")
        .allowlist_function("cuvidCtxLockDestroy")
        .allowlist_function("cuvidParseVideoData")
        // CUVID constants
        .allowlist_var("cudaVideoCodec_.*")
        .allowlist_var("cudaVideoSurfaceFormat_.*")
        .allowlist_var("cudaVideoDeinterlaceMode_.*")
        .allowlist_var("CUVID_PKT_.*")
        // NVENC types and functions
        .allowlist_type("NV_ENC_.*")
        .allowlist_function("NvEncodeAPI.*")
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to a single file
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings");
}
