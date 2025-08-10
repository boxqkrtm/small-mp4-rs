#![allow(dead_code)]
use anyhow::Result;
use anyhow::anyhow;
use log::{debug, info, warn};
use sysinfo::System;
use std::process::Command;

use super::{HardwareEncoder, CudaDevice};

pub struct CudaInfo {
    pub devices: Vec<CudaDevice>,
    pub encoders: Vec<HardwareEncoder>,
}

pub async fn detect_cuda_capabilities() -> Result<CudaInfo> {
    debug!("Starting CUDA capability detection");
    
    // First, check if FFmpeg can see CUDA hardware acceleration
    let cuda_available = check_cuda_in_ffmpeg();
    
    if !cuda_available {
        debug!("CUDA hardware acceleration not available in FFmpeg");
        return Err(anyhow!("CUDA not available in FFmpeg"));
    }
    
    info!("CUDA hardware acceleration detected in FFmpeg");
    
    // Try to query CUDA devices using nvidia-smi if available
    let cuda_devices = match query_cuda_devices_nvidia_smi().await {
        Ok(devices) => {
            info!("CUDA device detection via nvidia-smi successful: {} devices", devices.len());
            devices
        },
        Err(e) => {
            warn!("nvidia-smi detection failed: {}, trying alternative method", e);
            query_cuda_devices_fallback().await?
        }
    };
    
    // Determine available NVENC encoders based on detected devices
    let encoders = determine_nvenc_encoders(&cuda_devices);
    
    info!("CUDA detection complete: {} devices, {} encoders", cuda_devices.len(), encoders.len());
    
    Ok(CudaInfo {
        devices: cuda_devices,
        encoders,
    })
}

async fn query_cuda_devices_nvidia_smi() -> Result<Vec<CudaDevice>> {
    debug!("Querying CUDA devices via nvidia-smi");
    
    // Try to run nvidia-smi to get GPU information
    let output = Command::new("nvidia-smi")
        .args(&[
            "--query-gpu=index,name,compute_cap,memory.total,encoder.max_sessions",
            "--format=csv,noheader,nounits"
        ])
        .output()
        .map_err(|e| anyhow!("Failed to run nvidia-smi: {}", e))?;
    
    if !output.status.success() {
        return Err(anyhow!("nvidia-smi failed with status: {}", output.status));
    }
    
    let output_str = String::from_utf8(output.stdout)
        .map_err(|e| anyhow!("Invalid UTF-8 in nvidia-smi output: {}", e))?;
    
    let mut devices = Vec::new();
    
    for line in output_str.lines() {
        if line.trim().is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() < 4 {
            warn!("Unexpected nvidia-smi output format: {}", line);
            continue;
        }
        
        let device = parse_nvidia_smi_line(&parts)?;
        devices.push(device);
    }
    
    Ok(devices)
}

fn parse_nvidia_smi_line(parts: &[&str]) -> Result<CudaDevice> {
    let id: u32 = parts[0].parse()
        .map_err(|e| anyhow!("Failed to parse GPU index: {}", e))?;
    
    let name = parts[1].to_string();
    
    // Parse compute capability (e.g., "8.6" -> (8, 6))
    let compute_cap_parts: Vec<&str> = parts[2].split('.').collect();
    let compute_capability = if compute_cap_parts.len() == 2 {
        let major: u32 = compute_cap_parts[0].parse().unwrap_or(0);
        let minor: u32 = compute_cap_parts[1].parse().unwrap_or(0);
        (major, minor)
    } else {
        (0, 0)
    };
    
    // Parse memory in MB
    let memory_mb: u64 = parts[3].parse().unwrap_or(0);
    
    // Parse max concurrent sessions (may not be available on all GPUs)
    let max_concurrent_sessions: u32 = if parts.len() > 4 && !parts[4].is_empty() && parts[4] != "[Not Supported]" {
        parts[4].parse().unwrap_or(2) // Default to 2 if parsing fails
    } else {
        estimate_max_concurrent_sessions(compute_capability)
    };
    
    // Determine NVENC support based on compute capability
    let nvenc_support = compute_capability.0 >= 6; // NVENC supported on Pascal and newer
    
    Ok(CudaDevice {
        id,
        name,
        compute_capability,
        memory_mb,
        nvenc_support,
        max_concurrent_sessions,
    })
}

async fn query_cuda_devices_fallback() -> Result<Vec<CudaDevice>> {
    debug!("Using fallback CUDA device detection");
    
    // If nvidia-smi is not available, try to infer from system information
    // This is a basic fallback that assumes common scenarios
    
    let _system = System::new_all();
    
    // Look for NVIDIA GPUs in system info
    // This is a simplified approach - in a real implementation you might want to use
    // proper CUDA runtime APIs or other system-specific methods
    
    // For now, create a dummy device if we know CUDA is available from FFmpeg
    // but can't get detailed info
    let dummy_device = CudaDevice {
        id: 0,
        name: "Unknown NVIDIA GPU".to_string(),
        compute_capability: (6, 0), // Assume Pascal or newer for NVENC support
        memory_mb: 4096, // Conservative estimate
        nvenc_support: true,
        max_concurrent_sessions: 2,
    };
    
    info!("Fallback CUDA detection created dummy device");
    Ok(vec![dummy_device])
}

