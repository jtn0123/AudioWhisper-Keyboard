# Phase 3: the Compose-free render path

[Phase 3, item 4](phase3-view-root.md) gave the Rust renderer a plain-View root, with no
composition. That root still reached Compose in three ways: the icons it drew were Compose
`ImageVector`s, flattened by Compose's `PathParser`; the root and the theme named Compose types
(`IntSize`, `LayoutDirection`, `mutableStateOf`, `Color`, `toArgb`); and the Material You palette
came from materialkolor through `SnyggPalette`. A theme's own fonts were not drawn at all.

**Now** no compiled class in `com.jtn0123.audiowhisper.render` names `androidx.compose`, and the
gate keeps it that way (`:app:checkComposeFreeRender`):

* **Icons.** Rust holds every icon the renderer draws (`crates/icons`). Kotlin sends a token or
  nothing.
* **The palette.** Rust makes the Material You scheme (`crates/palette`). Its answers match
  materialkolor's bit for bit.
* **Compose state and types.** The root reads StateFlows, `android.util.Size` and a flag for
  the layout direction.
* **Theme fonts.** The renderer draws a theme's `@font` files through parley, falling back to
  today's chain.

The Compose fallback still runs with the renderer off. It was changed only mechanically: it
collects the StateFlows that replaced Compose state.

Branch `all-rust/compose-free-path`, rebased onto main 0f0b22d28. Its commits:

| Commit | What |
|---|---|
| c5898113c | `mutation_cases.py`: an edit with no search text makes its file |
| 1ceb0a139 | Rust holds every icon the renderer draws, matching Android's flattening (`IconGoldenDump`, `IconVectors`) |
| 8859a47df | Every key token draws the icon Android drew for it (`tests/fixtures/keys.tsv`) |
| a044dac8e | The settings screens draw their icons from Rust; `SettingsIcons.kt` deleted |
| 85add5810 | The keyboard sends icon tokens; `RendererIcons.kt`, `RustKeyPreview.kt` and `KeyPreviewMatchTest` deleted |
| fd025f39b | `crates/palette`: Material You schemes, bit-exact with MaterialKolor; `sin` and `cos` ported from bionic's fdlibm |
| 40f10672a | The icons crate's `NOTICE` |
| dc9637739 | The keyboard's Material You palette comes from Rust; `SnyggPalette.kt` deleted |
| cd58c585c | The plain-View root reads no Compose state or types |
| acde14b12 | The renderer draws a theme's `@font` files first, then today's chain |
| e49d9c5fd | The theme's font files cross the generated boundary |
| 8c2bd56c0 | Kotlin resolves the theme's `@font` files (`RendererThemeFonts.kt`) |
| 1899f8be6 | The theme-fonts mutation runner (27 faults, all caught) |
| b1fe69971 | Docs: a theme's own fonts in the renderer |
| 75e0abd66 | Ledger row for `RendererThemeFonts.kt` |
| 3036760fb | The session font tests draw the extract view's action label |
| 63108dc5c | A key's centre from its sides, not `FlorisRect`'s Compose `Offset` |
| c3afe2d11 | The service drops its unused Compose import |
| 80f0b9351 | `:app:checkComposeFreeRender` |
| bdd0e62e9 | `compose-free-mutations.py`: ten ways to break the check |
| 7b43ddf4c | The plain-View root allocates no `Size` on each layout |
| 24b27a7a2 | `PaletteArtDiff`: Rust's palette against materialkolor on ART |
| 2d7960cc2 | `compose-free-device.py`: the render path's device checks |
| 448521911 | Docs: this record, the trials and the owed checks |
| 828a13e2f | 16 `sin` and `cos` faults, each caught, and the three golden rows they needed |
| 41555b85e | The androidTest APK carries kotlin-test, which `GoldenTape` calls |
| e4b8b792b | The gate builds the androidTest APK, R8 included, instead of compiling its sources |
| 89e92c129 | The strip keeps its words while their refill is due (E56) |
| 7507b413a | Docs: how `sin` and `cos` were checked against bionic's C |
| 079a22a96 | The stale typing-plan cases point at the Rust that holds them |
| 771ca5eff | Docs: E56 closed |
| e5762249d | Docs: the mutation cases this branch's lines moved |

