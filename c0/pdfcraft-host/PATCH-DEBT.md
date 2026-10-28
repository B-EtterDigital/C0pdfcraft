# PdfCraft upstream patch debt

The pinned upstream is `5af102765222c8220df0f9eaa27bfa3bcf481ead`. The pre-merge
`origin/c0/main` baseline was **65 added lines**. New fork-owned files do not count.
The table counts added lines, including replacements; deletions are described separately.
All host switches default to upstream behaviour. No engine file is modified.

| File | Added lines | Hook / reason |
|---|---:|---|
| `apps/pdfcraft-web/Cargo.toml` | 4 | Opt-in page-control feature, native backends for feature tests, JSON dependency |
| `apps/pdfcraft-web/src/main.rs` | 9 | Register runner and attach the opt-in bridge |
| `crates/ui-egui/src/app.rs` | 5 | Browser session clock, default Chrome, replacement polling and title/tab/app-bar gates (moved from lib.rs) |
| `crates/ui-egui/src/app/files.rs` | 4 | Opt-in browser picker hook plus crate visibility for finish_open after upstream moved it into a private submodule |
| `crates/ui-egui/src/canvas.rs` | 1 | Delegate the quick-bar visibility and position to c0_host |
| `crates/ui-egui/src/canvas/bars.rs` | 1 | Delegate the strip column layout (moved from canvas.rs) |
| `crates/ui-egui/src/chrome.rs` | 5 | Gate the Discord button |
| `crates/ui-egui/src/commands.rs` | 3 | Refuse community commands when links are hidden |
| `crates/ui-egui/src/control.rs` | 3 | Default host-extension method for upstream test hosts |
| `crates/ui-egui/src/control/host.rs` | 3 | Forward host requests to c0_host (moved from control.rs) |
| `crates/ui-egui/src/control/requests.rs` | 1 | Dispatch c0.* and ui.render over the existing queue (moved from control.rs) |
| `crates/ui-egui/src/dialogs.rs` | 4 | Gate About mark and delegate community footer |
| `crates/ui-egui/src/dialogs/palette.rs` | 3 | Filter community commands (moved from palette.rs) |
| `crates/ui-egui/src/editing/save.rs` | 1 | Select browser download MIME by file extension (moved from editing.rs) |
| `crates/ui-egui/src/home.rs` | 4 | Gate Home and delegate the upstream dismissible community card |
| `crates/ui-egui/src/lib.rs` | 3 | Register host/browser modules and transient Chrome field |
| `crates/ui-egui/src/menus.rs` | 1 | Filter community menu rows in the new craft-menu builder (moved from commands.rs) |
| `crates/ui-egui/src/panels/side_panels.rs` | 2 | Project visible panes and gate rail without changing saved widths/open states (replaces docking.rs and Read-mode gate) |
| `crates/ui-egui/src/theme.rs` | 2 | Register override module and resolve tokens |
| `crates/ui-egui/src/workflows/combine_ui/parts.rs` | 1 | Browser-safe relative-date clock (moved from deleted combine_ui.rs) |
| `docs/brand/artcraft-logo-white.svg` | 1 | Empty 1x1 include placeholder required by the fork trademark exclusion |
| `docs/brand/artcraft-logo.svg` | 1 | Empty 1x1 include placeholder required by the fork trademark exclusion |
| `docs/brand/artcraft-mark.svg` | 1 | Empty 1x1 include placeholder required by the fork trademark exclusion |

Total: 63

Ratchet: 65

Measured using `git diff --numstat --diff-filter=M 5af102765222 HEAD -- . ':!c0'`.
Whole-file trademark/CI removals add no lines. The three one-line blank SVG
include placeholders are included in the total after the separate hygiene commit.

Deleted upstream blocks in `home.rs` and `dialogs.rs` live in `c0_host/community.rs`;
default rendering includes upstream's new close button. The browser clock and MIME
corrections remain in `c0_web.rs` / `c0_clock.rs` and are suitable upstream as is.
All other entries are fork-specific host policy/API and have not been offered upstream.

## Retired paths and previously missing conflicts

- `crates/ui-egui/src/combine_ui.rs`: deleted upstream; clock hook moved to
  `workflows/combine_ui/parts.rs`. Never resurrected.
- `crates/ui-egui/src/docking.rs`: deleted upstream; visibility/folding hooks moved to
  `panels/side_panels.rs`. Upstream's flat PaneHost list has no split paths to translate;
  the obsolete fork split-path helper is removed, with retained-geometry/fold tests ported
  to the new saved side-panel state. No host switch was dropped.
- `crates/ui-egui/src/editing.rs`: download implementation moved to `editing/save.rs`.
- `.github/workflows/ci.yml`, `freebsd.yml`, `packaging-lint.yml`, `release.yml`,
  `windows-arm64.yml`: all five modify/delete conflicts were missing from the old table.
  These are whole-file fork exclusions (zero added hook lines). Upstream versions were
  accepted for the merge, then removed in the required separate hygiene commit.

The moved control trait needs a default error body for upstream's independent fake hosts:
this raises its portion from 5 to 7 lines. Savings from the new pane renderer and menu
builder keep the overall hook debt below the old ratchet; no exception is requested.
