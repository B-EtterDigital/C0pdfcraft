# C0pdf: PdfCraft as a C0VIBE guest page

This page builds the existing `apps/pdfcraft-web` binary with the opt-in `page-control`
feature. It uses PdfCraft 0.10.1's engine, tools, egui interface and Glow renderer. It starts
without a demo, sample or blank document. The page and canvas are transparent until painting.

## Build

From this directory on the executor:

```sh
smoac-slot run heavy 2 3600 -- env CARGO_BUILD_JOBS=2 BINARYEN_CORES=2 NO_COLOR=true trunk build --release
```

Trunk writes `../../dist/c0host/`, hashes artifact names, and uses public URL `./`. Serve the
directory over HTTP(S); relative paths allow relocation. No separate application binary is
maintained. The ordinary web build continues to write `dist/web` and exposes no page function
without this feature. If `CRAFT_FONTS_DIR` names a missing checkout, unset it before building.
Without craft-fonts the embedded Japanese fallback is absent; Japanese UI glyphs may be boxes,
and PDF text that needs those faces returns the engine's missing-font error. No fonts are fetched.

The existing `?file=`, `?author=`, `?host=parent`, settings storage and beforeunload guard stay in
place. The same-origin parent transport is separate and unchanged. Omit `?file=` for an empty
guest. Host chrome policy is applied after restoring settings, and is never persisted.

## Page function and envelopes

Wait until `typeof window.__c0hostRequest === "function"`, then call it with JSON **text**.
The property aliases `window.__pdfcraftControl`, with the same getter/readiness semantics as
the GridCraft page. It returns a Promise of JSON text:

```js
const reply = JSON.parse(await window.__c0hostRequest('{"method":"c0.inspect","params":{}}'));
```

Requests use `{method, params?}`. Missing or null params mean `{}`; other non-object params
are rejected. Optional ids are ignored: each Promise pairs its request and reply.
Success is `{"ok":true,"result":...}`; failure is `{"ok":false,"error":"..."}`. Unknown methods,
bad JSON, invalid arguments and oversized requests use that same failure envelope.

