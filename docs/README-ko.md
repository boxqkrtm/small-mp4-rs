# 🎬 Small MP4 - 모던 비디오 압축기

> 동영상 공유를 위해서 영상을 꾸겨줍니다

Rust와 egui로 제작된 하드웨어 가속 지원 네이티브 GUI 비디오 압축 도구입니다.

## ✨ 주요 기능

### 🖥️ 네이티브 GUI 애플리케이션
- **⚡ 빠른 네이티브 성능**: 최소 메모리 사용으로 즉시 실행
- **📱 직관적인 인터페이스**: 깔끔한 UI와 드래그 앤 드롭 파일 업로드
- **📊 실시간 진행률**: 성능 지표와 함께 실시간 압축 진행률 표시
- **⚙️ 직접 제어**: 하드웨어 설정 및 고급 옵션

### 💻 핵심 기능
- **🚀 하드웨어 가속**: CUDA/NVENC, AMD VCE, Intel QuickSync 지원
- **🎯 크기 프리셋**: 빠른 1MB, 5MB, 10MB, 30MB, 50MB 타겟
- **⚡ 스마트 감지**: 자동 하드웨어 기능 감지
- **🔄 폴백 시스템**: 소프트웨어 인코딩으로 우아한 성능 저하
- **🌍 크로스 플랫폼**: Linux, macOS, Windows 지원
- **📊 지능형 추정**: 품질 인식 크기 예측
- **💻 CLI & 라이브러리**: 명령줄 도구 및 Rust 라이브러리

## 🚀 하드웨어 가속 지원

### NVIDIA GPU (NVENC)
- **H.264**: 모든 NVENC 지원 GPU (GTX 600 시리즈+)
- **H.265/HEVC**: Maxwell 2세대 이상 (GTX 900 시리즈+)
- **AV1**: Ada Lovelace 이상 (RTX 40 시리즈)
- **속도**: CPU 인코딩보다 최대 15배 빠름

### AMD GPU (VCE)
- **H.264**: GCN 1.0 이상 (HD 7000 시리즈+)
- **H.265/HEVC**: Polaris 이상 (RX 400 시리즈+)
- **속도**: CPU 인코딩보다 최대 8배 빠름

### Intel GPU (QuickSync)
- **H.264**: Sandy Bridge 이상 (2세대 Core+)
- **H.265/HEVC**: Skylake 이상 (6세대 Core+)
- **AV1**: Arc GPU 및 일부 12세대+
- **속도**: CPU 인코딩보다 최대 12배 빠름

### 플랫폼별
- **Linux**: AMD/Intel용 VAAPI 지원
- **macOS**: Apple Silicon/Intel용 VideoToolbox
- **Windows**: 네이티브 벤더 드라이버 지원

## 📦 설치 및 사용법

Small MP4는 **두 가지 인터페이스**를 제공합니다:

### 🖥️ 사용 방법

| 인터페이스 | 실행 방법 | 특징 | 권장 용도 |
|---------|----------|------|----------|
| **⚡ Native GUI** | `cargo run` | 빠른 실행, 가벼움, Rust 네이티브 | **일반 사용자 추천** |
| **💻 CLI 도구** | `cargo run compress video.mp4` | 명령줄 인터페이스 | **자동화, 스크립팅** |

## 🚀 빠른 시작

### 옵션 1: 네이티브 GUI (빠른 실행 ⚡)

Rust native egui를 사용한 경량 GUI로, 빠른 실행과 낮은 메모리 사용량이 장점입니다.

```bash
# 저장소 클론
git clone https://github.com/your-username/small-mp4-rs.git
cd small-mp4-rs

# 네이티브 GUI 직접 실행
cargo run

# 프로덕션 빌드
cargo build --release
```

#### 주요 기능:
- ⚡ 빠른 네이티브 Rust GUI
- 💾 낮은 메모리 사용량
- 🔧 직접 하드웨어 제어
- 📊 상세한 하드웨어 정보

### 옵션 2: 명령줄 인터페이스 (자동화 💻)

프로그래밍 자동화와 배치 처리를 위한 강력한 CLI 도구입니다.

```bash
# 저장소 클론
git clone https://github.com/your-username/small-mp4-rs.git
cd small-mp4-rs

# CLI 도구 빌드
cargo build --release

# 기본 사용 예제
cargo run compress input.mp4 --size 10mb
cargo run compress input.mov --hw-encoder nvenc-h264
cargo run list-hw  # 사용 가능한 하드웨어 표시
```

