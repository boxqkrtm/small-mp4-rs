use anyhow::Result;
use log::{debug, info};
use std::path::Path;

/// Detect VAAPI support on Linux
#[cfg(target_os = "linux")]
pub async fn detect_vaapi_support() -> bool {
    debug!("Detecting VAAPI support on Linux");
    
    // Check if VAAPI libraries are available
    let vaapi_paths = [
        "/usr/lib/x86_64-linux-gnu/libva.so",
        "/usr/lib64/libva.so",
        "/usr/lib/libva.so",
        "/lib/x86_64-linux-gnu/libva.so.2",
        "/usr/local/lib/libva.so",
    ];
    
    let has_vaapi_lib = vaapi_paths.iter().any(|path| Path::new(path).exists());
    
    if !has_vaapi_lib {
        debug!("VAAPI library not found");
        return false;
    }
    
    // Check if VAAPI devices are available
    let vaapi_devices = [
        "/dev/dri/renderD128",
        "/dev/dri/renderD129",
        "/dev/dri/card0",
        "/dev/dri/card1",
    ];
    
    let has_vaapi_device = vaapi_devices.iter().any(|path| Path::new(path).exists());
    
    if has_vaapi_device {
        info!("VAAPI support detected (library and device available)");
        true
    } else {
        debug!("VAAPI library found but no devices available");
        false
    }
}

#[cfg(not(target_os = "linux"))]
pub async fn detect_vaapi_support() -> bool {
    false // VAAPI is Linux-specific
}

/// Detect VideoToolbox support on macOS
#[cfg(target_os = "macos")]
pub async fn detect_videotoolbox_support() -> bool {
    debug!("Detecting VideoToolbox support on macOS");
    
    // VideoToolbox is available on all supported macOS versions
    // Check if the VideoToolbox framework exists
    let videotoolbox_path = "/System/Library/Frameworks/VideoToolbox.framework";
    
    if Path::new(videotoolbox_path).exists() {
        info!("VideoToolbox support detected");
        true
    } else {
        debug!("VideoToolbox framework not found");
        false
    }
}

#[cfg(not(target_os = "macos"))]
pub async fn detect_videotoolbox_support() -> bool {
    false // VideoToolbox is macOS-specific
}

/// Detect DirectX Video Acceleration (DXVA) support on Windows
#[cfg(target_os = "windows")]
pub async fn detect_dxva_support() -> bool {
    debug!("Detecting DXVA support on Windows");
    
    // DXVA is available on Windows Vista and later
    // For simplicity, we'll assume it's available if we're on Windows
    // In practice, you would check for specific DirectX components
    
    use std::process::Command;
    
    // Check if dxdiag is available (indicates DirectX is installed)
    if let Ok(output) = Command::new("where").arg("dxdiag").output() {
        if output.status.success() {
            info!("DXVA support likely available (DirectX detected)");
            return true;
        }
    }
    
    debug!("DirectX/DXVA support not detected");
    false
}

#[cfg(not(target_os = "windows"))]
pub async fn detect_dxva_support() -> bool {
    false // DXVA is Windows-specific
}

