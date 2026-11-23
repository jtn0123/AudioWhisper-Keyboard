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

**The subtype is written up, not moved.** Most of it was already Rust's before this item (below),
and what is left is entangled with the key graph's build row and with E41. The brief's stopping
point applies: the theme is done with its proof, and [what remains](#what-remains) says exactly
what the subtype still needs.

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

## Subtype: what is already Rust's

The subtype was not moved in this item. Most of it was Rust's before it:

* **Rules.** `SettingsSubtypes.plan` sends each subtype operation to `SettingsRules` (crates
  `settings`, `subtype.rs`): `load` (the list from the settings, repaired, and whether to write
  it back), `evaluate` (the active subtype, and whether to write its id back), `next` and
  `previous` (the language key). The settings screens' add, remove and reorder go through the
  same plan.
* **The codec and the defaults.** `SettingsCodecs.subtypes` and `SettingsCodecs.subtype` read
  and write the stored list in Rust; `SettingsRules.defaultSubtype()` is the default subtype,
  its layout map, its NLP providers and its punctuation rule.
* **Lookups.** `SettingsOp.SUBTYPE_INDEX` finds a subtype by id; `SettingsOp.LAYOUT_MAP` writes
  a layout map's string form.

What Kotlin still holds:

* **`SubtypeManager.kt`** (102 nonblank lines in the inventory): the two flows (the list, the
  active subtype), the Jetpref writes each plan asks for, `switchToSubtypeById`, and
  `getCurrencySet` (a lookup in the keyboard's loaded currency sets, else the fallback).
* **`Subtype.kt`** (177): the `@Serializable` data classes, `SubtypeLayoutMap.get(LayoutType)`
  and its `copy`, and `SubtypePreset.toSubtype`. `SubtypeSelectionState.kt` (4) is a two-field
  holder.
* **The key graph's subtype facts** (`RustKeyGraphs`, `KeyGraphRows`): the cache key
  (`subtype.hashCode()`), the layout names (`subtype.layoutMap::get`) and the popup mapping the
  build row names, the language's display name (Android's `Locale`), the number of subtypes
  (for the language key), and the currency set's slots.
* **`RendererSubtypes.kt`**: the subtypes' ids and display names for the renderer.

So what moving the subtype would change is the shape between Kotlin and Rust, not the rules: Rust
would hold the list and the active subtype and answer what each subtype names, and the key graph
would take the subtype's facts from Rust rather than from `KeyGraphRows`. That is entangled with
the key graph's build row and with E41's cost, so it is left for its own change ([what
remains](#what-remains)).

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
`--app` profiling finds no supported event. `switch-cost.py` (new) measures instead, tapping as
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
* **E29**, updated: a tap read as a long press, found from the digests alone for 20 of the 24
  wrong ones kept in this worktree; why, and the other four, are still owed.
* **E30**, updated: the parse error came back once, at a load average of 43.
* Not device checks: the subtype's move (below), the palette maths in Kotlin, and retiring
  `ReferenceThemeManager` and `ReferenceFlorisImeThemeBaseStyle` with the theme traces.

P4, the phone p95, stays owed, and E28, the CPU gate on a quiet host.

## What remains

**Of item 3.**

* **The subtype.** Rust already has the rules and the codec. Moving the rest means Rust holds
  the subtype list and the active subtype and answers what each one names: its layouts, popup
  mapping, currency set, composer and display name. Kotlin would then keep:
  * the Jetpref writes Rust asks for;
  * Android's `Locale` for the display name;
  * the loaded currency sets.

  The key graph's build row would take the subtype's facts from Rust instead of from
  `KeyGraphRows`, and `SubtypeManager.kt` (102 lines), `Subtype.kt` (177) and
  `RendererSubtypes.kt` would shrink to adapters. It touches the key graph's build row and the
  subtype switch E41 is about, so it should come with the step-level profile, on the phone or a
  rooted image.
* **The palette.** The keyboard's Material You palette is still made by materialkolor in
  Kotlin (`themePalette`, `SnyggPalette`), from the primary colour Rust picks. Porting the scheme
  maths to Rust would let `rendererTheme` hand over only the facts.
* **The theme's data model.** `ThemeExtension` and `ThemeExtensionComponent` stay with the
  extension manager, which still reads `extension.json` in Kotlin. `ThemeMode` is a Jetpref
  setting's type.
* **The references.** `ReferenceThemeManager` and `ReferenceFlorisImeThemeBaseStyle` stay in
  the test sources while the theme traces run against main.

**For item 4, the plain-View root.**

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
