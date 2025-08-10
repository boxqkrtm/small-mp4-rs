#![allow(dead_code)]
use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use tokio::fs;
use tokio::sync::mpsc;

use super::hardware::{HardwareCapabilities, HardwareEncoder, fallback::FallbackSystem};
use super::{CompressionSettings, SizeEstimator};
use super::metadata::get_video_metadata;
use regex::Regex;

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
        progress_tx: Option<mpsc::UnboundedSender<(f32, Option<std::time::Duration>)>>,
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
        if settings.compatibility_mode {
            info!("Using encoder: {:?} (H.264 compatibility mode)", settings.hardware_encoder);
        } else {
            info!("Using encoder: {:?}", settings.hardware_encoder);
        }
        
        // Try compression with fallback
        let mut current_settings = settings.clone();
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 3;
        
        while attempts < MAX_ATTEMPTS {
            attempts += 1;
            
            match self.try_compress(input_path, &output_path, &current_settings, progress_tx.clone()).await {
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
        progress_tx: Option<mpsc::UnboundedSender<(f32, Option<std::time::Duration>)>>,
    ) -> Result<CompressionResult> {
        let start_time = std::time::Instant::now();
        
        // Get video metadata first to calculate proper bitrate
        let metadata = get_video_metadata(input_path).await?;
        
        // Calculate target bitrate
        let target_bitrate = self.calculate_target_bitrate(settings, &metadata);
        info!("Using target bitrate: {} kbps", target_bitrate);
        
        // Check if we should use 2-pass encoding
        let use_two_pass = match &settings.hardware_encoder {
            HardwareEncoder::Software => true,
            // NVENC doesn't support traditional 2-pass, uses multipass instead
            _ => false,
        };
        
        if use_two_pass {
            info!("Using 2-pass encoding for better size accuracy");
            return self.two_pass_encode(input_path, output_path, settings, target_bitrate, &metadata, progress_tx).await;
        }
        
        // Build ffmpeg command using the shared function
        let mut cmd = self.build_ffmpeg_command(input_path, output_path, settings, target_bitrate, &metadata);
        
        // Add progress reporting
        cmd.arg("-progress").arg("pipe:2");
        
        debug!("FFmpeg command: {:?}", cmd);
        
        // Execute compression with real-time progress
        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("Failed to spawn FFmpeg: {}", e))?;
        
        // Parse progress from stderr if channel provided
        if let Some(tx) = progress_tx {
            if let Some(stderr) = child.stderr.take() {
                let reader = BufReader::new(stderr);
                let duration_seconds = metadata.duration_seconds;
                
                tokio::task::spawn_blocking(move || {
                    let start_time = std::time::Instant::now();
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            if line.starts_with("out_time_ms=") {
                                if let Ok(time_ms) = line.split('=').nth(1).unwrap_or("0").parse::<u64>() {
                                    let current_seconds = time_ms as f64 / 1_000_000.0;
                                    let progress = (current_seconds / duration_seconds as f64).min(1.0) as f32;
                                    
                                    // Calculate ETA
                                    let elapsed = start_time.elapsed();
                                    let eta = if progress > 0.01 {
                                        let total_estimated = elapsed.as_secs_f64() / progress as f64;
                                        let remaining = total_estimated - elapsed.as_secs_f64();
                                        if remaining > 0.0 {
                                            Some(std::time::Duration::from_secs_f64(remaining))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };
                                    
                                    // Send progress update through channel
                                    let _ = tx.send((progress, eta));
                                }
                            }
                        }
                    }
                });
            }
        }
        
        let output = child.wait_with_output()
            .map_err(|e| anyhow!("Failed to wait for FFmpeg: {}", e))?;
        
        
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
    
    async fn two_pass_encode(
        &self,
        input_path: &Path,
        output_path: &Path,
        settings: &CompressionSettings,
        target_bitrate: u32,
        metadata: &super::estimator::VideoMetadata,
        progress_tx: Option<mpsc::UnboundedSender<(f32, Option<std::time::Duration>)>>,
    ) -> Result<CompressionResult> {
        let start_time = std::time::Instant::now();
        let temp_log = format!("/tmp/ffmpeg2pass_{}", std::process::id());
        
        // First pass
        info!("Starting first pass analysis...");
        let first_pass_result = self.run_ffmpeg_pass(
            input_path,
            settings,
            target_bitrate,
            1,
            &temp_log,
            progress_tx.clone(),
        ).await?;
        
        if !first_pass_result {
            return Err(anyhow!("First pass failed"));
        }
        
        // Second pass
        info!("Starting second pass encoding...");
        let mut cmd = self.build_ffmpeg_command(input_path, output_path, settings, target_bitrate, metadata);
        
        // Add 2-pass specific arguments
        cmd.arg("-pass").arg("2");
        cmd.arg("-passlogfile").arg(&temp_log);
        
        // Execute second pass
        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("Failed to start FFmpeg: {}", e))?;
        
        // Monitor progress
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            let duration = metadata.duration_seconds;
            
            for line in reader.lines() {
                if let Ok(line) = line {
                    if let Some(progress) = parse_ffmpeg_progress(&line, duration as f64) {
                        // Second pass progress (50-100%)
                        let adjusted_progress = 0.5 + (progress * 0.5);
                        if let Some(ref tx) = progress_tx {
                            let eta = calculate_eta(adjusted_progress, start_time.elapsed());
                            let _ = tx.send((adjusted_progress, eta));
                        }
                    }
                }
            }
        }
        
        let output = child.wait_with_output()
            .map_err(|e| anyhow!("Failed to wait for FFmpeg: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("FFmpeg encoding failed: {}", stderr));
        }
        
        // Clean up log files
        let _ = std::fs::remove_file(format!("{}-0.log", temp_log));
        let _ = std::fs::remove_file(format!("{}-0.log.mbtree", temp_log));
        
        // Get results
        let encoding_time = start_time.elapsed();
        let output_size = fs::metadata(&output_path).await?.len();
        let output_size_mb = output_size as f64 / (1024.0 * 1024.0);
        let input_size = fs::metadata(input_path).await?.len();
        let input_size_mb = input_size as f64 / (1024.0 * 1024.0);
        let compression_ratio = input_size_mb / output_size_mb;
        
        // Check if we exceeded target size
        let target_mb = settings.target_size.as_mb();
        if output_size_mb > target_mb as f64 {
            warn!("Output size ({:.1} MB) exceeds target size ({:.1} MB)!", output_size_mb, target_mb);
        }
        
        info!("2-pass compression completed:");
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
    
    async fn run_ffmpeg_pass(
        &self,
        input_path: &Path,
        settings: &CompressionSettings,
        target_bitrate: u32,
        pass_num: u8,
        log_file: &str,
        progress_tx: Option<mpsc::UnboundedSender<(f32, Option<std::time::Duration>)>>,
    ) -> Result<bool> {
        let metadata = get_video_metadata(input_path).await?;
        let mut cmd = self.build_ffmpeg_command(input_path, &PathBuf::from("/dev/null"), settings, target_bitrate, &metadata);
        
        // Add pass-specific arguments
        cmd.arg("-pass").arg(pass_num.to_string());
        cmd.arg("-passlogfile").arg(log_file);
        
        // For first pass, we don't need audio
        if pass_num == 1 {
            cmd.arg("-an");
        }
        
        // For Windows, use NUL instead of /dev/null
        #[cfg(target_os = "windows")]
        {
            cmd.arg("-f").arg("null").arg("NUL");
        }
        #[cfg(not(target_os = "windows"))]
        {
            cmd.arg("-f").arg("null").arg("/dev/null");
        }
        
        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("Failed to start FFmpeg pass {}: {}", pass_num, e))?;
        
        // Get metadata for progress tracking
        let metadata = get_video_metadata(input_path).await?;
        let duration = metadata.duration_seconds;
        let start_time = std::time::Instant::now();
        
        // Monitor progress
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            
            for line in reader.lines() {
                if let Ok(line) = line {
                    if let Some(progress) = parse_ffmpeg_progress(&line, duration as f64) {
                        // First pass progress (0-50%)
                        let adjusted_progress = progress * 0.5;
                        if let Some(ref tx) = progress_tx {
                            let eta = calculate_eta(adjusted_progress, start_time.elapsed());
                            let _ = tx.send((adjusted_progress, eta));
                        }
                    }
                }
            }
        }
        
        let output = child.wait_with_output()
            .map_err(|e| anyhow!("Failed to wait for FFmpeg pass {}: {}", pass_num, e))?;
        
        Ok(output.status.success())
    }
    
    fn build_ffmpeg_command(
        &self,
        input_path: &Path,
        output_path: &Path,
        settings: &CompressionSettings,
        target_bitrate: u32,
        metadata: &super::estimator::VideoMetadata,
    ) -> Command {
        let mut cmd = Command::new("ffmpeg");
        
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
        
        cmd.arg("-i").arg(input_path);
        cmd.arg("-y"); // Overwrite output file
        
        // Configure video codec
        let codec = if settings.compatibility_mode {
            match &settings.hardware_encoder {
                HardwareEncoder::NvencH264 | HardwareEncoder::NvencH265 | HardwareEncoder::NvencAV1 => "h264_nvenc",
                HardwareEncoder::AmfH264 | HardwareEncoder::AmfH265 => "h264_amf",
                HardwareEncoder::QsvH264 | HardwareEncoder::QsvH265 | HardwareEncoder::QsvAV1 => "h264_qsv",
                HardwareEncoder::Vaapi => "h264_vaapi",
                HardwareEncoder::VideoToolbox => "h264_videotoolbox",
                HardwareEncoder::Software => "libx264",
            }
        } else {
            match &settings.hardware_encoder {
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
        };
        
        cmd.arg("-c:v").arg(codec);
        info!("Using codec: {}", codec);
        
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
                // NVENC uses multipass for better quality
                cmd.arg("-rc").arg("vbr");
                cmd.arg("-multipass").arg("fullres");
                cmd.arg("-cq").arg("0");
            },
            HardwareEncoder::AmfH264 | HardwareEncoder::AmfH265 => {
                cmd.arg("-quality").arg("speed");
                cmd.arg("-rc").arg("vbr_latency");
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
        
        // Configure audio encoding based on duration
        let audio_bitrate = if metadata.duration_seconds > 600.0 {
            "96k"  // 96 kbps for videos > 10 minutes
        } else if metadata.duration_seconds > 300.0 {
            "112k"  // 112 kbps for videos > 5 minutes
        } else {
            "128k"  // 128 kbps for shorter videos
        };
        
        cmd.arg("-c:a").arg("aac");
        cmd.arg("-b:a").arg(audio_bitrate);
        cmd.arg("-ac").arg("2"); // Stereo
        
        // Output format settings
        cmd.arg("-movflags").arg("+faststart");
        cmd.arg("-pix_fmt").arg("yuv420p");
        
        // Memory optimization
        if settings.memory_optimization {
            cmd.arg("-threads").arg("1");
        }
        
        // Add output path
        cmd.arg(output_path);
        
        cmd
    }
    
    fn calculate_target_bitrate(&self, settings: &CompressionSettings, metadata: &super::estimator::VideoMetadata) -> u32 {
        let target_mb = settings.target_size.as_mb();
        let duration_seconds = metadata.duration_seconds;
        
        // Calculate total available bits
        let total_bits = target_mb * 8.0 * 1024.0 * 1024.0;
        
        // Dynamic audio bitrate calculation
        // Use lower audio bitrate for longer videos to save space
        let audio_bitrate = if duration_seconds > 600.0 {
            96.0  // 96 kbps for videos > 10 minutes
        } else if duration_seconds > 300.0 {
            112.0  // 112 kbps for videos > 5 minutes
        } else {
            128.0  // 128 kbps for shorter videos
        };
        let audio_bits = audio_bitrate * 1024.0 * duration_seconds;
        
        // Reserve 1% for container overhead (reduced from 2%)
        let container_overhead = total_bits * 0.01;
        
        // Calculate available bits for video
        let available_video_bits = total_bits - audio_bits - container_overhead;
        
        // Calculate video bitrate in kbps
        let video_bitrate_bps = available_video_bits / duration_seconds;
        let video_bitrate_kbps = video_bitrate_bps / 1024.0;
        
        // Apply smaller safety margin of 0.98 for better size utilization
        let safe_bitrate = (video_bitrate_kbps * 0.98) as u32;
        
        // Calculate minimum bitrate based on resolution for quality
        let min_bitrate = if metadata.width >= 1920 {
            300  // 1080p+ needs at least 300 kbps
        } else if metadata.width >= 1280 {
            200  // 720p needs at least 200 kbps
        } else {
            150  // Lower resolutions
        };
        
        let final_bitrate = safe_bitrate.max(min_bitrate);
        
        info!("Bitrate calculation: target={:.1}MB, duration={:.1}s, audio={}kbps, video={}kbps", 
              target_mb, duration_seconds, audio_bitrate, final_bitrate);
        
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

fn parse_ffmpeg_progress(line: &str, total_duration: f64) -> Option<f32> {
    // Parse FFmpeg progress output
    // Example: "frame= 1234 fps=123.4 q=23.0 size= 1234kB time=00:01:23.45 bitrate= 123.4kbits/s speed=1.23x"
    lazy_static::lazy_static! {
        static ref TIME_RE: Regex = Regex::new(r"time=(\d{2}):(\d{2}):(\d{2}\.\d+)").unwrap();
    }
    
    if let Some(captures) = TIME_RE.captures(line) {
        let hours: f64 = captures.get(1)?.as_str().parse().ok()?;
        let minutes: f64 = captures.get(2)?.as_str().parse().ok()?;
        let seconds: f64 = captures.get(3)?.as_str().parse().ok()?;
        
        let current_time = hours * 3600.0 + minutes * 60.0 + seconds;
        let progress = (current_time / total_duration).min(1.0) as f32;
        
        return Some(progress);
    }
    
    None
}

fn calculate_eta(progress: f32, elapsed: std::time::Duration) -> Option<std::time::Duration> {
    if progress > 0.0 && progress < 1.0 {
        let total_time = elapsed.as_secs_f64() / progress as f64;
        let remaining_time = total_time - elapsed.as_secs_f64();
        
        if remaining_time > 0.0 {
            return Some(std::time::Duration::from_secs_f64(remaining_time));
        }
    }
    
    None
}
