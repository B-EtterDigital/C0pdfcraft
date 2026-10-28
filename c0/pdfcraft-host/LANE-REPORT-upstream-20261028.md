# PdfCraft upstream pull — 2026-10-28

Branch: `fix`. Fork baseline: `186bab60e8bd76b97148f61d0a818e5680a9f3a3`
(`origin/c0/main`). Pinned upstream: `5af102765222c8220df0f9eaa27bfa3bcf481ead`.
Merge, no rebase; all fork commits retain their SHAs. No push or GitHub login.
The merge brings 212 first-parent commits (1,855 reachable commits including merged branches).

## Conflict resolutions

| Conflicted file | Resolution |
|---|---|
| `apps/pdfcraft-web/Cargo.toml` | Re-applied the opt-in page-control feature and JSON dependency; kept upstream's new font features. |
| `crates/ui-egui/src/canvas.rs` | Kept upstream; re-applied the single quick-bar wrapper in `document_area`; moved the column hook to `canvas/bars.rs::quick_bar`. |
| `crates/ui-egui/src/combine_ui.rs` | Deleted as upstream requires; moved the browser-safe clock hook to `workflows/combine_ui/parts.rs::ago`. |
| `crates/ui-egui/src/commands.rs` | Re-applied the command refusal gate; moved menu filtering to the new `menus.rs` craft-menu builder. |
| `crates/ui-egui/src/control.rs` | Moved forwarding into `control/host.rs` and dispatch into `control/requests.rs`; kept the extension trait method, with a default error for upstream test hosts. |
| `crates/ui-egui/src/docking.rs` | Deleted as upstream requires; ported Tools hiding, right-pane folding and rail gating to `panels/side_panels.rs`. Host-owned visibility policy remains in `c0_host.rs`. |
| `crates/ui-egui/src/editing.rs` | Kept upstream; moved the browser MIME hook to `editing/save.rs::download`. |
| `crates/ui-egui/src/home.rs` | Re-applied the early Home gate; moved the updated community card into `c0_host/community.rs`, preserving upstream's dismiss button and per-run state. |
| `crates/ui-egui/src/lib.rs` | Kept upstream's split structure; retained module registrations/Chrome field. Moved session, polling and top-bar hooks into `app.rs`, and picker hook into `app/files.rs`. |
| `crates/ui-egui/src/theme.rs` | Re-applied token resolution through the host override; retained upstream's RTL and egui ID changes. |
| `.github/workflows/ci.yml` | Accepted upstream for the merge, then removed in the separate hygiene commit. |
| `.github/workflows/freebsd.yml` | Same. |
| `.github/workflows/packaging-lint.yml` | Same. |
| `.github/workflows/release.yml` | Same. |
| `.github/workflows/windows-arm64.yml` | Same. |

The CI deletion conflicts were absent from the old debt inventory; it now records them as
whole-file exclusions with zero added hook lines. Neither deleted Rust file was resurrected.
No host switch or browser correction was dropped. The obsolete host split-path translator was
removed because upstream has no docking split tree: PaneHost renders a flat pane list.
The saved-state/fold regression now checks `SidePanels::to_json` rather than the removed
`Workspace`; widths and open states must survive host folding. All final hook hunks were read
at their actual destination, including the auto-merged chrome, dialogs and palette hooks.
The moved `finish_open` method now needs crate visibility for prevalidated host replacements;
that one-line visibility change follows upstream style. Fork-owned code/tests follow the new egui `Id::unique` and `Role` API.

`Cargo.lock` started from upstream's lock; Cargo owns subsequent dependency resolution.
It was never hand-edited.

## Patch debt

Baseline: **65 added hook lines**. Resolved merge before Cargo resolution: **60 added hook lines**; ratchet remains 65.
See `PATCH-DEBT.md` for the per-file recount. The control extension grew from five to seven
lines because upstream now has independent fake hosts; smaller pane/menu integration pays for
this without growing the total. The three blank SVG lines bring the current full added-line total to **63**; Cargo may add
one dependency-bookkeeping line, for **64**, still below 65.

