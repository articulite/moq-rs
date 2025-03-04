use anyhow::{Result, anyhow};
use image::{ImageBuffer, Rgba};
use std::time::{Duration, Instant};
use windows::{
    Media::MediaProperties::*,
    Media::Transcoding::*,
    Storage::Streams::*,
    Win32::Media::MediaFoundation::*,
};

// Initialize Media Foundation
pub fn init_media_foundation() -> Result<()> {
    unsafe {
        MFStartup(MF_VERSION, MFSTARTUP_FULL)?;
    }
    Ok(())
}

// Shutdown Media Foundation
pub fn shutdown_media_foundation() -> Result<()> {
    unsafe {
        MFShutdown()?;
    }
    Ok(())
}

pub struct EncodedFrame {
    pub data: Vec<u8>,
    pub timestamp: Duration,
    pub is_keyframe: bool,
}

pub struct HEVCEncoder {
    width: u32,
    height: u32,
    bitrate: u32,
    fps: u32,
    last_keyframe: Instant,
    keyframe_interval: Duration,
    frame_count: u64,
    encoder: Option<MediaEncodingProfile>,
    transcoder: Option<MediaTranscoder>,
}

impl HEVCEncoder {
    pub fn new(width: u32, height: u32, bitrate: u32, fps: u32) -> Result<Self> {
        // Initialize Media Foundation
        init_media_foundation()?;
        
        // Create encoding profile for HEVC
        let profile = MediaEncodingProfile::new()?;
        
        // Set video properties
        let video_props = VideoEncodingProperties::CreateHevc()?;
        video_props.SetWidth(width)?;
        video_props.SetHeight(height)?;
        video_props.SetBitrate(bitrate * 1000)?; // Convert kbps to bps
        
        // Windows API doesn't have a direct way to set frame rate in this version
        // We'll rely on the default frame rate handling
        
        // Set the video properties on the profile
        profile.SetVideo(&video_props)?;
        
        // Create transcoder
        let transcoder = MediaTranscoder::new()?;
        transcoder.SetVideoProcessingAlgorithm(MediaVideoProcessingAlgorithm::Default)?;
        
        Ok(Self {
            width,
            height,
            bitrate,
            fps,
            last_keyframe: Instant::now(),
            keyframe_interval: Duration::from_secs(2), // 2 seconds between keyframes
            frame_count: 0,
            encoder: Some(profile),
            transcoder: Some(transcoder),
        })
    }
    
    pub fn encode_frame(&mut self, frame: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<EncodedFrame> {
        self.frame_count += 1;
        
        // Check if we need a keyframe
        let elapsed = self.last_keyframe.elapsed();
        let is_keyframe = elapsed >= self.keyframe_interval;
        
        if is_keyframe {
            self.last_keyframe = Instant::now();
        }
        
        // Convert RGBA image to HEVC
        let encoded_data = self.encode_rgba_to_hevc(frame, is_keyframe)?;
        
        Ok(EncodedFrame {
            data: encoded_data,
            timestamp: Duration::from_secs_f64(self.frame_count as f64 / self.fps as f64),
            is_keyframe,
        })
    }
    
    fn encode_rgba_to_hevc(&self, frame: &ImageBuffer<Rgba<u8>, Vec<u8>>, _force_keyframe: bool) -> Result<Vec<u8>> {
        // Create a memory stream for the input data
        let input_stream = InMemoryRandomAccessStream::new()?;
        let output_stream = InMemoryRandomAccessStream::new()?;
        
        // Write RGBA data to the input stream
        let data_writer = DataWriter::CreateDataWriter(&input_stream.GetOutputStreamAt(0)?)?;
        
        // Convert RGBA to NV12 (which is commonly used for encoding)
        let nv12_data = self.convert_rgba_to_nv12(frame)?;
        data_writer.WriteBytes(&nv12_data)?;
        data_writer.StoreAsync()?.get()?;
        data_writer.FlushAsync()?.get()?;
        data_writer.Close()?;
        
        // Prepare the transcoder
        let transcoder = self.transcoder.as_ref().ok_or_else(|| anyhow!("Transcoder not initialized"))?;
        let profile = self.encoder.as_ref().ok_or_else(|| anyhow!("Encoder profile not initialized"))?;
        
        // Prepare the transcoding operation
        let prepare_op = transcoder.PrepareStreamTranscodeAsync(
            &input_stream.CloneStream()?,
            &output_stream.CloneStream()?,
            profile,
        )?;
        
        // Get the transcoder
        let stream_transcoder = prepare_op.get()?;
        
        // Start transcoding
        let transcode_op = stream_transcoder.TranscodeAsync()?;
        let _result = transcode_op.get()?;
        
        // We'll assume transcoding was successful if we got this far
        // The result is a unit type () in this version of the API
        
        // Read the encoded data
        let data_reader = DataReader::CreateDataReader(&output_stream.GetInputStreamAt(0)?)?;
        let size = output_stream.Size()? as u32;
        data_reader.LoadAsync(size)?.get()?;
        
        let mut buffer = vec![0u8; size as usize];
        data_reader.ReadBytes(&mut buffer)?;
        data_reader.Close()?;
        
        Ok(buffer)
    }
    
    fn convert_rgba_to_nv12(&self, frame: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<Vec<u8>> {
        let width = self.width as usize;
        let height = self.height as usize;
        
        // NV12 format: Y plane followed by interleaved UV plane
        let y_plane_size = width * height;
        let uv_plane_size = width * height / 2;
        let mut nv12_data = vec![0u8; y_plane_size + uv_plane_size];
        
        // Fill Y plane
        for y in 0..height {
            for x in 0..width {
                let rgba = frame.get_pixel(x as u32, y as u32);
                let r = rgba[0] as f32;
                let g = rgba[1] as f32;
                let b = rgba[2] as f32;
                
                // RGB to Y conversion
                let y_value = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
                nv12_data[y * width + x] = y_value;
            }
        }
        
        // Fill interleaved UV plane (downsampled by 2 in both dimensions)
        for y in 0..height/2 {
            for x in 0..width/2 {
                // Average 4 pixels for each UV value
                let mut r_sum = 0f32;
                let mut g_sum = 0f32;
                let mut b_sum = 0f32;
                
                for dy in 0..2 {
                    for dx in 0..2 {
                        let rgba = frame.get_pixel((x*2 + dx) as u32, (y*2 + dy) as u32);
                        r_sum += rgba[0] as f32;
                        g_sum += rgba[1] as f32;
                        b_sum += rgba[2] as f32;
                    }
                }
                
                r_sum /= 4.0;
                g_sum /= 4.0;
                b_sum /= 4.0;
                
                // RGB to UV conversion
                let u_value = (-0.169 * r_sum - 0.331 * g_sum + 0.5 * b_sum + 128.0) as u8;
                let v_value = (0.5 * r_sum - 0.419 * g_sum - 0.081 * b_sum + 128.0) as u8;
                
                let uv_index = y_plane_size + y * width + x * 2;
                nv12_data[uv_index] = u_value;
                nv12_data[uv_index + 1] = v_value;
            }
        }
        
        Ok(nv12_data)
    }
}

impl Drop for HEVCEncoder {
    fn drop(&mut self) {
        // Shutdown Media Foundation when the encoder is dropped
        let _ = shutdown_media_foundation();
    }
} 