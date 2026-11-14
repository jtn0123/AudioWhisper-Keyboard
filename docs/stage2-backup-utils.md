# Stage 2 packages 18 and 19: backup, restore and utility rules

## Device boundary

The initial, single T3 `device_list` call returned **“The device hub did not answer
within 40 seconds.”** Its inventory was incomplete despite listing the old emulator.
As instructed, no device open/close, adb, recovery, installation or interaction followed.
The other session's emulator-5554 was untouched. Part A was **not run**. Package 12's
PCM hashes, real-model stress/leak check, ten timing runs per path, screenshot pairs,
full typing/dictation checks, timestamp-driven long delete on both renderers, cleanup
and final logcat/shutdown remain owed. No previous acceptance result is relabelled.

The extended-editor email assertion's cause is **unresolved**. The earlier unchanged
build failed it with the canonical store; the existing report records that the fixed
coordinates were inside the captured key bounds. The unavailable guest prevents the
required pre-change/main reproduction, so this package changes neither its expected
digest nor its baseline and makes no fixture, coordinate or production-bug diagnosis.

## Package 18 boundary

The Java-free `backup` crate forbids unsafe code and owns the six-group manifest,
archive paths and names, backup filenames, selection/tri-state policy, metadata
compatibility and warning precedence, ordered restore effects and clipboard conflict
admission. Kotlin packs plain facts and executes a complete manifest/plan. JNI lives
only in `awnative`; packets are copied UTF-16 and have no handles or callbacks.

Preserved behavior includes:

- Versions below 64 and blank package names are invalid. Older AND newer supported
  versions warn, rather than rejecting newer backups. A version warning takes
  precedence over the vendor warning. The fork's package keeps its legacy vendor warning.
- Preferences are one Jetpref datastore entry. Rust chooses merge versus erase import;
  the existing external Jetpref library remains responsible for its codec, preference
  types, notifications and import execution. No individual preference is excluded.
- Learned/kept typing stores (`files/learned-words.txt`, `files/kept-as-typed.txt`) remain
  excluded, matching the pre-change backup and the explicit provider comments. Therefore
  the requested changed-learned-word rollback check cannot pass using either build's
  backup alone. Restoring the canonical fixture afterward is cleanup, not restore proof.
  Adding private-store snapshots would be an intentional backup-policy feature change;
  it is not claimed by this compatibility port.
- Selected keyboard/theme directories overwrite incoming files during merge. Erase
  first clears the selected directory, even when the corresponding source is missing.
  Unselected groups remain untouched.
- Erase clears all clipboard history/media even if no clipboard category was selected.
  Missing optional entries are skipped. Type filtering and incoming conflict decisions
  run in Rust; Android supplies complete-item equality excluding the database id and
  existing-file facts. Conflicts keep existing clipboard media. Incoming clipboard rows
  compare with the original snapshot, including its historical repeated-incoming behavior.
- Extraction reuses package 9's Rust archive policy: canonical containment, UTF-16
  name/path lengths, directory/copy/empty/skip decisions. Traversal entries are skipped;
  duplicate ZIP entries retain the existing ZIP reader's behavior. Missing explicit
  parents and corrupt ZIP/JSON remain failures of the unchanged platform I/O/codec.

Rust supplies semantic error/warning keys and the exact missing-metadata message.
Android maps keys to the existing localized resources and retains the existing
IllegalStateException/serialization/ZIP/I/O exception classes. `BackupIO` retains SAF,
files/ZIP, URI/content-provider calls, Room, preference import/export and serialization.
The UI retains observable checkbox state, resource lookup and timestamp localization.
Backup metadata's app version and timestamp come from Android facts.

## Package 19 boundary

The Java-free `utilities` crate forbids unsafe code and owns MIME filter shape checks,
wildcard expansion, MIME component admission and any/all/exactly-one rules; FlorisRef's
legacy prefix normalization, URL scheme-default and classification/relative-path/validity
rules; locale tag splitting/formatting, capitalization eligibility, display fallbacks and
layout; version parsing, formatting, comparison and changelog/install decisions; and user
dictionary word/shortcut/frequency validation. UTF-16 is retained, including lone surrogates.

Android retains `Uri` parsing, encoding/building and file/context lookups. In particular,
legacy refs return an authority/path plan to `Uri.Builder`, preserving encoded-path
semantics for `?`, `#`, percent escapes and empty paths rather than reparsing a rendered URI.
Android/Java `Locale` construction, installed-locale discovery, localized display-name
lookup, locale-sensitive title casing and ISO names stay platform-side to preserve ICU
and installed platform data. Rust formats the returned names and decides fallbacks.
The existing version parser's broken suffix behavior and NumberFormatException are
preserved, rather than silently introducing new version ordering.

MIME filters historically expose the platform regular-expression grammar (including
unescaped metacharacters), not only MIME glob syntax. The platform regex compiler/matcher
therefore remains a primitive effect. Rust produces regex/input plans and reduces the
returned predicate facts. This preserves Java/Android regex classes, Unicode semantics,
errors and lone-surrogate behavior without inventing a partial alternative regex dialect.
No Kotlin wildcard, MIME admission or collection predicate policy remains. No filter is
printed. The JVM-only Kotlin module loads the same `awnative` library and its tests have
an explicit host-library build dependency; no second native library is introduced.

