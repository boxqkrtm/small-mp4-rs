#!/bin/bash

# macOS DMG build script for Small MP4
# NOTE: This script must be run on macOS

set -e

PROJECT_NAME="Small MP4"
BUNDLE_ID="com.small-mp4.app"
VERSION=$(grep version Cargo.toml | head -n1 | cut -d'"' -f2)

echo "🍎 Building macOS DMG for Small MP4 v$VERSION..."

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "❌ This script must be run on macOS"
    exit 1
fi

# Build for both Intel and Apple Silicon
echo "📦 Building for macOS Intel (x86_64)..."
cargo build --release --target=x86_64-apple-darwin --features=gui

echo "📦 Building for macOS Apple Silicon (aarch64)..."
cargo build --release --target=aarch64-apple-darwin --features=gui

# Create universal binary
echo "🔧 Creating universal binary..."
mkdir -p target/release/universal
lipo -create \
    target/x86_64-apple-darwin/release/small-mp4 \
    target/aarch64-apple-darwin/release/small-mp4 \
    -output target/release/universal/small-mp4

# Create app bundle structure
APP_DIR="target/release/$PROJECT_NAME.app"
echo "📁 Creating app bundle..."
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

# Copy binary
cp target/release/universal/small-mp4 "$APP_DIR/Contents/MacOS/small-mp4"
chmod +x "$APP_DIR/Contents/MacOS/small-mp4"

# Create Info.plist
cat > "$APP_DIR/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>small-mp4</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleName</key>
    <string>$PROJECT_NAME</string>
    <key>CFBundleDisplayName</key>
    <string>$PROJECT_NAME</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSMinimumSystemVersion</key>
    <string>10.12</string>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>mp4</string>
                <string>m4v</string>
                <string>mov</string>
                <string>avi</string>
                <string>mkv</string>
                <string>webm</string>
            </array>
            <key>CFBundleTypeName</key>
            <string>Video File</string>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
        </dict>
    </array>
</dict>
</plist>
EOF

# Create simple icon (placeholder)
# In a real scenario, you'd use iconutil to create a proper .icns file
touch "$APP_DIR/Contents/Resources/icon.icns"

# Sign the app (if certificates are available)
if command -v codesign &> /dev/null && security find-identity -v -p codesigning | grep -q "Developer ID"; then
    echo "🔏 Signing app..."
    codesign --force --deep --sign - "$APP_DIR"
else
    echo "⚠️  No signing identity found, creating unsigned app"
fi

# Create DMG
DMG_NAME="Small-MP4-${VERSION}-universal.dmg"
DMG_PATH="target/release/$DMG_NAME"
TEMP_DMG="target/release/temp.dmg"

echo "💿 Creating DMG..."
rm -f "$DMG_PATH" "$TEMP_DMG"

# Create temporary DMG
hdiutil create -volname "$PROJECT_NAME" -srcfolder "$APP_DIR" -ov -format UDRW "$TEMP_DMG"

# Mount the DMG
DEVICE=$(hdiutil attach -readwrite -noverify "$TEMP_DMG" | egrep '^/dev/' | sed 1q | awk '{print $1}')
VOLUME="/Volumes/$PROJECT_NAME"

# Wait for volume to mount
sleep 2

# Create symbolic link to Applications
ln -s /Applications "$VOLUME/Applications"

# Set volume icon positions (optional)
echo '
   tell application "Finder"
     tell disk "'$PROJECT_NAME'"
           open
           set current view of container window to icon view
           set toolbar visible of container window to false
           set statusbar visible of container window to false
           set the bounds of container window to {400, 100, 900, 400}
           set viewOptions to the icon view options of container window
           set arrangement of viewOptions to not arranged
           set icon size of viewOptions to 72
           set position of item "'$PROJECT_NAME'.app" of container window to {100, 100}
           set position of item "Applications" of container window to {375, 100}
           close
           open
           update without registering applications
           delay 2
     end tell
   end tell
' | osascript

# Unmount and convert to compressed DMG
hdiutil detach "$DEVICE"
hdiutil convert "$TEMP_DMG" -format UDZO -imagekey zlib-level=9 -o "$DMG_PATH"
rm -f "$TEMP_DMG"

# Clean up
rm -rf "$APP_DIR"

echo "✅ DMG created: $DMG_PATH"
echo "   Size: $(du -h "$DMG_PATH" | cut -f1)"

# Notarization (requires Apple Developer account)
if command -v xcrun &> /dev/null && [ ! -z "$APPLE_ID" ]; then
    echo "📤 Submitting for notarization..."
    xcrun altool --notarize-app \
        --primary-bundle-id "$BUNDLE_ID" \
        --username "$APPLE_ID" \
        --password "$APPLE_APP_PASSWORD" \
        --file "$DMG_PATH"
else
    echo "ℹ️  To notarize this DMG, set APPLE_ID and APPLE_APP_PASSWORD environment variables"
fi

echo "🎉 macOS DMG build complete!"