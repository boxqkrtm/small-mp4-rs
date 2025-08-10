

/// GUI utility functions
pub mod file_utils {
    use std::path::Path;
    
    
    /// Check if a file is a video file based on extension
    pub fn is_video_file(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            matches!(ext.as_str(), "mp4" | "avi" | "mov" | "mkv" | "flv" | "wmv" | "webm" | "m4v")
        } else {
            false
        }
    }
    
    /// Format file size in human readable format
    pub fn format_file_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}

/// GUI layout helpers
pub mod layout {
    use eframe::egui::*;
    
    /// Create a section with title and separator
    pub fn section(ui: &mut Ui, title: &str, content: impl FnOnce(&mut Ui)) {
        ui.add_space(10.0);
        ui.strong(title);
        ui.separator();
        ui.add_space(5.0);
        content(ui);
    }
    
    /// Create a labeled row with consistent spacing
    pub fn labeled_row(ui: &mut Ui, label: &str, content: impl FnOnce(&mut Ui)) {
        ui.horizontal(|ui| {
            ui.add_sized([80.0, 20.0], Label::new(label));
            content(ui);
        });
    }
}

/// Theme utilities
pub mod theme {
    use eframe::egui::*;
    
    pub fn setup_custom_styles(ctx: &Context) {
        let mut style = (*ctx.style()).clone();
        
        // Customize colors
        style.visuals.widgets.noninteractive.bg_fill = Color32::from_gray(240);
        style.visuals.widgets.inactive.bg_fill = Color32::from_gray(230);
        style.visuals.widgets.hovered.bg_fill = Color32::from_gray(250);
        style.visuals.widgets.active.bg_fill = Color32::from_gray(220);
        
        // Customize spacing
        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        style.spacing.button_padding = Vec2::new(12.0, 6.0);
        
        ctx.set_style(style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_file_size_formatting() {
        assert_eq!(file_utils::format_file_size(512), "512 B");
        assert_eq!(file_utils::format_file_size(1536), "1.5 KB");
        assert_eq!(file_utils::format_file_size(2_097_152), "2.0 MB");
    }
    
    #[test]
    fn test_video_file_detection() {
        assert!(file_utils::is_video_file(Path::new("test.mp4")));
        assert!(file_utils::is_video_file(Path::new("test.MP4")));
        assert!(!file_utils::is_video_file(Path::new("test.txt")));
    }
}
