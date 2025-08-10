# 🎬 Small MP4 - モダンビデオ圧縮ツール

<div align="center">

[English](README-en.md) | [한국어](README-ko.md) | [日本語](README-ja.md)

> 動画共有のために映像を圧縮します

</div>

RustとeguiでハードウェアアクセラレーションをサポートしたネイティブGUIビデオ圧縮ツールです。

## ✨ 主な機能

### 🖥️ ネイティブGUIアプリケーション
- **⚡ 高速ネイティブパフォーマンス**: 最小メモリ使用で即座に起動
- **📱 直感的なインターフェース**: クリーンなUIとドラッグ＆ドロップファイルアップロード
- **📊 リアルタイム進捗**: パフォーマンスメトリクスと共にライブ圧縮進捗を表示
- **⚙️ 直接制御**: ハードウェア設定と高度なオプション

### 💻 コア機能
- **🚀 ハードウェアアクセラレーション**: CUDA/NVENC、AMD VCE、Intel QuickSyncサポート
- **🎯 サイズプリセット**: クイック1MB、5MB、10MB、30MB、50MBターゲット
- **⚡ スマート検出**: 自動ハードウェア機能検出
- **🔄 フォールバックシステム**: ソフトウェアエンコーディングへの優雅な劣化
- **🌍 クロスプラットフォーム**: Linux、macOS、Windowsサポート
- **📊 インテリジェント推定**: 品質認識サイズ予測
- **💻 CLI＆ライブラリ**: コマンドラインツールとRustライブラリ

## 🚀 ハードウェアアクセラレーションサポート

### NVIDIA GPU (NVENC)
- **H.264**: すべてのNVENC対応GPU (GTX 600シリーズ+)
- **H.265/HEVC**: Maxwell第2世代以降 (GTX 900シリーズ+)
- **AV1**: Ada Lovelace以降 (RTX 40シリーズ)
- **速度**: CPUエンコーディングより最大15倍高速

### AMD GPU (VCE)
- **H.264**: GCN 1.0以降 (HD 7000シリーズ+)
- **H.265/HEVC**: Polaris以降 (RX 400シリーズ+)
- **速度**: CPUエンコーディングより最大8倍高速

### Intel GPU (QuickSync)
- **H.264**: Sandy Bridge以降 (第2世代Core+)
- **H.265/HEVC**: Skylake以降 (第6世代Core+)
- **AV1**: Arc GPUおよび一部の第12世代+
- **速度**: CPUエンコーディングより最大12倍高速

### プラットフォーム固有
- **Linux**: AMD/Intel用VAAPIサポート
- **macOS**: Apple Silicon/Intel用VideoToolbox
- **Windows**: ネイティブベンダードライバーサポート

## 📦 インストールと使用方法

Small MP4は**2つのインターフェース**を提供します：

### 🖥️ 使用方法

| インターフェース | 実行方法 | 特徴 | 推奨用途 |
|---------|----------|------|----------|
| **⚡ ネイティブGUI** | `cargo run` | 高速起動、軽量、Rustネイティブ | **一般ユーザー推奨** |
| **💻 CLIツール** | `cargo run compress video.mp4` | コマンドラインインターフェース | **自動化、スクリプト** |

## 🚀 クイックスタート

### オプション1: ネイティブGUI (高速起動 ⚡)

Rustネイティブeguiを使用した軽量GUIで、高速起動と低メモリ使用が特徴です。

```bash
# リポジトリをクローン
git clone https://github.com/your-username/small-mp4-rs.git
cd small-mp4-rs

# ネイティブGUIを直接実行
cargo run

# プロダクションビルド
cargo build --release
```

#### 主な機能:
- ⚡ 高速ネイティブRust GUI
- 💾 低メモリ使用量
- 🔧 直接ハードウェア制御
- 📊 詳細なハードウェア情報

### オプション2: コマンドラインインターフェース (自動化 💻)

プログラム自動化とバッチ処理のための強力なCLIツールです。

```bash
# リポジトリをクローン
git clone https://github.com/your-username/small-mp4-rs.git
cd small-mp4-rs

# CLIツールをビルド
cargo build --release

# 基本的な使用例
cargo run compress input.mp4 --size 10mb
cargo run compress input.mov --hw-encoder nvenc-h264
cargo run list-hw  # 利用可能なハードウェアを表示
```

