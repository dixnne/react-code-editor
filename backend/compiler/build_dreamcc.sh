#!/bin/bash
# Build script for dreamcc with proper LLVM configuration

export LLVM_SYS_180_PREFIX=/usr/lib/llvm-18

echo "Building Dream Language Compiler (dreamcc)..."
cargo build --bin dreamcc "$@"

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ Build successful!"
    echo "  Binary: target/debug/dreamcc"
    echo ""
    echo "To install system-wide:"
    echo "  sudo cp target/debug/dreamcc /usr/local/bin/"
else
    echo "✗ Build failed"
    exit 1
fi
