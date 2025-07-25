#!/bin/bash

# Build script for Flutter integration
# This script builds the Rust library for different platforms

set -e

echo "Building Rust library for Flutter integration..."

# Build for current platform (for testing)
echo "Building for current platform..."
cargo build --release

# Create output directory
mkdir -p flutter_lib

# Copy the library file based on the platform
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    echo "Detected macOS, copying .dylib file..."
    cp target/release/libexcel_to_xml.dylib flutter_lib/
    
    echo "Checking for iOS targets..."
    if rustup target list --installed | grep -q "x86_64-apple-ios"; then
        echo "Building for iOS simulator (x86_64)..."
        cargo build --release --target x86_64-apple-ios
    else
        echo "iOS simulator target not installed. Run: rustup target add x86_64-apple-ios"
    fi
    
    if rustup target list --installed | grep -q "aarch64-apple-ios"; then
        echo "Building for iOS device (aarch64)..."
        cargo build --release --target aarch64-apple-ios
        
        if [ -f "target/x86_64-apple-ios/release/libexcel_to_xml.a" ] && [ -f "target/aarch64-apple-ios/release/libexcel_to_xml.a" ]; then
            # Create universal iOS library
            mkdir -p flutter_lib/ios
            lipo -create \
                target/x86_64-apple-ios/release/libexcel_to_xml.a \
                target/aarch64-apple-ios/release/libexcel_to_xml.a \
                -output flutter_lib/ios/libexcel_to_xml.a
            echo "Universal iOS library created at flutter_lib/ios/libexcel_to_xml.a"
        fi
    else
        echo "iOS device target not installed. Run: rustup target add aarch64-apple-ios"
    fi
        
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    echo "Detected Linux, copying .so file..."
    cp target/release/libexcel_to_xml.so flutter_lib/
    
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    # Windows
    echo "Detected Windows, copying .dll file..."
    cp target/release/excel_to_xml.dll flutter_lib/
fi

echo "Build completed! Library files are in the flutter_lib directory:"
ls -la flutter_lib/
echo ""
echo "For Flutter integration:"
echo "1. Copy the appropriate library file to your Flutter project"
echo "2. Use the example code in flutter_example/ directory"
echo "3. Make sure to add 'ffi: ^2.0.0' to your pubspec.yaml dependencies"