#### 主な機能:
- 🤖 バッチ処理サポート
- 🔧 完全なハードウェア制御オプション
- 📊 詳細なハードウェア検出
- ⚙️ スクリプトフレンドリーな出力形式

## 🛠️ 前提条件と依存関係

### システム要件
- **Rust**: 1.70.0以降
- **FFmpeg**: システムインストールが必要

### FFmpegのインストール
```bash
# Ubuntu/Debian
sudo apt install ffmpeg

# macOS
brew install ffmpeg

# Windows
# ダウンロード: https://ffmpeg.org/download.html
```

## 🖥️ 使用方法

### コマンドラインインターフェース

#### 基本的な圧縮
```bash
# 自動検出されたハードウェアで10MBに圧縮
small-mp4 compress input.mov --size 10mb

# ソフトウェアエンコーディングを強制
small-mp4 compress input.mov --size 5mb --force-software
```

#### ハードウェア固有のオプション
```bash
# 特定のNVIDIA GPUを使用
small-mp4 compress input.mov --hw-encoder nvenc-h265 --cuda-device 0

# AMD VCEエンコーディング
small-mp4 compress input.mov --hw-encoder amf-h264 --hw-preset fast

# Intel QuickSync
small-mp4 compress input.mov --hw-encoder qsv-h265 --hw-quality constant
```

#### ハードウェア検出
```bash
# 利用可能なハードウェアエンコーダーをリスト
small-mp4 list-hw
```

## ⚡ パフォーマンスベンチマーク

さまざまなハードウェア構成での内部テストに基づく：

| エンコーダー | 入力解像度 | 速度向上 | 品質 | 備考 |
|---------|-----------------|-------------------|---------|--------|
| NVENC H.264 | 1080p | 8-15倍 | 良好 | 最高の互換性 |
| NVENC H.265 | 1080p | 8-12倍 | 優秀 | より良い圧縮 |
| NVENC AV1 | 1080p | 6-10倍 | 優秀 | 将来対応 |
| AMD VCE H.264 | 1080p | 5-8倍 | 良好 | 堅実な代替 |
| Intel QSV H.264 | 1080p | 6-12倍 | 良好 | ノートPCに最適 |
| ソフトウェア | 1080p | 1倍 | 優秀 | 最高品質 |

*パフォーマンスはコンテンツの複雑さ、システム仕様、エンコーディング設定によって異なります。*

## 🔧 設定

### ハードウェアプリセット
- **ultrafast**: 最速エンコーディング、低品質
- **fast**: 良好な速度/品質バランス  
- **medium**: デフォルトバランスプリセット
- **slow**: より良い品質、遅いエンコーディング
- **highest**: 最高品質

### 品質モード
- **auto**: ハードウェアが最適な設定を決定
- **constant**: CRFのような一定品質
- **variable**: サイズターゲット用の可変ビットレート
- **constrained**: 制限付き可変ビットレート

## 📋 要件

### システム要件
- **OS**: Linux、macOS 10.14+、Windows 10+
- **CPU**: 任意の最新CPU (64ビット)
- **RAM**: 4GB+推奨 (4Kの場合8GB+)
- **ディスク**: 最小1GBの空き容量

## 🐛 トラブルシューティング

### GUIディスプレイの問題

#### Waylandディスプレイエラー
`Gdk-Message: Error 71 (規約 오류) dispatching to Wayland display`が表示される場合：

```bash
# X11フォールバックで実行してみる
GDK_BACKEND=x11 cargo run

# 代替: XWaylandを使用
export DISPLAY=:0
cargo run
```

#### Linuxの権限問題
```bash
# ハードウェアアクセラレーションのためにvideoグループにユーザーを追加
sudo usermod -a -G video $USER
# 変更を有効にするにはログアウトして再度ログイン
```

## 🤝 貢献

貢献を歓迎します！関心のある分野：
- 追加のハードウェアエンコーダーサポート
- UI/GUI開発  
- パフォーマンス最適化
- プラットフォーム固有の改善
- ドキュメントと例

## 📄 ライセンス

MITライセンス - 詳細は[LICENSE](LICENSE)ファイルを参照してください。

---

高速で効率的なビデオ圧縮のためにRustで構築されました ❤️