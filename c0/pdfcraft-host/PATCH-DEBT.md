# PdfCraft upstream patch debt

Only `c0/main` receives commits. `main` remains the upstream mirror. Most host code is in new
`c0_*.rs`, `page_control*`, `theme/host.rs` and `tests/c0_*.rs` files, and this page directory.
The table counts **added** lines in modified upstream files, including replacements. Deleted
lines are still patch debt and are described below; they do not contribute to the requested ratchet.

| File | Lines | What | Why | Offered upstream |
|---|---:|---|---|---|
| `Cargo.lock` | 1 | Record the web binary's direct JSON dependency | Reproducible feature builds | no, dependency bookkeeping |
| `apps/pdfcraft-web/Cargo.toml` | 4 | Feature and JSON dependency; feature enables native window backends | Opt-in bridge; native feature tests (K-41) | no, host feature |
| `apps/pdfcraft-web/src/main.rs` | 9 | Register runner and attach bridge under feature | Same binary and existing startup; parent bridge unchanged | no, host transport |
| `crates/ui-egui/src/canvas.rs` | 2 (2 replaced) | The call to `quick_bar` goes through `c0_host::strip::show` (hidden, floating, grip); the strip's `ui.vertical` becomes `ui.scope_builder(c0_host::strip::columns(..))` (one or two columns) | Tool-strip modes; defaults draw exactly what upstream drew | no, host policy |
| `crates/ui-egui/src/chrome.rs` | 5 | Gate Discord button | Remove chrome promotion when requested | no, host policy |
| `crates/ui-egui/src/combine_ui.rs` | 1 | Use browser-safe clock in relative dates | `SystemTime::now()` panics on wasm32-unknown-unknown | **yes, as is** |
| `crates/ui-egui/src/commands.rs` | 6 | Gate community command execution and menu rows | Hide links consistently and refuse hidden actions | no, host policy |
| `crates/ui-egui/src/control.rs` | 5 | Host trait method, forwarding implementation and one dispatch arm for `c0.*` and `ui.render` | Reuse the existing queue | no, host API |
| `crates/ui-egui/src/dialogs.rs` | 4 | Gate About mark; delegate footer to new module | Retain name, contributors and licence without promotions | no, host policy |
| `crates/ui-egui/src/docking.rs` | 5 | Render projection and divider path translation (also while the right dock is folded) | Hide Tools without overwriting saved docking geometry | no, host policy |
| `crates/ui-egui/src/editing.rs` | 1 | Select browser download MIME by file extension | Exports other than PDF must not be labelled application/pdf | **yes, as is** |
| `crates/ui-egui/src/home.rs` | 4 | Early Home gate and delegate community card | Empty pasteboard; no hidden-card space | no, host policy |
| `crates/ui-egui/src/lib.rs` | 13 | Register modules/unsaved Chrome, initialize session, picker hook, replacement poll, and the title/app-bar/rail gates as one `c0_host::when` line each (9 lines before the tool-mode lane), plus the Read-mode side panel's fold gate | Host integration plus browser session wall clock | clock initialization: **yes, as is**; remaining lines: no |
| `crates/ui-egui/src/palette.rs` | 3 | Filter community commands | Palette cannot reintroduce hidden promotions | no, host policy |
| `crates/ui-egui/src/theme.rs` | 2 | New override module and one token resolution line | Persistent runtime overrides across theme changes | no, host palette |
| `docs/brand/artcraft-logo-white.svg` | 1 | Blank 1x1 SVG in place of the ArtCraft mark the build embeds (`widgets.rs`) | Upstream trademarks never ship in our build (K-46); the code still compiles unchanged | no |
| `docs/brand/artcraft-logo.svg` | 1 | Blank 1x1 SVG in place of the ArtCraft mark the build embeds (`widgets.rs`) | Upstream trademarks never ship in our build (K-46); the code still compiles unchanged | no |
| `docs/brand/artcraft-mark.svg` | 1 | Blank 1x1 SVG in place of the ArtCraft mark the build embeds (`widgets.rs`) | Upstream trademarks never ship in our build (K-46); the code still compiles unchanged | no |

Total: 68

Ratchet: 68

Measured on the tool-mode commit (was 68 at `e4ee673190f380a0a7484efa01e66891aa914dd7`; the lib.rs gates were compacted to pay for the strip and fold lines):

```sh
git diff --numstat --diff-filter=M main c0/main -- . ':!c0'
```

The added column is summed. New files do not count. `home.rs` and `dialogs.rs` also delete the
upstream community blocks moved into `c0_host/community.rs`; their default rendering is retained.
The three browser corrections live in new `c0_web.rs` / `c0_clock.rs` for this fork; an upstream submission can
place the same clock/MIME helpers in upstream's preferred modules. No engine file is modified.