fn determine_nvenc_encoders(cuda_devices: &[CudaDevice]) -> Vec<HardwareEncoder> {
    let mut encoders = Vec::new();
    
    // Check if we have any NVENC-capable devices
    let has_nvenc = cuda_devices.iter().any(|d| d.nvenc_support);
    
    if !has_nvenc {
        debug!("No NVENC-capable devices found");
        return encoders;
    }
    
    // Determine available encoders based on compute capability
    let best_compute_capability = cuda_devices.iter()
        .filter(|d| d.nvenc_support)
        .map(|d| d.compute_capability)
        .max()
        .unwrap_or((0, 0));
    
    // H.264 NVENC is available on all NVENC-capable GPUs
    encoders.push(HardwareEncoder::NvencH264);
    
    // H.265/HEVC NVENC is available on Maxwell 2nd gen (GM20x) and newer
    if best_compute_capability >= (5, 2) {
        encoders.push(HardwareEncoder::NvencH265);
    }
    
    // AV1 NVENC is available on Ada Lovelace (RTX 40 series) and newer
    if best_compute_capability >= (8, 9) {
        encoders.push(HardwareEncoder::NvencAV1);
    }
    
    info!("Determined NVENC encoders based on compute capability {:?}: {:?}", 
          best_compute_capability, encoders);
    
    encoders
}

fn estimate_max_concurrent_sessions(compute_capability: (u32, u32)) -> u32 {
    // Estimate based on GPU generation
    match compute_capability.0 {
        8..=9 => 5,  // RTX 30xx/40xx series - higher limits
        7 => 3,      // RTX 20xx series - moderate limits
        6 => 2,      // GTX 10xx series - basic limits
        _ => 1,      // Older or unknown - conservative
    }
}

/// Check if a specific CUDA device supports a given encoder
pub fn device_supports_encoder(device: &CudaDevice, encoder: &HardwareEncoder) -> bool {
    if !device.nvenc_support {
        return false;
    }
    
    match encoder {
        HardwareEncoder::NvencH264 => true, // Available on all NVENC GPUs
        HardwareEncoder::NvencH265 => device.compute_capability >= (5, 2),
        HardwareEncoder::NvencAV1 => device.compute_capability >= (8, 9),
        _ => false,
    }
}

/// Get optimal NVENC settings for a given device and encoder
pub fn get_optimal_nvenc_settings(device: &CudaDevice, encoder: &HardwareEncoder) -> Vec<(&'static str, String)> {
    let mut settings = Vec::new();
    
    // Basic NVENC settings
    settings.push(("hwaccel", "cuda".to_string()));
    settings.push(("hwaccel_output_format", "cuda".to_string()));
    settings.push(("hwaccel_device", device.id.to_string()));
    
    // Encoder-specific settings
    match encoder {
        HardwareEncoder::NvencH264 => {
            settings.push(("c:v", "h264_nvenc".to_string()));
            
            // Optimize based on GPU generation
            if device.compute_capability >= (8, 0) {
                // RTX 30xx/40xx series optimizations
                settings.push(("preset", "p4".to_string())); // Balanced preset
                settings.push(("tune", "hq".to_string())); // High quality tuning
                settings.push(("rc", "vbr".to_string())); // Variable bitrate
                settings.push(("multipass", "fullres".to_string())); // Two-pass encoding
            } else if device.compute_capability >= (7, 0) {
                // RTX 20xx series
                settings.push(("preset", "medium".to_string()));
                settings.push(("tune", "hq".to_string()));
                settings.push(("rc", "vbr".to_string()));
            } else {
                // Older GPUs - basic settings
                settings.push(("preset", "medium".to_string()));
                settings.push(("rc", "cbr".to_string()));
            }
        },
        HardwareEncoder::NvencH265 => {
            settings.push(("c:v", "hevc_nvenc".to_string()));
            
            // Similar optimizations for HEVC
            if device.compute_capability >= (8, 0) {
                settings.push(("preset", "p4".to_string()));
                settings.push(("tune", "hq".to_string()));
                settings.push(("rc", "vbr".to_string()));
                settings.push(("multipass", "fullres".to_string()));
            } else {
                settings.push(("preset", "medium".to_string()));
                settings.push(("tune", "hq".to_string()));
                settings.push(("rc", "vbr".to_string()));
            }
        },
        HardwareEncoder::NvencAV1 => {
            settings.push(("c:v", "av1_nvenc".to_string()));
            // AV1 NVENC is only on newest GPUs, use high-quality settings
            settings.push(("preset", "p4".to_string()));
            settings.push(("tune", "hq".to_string()));
            settings.push(("rc", "vbr".to_string()));
        },
        _ => {
            // Non-NVENC encoder
            warn!("get_optimal_nvenc_settings called with non-NVENC encoder: {:?}", encoder);
        }
    }
    
    settings
}

fn check_cuda_in_ffmpeg() -> bool {
    // Check if CUDA is available in FFmpeg
    let output = match Command::new("ffmpeg")
        .arg("-hwaccels")
        .output() {
        Ok(output) => output,
        Err(_) => return false,
    };
    
    if !output.status.success() {
        return false;
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().any(|line| line.trim() == "cuda")
}