## Verification

Pending. Commands run on this fleet host, with `CARGO_BUILD_JOBS=4`, the shared
`/srv/smoac/build/sow-fix-pdfcraft-target`, and `smoac-slot run gate 3 3600 -- ...`.
Disk is checked before each build and any usage above 88% stops the lane.
The first check queued at 12:50 UTC behind host landing mode, announced until 13:06 UTC.
It was requeued as a required proof at 13:08 UTC. Landing mode was renewed repeatedly;
at 13:26 UTC its current hold extended to approximately 13:45 UTC. Required proof priority
does not bypass an active landing hold. The merge is checkpointed without compilation
because this external admission block prevents the requested first green-unit checkpoint.

## New upstream commands and menu entries

Registry additions (no removed registry IDs):

| ID | Meaning |
|---|---|
| `edit.attach` | Edit → Attach a file… |
| `form.add.text_table` | Prepare a form → Text field table |
| `form.calculation_order` | Prepare a form → Calculation order |
| `view.page_gaps` | Show gaps between pages |
| `view.side_panel` | Show/hide side panel, F4 |
| `view.tools_pane` | Show/hide Tools panel, Shift+F4 |

`ui.dock.reset` keeps its ID but now means Reset side panels. The new menu builder reconstructs
File/Edit/View/Sign/Window/Help menus, a live Window document list, Create/Export/Protect,
Edit → Pages, Zoom/Rotate/Page navigation/Page display/Side panels/Display theme/Tools,
clipboard menu actions, and native macOS menus. Page-thumbnail context menus gain Split document.

Automation tool additions (no removed tool IDs): `attachment_add`, `attachment_delete`,
`bookmark_set_style`, `content_copy`, `form_add_field_table`, `form_calculation_order`,
`layer_set`, `standards_remove_id`. `ui.panel` replaces the old `ui.dock` layout protocol;
`ui.dock.reset` remains. Other existing tools gain parameters, including bookmark destination
coordinates, field renaming with formula updates, and rendering without an edited paragraph.

## What upstream added

- Features: text-field tables as one undo step; calculation-order inspection/repair; field
  renames update referencing calculations; expanded All tools view; optional page gaps;
  richer bookmarks, layer control and attachments; improved inline text/image editing,
  XFA forms, signature appearances, localization/RTL, and Office exports.
- Performance: PNG export uses opaque RGB and faster deflate; page-image export runs on
  workers; object-stream indices and CFF glyph-name lookups are cached; field appearances
  are reused; renderer debug settings are read once instead of per glyph.
- Crash/freeze fixes: bounded xref/JBIG2/graphics-state inputs; CUPS printer properties load
  off the UI thread with a deadline; stale page drags and asynchronous results are cancelled;
  undo/redo refresh is guarded and rolls back on failure (`895ccb90`, #3925).
- Save/undo fixes: new text boxes count as unsaved before focus leaves; background save prompts
  request attention without replacing a question already seen; saves preserve Windows
  hidden/system attributes and macOS extended attributes; password-field/XFA save handling
  improves; rejected flatten/mark edits leave the original page untouched.
- Packaging: per-user Windows installation by default, scope/architecture checks, and verified
  ONNX OCR model pairs. These are upstream changes, not qualification of installers here.

## Interaction with the fork UI

Home, community links, menus/palette, top bars, right rail, Tools pane, right-pane folding,
quick-toolbar modes, theme overrides, browser picker and page bridge all retain their switches.
Standalone defaults retain upstream behaviour, including the new dismissible community card.
The opt-in page transport still hides Home/title/tab/community chrome on attach.
Upstream side panes no longer support the old floating/tabbed docking geometry; upstream's
migration reads saved docking JSON into side widths and placement. The host gate does not
change that saved state. Native macOS menus are generated by the same filtered menu model.
The existing loopback/token control transport remains opt-in; c0 methods reuse its queue and
startup admission checks. No agent server is started by this merge.

## Assumptions I could not confirm

Pending final qualification notes.
