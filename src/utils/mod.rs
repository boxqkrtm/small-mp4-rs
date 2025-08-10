#![allow(dead_code)]
pub mod system_info;

use anyhow::Result;
use std::path::Path;

/// Format file size in human-readable format
pub fn format_file_size(size_bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;
    
    if size_bytes == 0 {
        return "0 B".to_string();
    }
    
    let size = size_bytes as f64;
    let unit_index = (size.log(THRESHOLD) as usize).min(UNITS.len() - 1);
    let normalized_size = size / THRESHOLD.powi(unit_index as i32);
    
    if unit_index == 0 {
        format!("{} {}", size_bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", normalized_size, UNITS[unit_index])
    }
}

/// Format duration in human-readable format
pub fn format_duration(seconds: f32) -> String {
    if seconds < 60.0 {
        format!("{:.1}s", seconds)
    } else if seconds < 3600.0 {
        let minutes = (seconds / 60.0) as u32;
        let remaining_seconds = seconds % 60.0;
        format!("{}m {:.1}s", minutes, remaining_seconds)
    } else {
        let hours = (seconds / 3600.0) as u32;
        let remaining_minutes = ((seconds % 3600.0) / 60.0) as u32;
        let remaining_seconds = seconds % 60.0;
        format!("{}h {}m {:.1}s", hours, remaining_minutes, remaining_seconds)
    }
}

/// Format compression ratio
pub fn format_compression_ratio(ratio: f64) -> String {
    if ratio >= 1.0 {
        format!("{:.1}:1", ratio)
    } else {
        format!("1:{:.1}", 1.0 / ratio)
    }
}

/// Generate a unique filename by appending a number if needed
pub fn generate_unique_filename(base_path: &Path) -> Result<std::path::PathBuf> {
    if !base_path.exists() {
        return Ok(base_path.to_path_buf());
    }
    
    let parent = base_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = base_path.file_stem().unwrap_or_default().to_string_lossy();
    let extension = base_path.extension().map(|e| e.to_string_lossy()).unwrap_or_default();
    
    for i in 1..=999 {
        let new_filename = if extension.is_empty() {
            format!("{}_{}", stem, i)
        } else {
            format!("{}_{}.{}", stem, i, extension)
        };
        
        let new_path = parent.join(new_filename);
        if !new_path.exists() {
            return Ok(new_path);
        }
    }
    
    Err(anyhow::anyhow!("Could not generate unique filename after 999 attempts"))
}

/// Validate that a path is safe to write to
pub fn validate_output_path(path: &Path) -> Result<()> {
    // Check parent directory exists or can be created
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    
    // Check we have write permissions
    if path.exists() {
        let metadata = std::fs::metadata(path)?;
        if metadata.permissions().readonly() {
            return Err(anyhow::anyhow!("Output path is read-only: {}", path.display()));
        }
    }
    
    Ok(())
}

/// Get file extension from path, handling edge cases
pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// Check if file is a video file based on extension
pub fn is_video_file(path: &Path) -> bool {
    const VIDEO_EXTENSIONS: &[&str] = &[
        "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v",
        "mpg", "mpeg", "3gp", "asf", "rm", "rmvb", "ts", "mts",
    ];
    
    get_file_extension(path)
        .map(|ext| VIDEO_EXTENSIONS.contains(&ext.as_str()))
        .unwrap_or(false)
}

/// Calculate compression percentage
pub fn calculate_compression_percentage(original_size: u64, compressed_size: u64) -> f64 {
    if original_size == 0 {
        return 0.0;
    }
    
    let reduction = original_size.saturating_sub(compressed_size) as f64;
    (reduction / original_size as f64) * 100.0
}

/// Create a progress indicator string
pub fn create_progress_bar(percentage: f32, width: usize) -> String {
    let filled = ((percentage / 100.0) * width as f32) as usize;
    let empty = width.saturating_sub(filled);
    
    format!(
        "[{}{}] {:.1}%",
        "█".repeat(filled),
        "░".repeat(empty),
        percentage
    )
}

/// Sanitize filename by removing/replacing invalid characters
pub fn sanitize_filename(filename: &str) -> String {
    const INVALID_CHARS: &[char] = &['<', '>', ':', '"', '|', '?', '*', '/', '\\'];
    
    filename
        .chars()
        .map(|c| if INVALID_CHARS.contains(&c) || c.is_control() { '_' } else { c })
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
        assert_eq!(format_file_size(1073741824), "1.0 GB");
    }
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30.0), "30.0s");
        assert_eq!(format_duration(90.0), "1m 30.0s");
        assert_eq!(format_duration(3660.0), "1h 1m 0.0s");
    }
    
    #[test]
    fn test_format_compression_ratio() {
        assert_eq!(format_compression_ratio(2.0), "2.0:1");
        assert_eq!(format_compression_ratio(0.5), "1:2.0");
        assert_eq!(format_compression_ratio(1.0), "1.0:1");
    }
    
    #[test]
    fn test_is_video_file() {
        assert!(is_video_file(Path::new("video.mp4")));
        assert!(is_video_file(Path::new("VIDEO.MP4")));
        assert!(is_video_file(Path::new("movie.mkv")));
        assert!(!is_video_file(Path::new("image.jpg")));
        assert!(!is_video_file(Path::new("document.txt")));
    }
    
    #[test]
    fn test_calculate_compression_percentage() {
        assert_eq!(calculate_compression_percentage(100, 50), 50.0);
        assert_eq!(calculate_compression_percentage(100, 25), 75.0);
        assert_eq!(calculate_compression_percentage(100, 100), 0.0);
        assert_eq!(calculate_compression_percentage(0, 50), 0.0);
    }
    
    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("normal_file.mp4"), "normal_file.mp4");
        assert_eq!(sanitize_filename("file<with>bad:chars"), "file_with_bad_chars");
        assert_eq!(sanitize_filename("file|with\"quotes"), "file_with_quotes");
    }
}
