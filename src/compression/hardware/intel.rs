#![allow(dead_code)]
use anyhow::{Result, anyhow};
use log::{debug, info, warn};
use std::process::Command;

use super::HardwareEncoder;

pub async fn detect_intel_quicksync() -> Result<Vec<HardwareEncoder>> {
    debug!("Starting Intel QuickSync detection");
    
    let mut encoders = Vec::new();
    
    if detect_intel_gpu().await? {
        info!("Intel GPU detected, checking QuickSync support");
        
        // Intel QuickSync H.264 is widely supported (Sandy Bridge and newer)
        encoders.push(HardwareEncoder::QsvH264);
        
        // Intel QuickSync H.265 is supported on Skylake and newer
        if check_hevc_support().await {
            encoders.push(HardwareEncoder::QsvH265);
        }
        
        // Intel QuickSync AV1 is supported on Arc GPUs and some newer integrated GPUs
        if check_av1_support().await {
            encoders.push(HardwareEncoder::QsvAV1);
        }
        
        info!("Intel QuickSync detection complete: {:?}", encoders);
    } else {
        debug!("No Intel GPU detected or QuickSync not supported");
    }
    
    Ok(encoders)
}

async fn detect_intel_gpu() -> Result<bool> {
    // Try multiple methods to detect Intel GPU
    
    #[cfg(target_os = "linux")]
    {
        if let Ok(has_intel) = check_intel_gpu_linux().await {
            return Ok(has_intel);
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(has_intel) = check_intel_gpu_windows().await {
            return Ok(has_intel);
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        if let Ok(has_intel) = check_intel_gpu_macos().await {
            return Ok(has_intel);
        }
    }
    
    Ok(false)
}

#[cfg(target_os = "linux")]
async fn check_intel_gpu_linux() -> Result<bool> {
    debug!("Checking for Intel GPU on Linux");
    
    // Method 1: Check lspci for Intel GPUs
    if let Ok(output) = Command::new("lspci").output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let has_intel = output_str.lines().any(|line| {
            line.to_lowercase().contains("intel") && 
            (line.to_lowercase().contains("vga") || 
             line.to_lowercase().contains("display") ||
             line.to_lowercase().contains("3d") ||
             line.to_lowercase().contains("integrated graphics"))
        });
        
        if has_intel {
            info!("Intel GPU detected via lspci");
            return Ok(true);
        }
    }
    
    // Method 2: Check for Intel GPU in /sys/class/drm
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            if let Ok(device_name) = std::fs::read_to_string(entry.path().join("device/vendor")) {
                // Intel vendor ID is 0x8086
                if device_name.trim() == "0x8086" {
                    info!("Intel GPU detected via /sys/class/drm");
                    return Ok(true);
                }
            }
        }
    }
    
    // Method 3: Check for i915 driver (Intel graphics driver)
    if std::path::Path::new("/sys/module/i915").exists() {
        info!("Intel GPU detected via i915 driver");
        return Ok(true);
    }
    
    debug!("No Intel GPU detected on Linux");
    Ok(false)
}

#[cfg(target_os = "windows")]
async fn check_intel_gpu_windows() -> Result<bool> {
    debug!("Checking for Intel GPU on Windows");
    
    let output = Command::new("wmic")
        .args(&["path", "win32_VideoController", "get", "name", "/format:list"])
        .output()
        .map_err(|e| anyhow!("Failed to run wmic: {}", e))?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let has_intel = output_str.lines().any(|line| {
        let line_lower = line.to_lowercase();
        line_lower.contains("intel") && 
        (line_lower.contains("hd") || 
         line_lower.contains("iris") || 
         line_lower.contains("uhd") ||
         line_lower.contains("arc") ||
         line_lower.contains("graphics"))
    });
    
    if has_intel {
        info!("Intel GPU detected on Windows via wmic");
    } else {
        debug!("No Intel GPU detected on Windows");
    }
    
    Ok(has_intel)
}