#### 주요 기능:
- 🤖 배치 처리 지원
- 🔧 전체 하드웨어 제어 옵션
- 📊 상세한 하드웨어 감지
- ⚙️ 스크립트 친화적 출력 형식

## 🛠️ 사전 요구사항 및 종속성

### 시스템 요구사항
- **Rust**: 1.70.0 이상
- **FFmpeg**: 시스템 설치 필요

### FFmpeg 설치
```bash
# Ubuntu/Debian
sudo apt install ffmpeg

# macOS
brew install ffmpeg

# Windows
# 다운로드: https://ffmpeg.org/download.html
```

## 🖥️ 사용법

### 명령줄 인터페이스

#### 기본 압축
```bash
# 자동 감지된 하드웨어로 10MB로 압축
small-mp4 compress input.mov --size 10mb

# 소프트웨어 인코딩 강제
small-mp4 compress input.mov --size 5mb --force-software
```

#### 하드웨어별 옵션
```bash
# 특정 NVIDIA GPU 사용
small-mp4 compress input.mov --hw-encoder nvenc-h265 --cuda-device 0

# AMD VCE 인코딩
small-mp4 compress input.mov --hw-encoder amf-h264 --hw-preset fast

# Intel QuickSync
small-mp4 compress input.mov --hw-encoder qsv-h265 --hw-quality constant
```

#### 하드웨어 감지
```bash
# 사용 가능한 하드웨어 인코더 나열
small-mp4 list-hw
```

## ⚡ 성능 벤치마크

다양한 하드웨어 구성으로 내부 테스트 기반:

| 인코더 | 입력 해상도 | 속도 향상 | 품질 | 참고 |
|---------|-----------------|-------------------|---------|--------|
| NVENC H.264 | 1080p | 8-15배 | 좋음 | 최고의 호환성 |
| NVENC H.265 | 1080p | 8-12배 | 우수함 | 더 나은 압축 |
| NVENC AV1 | 1080p | 6-10배 | 우수함 | 미래 지향적 |
| AMD VCE H.264 | 1080p | 5-8배 | 좋음 | 견고한 대안 |
| Intel QSV H.264 | 1080p | 6-12배 | 좋음 | 노트북에 최적 |
| 소프트웨어 | 1080p | 1배 | 우수함 | 최고 품질 |

*성능은 콘텐츠 복잡성, 시스템 사양 및 인코딩 설정에 따라 다릅니다.*

## 🔧 구성

### 하드웨어 프리셋
- **ultrafast**: 가장 빠른 인코딩, 낮은 품질
- **fast**: 좋은 속도/품질 균형  
- **medium**: 기본 균형 프리셋
- **slow**: 더 나은 품질, 느린 인코딩
- **highest**: 최대 품질

### 품질 모드
- **auto**: 하드웨어가 최적 설정 결정
- **constant**: CRF 같은 일정한 품질
- **variable**: 크기 목표를 위한 가변 비트레이트
- **constrained**: 제한된 가변 비트레이트

## 📋 요구사항

### 시스템 요구사항
- **OS**: Linux, macOS 10.14+, Windows 10+
- **CPU**: 모든 최신 CPU (64비트)
- **RAM**: 4GB+ 권장 (4K의 경우 8GB+)
- **디스크**: 최소 1GB 여유 공간

## 🐛 문제 해결

### GUI 디스플레이 문제

#### Wayland 디스플레이 오류
`Gdk-Message: Error 71 (규약 오류) dispatching to Wayland display`가 표시되는 경우:

```bash
# X11 폴백으로 실행 시도
GDK_BACKEND=x11 cargo run

# 대안: XWayland 사용
export DISPLAY=:0
cargo run
```

#### Linux 권한 문제
```bash
# 하드웨어 가속을 위해 비디오 그룹에 사용자 추가
sudo usermod -a -G video $USER
# 변경 사항을 적용하려면 로그아웃 후 다시 로그인
```

## 🤝 기여

기여를 환영합니다! 관심 분야:
- 추가 하드웨어 인코더 지원
- UI/GUI 개발  
- 성능 최적화
- 플랫폼별 개선사항
- 문서 및 예제

## 📄 라이선스

MIT 라이선스 - 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.

---

빠르고 효율적인 비디오 압축을 위해 Rust로 제작되었습니다 ❤️