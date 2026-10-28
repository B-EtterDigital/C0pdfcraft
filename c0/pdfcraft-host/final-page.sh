#!/usr/bin/env bash
# Final app/probe build and browser gates. Invoke under smoac-slot from the executor.
set -euo pipefail
cd "$(dirname "$0")/../.."
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}" BINARYEN_CORES=2
if [[ -n "${CRAFT_FONTS_DIR:-}" && ! -d "$CRAFT_FONTS_DIR" ]]; then unset CRAFT_FONTS_DIR; fi
logs="${C0PDF_LOG_DIR:-/tmp/c0pdf-lane}"
mkdir -p "$logs"
run() {
  local label="$1"
  shift
  printf '%s: running\n' "$label"
  local status=0
  timeout 3600 "$@" > "$logs/$label.log" 2>&1 || status=$?
  printf '%s\n' "$status" > "$logs/$label.exit"
  printf '%s: exit %s\n' "$label" "$status"
  return "$status"
}
run fmt-page-final cargo fmt --all --check
run wasm-check-final cargo check --target wasm32-unknown-unknown -p pdfcraft-web --features page-control
run layers-final cargo xtask layers
run assets-final cargo xtask assets
run clock-probe-build bash c0/pdfcraft-host/clock-probe/build.sh
run clock-probe-clippy env CARGO_TARGET_DIR="$PWD/target" cargo clippy --manifest-path c0/pdfcraft-host/clock-probe/Cargo.toml --target wasm32-unknown-unknown -- -D warnings
cat /proc/loadavg > "$logs/load-before-build-final.txt"
run page-build-final bash c0/pdfcraft-host/build-page.sh
cat /proc/loadavg > "$logs/load-after-build-final.txt"
run chromium-final env C0PDF_CLOCK_PROBE_DIR="$logs/clock-probe" node c0/pdfcraft-host/smoke.mjs
