use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

use super::hardware::{HardwareCapabilities, HardwareEncoder, fallback::FallbackSystem};
use super::{CompressionSettings, SizeEstimator};
use super::metadata::get_video_metadata;

pub struct CompressionEngine {
    capabilities: HardwareCapabilities,
    fallback_system: FallbackSystem,
    size_estimator: SizeEstimator,
}

impl CompressionEngine {
    pub fn new(capabilities: HardwareCapabilities) -> Self {
        let fallback_system = FallbackSystem::new(&capabilities);
        let size_estimator = SizeEstimator::new();
        
        Self {
            capabilities,
            fallback_system,
            size_estimator,
        }
    }
    
    pub async fn compress(
        &mut self,
        input_path: &Path,
        output_path: Option<&Path>,
        settings: &CompressionSettings,
    ) -> Result<CompressionResult> {
        info!("Starting video compression");
        info!("Input: {}", input_path.display());
        
        // Validate input file
        if !input_path.exists() {
            return Err(anyhow!("Input file does not exist: {}", input_path.display()));
        }
        
        // Generate output path if not provided
        let output_path = if let Some(path) = output_path {
            path.to_path_buf()
        } else {
            generate_output_path(input_path)?
        };
        
        info!("Output: {}", output_path.display());
        info!("Using encoder: {:?}", settings.hardware_encoder);
        
        // Try compression with fallback
        let mut current_settings = settings.clone();
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 3;
        
        while attempts < MAX_ATTEMPTS {
            attempts += 1;
            
            match self.try_compress(input_path, &output_path, &current_settings).await {
                Ok(result) => {
                    // Record success for the encoder
                    self.fallback_system.record_success(&current_settings.hardware_encoder);
                    
                    info!("Compression completed successfully in {} attempts", attempts);
                    return Ok(result);
                },
                Err(e) => {
                    error!("Compression attempt {} failed: {}", attempts, e);
                    
                    // Record failure for the current encoder
                    self.fallback_system.record_failure(&current_settings.hardware_encoder, &e);
                    
                    if attempts < MAX_ATTEMPTS {
                        // Try to find a fallback encoder
                        let fallback_encoder = self.fallback_system.get_next_encoder(&current_settings.hardware_encoder);
                        
                        if fallback_encoder != current_settings.hardware_encoder {
                            warn!("Attempting fallback to encoder: {:?}", fallback_encoder);
                            current_settings.hardware_encoder = fallback_encoder;
                            current_settings.enable_hardware_accel = fallback_encoder.is_hardware_accelerated();
                            continue;
                        }
                    }
                    
                    // If this was the last attempt or no fallback available
                    if attempts >= MAX_ATTEMPTS {
                        return Err(anyhow!("Compression failed after {} attempts. Last error: {}", attempts, e));
                    }
                }
            }
        }
        
        Err(anyhow!("Compression failed after maximum attempts"))
    }
    
