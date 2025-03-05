use std::env;
use std::path::PathBuf;

fn main() {
    // Try to find x265 using pkg-config
    if let Ok(lib) = pkg_config::probe_library("x265") {
        for include in &lib.include_paths {
            println!("cargo:include={}", include.display());
        }
    } else {
        // If pkg-config fails, try some common locations
        println!("cargo:warning=x265 not found via pkg-config, trying common locations");
        
        // On Windows, we'll need to specify the include and lib paths manually
        if cfg!(target_os = "windows") {
            // Check if X265_DIR environment variable is set
            if let Ok(x265_dir) = env::var("X265_DIR") {
                // Add the x64 lib directory to the search path
                println!("cargo:rustc-link-search=native={}/lib/x64", x265_dir);
                println!("cargo:include={}/include", x265_dir);
                
                // Print some debug information
                println!("cargo:warning=Using x265 from {}", x265_dir);
                println!("cargo:warning=Include path: {}/include", x265_dir);
                println!("cargo:warning=Library path: {}/lib/x64", x265_dir);
                
                // On Windows, we need to use the correct library name
                if std::path::Path::new(&format!("{}/lib/x64/x265.lib", x265_dir)).exists() {
                    println!("cargo:rustc-link-lib=x265");
                    println!("cargo:warning=Using x265.lib");
                } else if std::path::Path::new(&format!("{}/lib/x64/libx265.lib", x265_dir)).exists() {
                    println!("cargo:rustc-link-lib=libx265");
                    println!("cargo:warning=Using libx265.lib");
                } else {
                    println!("cargo:warning=Neither x265.lib nor libx265.lib found in {}/lib/x64", x265_dir);
                }
            } else {
                println!("cargo:warning=X265_DIR environment variable not set. Please set it to the x265 installation directory.");
                println!("cargo:warning=You can download x265 from https://www.videolan.org/developers/x265.html");
                println!("cargo:warning=Alternatively, you can install it via vcpkg: vcpkg install x265:x64-windows");
            }
        }
    }

    // Generate bindings with the correct include path
    let mut builder = bindgen::Builder::default();
    
    // If X265_DIR is set, use it to find the x265.h header
    if let Ok(x265_dir) = env::var("X265_DIR") {
        let header_path = format!("{}/include/x265.h", x265_dir);
        println!("cargo:warning=Using header at {}", header_path);
        
        if std::path::Path::new(&header_path).exists() {
            builder = builder.header(&header_path);
        } else {
            println!("cargo:warning=Header file not found at {}", header_path);
            builder = builder.header_contents("wrapper.h", "#include <x265.h>");
        }
    } else {
        builder = builder.header_contents("wrapper.h", "#include <x265.h>");
    }
    
    // Generate the bindings
    let bindings = builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
} 