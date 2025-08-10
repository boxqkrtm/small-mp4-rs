use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

pub mod hardware_cli;

use hardware_cli::{HardwareEncoderCli, HardwarePresetCli, HardwareQualityCli};

#[derive(Parser)]
#[command(name = "small-mp4")]
#[command(about = "Squeeze your videos for easy sharing - Now with hardware acceleration!")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Compress a video file
    Compress {
        /// Input video file
        input: PathBuf,
        
        /// Output file (optional, defaults to input_compressed.mp4)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        #[command(flatten)]
        settings: CompressionCliSettings,
    },
    
    /// Launch GUI interface
    Gui,
    
    /// List available hardware encoders
    #[command(name = "list-hw")]
    ListHardware,
}

#[derive(clap::Args)]
pub struct CompressionCliSettings {
    /// Target size preset
    #[arg(short, long, value_enum, default_value = "10mb")]
    pub size: SizePreset,
    
    /// Hardware encoder to use
    #[arg(long, value_enum)]
    pub hw_encoder: Option<HardwareEncoderCli>,
    
    /// Hardware encoding preset
    #[arg(long, value_enum, default_value = "medium")]
    pub hw_preset: HardwarePresetCli,
    
    /// Hardware quality control
    #[arg(long, value_enum, default_value = "auto")]
    pub hw_quality: HardwareQualityCli,
    
    /// Specific CUDA device ID (0, 1, 2, etc.)
    #[arg(long)]
    pub cuda_device: Option<u32>,
    
    /// Force software encoding (disable hardware acceleration)
    #[arg(long)]
    pub force_software: bool,
    
    /// Enable memory optimization for hardware encoding
    #[arg(long)]
    pub memory_opt: bool,
    
    /// Compatibility mode - Force x264 codec for maximum compatibility
    #[arg(long)]
    pub compatibility: bool,
    
    /// Language for output messages
    #[arg(short, long, value_enum, default_value = "en")]
    pub lang: Language,
}

#[derive(Clone, ValueEnum, Debug)]
pub enum SizePreset {
    #[value(name = "1mb")]
    Size1MB,
    #[value(name = "5mb")] 
    Size5MB,
    #[value(name = "10mb")]
    Size10MB,
    #[value(name = "30mb")]
    Size30MB,
    #[value(name = "50mb")]
    Size50MB,
    #[value(name = "100mb")]
    Size100MB,
    #[value(name = "250mb")]
    Size250MB,
    #[value(name = "500mb")]
    Size500MB,
    #[value(name = "1gb")]
    Size1GB,
}

impl SizePreset {
    pub fn as_mb(&self) -> f32 {
        match self {
            SizePreset::Size1MB => 1.0,
            SizePreset::Size5MB => 5.0,
            SizePreset::Size10MB => 10.0,
            SizePreset::Size30MB => 30.0,
            SizePreset::Size50MB => 50.0,
            SizePreset::Size100MB => 100.0,
            SizePreset::Size250MB => 250.0,
            SizePreset::Size500MB => 500.0,
            SizePreset::Size1GB => 1000.0,
        }
    }
}

#[derive(Clone, ValueEnum, Debug)]
pub enum Language {
    #[value(name = "ko")]
    Korean,
    #[value(name = "en")]
    English,
    #[value(name = "ja")]
    Japanese,
}

// Usage examples that can be shown in help:
// small-mp4 compress video.mov --size 10mb --hw-encoder nvenc-h264 --hw-preset fast
// small-mp4 compress video.mov --auto --hw-encoder auto --cuda-device 0
// small-mp4 list-hw  # List available hardware encoders
// small-mp4 compress video.mov --force-software  # Disable hardware acceleration