The trials and probes below name the builds they ran on, before the rebases: 51a7be022 is now
7b43ddf4c, and e75a095eb is 24b27a7a2. Main's build in them is 4d3b5d3ae.

## Part 1: icons Rust holds

**Before:** `RendererIcons.iconPath` flattened each icon in three steps: it took a Compose
`ImageVector` (a Material icon from `material-icons-extended` 1.7.8, or one of the app's vector
drawables), ran Compose's `toPath`, then ran Android's `Path.approximate(0.25)`. It sent the
points as text in a 24-unit box. The settings screens did the same through `SettingsIcons.kt`.

**Now** `lib/rust/crates/icons` holds those 130 vectors as Compose's path nodes
(`data/sources.tsv`). It flattens them with ports of each step:

* Compose's `toPath`;
* Skia's `SkPath`, including its arcs and conics;
* hwui's `approximate(0.25)`;
* Kotlin's `Float.toString`.

The renderer asks the crate for a path by token or by name:

* **Keys and panel actions:** the key's icon token, as the key graph picks it.
* **The Smartbar:** the token Rust's action plan picks (`SmartbarActionPlan.DRAWN`). New wire
  tokens 1007 to 1014 name the icons only the Smartbar and clipboard candidates draw.
* **Clipboard candidates:** `ClipboardSuggestionCandidate.iconToken`.
* **Panels:** the emoji, clipboard, GIF and search panels, the subtype sheet, the one-handed
  panel and the incognito watermark draw fixed icons Rust holds, so Kotlin sends none.
* **Settings:** the screens name an icon (`@icon:mail`) and Rust looks it up in the crate's
  settings table, whose 81 names are `SettingsIcons.kt`'s.

**Deleted:** `RendererIcons.kt`, `SettingsIcons.kt`, and `RustKeyPreview.kt` (which had no
caller), with the `keyPreview` and `previewSize` natives. `smartbarConfig` moved unchanged into
the Compose `Smartbar.kt`, its last caller.

**Licences.** The crate's `NOTICE` records the Material icons' and drawables' Apache-2.0 and the
ported Skia's BSD-3-Clause. The licences screen lists Skia (`config/libraries/skia.json`).

**Proof:**

* **Every icon, on a device.** `scripts/icon-golden.sh` runs `IconGoldenDump`, a verbatim copy
  of 29cb06dcd's `iconPath`, on an Android device. For each icon it records the point count and
  hashes of the text and of the raw float bits. `icons/src/golden.rs` holds the crate to that
  file. All 130 icons (4,445 points) match: the points bit for bit and the text digit for digit.
  A second run reproduced the file byte for byte. The dump also checks that the device's vectors
  are the file's, number for number; `IconSourcesTest` does the same on the JVM.
* **Every key token.** For each token from -10,000 to 2,000, the dump records which icon
  `keyIconVector` drew. Rust's `for_key` gives the same icon for all 38 tokens that draw one, and
  none for any other token except the eight new Smartbar and clipboard tokens.
* **The Smartbar.** `SmartbarActionsMatchTest` compares each action's drawn token with the
  token main's icon maps to, over 20,000 seeded Smartbars.
* **Settings.** `SettingsResourcesTest` reads the 81 names from the crate. The four "icon
  unmapped" mutation cases now remove a pair from that table.

## Part 2: no Compose state, type or palette

### The palette (`lib/rust/crates/palette`)

The crate ports material-color-utilities' HCT and scheme maths, by way of MaterialKolor 5.0.1:

* every palette style (9), contrast level (4) and spec version (2);
* light and dark;
* the 35 colour roles `SnyggPalette` answered.

Bit-exactness needs Java's maths, not Rust's `f64` methods:

* `pow`, `exp`, `cbrt`, `hypot` and `atan2` are fdlibm 5.3 ports, as `java.lang.StrictMath` and
  HotSpot's non-intrinsic `Math` are.
* `sin` and `cos` port Android's libm (bionic), which ART's `Math` calls: FreeBSD msun's
  `s_sin.c`, `s_cos.c`, `k_sin.c`, `k_cos.c` and `e_rem_pio2.c`, fdlibm 5.3 as FreeBSD revised
  it. They are evaluated as Android builds bionic, with `-ffp-contract=off`, so nothing is fused.

So every role is the ARGB word MaterialKolor gives on ART, where main's keyboard made its
palette, and on the desktop JVM that runs the gate's tests. See [the decision](decisions.md#the-keyboards-material-you-palette-is-rusts-matched-to-materialkolor-on-art-2026-11-23)
for why the target is ART, and what matches by construction rather than by measurement.

`ImeThemeRule.PALETTE` (`crates/imetheme`) replaces `PRIMARY`, which only chose the accent:

* **Takes:** the accent row (the theme's accent setting, else the system accent) and the theme's
  `ImeThemeMaterialYou` row (style, contrast, spec).
* **Answers:** the 35 role names, then the 35 light and the 35 dark colours.

The plain-View root, the inline chips' bundle and the theme editor's swatches read it through
`themePalette`. `lib/snygg`'s `SnyggPalette.kt` is deleted. `materialYouScheme` moved back into
`SnyggUi.kt` for `ProvideSnyggTheme`, the frozen fallback.

**Licences:** the crate's `NOTICE` covers material-color-utilities (Apache-2.0), MaterialKolor
(MIT) and fdlibm. The licences screen gains material-color-utilities and fdlibm (with its own
licence text).

### Compose state and types out of the root

* **`FlorisImeService`:**
  * its full-screen and extract-view flags and its localized context were `mutableStateOf`.
    They are StateFlows now: the Compose root collects them (`collectAsState`, a mechanical
    change to `FlorisImeUi` and `FlorisImeViews`), and `ImeRootView` reads them on each refresh;
  * the input view's size is an `android.util.Size`, which both roots set;
  * the service imports no Compose. Its last import, `padding`, was unused on main too.
* **`ImeRootView`** sets the layout direction through `KeyboardState.isRtl`, a flag over the
  same bit as Compose's `LayoutDirection`, so it names neither `IntSize` nor `LayoutDirection`.
* **`RendererTheme`, `ImeRootReads` and `ThemeManager`** carry colours as ARGB `Int`s.
* **`RustKeyboardView`** takes a key's centre from its sides. `FlorisRect.center` is a Compose
  `Offset`, a reference the check found that no import showed.

### Where materialkolor stays

* **`SnyggUi.materialYouScheme`** and `ProvideSnyggTheme`: the frozen Compose fallback.
* **`lib/color`'s `ColorMappings`:** the app theme's `neutralDynamicColorScheme`, for the
  Compose screens that are left.
* **The `MaterialYouFlags` enum types** (`PaletteStyle`, `Contrast`, `ColorSpec.SpecVersion`):
  the theme config stores their names, and the theme editor lists them. Rust receives the names.
* **Tests:**
  * `ReferenceSnyggPalette`, main's palette kept verbatim for `PaletteMatchTest`;
  * `InlineChipMatchTest`, `ThemeMatchTest` and `ThemeTrace`, through the Snygg references;
  * `PaletteArtDiff` on the device.

The Snygg references main kept for this branch (`ReferenceSnyggTheme`,
`ReferenceSnyggAppearanceValue`, `ReferenceSnyggPropertySetEditor`, `ReferenceSnyggSpec`,
`ReferenceSnyggValueBridge`) were kept green, not replaced.

**Proof:**

* **The gate.** `PaletteMatchTest` holds `themePalette` (Rust) to `ReferenceSnyggPalette`
  (materialkolor) on every run. It covers 200 accents (17 fixed: black, white, greys, primaries,
  near-greys and Material's defaults; the rest seeded) × 72 flag sets × light and dark: 28,800
  schemes, every one exact.
* **The 50,000-seed run** (`-PpaletteSeeds=50000`): 50,000 accents × 72 flag sets × light and
  dark, 7,200,000 schemes, every one exact, in 1,004 s.
* **The palette crate's own fixture:** digests over 2,000 seeds per style, contrast and spec. A
  JVM harness kept outside this repository wrote them, and compared 7.92 million schemes with no
  difference.
* **On ART** (`scripts/palette-art.sh`, `PaletteArtDiff` on emulator-5558, `aw_keyboard`,
  Android 16, arm64): 1,000 seeded accents × 72 flag sets × light and dark, 144,000 schemes,
  every one exact. No role differs between Rust's answer and materialkolor's on the device's
  ART, where main's keyboard made its palette. It ran on the bionic port, with the androidTest
  dependency 41555b85e adds, without which R8 stops the test APK. The phone's own libm is still
  to be compared (P36).
* **`sin` and `cos` against bionic's C:** the five msun files built with `-ffp-contract=off`,
  276,000,041 arguments, no result differs: 200 million uniform in [-9, 9], 24 million on
  either side of every branch boundary, 12 million within 200,000 ulps of each multiple of π/4
  up to 15, 40 million random bit patterns from 2^-60 to the medium range's end (2^20·π/2), and
  the specials. Built with `-ffp-contract=on` instead, 980,295 of 60,000,041 differ, so the
  arithmetic must not be fused. `trig.rs` pins 30 arguments' answers from that C, and a hash of
  200,001 more plus 3,096 within 64 ulps of multiples of π/4.
* **Mutations** (`scripts/palette-mutations.py`): 16 of 16 faults in `sin` and `cos` fail
  `cargo test -p palette`: fdlibm 5.3's polynomial order, a kernel fused, a reduction round or
  cancellation guard dropped, the quadrants swapped, a constant one ulp off and the tiny cut-offs
  moved. Three of them (a moved cut-off for sin, a last bit off pio2_1t, no third round) first
  survived the sweep, and each gave a golden row. Fusing a step of the reduction is not a case:
  it changed no answer in 550 million arguments.

## Part 3: a theme's own fonts

Rust draws a theme's `@font` files (commits b941d8a55, 2b0302015, d5c33bdcf, 53252e9b3,
fa093cb28, 1eaee84b6 and e0318b6a3; the full record is in
[rust.md](rust.md#a-themes-own-fonts-2026-11-23)).

* **Kotlin's part.** It resolves each `@font` rule's `src` through the theme's asset resolver
  (`RendererThemeFonts.kt`) and passes the URIs and paths with the theme.
* **Rust's part.** `render::theme_fonts` reads each file once, checks it is a whole font, and
  registers each distinct family.
* **Drawing.** An element whose `font-family` names the rule draws in that family first, then in
  today's chain for any glyph it lacks.

**Proof:**

* **Fixture font.** `render/tests/theme_fonts/` draws with "Fixture Script", a renamed Latin
  subset of Dancing Script (OFL).
  * The key changes 1,957 pixels and the Smartbar 1,594.
  * The chosen-face report gives "Hello " in Fixture Script Regular, and "7?" (which the font
    lacks) in Roboto Light.
  * Bold picks the bold face.
* **No fonts.** A theme without fonts draws byte-identical: no golden was re-recorded, and the
  default chain draws byte-identical after a theme font is registered.
* **Broken fonts.** Nine broken files each read as no font and draw pixel-identical to no font,
  without a panic. One broken face leaves the rest of its family.
* **Mutations.** `scripts/theme-fonts-mutations.py` catches 27 of 27 faults.
* **Through the session.** The awnative and JVM font tests drew a key preview. They now draw the
  landscape extract view's action label (`extracted-landscape-input-action`), because the key
  preview is gone.
* **On the emulator** (`compose-free-device.py run … fonts`, emulator-5558): the fixture day
  theme, then the same theme in turn with Fixture Script, with a truncated font file and with a
  missing one, then plain again.
  * With Fixture Script, 20,139 pixels of the keyboard differ from the plain theme. The letters
    draw in the script face, and the digits and symbols in Roboto.
  * The broken and the missing file each differ from the plain theme by 0 pixels, and from
    main's build in the same pose by 0. Neither crashed.
  * On main's build all three cases differ by 0, because main draws no theme fonts. That makes
    main's font case fail by design.
  * The first branch run had the keyboard's top at 1429 instead of 1430 for 30 poses in a row
    (E53), so its font cases did not line up with the plain shot. The script now compares a case
    only with a plain shot whose window has the same frame and top, and fails a case that has
    none.

## The check: no Compose in the render package

`scripts/compose-free-check.py`, run as `:app:checkComposeFreeRender` (in `check`, the gate
line, `.github/workflows/android.yml` and the README):

* **What it reads.** The constant pool of every class file under
  `com/jtn0123/audiowhisper/render`, in the debug Kotlin and Java class directories. Nested,
  lambda and synthetic classes count, and so do the `ImeRoot*` files.
* **How a class is known.** By the source file its `SourceFile` attribute names, not by its own
  name. A file's nested, lambda and differently named classes go with it: `ImeRootReads.kt`
  compiles to the class `RootReads`, and is still an `ImeRoot` file.
* **What fails.**
  * Any string holding `androidx/compose/`: a class reference, a descriptor, a generic
    signature, Kotlin metadata, or a string literal.
  * A missing directory, a package with no class, a run with no class from an `ImeRoot` file, or
    a class with no `SourceFile` attribute. The check never passes on nothing.
* **The one allowance.** The Compose compiler plugin stamps every class of the app module with
  `@StabilityInferred`, binary retention, which no source line writes. That exact descriptor is
  allowed. It goes when step K takes the plugin out of the module.
* **Exclusions:** none. `EXCLUDED` is keyed by source file name. The frozen Compose fallback files
  (`FlorisImeUi`, `FlorisImeViews`, the Compose keyboard and Smartbar) live outside this package,
  so it is empty, and the script refuses any `ImeRoot` file in it.

The first run found two leaks that no import showed:

* `RustKeyboardView` reached Compose's `Offset` through `FlorisRect.center`;
* the plugin's stamp, which is now the allowance above.

**Mutations:** `scripts/compose-free-mutations.py` makes ten faults, and each must fail the
task itself (a compilation failure never counts) with a problem naming the class:

* **The root:** the layout direction through Compose; a Compose `Color` in `ImeRootReads`; an
  anonymous `Runnable` holding an `IntSize`.
* **The theme:** `mutableIntStateOf`; a function taking an `IntSize`.
* **The keyboard:** the key's centre through `FlorisRect.center` again.
* **A new file** that uses `Dp`, which is checked without being listed anywhere.
* **The check itself:** `ImeRootView.kt` added to `EXCLUDED`; `ImeRootReads.kt` added to it,
  with the Compose `Color` back in, which a guard keyed on class names let through before the
  check was keyed by source file; the Kotlin classes directory renamed in Gradle.

The run of 2026-11-23 caught 10 of 10, and the restored source passed again
(`build/compose-free/mutations/result.json`).

**The smartbar's typing-plan cases.** Three of `typing-plan-mutations.py`'s cases named Kotlin
lines of `RendererSmartbar.kt` that are gone: `key-label-dropped` and `disabled-keys-enabled`
since `c66e98a04`, and `arrow-vector-swapped` since this item drew icons from tokens. They now
mutate the Rust that holds that logic: a key action's label dropped in `smartbar_actions.rs`'s
`encode`, its key's enabled flag ignored in `decode_action`, and the left arrow drawn as the
right one in `icons::smartbar_token`. Run by name with `listening-when-finishing`, the one
Kotlin case left there, all 4 were caught (`SmartbarActionsMatchTest` caught each, the Rust
smartbar tests caught the dropped label), and the restored source passed, before the rebase
onto 0f0b22d28 and again after it. All 41 cases apply, and the two that waited in the script's
`STALE` list for this branch left it in the commit that repoints them.

**The cases this branch's lines moved.** Main's `:app:checkMutationCases` found five more, and
each now follows its line in the commit that moved it. Run by name, each was caught and each
restored source passed (`build/repointed-0f0b/`):

* `key-graph-mutations.py`'s `key-icon-on-every-action` and `key-icon-token-lost`: the key
  icon token and the drawn token Rust picks from it in `smartbar_actions.rs`, where Kotlin's
  `keyIcon` read it before. The Rust smartbar tests caught both, `SmartbarActionsMatchTest` the
  lost token.
* `settings-ui-mutations.py`'s `licence-not-monospace` (the project licence asks for
  `Font::Mono` now) and `theme-code-described-icon-unspecified` (the key recorder sends a token
  as text): `settings_ui_project_licence` and `SettingsThemeCodeMatchTest`.
* `theme-subtype-mutations.py`'s `accent-not-sent`: the accent Rust is given, caught by the
  theme traces.

The four scripts this branch adds give `check_edits()` and take `--check`. Compose-free's
`new-file-leaks` makes a file, so `mutation_cases.py` now reads an edit with no search text as
making its file, which applies while the file does not exist (three unit tests). At every
commit of the branch the check passes; at its head, 32 scripts and every case.

## Proof on the whole

**The gate**, on e5762249d's tree (`--stacktrace test :app:lintDebug :lib:rust:check
:app:checkNativeDeclarations :app:checkJvmLedger :app:checkMutationCases
:app:checkComposeFreeRender assembleDebug :app:assembleDebugAndroidTest`, every task rerun):
green in 15 m 48 s, at a load average of 3.7 at its start and 7.8 at its end. The commit after
it changes only Markdown.

* JVM: 1,059 tests in 262 result files, 0 failures, 0 errors, 0 skipped (main 1,052 in 259).
* Cargo: 1,650 passed, 0 failed, 0 ignored (main 1,609).
* Natives: 464, none hand-written and none mismatched (main 466; `keyPreview` and `previewSize`
  went with the key preview).
* Ledger: 405 files, 405 rows (main 408).
* Lint: only the allowed IDs (AndroidGradlePluginVersion ×4, NewerVersionAvailable ×3,
  ChromeOsAbiSupport ×1, GradleDependency ×1). Its first run on this path found one more,
  DrawAllocation: the root made a new `android.util.Size` in `onLayout`, where Compose's
  `IntSize` had been a value class. 7b43ddf4c makes one only when the size changed.
* Mutation cases: 32 scripts, every case applies; the check's own 32 unit tests pass.
* The check: 96 classes in the render package, 23 of them from `ImeRoot` files, none naming
  Compose; no file excluded.
* The androidTest APK: built and shrunk, `minifyDebugAndroidTestWithR8` included, so main's
  `GoldenTape` in the test sources finds kotlin-test (41555b85e). On main's tree the same task
  fails on the missing `kotlin.test.AssertionsKt`. The gate used to compile those sources
  without building the APK, so it never noticed; e4b8b792b makes it build the APK.

**Digests.** Every kept CPU trial, main's and the branch's, typed the 300 presses to
`809d588b…` (300 units) and the 8 popups to `c14d19d2…` (308 units). Three trials typed a key as
its long-press hint (E29) and were run again; `explain-digest.py` explains each.

**Probes**, on the branch's build (51a7be022, the same app as e75a095eb), emulator-5558:

* **Suggestions** (`suggestions.py`): "teh" and a space commit "the ", and a candidate pick
  commits its word, checked by digest on both renderers. The learned-state files were moved
  aside first (E39) and put back byte for byte.
* **Reshow** (`reshow.py`): 16 of 16 trials drew their keys after a quick hide and show.
* **Reading order** (`a11y-order.py`, `compare-a11y.py --order`): the five label-free captures
  (letters 57 nodes, expanded 63, collapsed 57, words 59, a popup 69) read in screen order with
  no issue, the same node counts as the plain-View root's.

**CPU.** Six alternating trials against main's build, on a loaded host (load averages 7.8 to
12.5): neither met the 2 ms gate, and the pairs' p95s (main 6.043, 3.157, 2.116; the branch
6.147, 2.665, 2.269 ms) track the host, not the build. This is not a pass. See
[cpu-p95-results](cpu-p95-results.md#the-compose-free-render-path-2026-11-23).

**The strip's emptied row (E56).** Those trials also showed the branch drawing 36 to 38 more
strip-only frames for the same typing: after about 50 presses of 308 the strip's words changed
twice, against about 13 on main. After a press whose words must be found again, the NLP session
publishes an empty list, while the refill is due, and then the new one. The branch's rebuild
reads the candidates sooner (10.7 to 12.6 ms from its start at the median, against main's 19.3
to 25.4), so it drew the emptied row more often; padded by 8.2 ms, it changed the strip twice
after 14 to 29 presses, against main's 11 to 18. The fix is `StripHold` in `ImeRootReads.kt`:
while Rust's row is empty and its refill is due, the strip keeps the words it drew, for at most
200 ms, with a rebuild asked for at that deadline; the suggestion pipeline is unchanged.

* **One press, one strip change.** Every whole trial of the fix drew 308 strip-only frames for
  308 presses, against main's 318 to 335 and 354 to 361 with the hold bypassed. The final words
  came no later: a median of 84 to 119 ms after the press, against main's 89 to 135.
* **CPU,** in a quieter rotation of main, the fix and the fix without its hold: the fix's p95s
  (2.607, 2.407 ms) are within main's (2.593, 2.675), with 81 and 77 frames over 2 ms against
  main's 90 and 118. Still not the 2 ms gate, and not a quiet host.
* **Tests:** `StripHoldTest`, and `strip-hold-mutations.py`'s 8 faults, each caught.
* **Probes** on the fix's build: suggestions matched on both renderers, reshow 16 of 16, and the
  same reading order with no issue.

See [the strip's emptied row](cpu-p95-results.md#the-strips-emptied-row-e56-2026-11-23).

**On the emulator.** `scripts/render-benchmark/compose-free-device.py` ran on emulator-5558 with
main's build (4d3b5d3ae) and the branch's (e75a095eb, with the GIF fixture). It checks each pose
by the input method window's view dump, then counts the pixels that differ inside that window
between the two runs. Screenshots and dumps are under the ignored
`build/compose-free/device/{main,branch,branch-night,branch-e53}/`.

* **Every panel with an icon,** by day and by night: keys, emoji, the clipboard header, overflow,
  the actions editor, one-handed, the subtype sheet, the incognito watermark, GIF and its search.
  Each pose had one input method window holding `ImeRootView` and no Compose class, and the
  process stayed alive.
  * 16 of the 20 poses are pixel-identical to main's (7 by day, 9 by night).
  * **gif-day** differs in 147,005 pixels, all inside the results grid (rows 378 to 724). The
    grid is live GIPHY content. gif-night is identical.
  * **search-day** differs in 294 pixels: the search field's caret, caught in another blink
    phase.
  * **sheet-day and sheet-night** each differ in 315 pixels: a 5 px column at the same place in
    both, which is the probe app's caret behind the sheet's scrim.
* **Material You:** 9 of 9 poses are pixel-identical to main's, by day and by night.
  * The seeds and flags are purple TonalSpot/Default/2021, teal Expressive/High/2025, pink
    Rainbow/Reduced/2021 and amber Fidelity/Medium/2025, plus the built-in Material You theme on
    the system accent.
  * On this ART, main's materialkolor and Rust give the same colours, as `PaletteArtDiff` found.
* **Fonts:** see [Part 3](#part-3-a-themes-own-fonts).
* **The Compose fallback** with the renderer switched off, by day and by night, on both builds:
  the window holds Compose (3 classes) and no `ImeRootView`. It is pixel-identical to main's.
* **Restored.** The harness put back the settings byte for byte after each run, and also the
  system's state and accessibility, and took out the fixture theme.
  * Afterwards the plain branch build (51a7be022) was reinstalled in place of the GIF-fixture one.
  * The baseline settings were written back and hashed after the IME was selected:
    `b028bcde…`, the same as before the first run.
  * The IME was selected again and the device session closed.
* **Two runs hit E53.** In the first branch run, the keyboard's top was 1 px off for 30 poses.
  In the second, gif-night stopped on "System bars did not settle". `branch-night` reran the
  night panels, 10 of 10.

## Kotlin

Production Kotlin, as `scripts/jvm-ledger.py` counts it (non-blank, non-comment lines), against
main 0f0b22d28: **405 files and 31,810 lines, from 408 and 31,970**, 160 lines fewer.

* **Deleted:** `SettingsIcons.kt` (73), `RustKeyPreview.kt` (33), `RendererIcons.kt` (29) and
  lib/snygg's `SnyggPalette.kt` (19).
* **Added:** `RendererThemeFonts.kt` (19), which resolves a theme's `@font` sources.
* **Smaller:** `RendererSmartbar` 145 to 89, `RendererClipboard` 45 to 37, `RendererEmoji` 65 to
  58, `RendererTheme` 40 to 35, `RendererSubtypes` 12 to 8, `RendererSearch` 12 to 9,
  `FlorisImeService` 434 to 432, `RendererPanels` 47 to 46, `RustKeyboardView` 411 to 410.
* **Larger, each raised in the ledger with its reason:**
  * `ImeRootReads` 131 to 145: `StripHold`, which keeps the strip's words while their refill is
    due (E56). `ImeRootView` is at main's 221: 5 lines fewer, then 5 to ask the hold;
  * the Compose `Smartbar.kt` (step K) 419 to 443: `smartbarConfig`, moved in unchanged from
    `RendererSmartbar`, its one caller now;
  * `SuggestionCandidate` 66 to 76: a clipboard candidate's icon token, which Rust draws, with
    the `ImageVector` the Compose Smartbar draws now picked from it;
  * `SnyggUi` 285 to 290: `materialYouScheme`, back from the deleted `SnyggPalette` for
    `ProvideSnyggTheme`;
  * `FlorisImeUi` 134 to 137 and `KeyboardState` 137 to 140: the Compose root collecting the
    service's StateFlows, and the `isRtl` flag the plain-View root sets;
  * `ThemeManager` 203 to 205 and `SettingsThemeEditor` 110 to 111: the palette from
    `themePalette`.

The JVM tests gain `PaletteMatchTest`, `ReferenceSnyggPalette` (main's palette, verbatim),
`IconSourcesTest` with its shared `IconVectors`, and `RendererThemeFontsTest`, and lose
`KeyPreviewMatchTest` with the key preview. The instrumented tests gain `IconGoldenDump` and
`PaletteArtDiff`, each run only by its script on a fixture build.

## What remains

* **Step K.** What it still needs, outside this package, is listed in
  [compose-removal step K](compose-removal.md#k-the-compose-keyboard).
* **Owed** ([owed-checks](owed-checks.md)):
  * **P36:** the whole path on the S24 Ultra against main's build. That covers every panel with
    an icon by day and night, Material You on the phone's own accent and two set ones, and a font
    theme with a good file and a broken one.
  * **E28 and P4:** the 2 ms p95, on a quiet host and on the phone. These trials ran on a loaded
    host and are not a pass.
  * **E53:** this item hit it twice: a keyboard top 1 px off for 30 poses in one run, and the
    system bars not settling in another (see the note of 2026-11-23).
