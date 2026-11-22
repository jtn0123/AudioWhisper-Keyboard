# Stage 4: the licence screens

This page covers the two licence screens under About. Rust draws both. Compose stays the
fallback for the third-party list; the project licence's Compose screen was deleted at
compose-removal step 8.

| Route | Drawn by | Why |
| --- | --- | --- |
| `settings/about/third-party-licenses` | Rust | Ported here. |
| `settings/about/project-license` | Rust | Ported at compose-removal step 8, with read-only text selection (see [The project licence screen](#the-project-licence-screen)). |

`catalogue::native_routes()` includes both routes. Kotlin gets the native route list from
Rust, so nothing else had to change for them to route there.

## Data path

Kotlin only reads the file. Rust does the rest.

1. When the list is first drawn with no data, Rust sends a `["load", "libraries"]` plan. It
   asks only once until an answer comes back, or until the route is entered again.
2. `RustSettingsView` reads `R.raw.aboutlibraries` as UTF-8 off the main thread, the way
   `produceLibraries` does. It sends the text back with settings op 21 (pairs of name and text).
3. `licences/data.rs` parses the text the way AboutLibraries 15.3.0's `AndroidParser` does:
   - The licences are the keys of `licenses`. A licence must have a `name`; `content` is
     optional.
   - A library's licences are looked up by key, and unknown keys are dropped. They are then
     put in the order a Kotlin `toHashSet()` iterates them: a `HashSet(mapCapacity(n))` table,
     by bucket of the spread `String.hashCode`, then by insertion order.
   - The author is the developers' non-null names joined with ", ". With no developers it is
     the organisation's `optString("name")`, and otherwise "".
   - The name falls back to `uniqueId`. The version is `artifactVersion`, or none when it is
     absent or JSON null.
   - The libraries are sorted with `sortedBy { name.lowercase() }`: a stable sort in UTF-16
     order, not code-point order.
   - Any structural failure gives an empty list, like the parser's catch-all.
4. The list measures every row from line metrics and badge widths only, without shaping a
   name. It lays out and draws only the rows that intersect the viewport.

Op 3 gained two colours: field 56 is `onPrimaryContainer` (badge text) and field 57 is
`onBackground` (row and dialog text). `App.colors` now holds 16 colours.

## Row and dialog

The row is a Traditional row (`LibraryDefaultComponents`, `M3VariantTokens`):

| Part | Style |
| --- | --- |
| Padding | 20 dp horizontal, 14 dp vertical; children 4 dp apart |
| Name | titleSmall at 14 sp, Medium, 1.2 em lines, onBackground. One line with an end ellipsis, taking the width the version leaves. |
| Version | labelSmall at 11 sp, monospace, onSurfaceVariant. Shown when not blank, 8 dp after the name. |
| Author | bodySmall at 11 sp, 1.25 em lines, onSurfaceVariant. Shown when not blank; one line with an ellipsis. |
| Badges | A FlowRow 6 dp apart both ways. Each pill is labelSmall Medium in 10 dp by 3 dp padding, in primaryContainer and onPrimaryContainer. |

The scrollbar is `florisScrollbar` over a `LazyListState`:

- A row's share of the bar is `viewport / totalItems`.
- The bar is as tall as `visibleItems` rows, counted at the first laid-out frame and
  remembered after that.
- Its top is at the first visible row's index, plus how far that row is scrolled.

`LicenseDialog` uses `usePlatformDefaultWidth = false`:

- A Surface in shapes.medium (12 dp) and background, at most 560 dp wide, with a 16 dp by 8 dp
  margin.
- The licence text in 8 dp padding, at the app's bodyLarge.
- An `OK` TextButton aligned to the end, in primary, with 8 dp by 4 dp padding.

The text is laid out once per width and size, and only the lines on screen are drawn. Compose
now passes `licenseDialogConfirmText = stringRes(R.string.action__ok)`, so both renderers show
the app's string instead of AboutLibraries' literal "OK". They read the same in English.

### The dialog's text

The text is `AnnotatedString.fromHtml(htmlReadyLicenseContent)`. Compose prefixes
`<ContentHandlerReplacementTag />` and calls `Html.fromHtml(FROM_HTML_MODE_COMPACT)`, which
parses with TagSoup 1.2. `licences/scanner*.rs`, `tagsoup.rs` and `schema.rs` port TagSoup
1.2: its state table and its HTML schema. Both were dumped by reflection from the 1.2 jar,
because 1.2's state table differs from 1.2.1's. `html.rs` holds HtmlToSpannedConverter's text
rules for compact mode.

The port was fuzzed against the real jar. Each run had about 4,000 generated inputs, mixing
tags, entities, CR/LF, C1 controls, CDATA elements, broken markup and the real licence texts.
There were 9 runs, about 35,000 inputs in all, and **0 differences**, with two exceptions:

- `&#99999999;` makes Java build an unpaired surrogate. A Rust `String` cannot hold one, so it
  shows U+FFFD.
- One input made TagSoup itself throw `StringIndexOutOfBoundsException` in `Parser.split`. That
  would crash Compose. Rust shows the text it parsed.

## Parity

| Area | Status |
| --- | --- |
| Text, order, badges | Matched by `SettingsLicencesMatchTest`. It runs AboutLibraries' own parser, HashSet and sort, and TagSoup 1.2, over 18 generated resources: every row and every dialog. |
| TalkBack | A row is a `card` node (a Button to TalkBack), as other Rust rows are. Its label is the name, and its state is the version, author and badges joined with ", ". The dialog has a scroll node, the licence text as one focusable text node (id 9), and the `OK` button (id 7). Keyboard focus goes through the back button and every row, scrolling each into view. In the dialog it goes text, then OK. TalkBack's scroll forward and backward move a list by its own height, as Compose's do. |
| Light and dark | Colours come from the palette. The device pictures match in both (see Device results). |
| Font scale | Row heights come from the sp tables, as other screens' do. The geometry is recomputed when the width, density or font scale changes. The first visible row stays at the top, as LazyListState keeps it. |
| Forced LTR under an RTL locale | The project licence screen lays its text out left to right in any locale, as Compose forced. Under RTL, the third-party screen mirrors in Compose. The Rust settings renderer has no RTL layout on any screen yet (the APK ships English only), so RTL is owed renderer-wide. |
| Saved state | The dialog is not saved: LibrariesContainer keeps it in a plain `remember`. The list's scroll is saved like any screen's. Re-anchoring rows across activity recreation is not done: the saved pixel offset is restored. |

Known gaps, all documented and none of them visible with the shipped data:

- **HTML styling is dropped.** The text matches exactly, but bold, links and headings are not
  drawn styled. The shipped licence texts are plain text with line breaks.
- **The version text uses parley.** It is measured and drawn in monospace through parley, not
  through the hinted Android text path that other settings text uses.
- **org.json leniency is not copied.** The parser uses serde_json, which is strict. Android's
  org.json accepts some malformed JSON and keeps the last of duplicate keys. The generated file
  is strict JSON, so this does not matter for it.
- **The JVM reference cannot cover a JSON null organisation name.** Desktop org.json differs
  from Android's there. Rust follows Android's `"null"`.

## Virtualisation timings

These are host numbers: `cargo run --release -p render --example settings_licences_timing`
on an Apple-silicon Mac. They use the real 139-library `aboutlibraries.json` and Roboto, on a
1080 by 2400 window at 2.625 density. Each frame includes the full CPU raster. Device
numbers are under Device results.

| Step | Median | p95 | Max | Samples |
| --- | --- | --- | --- | --- |
| Open: parse, measure all rows, lay out and raster the first page | 2.46 ms | 3.01 ms | 3.01 ms | 20 |
| Scroll frame, 40 px steps through the whole list | 1.31 ms | 1.52 ms | 8.26 ms | 15,280 |
| Open a licence dialog (Apache 2.0 text) | 5.97 ms | 8.69 ms | 8.69 ms | 20 |

Without the raster, opening takes 0.67 ms and a scroll frame 0.02 ms (scene only). A frame
lays out only the 10–15 visible rows. A Rust test checks that with 500 libraries, fewer than
20 rows are laid out.

## Device results

Run on 2026-11-15 on aw_keyboard_ssd (emulator-5556, Android 16, 1080 by 2400 at 2.625), from
a clean `-PsettingsFixture=true` build of these commits. The device was muted and its private
baseline verified before and after. adb stayed connected from boot to shutdown (1,336 health
samples, 5 s apart, none dropped). Receipts are in `build/stage4-licences-device/`, which is
not committed.

**The suite.** `settings-device-tests.py ... all` passed every mode on both renderers
(`suite4/`):

- `ui`: 0 accessibility violations on either renderer, and byte-identical preference files
  (5,289 bytes). The group 1, group 2 and licences walks are identical.
  - The licences walk names 110 rows in the same order on both: 139 libraries with 110
    distinct names.
  - The first row's dialog has the same text hash on both, and its OK is a Button on both.
  - Group 2 audits the third-party screen as a Rust screen.
- `text`, `death` and `crash` (light and dark) passed. The text preference files are
  byte-identical.
- Compose's known defects: 286 audit nodes. 28 of them are new: `aboutlibraries-row-no-role`.
  AboutLibraries' row is a `Modifier.clickable` with no Role, so it is a View; Rust's row is
  a button. Rust has no known defects.

The first two runs found one real difference and three faults in the test itself. All four
are fixed:

- **TalkBack paging (Rust).** Scroll forward and backward moved Rust lists by 0.6 of the
  window. Compose moves a list by its own height. `App::accessibility_scroll` now does what
  Compose does: on device, both lists are 2,106 px at y = 231, so a page is 2,106 px on
  both. `talkback_pages_by_the_scroll_nodes_own_height_like_compose` pins it, and the
  `talkback-page-fraction` mutation fails that test.
- **The walk.** Compose labels a row clipped at the top from its first visible child, so a
  row's name could read "Apache License 2.0". The walk now counts a row on the page where its
  name line is inside the list. It also waits for Compose's animated page to stop, and hashes
  the dialog's text leaves rather than every container.
- **The keyboard bind.** `am instrument` kills the keyboard. The system rebinds it only on a
  start of input at least 3 s after its last bind attempt, and the Rust field was focused
  2.87 s after the kill. `awaitIme` now restarts input while no keyboard is bound.
- **The gallery.** `scroll top` does not move Compose's LazyColumn, so the gallery opens the
  screen again instead. The project licence screen is waited for by its title.

**Pictures.** The strict diff (3/255 per channel, a one-pixel neighbour band, system bars
masked), in `pictures/comparison.json`:

| Appearance | Licences (4 pairs) | Every pair | Max error |
| --- | --- | --- | --- |
| Light | 4/4, max 0 | 80/80 | 1/255 |
| Dark | 4/4, max 1 | 80/80 | 2/255 |
| Font scale 2.0 | 1/4 | 1/80 | 250/255 |

Font scale 2.0 is owed renderer-wide, as before. These counts are recorded, not chased. The
one match is the project licence screen, which Compose drew on both renderers in that run. The pixels
over 3/255 are 10,507 on the list, 9,694 on the lower page and 2,504 on the dialog.

**Timings.** `settings-licences-timing.py` ran the trials compose, rust, rust, compose with
Gradle idle (`timing/summary.json`), using A5's method: a Perfetto frame timeline, with input
joined to frames by vsync id. The figures are per trial.

| | Compose | Rust |
| --- | --- | --- |
| Open: tap to first present | 25.2, 24.5 ms | 20.9, 14.7 ms |
| Open: tap to settled | 520.9, 521.7 ms (32–33 frames) | One frame |
| Licence dialog: tap to first present | 65.1, 56.1 ms | 23.7, 25.0 ms |
| Licence dialog: tap to settled | 876.9, 874.5 ms (51–52 frames) | One frame |
| Scroll: input to present, p50 / p95 | 31.0 / 50.1, 28.6 / 43.9 ms | 31.8 / 45.8, 31.2 / 47.6 ms |
| Scroll: main-thread frame, p50 / p95 | 2.25 / 3.90, 2.18 / 3.87 ms | 1.60 / 2.72, 1.62 / 2.75 ms |
| Scroll: Rust scene, p50 / p95 | – | 0.94 / 1.62, 0.94 / 1.74 ms |
| Scroll: Rust raster, p50 / p95 | – | 2.09 / 2.43, 2.11 / 2.48 ms |
| Scroll: missed app deadlines | 0, 0 | 0, 0 |
| PSS after the trial | 90.0, 90.6 MiB | 112.1, 108.7 MiB |

Compose's settled times are its route transition and its dialog's enter animation. Rust shows
the screen and the dialog finished in one frame.

Still owed after this run:

- **Animation.** The Rust settings renderer has no route transition and no dialog enter
  animation. Compose also animates TalkBack's page scroll, where Rust jumps. Both are
  renderer-wide, not licences work.
- **Font scale 2.0 pictures**, renderer-wide, as above.
- **List semantics.** Compose's list is a View with CollectionInfo, which TalkBack calls a
  list; Rust's is a ScrollView. AboutLibraries' role-less rows are a Compose defect.
- **A real TalkBack session on a phone**, as for every settings screen.
- **RTL**, renderer-wide (above).

## The project licence screen

Compose's `ProjectLicenseScreen` was a `SelectionContainer` around a 10 sp monospace Text with
no soft wrap, forced left to right, inside a vertical and a horizontal scroll. Rust draws it
since compose-removal step 8 (`render/src/settings_ui/project_licence/`), and the Compose screen
is deleted.

**Data path.** As the third-party list's: at its first frame without text, Rust sends
`["asset", "license/project_license.txt"]`, once until an answer comes back or the screen is
opened again without text. `SettingsReadOnlyText` reads it with
`FlorisRef.assets(…).loadTextAsset` on `Dispatchers.IO` and answers with settings op 25
(`ASSET`): the path, `ok` or `failed`, then the text or the exception's message. A failure shows
`about__project_license__error_license_text_failed` with that message, as Compose's
`getOrElse` did. The text stays loaded when the screen is left.

**Layout.** One line per newline, shaped by parley at `sp(10)` in the renderer's monospace
face (DroidSansMono on a device), unwrapped and left to right. Only the lines inside the
viewport are drawn.

**Scrolling.** The vertical axis is the screen's own scroll, with its stretch, fling and
scrollbar. The horizontal axis is the licence's own clamped scroll, with a fling from Compose's
velocity tracker and a 4 dp scrollbar. Whichever axis passes the 8 dp touch slop first owns
the drag, the horizontal one on a tie, as the inner `horizontalScroll` sees the event first.

**Selection** (`selection.rs`, `touch.rs`):

- Holding still for 400 ms, Android 12's long-press timeout, selects the ICU word under the
  finger, or one character between words (`ensureAtLeastOneChar`), and ticks
  `TEXT_HANDLE_MOVE`. A drag that follows extends by whole words.
- A tap clears the selection.
- The handles are the editor's (`field.rs`'s `paint_handle`): 25 dp teardrops in the primary
  colour under the line, each hidden while its end is scrolled away, and drawn past the
  viewport as Compose's popup is. A handle moves its end by characters and keeps where it was
  grabbed. The ends may cross, and never collapse onto each other.
- Holding a selection drag within 32 dp of a viewport edge scrolls that way every frame: 600 dp
  a second at the edge, up to four times that deeper in. The selection follows the still finger.
- The highlight is the primary colour at 40%, as `TextSelectionColors` has it. It fills as
  Android's `Layout.getSelection` does for Compose: every line but the last runs on to the
  text's right edge and every line but the first starts at its left, so the lines between,
  blank ones too, are one block. The first build lit only each line's text; the emulator check
  showed the gap at a blank line.

**Toolbar.** The reply's `textToolbar` gives the selection's visible area with room for the
handles, and whether Select all is offered (not when everything is selected).
`SettingsSelectionMenu`, the editor's floating `ActionMode`, shows Copy and Select all there.
Copy comes back as a `["clip", text]` effect, Kotlin puts it on the clipboard labelled "plain
text" as Compose's `AndroidClipboardManager` does, and the selection clears. There is no
toolbar while a drag runs or when the selection is scrolled out of view.

**TalkBack.** The text is one node (id 11, a TextView) labelled with the whole text. It has the
character, word, line, paragraph and page granularities, its selection, and the actions
`ACTION_SET_SELECTION`, `ACTION_COPY` and next and previous at a granularity, extending or not.
Rust moves the cursor, which is the selection's focus, and Kotlin sends
`TYPE_VIEW_TEXT_TRAVERSED_AT_MOVEMENT_GRANULARITY`. At the text's end the action returns false,
so TalkBack moves on. The horizontal scroller is its own scroll node (id 12), and TalkBack's
scroll pages it by its width.

**Saved state.** Version 6 adds the horizontal scroll, which `rememberScrollState` also saves.
Both scrolls are held until the text has come back, then applied. A second recreation inside
that wait, a few milliseconds, saves the horizontal scroll as zero, so the text reopens at its
left edge (owed, not fixed).

### Where it differs from Compose

Improvements:

1. **The read is off the main thread.** Compose read the asset on the main thread inside
   composition. The cost is one empty frame while the read runs.
2. **The horizontal scrollbar stays on screen.** It runs along the viewport's bottom edge.
   Compose's bar sat inside the vertical scroll, at the bottom of the 200-line text, so it was
   seen only at the end.
3. **A drag held at an edge keeps scrolling.** Compose brings the dragged end into view only on
   each move, so a still finger stopped the scroll.
4. **TalkBack can extend a selection and move by page.** Compose's node for non-editable text
   ignores the extend argument and has no page granularity.

Gaps, in [owed checks](owed-checks.md):

- The text is shaped by parley in DroidSansMono, not by Android's text stack, so glyphs can sit
  a pixel apart from Compose's.
- The horizontal axis clamps at its ends. Compose's `horizontalScroll` stretches there.
- A hardware keyboard's Ctrl+C and Ctrl+A do nothing on the selection.

## Proof

- **Rust:** `crates/render/tests/settings_ui_licences.rs` has 15 tests:
  - HashSet order checked against a JVM, the UTF-16 sort, the author line, the fallbacks, and
    the catch-all.
  - HTML cases checked against TagSoup 1.2.
  - The load plan, row text, and virtualisation.
  - Focus walking, the lazy scrollbar, the dialog's controls and its scrolling, and
    re-anchoring after a width change.
- **JVM:** `ReferenceLicenceScreens.kt` and `SettingsLicencesMatchTest.kt` (4 tests).
  `SettingsTypographyTokensTest` pins labelSmall, bodySmall and titleSmall.
- **Mutations:** seven new cases in `scripts/settings-ui-mutations.py`:
  - `licence-set-file-order`, `library-order-code-points`, `author-organisation-first`,
    `blank-version-shown`, `html-legacy-block-lines` and `empty-licence-text-node` must fail
    the JVM match test.
  - `every-row-drawn` must fail the Rust licence tests.
  - `talkback-page-fraction` must fail the Rust screens tests. It was added after the device
    walk found the paging difference.
- **Device (run on 2026-11-15; see Device results):**
  - `SettingsLicencesDevice.kt` walks every row on both renderers, audits the top, a lower
    page and a dialog, and records the dialog's text hash and button.
  - `settings-device-tests.py` saves `licences.json` and requires the two walks to be equal.
    The group 2 walk now audits the third-party screen as a Rust screen.
  - `settings-pictures.py --sections licences` captures the top, a lower page, the first
    library's dialog, and the project licence screen (Compose on both in that run; Rust's since
    compose-removal step 8).
