pub mod hardware;
mod engine;
mod estimator;
mod metadata;
mod size_presets;

pub use engine::CompressionEngine;
pub use estimator::SizeEstimator;
pub use size_presets::TargetSize;

use anyhow::Result;
use log::warn;

use hardware::{HardwareCapabilities, HardwareEncoder, HardwarePreset, HardwareQuality};
use crate::cli::CompressionCliSettings;

#[derive(Debug, Clone)]
pub struct CompressionSettings {
    // Existing fields
    pub target_size: TargetSize,
    #[allow(dead_code)]
    pub estimated_size_mb: Option<f32>,
    
    // New hardware acceleration fields
    pub hardware_encoder: HardwareEncoder,
    pub enable_hardware_accel: bool,
    pub cuda_device_id: Option<u32>,
    pub hardware_preset: HardwarePreset,
    #[allow(dead_code)]
    pub hardware_quality: HardwareQuality,
    #[allow(dead_code)]
    pub force_software_fallback: bool,
    pub memory_optimization: bool,
    pub compatibility_mode: bool,  // Force x264 for maximum compatibility
}

impl CompressionSettings {
    pub fn from_cli_settings(
        cli_settings: &CompressionCliSettings,
        hw_capabilities: &HardwareCapabilities,
    ) -> Result<Self> {
        let target_size = TargetSize::from_mb(cli_settings.size.as_mb());
        
        // Determine hardware encoder
        let hardware_encoder = match &cli_settings.hw_encoder {
            Some(cli_encoder) => {
                let requested = cli_encoder.to_hardware_encoder();
                if hw_capabilities.available_encoders.contains(&requested) || cli_settings.force_software {
                    requested
                } else {
                    warn!("Requested encoder {:?} not available, using preferred encoder", requested);
                    hw_capabilities.preferred_encoder.clone().unwrap_or(HardwareEncoder::Software)
                }
            },
            None => {
                if cli_settings.force_software {
                    HardwareEncoder::Software
                } else {
                    hw_capabilities.preferred_encoder.clone().unwrap_or(HardwareEncoder::Software)
                }
            }
        };
        
        let enable_hardware_accel = !cli_settings.force_software && hardware_encoder != HardwareEncoder::Software;
        
        Ok(CompressionSettings {
            target_size,
            estimated_size_mb: None,
            hardware_encoder,
            enable_hardware_accel,
            cuda_device_id: cli_settings.cuda_device,
            hardware_preset: cli_settings.hw_preset.to_hardware_preset(),
            hardware_quality: cli_settings.hw_quality.to_hardware_quality(),
            force_software_fallback: cli_settings.force_software,
            memory_optimization: cli_settings.memory_opt,
            compatibility_mode: cli_settings.compatibility,
        })
    }
    
    #[allow(dead_code)]
    pub fn get_effective_target_mb(&self) -> Option<f32> {
        Some(self.target_size.as_mb())
    }
}

impl Default for CompressionSettings {
    fn default() -> Self {
        Self {
            target_size: TargetSize::Size10MB,
            estimated_size_mb: None,
            hardware_encoder: HardwareEncoder::Software,
            enable_hardware_accel: true,
            cuda_device_id: None,
            hardware_preset: HardwarePreset::Medium,
            hardware_quality: HardwareQuality::Auto,
            force_software_fallback: false,
            memory_optimization: false,
            compatibility_mode: true,  // Default to true for maximum compatibility
        }
    }
}
