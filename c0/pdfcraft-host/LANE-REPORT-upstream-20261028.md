# PdfCraft upstream pull — 2026-10-28

**Status: LANE-BLOCKED — host landing mode prevented all Cargo admission.**

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
this without growing the total. The three blank SVG lines bring the final current added-line total to **63**; Cargo may add
one dependency-bookkeeping line, for **64**, still below 65. `Cargo.lock` currently matches
the pinned upstream exactly. The debt inventory includes all three blank placeholders.

## Verification

No Cargo process was admitted; **zero tests ran**. This is an unvalidated merge checkpoint,
not a green build. The controller must complete the commands below before LANE-DONE.
Both the ordinary and required-proof attempts used `smoac-slot run gate 3 3600 -- ...`,
`CARGO_BUILD_JOBS=4` and `CARGO_TARGET_DIR=/srv/smoac/build/sow-fix-pdfcraft-target`.
The required proof additionally used `SMOAC_SLOT_KIND=proof SMOAC_SLOT_REQUIRED=1`.
Neither attempt reached Cargo; both were cancelled while still queued, at handback.

| Attempt | UTC interval | Load before (1/5/15 min) | Load after | Outcome |
|---|---|---|---|---|
| Native check, ordinary gate | 12:50–13:08 | 6.54 / 9.75 / 14.47 | 19.42 / 16.60 / 15.11 | Admission held; requeued as required proof |
| Native check, required proof | 13:08–13:28 | 19.42 / 16.60 / 15.11 | 14.36 / 14.36 / 15.51 | Admission held; cancelled while queued |

The command queued in both attempts was:

```sh
cargo check -p pdfcraft-ui-egui -p pdfcraft-web --features page-control
```

Disk was 81% before each attempt and 82% at handback; the 88% stop threshold was never
crossed. No build started, so there are no per-build or per-test load pairs to report.
The slot manager's first message announced landing mode until 13:06 UTC; subsequent
renewals announced 13:25 UTC. At 13:26 UTC, read-only inspection found the hold had been
renewed again. `/srv/smoac/landing-mode.json` at handback contained:

```json
{"by":"bdd-main","at":"2026-10-28T13:15:50.207Z","until":1793195150,"reason":"auto: 6 pending, nothing landed for 100 min","repo":"/home/bdd-main/DEV/Projects/002-C0X-v1"}
```

That hold expires at 13:45:50 UTC unless renewed. The slot implementation explicitly applies
landing mode to new `proof` claims too; required-proof priority does not bypass this hold.
No slot limits, holds, owners or host configuration were changed. No cargo/trunk command
ran outside the slot manager. To protect the tree from cleanup, the resolved merge was
checkpointed without compilation, then the mandatory separate hygiene commit was made.

Pending commands, each under the required gate wrapper above, with `df -h /` and host load
recorded before and after each; stop if disk usage exceeds 88%:

```sh
cargo test -p pdfcraft-ui-egui -p pdfcraft-web --features eframe/x11,eframe/wayland
cargo test -p pdfcraft-web --features page-control
cargo check -p pdfcraft-web --target wasm32-unknown-unknown --features page-control
cargo clippy -p pdfcraft-ui-egui -p pdfcraft-web --all-targets --features page-control -- -D warnings
cargo fmt --all --check
cargo xtask layers
```

Low-disk native settings prepared for these commands were `CARGO_PROFILE_DEV_DEBUG=0`,
`CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_PROFILE_DEV_OPT_LEVEL=0`,
`CARGO_PROFILE_TEST_OPT_LEVEL=0`, and `RUST_TEST_THREADS=1`.

| Gate | Pass count / result |
|---|---|
| `pdfcraft-ui-egui` tests, including the three fork integration suites | 0 executed; blocked before compilation |
| `pdfcraft-web` tests, including page-control feature tests | 0 executed; blocked before compilation |
| Crates under `c0/` | No crate with Rust tests: `clock-probe` contains two wasm probe exports and no `#[test]`; `c0_host` is a module covered by the UI crate tests |
| wasm web check | Not run; no release page built and no release wasm size measured; 41,943,040-byte cap unchanged |
| clippy | Not run; no warning-clean claim |
| `cargo fmt` / `cargo xtask layers` | Not run through Cargo |
| Direct rustfmt check of changed Rust files and fork host modules | Passed (no Cargo/build invocation) |
| `git diff --check` / unresolved conflict index scan | Passed |
| Fork and pinned-upstream ancestry | Both passed |

**Upstream-red tests:** none established because no tests ran. Any failing upstream test
must still be rerun with the same command in a pure `5af102765222...` worktree; the common
native command above deliberately avoids the fork-only feature so this comparison is possible.
Fork-only test failures must be fixed, not classified upstream-red.

**Asset inventory limitation:** the required removal of upstream trademarks leaves their
upstream attribution entries stale. The three contributor-original empty 1x1 SVGs and hashes
are documented in `BLANK-ASSETS.md`. `cargo xtask assets` was not run and is expected to
reject the stale upstream inventory; no claim of a passing asset gate is made. The upstream
manifest/tooling was left untouched to retain the binding patch-debt ratchet. This remains
additional work alongside the blocked compile/test qualification.

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

- Compilation, native UI tests, wasm type checking and warning cleanliness are unconfirmed
  because host landing mode denied admission. There is no passing runtime evidence yet.
- Windows/macOS installers, native macOS menu rendering, real browser transport behaviour,
  CJK fonts, and screenshot appearance were not qualified on this Linux host.
- The old split-path helper has no purpose in the new flat pane renderer by source inspection;
  the ported saved-state/fold tests still need to run before claiming runtime proof.
- The upstream attribution inventory needs fork reconciliation as described above. No release
  page was built, so the release wasm size is unknown.
- The current hold's expiry is observable, but whether the controller renews it cannot be
  confirmed from this repository. No controller question or GitHub login was attempted.

Merge checkpoint: `63881dee8521078c71dc40a2b0fb325bdff695c4`.
Trademark/CI cleanup: `12d9d69866c594d11ab398ba9035f8c7267c2ab3`.
All commits use `C0 upstream sync <c0-upstream-sync@betterdigital.invalid>` on `fix`.
The working tree is committed for controller retrieval over SSH; nothing was pushed.
