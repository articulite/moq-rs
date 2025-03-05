use anyhow::Result;
use moq_x265::{X265Decoder};
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() -> Result<()> {
    println!("Testing x265 decoder...");
    
    // Create decoder
    let mut decoder = X265Decoder::new();
    println!("Created x265 decoder");
    
    // Load a sample HEVC file
    let sample_path = Path::new("output/all_frames.hevc");
    if !sample_path.exists() {
        println!("Sample file not found. Please run test_x265.rs first to generate sample files.");
        return Ok(());
    }
    
    // Read the file
    let mut file = File::open(sample_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    
    println!("Read {} bytes from {}", data.len(), sample_path.display());
    
    // Try to decode the frame
    match decoder.decode_frame(&data) {
        Ok(Some(image)) => {
            println!("Successfully decoded image: {}x{}", image.width(), image.height());
        },
        Ok(None) => {
            println!("Decoder processed data but returned no image");
        },
        Err(e) => {
            println!("Failed to decode: {}", e);
        }
    }
    
    println!("Test completed!");
    Ok(())
} 