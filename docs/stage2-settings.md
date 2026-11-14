# Stage 2 package 17: settings and subtypes

## Device status

The single initial T3 `device_list` returned **“The device hub did not answer within
40 seconds.”** The inventory was incomplete despite showing the old `aw_keyboard`
guest. Per the requested stop rule, no open/close, adb, recovery, installation or
interaction followed. `emulator-5554` was untouched. No device state was modified.

Package 12 Part A from [stage2-tts.md](stage2-tts.md), all owed checks from
[stage2-backup-utils.md](stage2-backup-utils.md), and this package's device checks
remain owed. Existing device holds and the unresolved extended-editor email
assertion remain unchanged. JVM evidence does not establish Android acceptance.

## Production boundary

The Java-free `settings` crate forbids unsafe code. It owns:

- All **143 preference defaults**, enum membership, scalar admission, **27 slider
  ranges and increments**, and Kotlin-compatible editing clamps. The catalogue is
  embedded in Rust from reviewed `catalogue.tsv` (a final `#` terminates each
  record after its empty columns); it is not read from a device/file
  at runtime. Kotlin's preference declarations contain keys and platform types.
- Stored subtype, quick-action and emoji-history JSON, including nested key types,
  popup selectors, required/optional fields, unknown fields, defaults, declaration
  order, polymorphic fallbacks, JSON escaping and UTF-16 strings. Both encoding and
  decoding run in Rust. Theme component strings, layout-map strings, color hex and local-time formats run in Rust too.
- Every existing historical preference migration: media/emoji, advanced/other,
  clipboard, one-handed OFF, theme-editor representations, deleted app-language,
  app-icon/devtools keys and action-arrangement replacement/addition/deduplication.
- Subtype add/duplicate, modify, remove, reorder validation, active selection,
  previous/next cycling, missing-active and empty-list fallback, removed bundled
  input fallback, exact-locale/then-language preset choice and editor validation.
  Hint-mode derivation and resource dropdown selection also run in Rust.

The existing input crate's active-subtype/cycle entry points use these same native
functions. Both renderers therefore retain the shared policy instead of a second
settings-specific fallback implementation. Popup/layout assembly and currency slot
policy were already Rust-owned in `keys`; the saved selections now use the native
settings codec/defaults as well. Missing runtime currency resources still resolve
through the existing Android resource map to the Rust-backed `CurrencySet.Fallback`.

Kotlin retains Jetpref/DataStore I/O, storage type IDs, write handlers, caches and
Flows; platform primitive construction/formatting; Java/Android Locale creation,
system queries and display names; resource-map access; DTO construction and Compose.
`nativeScalar` retains Jetpref's original typed binding while its serializer sends
raw stored input to Rust. Enum DTO construction uses the returned native index.
The two SharedPreferences stores hold model-verification cache metadata and crash
timestamps; their Android I/O remains at the platform boundary.
Complex codecs transfer whole ordered value packets, rather than per-field JNI
calls. There are no new native handles, locks, callbacks or file operations. JNI is
confined to `awnative`'s existing checked copied-packet boundary.

`EnumDisplayEntries.kt` contains localized UI entries and remains unchanged.
`TypingScreen` contains UI visibility, confirmation and Android actions, with no
additional persisted validation to port. Generic extension/asset serialization and
Jetpref's outer `.prefs` container format remain their existing library boundaries;
the app's persisted complex preference codecs are no longer Kotlin JSON readers.

## Compatibility details

The fork has **no numbered preference schema dispatcher**. Jetpref invokes the
key-based migration on load/import, including current keys. Tests cover every old
key family; no invented version steps or new storage version are introduced.

Stored numeric admission remains wider than slider editing. The old datastore
accepts every representable integer/float, including values outside UI ranges,
NaN and infinities. This port preserves those values and keeps the UI's original
bounds. Imports do not silently clamp existing values. Unknown keys retain their
original migration result. Invalid enum/time parsing remains nullable. Invalid
emoji-history JSON still yields empty history. Invalid subtype/action JSON still
fails, with native diagnostics containing no preference contents.

