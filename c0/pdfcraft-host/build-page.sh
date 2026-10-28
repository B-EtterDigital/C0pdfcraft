#!/usr/bin/env bash
# Size tuning belongs to the guest page, not the workspace's native releases.
set -euo pipefail
cd "$(dirname "$0")"
export CARGO_PROFILE_RELEASE_OPT_LEVEL=z
export CARGO_PROFILE_RELEASE_LTO=fat
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
export CARGO_PROFILE_RELEASE_STRIP=debuginfo
# Keep upstream's panic strategy: document entry points rely on catch_unwind.
if [[ -n "${CRAFT_FONTS_DIR:-}" && ! -d "$CRAFT_FONTS_DIR" ]]; then unset CRAFT_FONTS_DIR; fi
NO_COLOR=true trunk build --release "$@"
