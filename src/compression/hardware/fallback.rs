#![allow(dead_code)]
use anyhow::Result;
use anyhow::anyhow;
use log::{info, warn, error, debug};
use std::collections::HashMap;

use super::{HardwareEncoder, HardwareCapabilities};

/// Fallback system for hardware acceleration failures
pub struct FallbackSystem {
    fallback_chain: Vec<HardwareEncoder>,
    failure_counts: HashMap<HardwareEncoder, u32>,
    max_failures: u32,
}

impl FallbackSystem {
    pub fn new(capabilities: &HardwareCapabilities) -> Self {
        let fallback_chain = create_fallback_chain(&capabilities.available_encoders);
        
        Self {
            fallback_chain,
            failure_counts: HashMap::new(),
            max_failures: 3, // Allow 3 failures before blacklisting an encoder
        }
    }
    
    /// Get the next encoder to try, considering previous failures
    pub fn get_next_encoder(&self, preferred: &HardwareEncoder) -> HardwareEncoder {
        // If preferred encoder hasn't failed too much, use it
        if self.is_encoder_usable(preferred) {
            return *preferred;
        }
        
        // Find the first usable encoder in the fallback chain
        for encoder in &self.fallback_chain {
            if self.is_encoder_usable(encoder) {
                info!("Falling back to encoder: {:?}", encoder);
                return *encoder;
            }
        }
        
        // Ultimate fallback: software encoding
        warn!("All hardware encoders failed, falling back to software");
        HardwareEncoder::Software
    }
    
    /// Record a failure for an encoder
    pub fn record_failure(&mut self, encoder: &HardwareEncoder, error: &anyhow::Error) {
        let count = self.failure_counts.entry(*encoder).or_insert(0);
        *count += 1;
        
        warn!("Encoder {:?} failed (attempt {}): {}", encoder, count, error);
        
        if *count >= self.max_failures {
            error!("Encoder {:?} has failed {} times, blacklisting", encoder, count);
        }
    }
    
    /// Record a success for an encoder (resets failure count)
    pub fn record_success(&mut self, encoder: &HardwareEncoder) {
        if self.failure_counts.contains_key(encoder) {
            debug!("Encoder {:?} succeeded, resetting failure count", encoder);
            self.failure_counts.insert(*encoder, 0);
        }
    }
    
    /// Check if an encoder is still usable (hasn't failed too many times)
    pub fn is_encoder_usable(&self, encoder: &HardwareEncoder) -> bool {
        let failure_count = self.failure_counts.get(encoder).unwrap_or(&0);
        *failure_count < self.max_failures
    }
    
    /// Get fallback recommendations for a specific error
    pub fn get_error_specific_fallback(&self, encoder: &HardwareEncoder, error: &str) -> Option<HardwareEncoder> {
        debug!("Analyzing error for fallback recommendation: {}", error);
        
        let error_lower = error.to_lowercase();
        
        // CUDA/NVENC specific errors
        if matches!(encoder, HardwareEncoder::NvencH264 | HardwareEncoder::NvencH265 | HardwareEncoder::NvencAV1) {
            if error_lower.contains("cuda") || error_lower.contains("nvenc") {
                return self.find_alternative_vendor_encoder(encoder);
            }
        }
        
        // AMD specific errors
        if matches!(encoder, HardwareEncoder::AmfH264 | HardwareEncoder::AmfH265) {
            if error_lower.contains("amf") || error_lower.contains("amd") {
                return self.find_alternative_vendor_encoder(encoder);
            }
        }
        
        // Intel specific errors
        if matches!(encoder, HardwareEncoder::QsvH264 | HardwareEncoder::QsvH265 | HardwareEncoder::QsvAV1) {
            if error_lower.contains("qsv") || error_lower.contains("intel") {
                return self.find_alternative_vendor_encoder(encoder);
            }
        }
        
        // Memory errors - try software fallback
        if error_lower.contains("memory") || error_lower.contains("out of memory") {
            return Some(HardwareEncoder::Software);
        }
        
        // Device errors - try different vendor or software
        if error_lower.contains("device") || error_lower.contains("unavailable") {
            return self.find_alternative_vendor_encoder(encoder);
        }
        
        None // No specific recommendation
    }
    