    async fn try_compress(
        &self,
        input_path: &Path,
        output_path: &Path,
        settings: &CompressionSettings,
    ) -> Result<CompressionResult> {
        let start_time = std::time::Instant::now();
        
        // Get video metadata first to calculate proper bitrate
        let metadata = get_video_metadata(input_path).await?;
        
        // Calculate target bitrate
        let target_bitrate = self.calculate_target_bitrate(settings, &metadata);
        info!("Using target bitrate: {} kbps", target_bitrate);
        
        // Build ffmpeg command
        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-i").arg(input_path);
        cmd.arg("-y"); // Overwrite output file
        
        // Configure hardware acceleration for input if needed
        if settings.enable_hardware_accel {
            match &settings.hardware_encoder {
                HardwareEncoder::NvencH264 | HardwareEncoder::NvencH265 | HardwareEncoder::NvencAV1 => {
                    cmd.arg("-hwaccel").arg("cuda");
                    if let Some(device_id) = settings.cuda_device_id {
                        cmd.arg("-hwaccel_device").arg(device_id.to_string());
                    }
                },
                HardwareEncoder::Vaapi => {
                    cmd.arg("-hwaccel").arg("vaapi");
                    cmd.arg("-hwaccel_device").arg("/dev/dri/renderD128");
                },
                HardwareEncoder::VideoToolbox => {
                    cmd.arg("-hwaccel").arg("videotoolbox");
                },
                _ => {}
            }
        }
        
        // Configure video codec
        let codec = match &settings.hardware_encoder {
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
        };
        
        cmd.arg("-c:v").arg(codec);
        
        // Set bitrate parameters
        cmd.arg("-b:v").arg(format!("{}k", target_bitrate));
        cmd.arg("-maxrate").arg(format!("{}k", target_bitrate));
        cmd.arg("-bufsize").arg(format!("{}k", target_bitrate * 2));
        
        // Set preset based on hardware
        match &settings.hardware_encoder {
            HardwareEncoder::Software => {
                let preset = settings.hardware_preset.software_preset();
                cmd.arg("-preset").arg(preset);
            },
            HardwareEncoder::NvencH264 | HardwareEncoder::NvencH265 | HardwareEncoder::NvencAV1 => {
                let preset = settings.hardware_preset.nvenc_preset();
                cmd.arg("-preset").arg(preset);
                // Use CBR for precise size control
                cmd.arg("-rc").arg("cbr");
                cmd.arg("-2pass").arg("1");
            },
            HardwareEncoder::AmfH264 | HardwareEncoder::AmfH265 => {
                cmd.arg("-quality").arg("speed");
                cmd.arg("-rc").arg("cbr");
            },
            HardwareEncoder::QsvH264 | HardwareEncoder::QsvH265 | HardwareEncoder::QsvAV1 => {
                cmd.arg("-preset").arg("medium");
                cmd.arg("-look_ahead").arg("1");
            },
            HardwareEncoder::Vaapi => {
                cmd.arg("-profile").arg("main");
                cmd.arg("-level").arg("4.0");
            },
            HardwareEncoder::VideoToolbox => {
                cmd.arg("-profile").arg("main");
            },
        }
        
        // Copy audio to avoid re-encoding
        cmd.arg("-c:a").arg("copy");
        
        // Output format settings
        cmd.arg("-movflags").arg("+faststart");
        cmd.arg("-pix_fmt").arg("yuv420p");
        
        // Memory optimization
        if settings.memory_optimization {
            cmd.arg("-threads").arg("1");
        }
        
        // Output file
        cmd.arg(output_path);
        
        debug!("FFmpeg command: {:?}", cmd);
        
        // Execute compression
        let output = cmd.output()
            .map_err(|e| anyhow!("Failed to execute FFmpeg: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("FFmpeg encoding failed: {}", stderr));
        }
        
        let encoding_time = start_time.elapsed();
        
        // Get output file size
        let output_size = fs::metadata(&output_path).await?.len();
        let output_size_mb = output_size as f64 / (1024.0 * 1024.0);
        
        // Get input file size for comparison
        let input_size = fs::metadata(input_path).await?.len();
        let input_size_mb = input_size as f64 / (1024.0 * 1024.0);
        let compression_ratio = input_size_mb / output_size_mb;
        
        // Check if we exceeded target size
        let target_mb = settings.target_size.as_mb();
        if output_size_mb > target_mb as f64 {
            warn!("Output size ({:.1} MB) exceeds target size ({:.1} MB)!", output_size_mb, target_mb);
        }
        
        info!("Compression completed:");
        info!("  Input size: {:.1} MB", input_size_mb);
        info!("  Output size: {:.1} MB", output_size_mb);
        info!("  Target size: {:.1} MB", target_mb);
        info!("  Compression ratio: {:.1}:1", compression_ratio);
        info!("  Encoding time: {:.1}s", encoding_time.as_secs_f64());
        
        Ok(CompressionResult {
            input_path: input_path.to_path_buf(),
            output_path: output_path.to_path_buf(),
            input_size_mb,
            output_size_mb,
            compression_ratio,
            encoding_time,
            encoder_used: settings.hardware_encoder.clone(),
            hardware_accelerated: settings.enable_hardware_accel,
        })
    }
    
    fn calculate_target_bitrate(&self, settings: &CompressionSettings, metadata: &super::estimator::VideoMetadata) -> u32 {
        let target_mb = settings.target_size.as_mb();
        let duration_seconds = metadata.duration_seconds;
        
        // Calculate total available bits
        let total_bits = target_mb * 8.0 * 1024.0 * 1024.0;
        
        // Reserve some space for audio (assume 128 kbps audio)
        let audio_bits = 128.0 * 1024.0 * duration_seconds;
        
        // Reserve 2% for container overhead
        let container_overhead = total_bits * 0.02;
        
        // Calculate available bits for video
        let available_video_bits = total_bits - audio_bits - container_overhead;
        
        // Calculate video bitrate in kbps
        let video_bitrate_bps = available_video_bits / duration_seconds;
        let video_bitrate_kbps = video_bitrate_bps / 1024.0;
        
        // Apply safety margin of 0.95 to ensure we never exceed target
        let safe_bitrate = (video_bitrate_kbps * 0.95) as u32;
        
        // Ensure minimum bitrate of 100 kbps
        let final_bitrate = safe_bitrate.max(100);
        
        debug!("Bitrate calculation: target={:.1}MB, duration={:.1}s, bitrate={}kbps", 
               target_mb, duration_seconds, final_bitrate);
        
        final_bitrate
    }
}

#[derive(Debug, Clone)]
pub struct CompressionResult {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub input_size_mb: f64,
    pub output_size_mb: f64,
    pub compression_ratio: f64,
    pub encoding_time: std::time::Duration,
    pub encoder_used: HardwareEncoder,
    pub hardware_accelerated: bool,
}

impl CompressionResult {
    pub fn summary(&self) -> String {
        format!(
            "Compressed {} ({:.1} MB) -> {} ({:.1} MB) in {:.1}s using {:?} ({:.1}x compression, {} encoder)",
            self.input_path.file_name().unwrap_or_default().to_string_lossy(),
            self.input_size_mb,
            self.output_path.file_name().unwrap_or_default().to_string_lossy(),
            self.output_size_mb,
            self.encoding_time.as_secs_f64(),
            self.encoder_used,
            self.compression_ratio,
            if self.hardware_accelerated { "hardware" } else { "software" }
        )
    }
}

fn generate_output_path(input_path: &Path) -> Result<PathBuf> {
    let mut output_path = input_path.to_path_buf();
    
    // Change extension to .mp4
    output_path.set_extension("mp4");
    
    // Add suffix to avoid overwriting
    let stem = input_path.file_stem()
        .ok_or_else(|| anyhow!("Invalid input filename"))?
        .to_string_lossy();
    
    let parent = input_path.parent()
        .ok_or_else(|| anyhow!("Input path has no parent directory"))?;
    
    // Try different suffixes until we find an available filename
    let suffixes = ["_compressed", "_small", "_squeezed", "_compact"];
    
    for suffix in &suffixes {
        let candidate = parent.join(format!("{}{}.mp4", stem, suffix));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    
    // If all suffixes are taken, add a number
    for i in 1..1000 {
        let candidate = parent.join(format!("{}_compressed_{}.mp4", stem, i));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    
    Err(anyhow!("Could not generate unique output filename"))
}