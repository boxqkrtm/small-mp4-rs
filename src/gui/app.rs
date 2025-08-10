use eframe::egui::{self};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::compression::{CompressionEngine, TargetSize};
use crate::compression::hardware::HardwareCapabilities;
use super::components::{DropZone, SizeSlider, PreviewPanel, ProgressBar};
use super::state::{AppState, CompressionStatus};
use super::{GuiConfig, Language};

pub struct SmallMp4App {
    pub config: GuiConfig,
    pub state: Arc<Mutex<AppState>>,
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
            state: Arc::new(Mutex::new(AppState::default())),
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
        let app = Self::default();
        
        // Set hardware capabilities
        *app.hardware_capabilities.lock().unwrap() = Some(hw_capabilities.clone());
        
        // Initialize compression engine
        *app.compression_engine.lock().unwrap() = Some(CompressionEngine::new(hw_capabilities.clone()));
        
        // Update compression settings to use available hardware by default
        if let Ok(mut state_guard) = app.state.lock() {
            if let Some(preferred_encoder) = &hw_capabilities.preferred_encoder {
                log::info!("Setting default hardware encoder to: {:?}", preferred_encoder);
                state_guard.compression_settings.hardware_encoder = preferred_encoder.clone();
                state_guard.compression_settings.enable_hardware_accel = true;
            }
        }
        
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
                "drag_drop" => "동영상 파일을 선택해주세요".to_string(),
                "browse" => "찾아보기".to_string(),
                "target_size" => "목표 크기:".to_string(),
                "auto" => "자동".to_string(),
                "compress" => "압축하기".to_string(),
                "stop" => "중지".to_string(),
                "cancel" => "취소".to_string(),
                "original" => "원본".to_string(),
                "preview" => "미리보기".to_string(),
                "advanced" => "고급 설정".to_string(),
                "about" => "정보".to_string(),
                "language" => "언어".to_string(),
                "hardware_acceleration" => "하드웨어 가속:".to_string(),
                "available_encoders" => "사용 가능한 인코더".to_string(),
                "recommended" => "권장".to_string(),
                "detecting_hardware" => "🔍 하드웨어 감지 중...".to_string(),
                "hardware_detection_progress" => "⚙️ 하드웨어 감지 진행 중...".to_string(),
                "enable_hardware_accel" => "하드웨어 가속 활성화".to_string(),
                "memory_optimization" => "메모리 최적화".to_string(),
                "advanced_settings" => "고급 설정".to_string(),
                "compatibility_mode" => "호환성 모드 (x264 only)".to_string(),
                _ => key.to_string(),
            },
            Language::Japanese => match key {
                "title" => "小さなmp4 - 動画圧縮ツール".to_string(), 
                "drag_drop" => "動画ファイルを選択してください".to_string(),
                "browse" => "参照".to_string(),
                "target_size" => "目標サイズ:".to_string(),
                "auto" => "自動".to_string(),
                "compress" => "圧縮".to_string(),
                "stop" => "停止".to_string(), 
                "cancel" => "キャンセル".to_string(),
                "original" => "元の動画".to_string(),
                "preview" => "プレビュー".to_string(),
                "advanced" => "詳細設定".to_string(),
                "about" => "について".to_string(),
                "language" => "言語".to_string(),
                "hardware_acceleration" => "ハードウェアアクセラレーション:".to_string(),
                "available_encoders" => "利用可能エンコーダー".to_string(),
                "recommended" => "推奨".to_string(),
                "detecting_hardware" => "🔍 ハードウェア検出中...".to_string(),
                "hardware_detection_progress" => "⚙️ ハードウェア検出進行中...".to_string(),
                "enable_hardware_accel" => "ハードウェアアクセラレーション有効化".to_string(),
                "memory_optimization" => "メモリ最適化".to_string(),
                "advanced_settings" => "詳細設定".to_string(),
                "compatibility_mode" => "互換性モード (x264のみ)".to_string(),
                _ => key.to_string(),
            },
            Language::English => match key {
                "title" => "Small MP4 - Video Compressor".to_string(),
                "drag_drop" => "Please select a video file".to_string(),
                "browse" => "Browse...".to_string(),
                "target_size" => "Target Size:".to_string(),
                "auto" => "Auto".to_string(),
                "compress" => "Compress".to_string(),
                "stop" => "Stop".to_string(),
                "cancel" => "Cancel".to_string(), 
                "original" => "Original".to_string(),
                "preview" => "Preview".to_string(),
                "advanced" => "Advanced".to_string(),
                "about" => "About".to_string(),
                "language" => "Language".to_string(),
                "hardware_acceleration" => "Hardware Acceleration:".to_string(),
                "available_encoders" => "Available Encoders".to_string(),
                "recommended" => "Recommended".to_string(),
                "detecting_hardware" => "🔍 Detecting hardware...".to_string(),
                "hardware_detection_progress" => "⚙️ Hardware detection in progress...".to_string(),
                "enable_hardware_accel" => "Enable hardware acceleration".to_string(),
                "memory_optimization" => "Memory optimization".to_string(),
                "advanced_settings" => "Advanced Settings".to_string(),
                "compatibility_mode" => "Compatibility mode (x264 only)".to_string(),
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
                        if let Ok(mut state_guard) = self.state.lock() {
                            state_guard.input_file = Some(path.clone());
                        }
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
        
        // Completion popup
        self.draw_completion_popup(ctx);
        
        // Request repaint for smooth animations
        if let Ok(state_guard) = self.state.lock() {
            if state_guard.status != CompressionStatus::Idle {
                ctx.request_repaint();
            }
        }
    }
}

