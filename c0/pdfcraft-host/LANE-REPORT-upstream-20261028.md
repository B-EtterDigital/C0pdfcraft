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


## Follow-up: merged-candidate package test repair (2026-10-28)

Starting candidate: `183a04aeb680b72b6e496aabae2f16474c1201b8`, branch `fix`.
This section records the later repair run; the blocked qualification above remains
historical evidence. No push is authorized or performed.

### Initial reproduction and ownership

The requested `cargo test -p pdfcraft-ui-egui --no-fail-fast` exited 101 before
executing tests. Full output: `/srv/smoac/build/sow-up1028/pdfcraft-tests.txt`.
There were three E0063 compile failures in the fork-owned `c0_host_contract` suite:

- `bytes_round_trip_preserves_dirty_revision_history_and_other_tabs`
- `navigation_is_zero_based_validated_and_outline_aware`
- `replacement_uses_existing_save_changes_cancel_discard_and_save`

Cause: upstream added optional `left` and `top` destination coordinates to
`pdfcraft_engine::Edit::AddBookmark`; the three fork fixtures still used the old
initializer. All three are ours, not upstream-red. The fix supplies `left: None`
and `top: None`, matching upstream's page-only bookmark fixtures and retaining the
existing navigation, undo/redo and replacement assertions. Product behavior is
unchanged. Cargo also resolved the web crate's existing `serde_json` dependency
into `Cargo.lock`; that single generated bookkeeping line is retained.

All Cargo commands use `CARGO_BUILD_JOBS=4`,
`CARGO_TARGET_DIR=/srv/smoac/build/sow-fix-pdfcraft-target`, and
`smoac-slot run gate 3 3600 --`. No cache was deleted. The initial target directory
was absent, so the first run compiled dependencies from scratch. The host owns the
actual gate capacity (reported as 4); its limits and holds were not modified.

| Command / phase | Load before (1/5/15 min) | Load after | Result |
|---|---|---|---|
| Initial UI reproduction, 13:57–14:08 UTC | 11.03 / 11.82 / 13.28 | 8.48 / 14.16 / 14.62 | Exit 101; three fork fixture compile errors, zero tests executed; gate waited 125 s |
| Contract suite `--test c0_host_contract --no-run`, 14:08–14:22 UTC | 10.02 / 14.30 / 14.66 | 13.79 / 14.41 / 14.94 | Passed; gate waited 804 s; compile finished in 4.44 s |

The compiling fixture/lockfile unit was immediately committed as
`c63f8e02b3a4a53355b37d1843b26c6870fa3b39` with the requested identity.
Direct `rustfmt --check` and `git diff --check` passed for that change.
The full UI rerun was initially queued ordinarily, then requeued using the slot
manager's supported `SMOAC_SLOT_KIND=proof SMOAC_SLOT_REQUIRED=1` mode. Both the UI
proof and a later web-check proof remained blocked behind active judging work;
these requests were cancelled before Cargo started. Once load fell below 9,
all remaining commands were combined in one ordinary gate session, which was
admitted after 592 seconds. No host policy was changed. The cancelled queue logs
are retained as `pdfcraft-tests-after-queue.txt` and
`pdfcraft-final-checks-queue.txt` beside the original reproduction log.
Neither admission kind bypassed host holds or load guards.


### Runtime failure and second fix

The first executable full UI run (`pdfcraft-tests-after.txt`) had **1,783 passed,
1 failed, 11 ignored**. The complete runtime failure list was:

- `c0_host_contract::community_switch_removes_home_about_menu_palette_and_command_links`

Cause: the fork test still opened the old hamburger `Menu` and hovered `Help ⏵`.
Upstream now renders a craft-menu bar; at this harness width the accessible
menu title is `Help`. The test failed with “No nodes found” for `Menu`, before
checking the menu contents. This is ours, not upstream-red.

Fix: click `Help` directly and assert `About PdfCraft` is visible before checking
that community links are absent. The positive assertion proves the menu actually
opened, preserving meaningful coverage of the existing host filter. All Home,
About, palette and direct-command refusal assertions remain. No production hook
or upstream default behavior was changed. The full contract suite then passed
**9/9** and the fix was immediately committed as
`7cac08334a67f6b64acd0154f3fb7f4f8285e8f4` with the requested identity.

### Final verification

All commands below ran in the same admitted ordinary gate session, using the
four-job limit and shared target directory stated above. Each command's full
output, exit code, UTC timestamps, load and disk snapshots are in
`/srv/smoac/build/sow-up1028/pdfcraft-<label>.{txt,exit,before,after}`.