Ordinary requests are limited to 1 MiB of UTF-8 JSON; `c0.*` requests to 100 MiB. Files and
exports are limited to 72 MiB before base64 encoding. Names must be nonempty basenames of at
most 255 bytes, without `/`, `\`, NUL, `.` or `..`. Bytes use standard padded base64.
The page caps `ui.drag.steps` at 32, in addition to upstream argument validation.

## When replies arrive

The bridge attaches through PdfCraft's public `attach_control` API and sends requests through
the same queue as native control. Each call lends a logic/layout pass from the actual WebRunner
through its event-listener mechanism. Immediate requests answer without presenting a frame,
including when animation frames are throttled. Synthetic input gets at most 40 extra inline
passes; pending replies are polled with another unpainted pass every 16 ms. Browser/host
callers own their transport deadlines. `ui.screenshot` still needs actual presented pixels
and retains the upstream five-second timeout. No server or network listener is installed.

## Host methods

| Method | Params | Success result |
|---|---|---|
| `c0.inspect` | `{}` | Document and interface state below; succeeds when empty |
| `c0.goTo` | `{page}` or `{outline}` | `{page}`; both indices are zero-based |
| `c0.documentBytes` | `{format?: "pdf"}` | `{name,format:"pdf",bytes,revision}` |
| `c0.openFile` | `{name,bytes}` | `{opened:true,pending:false}` or `{pending:true,needsConfirmation:true}` |
| `c0.newDocument` | `{template?: "blank"}`; empty string also accepted | Same replacement result; creates upstream's one-page US Letter PDF |
| `c0.selection` | `{}` | `{kind:"text",page,pageCount,selectedText?,pageText,documentTitle}`; `pageText` is at most 20000 characters |
| `c0.place` | `{note,page?,title?}` | `{ok:true,placed:"note",page,id,index,text}`; adds a sticky-note comment at the page top-left as one undo step |
| `c0.cancelFileDialog` | `{}` | `{cancelled:boolean}` |
| `c0.set` | `{themeTokens?,panels?}` | `{applied:string[],unknown:string[],invalid:string[]}` |

`c0.inspect` exposes:

```json
{
  "name":"example.pdf", "revision":0, "dirty":false, "canUndo":false, "canRedo":false,
  "pageCount":2, "currentPage":0, "zoom":1.0, "theme":"light",
  "outline":[{"level":0,"title":"Chapter","page":1}],
  "annotationCount":0, "formFieldCount":0,
  "documents":[{"name":"example.pdf","dirty":false,"active":true}],
  "panes":{"chrome":{"title_bar":false,"app_bar":true,"tab_strip":false,"start_screen":false,
    "community_links":false,"right_rail":true,"tools_panel":true,"toolbar":true,"toolbar_double":false,
    "toolbar_floating":false,"right_dock_collapsed":false,"toolbar_pos":null},
    "strip":{"visible":true,"columns":1,"floating":false,"pos":null},
    "tools":true,"left":"AllTools","right":null},
  "pendingReplacement":false, "fileDialog":false, "needsConfirmation":false
}
```

Illustrative values depend on the document and upstream view defaults. Empty state has null
`name`/`currentPage`, zero counts/revision, zoom 1, false dirty/history flags and empty arrays.
`revision` is the engine's active-document edit generation, not a global counter. Dirty includes
uncommitted typing. `theme` is the resolved palette; the saved preference remains available in
`ui.state`. Outline is flattened in document order, with zero-based nesting levels and pages;
non-navigable bookmarks have null pages. The host bounds it to 10,000 entries and depth 64.
Annotation and field counts come from the engine inspection, without calling the MCP crate.

`goTo` validates the destination before committing pending edits and navigating through the
existing document view. It rejects missing/out-of-range pages, non-navigable bookmarks, two
destinations at once and navigation while a modal is open.

`documentBytes` calls the engine's save-byte encoder without marking saved, changing history,
committing draft typing, downloading or opening a dialog. It exports the current committed PDF,
not rendered display-only appearances. It fails when no document is active or the format is not
PDF. The normal app logic pass retains upstream's ordinary pending-edit processing.

Open/New replace **only the active document**; other tabs survive. Incoming Open bytes are
prepared by the engine before any close request. PDF and the engine's existing supported image
and text conversions are accepted; malformed/encrypted imports fail before prompting or closing
anything. Password-protected PDFs remain accessible through normal Open. A dirty active document
uses the existing Save Changes prompt. Cancel retains it; Don't save or a completed Save allows
replacement on the next logic pass. A second replacement while pending is refused. A failed or
cancelled save retains the current document. Poll inspect to distinguish acceptance from completion.
Replacement errors use the existing notice, with no host-specific dialog.

The feature's browser Open chooser uses the normal importer/inbox, with a 72 MiB size cap and
a cancellation flag. `cancelFileDialog` retires this chooser request; a browser-owned chooser
may remain onscreen but its late result is ignored. It does not answer Save Changes, discard
work, cancel Save or intercept other feature-specific pickers. A validated Open/New also retires
the current Open chooser. When no such request is live it returns false.

## Chrome policy

The unsaved `Chrome` struct defaults **every field to true**. The guest page turns off
`title_bar`, `tab_strip`, `start_screen` and `community_links`; the host owns its title and Home.
All panel values in `c0.set` are booleans; the four switches below the table default to
upstream's behaviour (strip shown, one column, fixed in its corner, dock open). Successful keys are reported as `panels.<key>`,
unknown keys as unknown and known non-booleans as invalid. Repeated values are idempotent.

| Panel key | Effect |
|---|---|
| `title_bar` | Gates the app's title/tab strip together with `tab_strip` |
| `tab_strip` | Document tabs and Home button |
| `app_bar` | Mode/action bar; keep enabled to retain the guest's tool menus |
| `start_screen` | PdfCraft Home contents; false leaves an empty pasteboard |
| `community_links` | Community card, Discord chrome action, Help links, palette promotions and About marks/links; About retains PdfCraft, contributors and licence |
| `right_rail` | Page navigation/tool rail; panes remain available through commands |
| `tools_panel` | Hides Tools in the rendered dock projection, retaining the saved layout |
| `toolbar` | The quick-action strip (select, hand, comment, fill and sign); default true |
| `toolbar_double` | The strip in two columns; default false |
| `toolbar_floating` | The strip as a palette with a grip, dragged within the document area; default false |
| `right_dock_collapsed` | The right dock folds to the rail; default false |

## Tool strip modes

PdfCraft's tool strip is the quick-action bar (`canvas.rs`, `quick_bar`): a one-column strip over
the left edge of the document area, with flyout menus for the comment, highlight, draw and
Fill & Sign tools. The tools panel (`tools_panel`) is a catalogue of tool groups, not a strip, so
it keeps its own switch. The three strip keys use PhotoCraft's names and are not saved:

| `toolbar` | `toolbar_double` | `toolbar_floating` | Result |
|---|---|---|---|
| true | false | false | Upstream: one column, fixed 14 points from the document area's corner |
| true | true | false | Two columns, fixed in the corner |
| true | false | true | One column with a grip above it; drag the grip to move the strip, which stays inside the document area |
| true | true | true | Two columns, movable |
| false | any | any | No strip (Read mode never shows it, as upstream) |

Switching back to fixed returns the strip to its corner and keeps the dragged place for the next
float. `c0.inspect().panes.strip` reports `{visible, columns, floating, pos}` as drawn; `pos` is
the strip's offset in points from its corner position, `null` until dragged. Two columns wrap
at half the height of the buttons and options the current tool shows.

## Right dock fold

`right_dock_collapsed: true` drops the Inspector from the rendered dock (the saved layout, its
group and width are untouched) and from Read mode's side panel; the rail's icons stay. Pressing a
rail icon opens that pane and unfolds the dock, including the icon of the pane that was open.
The change is immediate; the document area takes the width the dock had. With `right_rail`
false nothing remains of the dock while folded.

Other navigation/inspector panes and the existing `ui.set` options remain available. Signing,
OCR and attachments remain visible with their upstream browser limitations: certificate signing
has no web save destination, OCR requires model files, and attachment downloads are not implemented.
No feature was cut to reduce artifact size.

## Runtime theme tokens

`themeTokens` replaces the context-local override set. `{}` or null clears it; omission retains
it. Values are `#rrggbb` or `#rrggbbaa` (case-insensitive, unpremultiplied alpha). Valid entries apply
alongside invalid/unknown siblings. Keys in the reply are prefixed `themeTokens.`. Malformed
top-level containers fail before any palette or panel change. Unknown top-level keys are reported.
The override survives light/dark/system changes and also updates egui widget visuals.

