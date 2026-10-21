# C0pdf guest-page lane report

Status: complete. Branch: `c0/main`. Nothing pushed. Upstream mirror `main` remains at
`416495145354828a9efefc39df381ba3ee8f864f` (PdfCraft 0.10.1).

## Commits

- `e4ee673190f380a0a7484efa01e66891aa914dd7` — `feat(c0pdf): embed PdfCraft as a C0VIBE guest page`.
- This report and the committed ratchet attestation follow in a `docs(c0pdf)` commit; its hash
  is printed in the final stdout hand-back.

## Delivered

`c0/pdfcraft-host` builds the existing `pdfcraft-web` binary with `page-control`, hashed relative
assets and output `dist/c0host`. The canvas starts transparent and no document is created at
startup. The page function feeds the app's existing control queue through `attach_control` and
the actual WebRunner's unpainted passes. The existing parent postMessage bridge, URL options,
storage and beforeunload guard are retained.

All seven requested `c0.*` methods work. Inspect includes the requested PDF fields and history,
dirty, revision, pane and theme state. Export is read only; imports are validated before the
existing Save Changes path replaces only the active document. Browser Open supports bounded
reads, cancellation and rejection of late chooser results. All Chrome defaults are true and
unsaved; the guest selects its own hidden title/Home/community policy. All 20 colour tokens
are overridable across light/dark changes. Community promotions disappear from Home, About,
Help, the palette and the title strip, retaining PdfCraft, contributors and licence.

Generic wasm corrections: inject the browser UTC clock and local offset into the session;
replace the unsupported SystemTime call used by relative dates; choose download MIME by
extension. All three are marked **Offered upstream: yes, as is** in PATCH-DEBT. No engine file,
font, asset, feature registry or parent bridge implementation changed. No external PR was sent.

## Gates and counts

| Gate | Result |
|---|---|
| Native `pdfcraft-web --features page-control` (K-41) | **2 passed**, 0 failed |
| Full engine suites, including doctests | **298 passed**, 0 failed, 3 upstream ignores |
| Full ui-egui suites, including doctests | **1,368 passed**, 0 failed, 5 upstream ignores |
| New c0 UI contracts / browser-helper native tests | **9 + 2 passed**, included in UI total |
| Combined native run | **1,668 passed**, 0 failed, 8 upstream ignores |
| `cargo fmt --all --check` | passed |
| Native clippy, both touched app crates, all targets, `-D warnings` | passed |
| `cargo check --target wasm32-unknown-unknown -p pdfcraft-web --features page-control` | passed |
| Clock-probe wasm build and clippy `-D warnings` | passed |
| `cargo xtask layers` | passed: 37 crates, no violations |
| `cargo xtask assets` | passed: 272 repository assets, 6 dependency-bundled and 22 fetched asset entries |
| `NO_COLOR=true trunk build --release` | passed; final Rust release compilation 3m 58s, followed by wasm optimization |
| Headless system Chromium | **44 recorded checks passed**, no runtime exceptions or page-render errors |

The Chromium run checked empty startup, every c0 method, an actual disk-backed synthetic PDF,
2 pages, 1 bookmark, 1 annotation and 1 form field; PDF export/open round trip; page/outline
navigation; palette validation and dark-mode retention; cancellation followed by a late real
chooser result; replies while animation frames were held; the actual Save Changes Cancel and
Don't save buttons; and current-year `/ModDate`. The registry exposes 202 commands including
`ui.dock`, confirming 201 engine commands. The final screenshot was visually inspected at
`/tmp/c0pdf-lane/browser.png`. Synthetic documents and the screenshot remain local test output.

Regression evidence: restoring the previous all-PDF MIME implementation made
`exported_files_use_their_own_mime_type` fail (application/pdf versus image/png); restoring the
fix passed. The restored source SHA-256 matched the source from the passing full suites.
The isolated test-only wasm clock probe caught the exact previous unsupported SystemTime call
as a WebAssembly RuntimeError and verified the actual replacement helper within 2 seconds of
browser UTC. That probe is a separate test workspace and adds nothing to the shipped site.

Native functional gates use `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`,
`CARGO_PROFILE_DEV_OPT_LEVEL=0`, `CARGO_PROFILE_TEST_OPT_LEVEL=0`, and `RUST_TEST_THREADS=1`.
Upstream dependency profile settings remain unchanged. The default debug UI compilation hit
the one-hour timeout before executing tests; the lighter native profile completed all suites.
An initial two-thread forms group passed 28 tests and failed one because two upstream tests
share `scratch("same-frame")` / process-id temporary paths. The complete sequential run passed
without changing or excluding tests. An initial helper return-type error and unused test import
were corrected before the final gates. Wasm checking retains 3 existing UI dead-code warnings;
the release build also retains 4 existing renderer dead-code warnings.

