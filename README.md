# 🎬 Small MP4 - Modern Video Compressor

> **English**: Squeeze your videos for easy sharing  
> **한국어**: 동영상 공유를 위해서 영상을 꾸겨줍니다  
> **日本語**: 動画共有のために映像を圧縮します

A fast, efficient video compression tool with hardware acceleration and native GUI built with Rust and egui.

## ✨ Features

### 🖥️ Native GUI Application
- **⚡ Fast Native Performance**: Instant startup with minimal memory usage
- **📱 Intuitive Interface**: Drag & drop file upload with clean UI
- **📊 Real-time Progress**: Live compression progress with performance metrics
- **⚙️ Direct Controls**: Hardware settings and advanced options

### 💻 Core Capabilities
- **🚀 Hardware Acceleration**: CUDA/NVENC, AMD VCE, Intel QuickSync support
- **🎯 Size Presets**: Quick 1MB, 5MB, 10MB, 30MB, 50MB targets
- **⚡ Smart Detection**: Automatic hardware capability detection
- **🔄 Fallback System**: Graceful degradation to software encoding
- **🌍 Cross-Platform**: Linux, macOS, Windows support
- **📊 Intelligent Estimation**: Quality-aware size prediction
- **💻 CLI & Library**: Command-line tool and Rust library

## 🚀 Hardware Acceleration Support

### NVIDIA GPUs (NVENC)
- **H.264**: All NVENC-capable GPUs (GTX 600 series+)
- **H.265/HEVC**: Maxwell 2nd gen and newer (GTX 900 series+)
- **AV1**: Ada Lovelace and newer (RTX 40 series)
- **Speed**: Up to 15x faster than CPU encoding

### AMD GPUs (VCE)
- **H.264**: GCN 1.0 and newer (HD 7000 series+)
- **H.265/HEVC**: Polaris and newer (RX 400 series+)
- **Speed**: Up to 8x faster than CPU encoding

### Intel GPUs (QuickSync)
- **H.264**: Sandy Bridge and newer (2nd gen Core+)
- **H.265/HEVC**: Skylake and newer (6th gen Core+)
- **AV1**: Arc GPUs and some 12th gen+
- **Speed**: Up to 12x faster than CPU encoding

### Platform-Specific
- **Linux**: VAAPI support for AMD/Intel
- **macOS**: VideoToolbox for Apple Silicon/Intel
- **Windows**: Native vendor driver support

## 📦 Installation & Usage

Small MP4는 **두 가지 인터페이스**를 제공합니다:

### 🖥️ 사용 방법

| 인터페이스 | 실행 방법 | 특징 | 권장 용도 |
|---------|----------|------|----------|
| **⚡ Native GUI** | `cargo run` | 빠른 실행, 가벼움, Rust 네이티브 | **일반 사용자 추천** |
| **💻 CLI 도구** | `cargo run compress video.mp4` | 명령줄 인터페이스 | **자동화, 스크립팅** |

## 🚀 Quick Start

### Option 1: Native GUI (빠른 실행 ⚡)

Rust native egui를 사용한 경량 GUI로, 빠른 실행과 낮은 메모리 사용량이 장점입니다.

```bash
# Clone the repository
git clone https://github.com/your-username/small-mp4-rs.git
cd small-mp4-rs

# Run the native GUI directly
cargo run

# Build for production
cargo build --release
```

#### 주요 기능:
- ⚡ Fast native Rust GUI
- 💾 Low memory usage
- 🔧 Direct hardware controls
- 📊 Detailed hardware information

### Option 2: Command Line Interface (자동화 💻)

프로그래밍 자동화와 배치 처리를 위한 강력한 CLI 도구입니다.

```bash
# Clone the repository
git clone https://github.com/your-username/small-mp4-rs.git
cd small-mp4-rs

# Build CLI tool
cargo build --release

# Basic usage examples
cargo run compress input.mp4 --size 10mb
cargo run compress input.mov --auto --hw-encoder nvenc-h264
cargo run list-hw  # Show available hardware
```

#### 주요 기능:
- 🤖 Batch processing support
- 🔧 Full hardware control options
- 📊 Detailed hardware detection
- ⚙️ Script-friendly output formats

## 🛠️ Prerequisites & Dependencies

### System Requirements
- **Rust**: 1.70.0 or newer
- **FFmpeg**: System installation required

### Install FFmpeg
```bash
# Ubuntu/Debian
sudo apt install ffmpeg

# macOS
brew install ffmpeg

# Windows
# Download from: https://ffmpeg.org/download.html
```

## 🖥️ Usage

### Command Line Interface

#### Basic Compression
```bash
# Compress to 10MB using auto-detected hardware
small-mp4 compress input.mov --size 10mb

# Force software encoding
small-mp4 compress input.mov --size 5mb --force-software

# Auto-size based on quality
small-mp4 compress input.mov --auto --hw-encoder nvenc-h264
```

#### Hardware-Specific Options
```bash
# Use specific NVIDIA GPU
small-mp4 compress input.mov --hw-encoder nvenc-h265 --cuda-device 0

# AMD VCE encoding
small-mp4 compress input.mov --hw-encoder amf-h264 --hw-preset fast

# Intel QuickSync
small-mp4 compress input.mov --hw-encoder qsv-h265 --hw-quality constant
```

#### Hardware Detection
```bash
# List available hardware encoders
small-mp4 list-hw
```

