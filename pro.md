# 작은mp4 (Small MP4) - Video Compression Tool

> **English**: Squeeze your videos for easy sharing  
> **한국어**: 동영상 공유를 위해서 영상을 꾸겨줍니다  
> **日本語**: 動画共有のために映像を圧縮します

## 📋 Project Overview

### Core Identity

- **Project Name**: `작은mp4` / `small-mp4`
- **Repository**: `small-mp4`
- **Tagline**:
  - **English**: “Squeeze your videos for easy sharing”
  - **한국어**: “동영상 공유를 위해서 영상을 꾸겨줍니다”
  - **日本語**: “動画共有のために映像を圧縮します”

### Key Features

- 🎚️ **Simple size slider**: 1, 5, 10, 30, 50 MB presets
- 🎬 **Universal MP4 output**: All formats → MP4
- 👁️ **Live preview**: Quality check before processing
- ⚡ **Progress control**: Start/stop/cancel operations
- 📁 **Smart file handling**: Auto-naming with collision detection
- 🌍 **Multi-language**: Korean, Japanese, English

## 🎨 User Interface Design

### Main Window Layout

```
┌─────────────────────────────────────────┐
│ 작은mp4 - Video Compressor              │
├─────────────────────────────────────────┤
│ Input: [Browse...] [📁] video.mov       │
│ Output: [Browse...] ☑️ Same folder      │
│                                         │
│ Target Size:                            │
│ ☑️ Auto  [●────○────○────○────○] 50 MB  │
│           1    5   10   30   50         │
│ Quality-based estimation                │
├─────────────────────────────────────────┤
│ ┌─────────────────┐ ┌─────────────────┐ │
│ │   Original      │ │   Preview       │ │
│ │   [Preview]     │ │   [Quality]     │ │
│ └─────────────────┘ └─────────────────┘ │
├─────────────────────────────────────────┤
│ Progress: ████████░░ 80%                │
│ [🎬 압축하기] [⏹️ 중지] [❌ 취소]         │
└─────────────────────────────────────────┘
```

## 🔧 Core Implementation

### Size Slider Component

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TargetSize {
    Size1MB = 1,
    Size5MB = 5,
    Size10MB = 10,
    Size30MB = 30,
    Size50MB = 50,
}

impl TargetSize {
    pub const ALL: [TargetSize; 5] = [
        TargetSize::Size1MB,
        TargetSize::Size5MB,
        TargetSize::Size10MB,
        TargetSize::Size30MB,
        TargetSize::Size50MB,
    ];
    
    pub fn as_mb(&self) -> f32 {
        *self as u8 as f32
    }
    
    pub fn from_index(index: usize) -> Option<TargetSize> {
        Self::ALL.get(index).copied()
    }
    
    pub fn to_index(&self) -> usize {
        Self::ALL.iter().position(|&s| s == *self).unwrap_or(4)
    }
}

#[derive(Debug, Clone)]
pub struct CompressionSettings {
    pub auto_size: bool,
    pub target_size: TargetSize,
    pub estimated_size_mb: Option<f32>,
    pub crf_quality: u8,
}

impl Default for CompressionSettings {
    fn default() -> Self {
        Self {
            auto_size: true,
            target_size: TargetSize::Size50MB,
            estimated_size_mb: None,
            crf_quality: 26,
        }
    }
}
```

### Enhanced Size Estimation

```rust
impl SizeEstimator {
    /// 각 크기별 최적 CRF 추천
    pub fn recommend_crf_for_size(
        &self,
        metadata: &VideoMetadata,
        target_size: TargetSize,
    ) -> Result<CrfRecommendation, EstimationError> {
        let target_mb = target_size.as_mb();
        let auto_size = self.estimate_size_crf26(metadata, 128)?;
        
        let crf = if auto_size <= target_mb {
            // 자동 추정이 목표보다 작으면 CRF 26 사용
            26
        } else {
            // 목표 크기에 맞춰 CRF 조정
            self.calculate_optimal_crf(metadata, target_mb)?
        };
        
        Ok(CrfRecommendation {
            recommended_crf: crf,
            estimated_quality: self.crf_to_quality_score(crf),
            size_achievable: target_mb,
        })
    }
    
