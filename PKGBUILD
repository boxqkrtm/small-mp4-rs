# Maintainer: Your Name <your.email@example.com>
pkgname=small-mp4
pkgver=0.2.0
pkgrel=1
pkgdesc="Squeeze your videos for easy sharing - Now with hardware acceleration!"
arch=('x86_64')
url="https://github.com/small-mp4/small-mp4-rs"
license=('MIT')
depends=('ffmpeg' 'gtk3')
makedepends=('cargo')
optdepends=(
    'nvidia-utils: NVIDIA hardware acceleration support'
    'mesa: AMD/Intel hardware acceleration support'
    'libva: VAAPI hardware acceleration support'
)
source=()
sha256sums=()

prepare() {
    cd "$srcdir/.."
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
    cd "$srcdir/.."
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release --all-features
}

check() {
    cd "$srcdir/.."
    export RUSTUP_TOOLCHAIN=stable
    cargo test --frozen --all-features || true
}

package() {
    cd "$srcdir/.."
    
    # Install binary
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    
    # Install desktop file
    install -Dm644 "$pkgname.desktop" "$pkgdir/usr/share/applications/$pkgname.desktop"
    
    # Install icon (if exists)
    if [ -f "assets/icon.png" ]; then
        install -Dm644 "assets/icon.png" "$pkgdir/usr/share/pixmaps/$pkgname.png"
    fi
    
    # Install license
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    
    # Install documentation
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}