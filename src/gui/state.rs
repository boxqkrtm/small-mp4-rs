use std::path::PathBuf;
use crate::compression::CompressionSettings;

/// Application state management
#[derive(Debug, Clone)]
pub struct AppState {
    // File handling
    pub input_file: Option<PathBuf>,
    pub output_file: Option<PathBuf>,
    pub output_folder: Option<PathBuf>,
    pub same_folder: bool,
    
    // Compression settings
    pub compression_settings: CompressionSettings,
    
    // UI state
    pub status: CompressionStatus,
    pub progress: f32,
    pub current_operation: String,
    pub estimated_time: Option<std::time::Duration>,
    
    // Preview state
    pub original_preview: Option<PreviewData>,
    pub compressed_preview: Option<PreviewData>,
    
    // Error handling
    pub last_error: Option<String>,
    
    // UI popups
    pub show_completion_popup: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompressionStatus {
    Idle,
    Preparing,
    Processing,
    Paused,
    Completed,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct PreviewData {
    pub thumbnail: Option<Vec<u8>>, // RGBA image data
    pub width: u32,
    pub height: u32,
    pub duration: f64,
    pub file_size: u64,
    pub codec: String,
    pub bitrate: u32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            input_file: None,
            output_file: None,
            output_folder: None,
            same_folder: true,
            compression_settings: CompressionSettings::default(),
            status: CompressionStatus::Idle,
            progress: 0.0,
            current_operation: String::new(),
            estimated_time: None,
            original_preview: None,
            compressed_preview: None,
            last_error: None,
            show_completion_popup: false,
        }
    }
}

impl AppState {
    pub fn set_input_file(&mut self, path: PathBuf) {
        self.input_file = Some(path.clone());
        
        // Auto-generate output file if same folder is selected
        if self.same_folder {
            if let Some(parent) = path.parent() {
                if let Some(stem) = path.file_stem() {
                    let output_name = format!("{}_small.mp4", stem.to_string_lossy());
                    self.output_file = Some(parent.join(output_name));
                }
            }
        }
        
        // Reset preview data
        self.original_preview = None;
        self.compressed_preview = None;
    }
    
    pub fn get_output_path(&self) -> Option<PathBuf> {
        if let Some(ref output) = self.output_file {
            Some(output.clone())
        } else if let (Some(ref input), Some(ref folder)) = (&self.input_file, &self.output_folder) {
            if let Some(stem) = input.file_stem() {
                let output_name = format!("{}_small.mp4", stem.to_string_lossy());
                Some(folder.join(output_name))
            } else {
                None
            }
        } else {
            None
        }
    }
    
    pub fn is_ready_to_compress(&self) -> bool {
        self.input_file.is_some() && 
        self.status == CompressionStatus::Idle &&
        self.get_output_path().is_some()
    }
    
    pub fn update_progress(&mut self, progress: f32, operation: String) {
        self.progress = progress.clamp(0.0, 1.0);
        self.current_operation = operation;
    }
    
    pub fn set_error(&mut self, error: String) {
        self.status = CompressionStatus::Error(error.clone());
        self.last_error = Some(error);
    }
    
    pub fn clear_error(&mut self) {
        self.last_error = None;
        if let CompressionStatus::Error(_) = self.status {
            self.status = CompressionStatus::Idle;
        }
    }
    
    pub fn reset_compression(&mut self) {
        self.status = CompressionStatus::Idle;
        self.progress = 0.0;
        self.current_operation.clear();
        self.estimated_time = None;
        self.compressed_preview = None;
        self.clear_error();
    }
    
    pub fn get_target_size_mb(&self) -> f32 {
        self.compression_settings.target_size.as_mb()
    }
}

/// UI component state for managing different parts of the interface
#[derive(Debug, Default)]
pub struct ComponentState {
    pub drop_zone_hovered: bool,
    pub size_slider_dragging: bool,
    pub preview_loading: bool,
    pub advanced_expanded: bool,
}

/// Progress tracking for different compression stages
#[derive(Debug, Clone)]
pub struct ProgressState {
    pub stage: CompressionStage,
    pub stage_progress: f32,
    pub overall_progress: f32,
    pub estimated_remaining: Option<std::time::Duration>,
    pub current_fps: Option<f32>,
    pub frames_processed: u64,
    pub total_frames: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompressionStage {
    Initializing,
    Analyzing,
    Encoding,
    Finalizing,
}

impl Default for ProgressState {
    fn default() -> Self {
        Self {
            stage: CompressionStage::Initializing,
            stage_progress: 0.0,
            overall_progress: 0.0,
            estimated_remaining: None,
            current_fps: None,
            frames_processed: 0,
            total_frames: None,
        }
    }
}

impl ProgressState {
    pub fn stage_name(&self) -> &'static str {
        match self.stage {
            CompressionStage::Initializing => "Initializing...",
            CompressionStage::Analyzing => "Analyzing video...",
            CompressionStage::Encoding => "Encoding...",
            CompressionStage::Finalizing => "Finalizing...",
        }
    }
    
    pub fn stage_weight(&self) -> f32 {
        match self.stage {
            CompressionStage::Initializing => 0.05,
            CompressionStage::Analyzing => 0.10,
            CompressionStage::Encoding => 0.80,
            CompressionStage::Finalizing => 0.05,
        }
    }
    
    pub fn update_progress(&mut self, stage_progress: f32) {
        self.stage_progress = stage_progress.clamp(0.0, 1.0);
        
        // Calculate overall progress based on stage weights
        let stage_offset = match self.stage {
            CompressionStage::Initializing => 0.0,
            CompressionStage::Analyzing => 0.05,
            CompressionStage::Encoding => 0.15,
            CompressionStage::Finalizing => 0.95,
        };
        
        self.overall_progress = stage_offset + (self.stage_weight() * self.stage_progress);
        self.overall_progress = self.overall_progress.clamp(0.0, 1.0);
    }
    
    pub fn advance_stage(&mut self, new_stage: CompressionStage) {
        self.stage = new_stage;
        self.stage_progress = 0.0;
    }
}
