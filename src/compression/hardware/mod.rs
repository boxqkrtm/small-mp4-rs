pub mod detection;
pub mod cuda;
pub mod amd;
pub mod intel;
pub mod platform;
pub mod fallback;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{info, warn, error, debug};

pub use detection::detect_hardware_capabilities;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HardwareEncoder {
    // NVIDIA NVENC
    NvencH264,
    NvencH265,
    NvencAV1,
    
    // AMD VCE
    AmfH264,
    AmfH265,
    
    // Intel QuickSync
    QsvH264,
    QsvH265,
    QsvAV1,
    
    // Platform-specific
    Vaapi,          // Linux
    VideoToolbox,   // macOS
    
    // Software fallback
    Software,
}

impl HardwareEncoder {
    pub fn display_name(&self) -> &'static str {
        match self {
            HardwareEncoder::NvencH264 => "NVIDIA NVENC H.264",
            HardwareEncoder::NvencH265 => "NVIDIA NVENC H.265/HEVC",
            HardwareEncoder::NvencAV1 => "NVIDIA NVENC AV1",
            HardwareEncoder::AmfH264 => "AMD VCE H.264",
            HardwareEncoder::AmfH265 => "AMD VCE H.265/HEVC",
            HardwareEncoder::QsvH264 => "Intel QuickSync H.264",
            HardwareEncoder::QsvH265 => "Intel QuickSync H.265/HEVC",
            HardwareEncoder::QsvAV1 => "Intel QuickSync AV1",
            HardwareEncoder::Vaapi => "VAAPI (Linux)",
            HardwareEncoder::VideoToolbox => "VideoToolbox (macOS)",
            HardwareEncoder::Software => "Software (CPU)",
        }
    }
    
    pub fn codec_name(&self) -> &'static str {
        match self {
            HardwareEncoder::NvencH264 => "h264_nvenc",
            HardwareEncoder::NvencH265 => "hevc_nvenc", 
            HardwareEncoder::NvencAV1 => "av1_nvenc",
            HardwareEncoder::AmfH264 => "h264_amf",
            HardwareEncoder::AmfH265 => "hevc_amf",
            HardwareEncoder::QsvH264 => "h264_qsv",
            HardwareEncoder::QsvH265 => "hevc_qsv",
            HardwareEncoder::QsvAV1 => "av1_qsv",
            HardwareEncoder::Vaapi => "h264_vaapi",
            HardwareEncoder::VideoToolbox => "h264_videotoolbox",
            HardwareEncoder::Software => "libx264",
        }
    }
    
    pub fn vendor(&self) -> &'static str {
        match self {
            HardwareEncoder::NvencH264 | HardwareEncoder::NvencH265 | HardwareEncoder::NvencAV1 => "NVIDIA",
            HardwareEncoder::AmfH264 | HardwareEncoder::AmfH265 => "AMD",
            HardwareEncoder::QsvH264 | HardwareEncoder::QsvH265 | HardwareEncoder::QsvAV1 => "Intel",
            HardwareEncoder::Vaapi => "Linux",
            HardwareEncoder::VideoToolbox => "Apple",
            HardwareEncoder::Software => "Software",
        }
    }
    
    pub fn is_hardware_accelerated(&self) -> bool {
        !matches!(self, HardwareEncoder::Software)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HardwarePreset {
    UltraFast,  // p1 - Fastest encoding, lower quality
    Faster,     // p2 - Fast encoding  
    Fast,       // p3 - Balanced speed/quality
    Medium,     // p4 - Default balanced
    Slow,       // p5 - Better quality
    Slower,     // p6 - High quality
    Highest,    // p7 - Maximum quality
}

impl HardwarePreset {
    pub fn nvenc_preset(&self) -> &'static str {
        match self {
            HardwarePreset::UltraFast => "p1",
            HardwarePreset::Faster => "p2",
            HardwarePreset::Fast => "p3",
            HardwarePreset::Medium => "p4",
            HardwarePreset::Slow => "p5",
            HardwarePreset::Slower => "p6",
            HardwarePreset::Highest => "p7",
        }
    }
    
    pub fn software_preset(&self) -> &'static str {
        match self {
            HardwarePreset::UltraFast => "ultrafast",
            HardwarePreset::Faster => "faster",
            HardwarePreset::Fast => "fast",
            HardwarePreset::Medium => "medium",
            HardwarePreset::Slow => "slow",
            HardwarePreset::Slower => "slower",
            HardwarePreset::Highest => "veryslow",
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            HardwarePreset::UltraFast => "Ultra Fast",
            HardwarePreset::Faster => "Faster",
            HardwarePreset::Fast => "Fast",
            HardwarePreset::Medium => "Medium",
            HardwarePreset::Slow => "Slow",
            HardwarePreset::Slower => "Slower",
            HardwarePreset::Highest => "Highest Quality",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HardwareQuality {
    Auto,       // Let hardware decide
    Constant,   // CRF-like constant quality
    Variable,   // Variable bitrate 
    Constrained, // Constrained variable bitrate
}

impl HardwareQuality {
    pub fn nvenc_rc_mode(&self) -> &'static str {
        match self {
            HardwareQuality::Auto => "vbr",
            HardwareQuality::Constant => "constqp",
            HardwareQuality::Variable => "vbr",
            HardwareQuality::Constrained => "cbr",
        }
    }
}

#[derive(Debug, Clone)]
pub struct HardwareCapabilities {
    pub available_encoders: Vec<HardwareEncoder>,
    pub cuda_devices: Vec<CudaDevice>,
    pub opencl_devices: Vec<OpenCLDevice>,
    pub preferred_encoder: Option<HardwareEncoder>,
    pub memory_usage_mb: u64,
    pub encoding_speed_multiplier: f32,
    pub encoder_performance: HashMap<HardwareEncoder, f32>,
}

#[derive(Debug, Clone)]
pub struct CudaDevice {
    pub id: u32,
    pub name: String,
    pub compute_capability: (u32, u32),
    pub memory_mb: u64,
    pub nvenc_support: bool,
    pub max_concurrent_sessions: u32,
}

#[derive(Debug, Clone)]
pub struct OpenCLDevice {
    pub id: u32,
    pub name: String,
    pub vendor: String,
    pub device_type: String,
    pub memory_mb: u64,
}

impl HardwareCapabilities {
    pub async fn detect() -> Result<Self> {
        let mut capabilities = HardwareCapabilities {
            available_encoders: Vec::new(),
            cuda_devices: Vec::new(),
            opencl_devices: Vec::new(),
            preferred_encoder: None,
            memory_usage_mb: 0,
            encoding_speed_multiplier: 1.0,
            encoder_performance: HashMap::new(),
        };
        
        // Detect NVIDIA CUDA/NVENC
        if let Ok(cuda_info) = cuda::detect_cuda_capabilities().await {
            capabilities.cuda_devices = cuda_info.devices;
            capabilities.available_encoders.extend(cuda_info.encoders);
            info!("CUDA detection successful: {} devices", capabilities.cuda_devices.len());
        } else {
            debug!("CUDA not available or detection failed");
        }
        
        // Detect AMD VCE
        if let Ok(amd_encoders) = amd::detect_amd_vce().await {
            capabilities.available_encoders.extend(amd_encoders);
            info!("AMD VCE detection successful");
        } else {
            debug!("AMD VCE not available or detection failed");
        }
        
        // Detect Intel QuickSync
        if let Ok(intel_encoders) = intel::detect_intel_quicksync().await {
            capabilities.available_encoders.extend(intel_encoders);
            info!("Intel QuickSync detection successful");
        } else {
            debug!("Intel QuickSync not available or detection failed");
        }
        
        // Platform-specific detection
        #[cfg(target_os = "linux")]
        if platform::detect_vaapi_support().await {
            capabilities.available_encoders.push(HardwareEncoder::Vaapi);
            info!("VAAPI support detected");
        }
        
        #[cfg(target_os = "macos")]
        if platform::detect_videotoolbox_support().await {
            capabilities.available_encoders.push(HardwareEncoder::VideoToolbox);
            info!("VideoToolbox support detected");
        }
        
        // Always include software encoding as fallback
        capabilities.available_encoders.push(HardwareEncoder::Software);
        
        // Calculate performance metrics for each encoder
        capabilities.calculate_performance_metrics();
        
        // Select preferred encoder based on performance benchmarks
        capabilities.preferred_encoder = capabilities.select_optimal_encoder();
        
        Ok(capabilities)
    }
    
    pub fn software_only() -> Self {
        HardwareCapabilities {
            available_encoders: vec![HardwareEncoder::Software],
            cuda_devices: Vec::new(),
            opencl_devices: Vec::new(),
            preferred_encoder: Some(HardwareEncoder::Software),
            memory_usage_mb: 0,
            encoding_speed_multiplier: 1.0,
            encoder_performance: HashMap::new(),
        }
    }
    
    pub fn speed_improvement(&self, encoder: &HardwareEncoder) -> f32 {
        self.encoder_performance.get(encoder).copied().unwrap_or_else(|| {
            match encoder {
                HardwareEncoder::NvencH264 | HardwareEncoder::NvencH265 => 8.0,
                HardwareEncoder::NvencAV1 => 6.0,
                HardwareEncoder::AmfH264 | HardwareEncoder::AmfH265 => 5.5,
                HardwareEncoder::QsvH264 | HardwareEncoder::QsvH265 => 7.0,
                HardwareEncoder::QsvAV1 => 5.0,
                HardwareEncoder::Vaapi => 4.0,
                HardwareEncoder::VideoToolbox => 6.0,
                HardwareEncoder::Software => 1.0,
            }
        })
    }
    
    fn calculate_performance_metrics(&mut self) {
        // Calculate performance metrics for each available encoder
        for encoder in &self.available_encoders {
            let performance = match encoder {
                HardwareEncoder::NvencH264 | HardwareEncoder::NvencH265 => {
                    // NVENC performance depends on GPU generation
                    if let Some(best_gpu) = self.cuda_devices.iter().max_by_key(|d| d.compute_capability.0 * 10 + d.compute_capability.1) {
                        match best_gpu.compute_capability.0 {
                            8..=9 => 12.0, // RTX 30xx/40xx series
                            7 => 10.0,     // RTX 20xx series
                            6 => 8.0,      // GTX 10xx series
                            _ => 6.0,      // Older GPUs
                        }
                    } else {
                        8.0
                    }
                },
                HardwareEncoder::NvencAV1 => 8.0, // AV1 is newer, slightly slower
                HardwareEncoder::AmfH264 | HardwareEncoder::AmfH265 => 6.0,
                HardwareEncoder::QsvH264 | HardwareEncoder::QsvH265 => 7.5,
                HardwareEncoder::QsvAV1 => 6.0,
                HardwareEncoder::Vaapi => 4.5,
                HardwareEncoder::VideoToolbox => 7.0,
                HardwareEncoder::Software => 1.0,
            };
            
            self.encoder_performance.insert(*encoder, performance);
        }
    }
    
    fn select_optimal_encoder(&self) -> Option<HardwareEncoder> {
        // Prioritize encoders by performance, availability, and reliability
        let priority_order = [
            HardwareEncoder::NvencH265,     // Best quality/speed balance
            HardwareEncoder::NvencH264,     // Most compatible
            HardwareEncoder::QsvH265,       // Good Intel alternative
            HardwareEncoder::QsvH264,       // Intel compatibility
            HardwareEncoder::VideoToolbox,  // macOS native
            HardwareEncoder::AmfH265,       // AMD modern
            HardwareEncoder::AmfH264,       // AMD compatible
            HardwareEncoder::Vaapi,         // Linux generic
            HardwareEncoder::NvencAV1,      // Future-proof but newer
            HardwareEncoder::QsvAV1,        // Intel AV1
            HardwareEncoder::Software,      // Always available fallback
        ];
        
        for encoder in priority_order {
            if self.available_encoders.contains(&encoder) {
                return Some(encoder);
            }
        }
        
        // Fallback to software
        Some(HardwareEncoder::Software)
    }
    
    pub fn has_cuda(&self) -> bool {
        !self.cuda_devices.is_empty()
    }
    
    pub fn get_best_cuda_device(&self) -> Option<&CudaDevice> {
        self.cuda_devices.iter()
            .filter(|d| d.nvenc_support)
            .max_by_key(|d| (d.compute_capability.0, d.compute_capability.1, d.memory_mb))
    }
}