#[cfg(target_os = "macos")]
async fn check_intel_gpu_macos() -> Result<bool> {
    debug!("Checking for Intel GPU on macOS");
    
    let output = Command::new("system_profiler")
        .args(&["SPDisplaysDataType", "-xml"])
        .output()
        .map_err(|e| anyhow!("Failed to run system_profiler: {}", e))?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let has_intel = output_str.to_lowercase().contains("intel") &&
                    (output_str.to_lowercase().contains("hd") || 
                     output_str.to_lowercase().contains("iris") ||
                     output_str.to_lowercase().contains("uhd"));
    
    if has_intel {
        info!("Intel GPU detected on macOS via system_profiler");
    } else {
        debug!("No Intel GPU detected on macOS");
    }
    
    Ok(has_intel)
}

async fn check_hevc_support() -> bool {
    // Intel QuickSync HEVC encoding is supported on:
    // - Skylake (6th gen Core) and newer for integrated graphics
    // - Some Broadwell (5th gen) processors
    
    // For simplicity, assume HEVC support if we detect Intel GPU
    // In practice, you would check the specific GPU model/generation
    debug!("Assuming HEVC support for detected Intel GPU");
    true
}

async fn check_av1_support() -> bool {
    // Intel QuickSync AV1 encoding is supported on:
    // - Intel Arc discrete GPUs
    // - Some 12th gen and newer integrated graphics
    
    // This is harder to detect without specific hardware queries
    // For now, we'll be conservative and assume no AV1 support
    debug!("Conservative approach: assuming no AV1 support");
    false
}

/// Get optimal Intel QuickSync settings for encoding
pub fn get_optimal_intel_settings(encoder: &HardwareEncoder) -> Vec<(&'static str, String)> {
    let mut settings = Vec::new();
    
    match encoder {
        HardwareEncoder::QsvH264 => {
            settings.push(("c:v", "h264_qsv".to_string()));
            settings.push(("preset", "medium".to_string()));
            settings.push(("look_ahead", "1".to_string())); // Enable look-ahead
            settings.push(("look_ahead_depth", "15".to_string()));
            settings.push(("global_quality", "23".to_string())); // Similar to CRF
        },
        HardwareEncoder::QsvH265 => {
            settings.push(("c:v", "hevc_qsv".to_string()));
            settings.push(("preset", "medium".to_string()));
            settings.push(("look_ahead", "1".to_string()));
            settings.push(("look_ahead_depth", "15".to_string()));
            settings.push(("global_quality", "25".to_string())); // HEVC can use slightly higher
        },
        HardwareEncoder::QsvAV1 => {
            settings.push(("c:v", "av1_qsv".to_string()));
            settings.push(("preset", "medium".to_string()));
            settings.push(("global_quality", "27".to_string()));
        },
        _ => {
            warn!("get_optimal_intel_settings called with non-QSV encoder: {:?}", encoder);
        }
    }
    
    settings
}

/// Check if the system supports Intel hardware acceleration
pub async fn is_intel_acceleration_available() -> bool {
    match detect_intel_quicksync().await {
        Ok(encoders) => !encoders.is_empty(),
        Err(_) => false,
    }
}

