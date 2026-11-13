# Stage 2 package 9: extension metadata and import policy

The policy lives in the Java-free, `forbid(unsafe_code)` `extensions` crate. The existing
extension models remain Kotlin UI/serialization representations. Thin JNI in `awnative`
transfers UTF-16 and entry records; Kotlin retains Android URIs, file/ZIP I/O, canonical-path
queries, Compose resources and the import UI.

The replaced logic was in `app/src/main/kotlin/dev/patrickgold/florisboard/lib/ext/`
(`ExtensionMaintainer`, `ExtensionValidation`, `ExtensionComponent`, `ExtensionManager`),
`ime/theme/ThemeExtensionComponent`, `lib/io/ZipUtils`, `lib/cache/CacheManager` and
`app/ext/ExtensionImportScreen`. There is no separate `lib/ext` Gradle module in this checkout.

Rust now owns:

- Package/component identifier rules and compound-name parsing, including error text.
- Maintainer recognition, delimiter/range selection, whitespace rules and formatting.
- Metadata validity, component validation/hints and theme component construction/default paths.
- Extension/composer type recognition, import target groups, eligibility and skip reasons.
- Archive name/canonical-path limits, containment decisions and the expanded-size rule.
  Kotlin executes the returned skip/directory/copy/empty-file decisions in original entry order.

## Exact legacy behavior

The editor's package ID regex permits a digit at the start of later segments (`a.9`). A version
is valid when nonblank; this importer has no semantic-version or manifest-schema restriction.
Unknown metadata fields/schema URIs are ignored. Required JSON fields and types remain the
serialization boundary's responsibility. Import does not invoke the editor's metadata validators
or require referenced component files to exist. Duplicate components and archive entries remain
accepted; unknown top-level extension types fail. Unknown composer types fall back to Appender.

Archive entry names over 255 UTF-16 units and canonical destinations over 1,023 units are skipped.
A destination must start with the canonical destination directory plus `/`. Backslashes follow
the platform file-system semantics. A file entry larger than 100,000,000 bytes creates/truncates
an empty file, as the former `copy` did. Missing parent directories still produce the same I/O
failure. No new aggregate limit, deduplication or stricter validation is introduced by this port.

The syntax reader supplies canonical JSON without converting strings through UTF-8. Rust also
examines the raw first discriminator: the old streaming serializer selects it before a later
duplicate can overwrite the object key. Kotlin materializes the selected UI model through the
original streaming serializer and uses its original diagnostic path on errors. This preserves
exception classes, JSON offsets, paths and messages rather than replacing them with tree-decoder
diagnostics. Maintainer parsing returns substring ranges so the platform also retains its exact
exception for unusual regex-accepted reversed delimiters.

## Match tests and mutation

All fixtures are generated locally in tests; none is downloaded. Replaced Kotlin is retained as
`ReferenceExtensionMaintainer`, `ReferenceExtensionValidation`, `ReferenceExtensionPolicy`
(including the original serializer registry), and `ReferenceZipUtils` under test sources.

- `ExtensionMaintainerMatchTest`: every UTF-16 unit alone and inside a maintainer, 20,000 seeded
  mixed strings, delimiter edge cases and 10,000 formatting inputs: 312,158 comparisons.
- `ExtensionMetadataMatchTest`: all seven bundled manifests and 148 generated/reordered/lenient/
  malformed documents (155 document comparisons), including bad IDs/versions, missing required
  fields/types, duplicate components/discriminators and unknown types. It compares
  canonical results or exact exception class/message. Validation compares 376,653 outcomes,
  including resource IDs and regex substitutions; component/editor/name/default-path policy
  matches 5,000 seeded cohorts, plus type/import combinations and combined metadata validity.
- `ExtensionArchiveMatchTest`: 10,000 seeded entry lists and 11 generated real ZIP archives.
  Real ZIP fixtures cover traversal, missing parents/files, duplicate names/manifests, oversized
  names and a streamed 100,000,001-byte entry. Expanded contents and failures match the old I/O
  implementation; the oversized output is verified to exist with length zero.

Changing which ID segments allow an initial digit makes the validation match test fail with an
assertion. The code was restored and all six match tests passed. A concurrent test-report write
failure was discarded; the recorded mutation proof was rerun sequentially and verifies both
Gradle's nonzero exit and the intended assertion failure.

## Android proof

The optional replay APK (`-PsuggestFixture=true`, instructions in `stage2-a1.md`) also runs
`ExtensionDeviceReplay` against Android's real ICU regex implementation: 302,152 comparisons,
zero differences. The same APK's complete A1 decoder/pipeline files still match the original
pre-port device baseline byte for byte.

The real import UI was baselined before porting with a locally generated valid theme, a missing
manifest and an unknown extension type. The valid theme imported and was deleted. Both invalid
fixtures displayed the existing corrupted-extension message. The final normal APK reproduced those outcomes and messages: both invalid fixtures were
skipped, the valid theme imported, and its installed archive was verified and then deleted
through the real extension UI.

## Source accounting

Using nonblank production `.kt`, `.java` and `.rs` lines, excluding test/reference sources,
generated character tables and the test-only C++ oracle crate:

| Package | Kotlin removed (net) | Java removed (net) | Rust added (net) |
|---|---:|---:|---:|
| 6 | 85 | 306 | 542 |
| 9 | 23 | 0 | 674 |

Git rename detection is disabled when counting: moving Java/Kotlin into test sources must count
as removing production lines. Build files, test harnesses and documentation are excluded.

The first complete gate caught stale incremental Compose bytecode referencing absent `$stable`
fields on Keyboard/Clipboard preferences and ExtensionIndex, a previously documented compiler
output issue (`stage3b-package8.md`). The guard test was retained. A clean app compile with the
build cache disabled regenerated callers and their classes together before the final gate.

## Final local gate and cleanup (2026-11-13)

The required command completed with Gradle process exit code **0**:

```sh
./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug
```

The app's 416 JVM tests have zero failures, including the retained Compose linkage guard.
Clippy runs with `-D warnings`; formatting and the full C++ oracle task pass. App lint has
only `AndroidGradlePluginVersion`, `NewerVersionAvailable` (two notices) and
`ChromeOsAbiSupport`. All changed source files are at most 500 lines.

The normal APK has no LatinIME Java dictionary class or C++ decoder library; its Android
`libawnative` exports neither legacy LatinIME JNI nor the fixture snapshot entry point.
Compose and Rust each retain both 300-press/eight-hold digests from the pre-change build,
and both autocorrect/candidate-pick digests. The final logcat check has zero matches for
`FATAL|AndroidRuntime`.

Cleanup verified prototype off, Always day / Floris Day, project IME selected, DND off,
muted/minimum audio streams, recorded accessibility values restored and no bound probe.
The imported theme, generated downloads, private replay files and disposable probe were
removed. The owned `aw_keyboard` runtime (`emulator-5556`) was shut down with the device tool.
Ignored evidence is in `build/stage2-a1` and `build/stage2-extensions`; none is committed.
