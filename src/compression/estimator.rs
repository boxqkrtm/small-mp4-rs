use anyhow::Result;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::hardware::HardwareEncoder;
use super::{CompressionSettings, TargetSize};

#[derive(Debug, Clone)]
pub struct SizeEstimator {
    // Precomputed bitrate tables for different encoder types
    encoder_efficiency: HashMap<HardwareEncoder, f32>,
}

impl SizeEstimator {
    pub fn new() -> Self {
        let mut encoder_efficiency = HashMap::new();
        
        // Hardware encoders are typically less efficient than software but much faster
        encoder_efficiency.insert(HardwareEncoder::NvencH264, 0.85);
        encoder_efficiency.insert(HardwareEncoder::NvencH265, 0.90); // HEVC is more efficient
        encoder_efficiency.insert(HardwareEncoder::NvencAV1, 0.95);  // AV1 is most efficient
        encoder_efficiency.insert(HardwareEncoder::AmfH264, 0.80);
        encoder_efficiency.insert(HardwareEncoder::AmfH265, 0.85);
        encoder_efficiency.insert(HardwareEncoder::QsvH264, 0.82);
        encoder_efficiency.insert(HardwareEncoder::QsvH265, 0.87);
        encoder_efficiency.insert(HardwareEncoder::QsvAV1, 0.92);
        encoder_efficiency.insert(HardwareEncoder::Vaapi, 0.80);
        encoder_efficiency.insert(HardwareEncoder::VideoToolbox, 0.83);
        encoder_efficiency.insert(HardwareEncoder::Software, 1.00); // Reference efficiency
        
        Self {
            encoder_efficiency,
        }
    }
    
    /// Estimate output size for given settings
    pub fn estimate_size(
        &self,
        metadata: &VideoMetadata,
        settings: &CompressionSettings,
    ) -> Result<SizeEstimation> {
        self.estimate_target_size(metadata, settings)
    }
    
    
    /// Estimate size for target size constraint
    fn estimate_target_size(
        &self,
        metadata: &VideoMetadata,
        settings: &CompressionSettings,
    ) -> Result<SizeEstimation> {
        let target_mb = settings.target_size.as_mb();
        let optimal_bitrate = self.calculate_bitrate_for_target(metadata, target_mb);
        
        let encoding_time = self.estimate_encoding_time(metadata, settings);
        let quality_score = self.estimate_quality_from_bitrate(metadata, optimal_bitrate);
        
        debug!("Target-size estimation: {:.1} MB target, optimal bitrate: {} kbps", target_mb, optimal_bitrate);
        
        Ok(SizeEstimation {
            estimated_size_mb: target_mb,
            confidence: 0.90, // Bitrate-based sizing has high confidence
            encoding_time_seconds: encoding_time,
            quality_score,
            recommended_bitrate_kbps: Some(optimal_bitrate),
        })
    }
    
    /// Calculate bitrate needed for target file size
    fn calculate_bitrate_for_target(&self, metadata: &VideoMetadata, target_mb: f32) -> u32 {
        let duration_seconds = metadata.duration_seconds;
        
        // Calculate total available bits
        let total_bits = target_mb * 8.0 * 1024.0 * 1024.0;
        
        // Reserve space for audio (assume 128 kbps)
        let audio_bits = 128.0 * 1024.0 * duration_seconds;
        
        // Reserve 2% for container overhead
        let container_overhead = total_bits * 0.02;
        
        // Calculate available bits for video
        let available_video_bits = total_bits - audio_bits - container_overhead;
        
        // Calculate video bitrate in kbps
        let video_bitrate_bps = available_video_bits / duration_seconds;
        let video_bitrate_kbps = video_bitrate_bps / 1024.0;
        
        // Apply encoder efficiency adjustment
        let efficiency = self.encoder_efficiency.get(&HardwareEncoder::Software).unwrap_or(&1.0);
        let adjusted_bitrate = video_bitrate_kbps / efficiency;
        
        // Apply safety margin of 0.95
        let safe_bitrate = (adjusted_bitrate * 0.95) as u32;
        
        // Ensure minimum bitrate
        safe_bitrate.max(100)
    }
    
    /// Estimate quality score from bitrate
    fn estimate_quality_from_bitrate(&self, metadata: &VideoMetadata, bitrate_kbps: u32) -> f32 {
        // Calculate bits per pixel per frame
        let pixel_count = metadata.width * metadata.height;
        let bits_per_pixel = (bitrate_kbps as f32 * 1024.0) / (pixel_count as f32 * metadata.fps);
        
        // Map bits per pixel to quality score
        let quality_score = match bits_per_pixel {
            x if x >= 0.20 => 0.95,  // Excellent quality
            x if x >= 0.15 => 0.85,  // Very good quality
            x if x >= 0.10 => 0.75,  // Good quality
            x if x >= 0.07 => 0.60,  // Acceptable quality
            x if x >= 0.04 => 0.45,  // Poor quality
            _ => 0.25,               // Very poor quality
        };
        
        // Adjust for content complexity
        let complexity_adjustment = match metadata.estimated_complexity {
            ContentComplexity::Low => 1.1,    // Simple content looks better at lower bitrates
            ContentComplexity::Medium => 1.0,  // Normal adjustment
            ContentComplexity::High => 0.9,    // Complex content needs more bitrate
        };
        
        let adjusted_score: f32 = quality_score * complexity_adjustment;
        adjusted_score.min(1.0)
    }
    
