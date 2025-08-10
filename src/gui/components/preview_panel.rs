use eframe::egui::*;
use crate::gui::state::PreviewData;

#[derive(Debug, Default)]
pub struct PreviewPanel {
    pub loading: bool,
    pub show_details: bool,
}

impl PreviewPanel {
    pub fn new() -> Self {
        Self {
            loading: false,
            show_details: false,
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui, original: &Option<PreviewData>, compressed: &Option<PreviewData>) {
        ui.horizontal(|ui| {
            // Original preview
            ui.group(|ui| {
                self.show_preview_side(ui, "Original", original, true);
            });
            
            ui.add_space(10.0);
            
            // Compressed preview
            ui.group(|ui| {
                self.show_preview_side(ui, "Preview", compressed, false);
            });
        });
        
        // Details toggle
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            if ui.small_button(if self.show_details { "Hide Details" } else { "Show Details" }).clicked() {
                self.show_details = !self.show_details;
            }
        });
        
        // Show detailed comparison if enabled
        if self.show_details {
            self.show_comparison_details(ui, original, compressed);
        }
    }
    
    fn show_preview_side(&mut self, ui: &mut Ui, title: &str, data: &Option<PreviewData>, is_original: bool) {
        ui.vertical_centered(|ui| {
            // Title
            ui.strong(title);
            ui.add_space(5.0);
            
            // Preview area (150x100)
            let preview_size = [150.0, 100.0];
            let (rect, _response) = ui.allocate_exact_size(preview_size.into(), Sense::hover());
            
            // Background
            ui.painter().rect_filled(rect, 4.0, ui.style().visuals.extreme_bg_color);
            
            if let Some(preview_data) = data {
                // Draw thumbnail if available
                if let Some(_thumbnail) = &preview_data.thumbnail {
                    // TODO: Implement actual image rendering
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        "🎬",
                        FontId::proportional(32.0),
                        ui.style().visuals.text_color()
                    );
                } else {
                    // Loading or placeholder
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        if self.loading { "⏳" } else { "📹" },
                        FontId::proportional(32.0),
                        ui.style().visuals.weak_text_color()
                    );
                }
                
                ui.add_space(5.0);
                
                // Basic info
                ui.label(format!("{}x{}", preview_data.width, preview_data.height));
                ui.label(format!("{:.1}s", preview_data.duration));
                ui.label(format!("{:.1} MB", preview_data.file_size as f64 / 1_048_576.0));
            } else {
                // No data state
                let icon = if is_original { "📁" } else { "⏳" };
                let text = if is_original { "No file selected" } else { "Ready to compress" };
                
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    icon,
                    FontId::proportional(32.0),
                    ui.style().visuals.weak_text_color()
                );
                
                ui.add_space(5.0);
                ui.label(RichText::new(text)
                    .size(11.0)
                    .color(ui.style().visuals.weak_text_color()));
            }
        });
    }
    
    fn show_comparison_details(&mut self, ui: &mut Ui, original: &Option<PreviewData>, compressed: &Option<PreviewData>) {
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(5.0);
        
        ui.columns(2, |columns| {
            // Original details
            columns[0].group(|ui| {
                ui.strong("Original Details");
                ui.add_space(5.0);
                
                if let Some(data) = original {
                    self.show_detailed_info(ui, data);
                } else {
                    ui.label("No file loaded");
                }
            });
            
            // Compressed details
            columns[1].group(|ui| {
                ui.strong("Compressed Details");
                ui.add_space(5.0);
                
                if let Some(data) = compressed {
                    self.show_detailed_info(ui, data);
                } else {
                    ui.label("Not compressed yet");
                }
            });
        });
        
        // Comparison stats
        if let (Some(orig), Some(comp)) = (original, compressed) {
            ui.add_space(10.0);
            self.show_comparison_stats(ui, orig, comp);
        }
    }
    
    fn show_detailed_info(&mut self, ui: &mut Ui, data: &PreviewData) {
        ui.horizontal(|ui| {
            ui.label("Resolution:");
            ui.label(format!("{}×{}", data.width, data.height));
        });
        
        ui.horizontal(|ui| {
            ui.label("Duration:");
            ui.label(format!("{:.2}s", data.duration));
        });
        
        ui.horizontal(|ui| {
            ui.label("File Size:");
            ui.label(format!("{:.2} MB", data.file_size as f64 / 1_048_576.0));
        });
        
        ui.horizontal(|ui| {
            ui.label("Codec:");
            ui.label(&data.codec);
        });
        
        ui.horizontal(|ui| {
            ui.label("Bitrate:");
            ui.label(format!("{} kbps", data.bitrate));
        });
    }
    
    fn show_comparison_stats(&mut self, ui: &mut Ui, original: &PreviewData, compressed: &PreviewData) {
        ui.group(|ui| {
            ui.strong("Compression Results");
            ui.add_space(5.0);
            
            // Size reduction
            let orig_size = original.file_size as f64;
            let comp_size = compressed.file_size as f64;
            let reduction_percent = ((orig_size - comp_size) / orig_size) * 100.0;
            let compression_ratio = orig_size / comp_size;
            
            ui.horizontal(|ui| {
                ui.label("Size reduction:");
                let color = if reduction_percent > 0.0 {
                    Color32::DARK_GREEN
                } else {
                    Color32::DARK_RED
                };
                ui.label(RichText::new(format!("{:.1}%", reduction_percent)).color(color));
            });
            
            ui.horizontal(|ui| {
                ui.label("Compression ratio:");
                ui.label(format!("{:.1}:1", compression_ratio));
            });
            
            // Quality comparison (if bitrate changed significantly)
            let bitrate_change = if original.bitrate > 0 {
                ((compressed.bitrate as f64 - original.bitrate as f64) / original.bitrate as f64) * 100.0
            } else {
                0.0
            };
            
            if bitrate_change.abs() > 5.0 {
                ui.horizontal(|ui| {
                    ui.label("Bitrate change:");
                    let color = if bitrate_change < 0.0 {
                        Color32::ORANGE
                    } else {
                        Color32::DARK_GREEN
                    };
                    ui.label(RichText::new(format!("{:+.1}%", bitrate_change)).color(color));
                });
            }
        });
    }
}

/// Preview thumbnail generation (placeholder for actual implementation)
pub struct ThumbnailGenerator {
    // This would interface with FFmpeg to generate thumbnails
}

impl ThumbnailGenerator {
    pub fn new() -> Self {
        Self {}
    }
    
    pub async fn generate_thumbnail(&self, _video_path: &std::path::Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // TODO: Implement actual thumbnail generation using FFmpeg
        // For now, return a placeholder
        Ok(vec![0; 150 * 100 * 4]) // RGBA placeholder
    }
    
    pub async fn get_video_info(&self, _video_path: &std::path::Path) -> Result<PreviewData, Box<dyn std::error::Error>> {
        // TODO: Implement actual video analysis using FFmpeg
        // For now, return placeholder data
        Ok(PreviewData {
            thumbnail: None,
            width: 1920,
            height: 1080,
            duration: 120.0,
            file_size: 50_000_000, // 50 MB
            codec: "H.264".to_string(),
            bitrate: 5000,
        })
    }
}
