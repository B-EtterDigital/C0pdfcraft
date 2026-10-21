#!/usr/bin/env bash
# Functional native gates without full workspace debug info / optimized UI code generation.
# Dependency package settings in upstream Cargo.toml remain unchanged.
set -uo pipefail
cd "$(dirname "$0")/../.."
export CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=1
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
export CARGO_PROFILE_DEV_OPT_LEVEL=0 CARGO_PROFILE_TEST_OPT_LEVEL=0
if [[ -n "${CRAFT_FONTS_DIR:-}" && ! -d "$CRAFT_FONTS_DIR" ]]; then unset CRAFT_FONTS_DIR; fi
logs="${C0PDF_LOG_DIR:-/tmp/c0pdf-lane}"
mkdir -p "$logs" || exit 1
failed=0
run() {
  local label="$1"
  shift
  printf '%s: running\n' "$label"
  timeout 3600 "$@" > "$logs/$label.log" 2>&1
  local status=$?
  printf '%s\n' "$status" > "$logs/$label.exit"
  printf '%s: exit %s\n' "$label" "$status"
  if (( status != 0 )); then failed=1; fi
}
run fmt-final cargo fmt --all --check
run web-native-final cargo test -p pdfcraft-web --features page-control
run upstream-tests-final cargo test -p pdfcraft-ui-egui -p pdfcraft-engine -p pdfcraft-web --features page-control
run clippy-final cargo clippy -p pdfcraft-ui-egui -p pdfcraft-web --all-targets --features page-control -- -D warnings
exit "$failed"
