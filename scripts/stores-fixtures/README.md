# Store fixtures from main's Room

The files in `app/src/androidTest/assets/stores/` and `app/src/test/resources/stores/` were
written by **main's own build** (48b2e0018, the last commit with Room) on the emulator, by the
two harness files here. They hold synthetic content only. See docs/phase5-stores.md for what
each proves.

| File | Written by | Read by |
|---|---|---|
| `history-v2.db`, `history-v3.db` | raw SQLite from Room's exported schema JSON (`app/schemas` on main), as Room made them | `StoresMigrationDeviceTest` |
| `history-v4.db`, `history-v4-empty.db` | main's `ClipboardHistoryDatabase` (Room) | `StoresMigrationDeviceTest` |
| `history-v4-wal.db` + `-wal` + `-shm` | main's Room, copied while open: its rows still in the WAL | `StoresMigrationDeviceTest` |
| `files-v1.db` | raw SQLite from schema 1 | `StoresMigrationDeviceTest` |
| `files-v2.db` | main's `ClipboardFilesDatabase` (Room) | `StoresMigrationDeviceTest` |
| `words-v1.db` | main's `FlorisUserDictionaryDatabase` (Room) | `StoresMigrationDeviceTest` |
| `*.expected.json` | what main's DAOs read from each database (via a copy, migrations and all), as row hashes | `StoresMigrationDeviceTest` |
| `backup-clips-*.json` + `.expected.json` | main's `writeJson`, as BackupIO writes a clipboard backup, and what main read back | `StoresBackupDeviceTest` |
| `clipboard-match.json` | 40 seeded sequences × 30 operations on main's Room DAO, driven as main's ClipboardManager drove it | `ClipboardStoreMatchTest` |
| `dictionary-match.json` | 24 seeded sequences × 25 operations on main's Room dictionary | `UserWordStoreMatchTest` |

## Regenerating

Only if a fixture must change; they are main's output, not this branch's.

```bash
git worktree add /Volumes/2TB/aw-scratch/stores-main 48b2e0018
cd /Volumes/2TB/aw-scratch/stores-main
cp <checkout>/local.properties .            # scratch only, never committed
mkdir -p app/src/androidTest/kotlin/com/jtn0123/audiowhisper/stores app/src/androidTest/assets/stores-schemas
cp <checkout>/scripts/stores-fixtures/*.kt app/src/androidTest/kotlin/com/jtn0123/audiowhisper/stores/
cp -R "app/schemas/dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardHistoryDatabase" app/src/androidTest/assets/stores-schemas/ClipboardHistoryDatabase
cp -R "app/schemas/dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardFilesDatabase" app/src/androidTest/assets/stores-schemas/ClipboardFilesDatabase
cp <checkout>/scripts/stores-fixtures/main-stores-fixture.pro app/stores-fixture.pro
# app/build.gradle.kts, beside the other fixture lines in the debug build type:
#   if (providers.gradleProperty("storesFixture").orNull == "true") proguardFile("stores-fixture.pro")
# 48b2e0018's androidTest Java doesn't compile (FixtureInstrumentation calls
# NlpManager.unlearnAutoCorrection, removed in ae452c49b): replace that one call with a throw.
./gradlew -PstoresFixture=true assembleDebug :app:assembleDebugAndroidTest
scripts/install-dev.sh emulator-5556
ANDROID_SERIAL=emulator-5556 scripts/install-dev.sh --probe app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk
adb -s emulator-5556 shell am instrument -w -e class com.jtn0123.audiowhisper.stores.StoresFixtureWriter \
  com.jtn0123.audiowhisper.keyboard.test/androidx.test.runner.AndroidJUnitRunner
adb -s emulator-5556 exec-out run-as com.jtn0123.audiowhisper.keyboard tar cf - files/stores-fixtures | tar xf - -C <scratchpad>
```

The writer deletes its own scratch databases; remove `files/stores-fixtures` from the device
afterwards. The goldens hold hashes; never print the databases' rows.