Subtype removal selects the first remaining stored subtype when the active id no
longer exists. An empty list uses the fixed English/US QWERTY fallback, id -1;
the old build did not replace that fallback with the current system locale. System
locale is used for preset suggestion, with complete platform Locale identity then
language fallback. Id assignment still uses the Android-supplied current time.
Reordering has a new whole-list plan: only a complete permutation is accepted;
invalid indices/duplicates are a no-op. Existing screen behavior is unchanged.

Action migration preserves JVM equality: separately decoded auto/multi key objects
and popup-bearing keys do not deduplicate merely because their JSON matches. Plain
text keys and inserted text retain value equality. Unknown action/key discriminator
values retain the old default-deserializer behavior. JSON strings retain all UTF-16
units, including unpaired surrogates and raw controls. The existing streaming lexer
also preserves missing object commas and bracket-only skipping of unknown values.
Repeated known fields validate every occurrence before retaining the final value;
a later valid field cannot hide an earlier corrupt value.
Native typed JSON and transport nesting are bounded (128/256) to avoid unbounded
native-stack recursion; ordinary/generated stored fixtures remain below those limits.

## References and match proof

Test-only `ReferencePreferenceModel`, `ReferencePreferenceMigration`,
`ReferenceSubtype*`, `ReferenceSettingsCodecs`, `ReferenceSettingsBounds`,
`ReferenceColorSerializer` and `ReferenceQuickAction*` retain the original rules.
The replaced persisted JSON configurations are now test-only too. No existing test
was removed, ignored or skipped. Generated state uses only synthetic values.

- `SettingsDefaultsMatchTest`: every declared preference/default/storage type,
  each enum's choices and invalid values, scalar extrema/overflow/odd inputs, every
  slider endpoint/step and seeded clamps, including NaN, infinity and signed zero.
- `SettingsCodecMatchTest`: real bundled presets, 2,000 generated subtype states,
  2,000 quick-action arrangements with every key/selector/popup variant, corrupt and
  unknown fields/discriminators and repeated known fields with corrupt earlier values; every UTF-16 unit and 1,000 generated histories.
- `SettingsMigrationMatchTest`: **1,000 generated historical stored states** across
  every old key family, corrupt/unknown entries and type IDs, plus **2,000 action
  arrangements** comparing complete migration action/type/key/value outputs.
- `SettingsSubtypeMatchTest`: **30,000 operations across 200 seeds**, including
  add/remove/modify/reorder/select/cycle/load, locale changes, removed resources,
  missing ids, fallback and exact/parent-language preset choices. Return values,
  complete lists, selected subtype and persisted JSON are compared after each step.
- `SettingsDerivationMatchTest`: 5,000 editor-validation/resource-selection cohorts,
  hint-mode combinations, 5,000 color values plus malformed/leading-zero cases,
  local-time boundary combinations and 5,000 time encodings.
- `SettingsNumericMatchTest`: more than 40,000 directed/seeded quoted and unquoted
  JSON integer cases, including exponent forms, wrapping accumulation, malformed
  signs and overflow, against the pinned kotlinx lexer. Key-type parsing also
  retains Kotlin uppercase mapping (including dotless i).
- `SettingsStoredFileMatchTest`: Jetpref itself generates **20 current-format real
  `.prefs` files** with mixed synthetic scalar/complex settings and two subtypes.
  Both old and new models load/migrate/export each file; every decoded preference
  and complete exported file bytes match. Native reload/export is byte-identical.
  Files are generated under ignored `build/`, never taken from a user device.

The comparisons caught an incorrect component round-trip assumption in the generator,
an invalid-reorder error instead of a no-op, and the native color parser's over-strict
length limit for leading zeroes. Tests retain the invalid cases and the corrected
implementation matches the original admission/output rules. The final codec
review also distinguishes JSON integer grammar from stored scalar grammar and
preserves Unicode key-type case mapping.

```sh
./gradlew :app:testDebugUnitTest --tests '*Settings*MatchTest'
python3 -I scripts/settings-mutations.py build/stage2-settings/mutations-final
./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug
```

## Owed device checks

