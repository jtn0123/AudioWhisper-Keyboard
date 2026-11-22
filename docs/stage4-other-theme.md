# Stage 4: Other and Theme

This page covers step 4 of [the Compose removal plan](compose-removal.md): the **Other** and
**Theme** settings screens are now drawn in Rust, and their Compose versions are deleted. Two
widgets they needed did not exist in Rust before: a **colour picker** and a **time picker**. Both
are new here.

| Route | Drawn by | Notes |
| --- | --- | --- |
| `settings/other` | Rust | New here. |
| `settings/theme` | Rust | New here. |
| `settings/other/physical-keyboard` | Rust | Already Rust; Other links to it. |
| `settings/other/backup`, `settings/other/restore` | Compose | Step 5. |
| `settings/theme/manage/SELECT_DAY` and `SELECT_NIGHT` (the theme manager) | Rust | Since step 9: [stage4-theme-editor.md](stage4-theme-editor.md). |
| `ext/list/EXT_THEME?showUpdate=true` (theme extensions) | Compose | Step 10. |

Rows that lead to a Compose screen open it in `ComposeSettingsFallbackActivity`, as before, and
Back from there returns to the Rust screen.

## What each screen shows

**Other** (no preview field):

1. *Settings theme*: a list (System default, AMOLED auto, Light, Dark, AMOLED dark).
2. *Settings accent color*: the colour picker. It colours the settings screens themselves.
3. *Physical keyboard*: opens the Rust screen.
4. A *Back up and restore* heading, then *Back up data* and *Restore data*, which open Compose.

**Theme** (with the preview field, so the keyboard can be tried out):

1. *Theme mode*: Always day, Always night, Follow system or Follow time.
2. *Day theme* and *Night theme*: each names the chosen keyboard theme and opens the theme
   manager. Day is greyed out under Always night, and Night under Always day.
3. *Sunrise time* and *Sunset time*: the time picker. Greyed out unless the mode is Follow time.
4. *Accent color (Material You themes)*: the colour picker. It colours the keyboard's Material
   You themes.
5. The *Managing theme extensions* box: a note and a *Go to Theme extensions* button.