Example output:
```
🔍 Detecting hardware acceleration capabilities...

✅ Hardware Detection Results:

🔥 NVIDIA CUDA Devices:
  [0] NVIDIA GeForce RTX 4090 - 24576MB VRAM (Compute 8.9)
      NVENC Support: ✅
      Max Sessions: 5

⚡ Available Hardware Encoders:
  • NvencH264 - 12.0x faster encoding
  • NvencH265 - 12.0x faster encoding  
  • NvencAV1 - 10.0x faster encoding
  • Software - 1.0x faster encoding

🎯 Recommended Encoder: NvencH265
   Memory Usage: 512MB
   Speed Multiplier: 12.0x
```

### Library Usage

```rust
use small_mp4::{CompressionEngine, CompressionSettings, HardwareCapabilities};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Detect hardware capabilities
    let hw_capabilities = HardwareCapabilities::detect().await?;
    
    // Create compression engine
    let mut engine = CompressionEngine::new(hw_capabilities);
    
    // Configure settings
    let settings = CompressionSettings {
        auto_size: false,
        target_size: TargetSize::Size10MB,
        hardware_encoder: HardwareEncoder::NvencH264,
        enable_hardware_accel: true,
        ..Default::default()
    };
    
    // Compress video
    let result = engine.compress(
        Path::new("input.mov"),
        Some(Path::new("output.mp4")),
        &settings
    ).await?;
    
    println!("Compression completed: {}", result.summary());
    Ok(())
}
```

## ⚡ Performance Benchmarks

Based on internal testing with various hardware configurations:

| Encoder | Input Resolution | Speed Improvement | Quality | Notes |
|---------|-----------------|-------------------|---------|--------|
| NVENC H.264 | 1080p | 8-15x | Good | Best compatibility |
| NVENC H.265 | 1080p | 8-12x | Excellent | Better compression |
| NVENC AV1 | 1080p | 6-10x | Excellent | Future-proof |
| AMD VCE H.264 | 1080p | 5-8x | Good | Solid alternative |
| Intel QSV H.264 | 1080p | 6-12x | Good | Great for laptops |
| Software | 1080p | 1x | Excellent | Highest quality |

*Performance varies based on content complexity, system specifications, and encoding settings.*

## 🔧 Configuration

### Hardware Presets
- **ultrafast**: Fastest encoding, lower quality
- **fast**: Good speed/quality balance  
- **medium**: Default balanced preset
- **slow**: Better quality, slower encoding
- **highest**: Maximum quality

### Quality Modes
- **auto**: Hardware decides optimal settings
- **constant**: CRF-like constant quality
- **variable**: Variable bitrate for size targets
- **constrained**: Bounded variable bitrate

## 📋 Requirements

### System Requirements
- **OS**: Linux, macOS 10.14+, Windows 10+
- **CPU**: Any modern CPU (64-bit)
- **RAM**: 4GB+ recommended (8GB+ for 4K)
- **Disk**: 1GB free space minimum

### Hardware Acceleration Requirements

#### NVIDIA (NVENC)
- **GPU**: GTX 600 series or newer
- **Driver**: 465.89+ recommended
- **CUDA**: Optional but recommended for best performance

#### AMD (VCE)  
- **GPU**: HD 7000 series or newer
- **Driver**: Adrenalin 21.4.1+ recommended

#### Intel (QuickSync)
- **CPU/GPU**: 2nd gen Core or newer with integrated graphics
- **Driver**: Latest Intel graphics drivers

## 🐛 Troubleshooting

### GUI Display Issues

#### Wayland Display Error
If you see `Gdk-Message: Error 71 (규약 오류) dispatching to Wayland display`:

```bash
# Try running with X11 fallback
GDK_BACKEND=x11 cargo run

# Alternative: Use XWayland
export DISPLAY=:0
cargo run
```

#### Permission Issues on Linux
```bash
# Add user to video group for hardware acceleration
sudo usermod -a -G video $USER
# Log out and back in for changes to take effect
```

### Common Issues

#### Hardware Not Detected
```bash
# Check FFmpeg hardware support
ffmpeg -hwaccels

# Verify drivers are installed
nvidia-smi          # NVIDIA
rocm-smi           # AMD  
intel_gpu_top      # Intel
```

#### Encoding Failures
The tool includes automatic fallback:
1. Try preferred hardware encoder
2. Fall back to alternative hardware
3. Fall back to software encoding

#### Performance Issues
- Close other applications
- Use faster presets (`--hw-preset ultrafast`)
- Enable memory optimization (`--memory-opt`)
- Try different encoder

## 📊 Size Estimation

The tool provides intelligent size estimation based on:
- Input video characteristics (resolution, bitrate, complexity)
- Encoder efficiency profiles
- Target quality settings
- Content analysis

Example estimation output:
```
📊 Compression Estimation:
  Target Size: 10.0 MB
  Estimated Quality: 85% (CRF 24)
  Encoding Time: ~45 seconds
  Compression Ratio: 12.5:1
  Encoder: NVIDIA NVENC H.264
```

## 🤝 Contributing

Contributions welcome! Areas of interest:
- Additional hardware encoder support
- UI/GUI development  
- Performance optimizations
- Platform-specific improvements
- Documentation and examples

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **ez-ffmpeg**: Rust FFmpeg bindings
- **FFmpeg**: Core video processing
- **NVIDIA**: NVENC hardware acceleration
- **AMD**: VCE hardware acceleration  
- **Intel**: QuickSync hardware acceleration

---

Built with ❤️ in Rust for fast, efficient video compression.