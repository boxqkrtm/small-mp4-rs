use eframe::egui::{self, *};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::compression::{CompressionEngine, TargetSize};
use crate::compression::hardware::HardwareCapabilities;
use super::components::{DropZone, SizeSlider, PreviewPanel, ProgressBar};
use super::state::{AppState, CompressionStatus};
use super::{GuiConfig, Language};

pub struct SmallMp4App {
    pub config: GuiConfig,
    pub state: AppState,
    pub hardware_capabilities: Arc<Mutex<Option<HardwareCapabilities>>>,
    pub compression_engine: Arc<Mutex<Option<CompressionEngine>>>,
    
    // UI Components
    drop_zone: DropZone,
    size_slider: SizeSlider,
    preview_panel: PreviewPanel,
    progress_bar: ProgressBar,
    
    // Advanced settings (hidden by default)
    show_advanced: bool,
    show_about: bool,
    
    // Drag and drop support
    dropped_files: Vec<PathBuf>,
}

impl Default for SmallMp4App {
    fn default() -> Self {
        Self {
            config: GuiConfig::default(),
            state: AppState::default(),
            hardware_capabilities: Arc::new(Mutex::new(None)),
            compression_engine: Arc::new(Mutex::new(None)),
            drop_zone: DropZone::default(),
            size_slider: SizeSlider::default(),
            preview_panel: PreviewPanel::default(),
            progress_bar: ProgressBar::default(),
            show_advanced: false,
            show_about: false,
            dropped_files: Vec::new(),
        }
    }
}

impl SmallMp4App {
    pub fn new(hw_capabilities: HardwareCapabilities) -> Self {
        let mut app = Self::default();
        
        // Set hardware capabilities
        *app.hardware_capabilities.lock().unwrap() = Some(hw_capabilities.clone());
        
        // Initialize compression engine
        *app.compression_engine.lock().unwrap() = Some(CompressionEngine::new(hw_capabilities));
        
        app
    }

    pub fn new_with_context(cc: &eframe::CreationContext<'_>, hw_capabilities: HardwareCapabilities) -> Self {
        // Configure fonts for international support
        Self::setup_fonts(&cc.egui_ctx);
        
        Self::new(hw_capabilities)
    }
    
    fn setup_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        
        // Add Korean font support
        fonts.font_data.insert(
            "noto_sans_cjk_kr".to_owned(),
            egui::FontData::from_static(include_bytes!("../../assets/fonts/NotoSansCJKkr-Regular.otf")),
        );
        
        // Add Japanese font support
        fonts.font_data.insert(
            "noto_sans_cjk_jp".to_owned(), 
            egui::FontData::from_static(include_bytes!("../../assets/fonts/NotoSansCJKjp-Regular.otf")),
        );
        
