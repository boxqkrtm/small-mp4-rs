use anyhow::{Result, anyhow};
use log::{debug, info};
use std::path::Path;
use std::process::Command;
use serde_json::Value;

use super::estimator::{VideoMetadata, ContentComplexity};

/// Extract video metadata using ffprobe
pub async fn get_video_metadata(video_path: &Path) -> Result<VideoMetadata> {
    if !video_path.exists() {
        return Err(anyhow!("Video file does not exist: {}", video_path.display()));
    }

    // Run ffprobe to get video information in JSON format
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            video_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| anyhow!("Failed to run ffprobe: {}. Is ffmpeg installed?", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("ffprobe failed: {}", stderr));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&json_str)
        .map_err(|e| anyhow!("Failed to parse ffprobe output: {}", e))?;

    debug!("ffprobe output: {}", json_str);

    // Extract video stream information
    let streams = json["streams"].as_array()
        .ok_or_else(|| anyhow!("No streams found in video"))?;
    
    let video_stream = streams.iter()
        .find(|s| s["codec_type"] == "video")
        .ok_or_else(|| anyhow!("No video stream found"))?;

    // Extract format information
    let format = &json["format"];

    // Parse video properties
    let width = video_stream["width"].as_u64()
        .ok_or_else(|| anyhow!("Failed to get video width"))? as u32;
    
    let height = video_stream["height"].as_u64()
        .ok_or_else(|| anyhow!("Failed to get video height"))? as u32;

    // Parse frame rate
    let fps = parse_frame_rate(video_stream)?;

    // Parse duration from format section (more reliable)
    let duration_seconds = format["duration"].as_str()
        .and_then(|s| s.parse::<f32>().ok())
        .or_else(|| {
            // Fallback to stream duration if format duration not available
            video_stream["duration"].as_str()
                .and_then(|s| s.parse::<f32>().ok())
        })
        .ok_or_else(|| anyhow!("Failed to get video duration"))?;

    // Parse bitrate
    let bitrate_kbps = format["bit_rate"].as_str()
        .and_then(|s| s.parse::<u32>().ok())
        .map(|b| b / 1000);

    // Get codec name
    let codec = video_stream["codec_name"].as_str()
        .unwrap_or("unknown")
        .to_string();

    let mut metadata = VideoMetadata {
        width,
        height,
        fps,
        duration_seconds,
        bitrate_kbps,
        codec,
        estimated_complexity: ContentComplexity::Medium,
    };

    // Estimate content complexity based on bitrate
    metadata.estimate_complexity_from_bitrate();

    info!("Video metadata: {}x{} @ {:.1}fps, duration: {:.1}s, bitrate: {:?} kbps",
         metadata.width, metadata.height, metadata.fps, metadata.duration_seconds, metadata.bitrate_kbps);

    Ok(metadata)
}

/// Parse frame rate from ffprobe output
fn parse_frame_rate(video_stream: &Value) -> Result<f32> {
    // Try r_frame_rate first (real frame rate)
    if let Some(r_frame_rate) = video_stream["r_frame_rate"].as_str() {
        if let Some(fps) = parse_fraction(r_frame_rate) {
            return Ok(fps);
        }
    }

    // Fallback to avg_frame_rate
    if let Some(avg_frame_rate) = video_stream["avg_frame_rate"].as_str() {
        if let Some(fps) = parse_fraction(avg_frame_rate) {
            return Ok(fps);
        }
    }

    // Last resort: try direct fps field
    if let Some(fps) = video_stream["fps"].as_f64() {
        return Ok(fps as f32);
    }

    Err(anyhow!("Failed to parse frame rate"))
}

/// Parse fraction string like "30/1" or "30000/1001"
fn parse_fraction(fraction: &str) -> Option<f32> {
    let parts: Vec<&str> = fraction.split('/').collect();
    if parts.len() == 2 {
        if let (Ok(num), Ok(den)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
            if den > 0.0 {
                return Some(num / den);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fraction() {
        assert_eq!(parse_fraction("30/1"), Some(30.0));
        assert_eq!(parse_fraction("25/1"), Some(25.0));
        assert_eq!(parse_fraction("30000/1001"), Some(29.97));
        assert_eq!(parse_fraction("24000/1001"), Some(23.976));
        assert_eq!(parse_fraction("invalid"), None);
        assert_eq!(parse_fraction("30/0"), None);
    }
}