impl SmallMp4App {
    fn draw_main_ui(&mut self, ui: &mut egui::Ui) {
        // Main content area first
        ui.vertical(|ui| {
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
            
            // Add flexible space
            ui.add_space(ui.available_height() - 50.0);
            
            // Menu bar at bottom
            ui.separator();
            self.draw_menu_bar(ui);
            ui.add_space(5.0);
        });
    }
    
    fn draw_input_section(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Input file
            ui.label("Input:");
            ui.horizontal(|ui| {
                // File path display
                let file_text = {
                    if let Ok(state_guard) = self.state.lock() {
                        state_guard.input_file
                            .as_ref()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| self.get_text("drag_drop").to_string())
                    } else {
                        self.get_text("drag_drop").to_string()
                    }
                };
                    
                ui.add_sized([350.0, 25.0], egui::TextEdit::singleline(&mut file_text.as_str())
                    .interactive(false));
                
                // Browse button
                if ui.button(self.get_text("browse")).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Video files", &["mp4", "avi", "mov", "mkv", "flv", "wmv"])
                        .pick_file() 
                    {
                        if let Ok(mut state_guard) = self.state.lock() {
                            state_guard.set_input_file(path);
                        }
                    }
                }
            });
            
            ui.add_space(5.0);
            
            // Output options
            ui.label("Output:");
            ui.horizontal(|ui| {
                let (same_folder, output_folder) = {
                    if let Ok(state_guard) = self.state.lock() {
                        (state_guard.same_folder, state_guard.output_folder.clone())
                    } else {
                        (true, None)
                    }
                };
                
                let mut same_folder_checkbox = same_folder;
                ui.checkbox(&mut same_folder_checkbox, "Same folder");
                
                if same_folder_checkbox != same_folder {
                    if let Ok(mut state_guard) = self.state.lock() {
                        state_guard.same_folder = same_folder_checkbox;
                    }
                }
                
                // Show output folder selection when not using same folder
                if !same_folder_checkbox {
                    ui.separator();
                    
                    let folder_text = output_folder
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Select output folder...".to_string());
                    
                    ui.add_sized([200.0, 25.0], egui::TextEdit::singleline(&mut folder_text.as_str())
                        .interactive(false));
                    
                    if ui.button("📁 Choose").clicked() {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            if let Ok(mut state_guard) = self.state.lock() {
                                state_guard.output_folder = Some(folder);
                            }
                        }
                    }
                }
            });
        });
    }
    
    fn draw_size_section(&mut self, ui: &mut egui::Ui) {
        ui.label(self.get_text("target_size"));
        
        // Size buttons in two rows
        ui.horizontal(|ui| {
            // First row: 1, 5, 10, 30, 50 MB
            let sizes_row1 = [
                (TargetSize::Size1MB, "1 MB"),
                (TargetSize::Size5MB, "5 MB"), 
                (TargetSize::Size10MB, "10 MB"),
                (TargetSize::Size30MB, "30 MB"),
                (TargetSize::Size50MB, "50 MB"),
            ];
            
            let current_target_size = {
                if let Ok(state_guard) = self.state.lock() {
                    state_guard.compression_settings.target_size.clone()
                } else {
                    TargetSize::Size10MB
                }
            };
            
            for (size, label) in sizes_row1 {
                let selected = current_target_size == size;
                if ui.selectable_label(selected, label).clicked() {
                    if let Ok(mut state_guard) = self.state.lock() {
                        state_guard.compression_settings.target_size = size;
                    }
                }
            }
        });
        
        ui.horizontal(|ui| {
            // Second row: 100, 250, 500, 1000 MB
            let sizes_row2 = [
                (TargetSize::Size100MB, "100 MB"),
                (TargetSize::Size250MB, "250 MB"), 
                (TargetSize::Size500MB, "500 MB"),
                (TargetSize::Size1000MB, "1 GB"),
            ];
            
            let current_target_size = {
                if let Ok(state_guard) = self.state.lock() {
                    state_guard.compression_settings.target_size.clone()
                } else {
                    TargetSize::Size10MB
                }
            };
            
            for (size, label) in sizes_row2 {
                let selected = current_target_size == size;
                if ui.selectable_label(selected, label).clicked() {
                    if let Ok(mut state_guard) = self.state.lock() {
                        state_guard.compression_settings.target_size = size;
                    }
                }
            }
        });
        
        ui.add_space(5.0);
        
        // Compatibility mode checkbox (important - show in main UI)
        let mut compatibility_mode = {
            if let Ok(state_guard) = self.state.lock() {
                state_guard.compression_settings.compatibility_mode
            } else {
                true // Default to true for maximum compatibility
            }
        };
        
        ui.checkbox(&mut compatibility_mode, self.get_text("compatibility_mode"));
        
        if let Ok(mut state_guard) = self.state.lock() {
            state_guard.compression_settings.compatibility_mode = compatibility_mode;
        }
    }
    
    
    fn draw_controls_section(&mut self, ui: &mut egui::Ui) {
        let (status, progress, has_input_file, eta) = {
            if let Ok(state_guard) = self.state.lock() {
                (
                    state_guard.status.clone(), 
                    state_guard.progress, 
                    state_guard.input_file.is_some(),
                    state_guard.estimated_time.clone()
                )
            } else {
                (CompressionStatus::Idle, 0.0, false, None)
            }
        };
        
        // Progress bar - always visible
        let progress_text = if status != CompressionStatus::Idle {
            if let Some(time_left) = eta {
                let seconds = time_left.as_secs();
                let mins = seconds / 60;
                let secs = seconds % 60;
                if mins > 0 {
                    format!("{}% ({}:{:02} 남음)", (progress * 100.0) as u32, mins, secs)
                } else {
                    format!("{}% ({}초 남음)", (progress * 100.0) as u32, secs)
                }
            } else {
                format!("{}%", (progress * 100.0) as u32)
            }
        } else {
            "0%".to_string()
        };
        
        ui.add(egui::ProgressBar::new(progress).text(progress_text));
        
        ui.add_space(5.0);
        
        // Control buttons - not horizontally centered anymore
        match status {
            CompressionStatus::Idle => {
                let compress_enabled = has_input_file;
                if ui.add_enabled(compress_enabled, 
                    egui::Button::new(format!("🎬 {}", self.get_text("compress")))
                        .min_size(egui::vec2(200.0, 35.0))
                ).clicked() {
                    self.start_compression();
                }
            },
            CompressionStatus::Processing => {
                ui.horizontal(|ui| {
                    if ui.button(format!("⏹️ {}", self.get_text("stop"))).clicked() {
                        self.stop_compression();
                    }
                    
                    ui.add_space(10.0);
                    
                    if ui.button(format!("❌ {}", self.get_text("cancel"))).clicked() {
                        self.cancel_compression();
                    }
                });
            },
            _ => {}
        }
    }
    
    fn draw_menu_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.small_button(self.get_text("advanced")).clicked() {
                self.show_advanced = !self.show_advanced;
            }
            
            if ui.small_button(self.get_text("about")).clicked() {
                self.show_about = !self.show_about;
            }
            
            // Language selector
            egui::ComboBox::from_label(self.get_text("language"))
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
        // Pre-collect all text to avoid borrow issues
        let window_title = self.get_text("advanced_settings");
        let hw_accel_text = self.get_text("hardware_acceleration");
        let available_encoders_text = self.get_text("available_encoders");
        let recommended_text = self.get_text("recommended");
        let detecting_hw_text = self.get_text("detecting_hardware");
        let detection_progress_text = self.get_text("hardware_detection_progress");
        let enable_hw_accel_text = self.get_text("enable_hardware_accel");
        let memory_opt_text = self.get_text("memory_optimization");
        
        egui::Window::new(window_title)
            .open(&mut self.show_advanced)
            .show(ctx, |ui| {
                ui.label(hw_accel_text);
                
                // Show detected hardware
                if let Ok(hw_caps) = self.hardware_capabilities.try_lock() {
                    if let Some(ref caps) = *hw_caps {
                        ui.label(format!("{}: {}", available_encoders_text, caps.available_encoders.len()));
                        if let Some(ref preferred) = caps.preferred_encoder {
                            ui.label(format!("{}: {:?}", recommended_text, preferred));
                        }
                    } else {
                        ui.label(detecting_hw_text);
                    }
                } else {
                    ui.label(detection_progress_text);
                }
                
                ui.separator();
                
                let (mut enable_hw_accel, mut memory_opt) = {
                    if let Ok(state_guard) = self.state.lock() {
                        (state_guard.compression_settings.enable_hardware_accel,
                         state_guard.compression_settings.memory_optimization)
                    } else {
                        (true, false)
                    }
                };
                
                ui.checkbox(&mut enable_hw_accel, &enable_hw_accel_text);
                ui.checkbox(&mut memory_opt, &memory_opt_text);
                
                // Update state if changed
                if let Ok(mut state_guard) = self.state.lock() {
                    state_guard.compression_settings.enable_hardware_accel = enable_hw_accel;
                    state_guard.compression_settings.memory_optimization = memory_opt;
                }
            });
    }
    
    fn draw_about_window(&mut self, ctx: &egui::Context) {
        let about_title = self.get_text("about");
        egui::Window::new(about_title)
            .open(&mut self.show_about)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Small MP4");
                    ui.label("Version 0.2.0");
                    ui.add_space(10.0);
                    
                    // Show description in selected language only
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            let (flag, description) = match self.config.language {
                                Language::Korean => ("🇰🇷", "동영상 공유를 위해서 영상을 꾸겨줍니다"),
                                Language::Japanese => ("🇯🇵", "動画共有のために映像を圧縮します"),
                                Language::English => ("🇺🇸", "Squeeze your videos for easy sharing"),
                            };
                            
                            ui.label(format!("{} {}", flag, match self.config.language {
                                Language::Korean => "한국어",
                                Language::Japanese => "日本語", 
                                Language::English => "English",
                            }));
                            ui.label(description);
                        });
                    });
                    
                    ui.add_space(10.0);
                    ui.hyperlink_to("GitHub", "https://github.com/small-mp4/small-mp4-rs");
                });
            });
    }
    
    fn start_compression(&mut self) {
        // Validate input file exists
        let (input_file, output_path) = {
            let mut state_guard = self.state.lock().unwrap();
            let input_file = match &state_guard.input_file {
                Some(file) => file.clone(),
                None => {
                    state_guard.set_error("No input file selected".to_string());
                    return;
                }
            };
            
            // Get output path
            let output_path = match state_guard.get_output_path() {
                Some(path) => path,
                None => {
                    state_guard.set_error("Could not determine output path".to_string());
                    return;
                }
            };
            
            (input_file, output_path)
        };
        
        {
            let mut state_guard = self.state.lock().unwrap();
            state_guard.status = CompressionStatus::Processing;
            state_guard.progress = 0.0;
        }
        log::info!("Starting compression...");
        log::info!("Input: {}", input_file.display());
        log::info!("Output: {}", output_path.display());
        
        // Clone what we need for the async task
        let settings = {
            let state_guard = self.state.lock().unwrap();
            state_guard.compression_settings.clone()
        };
        
        // Get compression engine and state (clone the Arc to move into async context)
        let engine = self.compression_engine.clone();
        let app_state = self.state.clone();
        
        // Spawn compression task in a separate thread using std::thread
        std::thread::spawn(move || {
            // Use a blocking runtime for this thread
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(async move {
                // Try to get a lock on the engine
                let result = {
                    let mut engine_guard = match engine.lock() {
                        Ok(guard) => guard,
                        Err(_) => {
                            log::error!("Could not lock compression engine (poisoned)");
                            return;
                        }
                    };
                    
                    if let Some(ref mut compression_engine) = *engine_guard {
                        // Create progress channel
                        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel();
                        
                        // Spawn task to receive progress updates
                        let app_state_progress = app_state.clone();
                        tokio::task::spawn(async move {
                            while let Some((progress, eta)) = progress_rx.recv().await {
                                if let Ok(mut state_guard) = app_state_progress.lock() {
                                    state_guard.progress = progress;
                                    state_guard.estimated_time = eta;
                                }
                            }
                        });
                        
                        compression_engine.compress(&input_file, Some(&output_path), &settings, Some(progress_tx)).await
                    } else {
                        log::error!("Compression engine not initialized");
                        return;
                    }
                };
                
                match result {
                    Ok(result) => {
                        log::info!("Compression completed successfully!");
                        log::info!("Input size: {:.1} MB, Output size: {:.1} MB", 
                            result.input_size_mb, result.output_size_mb);
                        
                        // Update GUI state to idle and show completion
                        if let Ok(mut state_guard) = app_state.lock() {
                            state_guard.status = CompressionStatus::Idle;
                            state_guard.progress = 1.0; // 100% complete
                            state_guard.last_compression_result = Some((result.input_size_mb, result.output_size_mb));
                            state_guard.show_completion_popup = true; // Show completion popup
                        }
                    }
                    Err(e) => {
                        log::error!("Compression failed: {}", e);
                        
                        // Update GUI state to idle with error
                        if let Ok(mut state_guard) = app_state.lock() {
                            state_guard.status = CompressionStatus::Idle;
                            state_guard.progress = 0.0;
                            state_guard.set_error(format!("Compression failed: {}", e));
                        }
                    }
                }
            });
        });
    }
    
    fn stop_compression(&mut self) {
        if let Ok(mut state_guard) = self.state.lock() {
            state_guard.status = CompressionStatus::Paused;
        }
        log::info!("Compression stopped");
    }
    
    fn cancel_compression(&mut self) {
        if let Ok(mut state_guard) = self.state.lock() {
            state_guard.status = CompressionStatus::Idle;
            state_guard.progress = 0.0;
        }
        log::info!("Compression cancelled");
    }
    
    fn draw_completion_popup(&mut self, ctx: &egui::Context) {
        let (show_popup, compression_result) = {
            if let Ok(state_guard) = self.state.lock() {
                (state_guard.show_completion_popup, state_guard.last_compression_result)
            } else {
                (false, None)
            }
        };
        
        if show_popup {
            let (title, message, size_before_text, size_after_text, button_text) = match self.config.language {
                Language::Korean => (
                    "압축 완료!",
                    "✅ 비디오 압축이 완료되었습니다!",
                    "압축 전:",
                    "압축 후:",
                    "확인"
                ),
                Language::Japanese => (
                    "圧縮完了！",
                    "✅ ビデオ圧縮が完了しました！",
                    "圧縮前:",
                    "圧縮後:",
                    "OK"
                ),
                Language::English => (
                    "Compression Complete!",
                    "✅ Video compression completed successfully!",
                    "Before:",
                    "After:",
                    "OK"
                ),
            };
            
            egui::Window::new(title)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.label(message);
                        ui.add_space(10.0);
                        
                        // Show file sizes if available
                        if let Some((input_size, output_size)) = compression_result {
                            ui.separator();
                            ui.add_space(5.0);
                            
                            // Size comparison in a nice layout
                            ui.horizontal(|ui| {
                                ui.label(size_before_text);
                                ui.strong(format!("{:.1} MB", input_size));
                                ui.label("→");
                                ui.label(size_after_text);
                                ui.strong(format!("{:.1} MB", output_size));
                            });
                            
                            // Show compression ratio
                            let compression_ratio = (1.0 - output_size / input_size) * 100.0;
                            ui.add_space(5.0);
                            ui.label(format!("({:.1}% smaller)", compression_ratio));
                            
                            ui.add_space(5.0);
                        }
                        
                        ui.separator();
                        ui.add_space(5.0);
                        
                        ui.horizontal(|ui| {
                            if ui.button(button_text).clicked() {
                                if let Ok(mut state_guard) = self.state.lock() {
                                    state_guard.show_completion_popup = false;
                                }
                            }
                        });
                        ui.add_space(5.0);
                    });
                });
        }
    }
}