    /// Get encoder efficiency multiplier
    fn get_encoder_efficiency(&self, encoder: &HardwareEncoder) -> f32 {
        self.encoder_efficiency.get(encoder).copied().unwrap_or(0.85)
    }
    
    /// Estimate encoding time based on settings and hardware
    fn estimate_encoding_time(&self, metadata: &VideoMetadata, settings: &CompressionSettings) -> f32 {
        let base_encode_time = metadata.duration_seconds * 0.2; // Assume 5x realtime for software
        
        // Apply hardware acceleration speedup
        let hw_speedup = if settings.enable_hardware_accel {
            match &settings.hardware_encoder {
                HardwareEncoder::NvencH264 | HardwareEncoder::NvencH265 => 8.0,
                HardwareEncoder::NvencAV1 => 6.0,
                HardwareEncoder::AmfH264 | HardwareEncoder::AmfH265 => 5.0,
                HardwareEncoder::QsvH264 | HardwareEncoder::QsvH265 => 6.0,
                HardwareEncoder::VideoToolbox => 4.0,
                HardwareEncoder::Vaapi => 3.0,
                _ => 1.0,
            }
        } else {
            1.0
        };
        
        // Apply preset modifier
        let preset_modifier = match &settings.hardware_preset {
            super::hardware::HardwarePreset::UltraFast => 0.5,
            super::hardware::HardwarePreset::Faster => 0.7,
            super::hardware::HardwarePreset::Fast => 0.85,
            super::hardware::HardwarePreset::Medium => 1.0,
            super::hardware::HardwarePreset::Slow => 1.3,
            super::hardware::HardwarePreset::Slower => 1.8,
            super::hardware::HardwarePreset::Highest => 2.5,
        };
        
        base_encode_time / hw_speedup * preset_modifier
    }
    
    
    /// Recommend bitrate for specific target size
    pub fn recommend_bitrate_for_size(
        &self,
        metadata: &VideoMetadata,
        target_size: TargetSize,
        encoder: &HardwareEncoder,
    ) -> Result<BitrateRecommendation> {
        let target_mb = target_size.as_mb();
        let optimal_bitrate = self.calculate_bitrate_for_target(metadata, target_mb);
        
        // Adjust for encoder efficiency
        let efficiency = self.get_encoder_efficiency(encoder);
        let adjusted_bitrate = (optimal_bitrate as f32 * efficiency) as u32;
        
        let quality_score = self.estimate_quality_from_bitrate(metadata, adjusted_bitrate);
        
        Ok(BitrateRecommendation {
            recommended_bitrate_kbps: adjusted_bitrate,
            estimated_quality: quality_score,
            size_achievable: target_mb,
            confidence: 0.9,
        })
    }
}

impl Default for SizeEstimator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub duration_seconds: f32,
    pub bitrate_kbps: Option<u32>,
    pub codec: String,
    pub estimated_complexity: ContentComplexity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentComplexity {
    Low,    // Slides, static content, low motion
    Medium, // Normal video content
    High,   // Action, gaming, high motion content
}

#[derive(Debug, Clone)]
pub struct SizeEstimation {
    pub estimated_size_mb: f32,
    pub confidence: f32, // 0.0 to 1.0
    pub encoding_time_seconds: f32,
    pub quality_score: f32, // 0.0 to 1.0
    pub recommended_bitrate_kbps: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BitrateRecommendation {
    pub recommended_bitrate_kbps: u32,
    pub estimated_quality: f32,
    pub size_achievable: f32,
    pub confidence: f32,
}

impl VideoMetadata {
    /// Create metadata with default values (useful for testing)
    pub fn default_hd() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 30.0,
            duration_seconds: 60.0,
            bitrate_kbps: Some(5000),
            codec: "h264".to_string(),
            estimated_complexity: ContentComplexity::Medium,
        }
    }
    
    /// Estimate content complexity based on resolution and bitrate
    pub fn estimate_complexity_from_bitrate(&mut self) {
        if let Some(bitrate) = self.bitrate_kbps {
            let pixel_count = self.width * self.height;
            let bits_per_pixel = (bitrate as f32 * 1000.0) / (pixel_count as f32 * self.fps);
            
            self.estimated_complexity = if bits_per_pixel > 0.2 {
                ContentComplexity::High
            } else if bits_per_pixel > 0.1 {
                ContentComplexity::Medium
            } else {
                ContentComplexity::Low
            };
        }
    }
    
    pub fn megapixels(&self) -> f32 {
        (self.width * self.height) as f32 / 1_000_000.0
    }
    
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
    
    pub fn is_high_resolution(&self) -> bool {
        self.width >= 1920 && self.height >= 1080
    }
    
    pub fn is_high_framerate(&self) -> bool {
        self.fps >= 60.0
    }
}