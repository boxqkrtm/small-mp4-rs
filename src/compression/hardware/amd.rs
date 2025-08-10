#![allow(dead_code)]
use anyhow::{Result, anyhow};
use log::{debug, info, warn};
use std::process::Command;

use super::HardwareEncoder;

pub async fn detect_amd_vce() -> Result<Vec<HardwareEncoder>> {
    debug!("Starting AMD VCE detection");
    
    let mut encoders = Vec::new();
    
    // Try to detect AMD GPUs and VCE support
    if detect_amd_gpu().await? {
        info!("AMD GPU detected, checking VCE support");
        
        // AMD VCE H.264 is widely supported
        encoders.push(HardwareEncoder::AmfH264);
        
        // AMD VCE H.265 is supported on newer GPUs (GCN 3.0+, Polaris and newer)
        if check_hevc_support().await {
            encoders.push(HardwareEncoder::AmfH265);
        }
        
        info!("AMD VCE detection complete: {:?}", encoders);
    } else {
        debug!("No AMD GPU detected or VCE not supported");
    }
    
    Ok(encoders)
}

async fn detect_amd_gpu() -> Result<bool> {
    // Try multiple methods to detect AMD GPU
    
    // Method 1: Check for AMD GPU via system commands
    #[cfg(target_os = "linux")]
    {
        if let Ok(has_amd) = check_amd_gpu_linux().await {
            return Ok(has_amd);
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(has_amd) = check_amd_gpu_windows().await {
            return Ok(has_amd);
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        if let Ok(has_amd) = check_amd_gpu_macos().await {
            return Ok(has_amd);
        }
    }
    
    // Fallback: assume no AMD GPU if we can't detect
    Ok(false)
}

#[cfg(target_os = "linux")]
async fn check_amd_gpu_linux() -> Result<bool> {
    debug!("Checking for AMD GPU on Linux");
    
    // Method 1: Check lspci for AMD GPUs
    if let Ok(output) = Command::new("lspci").output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let has_amd = output_str.lines().any(|line| {
            line.to_lowercase().contains("amd") && 
            (line.to_lowercase().contains("vga") || 
             line.to_lowercase().contains("display") ||
             line.to_lowercase().contains("3d"))
        });
        
        if has_amd {
            info!("AMD GPU detected via lspci");
            return Ok(true);
        }
    }
    
    // Method 2: Check /proc/driver/amdgpu if available
    if std::path::Path::new("/proc/driver/amdgpu").exists() {
        info!("AMD GPU detected via /proc/driver/amdgpu");
        return Ok(true);
    }
    
    // Method 3: Check for AMD GPU in /sys/class/drm
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            if let Ok(device_name) = std::fs::read_to_string(entry.path().join("device/vendor")) {
                // AMD vendor ID is 0x1002
                if device_name.trim() == "0x1002" {
                    info!("AMD GPU detected via /sys/class/drm");
                    return Ok(true);
                }
            }
        }
    }
    
    debug!("No AMD GPU detected on Linux");
    Ok(false)
}

#[cfg(target_os = "windows")]
async fn check_amd_gpu_windows() -> Result<bool> {
    debug!("Checking for AMD GPU on Windows");
    
    // Use wmic to query for AMD display adapters
    let output = Command::new("wmic")
        .args(&["path", "win32_VideoController", "get", "name", "/format:list"])
        .output()
        .map_err(|e| anyhow!("Failed to run wmic: {}", e))?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let has_amd = output_str.lines().any(|line| {
        line.to_lowercase().contains("amd") || 
        line.to_lowercase().contains("radeon") ||
        line.to_lowercase().contains("rx ")
    });
    
    if has_amd {
        info!("AMD GPU detected on Windows via wmic");
    } else {
        debug!("No AMD GPU detected on Windows");
    }
    
    Ok(has_amd)
}

#[cfg(target_os = "macos")]
async fn check_amd_gpu_macos() -> Result<bool> {
    debug!("Checking for AMD GPU on macOS");
    
    // Use system_profiler to check for AMD GPUs
    let output = Command::new("system_profiler")
        .args(&["SPDisplaysDataType", "-xml"])
        .output()
        .map_err(|e| anyhow!("Failed to run system_profiler: {}", e))?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let has_amd = output_str.to_lowercase().contains("amd") || 
                  output_str.to_lowercase().contains("radeon");
    
    if has_amd {
        info!("AMD GPU detected on macOS via system_profiler");
    } else {
        debug!("No AMD GPU detected on macOS");
    }
    
    Ok(has_amd)
}

async fn check_hevc_support() -> bool {
    // AMD VCE HEVC encoding is supported on:
    // - Polaris (RX 400/500 series) and newer
    // - Some Fiji (R9 Fury) GPUs
    // 
    // For simplicity, we'll assume HEVC support is available if we detect an AMD GPU
    // In a more sophisticated implementation, we would check the specific GPU model
    
    debug!("Assuming HEVC support for detected AMD GPU");
    true
}

