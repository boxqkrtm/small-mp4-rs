use eframe::egui::*;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct DropZone {
    pub hovered: bool,
    pub accepted_extensions: Vec<String>,
}

impl DropZone {
    pub fn new() -> Self {
        Self {
            hovered: false,
            accepted_extensions: vec![
                "mp4".to_string(), "avi".to_string(), "mov".to_string(),
                "mkv".to_string(), "flv".to_string(), "wmv".to_string(),
                "webm".to_string(), "m4v".to_string(),
            ],
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui, current_file: &Option<PathBuf>) -> Option<PathBuf> {
        let mut dropped_file = None;
        
        // Handle drag and drop
        if ui.ctx().input(|i| !i.raw.dropped_files.is_empty()) {
            ui.ctx().input(|i| {
                for file in &i.raw.dropped_files {
                    if let Some(path) = &file.path {
                        if self.is_video_file(path) {
                            dropped_file = Some(path.clone());
                            break;
                        }
                    }
                }
            });
        }
        
        // Detect hover state for visual feedback
        let response = ui.allocate_response(
            [ui.available_width(), 120.0].into(),
            Sense::hover()
        );
        
        self.hovered = response.hovered() || ui.ctx().input(|i| !i.raw.dropped_files.is_empty());
        
        // Draw drop zone
        let rect = response.rect;
        
        // Background color based on state
        let bg_color = if self.hovered {
            ui.style().visuals.selection.bg_fill
        } else {
            ui.style().visuals.window_fill
        };
        
        // Border style
        let stroke = if self.hovered {
            Stroke::new(2.0, ui.style().visuals.selection.stroke.color)
        } else {
            Stroke::new(1.0, ui.style().visuals.widgets.inactive.bg_stroke.color)
        };
        
        // Draw background
        ui.painter().rect(rect, 8.0, bg_color, stroke);
        
        // Draw content centered in the drop zone
        ui.allocate_ui_with_layout(
            rect.shrink(10.0).size(),
            Layout::top_down_justified(Align::Center),
            |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                
                // Icon
                let icon = if current_file.is_some() {
                    "🎬"
                } else {
                    "📁"
                };
                ui.label(RichText::new(icon).size(32.0));
                
                ui.add_space(5.0);
                
                // Text based on state
                let text = if let Some(ref file) = current_file {
                    format!("📄 {}", file.file_name().unwrap_or_default().to_string_lossy())
                } else if self.hovered {
                    "Drop video file here".to_string()
                } else {
                    "Drag & drop video files here\nor click Browse".to_string()
                };
                
                ui.label(RichText::new(text)
                    .size(14.0)
                    .color(if current_file.is_some() {
                        ui.style().visuals.text_color()
                    } else {
                        ui.style().visuals.weak_text_color()
                    }));
                
                ui.add_space(5.0);
                
                // Supported formats
                if current_file.is_none() {
                    ui.label(RichText::new("Supported: MP4, AVI, MOV, MKV, FLV, WMV")
                        .size(10.0)
                        .color(ui.style().visuals.weak_text_color()));
                }
            });
        });
        
        dropped_file
    }
    
    fn is_video_file(&self, path: &PathBuf) -> bool {
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            self.accepted_extensions.contains(&ext)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_video_file_detection() {
        let drop_zone = DropZone::new();
        
        assert!(drop_zone.is_video_file(&PathBuf::from("test.mp4")));
        assert!(drop_zone.is_video_file(&PathBuf::from("test.MP4")));
        assert!(drop_zone.is_video_file(&PathBuf::from("video.avi")));
        assert!(!drop_zone.is_video_file(&PathBuf::from("test.txt")));
        assert!(!drop_zone.is_video_file(&PathBuf::from("image.jpg")));
    }
}