All 20 `Color32` fields of PdfCraft's `Tokens` are supported:

`titlebar`, `chrome`, `panel`, `pasteboard`, `card`, `border`, `divider`, `text`, `text_muted`,
`text_faint`, `icon`, `hover`, `pressed`, `selected`, `accent`, `accent_text`, `accent_soft`,
`field`, `badge_new`, `page_shadow`.

`kind` and `radius` are not colours and are unknown token keys. Use `ui.set` with
`{"key":"theme","value":"light"}` (or dark/system) to change the ordinary theme preference.
Existing hard-coded tool/document colours outside Tokens remain unchanged.

## Ordinary control methods

All existing methods keep their upstream request/reply semantics inside the page envelope:

| Method | Params / purpose |
|---|---|
| `ui.state` | `{}`; documents, active page/zoom, panels, theme, dialogs, notices, window |
| `ui.inspect` | `{query?,role?,focused?,limit?}`; AccessKit widget tree |
| `ui.command` | `{id}`; run an enabled engine/UI command |
| `ui.commands` | `{}`; registered commands and enablement |
| `ui.set` | `{key,value}`; existing view/preferences options |
| `ui.click` | `{id}`, `{label}` or `{x,y,button?}` |
| `ui.focus` | `{id}` or `{label}` |
| `ui.move` | `{x,y}` |
| `ui.drag` | `{from:[x,y],to:[x,y],steps?,modifiers?,button?}` |
| `ui.type` | `{text}` |
| `ui.key` | `{key,modifiers?}` |
| `ui.dock`, `ui.dock.reset` | Existing panel layout controls |
| `ui.screenshot` | `{region?}`; base64 PNG, requires presentation |
| `ui.render` | `{page?,scale?}`; frameless: `{page,pageCount,scale,width,height,png_base64}`, the page alone from the engine's renderer. `page` is zero-based (default: the current page); `scale` 0.25 to 16 (default 1, one pixel per point). Answers without presenting a frame; changes nothing |
| `ui.open`, `ui.resize`, `ui.quit` | Native-only; return actionable errors on the web |

## Validation and maintenance

Only `c0/main` receives commits; `main` stays the upstream mirror. Never push from this lane.
See [PATCH-DEBT.md](PATCH-DEBT.md) for every upstream edit and the measured added-line ratchet,
and [LANE-REPORT.md](LANE-REPORT.md) for test counts, Chromium evidence, sizes and assumptions.
Use the slot wrapper and at most two Cargo jobs for every heavy command. Native feature tests
exercise the parser/envelopes (K-41); wasm check and Trunk compile the browser half. UI tests use
synthetic engine-generated documents, without adding any binary fixture or asset.

From the repository root:

```sh
smoac-slot run heavy 2 3600 -- env CARGO_BUILD_JOBS=2 bash c0/pdfcraft-host/native-checks.sh
smoac-slot run heavy 2 3600 -- env CARGO_BUILD_JOBS=2 bash c0/pdfcraft-host/final-page.sh
```

`native-checks.sh` runs format, native feature tests, the full UI/engine suites (including the
host tests), and native clippy with development/test optimization and debug info disabled: the upstream
default UI build exceeded this executor's one-hour per-command limit. Dependency-specific
profile settings stay upstream's; release artifacts use the ordinary release profile. Both
scripts run sequentially and cap Cargo jobs at two. Native gates use one test thread:
two upstream form tests share a process-specific temporary path. They save logs in
`/tmp/c0pdf-lane` (override with `C0PDF_LOG_DIR`). `node c0/pdfcraft-host/smoke.mjs` tests an
existing `dist/c0host` using system Chromium, with no Node package installation required.
`final-page.sh` runs format, wasm check, layers/assets, clock-probe build/clippy, the ordinary
release page build and the complete Chromium smoke including the clock regression.

The test-only clock probe isolates the exact old unsupported `SystemTime::now()` call from
the app, catches its wasm trap, and checks the actual replacement helper against browser UTC.
It shares the helper source, has its own small Cargo workspace, and never enters `dist/c0host`:

```sh
smoac-slot run heavy 2 3600 -- bash c0/pdfcraft-host/clock-probe/build.sh
smoac-slot run heavy 2 3600 -- env C0PDF_CLOCK_PROBE_DIR=/tmp/c0pdf-lane/clock-probe node c0/pdfcraft-host/smoke.mjs
```
