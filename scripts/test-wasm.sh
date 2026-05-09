#!/usr/bin/env bash
set -euo pipefail

WASI_SYSROOT_DIR="/tmp/wasi-sysroot-33.0+m"
CUSTOM_SYSROOT="/tmp/wasm-custom-sysroot"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

export CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime -W simd --dir=$PROJECT_DIR"
export RUSTFLAGS="-C target-feature=+simd128 --sysroot=$CUSTOM_SYSROOT"

cd "$PROJECT_DIR"
cargo test --target wasm32-wasip1 \
    -Zbuild-std=std,panic_abort \
    -Zpanic-abort-tests \
    -p resharp \
    "$@"
