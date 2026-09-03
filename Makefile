# Makefile to build XXKey (SwiftUI App with Rust Core Engine) for macOS

SHELL := /bin/bash

# Target binary paths
APP_NAME := XXKey
BUILD_DIR := build
APP_BUNDLE := $(BUILD_DIR)/$(APP_NAME).app
CARGO_TARGET_DIR := target

# Environment variables for Puccinialin Cargo/Rustup installation
export PATH := /Users/itaccountvn.ab-inbev.com/Library/Caches/puccinialin/cargo/bin:$(PATH)
export CARGO_HOME := /Users/itaccountvn.ab-inbev.com/Library/Caches/puccinialin/cargo
export RUSTUP_HOME := /Users/itaccountvn.ab-inbev.com/Library/Caches/puccinialin/rustup

.PHONY: all clean run build-rust build-app

all: build-app

# 1. Build Rust Core Engine as a static library
build-rust:
	export PATH="/Users/itaccountvn.ab-inbev.com/Library/Caches/puccinialin/cargo/bin:$$PATH"; \
	export CARGO_HOME="/Users/itaccountvn.ab-inbev.com/Library/Caches/puccinialin/cargo"; \
	export RUSTUP_HOME="/Users/itaccountvn.ab-inbev.com/Library/Caches/puccinialin/rustup"; \
	cargo build -p vietime-engine --release

# 2. Compile SwiftUI frontend and link it with Rust static library
build-app: build-rust
	@echo "Creating App Bundle structure..."
	mkdir -p $(APP_BUNDLE)/Contents/MacOS
	@echo "Compiling Swift files..."
	swiftc -import-objc-header platform-macos/swift/BridgeHeader.h \
		platform-macos/swift/VietimeEngineBridge.swift \
		platform-macos/swift/App.swift \
		-o $(APP_BUNDLE)/Contents/MacOS/$(APP_NAME) \
		-L $(CARGO_TARGET_DIR)/release -lvietime_engine
	@echo "Creating Info.plist..."
	@echo '<?xml version="1.0" encoding="UTF-8"?>' > $(APP_BUNDLE)/Contents/Info.plist
	@echo '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo '<plist version="1.0">' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo '<dict>' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo '    <key>CFBundleExecutable</key>' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo '    <string>$(APP_NAME)</string>' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo '    <key>CFBundleIdentifier</key>' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo '    <string>com.xxkey.app</string>' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo '    <key>CFBundleName</key>' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo '    <string>$(APP_NAME)</string>' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo '    <key>CFBundlePackageType</key>' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo '    <string>APPL</string>' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo '    <key>CFBundleShortVersionString</key>' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo '    <string>1.3.0</string>' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo '    <key>LSMinimumSystemVersion</key>' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo '    <string>10.15</string>' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo '    <key>LSUIElement</key>' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo '    <true/>' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo '</dict>' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo '</plist>' >> $(APP_BUNDLE)/Contents/Info.plist
	@echo "Signing App Bundle..."
	codesign -s - --force --deep $(APP_BUNDLE)
	@echo "Build successful! Created $(APP_BUNDLE)"

# 3. Clean all build artifacts
clean:
	export PATH="/Users/itaccountvn.ab-inbev.com/Library/Caches/puccinialin/cargo/bin:$$PATH"; \
	export CARGO_HOME="/Users/itaccountvn.ab-inbev.com/Library/Caches/puccinialin/cargo"; \
	export RUSTUP_HOME="/Users/itaccountvn.ab-inbev.com/Library/Caches/puccinialin/rustup"; \
	cargo clean
	rm -rf $(BUILD_DIR)

# 4. Compile and launch the app immediately
run: build-app
	open $(APP_BUNDLE)

# 5. Build and package zip distributions for macOS, Windows, and Linux
zips:
	./build_zips.sh
