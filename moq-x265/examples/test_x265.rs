use anyhow::Result;
use image::{ImageBuffer, Rgba};
use moq_x265::{X265Encoder, X265Decoder};
use std::time::Instant;

fn main() -> Result<()> {
    println!("Testing x265 encoder and decoder...");
    
    // Create a test image (blue gradient)
    let width = 640;
    let height = 480;
    let mut img = ImageBuffer::new(width, height);
    
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let x_factor = x as f32 / width as f32;
        let y_factor = y as f32 / height as f32;
        
        *pixel = Rgba([
            0,
            (y_factor * 255.0) as u8,
            (x_factor * 255.0) as u8,
            255,
        ]);
    }
    
    println!("Created test image {}x{}", width, height);
    
    // Create encoder
    let mut encoder = match X265Encoder::new(width, height, 5000, 30) {
        Ok(encoder) => {
            println!("Successfully created x265 encoder");
            encoder
        },
        Err(e) => {
            println!("Failed to create x265 encoder: {}", e);
            return Err(e);
        }
    };
    
    // Encode frame
    let start = Instant::now();
    let encoded_frame = match encoder.encode_frame(&img) {
        Ok(frame) => {
            println!("Successfully encoded frame: {} bytes, keyframe: {}", 
                     frame.data.len(), frame.is_keyframe);
            frame
        },
        Err(e) => {
            println!("Failed to encode frame: {}", e);
            return Err(e);
        }
    };
    println!("Encoding took: {:?}", start.elapsed());
    
    // Create decoder
    let mut decoder = X265Decoder::new();
    println!("Created x265 decoder");
    
    // Decode frame
    let start = Instant::now();
    let mut encoded_frames = 0;
    match decoder.decode_frame(&encoded_frame.data) {
        Ok(Some(img)) => {
            println!("Successfully decoded frame: {}x{}", img.width(), img.height());
            encoded_frames += 1;
        },
        Ok(None) => {
            println!("Decoder returned no image");
        },
        Err(e) => {
            println!("Failed to decode frame: {}", e);
            return Err(e);
        }
    };
    println!("Decoding took: {:?}", start.elapsed());
    
    println!("encoded {} frames", encoded_frames);
    println!("Test completed successfully!");
    Ok(())
} 