    /// Find an encoder from a different vendor for the same codec
    fn find_alternative_vendor_encoder(&self, failed_encoder: &HardwareEncoder) -> Option<HardwareEncoder> {
        // Get the codec type from the failed encoder
        let codec = match failed_encoder {
            HardwareEncoder::NvencH264 | HardwareEncoder::AmfH264 | HardwareEncoder::QsvH264 => "h264",
            HardwareEncoder::NvencH265 | HardwareEncoder::AmfH265 | HardwareEncoder::QsvH265 => "h265",
            HardwareEncoder::NvencAV1 | HardwareEncoder::QsvAV1 => "av1",
            _ => return None,
        };
        
        // Find alternative encoders for the same codec
        let alternatives: Vec<HardwareEncoder> = match codec {
            "h264" => vec![
                HardwareEncoder::NvencH264,
                HardwareEncoder::AmfH264,
                HardwareEncoder::QsvH264,
                HardwareEncoder::Vaapi,
                HardwareEncoder::VideoToolbox,
            ],
            "h265" => vec![
                HardwareEncoder::NvencH265,
                HardwareEncoder::AmfH265,
                HardwareEncoder::QsvH265,
            ],
            "av1" => vec![
                HardwareEncoder::NvencAV1,
                HardwareEncoder::QsvAV1,
            ],
            _ => vec![],
        };
        
        // Find the first alternative that's usable and different from the failed encoder
        for alternative in alternatives {
            if alternative != *failed_encoder && self.is_encoder_usable(&alternative) {
                if self.fallback_chain.contains(&alternative) {
                    debug!("Found alternative encoder: {:?}", alternative);
                    return Some(alternative);
                }
            }
        }
        
        None
    }
    
    /// Get statistics about encoder failures
    pub fn get_failure_stats(&self) -> HashMap<HardwareEncoder, u32> {
        self.failure_counts.clone()
    }
    
    /// Reset failure counts (useful for testing or recovery)
    pub fn reset_failures(&mut self) {
        self.failure_counts.clear();
        info!("Encoder failure counts reset");
    }
    
    /// Check if any hardware encoders are still available
    pub fn has_usable_hardware_encoders(&self) -> bool {
        self.fallback_chain.iter()
            .filter(|e| e.is_hardware_accelerated())
            .any(|e| self.is_encoder_usable(e))
    }
}

/// Create a fallback chain based on available encoders and their reliability
fn create_fallback_chain(available_encoders: &[HardwareEncoder]) -> Vec<HardwareEncoder> {
    // Priority order based on general reliability and performance
    let priority_order = [
        // NVIDIA NVENC (generally most reliable)
        HardwareEncoder::NvencH264,
        HardwareEncoder::NvencH265,
        
        // Platform-specific (usually reliable on their respective platforms)
        HardwareEncoder::VideoToolbox, // macOS
        
        // Intel QuickSync (good compatibility)
        HardwareEncoder::QsvH264,
        HardwareEncoder::QsvH265,
        
        // AMD VCE (can be less stable on some systems)
        HardwareEncoder::AmfH264,
        HardwareEncoder::AmfH265,
        
        // Linux VAAPI (depends on drivers)
        HardwareEncoder::Vaapi,
        
        // Newer codecs (less compatibility)
        HardwareEncoder::NvencAV1,
        HardwareEncoder::QsvAV1,
        
        // Software fallback (always works)
        HardwareEncoder::Software,
    ];
    
    // Filter to only include available encoders
    priority_order.iter()
        .filter(|encoder| available_encoders.contains(encoder))
        .cloned()
        .collect()
}

/// Analyze an error to determine the best recovery strategy
pub fn analyze_error_for_recovery(error: &anyhow::Error) -> RecoveryStrategy {
    let error_string = error.to_string().to_lowercase();
    
    // Memory-related errors
    if error_string.contains("out of memory") || error_string.contains("memory") {
        return RecoveryStrategy::ReduceMemoryUsage;
    }
    
    // Device-related errors
    if error_string.contains("device") || error_string.contains("unavailable") {
        return RecoveryStrategy::ChangeDevice;
    }
    
    // Driver-related errors
    if error_string.contains("driver") || error_string.contains("initialization") {
        return RecoveryStrategy::FallbackToSoftware;
    }
    
    // Codec-related errors
    if error_string.contains("codec") || error_string.contains("unsupported") {
        return RecoveryStrategy::ChangeEncoder;
    }
    
    // Hardware-specific errors
    if error_string.contains("nvenc") || error_string.contains("cuda") {
        return RecoveryStrategy::AvoidNvidia;
    }
    
    if error_string.contains("amf") || error_string.contains("amd") {
        return RecoveryStrategy::AvoidAmd;
    }
    
    if error_string.contains("qsv") || error_string.contains("intel") {
        return RecoveryStrategy::AvoidIntel;
    }
    
    // Generic fallback
    RecoveryStrategy::TryAlternative
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    ReduceMemoryUsage,     // Lower resolution, quality, or enable memory optimization
    ChangeDevice,          // Try a different GPU or hardware device
    ChangeEncoder,         // Switch to different encoder for same vendor
    AvoidNvidia,          // Temporarily avoid NVIDIA encoders
    AvoidAmd,             // Temporarily avoid AMD encoders
    AvoidIntel,           // Temporarily avoid Intel encoders
    FallbackToSoftware,   // Use CPU encoding
    TryAlternative,       // Generic alternative attempt
}

