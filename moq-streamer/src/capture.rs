use anyhow::{Context, Result};
use image::{ImageBuffer, Rgba};
use scrap::{Capturer, Display};
use std::io::ErrorKind;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, warn};

pub struct ScreenCapture {
    capturer: Capturer,
    display: Display,
    target_width: u32,
    target_height: u32,
    target_fps: u32,
    last_frame_time: Instant,
}

impl ScreenCapture {
    pub fn new(
        screen_index: usize,
        target_width: u32,
        target_height: u32,
        target_fps: u32,
    ) -> Result<Self> {
        // Get all displays
        let displays = Display::all()?;
        
        // Check if screen index is valid
        if screen_index >= displays.len() {
            anyhow::bail!("Screen index {} is out of range (max: {})", screen_index, displays.len() - 1);
        }
        
        // Get the requested display
        let display = displays[screen_index];
        
        // Create capturer
        let capturer = Capturer::new(display.clone())
            .context("Failed to create screen capturer")?;
        
        Ok(Self {
            capturer,
            display,
            target_width,
            target_height,
            target_fps,
            last_frame_time: Instant::now(),
        })
    }
    
    pub async fn capture_frame(&mut self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        // Calculate time until next frame should be captured
        let frame_duration = Duration::from_secs_f64(1.0 / self.target_fps as f64);
        let elapsed = self.last_frame_time.elapsed();
        
        if elapsed < frame_duration {
            // Wait until it's time for the next frame
            sleep(frame_duration - elapsed).await;
        }
        
        // Remember when we captured this frame
        self.last_frame_time = Instant::now();
        
        // Capture frame
        let frame = loop {
            match self.capturer.frame() {
                Ok(frame) => break frame,
                Err(error) => {
                    if error.kind() == ErrorKind::WouldBlock {
                        // Not ready yet, try again
                        sleep(Duration::from_millis(5)).await;
                        continue;
                    }
                    return Err(error).context("Failed to capture frame");
                }
            }
        };
        
        // Get display dimensions
        let width = self.display.width() as u32;
        let height = self.display.height() as u32;
        
        // Convert frame to RGBA image buffer
        let stride = frame.len() / height as usize;
        
        // Create image from raw pixels
        let img = ImageBuffer::from_fn(width, height, |x, y| {
            let i = y as usize * stride + x as usize * 4;
            Rgba([
                frame[i + 2], // B
                frame[i + 1], // G
                frame[i],     // R
                255,          // A
            ])
        });
        
        // Resize if needed
        if width != self.target_width || height != self.target_height {
            debug!("Resizing from {}x{} to {}x{}", width, height, self.target_width, self.target_height);
            
            let resized = image::imageops::resize(
                &img,
                self.target_width,
                self.target_height,
                image::imageops::FilterType::Triangle
            );
            
            Ok(resized)
        } else {
            Ok(img)
        }
    }
} 