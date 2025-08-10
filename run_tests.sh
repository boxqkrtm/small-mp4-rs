#!/bin/bash

echo "🧪 Small MP4 Test Suite"
echo "======================"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if test videos exist
if [ ! -d "test_videos/samples" ] || [ -z "$(ls -A test_videos/samples/*.mp4 2>/dev/null)" ]; then
    echo -e "${YELLOW}⚠️  Test videos not found. Generating...${NC}"
    cd test_videos && ./generate_test_videos.sh && cd ..
fi

# Create output directory
mkdir -p test_videos/output

echo ""
echo "📹 Test Videos:"
ls -lh test_videos/samples/*.mp4 2>/dev/null || echo "No test videos found!"

echo ""
echo "🔨 Building project..."
cargo build --release

echo ""
echo "🧪 Running unit tests..."
cargo test --lib -- --nocapture

echo ""
echo "🧪 Running integration tests..."
cargo test --test compression_tests -- --nocapture

echo ""
echo "🧪 Running CLI tests..."
cargo test --test cli_tests -- --nocapture

echo ""
echo "📊 Test Results:"
echo "==============="

# Check compressed outputs
if [ -d "test_videos/output" ] && [ -n "$(ls -A test_videos/output/*.mp4 2>/dev/null)" ]; then
    echo -e "${GREEN}✅ Compressed outputs:${NC}"
    ls -lh test_videos/output/*.mp4
else
    echo -e "${YELLOW}⚠️  No compressed outputs found${NC}"
fi

echo ""
echo "🧹 Cleanup (optional):"
echo "rm -rf test_videos/output/*.mp4"

echo ""
echo -e "${GREEN}✨ Test suite complete!${NC}"