/// Get information about detected Intel GPU(s)
pub async fn get_intel_gpu_info() -> Result<Vec<IntelGpuInfo>> {
    let mut gpus = Vec::new();
    
    #[cfg(target_os = "linux")]
    {
        if let Ok(linux_gpus) = get_intel_gpu_info_linux().await {
            gpus.extend(linux_gpus);
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(windows_gpus) = get_intel_gpu_info_windows().await {
            gpus.extend(windows_gpus);
        }
    }
    
    // If no specific info found but we detected Intel GPU, create generic info
    if gpus.is_empty() && detect_intel_gpu().await? {
        gpus.push(IntelGpuInfo {
            name: "Intel GPU".to_string(),
            generation: "Unknown".to_string(),
            quicksync_version: "Unknown".to_string(),
            execution_units: 0,
        });
    }
    
    Ok(gpus)
}

#[derive(Debug, Clone)]
pub struct IntelGpuInfo {
    pub name: String,
    pub generation: String,
    pub quicksync_version: String,
    pub execution_units: u32,
}

#[cfg(target_os = "linux")]
async fn get_intel_gpu_info_linux() -> Result<Vec<IntelGpuInfo>> {
    let mut gpus = Vec::new();
    
    if let Ok(output) = Command::new("lspci").args(&["-v"]).output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for section in output_str.split("

") {
            if section.to_lowercase().contains("intel") && 
               (section.to_lowercase().contains("vga") || 
                section.to_lowercase().contains("display")) {
                
                if let Some(first_line) = section.lines().next() {
                    if let Some(name_start) = first_line.find(": ") {
                        let name = first_line[name_start + 2..].to_string();
                        
                        // Try to determine generation from the name
                        let generation = determine_intel_generation(&name);
                        let quicksync_version = determine_quicksync_version(&generation);
                        
                        gpus.push(IntelGpuInfo {
                            name,
                            generation,
                            quicksync_version,
                            execution_units: 0, // Would need additional detection
                        });
                    }
                }
            }
        }
    }
    
    Ok(gpus)
}

#[cfg(target_os = "windows")]
async fn get_intel_gpu_info_windows() -> Result<Vec<IntelGpuInfo>> {
    let mut gpus = Vec::new();
    
    if let Ok(output) = Command::new("wmic")
        .args(&["path", "win32_VideoController", "where", "name like '%Intel%'", 
               "get", "name", "/format:list"])
        .output() {
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines() {
            if line.starts_with("Name=") && !line[5..].trim().is_empty() {
                let name = line[5..].trim().to_string();
                let generation = determine_intel_generation(&name);
                let quicksync_version = determine_quicksync_version(&generation);
                
                gpus.push(IntelGpuInfo {
                    name,
                    generation,
                    quicksync_version,
                    execution_units: 0,
                });
            }
        }
    }
    
    Ok(gpus)
}

fn determine_intel_generation(gpu_name: &str) -> String {
    let name_lower = gpu_name.to_lowercase();
    
    // Try to determine Intel GPU generation from name
    if name_lower.contains("arc") {
        "Arc".to_string()
    } else if name_lower.contains("xe") {
        "Xe".to_string()
    } else if name_lower.contains("uhd") {
        // UHD Graphics are typically 8th gen and newer
        if name_lower.contains("630") || name_lower.contains("620") {
            "Coffee Lake".to_string()
        } else if name_lower.contains("750") {
            "Ice Lake".to_string()
        } else {
            "Modern".to_string()
        }
    } else if name_lower.contains("hd") {
        // HD Graphics are older
        if name_lower.contains("2500") || name_lower.contains("3000") {
            "Sandy Bridge".to_string()
        } else if name_lower.contains("4000") {
            "Ivy Bridge".to_string()
        } else if name_lower.contains("4400") || name_lower.contains("4600") {
            "Haswell".to_string()
        } else if name_lower.contains("5500") || name_lower.contains("6000") {
            "Broadwell".to_string()
        } else if name_lower.contains("530") || name_lower.contains("540") {
            "Skylake".to_string()
        } else {
            "Legacy".to_string()
        }
    } else if name_lower.contains("iris") {
        "Modern".to_string()
    } else {
        "Unknown".to_string()
    }
}

fn determine_quicksync_version(generation: &str) -> String {
    match generation {
        "Sandy Bridge" => "QuickSync 1.0".to_string(),
        "Ivy Bridge" => "QuickSync 2.0".to_string(),
        "Haswell" => "QuickSync 3.0".to_string(),
        "Broadwell" => "QuickSync 4.0".to_string(),
        "Skylake" => "QuickSync 5.0".to_string(),
        "Coffee Lake" => "QuickSync 6.0".to_string(),
        "Ice Lake" => "QuickSync 7.0".to_string(),
        "Arc" | "Xe" => "QuickSync 8.0+".to_string(),
        "Modern" => "QuickSync 6.0+".to_string(),
        _ => "Unknown".to_string(),
    }
}