- **The project licence screen (compose-removal step 8):**
  - **Rust:** `crates/render/tests/settings_ui_project_licence.rs` has 17 tests: the route and
    its one asset read, only the visible lines drawn, the error text, axis lock and clamping on
    both axes, word selection by long press (one character between words), extending by words,
    handle drags across lines and past each other with the highlight filled between, the edge
    hold scrolling both ways and stopping away from the edges, Select all and the copy payload,
    hiding the handles and the toolbar when scrolled away, every TalkBack granularity forward,
    back and extending, TalkBack paging, keyboard focus, and both scrolls surviving recreation.
  - **JVM:** `SettingsProjectLicenceMatchTest.kt` (4 tests) drives the real natives as
    `RustSettingsView` does: the asset plan and the `ASSET` op, the text node's TalkBack facts,
    Select all, Copy and traversal through the `SELECTION` op, the toolbar in the reply, the
    error string, and the horizontal scroll across recreation. The group 1, group 2 and UI
    match tests list the route as native.
  - **Mutations:** 36 cases, all killed. 32 must fail the Rust tests; `text-node-facts-dropped`,
    `text-toolbar-dropped`, `selectable-label-twice` and `saved-horizontal-ignored` must fail
    the JVM test.
  - **Device (run on 2026-11-21):** aw_keyboard (emulator-5558, Android 16, 1080 by 2400 at
    2.625), muted, build `0.5.2-debug+0be2d6291` installed with `scripts/install-dev.sh`.
    Screenshots are in `build/stage4-licences-device/project-licence/`, which is not committed.
    - About → Project license opened on `RustSettingsView` in DroidSansMono. Both axes
      scrolled, and the horizontal one stopped at the longest line's right edge.
    - A long press on a word selected it with both handles and no toolbar while the finger
      was down. At the lift the floating toolbar offered Copy and Select all.
    - The end handle dragged across the URL and blank lines onto the TERMS line, and the start
      handle up onto "Apache". Held at the bottom edge, the end scrolled the text about 30
      lines and the selection followed. This run found the highlight gap at blank lines (see
      Selection), fixed before the final build.
    - Copy cleared the selection. Pasted into the render probe's Benchmark editor, the text was
      4,473 UTF-16 units with the SHA-256 of the asset's text from "Apache" to where the hold
      stopped.
    - Select all left only Copy on the toolbar. Copied and pasted, it was 11,363 units with the
      SHA-256 of `license/project_license.txt`.
    - No crash in logcat. TalkBack did not run: it has never been enabled on the emulator, and
      P6 owes a phone session for every settings screen.
