# C0pdf tool-mode consistency lane report

Branch `c0/main`, nothing pushed.

## Tool-mode decision

PdfCraft has a real tool strip: the quick-action bar (`canvas.rs` `quick_bar`), a one-column strip
over the left edge of the document area. It holds Select, Hand, three comment groups, Fill & Sign
and Edit text, each with flyouts. The tools panel (`tools_panel`) is a catalogue list of tool
groups, not a strip, so it keeps its existing switch and gets no column modes.

PdfCraft's strip is already an overlay anchored at the corner, so "docked" and "floating" differ
in whether you can move it:

- `toolbar` (default true): the strip is drawn, as PhotoCraft's `toolbar` switch means it.
- `toolbar_double` (default false): two columns.
- `toolbar_floating` (default false): a grip above the strip drags it within the document area.
  Upstream's fixed corner is the default.
- All three are unsaved and reported by `c0.inspect` under `panes.chrome` and `panes.strip`.
- Upstream's `quick_bar` still draws every button; we only choose whether, where and how many
  columns.
- Right dock: `right_dock_collapsed` drops the Inspector from the rendered dock and from Read
  mode's side panel. The rail keeps its icons, and pressing one opens that pane and unfolds.

## Commits

- `4e2fedc` feat(c0pdf): tool-strip modes, right-dock fold and frameless ui.render
- `0eb0657` test(c0pdf): cover tool modes and ui.render in the browser smoke
- a docs(c0pdf) commit with README, PATCH-DEBT and this report (hash in the hand-back)

## Tests

- `crates/ui-egui/tests/c0_tool_modes.rs`: 10 tests covering defaults, set/read-back, idempotence,
  invalid and unknown keys, hide, two columns, floating drag and clamp, the combination, fold and
  unfold from rail icons, the Read-mode fold, and `ui.render` (valid and refused requests, with no
  frame stepped).
- `c0_host_contract.rs`: the old "every Chrome field is true" assertion is now the exact default
  set.
- Combined `cargo test -p pdfcraft-ui-egui -p pdfcraft-engine -p pdfcraft-web --features page-control`
  (low-opt profile): **1,678 passed, 0 failed**.
- `cargo fmt --check`, clippy `--all-targets -D warnings` on both app crates, and
  `cargo xtask layers` and `assets`: pass.
- Wasm check and clock-probe build/clippy: pass.
- `trunk build --release`: pass.
- Chromium smoke: **62 checks, no runtime errors**. It sets each mode and reads it back, and runs
  `ui.render` while animation frames are held.

## Sizes and load

- wasm `pdfcraft-web-5f479aa4d0bab3b3_bg.wasm`: **39,244,651** bytes, +9,147 against the previous
  lane's 39,235,504.
- Load average (1 minute) before the release build: 10.72 (the full-gates run started at 20.50).
  After: 15.10. The host is shared, so these are not attributable to this build alone.
- Patch debt ratchet: **65** (was 68). The lib.rs title/app-bar/rail gates became one-line
  `c0_host::when` calls to pay for the strip and fold lines.

## ui.render

`ui.render {page?, scale?}` returns `{page, pageCount, scale, width, height, png_base64}` from
the engine's exporter. `page` is zero-based and defaults to the current page. `scale` runs from
0.25 to 16 and defaults to 1 (one pixel per point). It is answered inline and never presents a
frame. The browser bridge treats it as read-only.

## Assumptions I could not confirm

- The "pin cap" for wasm size is not recorded anywhere in the repo, so I only report the delta.
- I read "docked" as the strip's upstream fixed corner. If the controller wants a strip that takes
  layout width, that is a separate change in `canvas.rs`.
- Folding the dock widens the document area. Fit-width and fit-page views re-fit to the new width,
  so a fit view's zoom changes; manual zoom and scroll are kept.
- While folded, the rail still highlights the pane that was open.
- Two columns wrap at half the height of the current buttons and options. A comment tool's extra
  options can make a third column.
- Pointer-drag of the grip is covered by native tests and by `ui.drag`; there is no headless
  Chromium drag test.