| Command | Result / pass count | Log label |
|---|---|---|
| `cargo test -p pdfcraft-ui-egui --no-fail-fast` (before menu fix) | Exit 101; 1,783 passed, 1 failed, 11 ignored | `tests-after` |
| `cargo test -p pdfcraft-web` | Exit 0; 0 tests (default native binary) | `web-tests` |
| `cargo check --target wasm32-unknown-unknown -p pdfcraft-web` | Exit 0 | `web-wasm` |
| `cargo test -p pdfcraft-web --features page-control` | Exit 0; 2 passed | `web-page-control-tests` |
| `cargo check --target wasm32-unknown-unknown -p pdfcraft-web --features page-control` | Exit 0 | `web-page-control-wasm` |
| `cargo fmt --all --check` | Exit 0 | `fmt` |
| `cargo clippy -p pdfcraft-ui-egui -p pdfcraft-web --all-targets --features page-control -- -D warnings` | Exit 0 | `clippy` |
| `cargo xtask layers` | Exit 0; 37 crates, no violations | `layers` |
| `cargo xtask assets` | Exit 1; eight stale trademark inventory entries, detailed below | `assets` |
| `cargo test -p pdfcraft-ui-egui --test c0_host_contract` | Exit 0; 9 passed | `contract-tests` |
| `cargo test -p pdfcraft-ui-egui --no-fail-fast` (final) | **Exit 0; 1,784 passed, 0 failed, 11 ignored** | `tests-final` |

Final UI counts by target: library 577 passed / 1 ignored; `c0_host_contract`
9 passed; `c0_tool_modes` 10 passed; `c0_web_fixes` 2 passed; upstream `it`
1,183 passed / 10 ignored; performance 3 passed; doc-tests 0. The ignored tests
retain upstream's ignore annotations. Wasm checks passed with existing
conditional-compilation warnings (render: 4, UI: 6); no wasm warning-clean claim
is made. Native scoped clippy passed with warnings denied.

**Upstream-red test list: empty.** No upstream test failed in either executable
full package run, so no pure-upstream worktree comparison was necessary. All
observed compile/runtime failures belonged to the fork contract suite and were
fixed. No test was disabled, ignored or removed by this repair.

### Load and disk evidence for the admitted follow-up session

Loads below are the 1/5/15-minute averages immediately before/after each command.
The exact UTC intervals and disk snapshots are retained in the evidence files.

| Log label | UTC interval | Load before | Load after |
|---|---|---|---|
| `tests-after` | 15:06:38–15:07:12 | 12.83 / 13.41 / 15.15 | 16.09 / 14.15 / 15.33 |
| `web-tests` | 15:07:12–15:08:02 | 16.09 / 14.15 / 15.33 | 18.07 / 15.00 / 15.56 |
| `web-wasm` | 15:08:02–15:09:13 | 18.07 / 15.00 / 15.56 | 16.26 / 15.27 / 15.62 |
| `web-page-control-tests` | 15:09:14–15:11:27 | 16.26 / 15.27 / 15.62 | 18.24 / 16.26 / 15.92 |
| `web-page-control-wasm` | 15:11:27–15:11:33 | 18.24 / 16.26 / 15.92 | 17.50 / 16.14 / 15.88 |
| `fmt` | 15:11:33–15:11:37 | 17.50 / 16.14 / 15.88 | 16.90 / 16.03 / 15.85 |
| `clippy` | 15:11:37–15:12:45 | 16.90 / 16.03 / 15.85 | 15.04 / 15.71 / 15.75 |
| `layers` | 15:12:45–15:13:12 | 15.04 / 15.71 / 15.75 | 14.13 / 15.46 / 15.67 |
| `assets` | 15:13:12–15:13:12 | 14.13 / 15.46 / 15.67 | 14.13 / 15.46 / 15.67 |
| `contract-tests` | 15:13:12–15:13:15 | 14.13 / 15.46 / 15.67 | 14.13 / 15.46 / 15.67 |
| `tests-final` | 15:13:15–15:13:46 | 14.13 / 15.46 / 15.67 | 12.88 / 15.06 / 15.53 |

Disk usage during these admitted commands stayed at 83–84%; the 88% stop
threshold was not crossed. During the preceding queue waits it ranged from
81–86%. No cache or other worker's files were deleted by this lane.

### Remaining limitation and handoff

The extra asset audit confirms the limitation already documented before this
repair: five removed upstream trademark assets are still listed in
`ATTRIBUTION.toml`, and the three blank SVG placeholders do not match the old
trademark hashes. The five missing files are `artcraft-logo-white.png`,
`artcraft-logo.png`, `artcraft-mark-black.png`, `artcraft-mark-black.svg` and
`artcraft-mark.png`, all under `docs/brand/`; the three mismatched SVGs are
`artcraft-logo-white.svg`, `artcraft-logo.svg` and `artcraft-mark.svg` in that
directory. This is an existing fork inventory issue, not an upstream-red test.
No assets were added or restored, and asset metadata was not changed by this
package-test repair. The combined diagnostic script exits 1 because it retains
both the earlier failed UI attempt and this asset audit; individual final test
and wasm exit records above are 0.

Patch-debt total after Cargo's one generated lockfile line is **64**, below the
65-line ratchet; both Rust repairs are confined to a fork-owned test file.
This follows the SMA module-local ownership rule and preserves the small
upstream hook surface. No engine, UI hook, dependency version, ignored-test list,
CI, branding asset or release artifact changed. No push was performed.
The final report commit contains documentation only; the final full package
run tested the code committed at `7cac08334a67f6b64acd0154f3fb7f4f8285e8f4`.