    fn calculate_optimal_crf(&self, metadata: &VideoMetadata, target_mb: f32) -> Result<u8, EstimationError> {
        // 이진 탐색으로 최적 CRF 찾기
        let mut low_crf = 18;  // 고품질
        let mut high_crf = 51; // 저품질
        let mut best_crf = 26;
        
        for _ in 0..10 {  // 최대 10회 반복
            let test_crf = (low_crf + high_crf) / 2;
            let estimated_size = self.estimate_size_for_crf(metadata, test_crf)?;
            
            if (estimated_size - target_mb).abs() < 0.5 {
                // 0.5MB 오차 내면 성공
                best_crf = test_crf;
                break;
            }
            
            if estimated_size > target_mb {
                low_crf = test_crf + 1;  // 더 압축 필요
            } else {
                high_crf = test_crf - 1; // 덜 압축
            }
            
            best_crf = test_crf;
        }
        
        // 실용적 범위로 제한
        Ok(best_crf.clamp(20, 40))
    }
}

#[derive(Debug)]
pub struct CrfRecommendation {
    pub recommended_crf: u8,
    pub estimated_quality: f32,  // 0.0-1.0
    pub size_achievable: f32,
}
```

## 🎨 UI Component Updates

### Size Slider Component (React/TypeScript)

```typescript
// components/SizeSlider.tsx
import React, { useState, useCallback } from 'react';

interface SizeSliderProps {
  autoSize: boolean;
  currentSize: number; // 0-4 index
  estimatedSizeMB?: number;
  onAutoChange: (auto: boolean) => void;
  onSizeChange: (sizeIndex: number) => void;
}

const SIZE_OPTIONS = [1, 5, 10, 30, 50];

export const SizeSlider: React.FC<SizeSliderProps> = ({
  autoSize,
  currentSize,
  estimatedSizeMB,
  onAutoChange,
  onSizeChange,
}) => {
  const handleSliderChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const index = parseInt(e.target.value);
    onSizeChange(index);
  }, [onSizeChange]);

  return (
    <div className="size-slider-container">
      <div className="auto-checkbox">
        <label>
          <input
            type="checkbox"
            checked={autoSize}
            onChange={(e) => onAutoChange(e.target.checked)}
          />
          <span className="checkbox-text">Auto</span>
        </label>
      </div>
      
      <div className={`slider-section ${autoSize ? 'disabled' : ''}`}>
        <div className="slider-container">
          <input
            type="range"
            min="0"
            max="4"
            step="1"
            value={currentSize}
            onChange={handleSliderChange}
            disabled={autoSize}
            className="size-slider"
          />
          
          <div className="size-labels">
            {SIZE_OPTIONS.map((size, index) => (
              <span
                key={size}
                className={`size-label ${currentSize === index ? 'active' : ''}`}
              >
                {size}
              </span>
            ))}
          </div>
        </div>
        
        <div className="current-size">
          <span className="size-value">{SIZE_OPTIONS[currentSize]} MB</span>
        </div>
      </div>
      
      {autoSize && estimatedSizeMB && (
        <div className="auto-estimation">
          <span className="estimation-text">
            Quality-based estimation: <strong>{estimatedSizeMB.toFixed(1)} MB</strong>
          </span>
          <span className="crf-info">(CRF 26 baseline)</span>
        </div>
      )}
    </div>
  );
};
```

### CSS Styling

```css
/* styles/SizeSlider.css */
.size-slider-container {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  background: #fafafa;
}

.auto-checkbox label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-weight: 500;
}

.slider-section {
  transition: opacity 0.2s ease;
}

.slider-section.disabled {
  opacity: 0.4;
  pointer-events: none;
}

.slider-container {
  position: relative;
  margin: 20px 0;
}

.size-slider {
  width: 100%;
  height: 6px;
  border-radius: 3px;
  background: #ddd;
  outline: none;
  -webkit-appearance: none;
}

.size-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: #4CAF50;
  cursor: pointer;
  box-shadow: 0 2px 4px rgba(0,0,0,0.2);
}

