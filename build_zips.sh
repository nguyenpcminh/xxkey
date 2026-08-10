#!/bin/bash
set -e

# Export rust toolchain environment variables
export PATH="/Users/itaccountvn.ab-inbev.com/Library/Caches/puccinialin/cargo/bin:$PATH"
export CARGO_HOME="/Users/itaccountvn.ab-inbev.com/Library/Caches/puccinialin/cargo"
export RUSTUP_HOME="/Users/itaccountvn.ab-inbev.com/Library/Caches/puccinialin/rustup"

BUILD_DIR="build"
STAGE_WIN="$BUILD_DIR/windows-stage"
STAGE_LIN="$BUILD_DIR/linux-stage"

echo "=== 1. Building Native macOS App ==="
make build-app

echo "=== Packaging macOS App into ZIP ==="
cd "$BUILD_DIR"
zip -r XXKey-macOS.zip XXKey.app
cd ..

echo "=== 2. Cross-compiling Rust core engine for Windows ==="
cargo build -p vietime-engine --release --target x86_64-pc-windows-gnu

echo "=== Preparing Windows staging directory ==="
rm -rf "$STAGE_WIN"
mkdir -p "$STAGE_WIN/src"
mkdir -p "$STAGE_WIN/core"

# Copy pre-compiled core
cp target/x86_64-pc-windows-gnu/release/libvietime_engine.a "$STAGE_WIN/core/libvietime_engine.a"

# Copy source projects to allow local compilation on Windows
cp -r vietime-engine "$STAGE_WIN/src/"
cp -r platform-win "$STAGE_WIN/src/"
cp -r ui-settings "$STAGE_WIN/src/"
cp Cargo.toml "$STAGE_WIN/src/"
cp Cargo.lock "$STAGE_WIN/src/"

# Write build.bat
cat << 'EOF' > "$STAGE_WIN/build.bat"
@echo off
echo ==============================================
echo Building XXKey Daemon & Settings UI for Windows
echo ==============================================
cd src
cargo build --release -p platform-win -p ui-settings
echo.
echo Build complete! Executables can be found in:
echo   src\target\release\xxkey-daemon.exe
echo   src\target\release\xxkey-settings.exe
echo ==============================================
pause
EOF
chmod +x "$STAGE_WIN/build.bat"

# Write Windows README.md
cat << 'EOF' > "$STAGE_WIN/README.md"
# XXKey Windows Port

Thư mục này chứa mã nguồn và lõi xử lý được biên dịch sẵn của bộ gõ XXKey dành cho Windows.

## Yêu cầu hệ thống:
- Đã cài đặt Rust (cargo/rustc) trên Windows.

## Hướng dẫn cài đặt & biên dịch:
1. Nhấp đúp chuột vào file `build.bat` để tự động biên dịch.
2. Sau khi biên dịch hoàn tất, hai file thực thi sẽ được tạo tại:
   - `src\target\release\xxkey-daemon.exe` (Chương trình gõ phím chạy ẩn dưới khay hệ thống)
   - `src\target\release\xxkey-settings.exe` (Giao diện cấu hình bộ gõ Slint)
EOF

echo "=== Packaging Windows Build into ZIP ==="
cd "$BUILD_DIR"
zip -r XXKey-Windows.zip windows-stage -x "*.DS_Store"
cd ..

echo "=== 3. Cross-compiling Rust core engine for Linux ==="
cargo build -p vietime-engine --release --target x86_64-unknown-linux-gnu

echo "=== Preparing Linux staging directory ==="
rm -rf "$STAGE_LIN"
mkdir -p "$STAGE_LIN/src"
mkdir -p "$STAGE_LIN/core"

# Copy pre-compiled core
cp target/x86_64-unknown-linux-gnu/release/libvietime_engine.a "$STAGE_LIN/core/libvietime_engine.a"

# Copy source projects to allow local compilation on Linux
cp -r vietime-engine "$STAGE_LIN/src/"
cp -r platform-linux "$STAGE_LIN/src/"
cp -r ui-settings "$STAGE_LIN/src/"
cp Cargo.toml "$STAGE_LIN/src/"
cp Cargo.lock "$STAGE_LIN/src/"

# Write build.sh
cat << 'EOF' > "$STAGE_LIN/build.sh"
#!/bin/bash
set -e
echo "=============================================="
echo "Building XXKey Daemon & Settings UI for Linux"
echo "=============================================="
cd src
cargo build --release -p platform-linux -p ui-settings
echo
echo "Build complete! Executables can be found in:"
echo "  src/target/release/xxkey-daemon"
echo "  src/target/release/xxkey-settings"
echo "=============================================="
EOF
chmod +x "$STAGE_LIN/build.sh"

# Write Linux README.md
cat << 'EOF' > "$STAGE_LIN/README.md"
# XXKey Linux Port

Thư mục này chứa mã nguồn và lõi xử lý được biên dịch sẵn của bộ gõ XXKey dành cho Linux.

## Yêu cầu hệ thống:
- Đã cài đặt Rust (cargo/rustc) trên Linux.

## Hướng dẫn cài đặt & biên dịch:
1. Chạy lệnh: `chmod +x build.sh && ./build.sh`
2. Sau khi biên dịch hoàn tất, hai file thực thi sẽ được tạo tại:
   - `src/target/release/xxkey-daemon` (Chương trình gõ phím IBus/Fcitx5 chạy ẩn)
   - `src/target/release/xxkey-settings` (Giao diện cấu hình bộ gõ Slint)
EOF

echo "=== Packaging Linux Build into ZIP ==="
cd "$BUILD_DIR"
zip -r XXKey-Linux.zip linux-stage -x "*.DS_Store"
cd ..

# Clean staging directories
rm -rf "$STAGE_WIN"
rm -rf "$STAGE_LIN"

echo "=== Success! Generated ZIP files in $BUILD_DIR: ==="
ls -l "$BUILD_DIR"/*.zip
