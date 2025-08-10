#!/bin/bash

# Generate test videos using FFmpeg
echo "🎬 Generating test videos..."

# Create samples directory if it doesn't exist
mkdir -p samples

# 1. 720p 10 seconds - Low motion (color bars)
echo "Creating 720p 10s test video..."
ffmpeg -f lavfi -i testsrc=duration=10:size=1280x720:rate=30 \
    -f lavfi -i sine=frequency=1000:duration=10 \
    -c:v libx264 -preset fast -crf 23 \
    -c:a aac -b:a 128k \
    -y samples/test_720p_10s.mp4

# 2. 1080p 30 seconds - Medium motion (mandelbrot)
echo "Creating 1080p 30s test video..."
ffmpeg -f lavfi -i mandelbrot=size=1920x1080:rate=30 \
    -f lavfi -i sine=frequency=440:duration=30 \
    -t 30 \
    -c:v libx264 -preset fast -crf 23 \
    -c:a aac -b:a 128k \
    -y samples/test_1080p_30s.mp4

# 3. 4K 5 seconds - High motion (noise pattern)
echo "Creating 4K 5s test video..."
ffmpeg -f lavfi -i "nullsrc=s=3840x2160,geq=random(1)*255:128:128" \
    -f lavfi -i sine=frequency=880:duration=5 \
    -t 5 -r 30 \
    -c:v libx264 -preset fast -crf 23 \
    -c:a aac -b:a 128k \
    -y samples/test_4k_5s.mp4

# 4. 1080p 60 seconds - Static content (color test pattern)
echo "Creating 1080p 60s static test video..."
ffmpeg -f lavfi -i "color=c=blue:s=1920x1080:r=30,drawtext=text='Static Test Video':fontsize=64:fontcolor=white:x=(w-text_w)/2:y=(h-text_h)/2" \
    -f lavfi -i sine=frequency=220:duration=60 \
    -t 60 \
    -c:v libx264 -preset fast -crf 23 \
    -c:a aac -b:a 128k \
    -y samples/test_static_60s.mp4

# Show generated files
echo ""
echo "✅ Test videos generated:"
ls -lh samples/

echo ""
echo "File sizes:"
du -h samples/*.mp4