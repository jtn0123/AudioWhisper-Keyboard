# Phase 3, item 3: theme, then subtype

Phase 3 of [the all-Rust plan](all-rust.md#3-rust-owns-the-keyboard-l) hands the keyboard to Rust
one piece at a time. Item 3 is the theme, then the subtype.

**The theme moved.** Before this work, Kotlin's `ThemeManager` chose the active theme (day,
night, follow the system or the clock, a preview), decided whether to load it, keep it or show
one it had loaded before, and held the base stylesheet as a Kotlin builder
(`FlorisImeThemeBaseStyle`). The inline autofill chips took their colours from a Compose query
that the keyboard's Compose tree cached, and the Rust renderer's palette came from the Compose
root's composition locals. Now **Rust decides**: `crates/imetheme` chooses the theme, runs the
theme session (what to load, what to keep, what to show again), holds the base stylesheet,
resolves the chips' colours, and picks the accent the palette is made from. Kotlin reads the
settings and the system's night mode, unpacks and reads the archive Rust names, parses the
stylesheet for the Compose objects that still need it, and builds the Android bundles.

**The subtype moved, in this item's second half.** Before it, Rust had the subtype's rules and
codec, and Kotlin held the list, the active subtype and everything the key graph reads of a
subtype. Now `crates/imesubtype` holds the list and the active subtype and answers what each one
names; the key graph takes those facts from it. Kotlin names locale tags with Android's
`Locale`, reports the currency sets it loaded, makes the Jetpref writes Rust asks for and
publishes the flows. [The subtype half](#the-subtype-half) has what moved, its wire shapes and
its proof.

The Compose keyboard stays as the fallback until
[step K](compose-removal.md#k-the-compose-keyboard). The Rust renderer's path no longer reads
the Compose root's palette or the Compose chip cache.

Branch `all-rust/theme-subtype`, rebased on main 48b2e0018 (compose-removal step 12 and the
`install-dev.sh` fix). Its commits:

| Commit | What |
|---|---|
| 17285e68f | Deletes `FlorisImeUiSpec`, all of it commented out since Snygg v2 |
| 420d350f1 | `crates/imetheme`, the `ImeTheme` wire and boundary, and the base style as JSON |
| 89019851f | `ThemeManager`, the renderer's theme and the inline chips take Rust's decisions |
| 4b48cd624 | The seeded differential traces against main's frozen theme manager |
| 7abf25bc1 | The mutation script |
| 062b84102 | `theme-timing.py` and `explain-digest.py` |
| 95fc24574 | `switch-cost.py`, for E41 |

## What moved

### The theme session (`lib/rust/crates/imetheme`)

* **`choose`**: which theme the settings make active, `ThemeManager.evaluateActiveThemeName`. A
  previewed name wins; otherwise `ALWAYS_DAY` and `ALWAYS_NIGHT` choose by name,
  `FOLLOW_SYSTEM` by the system's night mode, and `FOLLOW_TIME` by the clock: day from sunrise
  to sunset, both included, and always night when sunset comes before sunrise (main's
  `current in sunrise..sunset`, kept as it was).
* **`session`**: what an update does, `ThemeManager.updateActiveTheme`. A theme the editor holds
  as a preview shows at once. A theme loaded before shows again from its slot. A name whose
  extension id is a keyboard extension's, or no theme extension's, or that has no archive, or no
  such component, changes nothing, as on main, where each of those returned early. Otherwise
  Android is asked to load the stylesheet at the path Rust names (`extensions::stylesheet_path`:
  the component's own, else `stylesheets/<id>.json`). A load that works is kept in a new slot; one
  that fails shows the base theme with the failure and is tried again on the next update; one
  that is never reported (the next update came first) is forgotten. An index change clears every
  slot first, as main cleared its cache.
* **`base`**: the base stylesheet, `base-style.json`, main's `FlorisImeThemeBaseStyle` written
  out once (7,290 bytes). `ThemeInfo.DEFAULT` parses it.
* **`inline`**: the inline autofill chips' background and foreground. Main queried
  `inline-autofill-chip` in Compose below the theme's root and the keyboard's window (each with
  the keyboard's mode and shift state), starting from a 14 sp font size, and took the static
  background colour (else white) and foreground (else black). Rust walks the same chain with
  `CompiledTheme::resolve_with_palette`. Compose passed only the palette colours the element's
  rules named; only those can end up in the result, so the whole palette gives the same answer.
* **`calls`**: the rows to and from Android, and the accent rule: the settings' accent if they
  name one, else the system's (`android.R.color.system_accent1_500`), the primary colour the
  Material You palette is made from.

### What the Rust renderer is handed

`rendererTheme` hands the renderer the active theme's stylesheet JSON, the palette, the display's
density and font scale, the font-size multiplier, and the keyboard's mode and shift state. The
palette is now made from Rust's primary and the theme's Material You flags (`themePalette`),
where before it was read from the Compose root's `LocalSnyggDynamic*` locals; the traces show the
two are the same palette. `rememberSnyggPalette`, which read those locals, had no caller left and
is deleted.

### Inline autofill styles

`ThemeManager.createInlineSuggestionUiStyleBundle` asks Rust for the chips' colours in the active
theme, for the keyboard's mode and shift state at the time Android asks, and builds the
`InlineSuggestionUi` style from them. The Compose chip cache (`InlineSuggestionsStyleCache`,
`CachedInlineSuggestionsChipStyleSet`) and the dead `RemoteColors` helpers are deleted.

## The wire shapes

All of it is declared in `crates/wire` (`ime_theme.rs`) and `crates/boundary`
(`declarations/ime_theme.rs`) and generated on both sides: four natives (`ImeThemeNative`'s
`create`, `free`, `call` and `rules`), none written by hand. The build has 400 natives; main has
396.

`ImeTheme` is a handle (`create`, `free`, synchronized) with one call and one static rule:

* **`ImeTheme.call(ImeThemeOp.UPDATE, …)`**: an `ImeThemeUpdate` row (`CLEAR`; `MODE`; the day
  and night names as `*_EXTENSION` and `*_COMPONENT`; `SUNRISE`, `SUNSET` and `NOW` in
  nanoseconds of the day; `SYSTEM_NIGHT`; `PREVIEW` with `PREVIEW_EXTENSION` and
  `PREVIEW_COMPONENT`; `PREVIEW_INFO`, whether the editor holds a preview theme; and the counts
  `KEYBOARDS` and `THEMES`), then the keyboard extensions' ids, then per theme extension an
  `ImeThemeExtension` row (`ID`, `SOURCE`, `COMPONENTS`) and an `ImeThemeComponent` row (`ID`,
  `STYLESHEET`) per theme. The answer is one `ImeThemeStep` row: `KIND`
  (`ImeThemeStepKind`: `PREVIEW`, `CACHED`, `KEEP`, `LOAD`), `CLEARED`, `SLOT`, and for a load
  `EXTENSION` and `COMPONENT` (indexes into what was sent) and `PATH`.
* **`ImeTheme.call(ImeThemeOp.LOADED, ok)`**: an `ImeThemeLoaded` row; the answer is a step of
  kind `ACTIVE` with its `SLOT`, or `FAILED`.
* **`ImeTheme.rules(ImeThemeRule.BASE)`**: the base stylesheet's JSON.
* **`ImeTheme.rules(ImeThemeRule.INLINE, …)`**: an `ImeThemeInline` row (`STYLESHEET`, `MODE`,
  `SHIFT`), then the palette's names, light colours and dark colours; the answer is an
  `ImeThemeChip` row (`BACKGROUND`, `FOREGROUND`) as ARGB.
* **`ImeTheme.rules(ImeThemeRule.PRIMARY, …)`**: an `ImeThemeAccent` row (`SPECIFIED`,
  `PREFERRED`, `SYSTEM`); the answer is the primary colour.

Loaded themes stay on the Android side, in the slot Rust names: a `ThemeInfo` holds the parsed
`SnyggStylesheet` the Compose keyboard and the theme editor still need, and its unpacked
directory, for fonts and images.

## What Kotlin still does, and why

Production Kotlin is **330 counted lines smaller** (code lines, as the ledger counts them). The
ledger has 401 rows for 401 production files (main: 402).

| File | Counted lines, main → branch | What it does now | Why it is Kotlin |
|---|---:|---|---|
| `ime/theme/ThemeManager.kt` | 309 → 203 | Reads the settings, the clock and the system's night mode into one row; follows the extension index; unpacks the archive Rust names and reads the stylesheet; parses it into the `SnyggStylesheet` the Compose views and the theme editor use; publishes the active theme; builds the `InlineSuggestionUi` style bundle from Rust's chip colours | Jetpref, `Resources.Configuration`, the extension manager's files, `ZipUtils`, and the inline suggestion API are Android's |
| `ime/theme/ThemeUpdates.kt` | new, 93 | Encodes the facts and the extensions as rows for `ImeThemeOp.UPDATE`, runs the load Rust asks for, reports it with `LOADED`, and keeps each loaded theme in the slot Rust names | The loaded themes are Android-side objects: a parsed stylesheet for Compose and an unpacked directory for fonts and images |
| `render/RendererTheme.kt` | 27 → 73 | Asks Rust for the primary colour, makes the Material You palette from it, hands the renderer the theme's JSON, the palette, density, font scale and the keyboard's state, and asks Rust for the chips' colours | The system accent (`system_accent1_500`) and `DisplayMetrics` are Android's; the palette maths is materialkolor's (see [what remains](#what-remains)) |
| `lib/snygg/.../ui/SnyggPalette.kt` | 27 → 19 | Makes a `SnyggPalette` from a primary colour and flags | `rememberSnyggPalette`, which read the Compose root's locals, lost its last caller and is deleted |
| `ime/smartbar/InlineSuggestionsUi.kt` | 74 → 62 | The inline chips' views | The Compose chip-style cache it filled is deleted |
| `FlorisInlineSuggestions.kt` | 43 → 38 | The inline suggestion request and response | It asks the theme manager for the style bundle at request time instead of reading the cache |
| `render/RustKeyboardLayout.kt` | 91 → 89 | The Rust renderer's root | Stopped filling the chip cache |
| `ime/text/TextInputLayout.kt` | 79 → 77 | The Compose keyboard's text layout (class K) | Stopped filling the chip cache; a deletion, as the Compose freeze allows |
| `ime/theme/FlorisImeThemeBaseStyle.kt` | 333 → deleted | | Moved to test sources as `ReferenceFlorisImeThemeBaseStyle.kt`; Rust holds the base style |
| `ime/theme/FlorisImeUiSpec.kt` | 1 → deleted | | All of it was commented out after Snygg v2 (upstream 3c2b2ae47) |

The ledger's budgets were tightened to these counts. One was raised: `RendererTheme.kt`, 27 → 73,
with its reason in the ledger: the renderer's theme, the palette's accent and the chips' colours
are handed over here instead of read from the Compose root, while `ThemeManager.kt` shrank by 106
and the base style left production.

Kotlin also still holds the theme files' data model, `ThemeExtension`, `ThemeExtensionComponent`
and `ThemeMode`. The first two are what the extension manager parses `extension.json` into, and
the extension manager (`lib/ext/ExtensionManager.kt`, `ExtensionCodec.kt`) still reads
`extension.json` in Kotlin; compose-removal step 10 moved the extension editor's screens to Rust,
and `SettingsExtEditor` and `ExtensionEditorWork` still take these types. They move with the
extension manager. `ThemeMode` is the type of a Jetpref setting. None of them makes a
decision now: Rust reads their fields from the rows.

### Theme files that now serve only the Compose fallback

None became fallback-only in this item. The Compose files that remain all still serve Compose
views the Rust renderer's keyboard also shows (the root, the panels, the dictation mic), so they
go with the Compose root at item 4, not with the Compose keyboard at step K:

* `ime/theme/FlorisImeTheme.kt` hosts the Compose root, which wraps the Rust renderer too. It
  makes no decision now: it provides the active theme Rust chose to the Compose views.
* `ime/theme/FlorisImeUi.kt`, the Snygg element and attribute names those views query.
* `lib/snygg/SnyggTheme.kt` and `lib/snygg/ui/*`, the Compose views' theme and its queries.

## Proof

### The differential traces

Both traces run main's frozen Kotlin side by side with the branch's code over seeded synthetic
input, three seeds (20261122, 7 and 424242) of 50,000 states each, and assert after every state.
Main's code is kept as test-only copies: `ReferenceThemeManager.kt` (main's `updateActiveTheme` and
`evaluateActiveThemeName`, with Android's reads behind an interface the trace fills),
`ReferenceFlorisImeThemeBaseStyle.kt` (main's base style), and, for the chips, Snygg's existing
`ReferenceSnyggTheme` (main's Compose query).

**The theme trace** (`ThemeMatchTest`, `ThemeTrace.kt`) builds one synthetic world from the bundled
theme extensions (read from `app/src/main/assets/ime/theme`) and lets random events change it:

* the extension index: extensions removed, added (synthetic ones, with zero to three themes,
  random Material You flags and stylesheet paths that are missing, empty, blank or their own),
  reordered, their archive toggled, or the index reset;
* the settings: the theme mode (all four), the day and night themes (mostly ones that exist,
  sometimes any name), sunrise and sunset (often equal to each other or to the clock);
* the clock, the system's night mode, the settings' accent (unset or any colour);
* the editor's previewed theme id and preview theme;
* keyboard extensions that take a theme extension's id;
* the stylesheet files: replaced by a bundled sheet, by one of 14 synthetic ones (empty, not
  JSON, the wrong schema, an unknown element, bad values, a duplicated rule, a lone surrogate),
  deleted, or made to throw a cancellation when read;
* an update cancelled before it takes the lock, and the service made again (a new manager on
  each side, starting from the base theme).

After every event both sides update, and the theme each shows must match field by field: its
name, its configuration, its parsed stylesheet, its directory, and its failure (extension,
component, cause class and message). Then what the renderer is handed is built from each for a
random density (0.75 to 3), font scale and font-size multiplier (0.5 to 1.5), keyboard mode and
shift state: main's from the active theme's JSON and the palette main's Compose root made, the
branch's from `rendererTheme` and Rust's primary. The two `RendererTheme` values must be equal.

| Seed | States | Loads | Cached | Kept | Recreated | Base / loaded / preview / failed shown |
|---|---:|---:|---:|---:|---:|---|
| 20261122 | 50,000, identical | 5,685 | 1,622 | 24,643 | 1,484 | 19,989 / 8,289 / 6,307 / 15,415 |
| 7 | 50,000, identical | 5,413 | 1,713 | 24,395 | 1,477 | 19,860 / 8,686 / 6,487 / 14,967 |
| 424242 | 50,000, identical | 5,583 | 1,526 | 24,909 | 1,498 | 19,882 / 7,845 / 6,055 / 16,218 |

Each seed set every theme mode about 3,000 times, and saw every failure kind (an unreadable
stylesheet, an unparsable one, a cancelled load) thousands of times. The test fails if a seed
misses any of 23 named cases. The full counts are in `build/theme-subtype/traces-final.txt`, and
the last gate's run gave the same counts for both traces.

**The base style.** A second test checks that the JSON Rust holds is main's
`FlorisImeThemeBaseStyle` written out, that `ThemeInfo.DEFAULT` parses to main's stylesheet, and
that the whole default theme equals main's. Since the base style is a bundled stylesheet,
`RenderThemeMatchTest` now also checks that the Rust renderer resolves it like main's Kotlin, in
every drawing state, and the render crate's frozen fixture holds its expectations.

**The chip trace** (`InlineChipMatchTest`, in `lib/snygg`) compares the chips' colours: main's
Compose query (`inline-autofill-chip` below the root and the window, each with the keyboard's
mode and shift state, from a 14 sp font size; the static background else white, the foreground
else black) against Rust's `inline` rule. Each seed draws from the bundled stylesheets and 400
synthetic ones. Those style the chip, the window, the root and keys with every colour form
(hex, `rgba`, `transparent`, light and dark dynamic colours, variables, `undefined`, `inherit`,
garbage), under mode and shift-state attributes and the pressed selector. Each state picks a
stylesheet, one of eight modes and four shift states, and one of nine palettes (the yellow
fallback and four primaries in two Material You styles).

| Seed | States | Stylesheets | Synthetic ones the parser rejected | States whose sheet names that mode or shift state | Themed background / foreground |
|---|---:|---:|---:|---:|---|
| 20261122 | 50,000, identical | 413 | 349 | 14,094 | 19,807 / 21,991 |
| 7 | 50,000, identical | 413 | 346 | 14,338 | 18,804 / 21,049 |
| 424242 | 50,000, identical | 413 | 357 | 14,158 | 19,896 / 23,423 |

The 13 bundled sheets are the 12 bundled themes and the base style, which names the Snygg schema
too (see [the gate](#the-gate)). Rejected sheets are replaced, not counted: the test takes 400
that parse. The counts are from the final gate's run; before the base style was counted there were
412 sheets, and every state was identical then too.

Writing this trace found that its first version had quoted attribute values wrongly
(``[mode=symbols]`` where Snygg needs ``[mode=`symbols`]``). The parser rejected every sheet with an attribute, so attributes were
never exercised. The test now asserts that more than half of the synthetic sheets have
attributes and that every case (bundled, synthetic, default and themed colours, a sheet naming
the state) is seen more than 1,000 times.

A third test, `theChipsAreStyledFromTheWholePaletteInTheKeyboardsState`, checks the adapter:
`inlineChipColors` sends the stylesheet, mode, shift state and the whole palette, light and dark,
as Rust reads them.

### Intended differences

The traces found none. One behaviour differs by design, outside what they compare:

* **The inline style bundle is always there.** Main built it from a cache the Compose keyboard
  filled when it composed the chip's style. Before that first composition the bundle was null, and
  the keyboard made no inline suggestions request. The cache also held the style for the mode and
  shift state of that composition. The branch asks Rust for the active theme's chip colours, for
  the mode and shift state at the moment Android asks, so the first request has a style and it is
  never stale.

The renderer's palette now comes from Rust's primary and the theme's flags instead of the Compose
root's locals. That is a change of source, not of value: main's root made its schemes with the same
`materialYouScheme(systemAccentOrDefault(accent), …, flags)`, and the theme trace asserts the two
`RendererTheme` values are equal in every state.

### Mutations

`scripts/theme-subtype-mutations.py` makes 42 faults, one at a time, in the theme's decisions
and in the Kotlin that carries them, and runs the tests each one names. **All 42 were caught** on
the final build (95fc24574), in 30 minutes; the results are in
`build/theme-subtype/mutations-final/result.json`.

| File | Faults | Caught by `imetheme` (cargo) | by `*ThemeMatchTest` | by `*InlineChipMatchTest` |
|---|---|---|---|---|
| `imetheme/src/choose.rs` | 6 | 5 | 6 | – |
| `imetheme/src/session.rs` | 10 | 10 | 7 | – |
| `imetheme/src/calls.rs` | 7 | 4 | 5 | 1 |
| `imetheme/src/inline.rs` | 6 | 3 | – | 6 |
| `ime/theme/ThemeUpdates.kt` | 8 | – | 8 | – |
| `render/RendererTheme.kt` | 5 | – | 5 | – |

Of the 42, 17 were caught only by the JVM theme tests, 3 only by the chip trace, 4 only by the
crate's own tests, 14 by the crate and the theme tests, and 4 by the crate and the chip trace.
The faults cover:

* **the choice:** a preview ignored, day and night swapped, the system's night mode ignored,
  sunrise or sunset left out of the day, day wrapped past midnight when sunset comes before
  sunrise (main is always night then);
* **the session:** a load still finished after a newer update, a clear or the cache ignored, a
  preview's info ignored, a keyboard extension read as a theme, the wrong component or a theme
  with no archive loaded, the component's own stylesheet path ignored, a failure kept, a slot reused;
* **the rows:** the preview or source flag dropped, day read as night, the keyboard list dropped,
  a clear not reported, the system's accent always used, the chips' foreground given as their
  background;
* **the chips:** the chip resolved outside the keyboard's window, a root with no attributes, a
  pressed chip, light and dark swapped, a white default foreground, red and blue swapped;
* **the Kotlin adapters:** the system's night mode, a preview's info, the clock or the
  stylesheet path not sent; an archive taken as always there; a kept theme not shown; a loaded
  theme named without its extension; a failure without its cause; the accent, the palette, the mode or the
  shift state not handed to the renderer or the chips.

How it counts: a fault is caught only by a fresh assertion. A Rust target needs a `panicked at`
line, and a JVM target an assertion failure in a test report written after the fault was made, so
a stale report or a fault that does not compile is never a catch; none of the 42 failed to
compile. Every target a fault names is run, so the table says which tests catch it, not only the
first. `--check` confirms that every fault's location is still in the source, once. After the
last fault the source is written back, all three targets pass on it (exit 0, no assertion
failures), and the result records each file's SHA-256, which matches the committed files.

### Digests

Both digests are unchanged on the final build (95fc24574, the gated code), from
`scripts/render-benchmark/run.sh` on `aw_keyboard` (emulator-5558) with the Rust renderer:

* the 300-press digest `809d588bf48a1e90…` (300 UTF-16 units);
* the 8-popup digest `c14d19d26f290a24…` (308 units).

The first three runs on the final build each typed 300 units with another digest, and are kept
as `build/theme-subtype/render-benchmark-final-{1,2,3}-mismatch`. `explain-digest.py` explains
each from its digest alone: one key typed as its long-press hint (unit 178 `f` as `%`, unit 26
`w` as `2`, unit 93 `r` as `4`). That is E29, which hits main as often: in the CPU trials main's
build typed another digest in 3 of 5 runs, the branch's in 2 of 4. The fourth run
matched. The host's load average was 4.6 to 12.8 during the four. The four kept CPU trials, on
the branch before its last rebase (0ca6c1ced) and on main, all matched both digests too.

The autocorrect-on-space digest `6e5ce6af…0051c` ("teh" and a space giving "the ", 4 units) was
reproduced on the final build and on 0ca6c1ced, each with the device's kept-as-typed and
learned-word files moved aside and put back afterwards (E39).

### Probes

Both ran on the final build (95fc24574), and on 0ca6c1ced before it with the same result.

* **Reshow** (`reshow.py`): all 16 trials drew their keys, every key row matching. Each of the
  four ways of hiding (the keyboard's own Hide, Back, a tap outside, another app) was tried at
  gaps of 100, 300, 600 and 1,000 ms. Results in `build/theme-subtype/reshow-final/`.
* **Reading order** (`a11y-order.py`): captured in five states (letters 62 nodes, expanded 68,
  collapsed 62, the row of words 64, a popup 74). `compare-a11y.py --order` found no ordering
  issue and exited 0. Results in `build/theme-subtype/a11y-order-final/`.

### Device checks (emulator-5558)

On the branch build (0ca6c1ced), on the settings app's own field, with the Rust renderer on unless
the row says otherwise. Nothing was typed. The screenshots are in `build/theme-subtype/device/`
and are not committed.

| Check | Result | Screenshot |
|---|---|---|
| Day theme | Floris Day: white keys on light grey, the green action key | `01-day-theme.png` |
| Always night | Floris Night: dark grey keys on a darker background | `02-always-night.png` |
| Follow system | Floris Day while the system was light; after `cmd uimode night yes`, Floris Night, with no other step | `03a-follow-system-light.png`, `03b-follow-system-dark.png` |
| Another bundled theme | Floris Day (Material You): the keys, background and action key in the accent's palette | `04-material-you-day.png` |
| A borderless Material You theme | Floris Pure Night (Borderless, Material You) as the day theme: borderless keys on black, the action key in the accent's blue | `05-pure-night-borderless-material-you.png` |
| The Compose fallback | With "Draw the keyboard in Rust" off, the Compose keyboard drew the same theme: borderless keys on black and the accent-blue action key. Then switched back on, and the Rust renderer drew it again | `06a-compose-fallback.png`, `06b-rust-again.png` |
| Adding subtypes | English (US), then German (Germany) added from the presets; each row named its layout, popup mapping and currency (QWERTY / Western / Dollar, QWERTZ / Western / Euro) | `07a-two-subtypes-english.png` |
| The language key | With the utility key set to switch language, the key showed; one press switched to German: QWERTZ, the "€" hint on "d", and "German (Germany)" on the space bar | `07b-language-key.png`, `07c-switched-to-german.png` |
| Reordering, then a restart | German moved above English. The keyboard's process was stopped (`am force-stop`, then our IME selected again) and started again (a new pid). The order and the active subtype, German, were both kept | `07d-after-restart.png` |
| Removing the active subtype | Deleting German, while active, fell back to English (US): QWERTY, its name on the space bar, and the language key dimmed with one subtype left | `07e-german-removed.png` |
| Restored | Always day, Floris Day, no subtypes (the English/QWERTY fallback), the emoji utility key | `08-restored.png` |

Before the checks the emulator had no subtypes, the utility key on emojis, Always day with Floris
Day and Floris Night, the accent as found, and the system's night mode off. All of that was put
back. An extension theme that is not bundled could not be checked: none is installed. Inline
autofill could not be reached either: the emulator has no autofill service that offers inline
suggestions. Both are owed (P28).

The language key could not be pressed by its accessibility node, because agent-device refuses a
target it judges to be behind the keyboard. It was pressed at its position instead, through
agent-device.

The final build (95fc24574) was installed with `scripts/install-dev.sh`, which took a fresh copy
of the device's settings into `.device-backup/avd-aw_keyboard/` as main's new naming expects.
The keyboard drew Floris Day on the probe's field, and the digests, "teh" and the probes above
ran on it. Afterwards the device was put back: the settings file byte for byte (the
reading-order probe had left the smartbar's expanded state changed), the system's night mode
off, our keyboard the default, rotation untouched, no file left moved aside, and the media volume
at 0 (`11-restored-final.png`).

### The gate

On the branch rebased onto 48b2e0018: `cargo fmt`, then `./gradlew --stacktrace test
:app:lintDebug :lib:rust:check :app:checkNativeDeclarations :app:checkJvmLedger assembleDebug
:app:compileDebugAndroidTestKotlin`. It passed.

* **JVM:** 1,015 tests in 248 files, with 0 failures, 0 errors and 0 skipped. Main has 1,011 in
  246; the new files are `ThemeMatchTest` (three tests: the theme trace, the base style, the
  chips' adapter) and `InlineChipMatchTest` (the chip trace).
* **Cargo:** 1,483 tests, 0 failed and 0 ignored (main: 1,476); the seven new ones are
  `imetheme`'s.
* **Natives:** 400 generated, 0 hand-written, 0 missing and 0 mismatched (main: 396).
* **Ledger:** 401 rows for 401 production files (main: 402).
* **Lint:** only the allowed IDs: AndroidGradlePluginVersion ×4, NewerVersionAvailable ×4,
  ChromeOsAbiSupport ×1 and GradleDependency ×1.

The first gate on the rebased branch failed in `RenderThemeMatchTest`, which expected 12 bundled
stylesheets and found 13. Its `sheets()` collects every tracked JSON file that names the Snygg
schema, and `base-style.json` does. The base style is a stylesheet the keyboard draws with (the
default theme's), so it is now compared too, not excluded: the test expects 13 sheets and 82,368
queries (13 × 6,336), the render crate's frozen fixture `tests/themes.json` gained the base
style's key with the other 12 themes' expectations unchanged (checked with a JSON diff), and
`themes.rs` expects 13. The change is in 420d350f1, the commit that adds the file, so every
commit passes on its own. `InlineChipMatchTest` takes the same 13 bundled sheets.

### CPU trials

`cpu-trials.py --rust-only --trials 1`, alternating main (f4aa9fc30) and the branch before its
last rebase (0ca6c1ced), two each; the full entry is in
[cpu-p95-results](cpu-p95-results.md#theme-and-subtype-2026-11-22). **The 2 ms wall-clock gate
was not met, by main or by the branch. This is not a pass.**

| Trial | Frames | p50 | p95 | p99 | Over 2 ms | CPU time p95 |
|---|---:|---:|---:|---:|---:|---:|
| main 1 | 1,061 | 1.054 | 3.010 | 5.411 | 12.3% | 1.476 |
| branch 1 | 1,075 | 1.031 | 2.741 | 5.049 | 11.1% | 1.513 |
| main 2 | 1,067 | 0.996 | 3.031 | 6.684 | 11.2% | 1.576 |
| branch 2 | 1,080 | 0.892 | 2.789 | 5.645 | 10.4% | 1.493 |

The branch is not slower beyond noise: its p95 is below both of main's, and its CPU time p95 lies
between them. Its extra frames are all frames that brought only new words to the strip, within
the range earlier items saw on unchanged builds. Six trials were thrown out and run again (their
folders are kept): five typed another digest (E29) and one was stopped by agent-device (E30).

### Switch timings, and E41

Run on `aw_keyboard`, alternating the builds of the CPU trials (branch 0ca6c1ced, main
f4aa9fc30), three each, branch first, with a fixture of the settings written for them and the
device's own settings put back byte for byte afterwards. The fixture: English (US) and German
(Germany) subtypes, the utility key switching language, and *Follow system* with Floris Day and
Floris Night. 20 switches of each kind per run; every run saw 20 of 20 taps and switches.

* **Mode** and **subtype** (`rebuild-timing.py`): the `?123` key and back, and the language
  key, from the tap's release to the last frame the renderer posted for it.
* **Theme** (`theme-timing.py`, new): `cmd uimode night yes` and `no`, from a logcat marker
  written just before the command to the renderer's last frame for it. The probe's activity is
  recreated by each switch on both builds, so that is in every number.

p50 / p95 / max, in ms, and the host's load average at the start, after the rebuilds and at the
end:

| Run | Mode | Subtype | Theme | Load |
|---|---|---|---|---|
| branch 1 | 357.8 / 507.3 / 531.6 | 227.9 / 400.8 / 538.6 | 520.2 / 722.2 / 777.8 | 9.7 → 9.7 → 11.4 |
| main 1 | 336.9 / 541.8 / 584.4 | 263.6 / 344.9 / 350.1 | 547.2 / 601.1 / 613.2 | 11.6 → 11.7 → 10.9 |
| branch 2 | 368.9 / 529.7 / 1,285.4 | 268.9 / 476.3 / 504.9 | 596.7 / 3,828.4 / 7,294.2 | 10.9 → 16.1 → 21.3 |
| main 2 | 321.7 / 450.5 / 450.8 | 241.0 / 477.1 / 921.4 | 609.2 / 1,178.3 / 1,272.7 | 18.2 → 13.1 → 13.3 |
| branch 3 | 407.5 / 539.1 / 581.3 | 308.1 / 420.0 / 447.0 | 574.5 / 688.8 / 788.3 | 13.2 → 7.6 → 6.6 |
| main 3 | 325.9 / 397.0 / 483.8 | 252.5 / 343.7 / 412.8 | 508.6 / 648.9 / 739.2 | 6.8 → 4.9 → 5.0 |

* **Theme switches are level.** The branch's p50s, 520 to 597 ms, lie within main's 509 to 609.
  Branch 2's p95 and max came from a burst (switches 3 and 9 to 11) while the host's load rose
  to 21; mode had its 1,285 ms outlier then too. Each switch drew one or three frames on both
  builds.
* **Subtype switches are level within noise**: 228 to 308 ms at p50 on the branch, 241 to 264 on
  main, each build above the other in some pair.
* **Mode switches are slower on the branch in all three pairs**: 358, 369 and 408 ms at p50
  against 337, 322 and 326. That is owed as E44 (below), not passed.

**Where the time goes (E41, and E44).** E41 asked for a profile of the subtype switch's rebuild.
simpleperf cannot run on this emulator without root: its kernel cannot count user space alone
(`u` is not a supported modifier) and SELinux denies the shell the kernel's events, and
`--app` profiling finds no supported event. (The subtype half later found that its software
clock, `-e cpu-clock`, does run with `--app` and no root; [its profile](#switch-timings-the-profile-e41-and-e44)
is below.) `switch-cost.py` (new) measures instead, tapping as
`rebuild-timing.py` does, two runs per build:

* the keyboard process's CPU time per switch, all threads and the main thread, from `/proc`
  through `run-as`, which does not depend on when the emulator delivers a tap;
* each tap split from the renderer's frame log into **input** (the release's touch to its
  frame's callback, which includes anything the main thread did first), **build** (that
  frame's post to the rebuilt frame's callback) and **draw** (the rebuilt frame's callback to
  its post).

CPU per switch, in ms, main thread / all threads, each run right after an install (the folders
are `e41/cpu-*`):

| Run | Mode | Subtype |
|---|---|---|
| main 1 | 79 / 154 | 93 / 253.5 |
| main 2 | 83.5 / 160.5 | 91 / 238 |
| branch 2 | 80.5 / 154.5 | 92 / 280 |
| branch 3 | 75 / 144.5 | 85.5 / 227.5 |

Branch 1, run warm and at a load average of 43, gave 54.5 / 122.5 and 74.5 / 235 and is not
compared. 15 idle seconds cost 40 to 80 ms on all threads.

The split, medians in ms, input / build / draw / total (`e41/split-*`):

| Run | Mode | Subtype |
|---|---|---|
| main 1 | 66.9 / 138.2 / 79.1 / 328.9 | 121.7 / 27.3 / 52.6 / 215.6 |
| branch 1 | 106.4 / 81.9 / 96.3 / 383.5 | 140.2 / 37.5 / 40.6 / 264.7 |
| main 2 | 94.5 / 95.0 / 89.5 / 313.0 | 162.7 / 22.5 / 35.9 / 214.1 |
| branch 2 | 109.9 / 109.7 / 71.8 / 337.8 | 110.7 / 38.6 / 35.2 / 223.5 |

* **The branch adds no work per switch.** Its CPU per mode and subtype switch lies within or
  below main's on the main thread and on all threads.
* **E41 is explained as far as the emulator allows.** A subtype switch's wall time is mostly
  input delivery: 111 to 163 ms, which moved by 41 ms between two runs of the same build, while
  the build step is 22 to 39 ms. The key-graph branch's 17 to 57 ms gap at p50 lies inside that
  spread, so the wall-clock comparison could not show a cost that size. Which step of the build
  costs what is still owed: it needs simpleperf on a rooted image or the phone. The branch's
  subtype build step was 10 to 16 ms above main's in both split runs while its CPU was level,
  which E44 carries.
* **E44, the mode switch.** The branch's mode gap is in input, 106.4 and 109.9 ms against 66.9
  and 94.5, with build and draw mixed and CPU level. Reading the code finds no theme work on a
  mode change on either build: both builds' `rendererTheme` is remembered on the same keys, the
  palette on the accent, the system accent and the flags, and the theme manager updates on the
  same triggers; the branch even drops the chip-style cache main rebuilt on each composition.
  The cause is not found.

`build/theme-subtype/e41/` holds the runs (`split-*`, `cpu-*`) and the failed profiler log.

## What is owed

Added to [owed-checks](owed-checks.md):

* **P28**, the theme on the phone: the inline autofill chips (which the emulator cannot show) in
  day and night themes, in the symbols layout and with caps on; a theme from an imported
  extension; *Follow time* at sunrise and sunset; a Material You theme in the phone's accent and
  in a chosen one; and *Follow system* following the phone's dark mode.
* **E44**, the mode switch slower on the branch in every timing run while its CPU is level.
* **E41**, updated: measured with `/proc` and the frame log, explained as far as the emulator
  allows, and still owed the step-level profile (root or the phone).
* (The subtype half closed both: [the profile, E41 and E44](#switch-timings-the-profile-e41-and-e44).)
* **E29**, updated: a tap read as a long press, found from the digests alone for 20 of the 24
  wrong ones kept in this worktree; why, and the other four, are still owed.
* **E30**, updated: the parse error came back once, at a load average of 43.
* Not device checks: the palette maths in Kotlin, and retiring `ReferenceThemeManager` and
  `ReferenceFlorisImeThemeBaseStyle` with the theme traces.

P4, the phone p95, stays owed, and E28, the CPU gate on a quiet host.

## The subtype half

Branch `all-rust/subtype`, rebased on main b1cf45f18 (phase 5's stores). Its commits:

| Commit | What |
|---|---|
| b5097f2ca | `crates/imesubtype`, the `ImeSubtype` wire and boundary: the session holds the list and the active subtype |
| 9a6a8325c | `SubtypeManager` and the key graph read the subtype from Rust; `SubtypeSession.kt`; `SubtypeFallback.kt` deleted |
| 0a9fbaaf4 | The seeded traces against main's frozen subtype manager and build row |
| 7c0d42c63 | The active id stored only when Rust asks; keyboards keyed in one helper |
| 03207311e | The mutation script, 34 faults on the theme half's runner |
| 35813287b | The ledger's rows and budgets |
| cec9a2be3 | The trace's task count is an assertion |
| d89e9cb42 | `switch-cost.py`'s note: simpleperf's software clock runs without root |
| 4b32b51bb | The glide fix: keys are paired with the subtype they were built for |
| fee38c473 | `switch-trace.py`, for E44 |
| 9517a7440 | `switch-trace.py` splits the release's frame wait at the app vsync |

### What moved

**Before**, Rust already had the rules: `SettingsSubtypes.plan` sent `load`, `evaluate`,
`next` and `previous` to `SettingsRules` (crates `settings`, `subtype.rs`), and the codec and the
default subtype were Rust's. Kotlin held everything those rules worked on. `SubtypeManager` kept
the list and the active subtype as Kotlin `Subtype`s, ran each plan's steps, found a subtype by
id through `SettingsOp.SUBTYPE_INDEX`, and matched a subtype's currency set against the loaded
ones. The key graph read each subtype's layouts and popup mapping from `Subtype` itself
(`KeyGraphRows`), and `RustKeyGraphs` kept keyboards under `subtype.hashCode()`.

**Now Rust holds the subtypes.** `crates/imesubtype` keeps one `Session` for the keyboard. It
holds:

* the list, decoded and repaired from the stored text by the same `settings` functions main's
  load plan called;
* the active subtype;
* each locale tag as Android named it;
* the names of the currency sets Android reported loading;
* a cache key for each distinct subtype.

The session decides:

* **load**: decodes the list (blank is empty), repairs each subtype the keyboard can no longer
  use, and says whether to write the repaired list back;
* **evaluate**, after a load: the subtype whose id is stored, else the first, else the default,
  and whether to store its id (when it differs from the stored one);
* **next** and **previous**, the language key: wrapping, the default when the active subtype is
  not in the list, and always stored;
* **select**, a switch by id: stored, or nothing when the list has no such subtype;
* **active**: the active subtype, never stored;
* **currency**: which reported set a subtype's currency set names, or none (Android then uses
  its fallback set).

It answers every switch with the subtype's row: its id and cache key, its locales, its NLP
providers, composer, currency set, punctuation rule and popup mapping, and a layout for each
`LayoutType`.

**The key graph takes those facts from Rust.** `KeyboardManager` reads the active entry, a
`Subtype` and its `SubtypeFacts`, once per evaluation. `RustKeyGraphs.keyboard(mode, facts)`
keeps keyboards under `(mode, facts.key)` through one helper, `LayoutCache.subtypeKeyboard`,
which the trace uses too. `subtypeFiles(facts, …)` names each layout and the popup mapping from
Rust's row rather than from `Subtype.layoutMap`.

Two choices in the shape:

* **Locale tags.** Main passed every stored tag through `FlorisLocale.fromTag(tag)` and wrote it
  back as its `languageTag()`. That naming is `java.util.Locale`'s (`iw` for Hebrew, for one), so
  a load asks Android once for each tag it has not seen (`LOCALES`) and loads again. The session
  keeps every subtype with its tags as Android named them, so two subtypes are the same exactly
  when two Kotlin `Subtype`s were equal.
* **The cache key.** `hashCode()` let two different subtypes with the same hash share keyboards.
  Rust gives each distinct subtype value its own number, kept for the session's life (the session
  and the cache are both the application's). The traces compare every cache hit and build with
  main's, so no state they reached was affected by a collision.

**Intended difference.** Main's evaluate captured the list when it was launched; Rust evaluates
against the latest list it loaded. The two differ only when a load and an evaluate interleave on
two threads, which the traces (one dispatcher per keyboard) do not do; in that case Rust's answer
is the one that matches the stored list.

`SettingsOp.SUBTYPE_INDEX` lost its last caller and is retired. `SettingsSubtypes.plan` stays for
the settings screens (`settingsui/SettingsLanguages.kt`), whose add, remove and reorder edit the
stored list; the session reloads when it changes.

### The subtype's wire shapes

Declared in `crates/wire` (`ime_subtype.rs`) and `crates/boundary`
(`declarations/ime_subtype.rs`) and generated on both sides: three natives
(`ImeSubtypesNative`'s `create`, `free` and `call`), none written by hand. The build has
430 natives; main has 427.

`ImeSubtypes` is a handle (`create`, `free`, synchronized) with one call,
`ImeSubtypes.call(ImeSubtypeOp, vararg fields)`:

| Op | Sends | Answers |
|---|---|---|
| `LOAD` | an `ImeSubtypeLoad` row (`RAW`, the stored list) | an `ImeSubtypeLoaded` row (`ASKS`, `CHANGED`, `RAW`, `COUNT`), then either the `ASKS` tags Android must name, or `COUNT` subtype rows |
| `LOCALES` | each asked tag as `FlorisLocale` names it, in order | nothing |
| `EVALUATE` | an `ImeSubtypeEvaluate` row (`STORED`, the stored active id) | a switch |
| `NEXT`, `PREVIOUS`, `ACTIVE` | nothing | a switch (`ACTIVE` never asks for a write) |
| `SELECT` | an `ImeSubtypeSelect` row (`ID`) | a switch, or a switched row alone when no subtype has that id |
| `SETS` | the names of the currency sets Android loaded | nothing |
| `CURRENCY` | a currency set's name | its index among the sets reported, or -1 |

A switch is an `ImeSubtypeSwitched` row (`WRITE`, whether Android stores `ID`; `ID`; `ACTIVE`,
whether a subtype row follows) and then the active subtype's `ImeSubtypeRow`: `ID`, `KEY`,
`LOCALE`, `SPELLING`, `SUGGESTION`, `COMPOSER`, `CURRENCY`, `PUNCTUATION`, `POPUP`, one field
per `LayoutType` in its order (`CHARACTERS` … `SYMBOLS2_MOD`, empty for a type the subtype does
not name), and `SECONDARY`, the number of secondary locale tags that follow.

### What Kotlin still does for the subtype, and why

Production Kotlin is **83 counted lines larger** (877 → 960 in the ten files the change
touched). The new adapter, `SubtypeSession.kt`, is 76 of them; the other 7 are the glide fix
([below](#the-glide-index-rebuilt-on-a-subtype-switch)), which hands glide the subtype each
keyboard was built for. The ledger has 404 rows for 404 production files.

| File | Counted lines, main → branch | What it does now | Why it is Kotlin |
|---|---:|---|---|
| `ime/core/SubtypeSession.kt` | new, 76 | Calls the session; names locale tags with `FlorisLocale`; reports the currency sets the keyboard loaded, when it loads a new map of them; builds the Kotlin `Subtype` and `SubtypeFacts` each row describes | `java.util.Locale`'s naming and the loaded `CurrencySet`s are Android's; the readers below take Kotlin objects |
| `ime/core/SubtypeManager.kt` | 63 → 77 | Publishes the list and the active subtype to the flows the Compose keyboard, the NLP, the settings screens and the renderer collect; makes the Jetpref writes Rust asks for (the repaired list, the active id) | Jetpref and the flows are Kotlin's. It grew by the 13-line `SubtypeStore` seam (the stored list and active id, which the traces replace with their own settings) and the active entry |
| `ime/core/Subtype.kt` | 131 → 118 | The data classes the readers and `extension.json` use; `SubtypePreset.toSubtype` | The `copy(LayoutType, …)` overload, which only the old fallback and one test used, moved into that test |
| `ime/core/SubtypeFallback.kt` | 6 → deleted | | Rust answers the fallback |
| `render/KeyGraphRows.kt` | 86 → 94 | The key graph's build row, its files named from Rust's facts; `subtypeKeyboard`, the one place keyboards are keyed | The layout and popup files are read from Android's assets and extensions |
| `render/RustKeyGraphs.kt` | 83 → 82 | Keeps and computes the Rust renderer's keyboards, each built with the subtype whose facts it came from | Its subtype lookups moved to Rust |
| `render/RustKeyboard.kt` | 104 → 105 | Reads the key graph's keys back for the input path; keeps the subtype it was built for | The glide fix |
| `render/RendererGestureHost.kt` | 81 → 83 | Hands glide the snapshot's keys with their keyboard's subtype | The glide fix |
| `ime/text/gestures/GlideTypingManager.kt` | 77 → 78 | `setLayout` takes the keys' subtype; the Compose keyboard passes none and gets the active one | The glide fix |
| `ime/keyboard/KeyboardManager.kt` | 246 → 247 | Reads the active entry once per evaluation and hands its facts to `RustKeyGraphs` | |
| `render/RendererSubtypes.kt` | 21, unchanged | The subtype sheet's ids and the language names the renderer draws | A Compose reader of the flows; the display names are `Locale`'s. It was already an adapter |

`@Serializable` stays where something still needs it. `SubtypePreset`, `SubtypeLayoutMap` and
`SubtypeNlpProviderMap` are parsed from `extension.json` (`KeyboardExtension.subtypePresets`).
`Subtype` itself is no longer serialized in production; it keeps the annotation for the frozen
copies of main's codec in the test sources (`ReferenceSettingsCodecs`,
`ReferenceSubtypeManager`, `ReferenceSubtypeEditorState`, `SettingsCodecMatchTest`), which decode
the stored list through kotlinx to check Rust's codec against it. It can go when they are retired.

The ledger's budgets were tightened to these counts. Six were raised from main's, each with its
reason in the ledger: `SubtypeManager.kt` (63 → 77, and from class M to A: it now only adapts
Rust's session), `KeyGraphRows.kt` (86 → 94, the keyboards' one cache helper and the files named
from Rust's facts), `KeyboardManager.kt` (246 → 247, one read of the active entry), and for the
glide fix `RustKeyboard.kt` (104 → 105), `RendererGestureHost.kt` (81 → 83) and
`GlideTypingManager.kt` (77 → 78). Two were lowered: `Subtype.kt` (131 → 118) and
`RustKeyGraphs.kt` (83 → 82). `SubtypeSession.kt` has a new row at 76, and `SubtypeFallback.kt`'s
row went with the file.

The tests grew by 656 counted lines (605 → 1,261 in the files the change touched): the trace
(`SubtypeTrace*.kt`, `SubtypeMatchTest.kt`), the frozen references and `SubtypeSessionTest.kt`
are new, `SubtypeFallbackTest.kt` (53) went with its file, `LayoutAssemblyMatchTest.kt` took
the `copy` overload it was the last user of, and the two tests that build a `RustKeyboard` name
its subtype.

### Proof of the subtype half

#### The differential traces

`SubtypeMatchTest` runs three seeds (20261122, 11, 31337) of 50,000 states each against frozen
test-only copies of main's code: `ReferenceKeyboardSubtypes.kt` (main's `SubtypeManager`, with its
plans) and `ReferenceSubtypeBuildRow.kt` (main's key graph build row and cache key). Main's side
and the branch's each run over their own settings, coroutine scope and queued dispatcher.

Each state is one event, fed to both:

* the settings screens' edits, written to the stored list: add (with a duplicate id, and as a
  copy of a subtype in the list), remove, modify, reorder, move, rewrite, and two edits at once;
  a third of them also store an active id;
* the language key's next and previous, a switch by id, and an active id stored from outside;
* the currency sets loaded again, the number row and number hints toggled, resources changed;
* process death, which ends both scopes and starts again from the stored settings, and process
  death mid-flight, with an edit's tasks partly run.

Then both dispatchers run their queued tasks: all of them, or in one state in eight only the
first few, so the keyboards are compared mid-flight. The same number of tasks must run on both.

After each state the trace compares:

* the stored list, the stored active id, and every write each side made, in order;
* the list's size and every subtype field by field: id, primary locale and its tag, secondary
  locales and their tags, providers, composer, currency set, punctuation rule, popup mapping,
  layouts, the whole, and its hash;
* the active subtype the same way, and the branch's active entry against its flow;
* the currency set each side finds for the active subtype and one in the list, as the same
  object;
* failures: both sides must fail alike, after which the settings are rewritten and both restart;
* the key graph: the active subtype's keyboard for a mode (the characters in two states in
  three), which both must find in their cache or both build, from the same build row field by
  field; in one state in three both compute it, with the same facts, to the same keys.

All 150,000 states were identical, in 224 s. How often each case came up:

| Case | 20261122 | 11 | 31337 |
|---|---:|---:|---:|
| add / remove / modify | 4,557 / 2,533 / 2,516 | 4,606 / 2,458 / 2,542 | 4,434 / 2,544 / 2,588 |
| reorder / move / rewrite | 1,333 / 1,254 / 1,259 | 1,260 / 1,304 / 1,292 | 1,248 / 1,244 / 1,290 |
| next / previous / switch by id | 10,078 / 4,949 / 5,005 | 9,937 / 5,040 / 5,043 | 9,919 / 5,015 / 4,929 |
| active id stored from outside | 2,510 | 2,547 | 2,589 |
| process death / mid-flight | 997 / 511 | 950 / 513 | 1,008 / 433 |
| compared mid-flight | 4,414 | 4,389 | 4,313 |
| failed alike | 39 | 50 | 46 |
| keyboard built / from the cache / computed | 26,521 / 23,440 / 16,771 | 26,116 / 23,834 / 16,686 | 26,495 / 23,459 / 16,683 |
| a right-to-left subtype in the list | 12,772 | 14,469 | 12,832 |
| lists of 0 / 5 or more | 2,081 / 30,872 | 2,522 / 30,045 | 2,241 / 31,126 |
| duplicate ids in the list | 27,708 | 26,752 | 28,877 |
| fallback / loaded currency set | 32,535 / 65,306 | 34,091 / 63,287 | 33,853 / 63,814 |

`SubtypeSessionTest` (9 tests) covers the Kotlin adapter directly, and the `imesubtype` crate's
own tests cover the session and the rows.

#### Mutations

`scripts/subtype-mutations.py` runs `theme-subtype-mutations.py`'s runner with its rules: every
target a case names is run, a mutant that does not compile is never caught, an `AssertionError`
must appear in a fresh report, and the restored source must pass. 34 faults, across
`session.rs` (11), `calls.rs` (8), the settings codec's blank list (1), `SubtypeSession.kt` (7),
`SubtypeManager.kt` (5) and `KeyGraphRows.kt` (2). The targets are the `imesubtype` and
`settings` crates' tests, and the JVM's `SubtypeSessionTest` with `SubtypeMatchTest`.

**All 34 are caught** on the final commit (4b32b51bb), and the restored source passes all three
targets. The JVM target caught 32, the `imesubtype` crate's tests 16 and the `settings` crate's
1: 15 by the crate and the JVM, 17 by the JVM alone, and two by a crate alone. Those two are
`active-asks-a-write`, a store asked when Kotlin only reads the active subtype, which Kotlin's
`SubtypeSession.active()` never acts on, and `blank-list-refused`, the codec refusing a blank
stored list, which the trace did not reach. Every mutant compiled. The result is
`build/subtype/mutations-final/result.json`.

The first run missed two, removing the repaired list's write and removing the evaluation after a
load. Both made `SubtypeMatchTest` fail, but through `check()`, an `IllegalStateException`: the
two keyboards ran different numbers of tasks. That count is a difference the trace compares, so
it is asserted now like the others (cec9a2be3), and both are caught by it.

Left out as equivalent: not naming the default subtype's tags (its only tag, `en-US`, is already
Android's name for it), and storing a switch by id before or after making it active (the store
never suspends, so nothing can tell the orders apart).

#### Digests and probes

Both digests are unchanged on the final build (4b32b51bb), from `scripts/render-benchmark/run.sh`
on `aw_keyboard` (emulator-5558) with the Rust renderer:

* the 300-press digest `809d588bf48a1e90…` (300 UTF-16 units);
* the 8-popup digest `c14d19d26f290a24…` (308 units).

The first two runs each typed 300 units with another digest, kept as
`build/subtype/render-benchmark-final-{1,2}-mismatch`. `explain-digest.py` explains both from the
digest alone: keys typed as their long-press hints (units 47 and 186, `w` and `f` as `2` and `%`;
unit 192, `f` as `%`). That is E29. The third run matched, at a load average of 4.2 to 5.2.
(Three earlier runs, `render-benchmark-final-{1,2,3}-editor-not-empty`, never typed: `run.sh`
refused to start because the CPU trials had left text in the probe's editor. The rerun sets the
probe up as `cpu-trials.py` does first.) Six of the eight CPU trials below matched both digests.
Main's third and the final build's first typed another (E29): `explain-digest.py` explains
main's (two `r`s typed as `4`), not the final build's.

The trace for E44 found what E29 had been missing on the emulator: agent-device's injector
delivers a press's DOWN and UP each late by its own amount, so its 100 ms presses reached the
keyboard as holds of 22 to 374 ms, 300 ms or more on 4 of 80. A hold past the long-press delay
(300 ms by default) is a long press. [E29](owed-checks.md) has the details.

**Probes**, on the final build:

* **"teh" and a space** (`suggestions.py`, with the device's kept-as-typed and learned-word files
  moved aside and put back afterwards): both renderers matched the autocorrect digest
  `6e5ce…0051c` (4 units, "the ") and the candidate-pick digest.
* **Reshow** (`reshow.py`): all 16 trials drew their keys, every key row matching, across the
  four ways of hiding at gaps of 100, 300, 600 and 1,000 ms (`build/subtype/reshow-final/`).
* **Reading order** (`a11y-order.py`): captured in five states (letters 62 nodes, expanded 68,
  collapsed 62, the row of words 64, a popup 74). `compare-a11y.py --order` found no ordering
  issue and exited 0 (`build/subtype/a11y-order-final/`).

#### CPU trials

`cpu-trials.py --rust-only --trials 1`, alternating main (b1cf45f18) and the final build
(4b32b51bb), three each; the full entry is in
[cpu-p95-results](cpu-p95-results.md#subtype-2026-11-22). **The 2 ms wall-clock gate was not
met, by main or by the branch. This is not a pass.**

| Trial | Frames | p50 | p95 | p99 | Over 2 ms | CPU time p50 | CPU time p95 |
|---|---:|---:|---:|---:|---:|---:|---:|
| main 1 | 1,043 | 0.996 | 3.030 | 5.259 | 124 (11.9%) | 0.852 | 1.423 |
| branch 1 | 1,108 | 0.986 | 3.395 | 6.343 | 162 (14.6%) | 0.849 | 1.610 |
| main 2 | 1,092 | 1.000 | 2.860 | 4.687 | 134 (12.3%) | 0.848 | 1.438 |
| branch 2 | 1,046 | 0.986 | 3.135 | 5.331 | 113 (10.8%) | 0.837 | 1.388 |
| main 3 | 1,063 | 1.014 | 3.459 | 6.467 | 141 (13.3%) | 0.851 | 1.585 |
| branch 3 | 1,113 | 0.910 | 3.293 | 7.553 | 125 (11.2%) | 0.799 | 1.457 |

The branch is not slower beyond noise: its p95 and its CPU time p95 lie within main's range, and
its p50 and CPU time p50 are at or below main's in every trial. Two trials typed another digest
(E29) and were run again, the branch's first and main's third. The host's load average ran from
2.5 to 8.0.

#### Device checks (emulator-5558)

On `aw_keyboard`, with the Rust renderer unless named, and the branch's build before its rebase
and the glide fix (06788615f), which has the final commit's subtype code. The device's settings
were saved first and written back byte for byte at the end (below). Screenshots are in `build/subtype/device/`, which is not
committed.

* **Two subtypes added**, then two more (French AZERTY, IPA, and a custom subtype with an Arabic
  locale on QWERTY), with the utility key on *Switch language* and the language names in their
  native locale (`01`, `02`, `03`).
* **The language key forward**, through every subtype and wrapping to the first (`04` to `07`).
  **Back** is the space bar's swipe left, set to *Previous subtype*: with a 30 to 60 ms pan it
  went from the first subtype to the last, wrapping, and back down one at a time (`21`). Main
  needed the same fast pan; slower pans (300 to 600 ms) switch on neither build, as the swipe's
  velocity threshold expects.
* **The subtype sheet**: a long press on the space bar, set to *Show subtype picker*, opens it,
  and picking the Arabic subtype makes it active (`09` to `11`).
* **Reorder**: moving the last subtype up one changed the language key's order to match (`15`,
  `16`).
* **Removing the active subtype**: the first became active, and its id was stored (`17`).
* **Restart**: after `am force-stop`, the second subtype was still active (`18`).
* **Process death**: the keyboard's process killed while it was shown came back with the third
  subtype still active (`19`).
* **A non-Latin and a right-to-left layout**: none is bundled; every preset and layout in the
  build is Latin script. IPA (phonetic letters) and the Arabic-locale subtype stood in, and both
  draw and switch. The Arabic language name draws as boxes on the Rust renderer's space bar
  (`16`) and in the settings screen's list (`15`), while the Compose keyboard draws it (`12`).
  Some of IPA's hint glyphs draw as boxes too, identically on main (`20`) and the branch (`21`).
  The renderer's fonts and the label text it is handed are unchanged here, so this is a font
  gap that predates the change; the Arabic name was not rechecked on main (owed below).
* **The Compose fallback**: with *Draw the keyboard in Rust* off, the keyboard drew and the
  language key switched, wrapping (`12`, `13`); turned back on, the Rust renderer drew again
  (`14`).

The state was read after each step only as counts and positions: how many subtypes are stored
and where the active one is in the list, never the list's text.

**Restored.** The device's settings file was written back with the keyboard stopped and checked
by its SHA-256 (`5830f04414f3eb52…`, 844 bytes): it matched after every run that changed it,
the last at 22:38. The stream volumes matched the copy taken before the trials (media muted),
accessibility matched its original by hash, the `aw-*` log tags are back at `INFO`, our IME is
the default, and the final build (4b32b51bb) is installed. The kept-as-typed and learned-word
files were moved aside for the "teh" probe and back (two files each way).

#### Switch timings, the profile, E41 and E44

On `aw_keyboard`, with the timing fixture the theme half used (English (US) and German (Germany)
subtypes, the utility key switching language), written before each run and the device's own
settings put back byte for byte afterwards. Every run below installs its build first.

##### The profile (E41)

simpleperf's software clock runs on this emulator with no root, which the theme half had not
found: `simpleperf record --app com.jtn0123.audiowhisper.keyboard -e cpu-clock -f 1000
--call-graph dwarf`. At 1,000 samples a second a sample is about 1 ms of a thread's CPU. Each
recording covers 20 mode switches or 20 subtype switches, two pairs, main (61f1513fb) against the
branch before its rebase (06788615f, the same subtype code as the final commit). Pair 1's
branch recording of the subtype switches kept 2,032 samples in 42 s, against 4,700 to 5,900 for
the others, and is not compared. `build/subtype/e41/` holds them.

* **The subtype's own work is under a millisecond a switch.** Every symbol of the subtype and
  the key graph's cache on the main thread (`SubtypeManager`, `ImeSubtypes`, `KeyGraph.call`,
  `KeyboardManager`'s reads) is 0.1 to 0.6 ms a switch on both builds, and none differs between
  them by more than 0.2 ms.
* **The main thread is mostly Compose and the renderer's refresh.** A subtype switch costs 85 to
  101 ms of main-thread CPU. The keyboard's composition recomposing is 51 to 59 ms of it, and
  `RendererAccessibility.refresh`, which parses the whole semantic snapshot at every scene while
  accessibility is enabled (agent-device's automation keeps it so), is 18 to 21% of it, 26 to
  36% in mode switches. That is the same on both builds and is owed as E48.
* **Off the main thread, glide rebuilt its index.** Bursts of `SwipeIndex.build` on
  `Dispatchers.Default`, 27 to 182 ms of CPU each, came 5 to 11 times in 20 subtype switches on
  both builds, and never in mode switches. Which build had more flipped between the pairs. That
  is the variable part of a subtype switch's cost, and it was a bug.

##### The glide index rebuilt on a subtype switch

`StatisticalGlideTypingClassifier` keeps the last five subtypes' indexes, and treats keys that
change while the subtype stays the same as a new layout for that subtype: it drops the cached
index and builds it again, from the dictionary's words. The Rust renderer handed glide each
scene's keys with the *active* subtype. A scene that still showed the last subtype's keys after
the switch paired them with the new subtype, so when the new subtype's keys arrived they read as
a new layout, and the index was built again instead of taken from the cache.

The fix (4b32b51bb) hands glide each keyboard's own subtype: a `RustKeyboard` keeps the subtype
it was built for, and `RendererGestureHost` passes it with the keys. The Compose keyboard passes
none and gets the active subtype, as before. Builds in 20 subtype switches, counted from
simpleperf the same way (`SwipeIndex.build` bursts per thread, split at 30 ms gaps):

| Build | Run 1 | Run 2 |
|---|---|---|
| main before the rebase (61f1513fb) | 10 | 9 |
| the branch before the rebase and the fix (06788615f) | 5 | 11 |
| main (b1cf45f18) | 7 (39 to 53 ms each) | 7 (40 to 147 ms each) |
| **the final build (4b32b51bb)** | **1** (41 ms) | **1** (40 ms) |

The final build's one build is in the first two seconds, the second subtype's index made the
first time it is used and then kept; main's first build is the same one, and its other six
come every few switches to the end. None came in mode switches. The same runs read the
keyboard's CPU per switch from `/proc` (main thread / all threads, ms):

| Run | Mode | Subtype |
|---|---|---|
| main 1 | 77 / 148.5 | 92.5 / 248.5 |
| final 1 | 78 / 144.5 | 95 / 214 |
| main 2 | 82.5 / 152.5 | 99 / 258.5 |
| final 2 | 82 / 154 | 97.5 / 218 |

A subtype switch now costs 34 to 41 ms less CPU on all threads than on main, and the same on the
main thread. `build/subtype/glide/` holds the recordings.

##### Where the release waits (E44)

`scripts/render-benchmark/switch-trace.py` (new) records a system trace with atrace, which needs
no root either, while it taps 10 mode and 10 subtype switches as `switch-cost.py` does. It
follows each of agent-device's DOWN and UP events from its timestamp to system_server, where an
injected event arrives (`interceptMotionBeforeQueueing`), to the input dispatcher's publish on
the keyboard's channel, to the keyboard's main thread dispatching it, and from the UP to the
app vsync tick (SurfaceFlinger's `VSYNC-app`) the keyboard's next frame ran on, and from that tick
to the frame's `Choreographer#doFrame`. The first four runs are the theme half's builds, recorded
before the rebase; the last four are main and the final build, alternating. Medians of the UP in
ms per run, 10 taps each:

| Run | Mode: stamp → system_server | system_server → dispatched | dispatched → vsync | vsync → frame | Subtype: stamp → system_server | system_server → dispatched | dispatched → vsync | vsync → frame |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| pre-theme main f4aa9fc30, 1 | 69 | 0.6 | 33 | 1.7 | 107 | 0.7 | 10 | 2.9 |
| main 61f1513fb, 1 | 91 | 0.6 | 11 | 2.2 | 84 | 0.6 | 12 | 3.2 |
| pre-theme main f4aa9fc30, 2 | 91 | 0.7 | 11 | 7.5 | 115 | 0.6 | 10 | 1.7 |
| main 61f1513fb, 2 | 137 | 0.7 | 7 | 4.0 | 52 | 0.6 | 9 | 4.7 |
| main b1cf45f18, 1 | 78 | 0.4 | 20 | 1.0 | 92 | 0.6 | 12 | 2.2 |
| final 4b32b51bb, 1 | 103 | 0.5 | 17 | 3.2 | 113 | 0.6 | 9 | 3.5 |
| main b1cf45f18, 2 | 83 | 0.6 | 15 | 2.8 | 66 | 0.5 | 12 | 2.2 |
| final 4b32b51bb, 2 | 107 | 0.4 | 88 | 0.8 | 109 | 0.8 | 5 | 3.1 |

The final runs' touch stamps were not kept, so their vsync split pairs each UP with its own DOWN's
publish instead; it needs nothing else. `build/subtype/e44/` and `build/subtype/e44-final/` hold
the traces.

* **The release waits in agent-device, not in the keyboard.** From its timestamp, the UP takes
  52 to 137 ms to reach system_server, varying that much between runs of one build. From there it
  is published to the keyboard and dispatched on its main thread in under a millisecond. While it
  waits the keyboard's main thread is asleep. agent-device runs each press as a new
  instrumentation process (its `TouchPlanInjector`), which stamps the press's events before it
  injects them.
* **The frame after it waits for the emulator's vsync, not the keyboard.** The keyboard asks for a
  frame as it dispatches the UP (`Choreographer#scheduleVsyncLocked`) and runs it within a few
  milliseconds of the next `VSYNC-app` tick. The median wait for the tick is 5 to 33 ms in every
  run but one, and on some taps it was 63 to 162 ms, on both builds: of the ten mode taps, four
  and two in main's final runs, three and six in the final build's, which is that run's 88 ms
  median; of the subtype taps, one in the final build's first run. The keyboard traces nothing in
  those gaps. The mutation run was using
  the host's CPU while the final four runs were recorded.
* **The timings' "input" measures that wait.** `switch-cost.py`'s input is the release's touch
  to its frame's callback. For a mode switch the release's own frame is replaced by the rebuilt
  keyboard's, so the frame it reads carries the press's touch, and its input includes the 100 ms
  hold as well.
* **E44's mode gap is that wait.** The pre-theme main (f4aa9fc30) and the main the theme was
  merged into (61f1513fb) differ only by the injector's lag, which crosses over between runs.

So E44 is closed: it asked where the release waits, and it waits in the tap injector. What the
keyboard itself does from the release, which a phone's finger would show without the injector,
goes to P30.

##### Switch timings

The final runs, on main (b1cf45f18) and the final build (4b32b51bb), alternating, three each,
the final build first, after the traces above. Each run installs its build, writes the fixture,
and runs `switch-cost.py` then `rebuild-timing.py`; 20 switches of each kind each. Every run saw
20 of 20 taps and switches. `build/subtype/timing-final/` holds them.

`rebuild-timing.py`, from the tap's release to the last frame the renderer posted for it, p50 /
p95 / max in ms, and the host's load average at the start, between the two scripts and at the
end:

| Run | Mode | Subtype | Load |
|---|---|---|---|
| final 1 | 319.9 / 479.4 / 563.6 | 248.8 / 341.3 / 410.9 | 3.5 → 4.8 → 7.6 |
| main 1 | 318.0 / 413.7 / 484.5 | 234.1 / 438.2 / 438.9 | 7.6 → 7.0 → 5.0 |
| final 2 | 313.8 / 405.9 / 441.8 | 258.3 / 406.3 / 431.7 | 5.0 → 3.6 → 3.9 |
| main 2 | 348.0 / 404.6 / 463.3 | 255.2 / 341.7 / 405.0 | 3.9 → 4.3 → 3.4 |
| final 3 | 351.4 / 478.2 / 557.3 | 263.8 / 354.2 / 362.8 | 3.4 → 5.0 → 3.0 |
| main 3 | 346.4 / 453.0 / 488.9 | 283.5 / 393.0 / 416.3 | 3.0 → 2.9 → 2.7 |

`switch-cost.py`, CPU per switch in ms (main thread / all threads), and each tap's split, medians
in ms (input / build / draw):

| Run | Mode CPU | Subtype CPU | Mode split | Subtype split |
|---|---|---|---|---|
| final 1 | 77 / 145.5 | 92.5 / 209.5 | 149.2 / 49.8 / 112.3 | 100.5 / 63.5 / 63.8 |
| main 1 | 39.5 / 79.5 | 107.5 / 264 | 105.3 / 94.7 / 52.2 | 128.8 / 52.9 / 70.6 |
| final 2 | 71 / 137 | 85 / 191.5 | 94.8 / 64.4 / 109.4 | 103.7 / 59.2 / 57.9 |
| main 2 | 75 / 146.5 | 88.5 / 239.5 | 153.5 / 27.0 / 119.8 | 141.0 / 65.5 / 69.5 |
| final 3 | 60.5 / 112 | 93 / 206.5 | 117.2 / 64.1 / 80.5 | 143.8 / 55.1 / 77.6 |
| main 3 | 72.5 / 132.5 | 97 / 268 | 112.5 / 39.3 / 104.6 | 158.5 / 61.8 / 74.5 |

* **A subtype switch costs less CPU than on main, in every pair.** On all threads, 191.5 to
  209.5 ms against main's 239.5 to 268, 48 to 62 ms less a switch in each pair: the glide index
  no longer rebuilds. On the main thread, 85 to 93 against 88.5 to 107.5, 3.5 to 15 ms less in
  each pair.
* **Subtype switches are level in wall time**: 249 to 264 ms at p50 against main's 234 to 284,
  each build above the other in some pair.
* **Mode switches are level too**, which they were not in the theme half's runs (E44): 314 to
  351 ms at p50 against main's 318 to 348, each above the other in some pair. Their CPU is
  level, 60.5 to 77 ms on the main thread against main's 72.5 and 75, except main 1's, about half
  of every other run's on both threads. Its process started with
  the run and stayed up, its 20 switches drew, and its subtype phase was ordinary; nothing found
  explains it, so it is not compared.
* The split moves by tens of milliseconds between runs of one build, in every column, as in the
  theme half. That is the injector's lag and the late vsync above, not the keyboard.

So E41 is closed as well: its gap was not in the subtype's work, which is under a millisecond a
switch, and the one real cost found, glide's rebuilt index, is fixed. The phone's own timing of
both switches is P30.

#### The gate

On the branch's final commit (these docs; its last code commit is 9517a7440): `cargo fmt`, then
`./gradlew --stacktrace test :app:lintDebug :lib:rust:check :app:checkNativeDeclarations
:app:checkJvmLedger assembleDebug :app:compileDebugAndroidTestKotlin
:app:compileDebugAndroidTestJavaWithJavac`. It passed.

* **JVM:** 1,024 tests in 251 files, with 0 failures, 0 errors and 0 skipped. Main has 1,017 in
  250: `SubtypeMatchTest` (the trace) and `SubtypeSessionTest` (9 tests) are new, and
  `SubtypeFallbackTest` (3) went with `SubtypeFallback.kt`.
* **Cargo:** 1,535 tests, 0 failed and 0 ignored (main: 1,523): 11 new in `imesubtype` and one
  in `settings`, for the codec's blank list.
* **Natives:** 430 generated, 0 hand-written, 0 missing and 0 mismatched (main: 427).
* **Ledger:** 404 rows for 404 production files, the same as main; `--tighten` found nothing
  to lower.
* **Lint:** only the allowed IDs: AndroidGradlePluginVersion ×4, NewerVersionAvailable ×4,
  ChromeOsAbiSupport ×1 and GradleDependency ×1.

### What is owed from the subtype half

Added to [owed-checks](owed-checks.md):

* **P30**: the subtypes on the phone, with its own subtypes and a language pack's non-Latin or
  right-to-left layout, and the mode and language keys timed with a finger on main's build and
  this one.
* **E46**: the Rust renderer draws an Arabic subtype name, and some IPA hint glyphs, as boxes.
  Check the name on main and find the renderer's font fallback.
* **E47**: no bundled layout is non-Latin or right-to-left. Install one of each and add, draw,
  switch to and type on it, on both renderers.
* **E48**: `RendererAccessibility.refresh` parses the whole semantic snapshot on the main thread
  at every scene while accessibility is enabled, 18 to 28 ms a switch.

E41 and E44 are closed by this half (above), and the [notes](owed-checks.md#notes) say so.

### What item 4 needs from the subtype side

The subtype side is ready for the plain-View root. Nothing it needs is Compose's.

* `SubtypeManager` and `SubtypeSession` have no Compose dependency. They publish the list and
  the active subtype on `subtypesFlow` and `activeSubtypeFlow`, which a plain collector can read.
* `RendererSubtypes.kt` is the one Compose reader on the Rust renderer's side.
  `rendererSubtypes()` collects the two flows with `collectAsState` and names each subtype with
  `Locale`'s display name. The root replaces it with a collector of the same flows, and the
  display names stay with Android's `Locale`.
* The key graph's facts come from `subtypeManager.active` in `KeyboardManager`, not from the
  composition, so the root changes nothing there.
* The Compose fallback's own subtype readers go with the Compose keyboard at step K: its
  `keyboardCache`, keyed by `Subtype`, and `TextKeyboardLayout`'s glide `setLayout`.
* `SettingsSubtypes.plan` stays with the settings screens, which edit the stored list. The
  keyboard's session reloads when the list changes.

## What remains

**Of item 3.**

* **The palette.** The keyboard's Material You palette is still made by materialkolor in
  Kotlin (`themePalette`, `SnyggPalette`), from the primary colour Rust picks. Porting the scheme
  maths to Rust would let `rendererTheme` hand over only the facts.
* **The theme's data model.** `ThemeExtension` and `ThemeExtensionComponent` stay with the
  extension manager, which still reads `extension.json` in Kotlin. `ThemeMode` is a Jetpref
  setting's type.
* **The subtype's data model.** `Subtype`, `SubtypePreset` and their maps stay Kotlin data
  classes: the readers take them, and the presets are parsed from `extension.json` in Kotlin.
* **The references.** `ReferenceThemeManager` and `ReferenceFlorisImeThemeBaseStyle` stay in
  the test sources while the theme traces run against main, and `ReferenceKeyboardSubtypes` and
  `ReferenceSubtypeBuildRow` while the subtype traces do.

**For item 4, the plain-View root.** The subtype's side is in [what item 4 needs from the
subtype side](#what-item-4-needs-from-the-subtype-side); the theme's:

* `FlorisImeTheme.kt` hosts the Compose root and provides the active theme to the Compose views.
  The plain-View root takes the active theme from `ThemeManager`'s flow, which is fed by Rust's
  decisions now, without Compose.
* Snygg backgrounds: the window's and the root's background come from the active theme. Rust
  has the stylesheet and resolves the renderer's styles from it, so the root's background can be
  asked of Rust the same way the chips' colours are.
* Font changes: a theme's fonts and images are in the unpacked directory `ThemeInfo` keeps on
  the Android side, by the slot Rust names.
* Live inline autofill: the chips' style bundle no longer needs a Compose composition. Rust
  answers it at request time for the keyboard's state, so the plain-View root needs only the
  `InlineContentView`s. The live check on a phone with an autofill service is P28.
* `FlorisImeUi.kt` and `lib/snygg/SnyggTheme.kt` with `lib/snygg/ui/*` go with the Compose views
  that use them.
