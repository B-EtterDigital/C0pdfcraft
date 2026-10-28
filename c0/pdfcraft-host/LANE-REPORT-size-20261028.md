# C0pdf page size lane — 2026-10-28

## Scope and reproduction

Lane branch: `fix`, starting at `1aa9b7af989becc0cf0de6af0901ebbc1d619e91`.
Validated code/build commit: `35f9204dc7a751f6bf20c9436062515ce3166eb6`.
Pure upstream: `5af102765222c8220df0f9eaa27bfa3bcf481ead`, in the detached
`/srv/smoac/build/sow-size-pdfcraft-upstream-20261028` worktree.
Cap: **41,943,040 bytes**, unchanged. Target: below 40,000,000 bytes.
No push or deployment was attempted.

Every heavy command runs under `smoac-slot run gate 3 3600 --`, with
`CARGO_BUILD_JOBS=4` for Cargo builds and `BINARYEN_CORES=2` for Trunk.
`df -h /` is checked before builds; no build starts above 88% disk use.
Toolchain: rustc 1.99.0 (`b940084d7`), Trunk 0.21.14, Binaryen 120, twiggy 0.8.0.
`CRAFT_FONTS_DIR` is unset in both measurements, as in the starting environment.

Baseline command, from `c0/pdfcraft-host`:

```sh
smoac-slot run gate 3 3600 -- env CARGO_BUILD_JOBS=4 BINARYEN_CORES=2 NO_COLOR=true trunk build --release
```

Upstream has no C0 page or `page-control` feature. Its temporary, untracked
`c0/pdfcraft-host` contains copies of the baseline Trunk config and HTML, with
only `data-cargo-features="page-control"` removed. All Rust sources, manifests
and lockfile stay pure upstream. The exact same command is used, adding only
`CARGO_TARGET_DIR` to reuse lane-owned dependency artifacts sequentially.
Both builds use `data-wasm-opt="z"` and `data-weak-refs`.

## Measurements

| Step | Final wasm bytes | Result |
|---|---:|---|
| Published revision (brief, not rebuilt) | 39,244,638 | Reference |
| HEAD baseline, upstream release profile | 46,351,642 | 4,408,602 above cap |
| Pure upstream, same release build | 46,268,773 | 4,325,733 above cap |
| Page-only `z` + fat LTO + one codegen unit + debug stripping | 30,384,877 | 11,558,163 below cap |
| Same profile after required asset-gate repair | 30,384,666 | Final acceptance artifact; 11,558,374 below cap |

The baseline differs by 19 bytes from the brief's 46,351,623-byte candidate.
Rust compilation took 9m 19s, followed by Trunk/wasm-opt processing.
Pure upstream compilation took 5m 24s with cached dependencies. The first tuned
build took 8m 28s of Rust compilation and reduced final size by 15,966,765 bytes
(34.45%). The fork adds
82,869 bytes over pure upstream; upstream alone is already over the cap.
Final sizes are uncompressed `_bg.wasm` files, never gzip sizes or the larger
compiler outputs containing symbol names and wasm-bindgen metadata.
The final artifact is 15,966,976 bytes (34.45%) smaller than the baseline and
9,615,334 bytes below the 40,000,000-byte aim. Final Rust compilation took
9m 27s, followed by wasm-opt. Profile tuning met the target without any feature
pruning; the required asset repair reduced the tuned artifact by another 211 bytes.

## What grew

An existing clean checkout at published fork commit
`304539a5e385d1a3513bfb4f96a73ca376168361` retains a 39,244,651-byte wasm.
It is 13 bytes larger than the brief's published file; it is used only for
additional section-size evidence, not substituted for the requested builds.

| Wasm section payload | Older local artifact | HEAD baseline | Increase |
|---|---:|---:|---:|
| Code | 29,059,466 | 33,045,338 | 3,985,872 |
| Data | 10,085,223 | 13,200,524 | 3,115,301 |