        // Configure font families - Add CJK fonts with higher priority
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
            .insert(0, "noto_sans_cjk_kr".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
            .insert(1, "noto_sans_cjk_jp".to_owned());
            
        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap()
            .insert(0, "noto_sans_cjk_kr".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap()
            .insert(1, "noto_sans_cjk_jp".to_owned());
        
        ctx.set_fonts(fonts);
        
        // Adjust text styles for better CJK rendering
        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(14.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(14.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(18.0, egui::FontFamily::Proportional),
        );
        ctx.set_style(style);
    }
    
    fn get_text(&self, key: &str) -> String {
        match self.config.language {
            Language::Korean => match key {
                "title" => "작은mp4 - 동영상 압축기".to_string(),
                "drag_drop" => "동영상 파일을 여기에 끌어다 놓으세요".to_string(),
                "browse" => "찾아보기".to_string(),
                "target_size" => "목표 크기:".to_string(),
                "auto" => "자동".to_string(),
                "compress" => "압축하기".to_string(),
                "stop" => "중지".to_string(),
                "cancel" => "취소".to_string(),
                "original" => "원본".to_string(),
                "preview" => "미리보기".to_string(),
                _ => key.to_string(),
            },
            Language::Japanese => match key {
                "title" => "小さなmp4 - 動画圧縮ツール".to_string(), 
                "drag_drop" => "動画ファイルをここにドラッグ&ドロップ".to_string(),
                "browse" => "参照".to_string(),
                "target_size" => "目標サイズ:".to_string(),
                "auto" => "自動".to_string(),
                "compress" => "圧縮".to_string(),
                "stop" => "停止".to_string(), 
                "cancel" => "キャンセル".to_string(),
                "original" => "元の動画".to_string(),
                "preview" => "プレビュー".to_string(),
                _ => key.to_string(),
            },
            Language::English => match key {
                "title" => "Small MP4 - Video Compressor".to_string(),
                "drag_drop" => "Drag & drop video files here".to_string(),
                "browse" => "Browse...".to_string(),
                "target_size" => "Target Size:".to_string(),
                "auto" => "Auto".to_string(),
                "compress" => "Compress".to_string(),
                "stop" => "Stop".to_string(),
                "cancel" => "Cancel".to_string(), 
                "original" => "Original".to_string(),
                "preview" => "Preview".to_string(),
                _ => key.to_string(),
            },
        }
    }
}

impl eframe::App for SmallMp4App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle dropped files
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                for file in &i.raw.dropped_files {
                    if let Some(path) = &file.path {
                        self.state.input_file = Some(path.clone());
                        log::info!("File dropped: {:?}", path);
                    }
                }
            }
        });
        
        // Main UI
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_main_ui(ui);
        });
        
        // Advanced settings window
        if self.show_advanced {
            self.draw_advanced_window(ctx);
        }
        
        // About window
        if self.show_about {
            self.draw_about_window(ctx);
        }
        
        // Request repaint for smooth animations
        if self.state.status != CompressionStatus::Idle {
            ctx.request_repaint();
        }
    }
}