.size-slider::-moz-range-thumb {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: #4CAF50;
  cursor: pointer;
  border: none;
  box-shadow: 0 2px 4px rgba(0,0,0,0.2);
}

.size-labels {
  display: flex;
  justify-content: space-between;
  margin-top: 8px;
}

.size-label {
  font-size: 12px;
  color: #666;
  font-weight: 500;
  transition: color 0.2s ease;
}

.size-label.active {
  color: #4CAF50;
  font-weight: 700;
}

.current-size {
  text-align: center;
  margin-top: 8px;
}

.size-value {
  font-size: 18px;
  font-weight: 700;
  color: #333;
}

.auto-estimation {
  padding: 12px;
  background: #e8f5e8;
  border-radius: 6px;
  text-align: center;
}

.estimation-text {
  display: block;
  color: #2e7d2e;
  font-weight: 500;
}

.crf-info {
  display: block;
  font-size: 12px;
  color: #666;
  margin-top: 4px;
}
```

## 🌍 Multi-language Support

### Language Files

```json
// locales/ko.json
{
  "app.title": "작은mp4 - 동영상 압축기",
  "app.description": "동영상 공유를 위해서 영상을 꾸겨줍니다",
  "input.select": "입력 파일 선택",
  "output.folder": "출력 폴더",
  "output.same_folder": "입력 파일과 같은 폴더",
  "size.auto": "자동",
  "size.target": "목표 크기",
  "size.estimation": "품질 기반 추정: {size} MB",
  "size.crf_baseline": "(CRF 26 기준)",
  "size.1mb": "1 MB",
  "size.5mb": "5 MB", 
  "size.10mb": "10 MB",
  "size.30mb": "30 MB",
  "size.50mb": "50 MB",
  "action.compress": "압축하기",
  "action.stop": "중지",
  "action.cancel": "취소",
  "status.processing": "압축 중...",
  "status.completed": "압축 완료!",
  "preview.original": "원본",
  "preview.compressed": "미리보기"
}

// locales/en.json
{
  "app.title": "Small MP4 - Video Compressor",
  "app.description": "Squeeze your videos for easy sharing",
  "input.select": "Select Input File",
  "output.folder": "Output Folder", 
  "output.same_folder": "Same as input folder",
  "size.auto": "Auto",
  "size.target": "Target Size",
  "size.estimation": "Quality-based estimation: {size} MB",
  "size.crf_baseline": "(CRF 26 baseline)",
  "size.1mb": "1 MB",
  "size.5mb": "5 MB",
  "size.10mb": "10 MB", 
  "size.30mb": "30 MB",
  "size.50mb": "50 MB",
  "action.compress": "Compress",
  "action.stop": "Stop",
  "action.cancel": "Cancel",
  "status.processing": "Compressing...",
  "status.completed": "Compression completed!",
  "preview.original": "Original",
  "preview.compressed": "Preview"
}

// locales/ja.json
{
  "app.title": "小さなmp4 - 動画圧縮ツール",
  "app.description": "動画共有のために映像を圧縮します",
  "input.select": "入力ファイルを選択",
  "output.folder": "出力フォルダ",
  "output.same_folder": "入力ファイルと同じフォルダ",
  "size.auto": "自動",
  "size.target": "目標サイズ",
  "size.estimation": "品質ベース推定: {size} MB",
  "size.crf_baseline": "(CRF 26ベースライン)",
  "size.1mb": "1 MB",
  "size.5mb": "5 MB",
  "size.10mb": "10 MB",
  "size.30mb": "30 MB", 
  "size.50mb": "50 MB",
  "action.compress": "圧縮する",
  "action.stop": "停止",
  "action.cancel": "キャンセル",
  "status.processing": "圧縮中...",
  "status.completed": "圧縮完了！",
  "preview.original": "オリジナル",
  "preview.compressed": "プレビュー"
}
```

## 🔧 Updated State Management

### App State with Slider

```rust
#[derive(Debug, Clone)]
pub struct AppState {
    pub input_file: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub same_folder: bool,
    pub compression_settings: CompressionSettings,
    pub current_job: Option<CompressionJob>,
    pub ui_language: Language,
    pub recent_files: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct CompressionSettings {
    pub auto_size: bool,
    pub target_size: TargetSize,
    pub estimated_size_mb: Option<f32>,
    pub compression_mode: CompressionMode,
}

#[derive(Debug, Clone)]
pub enum CompressionMode {
    Auto { crf: u8 },
    TargetSize { size_mb: f32 },
}

impl CompressionSettings {
    pub fn get_effective_target_mb(&self) -> Option<f32> {
        if self.auto_size {
            self.estimated_size_mb
        } else {
            Some(self.target_size.as_mb())
        }
    }
}
```

## 🚀 CLI Interface Updates

### Simple CLI with Size Presets

```rust
use clap::{Args, Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "small-mp4")]
#[command(about = "Squeeze your videos for easy sharing")]
pub struct Cli {
    /// Input video file
    pub input: PathBuf,
    
