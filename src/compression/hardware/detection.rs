#![allow(dead_code)]
use anyhow::Result;
use log::{info, debug, warn};
use std::process::Command;

use super::{HardwareCapabilities, HardwareEncoder};

#[derive(Debug, Clone)]
struct HWAccelInfo {
    name: String,
}

pub async fn detect_hardware_capabilities() -> Result<HardwareCapabilities> {
    info!("Starting comprehensive hardware acceleration detection");
    
    let mut capabilities = HardwareCapabilities {
        available_encoders: Vec::new(),
        cuda_devices: Vec::new(),
        opencl_devices: Vec::new(),
        preferred_encoder: None,
        memory_usage_mb: 0,
        encoding_speed_multiplier: 1.0,
        encoder_performance: std::collections::HashMap::new(),
    };
    
    // Start with FFmpeg hardware acceleration detection
    let ffmpeg_hwaccels = detect_ffmpeg_hwaccels();
    info!("FFmpeg hardware accelerations detected: {:?}", 
          ffmpeg_hwaccels.iter().map(|h| &h.name).collect::<Vec<_>>());
    
    // Detect NVIDIA CUDA/NVENC
    if ffmpeg_hwaccels.iter().any(|h| h.name == "cuda") {
        match super::cuda::detect_cuda_capabilities().await {
            Ok(cuda_info) => {
                let encoder_count = cuda_info.encoders.len();
                capabilities.cuda_devices = cuda_info.devices;
                capabilities.available_encoders.extend(cuda_info.encoders);
                info!("CUDA detection successful: {} devices, {} encoders", 
                      capabilities.cuda_devices.len(), encoder_count);
            },
            Err(e) => {
                warn!("CUDA detection failed: {}", e);
            }
        }
    } else {
        debug!("CUDA not available in FFmpeg");
    }
    
    // Detect AMD VCE
    if is_amd_hardware_available(&ffmpeg_hwaccels) {
        match super::amd::detect_amd_vce().await {
            Ok(amd_encoders) => {
                let encoder_count = amd_encoders.len();
                capabilities.available_encoders.extend(amd_encoders);
                info!("AMD VCE detection successful: {} encoders", encoder_count);
            },
            Err(e) => {
                warn!("AMD VCE detection failed: {}", e);
            }
        }
    }
    
    // Detect Intel QuickSync
    if is_intel_hardware_available(&ffmpeg_hwaccels) {
        match super::intel::detect_intel_quicksync().await {
            Ok(intel_encoders) => {
                let encoder_count = intel_encoders.len();
                capabilities.available_encoders.extend(intel_encoders);
                info!("Intel QuickSync detection successful: {} encoders", encoder_count);
            },
            Err(e) => {
                warn!("Intel QuickSync detection failed: {}", e);
            }
        }
    }
    
    // Platform-specific detection
    #[cfg(target_os = "linux")]
    {
        if ffmpeg_hwaccels.iter().any(|h| h.name == "vaapi") {
            if super::platform::detect_vaapi_support().await {
                capabilities.available_encoders.push(HardwareEncoder::Vaapi);
                info!("VAAPI support detected and added");
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        if ffmpeg_hwaccels.iter().any(|h| h.name == "videotoolbox") {
            if super::platform::detect_videotoolbox_support().await {
                capabilities.available_encoders.push(HardwareEncoder::VideoToolbox);
                info!("VideoToolbox support detected and added");
            }
        }
    }
    
    // Always include software encoding as fallback
    capabilities.available_encoders.push(HardwareEncoder::Software);
    
    // Remove duplicates and sort by preference
    capabilities.available_encoders.sort_unstable();
    capabilities.available_encoders.dedup();
    
    // Calculate performance metrics
    capabilities.calculate_performance_metrics();
    
    // Select preferred encoder
    capabilities.preferred_encoder = capabilities.select_optimal_encoder();
    
    // Calculate overall system metrics
    calculate_system_metrics(&mut capabilities);
    
    info!("Hardware detection complete. Available encoders: {:?}", capabilities.available_encoders);
    info!("Preferred encoder: {:?}", capabilities.preferred_encoder);
    
    Ok(capabilities)
}

fn detect_ffmpeg_hwaccels() -> Vec<HWAccelInfo> {
    // Use ffmpeg to detect available hardware accelerations
    let output = match Command::new("ffmpeg")
        .arg("-hwaccels")
        .output() {
        Ok(output) => output,
        Err(_) => return Vec::new(),
    };
    
    if !output.status.success() {
        return Vec::new();
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut hwaccels = Vec::new();
    let mut found_header = false;
    
    for line in stdout.lines() {
        if line.contains("Hardware acceleration methods:") {
            found_header = true;
            continue;
        }
        
        if found_header && !line.trim().is_empty() {
            hwaccels.push(HWAccelInfo {
                name: line.trim().to_string(),
            });
        }
    }
    
    hwaccels
}

// Remove this - we're using the one from ez_ffmpeg now

fn is_amd_hardware_available(hwaccels: &[HWAccelInfo]) -> bool {
    // Check for AMD-specific hardware acceleration methods
    hwaccels.iter().any(|h| {
        h.name.contains("amf") || 
        h.name.contains("amd") ||
        h.name == "vaapi" // VAAPI can be used with AMD on Linux
    })
}

fn is_intel_hardware_available(hwaccels: &[HWAccelInfo]) -> bool {
    // Check for Intel-specific hardware acceleration methods
    hwaccels.iter().any(|h| {
        h.name.contains("qsv") || 
        h.name.contains("intel") ||
        h.name == "vaapi" || // VAAPI is commonly used with Intel
        h.name == "dxva2"    // Intel can use DXVA2 on Windows
    })
}

fn calculate_system_metrics(capabilities: &mut HardwareCapabilities) {
    // Estimate system-wide metrics based on detected hardware
    
    // Calculate memory usage estimate
    let base_memory_mb = 100; // Base FFmpeg memory usage
    let hw_memory_mb = if capabilities.has_cuda() {
        // CUDA encoding uses more GPU memory
        if let Some(best_device) = capabilities.get_best_cuda_device() {
            std::cmp::min(512, best_device.memory_mb / 8) // Use up to 1/8 of GPU memory
        } else {
            256
        }
    } else if capabilities.available_encoders.iter().any(|e| e.is_hardware_accelerated()) {
        // Other hardware encoders use less memory
        128
    } else {
        // Software encoding uses more CPU memory
        256
    };
    
    capabilities.memory_usage_mb = base_memory_mb + hw_memory_mb;
    
    // Calculate overall encoding speed multiplier
    let speed_multiplier = if let Some(preferred) = &capabilities.preferred_encoder {
        capabilities.speed_improvement(preferred)
    } else {
        1.0
    };
    
    capabilities.encoding_speed_multiplier = speed_multiplier;
    
    debug!("System metrics calculated - Memory: {}MB, Speed: {:.1}x", 
           capabilities.memory_usage_mb, capabilities.encoding_speed_multiplier);
}

/// Quick hardware detection for CLI --list-hw command
pub async fn quick_hardware_detection() -> Vec<String> {
    let mut available = Vec::new();
    
    // Get FFmpeg hardware accelerations
    let hwaccels = detect_ffmpeg_hwaccels();
    
    for accel in hwaccels {
        match accel.name.as_str() {
            "cuda" => {
                if let Ok(_) = super::cuda::detect_cuda_capabilities().await {
                    available.push("NVIDIA NVENC (CUDA)".to_string());
                }
            },
            "vaapi" => available.push("VAAPI (Linux)".to_string()),
            "videotoolbox" => available.push("VideoToolbox (macOS)".to_string()),
            "qsv" => available.push("Intel QuickSync".to_string()),
            "amf" => available.push("AMD VCE".to_string()),
            _ => {
                // Other hardware accelerations
                available.push(format!("Hardware: {}", accel.name));
            }
        }
    }
    
    // Always include software
    available.push("Software (CPU)".to_string());
    
    available
}

/// Test if a specific hardware encoder is functional
pub async fn test_encoder_functionality(encoder: &HardwareEncoder) -> Result<bool> {
    info!("Testing functionality of encoder: {:?}", encoder);
    
    // This would typically involve trying to encode a small test video
    // For now, we'll do basic availability checks
    
    match encoder {
        HardwareEncoder::Software => Ok(true), // Software always available
        
        HardwareEncoder::NvencH264 | HardwareEncoder::NvencH265 | HardwareEncoder::NvencAV1 => {
            // Test NVENC availability
            match super::cuda::detect_cuda_capabilities().await {
                Ok(cuda_info) => {
                    let has_capable_device = cuda_info.devices.iter()
                        .any(|d| super::cuda::device_supports_encoder(d, encoder));
                    Ok(has_capable_device)
                },
                Err(_) => Ok(false),
            }
        },
        
        _ => {
            // For other encoders, check if they're in our detected list
            let caps = detect_hardware_capabilities().await?;
            Ok(caps.available_encoders.contains(encoder))
        }
    }
}

/// Get recommended settings for optimal performance
pub fn get_encoder_recommendations(encoder: &HardwareEncoder) -> Vec<String> {
    match encoder {
        HardwareEncoder::NvencH264 => vec![
            "Use preset 'p4' (medium) for balanced quality/speed".to_string(),
            "Enable 'tune hq' for better quality".to_string(),
            "Use variable bitrate (VBR) for optimal size control".to_string(),
        ],
        HardwareEncoder::NvencH265 => vec![
            "HEVC provides better compression than H.264".to_string(),
            "May have slightly slower encoding than H.264".to_string(),
            "Recommended for smaller file sizes".to_string(),
        ],
        HardwareEncoder::VideoToolbox => vec![
            "Native macOS hardware acceleration".to_string(),
            "Optimized for Apple Silicon and Intel Macs".to_string(),
            "Good balance of quality and performance".to_string(),
        ],
        HardwareEncoder::Software => vec![
            "Most compatible but slowest option".to_string(),
            "Uses CPU instead of dedicated hardware".to_string(),
            "Supports all advanced encoding features".to_string(),
        ],
        _ => vec![
            "Hardware-accelerated encoding available".to_string(),
            "May provide significant speed improvements".to_string(),
        ],
    }
}
