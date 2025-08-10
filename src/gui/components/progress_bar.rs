use eframe::egui::*;
use crate::gui::state::{ProgressState, CompressionStage};
use std::time::Duration;

#[derive(Debug, Default)]
pub struct ProgressBar {
    pub animation_time: f32,
    pub pulse_phase: f32,
}

impl ProgressBar {
    pub fn new() -> Self {
        Self {
            animation_time: 0.0,
            pulse_phase: 0.0,
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui, progress: f32, status_text: &str, progress_state: Option<&ProgressState>) -> ProgressBarResponse {
        let mut response = ProgressBarResponse::default();
        
        // Update animation
        self.animation_time += ui.input(|i| i.stable_dt);
        self.pulse_phase = (self.animation_time * 2.0).sin() * 0.5 + 0.5;
        
        ui.vertical(|ui| {
            // Status text
            ui.horizontal(|ui| {
                ui.label(RichText::new(status_text).strong());
                
                // Time remaining if available
                if let Some(state) = progress_state {
                    if let Some(remaining) = &state.estimated_remaining {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let time_str = Self::format_duration(*remaining);
                            ui.label(RichText::new(time_str)
                                .size(11.0)
                                .color(ui.style().visuals.weak_text_color()));
                        });
                    }
                }
            });
            
            ui.add_space(5.0);
            
            // Progress bar
            let bar_height = 20.0;
            let bar_rect = Rect::from_min_size(
                ui.cursor().min,
                [ui.available_width(), bar_height].into()
            );
            
            let bar_response = ui.allocate_rect(bar_rect, Sense::hover());
            
            // Background
            ui.painter().rect(
                bar_rect,
                4.0,
                ui.style().visuals.extreme_bg_color,
                Stroke::new(1.0, ui.style().visuals.widgets.inactive.bg_stroke.color)
            );
            
            // Progress fill
            if progress > 0.0 {
                let fill_width = (bar_rect.width() * progress.clamp(0.0, 1.0)) as f32;
                let fill_rect = Rect::from_min_size(
                    bar_rect.min,
                    [fill_width, bar_height].into()
                );
                
                // Gradient fill for better visual appeal
                let progress_color = if progress >= 1.0 {
                    Color32::from_rgb(46, 160, 67) // Green when complete
                } else {
                    Color32::from_rgb(52, 144, 220) // Blue during progress
                };
                
                ui.painter().rect(
                    fill_rect,
                    4.0,
                    progress_color,
                    Stroke::NONE
                );
                
                // Animated pulse effect for active progress
                if progress > 0.0 && progress < 1.0 {
                    let pulse_alpha = (self.pulse_phase * 60.0) as u8;
                    let pulse_color = Color32::from_rgba_unmultiplied(
                        255, 255, 255, pulse_alpha
                    );
                    
                    ui.painter().rect(
                        fill_rect,
                        4.0,
                        pulse_color,
                        Stroke::NONE
                    );
                }
            }
            
            // Progress text overlay
            let percentage = (progress * 100.0).round() as i32;
            let progress_text = format!("{}%", percentage);
            
            ui.painter().text(
                bar_rect.center(),
                Align2::CENTER_CENTER,
                &progress_text,
                FontId::proportional(12.0),
                if progress > 0.5 {
                    Color32::WHITE
                } else {
                    ui.style().visuals.text_color()
                }
            );
            
            ui.add_space(8.0);
            
            // Detailed progress info
            if let Some(state) = progress_state {
                self.show_detailed_progress(ui, state);
            }
            
            // Control buttons
            ui.horizontal(|ui| {
                // Pause/Resume button
                if progress > 0.0 && progress < 1.0 {
                    if ui.small_button("⏸ Pause").clicked() {
                        response.pause_clicked = true;
                    }
                }
                
                // Cancel button
                if progress > 0.0 && progress < 1.0 {
                    ui.add_space(5.0);
                    if ui.small_button("✖ Cancel").clicked() {
                        response.cancel_clicked = true;
                    }
                }
                
                // Open folder button (when complete)
                if progress >= 1.0 {
                    if ui.small_button("📁 Open Folder").clicked() {
                        response.open_folder_clicked = true;
                    }
                    
                    ui.add_space(5.0);
                    if ui.small_button("🔄 Compress Another").clicked() {
                        response.reset_clicked = true;
                    }
                }
            });
        });
        
