#!/usr/bin/env bash
# Invoke under smoac-slot. Writes only test artifacts outside the site distribution.
set -euo pipefail
cd "$(dirname "$0")/../../.."
export CARGO_BUILD_JOBS=2
export CARGO_TARGET_DIR="$PWD/target"
probe_logs="${C0PDF_LOG_DIR:-/tmp/c0pdf-lane}"
mkdir -p "$probe_logs/clock-probe"
cargo build --release --target wasm32-unknown-unknown --manifest-path c0/pdfcraft-host/clock-probe/Cargo.toml
wasm-bindgen --target web --out-name clock_probe --out-dir "$probe_logs/clock-probe" "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/c0pdf_clock_probe.wasm"
