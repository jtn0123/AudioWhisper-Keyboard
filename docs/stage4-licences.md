# Stage 4: the licence screens

This page covers the two licence screens under About. Both are still behind the default-off
renderer selector, and Compose stays the fallback.

| Route | Drawn by | Why |
| --- | --- | --- |
| `settings/about/third-party-licenses` | Rust | Ported here. |
| `settings/about/project-license` | Compose, on both renderers | It needs text selection, which is owed (see below). |

`catalogue::native_routes()` now includes the third-party route. Kotlin gets the native route
list from Rust, so nothing else had to change for it to route there.

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
| Forced LTR under an RTL locale | Applies to the project licence screen, which stays in Compose. Under RTL, the third-party screen mirrors in Compose. The Rust settings renderer has no RTL layout on any screen yet (the APK ships English only), so RTL is owed renderer-wide. |
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
one match is the project licence screen, which Compose draws on both renderers. The pixels
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
- **The project licence screen's text selection**, and RTL (both below).

## Text selection (project licence screen): owed

The project licence screen is a `SelectionContainer` around a monospace Text at 10 sp, with no
soft wrap, that scrolls both ways. Porting it needs:

- Two-axis scrolling.
- Long-press word selection over a no-wrap monospace layout.
- Selection handles and drag.
- The floating copy toolbar for read-only text.
- Accessibility selection actions.

The editor already has handles and a toolbar, but only for an editable field. Turning them
into a read-only selection model over a 200-line, horizontally scrolling text is estimated at
**2–3 days**, including device checks of handles at the scroll edges. Until then, the route
stays on Compose for both renderers.

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
    library's dialog, and the project licence screen (Compose on both).