impl RecoveryStrategy {
    pub fn description(&self) -> &'static str {
        match self {
            RecoveryStrategy::ReduceMemoryUsage => "Reduce memory usage and try again",
            RecoveryStrategy::ChangeDevice => "Try a different hardware device",
            RecoveryStrategy::ChangeEncoder => "Switch to different encoder",
            RecoveryStrategy::AvoidNvidia => "Avoid NVIDIA encoders temporarily",
            RecoveryStrategy::AvoidAmd => "Avoid AMD encoders temporarily",
            RecoveryStrategy::AvoidIntel => "Avoid Intel encoders temporarily",
            RecoveryStrategy::FallbackToSoftware => "Use software encoding",
            RecoveryStrategy::TryAlternative => "Try alternative approach",
        }
    }
}

/// Apply a recovery strategy to compression settings
pub fn apply_recovery_strategy(
    strategy: &RecoveryStrategy,
    current_encoder: &HardwareEncoder,
    capabilities: &HardwareCapabilities,
) -> Result<HardwareEncoder> {
    match strategy {
        RecoveryStrategy::ReduceMemoryUsage => {
            // For now, keep the same encoder but caller should reduce quality/resolution
            Ok(*current_encoder)
        },
        
        RecoveryStrategy::ChangeDevice => {
            // Try to find same vendor encoder with different device
            if capabilities.cuda_devices.len() > 1 && current_encoder.vendor() == "NVIDIA" {
                // Could try different CUDA device, but for now return same encoder
                Ok(*current_encoder)
            } else {
                Err(anyhow!("No alternative devices available"))
            }
        },
        
        RecoveryStrategy::ChangeEncoder => {
            // Try different encoder from same vendor
            match current_encoder {
                HardwareEncoder::NvencH265 => Ok(HardwareEncoder::NvencH264),
                HardwareEncoder::NvencAV1 => Ok(HardwareEncoder::NvencH264),
                HardwareEncoder::AmfH265 => Ok(HardwareEncoder::AmfH264),
                HardwareEncoder::QsvH265 => Ok(HardwareEncoder::QsvH264),
                HardwareEncoder::QsvAV1 => Ok(HardwareEncoder::QsvH264),
                _ => Err(anyhow!("No alternative encoder for same vendor")),
            }
        },
        
        RecoveryStrategy::AvoidNvidia => {
            // Find non-NVIDIA encoder
            let alternatives = [
                HardwareEncoder::AmfH264,
                HardwareEncoder::QsvH264,
                HardwareEncoder::Vaapi,
                HardwareEncoder::VideoToolbox,
                HardwareEncoder::Software,
            ];
            
            for &alt in &alternatives {
                if capabilities.available_encoders.contains(&alt) {
                    return Ok(alt);
                }
            }
            
            Ok(HardwareEncoder::Software)
        },
        
        RecoveryStrategy::AvoidAmd => {
            // Find non-AMD encoder
            let alternatives = [
                HardwareEncoder::NvencH264,
                HardwareEncoder::QsvH264,
                HardwareEncoder::Vaapi,
                HardwareEncoder::VideoToolbox,
                HardwareEncoder::Software,
            ];
            
            for &alt in &alternatives {
                if capabilities.available_encoders.contains(&alt) {
                    return Ok(alt);
                }
            }
            
            Ok(HardwareEncoder::Software)
        },
        
        RecoveryStrategy::AvoidIntel => {
            // Find non-Intel encoder
            let alternatives = [
                HardwareEncoder::NvencH264,
                HardwareEncoder::AmfH264,
                HardwareEncoder::Vaapi,
                HardwareEncoder::VideoToolbox,
                HardwareEncoder::Software,
            ];
            
            for &alt in &alternatives {
                if capabilities.available_encoders.contains(&alt) {
                    return Ok(alt);
                }
            }
            
            Ok(HardwareEncoder::Software)
        },
        
        RecoveryStrategy::FallbackToSoftware => {
            Ok(HardwareEncoder::Software)
        },
        
        RecoveryStrategy::TryAlternative => {
            // Generic fallback logic
            if let Some(preferred) = &capabilities.preferred_encoder {
                if *preferred != *current_encoder {
                    return Ok(*preferred);
                }
            }
            
            Ok(HardwareEncoder::Software)
        },
    }
}