All heavy work ran here under `smoac-slot`, with `CARGO_BUILD_JOBS=2`; Binaryen used 2 threads.
The first complete gate run waited 3,386 seconds for heavy admission. No fleet redispatch,
Gen3 lease/claim change, shared-branch push or SSB telemetry was attempted. The controller owns
telemetry and collection. Reproduction scripts and protocol details are in README.md.

## Artifact sizes

| Artifact | Bytes |
|---|---:|
| `pdfcraft-web-45bd5c8bad8ec29b_bg.wasm` | **39,235,504** (37.42 MiB) |
| wasm gzip level 6 measurement | 14,486,950 (13.82 MiB) |
| Generated JavaScript | 167,738 |
| `index.html` | 1,233 |
| **Total `dist/c0host` (3 files)** | **39,404,475** (37.58 MiB) |

Wasm SHA-256: `d49dc023d80fed5ae61d71a29ba4ac64e9fea7820508779baff5a7c163f2693a`.
The scout's roughly 13 MB raw / 5 MB compressed estimate was not reproduced by this checkout.
The full application feature set remains enabled; the ordinary upstream release profile and
Trunk `data-wasm-opt="z"` are used. The build was made from the validated working tree before
the implementation/report commits.

`cat /proc/loadavg` immediately before the final page build:

```text
15.12 15.23 17.15 7/2273 3558450
```

Immediately after the final build:

```text
29.23 26.47 21.80 38/2624 4000802
```

## Every modified upstream file

Added lines from `git diff --numstat --diff-filter=M main c0/main -- . ':!c0'`:

| File | Added lines |
|---|---:|
| `Cargo.lock` | 1 |
| `apps/pdfcraft-web/Cargo.toml` | 4 |
| `apps/pdfcraft-web/src/main.rs` | 9 |
| `crates/ui-egui/src/chrome.rs` | 5 |
| `crates/ui-egui/src/combine_ui.rs` | 1 |
| `crates/ui-egui/src/commands.rs` | 6 |
| `crates/ui-egui/src/control.rs` | 5 |
| `crates/ui-egui/src/dialogs.rs` | 4 |
| `crates/ui-egui/src/docking.rs` | 5 |
| `crates/ui-egui/src/editing.rs` | 1 |
| `crates/ui-egui/src/home.rs` | 4 |
| `crates/ui-egui/src/lib.rs` | 18 |
| `crates/ui-egui/src/palette.rs` | 3 |
| `crates/ui-egui/src/theme.rs` | 2 |
| **Total / Ratchet** | **68** |

PATCH-DEBT.md records purpose, deleted/moved blocks and upstream-offer status. New files are
excluded by the requested modified-file ratchet. `main` was never committed to.

## Remaining upstream colour literals

All Tokens colour fields are supported; existing literals elsewhere remain. The most visible
exceptions are `edit_text_ui.rs`'s ACCENT, `forms_ui.rs`'s FOCUS_BLUE/FIELD_FILL, `comments.rs`'s
SELECT_BLUE, signing drag/selection blues in `sign_ui.rs`, measurement palettes in
`measure_ui.rs`, and tool/badge hues in `panels.rs` / `widgets.rs`. PDF paper and document colours
also retain their original values. A source scan before cfg(test) found 71 literal numeric RGB
call sites outside theme.rs; a broader line scan including named colours found 225 lines. These
filters differ from the scout's 111-colour count and are not claims of full colour coverage.

## Assumptions I could not confirm

- The actual C0VIBE parent application is outside this checkout. Its mounting/auth/session
  integration was not exercised; the finished standalone guest contract was tested in Chromium.
- `CRAFT_FONTS_DIR` was unset. No craft-fonts Japanese face was embedded: browser Japanese UI
  may show replacement boxes, and Japanese PDF writing reports the engine's missing-face error.
  Japanese font-dependent coverage is therefore limited; no font was fetched or committed.
- Existing web signing-save, OCR-model and attachment-download limitations remain visible as
  upstream provides them. The guest does not supply those missing web implementations.

Detailed gate logs are in `/tmp/c0pdf-lane`. No files from the test-only clock probe or generated
PDF fixtures are included in `dist/c0host`.
