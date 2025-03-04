use anyhow::{Context, Result};
use image::{ImageBuffer, Rgba};
use scrap::{Capturer, Display};
use std::io::ErrorKind;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::info;

pub struct ScreenCapture {
    capturer: Capturer,
    width: u32,
    height: u32,
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
        
        // Get the requested display dimensions
        let width = displays[screen_index].width() as u32;
        let height = displays[screen_index].height() as u32;
        
        // Create capturer
        let capturer = if screen_index == 0 {
            // For the primary display (index 0), use Display::primary()
            let primary = Display::primary()?;
            info!("Using primary display: {}x{}", primary.width(), primary.height());
            Capturer::new(primary).context("Failed to create screen capturer for primary display")?
        } else {
            // For other displays, we need to get a fresh copy
            // We'll collect all displays again and then take the one we need
            let mut fresh_displays = Display::all()?;
            
            if screen_index < fresh_displays.len() {
                // Take ownership of the display at the specified index
                // This removes it from the vector, giving us ownership
                let display_opt = fresh_displays.drain(screen_index..=screen_index).next();
                
                if let Some(display) = display_opt {
                    info!("Using display {}", screen_index);
                    Capturer::new(display).context("Failed to create screen capturer")?
                } else {
                    anyhow::bail!("Failed to get display at index {}", screen_index);
                }
            } else {
                anyhow::bail!("Display index {} no longer valid", screen_index);
            }
        };
        
        Ok(Self {
            capturer,
            width,
            height,
            target_width,
            target_height,
            target_fps,
            last_frame_time: Instant::now(),
        })
    }
    
    pub async fn capture_frame(&mut self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        // Throttle capture rate to target FPS
        let frame_duration = Duration::from_secs_f64(1.0 / self.target_fps as f64);
        let elapsed = self.last_frame_time.elapsed();
        
        if elapsed < frame_duration {
            let sleep_duration = frame_duration - elapsed;
            sleep(sleep_duration).await;
        }
        
        self.last_frame_time = Instant::now();
        
        // Capture frame
        let frame = loop {
            match self.capturer.frame() {
                Ok(frame) => break frame,
                Err(error) => {
                    if error.kind() == ErrorKind::WouldBlock {
                        // Frame not ready yet, wait a bit
                        sleep(Duration::from_millis(5)).await;
                        continue;
                    } else {
                        return Err(error).context("Failed to capture frame");
                    }
                }
            }
        };
        
        // Create image buffer
        let mut img = ImageBuffer::new(self.target_width, self.target_height);
        
        // Copy pixels from frame to image buffer
        // Note: frame.data is in BGRA format
        for y in 0..self.target_height {
            for x in 0..self.target_width {
                // Scale coordinates to source frame
                let src_x = (x as f32 * self.width as f32 / self.target_width as f32) as usize;
                let src_y = (y as f32 * self.height as f32 / self.target_height as f32) as usize;
                
                // Calculate pixel index in source frame
                let src_idx = (src_y * self.width as usize + src_x) * 4;
                
                // Check bounds
                if src_idx + 3 < frame.len() {
                    // Convert BGRA to RGBA
                    let b = frame[src_idx];
                    let g = frame[src_idx + 1];
                    let r = frame[src_idx + 2];
                    let a = frame[src_idx + 3];
                    
                    img.put_pixel(x, y, Rgba([r, g, b, a]));
                }
            }
        }
        
        Ok(img)
    }
} 