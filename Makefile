.PHONY: help build test clippy fmt fmt-check doc clean \
        build-android build-android-all build-aar generate-bindings \
        publish-local publish-maven-central \
        ci ci-rust ci-android pre-commit

# ============================================
# PEAT-LITE Build System
# ============================================

# Configuration
ANDROID_SDK ?= $(HOME)/Android/Sdk
NDK_VERSION ?= 27.0.12077973
NDK_PATH ?= $(ANDROID_SDK)/ndk/$(NDK_VERSION)
NDK_TOOLCHAIN ?= $(NDK_PATH)/toolchains/llvm/prebuilt/linux-x86_64/bin

# Android architectures
ANDROID_TARGETS = aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
ANDROID_ABIS = arm64-v8a armeabi-v7a x86_64

# ============================================
# Help
# ============================================

help:
	@echo "PEAT-LITE Build System"
	@echo ""
	@echo "Rust Targets:"
	@echo "  build          - Build library (std feature)"
	@echo "  build-nostd    - Build library (no_std)"
	@echo "  test           - Run all tests"
	@echo "  clippy         - Run clippy lints"
	@echo "  fmt            - Format code"
	@echo "  fmt-check      - Check code formatting"
	@echo "  doc            - Generate documentation"
	@echo ""
	@echo "Android Targets:"
	@echo "  build-android      - Build native libs for all Android ABIs"
	@echo "  build-aar          - Build AAR package (native + bindings)"
	@echo "  generate-bindings  - Generate Kotlin bindings from UniFFI"
	@echo ""
	@echo "Publishing Targets:"
	@echo "  publish-local      - Publish AAR to local Maven (~/.m2)"
	@echo "  publish-maven-central - Publish AAR to Maven Central"
	@echo ""
	@echo "CI Targets:"
	@echo "  ci             - Run full CI pipeline"
	@echo "  ci-rust        - Run Rust CI checks"
	@echo "  ci-android     - Run Android CI checks"
	@echo "  pre-commit     - Run pre-commit checks (fmt, clippy, test)"
	@echo ""
	@echo "Other:"
	@echo "  clean          - Clean all build artifacts"
	@echo ""

# ============================================
# Rust Targets
# ============================================

build:
	cargo build --features std

build-nostd:
	cargo build --no-default-features

test:
	cargo test --features std
	cd android-ffi && cargo test

clippy:
	cargo clippy --features std -- -D warnings
	cd android-ffi && cargo clippy -- -D warnings

fmt:
	cargo fmt
	cd android-ffi && cargo fmt

fmt-check:
	cargo fmt --check
	cd android-ffi && cargo fmt --check

doc:
	cargo doc --features std --no-deps --open

# ============================================
# Android Targets
# ============================================

# Build native library for a single Android target
# Usage: make build-android-target TARGET=aarch64-linux-android ABI=arm64-v8a
build-android-target:
	@echo "Building for $(TARGET) -> $(ABI)..."
	cd android-ffi && cargo build --release --target $(TARGET)
	mkdir -p android/src/main/jniLibs/$(ABI)
	cp android-ffi/target/$(TARGET)/release/libpeat_lite_android.so android/src/main/jniLibs/$(ABI)/

# Build native libraries for all Android ABIs
build-android:
	@echo "Building peat-lite-android native libraries..."
	$(MAKE) build-android-target TARGET=aarch64-linux-android ABI=arm64-v8a
	$(MAKE) build-android-target TARGET=armv7-linux-androideabi ABI=armeabi-v7a
	$(MAKE) build-android-target TARGET=x86_64-linux-android ABI=x86_64
	@echo ""
	@echo "Native libraries built:"
	@ls -la android/src/main/jniLibs/*/libpeat_lite_android.so 2>/dev/null || echo "  (none found)"

# Generate Kotlin bindings from UniFFI
generate-bindings: build-android
	@echo "Generating Kotlin bindings..."
	cd android-ffi && cargo run --bin uniffi-bindgen generate \
		--library target/aarch64-linux-android/release/libpeat_lite_android.so \
		--language kotlin \
		--out-dir ../android/src/main/java
	@echo "Kotlin bindings generated in android/src/main/java/"

# Build complete AAR package
build-aar: generate-bindings
	@echo "Building AAR..."
	cd android && ./gradlew assembleRelease
	@echo ""
	@echo "AAR built at:"
	@ls -la android/build/outputs/aar/*.aar 2>/dev/null || echo "  (build failed)"

# ============================================
# Publishing Targets
# ============================================

# Publish to local Maven repository (~/.m2) for testing
publish-local: generate-bindings
	@echo "Publishing to local Maven repository..."
	cd android && ./gradlew publishToMavenLocal
	@echo ""
	@echo "Published to ~/.m2/repository/com/defenseunicorns/peat-lite/"
	@ls -la ~/.m2/repository/com/defenseunicorns/peat-lite/ 2>/dev/null || echo "  (not found)"

# Publish to Maven Central (requires SONATYPE_USERNAME and SONATYPE_PASSWORD)
publish-maven-central: generate-bindings
	@echo "Publishing to Maven Central..."
	cd android && ./gradlew publishToMavenCentral --no-configuration-cache
	@echo ""
	@echo "Published to Maven Central"

# ============================================
# CI Targets
# ============================================

ci: ci-rust ci-android
	@echo ""
	@echo "✓ All CI checks passed!"

ci-rust:
	@echo "Running Rust CI checks..."
	cargo fmt --check
	cd android-ffi && cargo fmt --check
	cargo clippy --features std -- -D warnings
	cd android-ffi && cargo clippy -- -D warnings
	cargo test --features std
	cd android-ffi && cargo test
	cargo build --no-default-features  # Verify no_std works
	@echo "✓ Rust CI passed"

ci-android: build-android
	@echo "✓ Android CI passed (native libs built)"

# Pre-commit hook target (for CLAUDE.md compliance)
pre-commit: fmt-check clippy test
	@echo "✓ Pre-commit checks passed"

# ============================================
# Clean
# ============================================

clean:
	cargo clean
	rm -rf android/src/main/jniLibs/*/libpeat_lite_android.so
	rm -rf android/build
	@echo "Cleaned build artifacts"
