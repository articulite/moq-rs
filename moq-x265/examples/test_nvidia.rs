use anyhow::Result;
use image::{ImageBuffer, Rgba};
use moq_x265::{create_hardware_encoder, create_hardware_decoder};
use std::time::{Instant, Duration};
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() -> Result<()> {
    // Create output directory if it doesn't exist
    let output_dir = Path::new("output");
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)?;
    }

    // Configuration
    let width = 640;
    let height = 480;
    let bitrate = 2000000; // 2 Mbps
    let fps = 30;
    let keyframe_interval = 30;
    let num_frames = 60;

    println!("Creating NVIDIA hardware encoder...");
    let mut encoder = create_hardware_encoder(width, height, bitrate, fps, keyframe_interval)?;
    
    println!("Creating NVIDIA hardware decoder...");
    let mut decoder = create_hardware_decoder()?;
    
    // Create a file to save encoded frames
    let mut encoded_file = File::create(output_dir.join("nvidia_encoded.h265"))?;
    
    // Encoding benchmark
    let mut total_encoding_time = Duration::new(0, 0);
    let mut total_encoded_size = 0;
    let mut encoded_frames = Vec::new();
    
    println!("Encoding {} frames...", num_frames);
    for i in 0..num_frames {
        // Create a test frame (alternating colors)
        let mut frame = ImageBuffer::new(width, height);
        let color = if i % 2 == 0 {
            [255, 0, 255, 255] // Purple
        } else {
            [0, 0, 255, 255] // Blue
        };
        
        for (_, _, pixel) in frame.enumerate_pixels_mut() {
            *pixel = Rgba(color);
        }
        
        // Encode frame
        let start = Instant::now();
        let encoded = encoder.encode_frame(&frame)?;
        let duration = start.elapsed();
        
        total_encoding_time += duration;
        total_encoded_size += encoded.data.len();
        
        // Write encoded frame to file
        encoded_file.write_all(&encoded.data)?;
        
        // Save encoded frame for decoding
        encoded_frames.push(encoded);
        
        println!("Frame {}: Encoded {} bytes in {:?}", i, encoded.data.len(), duration);
    }
    
    // Calculate encoding statistics
    let avg_encoding_time = total_encoding_time.as_secs_f64() / num_frames as f64;
    let avg_frame_size = total_encoded_size as f64 / num_frames as f64;
    let encoding_fps = 1.0 / avg_encoding_time;
    
    println!("\nEncoding Statistics:");
    println!("Total frames: {}", num_frames);
    println!("Total encoding time: {:?}", total_encoding_time);
    println!("Average encoding time: {:.2} ms per frame", avg_encoding_time * 1000.0);
    println!("Encoding speed: {:.2} fps", encoding_fps);
    println!("Average frame size: {:.2} KB", avg_frame_size / 1024.0);
    println!("Total encoded size: {:.2} KB", total_encoded_size as f64 / 1024.0);
    println!("Bitrate: {:.2} Kbps", (total_encoded_size as f64 * 8.0 * fps as f64) / (num_frames as f64 * 1000.0));
    
    // Decoding benchmark
    let mut total_decoding_time = Duration::new(0, 0);
    let mut decoded_frames = 0;
    
    println!("\nDecoding {} frames...", encoded_frames.len());
    for (i, encoded) in encoded_frames.iter().enumerate() {
        // Decode frame
        let start = Instant::now();
        let decoded = decoder.decode_frame(&encoded.data)?;
        let duration = start.elapsed();
        
        total_decoding_time += duration;
        
        // Check if frame was decoded
        if let Some(frame) = decoded {
            decoded_frames += 1;
            
            // Save first, middle, and last frame
            if i == 0 || i == num_frames / 2 || i == num_frames - 1 {
                let filename = format!("output/nvidia_decoded_{}.png", i);
                frame.save(&filename)?;
                println!("Saved decoded frame to {}", filename);
            }
            
            println!("Frame {}: Decoded in {:?}", i, duration);
        } else {
            println!("Frame {}: No output (probably a reference frame)", i);
        }
    }
    
    // Calculate decoding statistics
    let avg_decoding_time = total_decoding_time.as_secs_f64() / decoded_frames as f64;
    let decoding_fps = 1.0 / avg_decoding_time;
    
    println!("\nDecoding Statistics:");
    println!("Total frames decoded: {}/{}", decoded_frames, encoded_frames.len());
    println!("Total decoding time: {:?}", total_decoding_time);
    println!("Average decoding time: {:.2} ms per frame", avg_decoding_time * 1000.0);
    println!("Decoding speed: {:.2} fps", decoding_fps);
    
    Ok(())
} 