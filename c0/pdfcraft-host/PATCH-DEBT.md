# PdfCraft upstream patch debt

The pinned upstream is `d2916dec` (storytold/pdfcraft `main`, merged 2026-11-11; it was
`5af102765222c8220df0f9eaa27bfa3bcf481ead`). The pre-merge
`origin/c0/main` baseline was **65 added lines**. New fork-owned files do not count.
The table counts added lines, including replacements; deletions are described separately.
All host switches default to upstream behaviour. No engine file is modified.

| File | Added lines | Hook / reason |
|---|---:|---|
| `apps/pdfcraft-web/Cargo.toml` | 3 | Opt-in page-control feature, native backends for feature tests (upstream now carries the JSON dependency) |
| `apps/pdfcraft-web/src/main.rs` | 8 | Register runner and attach the opt-in bridge (beside upstream's `?control` page API, 2026-11-09) |
| `crates/ui-egui/src/app.rs` | 5 | Browser session clock, default Chrome, replacement polling and title/tab/app-bar gates (moved from lib.rs) |
| `crates/ui-egui/src/app/files.rs` | 4 | Opt-in browser picker hook plus crate visibility for finish_open after upstream moved it into a private submodule |
| `crates/ui-egui/src/canvas.rs` | 3 | Draw upstream's hotbar only when the host's toolbar switch is on (`c0_host::strip::sync`) |
| `crates/ui-egui/src/chrome.rs` | 5 | Gate the Discord button |
| `crates/ui-egui/src/commands.rs` | 3 | Refuse community commands when links are hidden |
| `crates/ui-egui/src/control.rs` | 7 | Default host-extension method for upstream test hosts; `Host::command` takes the request's params (`ui.command` arguments, 2) |
| `crates/ui-egui/src/control/host.rs` | 5 | Forward host requests to c0_host (moved from control.rs); `command` hands its params to `c0_host::command_args` (2) |
| `crates/ui-egui/src/control/requests.rs` | 2 | Dispatch c0.* and ui.render over the existing queue (moved from control.rs); `ui.command` forwards `params` (1) |
| `crates/ui-egui/src/dialogs.rs` | 6 | Gate About mark and delegate community footer; pass the app into upstream's new `dialog_about` (call site and signature, 2, forced by the 20261030 pull) |
| `crates/ui-egui/src/dialogs/palette.rs` | 3 | Filter community commands (moved from palette.rs) |
| `crates/ui-egui/src/editing/save.rs` | 1 | Browser download MIME: the one call site now goes through `c0_web::mime` (moved from editing.rs), which defers to upstream's `download_mime_type` table added in d2916dec and keeps only zip (2026-11-11 merge: upstream's line `download_mime_type(name)` was the same fix, so the call is ours and the table is theirs) |
| `crates/ui-egui/src/home.rs` | 4 | Gate Home and delegate the upstream dismissible community card |
| `crates/ui-egui/src/hotbar.rs` | 3 | Vertical width and the two vertical button lists wrap into a second column when the host asks (`c0_host::strip::{width,wrap}`); docked and floating use the hotbar's own dock |
| `crates/ui-egui/src/lib.rs` | 3 | Register host/browser modules and transient Chrome field |
| `crates/ui-egui/src/control/lifecycle_tests.rs` | 3 | Upstream's three fake hosts take the `ui.command` params (signature follows `Host::command`) |
| `crates/ui-egui/src/menus.rs` | 1 | Filter community menu rows in the new craft-menu builder (moved from commands.rs) |
| `crates/ui-egui/src/panels/side_panels.rs` | 2 | Project visible panes and gate rail without changing saved widths/open states (replaces docking.rs and Read-mode gate) |
| `crates/ui-egui/src/theme.rs` | 2 | Register override module and resolve tokens |
| `crates/ui-egui/src/workflows/combine_ui/parts.rs` | 1 | Browser-safe relative-date clock (moved from deleted combine_ui.rs) |
| `crates/ui-egui/src/widgets.rs` | 1 | Re-export fork-owned empty layout slots instead of embedding trademark placeholders |

Total: 75

The tool list and opening (`c0.tools`, `c0.openTool`, `inspect.tool`) live in the fork-owned
`crates/ui-egui/src/c0_host/tools.rs` and add no upstream lines; the total is unchanged.

Ratchet: 75 (2026-11-11: +10 for `ui.command` params, the minimum the upstream `Host::command` signature allows; 2026-11-09: upstream now carries serde_json; the quick-bar strip moved onto upstream's hotbar: canvas.rs 1 -> 3, hotbar.rs +3, bars.rs and the attribution/lock lines gone; 63 before the 20261030 pull; upstream moved the About tab into `dialog_about`, which had no app handle)

2026-11-11: `ui.command` forwards its `params` to `c0_host::command_args` (control.rs +4, control/host.rs +2, control/requests.rs +1, control/lifecycle_tests.rs +3 for the fake hosts), so the total is 75. The SOW owner raised the ratchet to 75 for it on 2026-11-11.

2026-11-11, merge of upstream `d2916dec` into `c0/args-20261111` (`c0/merge-20261111`): the four conflicted files kept every hook and moved none. Each row above is unchanged: the per-file added-line counts were measured before the merge against upstream `bfed7bba` and after it against `d2916dec`, and both give 75. The total and the Ratchet stay 75. The save.rs row's reason changed (see the table). The per-file counts come from `git diff --numstat <upstream> -- <file>` on the listed files, not from the whole-repository command quoted below, which reports every changed file in `vendor/` and `xtask/` as well.

Measured using `git diff --numstat --diff-filter=M c735fb7da552 HEAD -- . ':!c0'`.
Whole-file trademark/CI removals add no lines. The size lane replaces three blank
SVG placeholders with one widget re-export; the empty layout implementations live
in the fork-owned `c0_host/brand.rs`. It also deletes the obsolete trademark asset
records from `ATTRIBUTION.toml` and regenerates `ATTRIBUTION.md` (eight deleted
rows and one replaced count heading, counted above).
Page profile tuning lives entirely in C0-owned build tooling. The previous total
of 63 omitted the existing one-line `Cargo.lock` dependency entry; the corrected
starting total was 64, before this lane reduced it to 63.

Deleted upstream blocks in `home.rs` and `dialogs.rs` live in `c0_host/community.rs`;
default rendering includes upstream's new close button. The browser clock correction
remains in `c0_web.rs` / `c0_clock.rs` and is suitable upstream as is. The MIME correction
is upstream's since d2916dec (`download_mime_type`); `c0_web::mime` keeps only zip, which
that table does not list.
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
