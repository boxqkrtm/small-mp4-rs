use eframe::egui::*;
use crate::compression::TargetSize;

#[derive(Debug, Default)]
pub struct SizeSlider {
    pub dragging: bool,
    pub animation_time: f32,
}

impl SizeSlider {
    pub fn new() -> Self {
        Self {
            dragging: false,
            animation_time: 0.0,
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui, current_size: &mut TargetSize, auto_enabled: bool) -> bool {
        let mut changed = false;
        
        ui.horizontal(|ui| {
            // Size preset buttons
            let sizes = [
                (TargetSize::Size1MB, "1 MB", "Small files, lower quality"),
                (TargetSize::Size5MB, "5 MB", "Good for messaging"),
                (TargetSize::Size10MB, "10 MB", "Balanced quality/size"),
                (TargetSize::Size30MB, "30 MB", "High quality"),
                (TargetSize::Size50MB, "50 MB", "Maximum quality"),
            ];
            
            for (i, (size, label, tooltip)) in sizes.iter().enumerate() {
                let selected = *current_size == *size && !auto_enabled;
                
                // Button styling
                let button_color = if selected {
                    ui.style().visuals.selection.bg_fill
                } else if auto_enabled {
                    ui.style().visuals.widgets.inactive.bg_fill
                } else {
                    ui.style().visuals.widgets.noninteractive.bg_fill
                };
                
                let text_color = if selected {
                    ui.style().visuals.selection.stroke.color
                } else if auto_enabled {
                    ui.style().visuals.weak_text_color()
                } else {
                    ui.style().visuals.text_color()
                };
                
                // Custom button with styling
                let button_rect = Rect::from_min_size(
                    ui.cursor().min,
                    [60.0, 35.0].into()
                );
                
                let response = ui.allocate_rect(button_rect, Sense::click());
                
                // Draw button background
                let bg_color = if response.hovered() && !auto_enabled {
                    ui.style().visuals.widgets.hovered.bg_fill
                } else {
                    button_color
                };
                
                ui.painter().rect(
                    button_rect,
                    4.0,
                    bg_color,
                    Stroke::new(1.0, if selected {
                        ui.style().visuals.selection.stroke.color
                    } else {
                        ui.style().visuals.widgets.inactive.bg_stroke.color
                    })
                );
                
                // Draw button text
                ui.painter().text(
                    button_rect.center(),
                    Align2::CENTER_CENTER,
                    label,
                    FontId::default(),
                    text_color
                );
                
                // Handle click
                if response.clicked() && !auto_enabled {
                    *current_size = *size;
                    changed = true;
                }
                
                // Tooltip
                if response.hovered() {
                    response.on_hover_text(*tooltip);
                }
                
                // Spacing between buttons
                if i < sizes.len() - 1 {
                    ui.add_space(8.0);
                }
            }
        });
        
        ui.add_space(8.0);
        
        // Visual size indicator
        ui.horizontal(|ui| {
            ui.label("Size:");
            
            // Progress bar showing relative size
            let progress = match current_size {
                TargetSize::Size1MB => 0.1,
                TargetSize::Size5MB => 0.2,
                TargetSize::Size10MB => 0.3,
                TargetSize::Size30MB => 0.4,
                TargetSize::Size50MB => 0.5,
                TargetSize::Size100MB => 0.6,
                TargetSize::Size250MB => 0.7,
                TargetSize::Size500MB => 0.8,
                TargetSize::Size1000MB => 1.0,
            };
            
            let bar_rect = Rect::from_min_size(
                ui.cursor().min,
                [200.0, 12.0].into()
            );
            
            ui.allocate_rect(bar_rect, Sense::hover());
            
            // Background
            ui.painter().rect(
                bar_rect,
                6.0,
                ui.style().visuals.extreme_bg_color,
                Stroke::NONE
            );
            
            // Progress fill
            if !auto_enabled {
                let fill_rect = Rect::from_min_size(
                    bar_rect.min,
                    [bar_rect.width() * progress, bar_rect.height()].into()
                );
                
                ui.painter().rect(
                    fill_rect,
                    6.0,
                    ui.style().visuals.selection.bg_fill,
                    Stroke::NONE
                );
            }
            
            ui.add_space(10.0);
            
            // Current size text
            let size_text = if auto_enabled {
                "Auto".to_string()
            } else {
                format!("{} MB", current_size.as_mb() as u32)
            };
            
            ui.label(RichText::new(size_text).strong());
        });
        
        changed
    }
    
    pub fn show_estimation(&mut self, ui: &mut Ui, estimated_mb: Option<f32>, original_mb: Option<f32>) {
        if let (Some(estimated), Some(original)) = (estimated_mb, original_mb) {
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                ui.label("Estimated:");
                ui.label(RichText::new(format!("{:.1} MB", estimated)).color(Color32::DARK_GREEN));
                
                let compression_ratio = (1.0 - (estimated / original)) * 100.0;
                if compression_ratio > 0.0 {
                    ui.label(RichText::new(format!("({:.0}% smaller)", compression_ratio))
                        .size(11.0)
                        .color(ui.style().visuals.weak_text_color()));
                }
            });
        }
    }
}

/// Custom size slider widget for more precise control
pub struct CustomSizeSlider {
    pub min_mb: f32,
    pub max_mb: f32,
    pub current_mb: f32,
    pub logarithmic: bool,
}

impl CustomSizeSlider {
    pub fn new(min_mb: f32, max_mb: f32) -> Self {
        Self {
            min_mb,
            max_mb,
            current_mb: (min_mb + max_mb) / 2.0,
            logarithmic: true, // Better for file size ranges
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;
        
        ui.horizontal(|ui| {
            ui.label("Custom size:");
            
            // Logarithmic slider for better control across wide ranges
            let mut log_value = if self.logarithmic {
                self.current_mb.ln()
            } else {
                self.current_mb
            };
            
            let log_min = if self.logarithmic { self.min_mb.ln() } else { self.min_mb };
            let log_max = if self.logarithmic { self.max_mb.ln() } else { self.max_mb };
            
            let response = ui.add(Slider::new(&mut log_value, log_min..=log_max)
                .show_value(false));
            
            if response.changed() {
                self.current_mb = if self.logarithmic {
                    log_value.exp()
                } else {
                    log_value
                };
                changed = true;
            }
            
            ui.add(DragValue::new(&mut self.current_mb)
                .speed(0.1)
                .range(self.min_mb..=self.max_mb)
                .suffix(" MB"));
        });
        
        changed
    }
}
