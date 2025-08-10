use small_mp4::compression::{CompressionEngine, CompressionSettings, TargetSize};
use small_mp4::compression::hardware::{HardwareCapabilities, HardwareEncoder};
use std::path::Path;
use anyhow::Result;

#[tokio::test]
async fn test_target_sizes() {
    // Test that all target sizes are correctly converted to MB
    assert_eq!(TargetSize::Size1MB.as_mb(), 1.0);
    assert_eq!(TargetSize::Size5MB.as_mb(), 5.0);
    assert_eq!(TargetSize::Size10MB.as_mb(), 10.0);
    assert_eq!(TargetSize::Size30MB.as_mb(), 30.0);
    assert_eq!(TargetSize::Size50MB.as_mb(), 50.0);
}

#[tokio::test]
async fn test_compression_settings_default() {
    let settings = CompressionSettings::default();
    assert_eq!(settings.target_size, TargetSize::Size10MB);
    assert!(settings.enable_hardware_accel);
    assert!(settings.memory_optimization);
}

#[tokio::test]
async fn test_hardware_capabilities_software_only() {
    let hw_caps = HardwareCapabilities::software_only();
    assert_eq!(hw_caps.available_encoders.len(), 1);
    assert_eq!(hw_caps.available_encoders[0], HardwareEncoder::Software);
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    fn get_test_video_path(filename: &str) -> std::path::PathBuf {
        std::path::Path::new("test_videos/samples").join(filename)
    }

    fn get_output_path(filename: &str) -> std::path::PathBuf {
        std::path::Path::new("test_videos/output").join(filename)
    }

    #[tokio::test]
    async fn test_compress_720p_to_1mb() -> Result<()> {
        let input_path = get_test_video_path("test_720p_10s.mp4");
        let output_path = get_output_path("test_720p_10s_1mb.mp4");

        // Skip test if input doesn't exist
        if !input_path.exists() {
            eprintln!("Skipping test: {} not found. Run ./test_videos/generate_test_videos.sh first", input_path.display());
            return Ok(());
        }

        // Create output directory
        fs::create_dir_all("test_videos/output")?;

        // Setup compression
        let hw_caps = HardwareCapabilities::software_only();
        let mut engine = CompressionEngine::new(hw_caps);
        
        let mut settings = CompressionSettings::default();
        settings.target_size = TargetSize::Size1MB;
        settings.enable_hardware_accel = false; // Use software for consistent testing

        // Compress
        engine.compress(&input_path, Some(&output_path), &settings).await?;

        // Verify output exists and is under target size
        assert!(output_path.exists());
        let metadata = fs::metadata(&output_path)?;
        let size_mb = metadata.len() as f32 / (1024.0 * 1024.0);
        
        // Should be under 1MB with some tolerance
        assert!(size_mb <= 1.05, "Output size {} MB exceeds target of 1MB", size_mb);

        Ok(())
    }

    #[tokio::test]
    async fn test_compress_1080p_to_5mb() -> Result<()> {
        let input_path = get_test_video_path("test_1080p_30s.mp4");
        let output_path = get_output_path("test_1080p_30s_5mb.mp4");

        if !input_path.exists() {
            eprintln!("Skipping test: {} not found", input_path.display());
            return Ok(());
        }

        fs::create_dir_all("test_videos/output")?;

        let hw_caps = HardwareCapabilities::software_only();
        let mut engine = CompressionEngine::new(hw_caps);
        
        let mut settings = CompressionSettings::default();
        settings.target_size = TargetSize::Size5MB;
        settings.enable_hardware_accel = false;

        engine.compress(&input_path, Some(&output_path), &settings).await?;

        assert!(output_path.exists());
        let metadata = fs::metadata(&output_path)?;
        let size_mb = metadata.len() as f32 / (1024.0 * 1024.0);
        
        assert!(size_mb <= 5.25, "Output size {} MB exceeds target of 5MB", size_mb);

        Ok(())
    }

    #[tokio::test]
    async fn test_compress_static_to_1mb() -> Result<()> {
        let input_path = get_test_video_path("test_static_60s.mp4");
        let output_path = get_output_path("test_static_60s_1mb.mp4");

        if !input_path.exists() {
            eprintln!("Skipping test: {} not found", input_path.display());
            return Ok(());
        }

        fs::create_dir_all("test_videos/output")?;

        let hw_caps = HardwareCapabilities::software_only();
        let mut engine = CompressionEngine::new(hw_caps);
        
        let mut settings = CompressionSettings::default();
        settings.target_size = TargetSize::Size1MB;
        settings.enable_hardware_accel = false;

        engine.compress(&input_path, Some(&output_path), &settings).await?;

        assert!(output_path.exists());
        let metadata = fs::metadata(&output_path)?;
        let size_mb = metadata.len() as f32 / (1024.0 * 1024.0);
        
        // Static content should compress very well
        assert!(size_mb <= 1.05, "Output size {} MB exceeds target of 1MB", size_mb);

        Ok(())
    }

    #[tokio::test]
    async fn test_compress_4k_to_10mb() -> Result<()> {
        let input_path = get_test_video_path("test_4k_5s.mp4");
        let output_path = get_output_path("test_4k_5s_10mb.mp4");

        if !input_path.exists() {
            eprintln!("Skipping test: {} not found", input_path.display());
            return Ok(());
        }

        fs::create_dir_all("test_videos/output")?;

        let hw_caps = HardwareCapabilities::software_only();
        let mut engine = CompressionEngine::new(hw_caps);
        
        let mut settings = CompressionSettings::default();
        settings.target_size = TargetSize::Size10MB;
        settings.enable_hardware_accel = false;

        engine.compress(&input_path, Some(&output_path), &settings).await?;

        assert!(output_path.exists());
        let metadata = fs::metadata(&output_path)?;
        let size_mb = metadata.len() as f32 / (1024.0 * 1024.0);
        
        assert!(size_mb <= 10.5, "Output size {} MB exceeds target of 10MB", size_mb);

        Ok(())
    }

    #[tokio::test]
    #[should_panic(expected = "FileNotFound")]
    async fn test_compress_nonexistent_file() {
        let input_path = Path::new("nonexistent_video.mp4");
        let output_path = get_output_path("output.mp4");

        let hw_caps = HardwareCapabilities::software_only();
        let mut engine = CompressionEngine::new(hw_caps);
        let settings = CompressionSettings::default();

        // This should panic or return an error
        let _ = engine.compress(&input_path, Some(&output_path), &settings).await;
    }
}