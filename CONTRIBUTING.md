# Contributing to Small MP4

Thank you for your interest in contributing to Small MP4! 🎉

## Development Setup

### Prerequisites

- **Node.js** 18+ 
- **Rust** 1.70+
- **FFmpeg** (system installation)

### Getting Started

```bash
# Clone the repository
git clone https://github.com/your-username/small-mp4-rs.git
cd small-mp4-rs

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev
```

## Project Structure

```
small-mp4-rs/
├── src/                    # React frontend
│   ├── App.tsx            # Main app component
│   ├── styles/            # CSS and styling
│   └── components/        # React components (future)
├── src-tauri/             # Tauri backend
│   ├── src/
│   │   ├── main.rs        # Main Tauri app
│   │   ├── compression/   # Compression engine
│   │   └── utils/         # Utility functions
│   └── Cargo.toml         # Rust dependencies
└── README.md
```

## Development Guidelines

### Frontend (React + TypeScript)
- Use TypeScript for all components
- Follow React best practices
- Use Tailwind CSS for styling
- Implement responsive design
- Add proper error handling

### Backend (Rust + Tauri)
- Follow Rust conventions
- Add proper error handling with `anyhow`
- Use async/await for I/O operations
- Add logging with `log` crate
- Write tests for critical functions

### UI/UX Design
- Maintain the modern black/glassmorphism theme
- Ensure smooth animations
- Keep drag & drop functionality intuitive
- Provide clear feedback for all user actions

## Code Style

### Rust
```rust
// Use descriptive function names
async fn detect_hardware_capabilities() -> Result<HardwareInfo> {
    // Implementation
}

// Proper error handling
match some_operation().await {
    Ok(result) => handle_success(result),
    Err(e) => {
        log::error!("Operation failed: {}", e);
        Err(e.into())
    }
}
```

### TypeScript/React
```typescript
// Use proper typing
interface VideoFile {
  name: string;
  path: string;
  size: number;
}

// Use async/await for Tauri calls
const compressVideo = async (file: VideoFile) => {
  try {
    const result = await invoke('compress_video', { 
      inputPath: file.path 
    });
    return result;
  } catch (error) {
    console.error('Compression failed:', error);
    throw error;
  }
};
```

## Commit Messages

Follow conventional commits:

```
feat: add new compression preset
fix: resolve hardware detection on AMD GPUs
docs: update installation instructions
style: improve glassmorphism effects
perf: optimize video encoding pipeline
test: add hardware detection tests
```

## Testing

### Running Tests
```bash
# Rust tests
cd src-tauri && cargo test

# Frontend tests (when implemented)
npm test
```

### Manual Testing
- Test on different operating systems
- Verify hardware acceleration works
- Test with various video formats
- Ensure UI responds correctly to errors

## Contributing Process

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Make** your changes
4. **Test** thoroughly
5. **Commit** with conventional commit messages
6. **Push** to your branch (`git push origin feature/amazing-feature`)
7. **Create** a Pull Request

## Areas for Contribution

### High Priority
- 🎨 UI/UX improvements
- 🚀 Performance optimizations
- 🔧 Hardware encoder support
- 📱 Better responsive design
- 🐛 Bug fixes

### Medium Priority
- 📊 Better progress indicators
- 🎯 Additional size presets
- 🌍 Internationalization (i18n)
- 📋 Batch processing
- 🎥 Video preview

### Future Features
- 🔄 Drag & drop improvements
- ⚙️ Advanced settings panel
- 📈 Compression analytics
- 🎬 Video filters/effects
- 🌐 Web version

## Bug Reports

When reporting bugs, please include:

- **Operating System** and version
- **GPU** information (if relevant)
- **Steps to reproduce**
- **Expected behavior**
- **Actual behavior**
- **Error messages** or logs
- **Screenshots** (if applicable)

## Feature Requests

For feature requests, please:

- Check if it already exists in issues
- Provide clear description of the feature
- Explain why it would be useful
- Consider implementation complexity
- Provide mockups if UI-related

## Code of Conduct

- Be respectful and inclusive
- Help others learn and improve
- Focus on constructive feedback
- Keep discussions on-topic

## Getting Help

- Check the [README](README.md) first
- Search existing [Issues](https://github.com/your-username/small-mp4-rs/issues)
- Create a new issue with proper labels
- Join our discussions

Thank you for contributing! 🚀