impl SmallMp4App {
    fn draw_main_ui(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            // Title
            ui.add_space(5.0);
            ui.heading(self.get_text("title"));
            ui.add_space(10.0);
            
            // Input section with drag & drop
            ui.group(|ui| {
                self.draw_input_section(ui);
            });
            
            ui.add_space(10.0);
            
            // Size slider section  
            ui.group(|ui| {
                self.draw_size_section(ui);
            });
            
            ui.add_space(10.0);
            
            // Progress and controls
            self.draw_controls_section(ui);
            
            // Menu bar at bottom
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                self.draw_menu_bar(ui);
            });
        });
    }
    
    fn draw_input_section(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Input file
            ui.label("Input:");
            ui.horizontal(|ui| {
                // File path display
                let file_text = self.state.input_file
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| self.get_text("drag_drop").to_string());
                    
                ui.add_sized([350.0, 25.0], egui::TextEdit::singleline(&mut file_text.as_str())
                    .interactive(false));
                
                // Browse button
                if ui.button(self.get_text("browse")).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Video files", &["mp4", "avi", "mov", "mkv", "flv", "wmv"])
                        .pick_file() 
                    {
                        self.state.set_input_file(path);
                    }
                }
            });
            
            ui.add_space(5.0);
            
            // Output options
            ui.label("Output:");
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.state.same_folder, "Same folder");
                
                // Show output folder selection when not using same folder
                if !self.state.same_folder {
                    ui.separator();
                    
                    let folder_text = self.state.output_folder
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Select output folder...".to_string());
                    
                    ui.add_sized([200.0, 25.0], egui::TextEdit::singleline(&mut folder_text.as_str())
                        .interactive(false));
                    
                    if ui.button("📁 Choose").clicked() {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            self.state.output_folder = Some(folder);
                        }
                    }
                }
            });
        });
    }
    
    fn draw_size_section(&mut self, ui: &mut egui::Ui) {
        ui.label(self.get_text("target_size"));
        
        ui.horizontal(|ui| {
            // Size buttons (1, 5, 10, 30, 50 MB)
            let sizes = [
                (TargetSize::Size1MB, "1 MB"),
                (TargetSize::Size5MB, "5 MB"), 
                (TargetSize::Size10MB, "10 MB"),
                (TargetSize::Size30MB, "30 MB"),
                (TargetSize::Size50MB, "50 MB"),
            ];
            
            for (size, label) in sizes {
                let selected = self.state.compression_settings.target_size == size;
                if ui.selectable_label(selected, label).clicked() {
                    self.state.compression_settings.target_size = size;
                }
            }
        });
    }
    
    
    fn draw_controls_section(&mut self, ui: &mut egui::Ui) {
        // Progress bar
        if self.state.status != CompressionStatus::Idle {
            ui.add(egui::ProgressBar::new(self.state.progress)
                .text(format!("{}%", (self.state.progress * 100.0) as u32)));
        }
        
        ui.add_space(5.0);
        
        // Control buttons
        ui.horizontal_centered(|ui| {
            match self.state.status {
                CompressionStatus::Idle => {
                    let compress_enabled = self.state.input_file.is_some();
                    if ui.add_enabled(compress_enabled, 
                        egui::Button::new(format!("🎬 {}", self.get_text("compress")))
                    ).clicked() {
                        self.start_compression();
                    }
                },
                CompressionStatus::Processing => {
                    if ui.button(format!("⏹️ {}", self.get_text("stop"))).clicked() {
                        self.stop_compression();
                    }
                    
                    ui.add_space(10.0);
                    
                    if ui.button(format!("❌ {}", self.get_text("cancel"))).clicked() {
                        self.cancel_compression();
                    }
                },
                _ => {}
            }
        });
    }
    
    fn draw_menu_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.small_button("Advanced").clicked() {
                self.show_advanced = !self.show_advanced;
            }
            
            if ui.small_button("About").clicked() {
                self.show_about = !self.show_about;
            }
            
            // Language selector
            egui::ComboBox::from_label("Language")
                .selected_text(match self.config.language {
                    Language::English => "English",
                    Language::Korean => "한국어", 
                    Language::Japanese => "日本語",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.config.language, Language::English, "English");
                    ui.selectable_value(&mut self.config.language, Language::Korean, "한국어");
                    ui.selectable_value(&mut self.config.language, Language::Japanese, "日本語");
                });
        });
    }
    
    fn draw_advanced_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("Advanced Settings")
            .open(&mut self.show_advanced)
            .show(ctx, |ui| {
                ui.label("Hardware Acceleration:");
                
                // Show detected hardware
                if let Ok(hw_caps) = self.hardware_capabilities.try_lock() {
                    if let Some(ref caps) = *hw_caps {
                        ui.label(format!("Available encoders: {}", caps.available_encoders.len()));
                        if let Some(ref preferred) = caps.preferred_encoder {
                            ui.label(format!("Recommended: {:?}", preferred));
                        }
                    } else {
                        ui.label("🔍 Detecting hardware...");
                    }
                } else {
                    ui.label("⚙️ Hardware detection in progress...");
                }
                
                ui.separator();
                
                ui.checkbox(&mut self.state.compression_settings.enable_hardware_accel, 
                    "Enable hardware acceleration");
                ui.checkbox(&mut self.state.compression_settings.memory_optimization, 
                    "Memory optimization");
            });
    }
    
    fn draw_about_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("About")
            .open(&mut self.show_about)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Small MP4");
                    ui.label("Version 0.2.0");
                    ui.add_space(10.0);
                    ui.label("Squeeze your videos for easy sharing");
                    ui.label("동영상 공유를 위해서 영상을 꾸겨줍니다");
                    ui.label("動画共有のために映像を圧縮します");
                    ui.add_space(10.0);
                    ui.hyperlink_to("GitHub", "https://github.com/small-mp4/small-mp4-rs");
                });
            });
    }
    
    fn start_compression(&mut self) {
        self.state.status = CompressionStatus::Processing;
        self.state.progress = 0.0;
        
        // TODO: Start actual compression in background
        log::info!("Starting compression...");
    }
    
    fn stop_compression(&mut self) {
        self.state.status = CompressionStatus::Paused;
        log::info!("Compression stopped");
    }
    
    fn cancel_compression(&mut self) {
        self.state.status = CompressionStatus::Idle;
        self.state.progress = 0.0;
        log::info!("Compression cancelled");
    }
}