    /// Output file (optional)
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    
    /// Target size preset
    #[arg(short, long, value_enum, default_value = "50mb")]
    pub size: SizePreset,
    
    /// Use auto sizing based on quality
    #[arg(long)]
    pub auto: bool,
    
    /// Language for output messages
    #[arg(short, long, value_enum, default_value = "en")]
    pub lang: Language,
}

#[derive(Clone, ValueEnum)]
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
}

impl SizePreset {
    pub fn to_target_size(&self) -> TargetSize {
        match self {
            SizePreset::Size1MB => TargetSize::Size1MB,
            SizePreset::Size5MB => TargetSize::Size5MB,
            SizePreset::Size10MB => TargetSize::Size10MB,
            SizePreset::Size30MB => TargetSize::Size30MB,
            SizePreset::Size50MB => TargetSize::Size50MB,
        }
    }
}

#[derive(Clone, ValueEnum)]
pub enum Language {
    #[value(name = "ko")]
    Korean,
    #[value(name = "en")]
    English,
    #[value(name = "ja")]
    Japanese,
}

// Usage examples:
// small-mp4 video.mov --size 10mb
// small-mp4 video.mov --auto --lang ko
// small-mp4 video.mov --size 5mb --output compressed.mp4
```

## 📦 Project Structure

```
small-mp4/
├── src/
│   ├── main.rs
│   ├── app/
│   │   ├── mod.rs
│   │   ├── state.rs
│   │   └── settings.rs
│   ├── compression/
│   │   ├── mod.rs
│   │   ├── engine.rs
│   │   ├── estimator.rs
│   │   └── size_presets.rs
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── components/
│   │   │   ├── size_slider.rs
│   │   │   ├── file_input.rs
│   │   │   └── progress_bar.rs
│   │   └── i18n.rs
│   └── cli/
│       ├── mod.rs
│       └── commands.rs
├── locales/
│   ├── ko.json
│   ├── en.json
│   └── ja.json
├── assets/
│   ├── icons/
│   └── styles/
└── Cargo.toml
```

또한 기존 비트레이트를 감지해서 더 놏은 비트레이트로 압축시도할경우 할필요가 없다는 메시지 출력

---

## 🚀 Hardware Acceleration Support

### 🎯 Core Hardware Features

- **⚡ CUDA/NVENC**: NVIDIA GPU acceleration for H.264, H.265/HEVC, and AV1 encoding
- **🔥 AMD VCE**: AMD GPU acceleration for H.264 and H.265 encoding  
- **💎 Intel QuickSync**: Intel integrated graphics acceleration
- **🖥️ VAAPI**: Linux video acceleration API support
- **🍎 VideoToolbox**: macOS hardware acceleration support
- **⚙️ Auto-Detection**: Automatic hardware capability detection and selection
- **🔄 Fallback System**: Graceful degradation to software encoding when hardware unavailable

### Hardware Acceleration Architecture

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum HardwareEncoder {
    // NVIDIA NVENC
    NvencH264,
    NvencH265,
    NvencAV1,
    
    // AMD VCE
    AmfH264,
    AmfH265,
    
    // Intel QuickSync
    QsvH264,
    QsvH265,
    QsvAV1,
    
    // Platform-specific
    Vaapi,          // Linux
    VideoToolbox,   // macOS
    
    // Software fallback
    Software,
}

#[derive(Debug, Clone)]
pub struct HardwareCapabilities {
    pub available_encoders: Vec<HardwareEncoder>,
    pub cuda_devices: Vec<CudaDevice>,
    pub opencl_devices: Vec<OpenCLDevice>,
    pub preferred_encoder: Option<HardwareEncoder>,
    pub memory_usage_mb: u64,
    pub encoding_speed_multiplier: f32,
}

#[derive(Debug, Clone)]
pub struct CudaDevice {
    pub id: u32,
    pub name: String,
    pub compute_capability: (u32, u32),
    pub memory_mb: u64,
    pub nvenc_support: bool,
    pub max_concurrent_sessions: u32,
}
```