Every row, summary, greyed-out rule and link is the Compose screen's, checked against a frozen copy
of it (see [Tests](#tests)).

## Theme names

The day and night rows show a theme's name, such as "Floris Day", not its id. The names come from
the extension index, which only Kotlin can read. `SettingsFacts` watches
`ThemeManager.indexedThemeConfigs` and sends one fact per theme,
`theme_label:<extension>:<component>` with the name, whenever the index changes. Rust picks the
right one for the stored id. When it finds none, it shows the id, as Compose did. An id that is
not two parts separated by one colon falls back to the default theme, as Jetpref's reader does.

## The colour picker

It is Jetpref's `ColorPickerPreference` with the advanced layout and no alpha slider, the dialog
Compose showed. It has two pages, and the middle button switches between them.

* **Presets:** FlorisBoard's 19 colours, then *Default*, which is the phone's own accent colour
  (the system accent, `system_accent1_500`).
* **Custom:** a square for saturation and brightness, a hue bar, a preview circle and a text
  field. The field shows the colour as hex. After *Edit color* it takes typed hex, `rgb(…)` or
  `hsv(…)`, which *Apply changes* checks; a bad value turns the field red.

**What is stored.** OK stores the colour as Jetpref stores a Compose `Color`: 16 hex digits. The
system accent is stored as "no colour" (`Color.Unspecified`), so the setting keeps following the
wallpaper, as Compose's `colorOverride` made it do.

**Where the maths lives.** `settings_ui/color.rs` copies the Compose and Jetpref steps one for
one, in the same order and in 32-bit floats, so the same input gives the same bytes:

* `Color.hsv` and the sRGB packing;
* Jetpref's `toHsv`, its three text formats and its parser;
* AndroidX's `calculateLuminance`, which picks a dark or light tick on a swatch.

It also draws the square, the hue bar and the checkered backing as pixel images, because the Rust
panels can only clip to rectangles. `settings_ui/picker.rs` holds the dialog's state, kept as
Compose keeps it: the hue, saturation and brightness survive a page switch, and the field's text
and error start again.

## The time picker

It is Jetpref's `LocalTimePickerPreference` dialog around Material 3 (1.4.0)'s `TimePicker`
(the dial) and `TimeInput` (two number fields). An icon button switches between them, and the
dialog remembers the choice for each row while the screen is open.

* **The dial.** It shows 12 hours, or 24 with the afternoon on an inner ring, or every fifth
  minute. A tap picks a number. A drag moves the hand once the finger has moved 8 dp (the touch
  slop). The hand then trails the finger by that 8 dp, as Material's drag detector leaves it.
  After the hour the dial moves on to the minute by itself, unless TalkBack, Switch Access or
  Voice Access is on, as in Material.
* **The fields.** Each field accepts what Material's accepts. On a 12-hour clock, an hour above 1
  moves on to the minute. On a 24-hour clock nothing moves on by itself, and Next does. The
  keyboard is the number keyboard, with Next on the hour and Done on the minute.
* **12 or 24 hours** follows the phone's setting (`DateFormat.is24HourFormat`), which Kotlin
  reports.
* **What is stored.** OK stores `HH:mm:00.000`, as Jetpref writes a `LocalTime`. The row's summary
  is Jetpref's: "18:05", or "6:05 PM" on a 12-hour clock.

`settings_ui/time.rs` holds the rules (the stored form, the summary, the dial's angles and what a
field accepts); `time_events.rs` wires them to the dialog; `time_dialog.rs` and `time_dial.rs`
draw it.

## What Kotlin does

Kotlin only reads Android and passes things through:

* **Facts:** the theme names, the system accent, the 24-hour setting, and whether an accessibility
  service that turns off the minute move is running. Rust decides what each one means.
* **Colours:** three more palette colours for the dialogs (tertiary container and its text, and
  the error colour).
* **Text sizes:** the time fields' 45 and 57 sp, in pixels.
* **Input:** touches and keys are forwarded, and a time field gets the number keyboard.

## Differences from Compose

Each one is deliberate.

| Difference | Why |
| --- | --- |
| A preset swatch is read out as its hex value; in Compose it has no name. | Compose's swatches are silent to TalkBack. A name costs nothing. |
| *Default* is one control; Compose has a row and a separate colour box. | One thing to tap, one thing to read. |
| The read-only hex field is text, with no tap action. Compose lets it take focus and select. | It cannot be edited until *Edit color*. Copying it is owed with text selection (step 8). |
| The colour dialog's content scrolls when it does not fit; Compose's does not. | Every Rust dialog scrolls content that does not fit the window. |
| The hue bar takes touches 48 dp tall, not just its 12 dp track. | Material's minimum touch size. |
| The checkered backing is drawn only for a see-through colour. | An opaque colour covers it anyway. |
| The square and the hue bar are not accessibility nodes. | Compose's are not either. TalkBack users type a value. |
| Rotating the phone closes a colour or time dialog. Other dialogs stay open. | Compose kept these two dialogs' state in `remember`, which a rotation drops. Kept as it was. |
| The clock hand jumps; Compose animates it. There is no 100 ms pause before the dial moves on to the minute. | No animation system in the settings renderer yet. The value is the same. |
| A time field takes ASCII digits only; Material also takes other digits and a minus sign that then fails. | The keyboard is a number pad; the result is the same for every value it can type. |
| A hardware digit key typed while a clock number has focus does not jump to the fields. | Material does this through a focus quirk; Rust's key focus works differently. |
| Clock numbers are read as buttons; the AM/PM pair is two buttons, not a group. | Rust has no group role. Each one still says if it is selected. |
| On a 24-hour clock the dial reads 1 as "1 hour"; Material says "1 hours". | Material's "%1$d hours" and "%1$d minutes" are plurals here, as Android's lint asks, so Android picks the right form. Every other number reads as before. |
| The strings Jetpref and Material wrote in English ("Presets", "Custom", "Select hour"…) are English app strings. | Compose showed them in English too. Translating them is a separate change. |
| Focus outlines are drawn for keyboard and D-pad focus, and the control last tapped in an open dialog keeps one (the *Presets*/*Custom* button, the clock button). | Every Rust screen does this today; whether a touch should hide them is owed for all of them. |
| Selected text in a time field is the primary colour at 40 %. | Compose's default selection colour. |
| The `:` between the fields and the tracking of the digits are close copies, not exact. | Compose trims the font's padding, which the Rust text layout does not. |
| Accessibility facts are read when the screen resumes, not live. | The other facts work this way. Turning TalkBack on means leaving the app anyway. |

## Tests

* **Frozen Compose reference.** `ReferenceOtherThemeScreens` (JVM test sources) is a copy of the two
  Compose screens' rules as they were at `b0d4a9a4f`, including the 19 preset colours.
  `SettingsOtherThemeMatchTest` holds the Rust screens against it:
  * every row's title, summary, role, greyed-out state and click, over every combination of
    stored values and facts: Other's 15 (5 settings themes × 3 accents) and Theme's 1,536 (4 modes
    × 4 day ids × 3 night ids × 4 sunrise and 2 sunset times × 2 sets of theme names × 2 clocks).
    The ids include ones not in the index and texts that are not ids at all; the times include
    midnight, a time with seconds, and one Jetpref cannot read;
  * every link goes where Compose's went;
  * about 1,300 typed colours parse, show and store as Jetpref's own parser and formats do, and
    random picking, paging, cancelling and confirming match Jetpref's state;
  * the time summary at every minute of the day on both clocks matches Jetpref's `LocalTime`;
  * what the time dialog stores, on the dial and typed, matches Jetpref's serializer.
* **Rust unit tests** for the maths: HSV and RGB both ways, hex parsing, clamping and rounding
  (`settings_ui_color.rs`, 9); the time rules, including midnight, noon and both clocks
  (`settings_ui_time.rs`, 12); the dialog flows and layout (`settings_ui_theme.rs`, 9;
  `settings_ui_other_theme.rs`, 4).
* **Icons.** `SettingsResourcesTest` now checks that every icon a Rust screen names has a vector,
  because a missing one draws nothing and fails no other test.

## Mutations

A mutation is a small deliberate mistake put into the code to check that some test notices it.
`scripts/settings-ui-mutations.py` has 22 new ones for these screens (now in
`scripts/settings_ui_mutation_cases/other_theme.py`); each must make a test fail,
and the code is put back after each one. **All 22 were caught**, and the restored code passed
(`build/stage4-other-theme/ui-mutations/result.json` for the first 20, then
`ui-mutations-label2/` and `ui-mutations-plural/` for the two added with the late fixes).

| Mutation | What it breaks | Caught by |
| --- | --- | --- |
| theme-day-enabled-by-night | Day theme greyed out under Always day instead of Always night | match test |
| times-follow-system | Sunrise and sunset enabled under Follow system, not Follow time | match test (2) |
| theme-no-preview | Theme loses its preview field | match test |
| addon-list-without-update | *Go to Theme extensions* opens the list without its update check | match test |
| backup-opens-restore | *Back up data* opens Restore | match test |
| theme-route-dropped | Theme is no longer a Rust screen | match tests (7) |
| theme-label-not-looked-up | Rust looks theme names up under the wrong key | match test |
| theme-id-any-colons | An id with two colons counts as an id | match test |
| theme-label-facts-misnamed | Kotlin sends theme names under the wrong key | match test |
| addon-title-not-lowercased | The box title keeps "Theme Extensions" capitalised | match test |
| time-hour-padded | "06:05 AM" instead of "6:05 AM" | match test (3) |
| time-noon-am | Noon shown as AM | match test (3) |
| time-stored-milliseconds | A time stored with a stray millisecond | match test |
| time-enter-on-hour-finishes | Enter on the hour field closes instead of moving to the minute | match test |
| accent-stored-as-colour | The system accent stored as a fixed colour, so it stops following the wallpaper | match test |
| time-drag-rounds-down | The dial rounds a drag down instead of to the nearest mark | Rust time test |
| dial-moves-on-under-accessibility | The dial moves on to the minute even with TalkBack on | Rust dialog test |
| drag-without-slop-trail | The hand sits under the finger instead of trailing it by the slop | Rust dialog test |
| keyboard-mode-forgotten | Reopening a time dialog forgets it was in keyboard mode | Rust dialog test |
| time-default-button-keeps | The time dialog's *Default* button leaves the stored time as it was | Rust dialog test |
| hex-label-zero-line | The hex field's *Color* label is laid out zero lines tall, so the colour is drawn over it | Rust dialog test |
| dial-minutes-read-as-hours | The dial's minutes are read out with the hour plural | Rust dialog test |

`scripts/settings-mutations.py` gains one: the sunrise default changed by a millisecond in the
settings catalogue, which `SettingsDefaultsMatchTest` caught. Its four older mutations were caught
again.

## Device checks

`scripts/settings-device-tests.py ... other-theme` drives both screens on the emulator through
`SettingsOtherThemeDevice`, once with a 12-hour clock and once with a 24-hour clock, after
`settings-device-baseline.py --restore`. **Every step passed on both clocks**, on `04b031f9b`, the
build before the dial's plurals (`build/stage4-other-theme/device/other-theme-12-result.json` and
`-24-result.json`):

* **Other:** opens and passes the accessibility audit; the settings theme list stores its choice;
  the accent colour through *Presets*, through *Custom* with the square and the hue bar dragged,
  typed as hex, and back to *Default*; Back closes the dialog without storing; rotating closes it;
  *Physical keyboard* opens; *Back up data* and *Restore data* open in Compose and Back returns.
* **Theme:** every mode greys out the right rows; the keyboard's theme follows day and night;
  the day and night rows open the theme manager in Compose, a choice there renames the row, and
  Back returns; the Material You accent is stored and the keyboard redraws with it (99.5 % of
  the sampled keyboard pixels changed, and a repeat of the same colour changed none); the sunrise
  time set on the dial (a tap to 7:30, a drag to 15:15) and typed (19:45) stores the right
  `HH:mm:00.000`; Back and rotation close the time dialog; *Follow time* follows the stored times;
  the theme mode list survives a rotation; *Go to Theme extensions* opens in Compose and Back
  returns.
* **Accessibility:** 0 violations over every screen and dialog visited.
* The older `ui` mode still passes on the same build.

That run predates step 9. Since then the theme manager is a Rust screen, and
`SettingsOtherThemeDevice` picks the day and night themes there, waits for the keyboard's preview
to start and end, and checks the row's TalkBack state ([stage4-theme-editor.md](stage4-theme-editor.md)).

The check waits 1.5 seconds after a dialog closes before it brings the keyboard back, because of
a keyboard bug this work found (see [Owed](#owed)): shown again within about a second of hiding,
the keyboard draws no keys. That bug is in the keyboard, not in these screens, and it happens
with or without a colour change, so the wait keeps the check on what it is testing.

## Pictures

`scripts/settings-pictures.py --sections other-theme` takes both screens from the top, every
dialog, and each dialog's second page (*Custom*, and the time dialog's keyboard entry), in light
and dark. `settings-picture-diff.py` compared them with the same pictures of the Compose screens
at `b0d4a9a4f` (`build/stage4-other-theme/compare/comparison.json`): **20 pairs, 14 identical**
(no channel off by more than 3).

* **Identical:** both main screens (not a single pixel different), the settings theme and theme
  mode lists, the colour dialogs' *Presets* page and the time dialog's dial.
* **The *Custom* colour page** (Other's and Theme's, light and dark) differs in two ways. The
  *Presets* button just tapped keeps the focus highlight (see the differences table). Apart from
  that band, 1,937 pixels (light) and 1,341 (dark) differ: the edges of the square's and the hue
  bar's round thumbs, by at most 12 out of 255, and 69 pixels in the glyphs of the hex value.
* **The time dialog's keyboard entry** differs only in the highlight on the clock button just
  tapped; outside that band, 0 pixels differ.

The pictures found one real bug: the hex field's *Color* label was missing, because it was laid
out with a text size the layout code gives no line height. It is fixed in `04b031f9b`, which a
Rust test and a mutation now hold.

## Deleted

`OtherScreen` and `ThemeScreen`, `AddonManagementReferenceBox`, the app theme and theme mode enum
display entries, and `ColorMappings` with `DEFAULT_GREEN`: 321 non-blank production lines, with
4 added where they were called. Over the whole branch, against `b0d4a9a4f`, production code gains
2,854 non-blank Rust lines and loses 71 Rust lines it rewrote; Kotlin gains 121 (the facts,
string ids and icons) and loses 329; string resources gain 29.

Jetpref's `ColorPickerPreference` and `LocalTimePickerPreference` are no longer called. Its
`JetPrefColorPicker` stays, for the theme editor's colour dialog (step 9).

## Owed

* **The keyboard's blank keys after a quick hide and show.** Hide the keyboard and show it
  again within about a second, and it draws its background but no keys, for at least three
  seconds. A one-second gap is fine. It happens with or without a theme change. The log shows the
  keyboard's drawing window released on hide and never acquired again; every later window is the
  settings screen's. It is in `RustKeyboardView`'s surface lifecycle, not in these screens, so it
  is left for its own fix. The device check waits it out.
* **Text fields and accessibility focus.** The Rust edit nodes do not report `isFocused` or offer
  `ACTION_FOCUS`, so tools that type through accessibility refuse them. The device check types
  with key events.
* **Copying the read-only hex value**, with text selection (step 8).
* **Translating** the English strings Jetpref and Material wrote ("Presets", "Custom",
  "Select hour"…).
* **The clock hand animation** and Material's 100 ms pause before the dial moves on.
* **Focus highlights after a touch.** Whether a tap should hide the focus highlight, as Android's
  touch mode does, is a question for every Rust screen, not just these.
* **The thumbs' and glyphs' few anti-aliased pixels** on the *Custom* page.
* **The dial's plurals on a device.** `b4b47eb0e` made the hour and minute names plurals after
  the device run, and added a device step that looks for "1 hour" on a 24-hour dial. That step
  and the rest of the check have not run on it: the device tools could not boot the emulator
  again ("Could not communicate with device support"). The Rust, JVM and resource tests cover it.
* **No phone check.** Everything above ran on the emulator only.
* **Test-harness limits, noted so nobody trips on them:** Compose refuses accessibility focus
  unless touch exploration is on, so the check opens Compose rows with a plain click; and
  `UiAutomation`'s global Back does not reach the keyboard, so the check hides it with
  `requestHideSelf`.