/// Get optimal AMD VCE settings for encoding
pub fn get_optimal_amd_settings(encoder: &HardwareEncoder) -> Vec<(&'static str, String)> {
    let mut settings = Vec::new();
    
    match encoder {
        HardwareEncoder::AmfH264 => {
            settings.push(("c:v", "h264_amf".to_string()));
            settings.push(("quality", "speed".to_string())); // Balance quality and speed
            settings.push(("rc", "vbr_peak".to_string())); // Variable bitrate with peak
            settings.push(("qmin", "18".to_string()));
            settings.push(("qmax", "30".to_string()));
        },
        HardwareEncoder::AmfH265 => {
            settings.push(("c:v", "hevc_amf".to_string()));
            settings.push(("quality", "speed".to_string()));
            settings.push(("rc", "vbr_peak".to_string()));
            settings.push(("qmin", "20".to_string()));
            settings.push(("qmax", "32".to_string()));
        },
        _ => {
            warn!("get_optimal_amd_settings called with non-AMF encoder: {:?}", encoder);
        }
    }
    
    settings
}

/// Check if the system supports AMD hardware acceleration
pub async fn is_amd_acceleration_available() -> bool {
    match detect_amd_vce().await {
        Ok(encoders) => !encoders.is_empty(),
        Err(_) => false,
    }
}

/// Get information about detected AMD GPU(s)
pub async fn get_amd_gpu_info() -> Result<Vec<AmdGpuInfo>> {
    let mut gpus = Vec::new();
    
    #[cfg(target_os = "linux")]
    {
        if let Ok(linux_gpus) = get_amd_gpu_info_linux().await {
            gpus.extend(linux_gpus);
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(windows_gpus) = get_amd_gpu_info_windows().await {
            gpus.extend(windows_gpus);
        }
    }
    
    // If no specific info found but we detected AMD GPU, create generic info
    if gpus.is_empty() && detect_amd_gpu().await? {
        gpus.push(AmdGpuInfo {
            name: "AMD GPU".to_string(),
            memory_mb: 0, // Unknown
            architecture: "Unknown".to_string(),
            vce_version: "Unknown".to_string(),
        });
    }
    
    Ok(gpus)
}

#[derive(Debug, Clone)]
pub struct AmdGpuInfo {
    pub name: String,
    pub memory_mb: u64,
    pub architecture: String,
    pub vce_version: String,
}

#[cfg(target_os = "linux")]
async fn get_amd_gpu_info_linux() -> Result<Vec<AmdGpuInfo>> {
    // Try to get AMD GPU info from various sources on Linux
    let mut gpus = Vec::new();
    
    // This is a simplified implementation
    // In practice, you might want to use proper AMD GPU management libraries
    
    if let Ok(output) = Command::new("lspci").args(&["-v"]).output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for section in output_str.split("

") {
            if section.to_lowercase().contains("amd") && 
               (section.to_lowercase().contains("vga") || 
                section.to_lowercase().contains("display")) {
                
                // Extract GPU name from lspci output
                if let Some(first_line) = section.lines().next() {
                    if let Some(name_start) = first_line.find(": ") {
                        let name = first_line[name_start + 2..].to_string();
                        
                        gpus.push(AmdGpuInfo {
                            name,
                            memory_mb: 0, // Would need additional detection
                            architecture: "Unknown".to_string(),
                            vce_version: "Unknown".to_string(),
                        });
                    }
                }
            }
        }
    }
    
    Ok(gpus)
}

#[cfg(target_os = "windows")]
async fn get_amd_gpu_info_windows() -> Result<Vec<AmdGpuInfo>> {
    let mut gpus = Vec::new();
    
    // Use wmic to get detailed GPU information
    if let Ok(output) = Command::new("wmic")
        .args(&["path", "win32_VideoController", "where", "name like '%AMD%' or name like '%Radeon%'", 
               "get", "name,AdapterRAM", "/format:list"])
        .output() {
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut current_gpu = AmdGpuInfo {
            name: String::new(),
            memory_mb: 0,
            architecture: "Unknown".to_string(),
            vce_version: "Unknown".to_string(),
        };
        
        for line in output_str.lines() {
            if line.starts_with("Name=") && !line[5..].trim().is_empty() {
                current_gpu.name = line[5..].trim().to_string();
            } else if line.starts_with("AdapterRAM=") && !line[11..].trim().is_empty() {
                if let Ok(ram_bytes) = line[11..].trim().parse::<u64>() {
                    current_gpu.memory_mb = ram_bytes / (1024 * 1024);
                }
            }
            
            // If we have a name, we have a complete GPU entry
            if !current_gpu.name.is_empty() {
                gpus.push(current_gpu.clone());
                current_gpu = AmdGpuInfo {
                    name: String::new(),
                    memory_mb: 0,
                    architecture: "Unknown".to_string(),
                    vce_version: "Unknown".to_string(),
                };
            }
        }
    }
    
    Ok(gpus)
}