/// Get platform-specific hardware acceleration options
pub async fn get_platform_hwaccels() -> Vec<PlatformHwAccel> {
    let mut hwaccels = Vec::new();
    
    #[cfg(target_os = "linux")]
    {
        if detect_vaapi_support().await {
            hwaccels.push(PlatformHwAccel {
                name: "VAAPI".to_string(),
                description: "Video Acceleration API (Linux)".to_string(),
                codec_support: vec!["H.264".to_string(), "H.265".to_string()],
                platform: "Linux".to_string(),
            });
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        if detect_videotoolbox_support().await {
            hwaccels.push(PlatformHwAccel {
                name: "VideoToolbox".to_string(),
                description: "Apple VideoToolbox (macOS)".to_string(),
                codec_support: vec!["H.264".to_string(), "H.265".to_string()],
                platform: "macOS".to_string(),
            });
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if detect_dxva_support().await {
            hwaccels.push(PlatformHwAccel {
                name: "DXVA".to_string(),
                description: "DirectX Video Acceleration (Windows)".to_string(),
                codec_support: vec!["H.264".to_string(), "H.265".to_string()],
                platform: "Windows".to_string(),
            });
        }
    }
    
    hwaccels
}

#[derive(Debug, Clone)]
pub struct PlatformHwAccel {
    pub name: String,
    pub description: String,
    pub codec_support: Vec<String>,
    pub platform: String,
}

/// Get VAAPI-specific settings for Linux
#[cfg(target_os = "linux")]
pub fn get_vaapi_settings() -> Vec<(&'static str, String)> {
    vec![
        ("hwaccel", "vaapi".to_string()),
        ("hwaccel_device", "/dev/dri/renderD128".to_string()),
        ("hwaccel_output_format", "vaapi".to_string()),
        ("c:v", "h264_vaapi".to_string()),
        ("profile:v", "main".to_string()),
        ("level", "4.0".to_string()),
    ]
}

/// Get VideoToolbox-specific settings for macOS
#[cfg(target_os = "macos")]
pub fn get_videotoolbox_settings() -> Vec<(&'static str, String)> {
    vec![
        ("hwaccel", "videotoolbox".to_string()),
        ("c:v", "h264_videotoolbox".to_string()),
        ("profile:v", "main".to_string()),
        ("level", "4.0".to_string()),
        ("realtime", "true".to_string()), // Enable real-time encoding
    ]
}

/// Get platform-specific optimal settings
pub fn get_platform_optimal_settings(platform: &str) -> Vec<(&'static str, String)> {
    match platform.to_lowercase().as_str() {
        #[cfg(target_os = "linux")]
        "linux" | "vaapi" => get_vaapi_settings(),
        
        #[cfg(target_os = "macos")]
        "macos" | "videotoolbox" => get_videotoolbox_settings(),
        
        #[cfg(target_os = "windows")]
        "windows" | "dxva" => vec![
            ("hwaccel", "dxva2".to_string()),
            ("c:v", "h264".to_string()), // Fallback to software with DXVA decode
        ],
        
        _ => vec![], // No platform-specific settings
    }
}

/// Check if the current platform supports hardware acceleration
pub async fn platform_has_hardware_support() -> bool {
    #[cfg(target_os = "linux")]
    {
        detect_vaapi_support().await
    }
    
    #[cfg(target_os = "macos")]
    {
        detect_videotoolbox_support().await
    }
    
    #[cfg(target_os = "windows")]
    {
        detect_dxva_support().await
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        false // Unsupported platform
    }
}

/// Get recommended platform-specific encoder
pub async fn get_recommended_platform_encoder() -> Option<super::HardwareEncoder> {
    #[cfg(target_os = "linux")]
    {
        if detect_vaapi_support().await {
            Some(super::HardwareEncoder::Vaapi)
        } else {
            None
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        if detect_videotoolbox_support().await {
            Some(super::HardwareEncoder::VideoToolbox)
        } else {
            None
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        // Windows typically uses vendor-specific encoders (NVENC, QuickSync, AMF)
        // rather than DXVA for encoding
        None
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

/// Get platform information string
pub fn get_platform_info() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    
    format!("{} ({})", os, arch)
}

/// Check if a specific hardware acceleration method is available on this platform
pub async fn is_hwaccel_available_on_platform(hwaccel: &str) -> bool {
    match hwaccel.to_lowercase().as_str() {
        "vaapi" => {
            #[cfg(target_os = "linux")]
            { detect_vaapi_support().await }
            #[cfg(not(target_os = "linux"))]
            { false }
        },
        "videotoolbox" => {
            #[cfg(target_os = "macos")]
            { detect_videotoolbox_support().await }
            #[cfg(not(target_os = "macos"))]
            { false }
        },
        "dxva" | "dxva2" => {
            #[cfg(target_os = "windows")]
            { detect_dxva_support().await }
            #[cfg(not(target_os = "windows"))]
            { false }
        },
        _ => false,
    }
}