The major identifiable data growth is `pdfcraft-ui-egui` localization:
22 TSV catalogs / 3,519,308 source bytes became 34 / 6,152,589 bytes
(+2,633,281). `i18n.rs` embeds them with `include_str!`; every catalog's entire
byte sequence was found verbatim in its respective final wasm (22/22 and
34/34). This is measured shipped data growth, not only a source-size proxy.
All languages remain supported.

The upstream dependency diff from `41649514` to `5af102765222` also adds
`craft-i18n` (ICU decimal/plural/locale fallback and embedded CLDR data),
`craft-menu`, `craft-keymap`, and `rten/onnx_format` → `rten-onnx`, and upgrades
egui/epaint/eframe from 0.36.2 to 0.37.0. It explicitly enables monochrome
emoji fonts; the current two emoji font files total 742,936 bytes, too small
alone to explain the regression. This is not claimed as entirely new font data:
the old egui version already bundled emoji fonts.

`twiggy top -n 40` on the HEAD compiler wasm identifies UI canvas closures
(182,766 bytes), dialogs closures (139,900), canvas document area (133,200),
app drop glue (99,577), dialogs (79,747), and restore (73,936) as its largest
code items. Combine workflow, renderer inspection, redaction, hayro, WGPU/Naga,
image resizing and two skrifa versions also appear. Its 13,444,867-byte rodata
segment dominates shallow size. The 4,766,949-byte function-name subsection
and 802,616-byte wasm-bindgen metadata are compiler-only analysis overhead,
not shipped size that can be saved again by stripping the published file.

These symbols and manifest changes identify the affected crates/features;
they are not a byte-exact per-crate attribution of the old-to-new optimized
binary. Optimizer inlining, sharing and dead-code removal prevent simply
adding source sizes or twiggy retained sizes to obtain that delta.
The full top-40 and dominator outputs for both compiler wasm files are in
[`SIZE-EVIDENCE-20261028.txt`](SIZE-EVIDENCE-20261028.txt). Dominators at depth
two are mostly the shared indirect-call table (89.38% HEAD / 89.39% upstream),
so retained sizes are not treated as independently additive crate costs.

## Change

`build-page.sh` scopes release environment overrides to the guest page:
`opt-level=z`, `lto=fat`, `codegen-units=1`, `strip=debuginfo`.
`final-page.sh` calls it, so the controller's payload uses the same tuning.
The documented standalone build also uses it. `final-page.sh` now honors the
dispatcher's Cargo job setting, defaulting to its previous two jobs otherwise.
Trunk already runs `wasm-opt -Oz`; that setting stays in place.
No feature, codec, OCR format, language, font or C0 bridge behavior is removed.
Panic strategy is untouched, per the repository's recovery requirement.
Native and ordinary upstream web release profiles are unchanged.
All 34 catalog byte sequences are also present verbatim in the first tuned
wasm. Its code payload is 17,822,273 bytes and data payload 12,445,461 bytes:
the reduction is principally code optimization, not a new metadata-strip trick.

The asset gate exposed a pre-existing fork inconsistency: eight attribution
records still named removed trademarks, and three named blank SVG replacements
whose hashes differed from the original marks. Rather than restore trademarks,
the gate repair removes those obsolete records and placeholder SVGs, regenerates
the attribution Markdown, and re-exports two C0-owned empty layout functions
from `widgets.rs`. The wordmark uses egui's own `ImageSize` calculation with
the blank SVG's 1:1 aspect and the previous height cap; the square mark retains
its explicitly requested size. Both allocate the previous hover-only responses
and retain the image role and alt text for accessibility/headless inspection.
No new asset is introduced. The prebuilt asset checker passes after this repair.

Patch debt is **63 added lines**, below the 65-line ratchet: the three placeholder
SVG lines are replaced by one widget re-export and one regenerated count heading.
The starting table said 63 but omitted the existing one-line `Cargo.lock` JSON
dependency entry; the corrected starting total was 64. The manifest cleanup is
deletions only. No engine file is modified.
No module graph or Gen3 config exists in this checkout; targeted file reads
were used, without changing the executor's prohibited control plane.