### Enhanced Compression Settings

```rust
#[derive(Debug, Clone)]
pub struct CompressionSettings {
    // Existing fields
    pub auto_size: bool,
    pub target_size: TargetSize,
    pub estimated_size_mb: Option<f32>,
    pub crf_quality: u8,
    
    // New hardware acceleration fields
    pub hardware_encoder: HardwareEncoder,
    pub enable_hardware_accel: bool,
    pub cuda_device_id: Option<u32>,
    pub hardware_preset: HardwarePreset,
    pub hardware_quality: HardwareQuality,
    pub force_software_fallback: bool,
    pub memory_optimization: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HardwarePreset {
    UltraFast,  // p1 - Fastest encoding, lower quality
    Faster,     // p2 - Fast encoding  
    Fast,       // p3 - Balanced speed/quality
    Medium,     // p4 - Default balanced
    Slow,       // p5 - Better quality
    Slower,     // p6 - High quality
    Highest,    // p7 - Maximum quality
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HardwareQuality {
    Auto,       // Let hardware decide
    Constant,   // CRF-like constant quality
    Variable,   // Variable bitrate 
    Constrained, // Constrained variable bitrate
}
```

### Hardware Detection System

```rust
impl HardwareCapabilities {
    /// Detect all available hardware acceleration capabilities
    pub fn detect() -> Result<Self, HardwareError> {
        let mut capabilities = HardwareCapabilities::default();
        
        // Detect NVIDIA CUDA/NVENC using ez-ffmpeg
        if let Ok(cuda_info) = detect_cuda_capabilities() {
            capabilities.cuda_devices = cuda_info.devices;
            capabilities.available_encoders.extend(cuda_info.encoders);
        }
        
        // Detect AMD VCE
        if let Ok(amd_encoders) = detect_amd_vce() {
            capabilities.available_encoders.extend(amd_encoders);
        }
        
        // Detect Intel QuickSync
        if let Ok(intel_encoders) = detect_intel_quicksync() {
            capabilities.available_encoders.extend(intel_encoders);
        }
        
        // Platform-specific detection
        #[cfg(target_os = "linux")]
        if detect_vaapi_support() {
            capabilities.available_encoders.push(HardwareEncoder::Vaapi);
        }
        
        #[cfg(target_os = "macos")]
        if detect_videotoolbox_support() {
            capabilities.available_encoders.push(HardwareEncoder::VideoToolbox);
        }
        
        // Select preferred encoder based on performance benchmarks
        capabilities.preferred_encoder = select_optimal_encoder(&capabilities.available_encoders)?;
        
        Ok(capabilities)
    }
    
    /// Get encoding speed improvement estimate
    pub fn speed_improvement(&self, encoder: &HardwareEncoder) -> f32 {
        match encoder {
            HardwareEncoder::NvencH264 | HardwareEncoder::NvencH265 => 8.0,
            HardwareEncoder::AmfH264 | HardwareEncoder::AmfH265 => 5.5,
            HardwareEncoder::QsvH264 | HardwareEncoder::QsvH265 => 7.0,
            HardwareEncoder::Vaapi => 4.0,
            HardwareEncoder::VideoToolbox => 6.0,
            HardwareEncoder::Software => 1.0,
            _ => 2.0,
        }
    }
}

fn detect_cuda_capabilities() -> Result<CudaInfo, HardwareError> {
    use ez_ffmpeg::core::hwaccel::get_hwaccels;
    
    // Query FFmpeg for CUDA support
    let hwaccels = get_hwaccels();
    let cuda_available = hwaccels.iter().any(|accel| accel.name == "cuda");
    
    if !cuda_available {
        return Err(HardwareError::CudaNotAvailable);
    }
    
    // Query CUDA devices
    let cuda_devices = query_cuda_devices()?;
    
    // Determine available NVENC encoders
    let encoders = determine_nvenc_encoders(&cuda_devices)?;
    
    Ok(CudaInfo {
        devices: cuda_devices,
        encoders,
    })
}
```

