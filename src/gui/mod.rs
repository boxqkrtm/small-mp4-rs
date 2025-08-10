pub mod app;
pub mod components;
pub mod state;
pub mod utils;

pub use app::SmallMp4App;

// GUI module
use std::path::PathBuf;

/// Main GUI configuration and theme settings
#[derive(Debug, Clone)]
pub struct GuiConfig {
    pub theme: Theme,
    pub language: Language,
    pub auto_save_location: bool,
    pub remember_settings: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    English,
    Korean,
    Japanese,
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            theme: Theme::Auto,
            language: Self::detect_system_language(),
            auto_save_location: true,
            remember_settings: true,
        }
    }
}

impl GuiConfig {
    fn detect_system_language() -> Language {
        // Get system language from environment
        if let Ok(lang) = std::env::var("LANG") {
            let lang = lang.to_lowercase();
            if lang.starts_with("ko") || lang.contains("korean") {
                return Language::Korean;
            } else if lang.starts_with("ja") || lang.contains("japanese") {
                return Language::Japanese;
            }
        }
        
        // Fallback to LANGUAGE variable
        if let Ok(lang) = std::env::var("LANGUAGE") {
            let lang = lang.to_lowercase();
            if lang.starts_with("ko") {
                return Language::Korean;
            } else if lang.starts_with("ja") {
                return Language::Japanese;
            }
        }
        
        // Default to English
        Language::English
    }
}

/// GUI-specific errors
#[derive(thiserror::Error, Debug)]
pub enum GuiError {
    #[error("Failed to open file dialog")]
    FileDialogError,
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    #[error("Compression error: {0}")]
    CompressionError(String),
}

pub type Result<T> = std::result::Result<T, GuiError>;