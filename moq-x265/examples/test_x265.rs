use anyhow::Result;
use image::{ImageBuffer, Rgba};
use moq_x265::{X265Encoder, X265Decoder, EncodedFrame};
use std::time::{Duration, Instant};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() -> Result<()> {
    println!("Testing x265 encoder and decoder...");
    
    // Configuration
    let width = 640;
    let height = 480;
    let fps = 30;
    let keyframe_interval = 60; // 2 seconds at 30fps
    let bitrate = 5000; // 5 Mbps
    let duration_seconds = 5;
    let total_frames = fps * duration_seconds;
    
    // Create encoder
    let mut encoder = match X265Encoder::new(width, height, bitrate, fps, keyframe_interval) {
        Ok(encoder) => {
            println!("Successfully created x265 encoder");
            encoder
        },
        Err(e) => {
            println!("Failed to create x265 encoder: {}", e);
            return Err(e);
        }
    };
    
    // Create decoder
    let mut decoder = X265Decoder::new();
    println!("Created x265 decoder");
    
    // Generate and encode frames
    let mut encoded_frames = Vec::new();
    let start_time = Instant::now();
    
    // Create output directory if it doesn't exist
    let output_dir = Path::new("output");
    if !output_dir.exists() {
        std::fs::create_dir(output_dir)?;
        println!("Created output directory");
    }
    
    for frame_idx in 0..total_frames {
        // Create a test frame (alternating between purple and blue every second)
        let is_purple = (frame_idx / fps) % 2 == 0; // Change color every second instead of every frame
        let mut img = ImageBuffer::new(width, height);
        
        for (_, _, pixel) in img.enumerate_pixels_mut() {
            if is_purple {
                // Purple
                *pixel = Rgba([128, 0, 128, 255]);
            } else {
                // Blue
                *pixel = Rgba([0, 0, 255, 255]);
            }
        }
        
        println!("Frame {}/{}: {}", frame_idx + 1, total_frames, if is_purple { "Purple" } else { "Blue" });
        
        // Encode frame
        match encoder.encode_frame(&img) {
            Ok(frame) => {
                println!("  Encoded: {} bytes, keyframe: {}", frame.data.len(), frame.is_keyframe);
                
                // Save the first frame and keyframes to disk
                if frame_idx == 0 || frame.is_keyframe {
                    let filename = if frame_idx == 0 {
                        format!("output/first_frame.hevc")
                    } else {
                        format!("output/keyframe_{}.hevc", frame_idx)
                    };
                    
                    let mut file = File::create(&filename)?;
                    file.write_all(&frame.data)?;
                    println!("  Saved frame to {}", filename);
                }
                
                encoded_frames.push(frame);
            },
            Err(e) => {
                println!("  Failed to encode frame: {}", e);
                return Err(e);
            }
        }
    }
    
    // Flush encoder to get any remaining frames
    while let Ok(Some(frame)) = encoder.flush() {
        println!("Flushed frame: {} bytes", frame.data.len());
        encoded_frames.push(frame);
    }
    
    let encoding_time = start_time.elapsed();
    println!("Encoded {} frames in {:?} ({:.2} fps)", 
             encoded_frames.len(), 
             encoding_time,
             encoded_frames.len() as f64 / encoding_time.as_secs_f64());
    
    // Save all frames to a single file
    let raw_hevc_path = "output/all_frames.hevc";
    let mut all_frames_file = File::create(raw_hevc_path)?;
    for frame in &encoded_frames {
        all_frames_file.write_all(&frame.data)?;
    }
    println!("Saved all frames to {}", raw_hevc_path);
    
    // Create an MP4 file from the raw HEVC bitstream
    // This requires ffmpeg to be installed on the system
    let mp4_path = "output/color_alternating.mp4";
    println!("Creating MP4 file at {}", mp4_path);
    
    // Check if ffmpeg is available
    let ffmpeg_result = Command::new("ffmpeg")
        .arg("-version")
        .output();
    
    match ffmpeg_result {
        Ok(_) => {
            // Create MP4 file using ffmpeg
            let status = Command::new("ffmpeg")
                .args([
                    "-f", "hevc",                // Input format is raw HEVC
                    "-i", raw_hevc_path,         // Input file
                    "-c:v", "copy",              // Copy video stream without re-encoding
                    "-y",                        // Overwrite output file if it exists
                    mp4_path                     // Output file
                ])
                .status()?;
            
            if status.success() {
                println!("Successfully created MP4 file at {}", mp4_path);
                println!("You can now play this file in any video player that supports HEVC/H.265");
            } else {
                println!("Failed to create MP4 file. ffmpeg exited with status: {}", status);
            }
        },
        Err(_) => {
            println!("ffmpeg not found. To create an MP4 file, install ffmpeg and run:");
            println!("ffmpeg -f hevc -i {} -c:v copy {}", raw_hevc_path, mp4_path);
        }
    }
    
    // Decode frames
    let start_time = Instant::now();
    let mut decoded_frames = 0;
    
    for (i, frame) in encoded_frames.iter().enumerate() {
        match decoder.decode_frame(&frame.data) {
            Ok(Some(decoded_image)) => {
                println!("Decoded frame {}: {}x{}", i, decoded_image.width(), decoded_image.height());
                decoded_frames += 1;
            },
            Ok(None) => {
                println!("Frame {} processed but no image returned", i);
            },
            Err(e) => {
                println!("Failed to decode frame {}: {}", i, e);
            }
        }
    }
    
    let decoding_time = start_time.elapsed();
    println!("Decoded {} frames in {:?} ({:.2} fps)", 
             decoded_frames, 
             decoding_time,
             decoded_frames as f64 / decoding_time.as_secs_f64());
    
    println!("Test completed successfully!");
    Ok(())
} 