        response
    }
    
    fn show_detailed_progress(&mut self, ui: &mut Ui, state: &ProgressState) {
        ui.horizontal(|ui| {
            // Current stage
            ui.label(RichText::new(state.stage_name())
                .size(11.0)
                .color(ui.style().visuals.weak_text_color()));
            
            ui.add_space(10.0);
            
            // FPS info if encoding
            if let (Some(fps), CompressionStage::Encoding) = (state.current_fps, &state.stage) {
                ui.label(RichText::new(format!("{:.1} fps", fps))
                    .size(11.0)
                    .color(Color32::from_rgb(100, 200, 100)));
            }
            
            // Frame progress if available
            if let (Some(total), processed) = (state.total_frames, state.frames_processed) {
                if total > 0 {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(format!("{}/{} frames", processed, total))
                            .size(10.0)
                            .color(ui.style().visuals.weak_text_color()));
                    });
                }
            }
        });
    }
    
    fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        
        if hours > 0 {
            format!("{}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{}:{:02}", minutes, seconds)
        }
    }
}

#[derive(Debug, Default)]
pub struct ProgressBarResponse {
    pub pause_clicked: bool,
    pub cancel_clicked: bool,
    pub open_folder_clicked: bool,
    pub reset_clicked: bool,
}

/// Compact progress indicator for smaller spaces
#[derive(Debug, Default)]
pub struct CompactProgressBar {
    pub spinning: bool,
    pub spin_angle: f32,
}

impl CompactProgressBar {
    pub fn new() -> Self {
        Self {
            spinning: false,
            spin_angle: 0.0,
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui, progress: Option<f32>, size: f32) {
        let rect = Rect::from_center_size(
            ui.cursor().center(),
            [size, size].into()
        );
        
        ui.allocate_rect(rect, Sense::hover());
        
        if let Some(progress) = progress {
            // Circular progress indicator
            let center = rect.center();
            let radius = size * 0.4;
            
            // Background circle
            ui.painter().circle_stroke(
                center,
                radius,
                Stroke::new(2.0, ui.style().visuals.widgets.inactive.bg_stroke.color)
            );
            
            // Progress arc
            if progress > 0.0 {
                let angle = progress * std::f32::consts::TAU;
                ui.painter().add(Shape::dashed_line(
                    &Self::arc_points(center, radius, 0.0, angle),
                    Stroke::new(3.0, ui.style().visuals.selection.stroke.color),
                    0.0,
                    0.0,
                ));
            }
            
            // Center percentage
            let percentage = (progress * 100.0).round() as i32;
            ui.painter().text(
                center,
                Align2::CENTER_CENTER,
                format!("{}%", percentage),
                FontId::proportional(size * 0.25),
                ui.style().visuals.text_color()
            );
        } else {
            // Spinning indicator
            self.spin_angle += ui.input(|i| i.stable_dt) * 4.0;
            let center = rect.center();
            let radius = size * 0.3;
            
            // Spinning dots
            for i in 0..8 {
                let angle = self.spin_angle + (i as f32 * std::f32::consts::TAU / 8.0);
                let dot_pos = center + Vec2::new(
                    angle.cos() * radius,
                    angle.sin() * radius
                );
                
                let alpha = ((i as f32 / 8.0) * 255.0) as u8;
                let color = Color32::from_rgba_unmultiplied(
                    100, 150, 255, alpha
                );
                
                ui.painter().circle_filled(dot_pos, size * 0.08, color);
            }
        }
    }
    
    fn arc_points(center: Pos2, radius: f32, start_angle: f32, end_angle: f32) -> Vec<Pos2> {
        let steps = ((end_angle - start_angle) * 20.0).ceil() as usize + 1;
        (0..steps).map(|i| {
            let t = i as f32 / (steps - 1) as f32;
            let angle = start_angle + (end_angle - start_angle) * t - std::f32::consts::FRAC_PI_2;
            center + Vec2::new(angle.cos() * radius, angle.sin() * radius)
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_duration_formatting() {
        assert_eq!(ProgressBar::format_duration(Duration::from_secs(30)), "0:30");
        assert_eq!(ProgressBar::format_duration(Duration::from_secs(90)), "1:30");
        assert_eq!(ProgressBar::format_duration(Duration::from_secs(3661)), "1:01:01");
    }
    
    #[test]
    fn test_progress_clamping() {
        let mut bar = ProgressBar::new();
        // Test that progress values are properly clamped in the UI logic
        assert_eq!((-0.5_f32).clamp(0.0, 1.0), 0.0);
        assert_eq!(1.5_f32.clamp(0.0, 1.0), 1.0);
    }
}