`lib/Validation.kt` remains the Compose resource/result adapter, type contracts and generic
JVM callback DSL. Its extension rules were already Rust-owned in package 9; actual user
dictionary rules now use Rust too. Locale validation still executes platform construction.
Generic Kotlin collection/coroutine/type helpers are JVM adapters, not additional keyboard
policies. The package-8 deletion manifest's fallback Snygg/Compose drawing utilities are
not ported: their owners are scheduled for deletion after phone acceptance.

## Proof and reproduction

Test-only `ReferenceBackupPolicy`, `ReferenceMimeTypeFilter`, `ReferenceFlorisRef`,
`ReferenceFlorisLocale`, `ReferenceVersionName` and `ReferenceUtilityRules` preserve the
replaced code. Android-free reference adapters record plans/effects where JVM tests
cannot instantiate Android Uri, SAF, Room or the live preference store. The full frozen
FlorisRef remains available for device comparison. No test was deleted or skipped.

`BackupPlanMatchTest` covers all 64 selections × 64 incoming-presence combinations ×
two restore modes (8,192 ordered plans), manifest/tri-state combinations, all 256 eight-item
clipboard conflict masks and 10,000 seeded conflict/type cohorts. Compatibility/name tests
cover 294 package/version/current combinations and 882 timestamped filenames.
`BackupArchiveMatchTest` generates 13 archive cohorts plus two corrupt ZIPs locally:
valid, older/newer/unsupported metadata, missing/corrupt metadata and clipboard entries,
traversal, duplicate ordinary/metadata entries, missing parents and excessive names.
It compares extraction outcomes and file hashes, metadata errors and restore plans.
Nothing is downloaded, and generated archives stay under ignored `build/`.

`MimePolicyMatchTest` discovers literal production filters and exercises directed and seeded
patterns, malformed/null values, errors and all collection modes. Existing MIME tests stay.
`UtilityRulesMatchTest` compares real source references and 10,000 seeded refs plus 10,000
classification cohorts; installed locales and 10,000 generated tags; 10,000 generated version
strings and 20,000 comparison/format pairs; and every UTF-16 unit plus 10,000 generated strings
in all three dictionary validation rules. Native tests cover malformed packets and boundary
invariants. Android URI round trips and ICU/device parity remain owed.

```sh
./gradlew :app:testDebugUnitTest --tests '*Backup*MatchTest' --tests '*UtilityRulesMatchTest' :lib:kotlin:test
python3 -I scripts/backup-utils-mutations.py build/stage2-backup-utils/mutations-final
./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug
```

The mutation runner requires fresh assertion failures, restores exact bytes in `finally`
and reruns the restored suites. Compiler failures do not count as caught mutations.
Final mutation and gate results are recorded below after verification.

## Owed device acceptance

After Part A and its email-baseline investigation, on the recovered owned `aw_keyboard`:
back up through the real SAF UI, snapshot preferences and canonical-store hashes, change
settings and add the synthetic learned-word fixture, restore through the UI and compare
preference values/store hashes. Compare a generated incompatible archive's message with
the pre-change build. Check MIME/locale/ref behavior on Android. Restore prototype off,
Always day + Floris Day, canonical stores, original preferences/accessibility/permissions;
scan final logcat and close with `shutdown: true`. None of these is claimed by JVM proof.

## Restored mutations

| Deliberate change | Fresh failing reference assertions | Mutant Gradle exit |
| --- | ---: | ---: |
| Minimum restore version 64 → 65 | 2 | 1 |
| Admit existing clipboard conflicts instead of new rows/files | 1 | 1 |
| MIME wildcard requires zero-or-more instead of one-or-more characters | 1 | 1 |
| Invert the legacy FlorisRef leading-slash normalization condition | 1 | 1 |

All four original files were restored byte-for-byte. Fresh mutation evidence is in
ignored `build/stage2-backup-utils/mutations-final/`; the runner separately checks
that the restored backup, utility and MIME suites exit zero.

## Source accounting

Using the nonblank tracked production-source exclusions in `rust.md`, against `a846b8ab`:

| Language | Before | After | Net |
| --- | ---: | ---: | ---: |
| Kotlin | 53,186 | 52,985 | -201 |
| Rust | 46,516 | 47,124 | +608 |
| Java | 321 | 321 | 0 |

Rust is 46.92% of counted production source. Frozen references, generated character tables,
benchmarks and latin-oracle are excluded. All 35 changed/new source files are at most
479 lines. No warning suppression, lint baseline or skipped test is added. Generated ZIPs,
private state, screenshots, models, audio and build outputs are not committed.

## Final local gate

The exact required command exits **0**, captured from the Gradle process itself rather
than from a pipe. There are **788 JVM tests: zero failures, errors or skips** (461 app,
170 Rust-wrapper, 128 Snygg and 29 Kotlin-library tests). Rust tests, formatting, clippy
with `-D warnings`, the C++ oracle and normal debug assembly pass. App lint has exactly
five permitted warnings: one AndroidGradlePluginVersion, three NewerVersionAvailable
and one ChromeOsAbiSupport. The Rust library's Android lint report has no issues.

The first full invocation failed clippy's fixed-chunk API lint; changing it to
`as_chunks::<2>()` fixes the cause. The complete command was then rerun successfully,
without suppressions or changes to assertions. The named restored mutation suites
also all exit zero. Evidence is under ignored `build/stage2-backup-utils/`:
`final-gate.log`, `final-gate-status.json`, `gate-report-summary.json`,
`source-count.json`, `source-lengths.json` and `mutations-final/`.
Hosted CI is not the acceptance gate because of the stated account billing limit.
