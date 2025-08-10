# Test Videos

This directory contains sample videos for testing the compression functionality.

## Directory Structure

- `samples/` - Input test videos
- `output/` - Compressed output videos (gitignored)

## Generating Test Videos

Run the generation script to create test videos:

```bash
./generate_test_videos.sh
```

This will create:
- `test_720p_10s.mp4` - 720p, 10 seconds, low motion
- `test_1080p_30s.mp4` - 1080p, 30 seconds, medium motion  
- `test_4k_5s.mp4` - 4K, 5 seconds, high motion
- `test_static_60s.mp4` - 1080p, 60 seconds, static content

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_compression_sizes

# Run with output
cargo test -- --nocapture
```