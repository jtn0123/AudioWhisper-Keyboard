# Stage 2 packages 18 and 19: backup, restore and utility rules

**2026-11-14 update:** fresh SSD acceptance is recorded in
[the appended device record](#fresh-ssd-acceptance-2026-11-14). Earlier holds below
remain historical evidence.

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


## Fresh SSD acceptance (2026-11-14)

The owned new `aw_keyboard_ssd` is runtime `emulator-5556`; the old AVD and 5554 are
untouched. Evidence is under ignored `build/device-acceptance-2026-11-14/`. The pre-port
control is **a846b8ab**, with 908 selected Kotlin/Java/Rust production files verified
byte-identical; added probe/keep rules are test-only. Both APKs use `install-dev.sh`.

**PASS — real SAF round trip.** Main exports a preferences-only archive through the
system picker. Its guarded archive has two files (preferences and metadata). The probe
captures all 143 preferences and canonical store hashes. Production setters change three
settings; four synthetic learned-word additions change the known count 1→2 and store SHA
from `7848a5161495f8822d228eb8e2d298b38359d3e229a79a5db04ea5babf7276f0` to
`1ed8f403ee13f6cd0ecd71702917d69776f20fbd6cfa50cff807cf76b4fb9ca1`.
Restore selects this archive through the real SAF UI and ERASE/overwrite. All **143**
exported preference values return exactly. The mutated learned store remains unchanged;
the kept-as-typed store remains empty, SHA
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

The unchanged pre-port build restores the same archive through SAF with identical results.
Learned/kept stores are excluded from this legacy archive format: learned-store rollback
is unsupported on both builds, rather than a new restore regression. Each wrapper then
restores the separate canonical test baseline with exact preference/store hash checks.
Evidence: `saf-accepted/comparison.json`, `saf-old/comparison.json`.

**PASS — incompatible archive.** A guarded metadata-only version-63 archive is selected
through SAF on old/main. Both disable Restore and show the same 167-unit message hash
`c3f124a613693bd014128d55880d716eb53750cbae40f359ffa0a41a599751c9`.
All 12 visible application labels also match, aggregate SHA
`c49a0fa3024b525c6320c9ec16026ad5c6eecfdad35ccc1c26ed4c1c1d6d2aa8`.
Evidence: both `incompatible.json` records. No archive payload is trusted before guards.

**PASS — Android utilities.** Old/main match all 50 MIME booleans (aggregate SHA
`a9102ba77137f79f5fb514d52221fba28b22f1545e37669c29729504b22cb4e7`), six
locale hashes (`c4d69420a419d979203afec3ab6d4f4ec81429a6dc52c009a7c913be8149bfda`)
and six ref hashes (`0ed9f53f0e64836c271b9a9976f0d75be6c85882457834e22bd880a3f3ddade8`).
The only old/main initial preference difference is the last-use version marker, verified
against each APK manifest and `AppVersionUtils.updateVersionOnInstallAndLastUse`.
Earlier fixture calls failed because shrinking removed test-only probe entry points;
opt-in fixture keep rules fix those calls without changing production shrinking or policy.
Those failed attempts remain separate evidence. No package-18/19 acceptance check remains
unrun; final gate, restoration and shutdown are recorded after cleanup.


### Final fresh-device gate and hold

The final exact whole-codebase gate exits **0** (4m 48s): **803 JVM tests**, no
failures/errors/skips; Rust checks/oracle pass and only the five allowed lint warnings
remain. Normal debug is installed; final receipts verify exact baseline preference/store
hashes and accessibility null/0. Prototype off, Always day/Floris Day, owned IME and mic
permission pass before a final volume-verifier assertion. Disposable APKs/private audio
are removed; models remain. A normal launcher intent fails, and its one advised debug
retry also fails. Device work stops under the requested rule. **Still owed:** corrected
volume/DND readback, literal `FATAL|AndroidRuntime` final scan and shutdown. The device
is not claimed closed. See [the complete hold and diagnostic](stage2-tts.md#final-gate-and-device-hold);
this does not alter the package-specific acceptance results above.