After T3 recovery, run Part A and backup/utils acceptance first. Then install
`33b8db50` with `scripts/install-dev.sh`, configure mixed synthetic settings and two
subtypes, capture values via the probe, upgrade with the same installer and compare
all values. Remove the active subtype through the real UI and compare fallback.
Run the complete existing fixtures on both renderers, including `809d588b…`,
`c14d19d2…`, `6e5ce6af…`, `106a5842…` and the timestamp-driven long-delete oracle.
Keep builds idle during timed checks. Require a clean literal FATAL/AndroidRuntime
scan. Back up touched state and verify exact hash restoration after every wrapper;
restore canonical stores, preferences/subtypes/accessibility/permissions, prototype
off, Always day + Floris Day, IME ownership and the specified silent-stream/DND state;
close the owned device with `shutdown: true`. None of this ran in this task.

## Restored mutations

`scripts/settings-mutations.py` mutates production policy, requires fresh JVM
AssertionFailedError reports (compiler failures do not count), restores exact
original bytes in `finally`, then reruns all settings match suites.

| Mutation | Fresh assertion failures | Restoration |
| --- | ---: | --- |
| Number-row default false → true | 1 | Byte-identical |
| Long-press editing minimum 100 → 101 | 1 | Byte-identical |
| Advanced settings-theme migration targets the wrong key | 1 | Byte-identical |
| Missing active id chooses the last subtype instead of the first | 2 | Byte-identical |

Final evidence is under ignored `build/stage2-settings/mutations-final-catalogue/`.
A final duplicate-field review added three failing differential assertions before
fixing the subtype/action/history decoder to validate each known occurrence. All
13 settings match tests pass afterward (`duplicate-field-after.log`). The default,
bound, migration and fallback mutation target files are byte-identical to their
successful control run, verified by `final-mutation-target-sha256.json`; the complete
local gate is rerun on the corrected decoder.
An earlier mutation run caught all four changes too; adding stricter JSON-number
cases during its final restored-suite build exposed a remaining numeric-admission
mismatch. That run is retained separately, and the complete controls are rerun on
the final source rather than relabelled. The final native integer codec follows
the pinned kotlinx lexer; the expanded numeric comparison passes.

## Source accounting

Using `rust.md`'s tracked nonblank production-source rules against `33b8db50`:

| Language | Before | After | Net |
| --- | ---: | ---: | ---: |
| Kotlin | 52,985 | 52,819 | -166 |
| Rust | 47,124 | 48,567 | +1,443 |
| Java | 321 | 321 | 0 |

Rust is **47.75%** of counted production source. Test-only references, generated
character tables, benchmarks and the C++ oracle are excluded. All 60 changed/new
source files are at most 481 lines. No suppression, lint baseline or ignored test
was added. Preference data and generated files remain under ignored `build/`.

## Complete local gate

The exact required command exits **0**, captured directly from the Gradle process
by `subprocess.run`, without a pipe:

```sh
./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug
```

There are **802 JVM tests: zero failures, errors or skips** (475 app, 170 Rust
wrapper, 128 Snygg, 29 Kotlin-library). Rust formatting/tests, clippy with
`-D warnings`, the C++ oracle, Android native build and normal debug assembly pass.
App lint reports exactly five permitted warnings: one AndroidGradlePluginVersion,
three NewerVersionAvailable and one ChromeOsAbiSupport. No new warning is hidden.
The unchanged Compose stability linkage test passes after the clean rebuild.

The final complete run takes 3m 30s. Evidence under ignored `build/stage2-settings/`
includes `final-gate.log`, `final-gate-status.json`, `gate-report-summary.json`,
`source-count.json`, `source-lengths.json` and `mutations-final-catalogue/` (all four
mutants caught; all 13 restored settings match tests pass). Hosted CI is excluded
as the acceptance gate because of the stated account billing limit.

The first full invocation found stale incremental Compose callers referencing
`FlorisPreferenceModel.Keyboard.$stable` and `.Clipboard.$stable` after the model's
base binding changed. Inspection of the actual runtime class JAR confirmed callers
referenced fields absent from the newly compiled owners. `:app:clean` discards the
stale app output; the linkage test and its assertions are retained unchanged. The
full command is rerun after fresh app compilation. Both the failed gate and the
class-file diagnosis remain in ignored `gate-before-clean/` and
`stability-diagnostic.json`; this is not treated as a passed gate.