## Validation, load and artifacts

The final source was verified with the normal upstream dev/test optimization
level (1), debug info disabled, one test thread and four Cargo build jobs:

```sh
cargo test -p pdfcraft-ui-egui -p pdfcraft-web --features page-control --no-fail-fast
cargo clippy -p pdfcraft-ui-egui -p pdfcraft-web --all-targets --features page-control -- -D warnings
```

| Native attempt | Passed | Failed | Ignored | Explanation |
|---|---:|---:|---:|---|
| Initial low-cost `opt-level=0` | 1,783 | 1 | 11 | 500-page scrolling averaged 46.019 ms, over the unchanged 40 ms budget; Cargo stopped before the web tests |
| Normal optimization, before asset repair | 1,786 | 0 | 11 | Includes all three performance tests, which ran for 11.05 s |
| First asset repair | 1,785 | 1 | 11 | Existing About test caught missing image alt text on the empty slots; fixed without changing the test |
| Final source | 1,786 | 0 | 11 | All eight test targets/doc-test summaries green; three performance tests ran for 16.15 s |

No test, assertion, budget, feature or ignore annotation was relaxed. The initial
asset-gate failure (eight obsolete trademark records/hashes) was repaired as
described above; the prebuilt checker then passed with 281 repository assets,
6 bundled dependency assets and 22 fetched-at-build-time assets.

Final `cargo fmt --all --check`, native clippy with `-D warnings`, `cargo xtask
layers` (37 crates) and `cargo xtask assets` all passed. Chromium smoke passed
**62 checks**, exposed **208 registered commands**, and reported **zero runtime
errors**. It covers empty startup, Open/New, replacement Cancel/Discard,
navigation, byte export, browser-chooser cancellation, theme tokens, strip/dock
modes, frameless render with animation frames disabled, and browser UTC save dates.
The first browser attempt failed writing `/tmp/c0pdf-lane/browser.png` because
that output directory was missing. Creating it and rerunning the unchanged
harness succeeded. The corrected run waited for host gate capacity; no slot
capacity, reservation, hold or control-plane setting was changed.
Screenshot: `/tmp/c0pdf-lane/browser.png`. The separate isolated clock-trap probe
was not rebuilt; the existing integrated browser UTC/ModDate check passed.
Release wasm compilation retains the upstream's existing
render/UI conditional-compilation warnings; no wasm warning-clean claim is made.

Final artifact: `dist/c0host/pdfcraft-web-a426b5b02484ac46_bg.wasm`.
SHA-256: `bd03cf3dae9e692223d0fe72a7cf5a9d428fa5ef2714466c81a0e0430239e0f2`.
All 34 catalog byte sequences remain verbatim in this final file as well.

Load averages below are one/five/fifteen minutes, sampled around command
submission/completion; slot admission can wait between submission and execution.

| Measurement | Before | After |
|---|---|---|
| Session start / baseline | 24.32 / 24.44 / 26.26 (earliest session snapshot) | 27.32 / 24.00 / 25.03 |
| Pure upstream | 27.32 / 24.00 / 25.03 | 21.77 / 24.00 / 24.40 |
| First tuned page | 21.77 / 24.00 / 24.40 | 17.72 / 17.79 / 19.44 |
| Final page after asset repair | 29.53 / 24.73 / 21.27 | 25.94 / 50.11 / 41.10 |

Disk use was 81% before the baseline, 82% before upstream, and 83% before the
first tuned build. It reached 85% during shared host activity; no build was
launched above the 88% stop threshold. The final verification script checks
disk use again inside its admitted slot before each build/check.
The host ran other lanes concurrently; observed load1 reached 77.16 during the
final link, with swap nearly full. These are whole-host snapshots, not a claim
that the page build alone caused that load. Cargo stayed at four jobs and
Binaryen at two cores throughout.

Raw build logs and analysis outputs are retained in
`/tmp/c0pdf-size-20261028/` on this executor. Compiler wasm files are kept there
for symbol analysis; build artifacts are not committed.