## 🚀 Updated CLI Interface

### Enhanced CLI with Hardware Options

```rust
use clap::{Args, Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "small-mp4")]
#[command(about = "Squeeze your videos for easy sharing - Now with hardware acceleration!")]
pub struct Cli {
    /// Input video file
    pub input: PathBuf,
    
    /// Output file (optional)
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    
    /// Target size preset
    #[arg(short, long, value_enum, default_value = "50mb")]
    pub size: SizePreset,
    
    /// Use auto sizing based on quality
    #[arg(long)]
    pub auto: bool,
    
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
    
    /// List available hardware encoders and exit
    #[arg(long)]
    pub list_hw: bool,
    
    /// Enable memory optimization for hardware encoding
    #[arg(long)]
    pub memory_opt: bool,
    
    /// Language for output messages
    #[arg(short, long, value_enum, default_value = "en")]
    pub lang: Language,
}

// Usage examples:
// small-mp4 video.mov --size 10mb --hw-encoder nvenc-h264 --hw-preset fast
// small-mp4 video.mov --auto --hw-encoder auto --cuda-device 0
// small-mp4 --list-hw  # List available hardware encoders
// small-mp4 video.mov --force-software  # Disable hardware acceleration
```

## 📦 Updated Project Structure

```
small-mp4/
├── src/
│   ├── main.rs
│   ├── app/
│   │   ├── mod.rs
│   │   ├── state.rs
│   │   └── settings.rs
│   ├── compression/
│   │   ├── mod.rs
│   │   ├── engine.rs
│   │   ├── estimator.rs
│   │   ├── size_presets.rs
│   │   └── hardware/           # New hardware acceleration module
│   │       ├── mod.rs
│   │       ├── detection.rs    # Hardware capability detection
│   │       ├── cuda.rs         # CUDA/NVENC support
│   │       ├── amd.rs          # AMD VCE support  
│   │       ├── intel.rs        # Intel QuickSync support
│   │       ├── platform.rs     # Platform-specific acceleration
│   │       └── fallback.rs     # Software fallback handling
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── components/
│   │   │   ├── size_slider.rs
│   │   │   ├── file_input.rs
│   │   │   ├── progress_bar.rs
│   │   │   ├── hardware_status.rs     # New hardware status component
│   │   │   ├── encoder_selector.rs    # New encoder selection component
│   │   │   └── preset_controls.rs     # New hardware preset controls
│   │   └── i18n.rs
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── commands.rs
│   │   └── hardware_cli.rs     # New hardware CLI commands
│   └── utils/
│       ├── mod.rs
│       ├── benchmarking.rs     # New performance benchmarking
│       └── system_info.rs      # New system information utilities
├── Cargo.toml
└── README.md
```

## 🔧 Cargo.toml Configuration

```toml
[package]
name = "small-mp4"
version = "0.2.0"
edition = "2021"
description = "Squeeze your videos for easy sharing - Now with hardware acceleration!"

[dependencies]
# Core dependencies
clap = { version = "4.0", features = ["derive", "cargo"] }
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }

# FFmpeg and hardware acceleration
ez-ffmpeg = { version = "0.5", features = ["opengl", "async"] }

# Logging and monitoring  
log = "0.4"
env_logger = "0.11"
sysinfo = "0.30"

[features]
default = ["hardware-accel"]
hardware-accel = ["cuda", "amd", "intel", "vaapi", "videotoolbox"]
cuda = []
amd = []
intel = []
vaapi = []
videotoolbox = []

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
tempfile = "3.0"
```
