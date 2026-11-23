# Phase 5: the platform edges' decisions move to Rust

2026-11-22. This is [all-rust.md](all-rust.md) phase 5's second item, "Decisions move to Rust",
for three of its four parts: the GIF request lifetime, the sequencing and fallback decisions of
dictation and read-aloud, and crash handling. These are [stage 5](stage5-inventory.md) packages
4 to 6. Locale is a separate piece of work and is not covered here. Branch `all-rust/edges`, from
main b1cf45f18, rebased on 218de9e55.

`AW` is `app/src/main/kotlin/com/jtn0123/audiowhisper` and `FB` is
`app/src/main/kotlin/dev/patrickgold/florisboard`.

| Part | Rust now decides | Kotlin keeps |
|---|---|---|
| [GIF](#gif) | the panel's session: searches, their generations and cache, chips, the search field, recents, which previews stay, each send's steps, request limits | the network, ImageDecoder and the preview bytes, URI grants and `commitContent`, the recents file's JSON, the threads |
| [Dictation](#dictation) | what a mic read means, its level, the PCM in a fixture WAV, the mic's buffer, whether a model's files are all there and whole, which to hash and what to remember | AudioRecord and its thread, the file reads, SHA-256, SharedPreferences |
| [Read-aloud](#read-aloud) | every step of a reading, the window it reads, where a reading asked for goes, how long the neural voice stays loaded | the AccessibilityService and its nodes, the voices, AudioTrack, audio focus, the timers, the stop button's window |
| [Crash](#crash) | whether a crash repeats the last, whether crashes come in a loop, the report shown and copied | the report files, the notification, both crash screens |

Each decision is a plain Rust module with its own tests. It sits behind natives generated from
`crates/boundary`, and its wire shapes are declared in `crates/wire`. No `external fun` was written
by hand. Four native objects are new: `GifSessionNative` (14 methods), `DictationEdgesNative` (8),
`ReadingNative` (20) and `CrashNative` (3), 45 natives in all. The 10 the GIF cache and recents
used are retired, so the build declares 465 natives, up from main's 430.

## GIF

### What moved

`crates/gif`:

* `session.rs` is the panel, one per process. Kotlin sends each call (`GifOp`), and the session
  answers with its state and the effects for Kotlin to run in order. It owns the searches and the
  10-minute results cache, which replaces `GifResultCache.kt`. It also owns the chips, the search
  field the keyboard types into, the typed word offered, and a swipe restored. A fetch or a
  recents read that answers after it stopped mattering is told apart by its generation, and
  ignored.
* `session/recent.rs` handles recents. They are read on another thread, keeping any GIF sent
  meanwhile, and saved whole after each send. Attaching another file starts a new generation.
* `previews.rs` decides which previews stay in memory: up to 24 MB, the least recently used going
  first. Android's `LruCache` used to decide this inside `GifImages`.
* `send.rs` runs one send. If the app takes GIFs, the GIF is downloaded once into a part file,
  renamed only when whole and committed as an image. Otherwise its page link is typed. Old files
  are pruned.
* `reply.rs` sets how much of each body to read, how long to wait and which status counts, and
  works out what a GIPHY answer comes to.
* `rows.rs` sends a GIF across as a row of six fields.

### Wire shapes (`wire::gif`)

* `GifOp`, 18 string ops: `source`, `attach`, `merged`, `show`, `offer`, `sent`, `startTyping`,
  `stopTyping`, `type`, `typeWord`, `deleteLast`, `deleteWord`, `swipe`, `submit`, `load`,
  `fetched`, `reset`, `snapshot`.
* `GifPlan` fields: `SHOWING`, `TYPING`, `QUERY`, `SHOWN`.
* `GifEffect`: `results`, `chips`, `fetch`, `cancel`, `read`, `save`, `forgetPreviews`.
* `GifResults`: `LOADING` 0, `LOADED` 1, `FAILED` 2.
* `GifReply`: `ok`, `status`, `tooLarge`, `unreachable`.
* `GifRow`: `ID`, `TITLE`, `PREVIEW_URL`, `RATIO`, `SEND_URL`, `PAGE_URL`.
* `GifRequest`: `MAX_BYTES`, `TIMEOUT_MS`, `OK_STATUS`.
* `GifMedia`: `API` 0, `PREVIEW` 1, `SEND` 2.
* `GifSendStep`: `start`, `checked`, `saved`, `renamed`, `committed`, `typed`.
* `GifSendAction`: `typeLink`, `prune`, `touch`, `download`, `rename`, `deletePart`, `commit`,
  `sent`, `linked`, `failed`.

`GifSessionNative` has `create`, `free`, `call`, `read`, `rows`, `request`, `previewsCreate`,
`previewsFree`, `previewsGet`, `previewsPut`, `previewsClear`, `sendNext`, `sendFiles` and
`sendStale`. The Kotlin wrappers are `GifSessionPlan` and `GifPreviewKeys`.
`AW/gif/GifResultCache.kt` and `lib/rust`'s `Gif.kt` are deleted. So are the 10 `GifNative` calls that `GifResultCache` and
`GifRecents` used for the cache and recents, which the session now keeps: `cacheTtlMs`,
`cacheCapacity`, `recentsCapacity`, `cacheKey`, `newCache`, `freeCache`, `cacheGet`, `cachePut`,
`recentsAdd` and `recentsLoad`.

### What Kotlin still does, and why

* `GifSearchSession.kt` (new) runs the session's effects. It starts the fetch jobs and cancels
  them, reads and writes the recents file on the IO thread, and keeps the flows the panel draws.
  Coroutines and threads belong to Android's side.
* `Giphy.kt` makes the `HttpURLConnection` request, reading no more than the request row allows.
* `GifImages.kt` decodes previews with ImageDecoder and holds their bytes. Rust holds only the keys.
* `GifSender.kt` does the FileProvider URI grant and `InputConnection.commitContent`, and the file
  operations the steps name.
* `GifRecents.kt` reads and writes the recents file's JSON. It stays M.
* `GifPanel.kt` is under the Compose freeze and is unchanged.

There is no GIPHY key. Everything here was proven on local fixtures, and nothing claims the live
API accepts these requests.

### Proof

* **Cargo:** 16 new tests in `crates/gif`, and `GifSessionPlanTest` in `lib/rust`.
* **`GifSearchTraceTest`** runs against `ReferenceGifSearch`, which is main's `GifSearch`,
  `GifRecents`, `GifResultCache`, `GifImages` and `GifSender`, together with `ReferenceLruCache`
  and `ReferenceGiphy`.
  * Each seed runs 50,000 events. They cover:
    * the panel opened and closed;
    * typing and swiping in the search, and chips;
    * searches answered, refused, cut off, garbled, or cancelled by a new search;
    * the clock passing the cache's 10 minutes, and previews past the 24 MB held;
    * sends into apps that take GIFs and apps that don't;
    * part files a crash left behind, recents read while GIFs were sent, and damaged recents;
    * process death and the service recreated.
  * After every event, these must match on both sides: the flows, the requests, the recents file,
    the previews and the sent-GIF folder.

  | Seed | 20261122 | 7 | 4000000007 |
  |---|---:|---:|---:|
  | Checks | 645,993 | 644,669 | 646,342 |
  | Differences | 0 | 0 | 0 |
  | Answers ignored after a cancel | 868 | 905 | 904 |
  | Cache hits | 929 | 925 | 972 |
  | Process deaths | 1,114 | 1,111 | 1,046 |
  | Service recreated | 744 | 748 | 760 |
  | Sent as images | 2,184 | 2,076 | 2,032 |
  | Sent as links | 730 | 688 | 724 |
  | Sends failed | 586 | 664 | 637 |
  | Recents read with sends meanwhile | 227 | 234 | 207 |
  | Recents damaged | 166 | 134 | 166 |
  | Previews evicted | 86 | 69 | 123 |
  | Part files pruned | 170 | 187 | 191 |

* **`GiphyTransportTest`** (5 tests) checks that each request is made as `GifRequest` says. It runs
  against a server on loopback, never GIPHY.
* **Intended differences:** none.

## Dictation

Dictation's session was already Rust's on main (`DictationEffect`): what a press does, when to
stop and what to discard. What Kotlin still decided was the microphone's pump and the model
check. Those have moved now.

### What moved

* `crates/dictation`'s `audio.rs` holds:
  * a chunk's level;
  * what a read's count means (`MicRead`: `DELIVER`, `SKIP`, `REPORT`, `END`);
  * the PCM in a WAV file a debug build plays in place of the microphone;
  * the size of the mic's buffer.
* `model_check.rs` decides whether a model's files are all there and whole. A quick check looks at
  names and sizes. A full check also needs each file's SHA-256, hashed once and remembered by its
  path, size and modification time. It also decides which files to hash, the verdict
  (`ModelCheckState`: `READY`, `MISSING`, `DAMAGED`) and which keys to remember. The model is
  either dictation's or the read-aloud voice's.

`DictationEdgesNative` has `level`, `read`, `wav`, `bufferBytes`, `quick`, `keys`, `toHash` and
`verdict`, with the wrappers `MicRules` and `ModelCheckRules`.

### What Kotlin still does

* `MicRecorder.kt` and `MicAudio.kt` run AudioRecord on its thread and read the WAV file's bytes.
* `ModelCheck.kt` lists the files, hashes the ones Rust asks for, and keeps the SharedPreferences
  keys Rust says to remember.

### Proof

* **Cargo:** 11 new tests in `crates/dictation`.
* **`MicTraceTest`** runs against `ReferenceMic`, which is main's `MicRecorder` and `MicAudio`.
  Recordings are read by both recorders and compared effect by effect. WAV files of every shape
  are parsed by both. The buffer rule is checked over 10,008 minimums, with 0 differences.
* **`ModelCheckTraceTest`** runs against `ReferenceModelCheck`. A model's files are written whole,
  cut short, changed, deleted, touched, made unreadable, and remembered or forgotten. After each
  event, both sides run a quick or full check. The state, every key looked up and every key
  remembered are compared in order.

| Seed | 20261122 | 7 | 4000000007 |
|---|---:|---:|---:|
| Mic reads | 50,118 | 50,002 | 50,003 |
| Mic effects compared | 30,090 | 29,361 | 29,076 |
| WAV files | 20,000 | 20,000 | 20,000 |
| WAV with audio | 3,657 | 3,704 | 3,645 |
| Not a WAV | 5,802 | 5,700 | 5,690 |
| Model check events | 50,000 | 50,000 | 50,000 |
| Differences, intended ones aside | 0 | 0 | 0 |

**Two intended differences, counted by the traces:**

1. **A malformed WAV file is refused as holding no audio.** For other bad files, main threw an
   IllegalArgumentException; these files now get the same answer. This path exists only in debug
   builds, where a WAV file replaces the microphone. On the files below, main never returned or
   crashed:

   | Seed | 20261122 | 7 | 4000000007 |
   |---|---:|---:|---:|
   | Never returned (its walk came back to a chunk it had read) | 143 | 135 | 127 |
   | Threw IndexOutOfBoundsException | 2,248 | 2,306 | 2,303 |
   | Threw NegativeArraySizeException | 728 | 670 | 677 |
   | Threw StringIndexOutOfBoundsException | 61 | 74 | 60 |

2. **A model check remembers the files that passed together, once the check is over.** Main
   remembered each file as it passed. So if process death cuts a check short, those files are
   hashed again next time. This affected full checks that came out ready 1,828, 1,894 and 1,916
   times per seed, and damaged 882, 953 and 884 times.

## Read-aloud

### What moved

* `crates/reader`'s `reading.rs` and `reading/look.rs` sequence a reading:
  * the stop button first;
  * then the voice: the neural one if it loads (saying so if it has just failed), otherwise the
    phone's;
  * then a look at the screen. It is walked until a walk finds text, at most 8 times. A web page
    whose text is all in sight is walked again while its tree grows.
  * then reading from where the reading starts, keeping two blocks queued ahead and scrolling the
    list on when they run out;
  * reading from the top scrolls the list back at most 60 times;
  * the last 50 blocks are kept to find the place again after a scroll;
  * a selection is found on the screen, or spoken by itself.
* `window.rs` chooses the window to read: the app in front, and the part of it shown above the
  keyboard.
* `lifetime.rs` holds two decisions:
  * `VoiceHold` keeps the neural voice loaded between readings and frees it after a time idle.
  * The route decides where a reading asked for goes while the service isn't up yet.

### Wire shapes (`wire::reading`)

* `ReadingEffect`, 17 effects, each sent as three ints (the kind and two arguments): `SHOW_BUTTON`,
  `TAKE_VOICE`, `TELL`, `MAKE_VOICE`, `BEGIN`, `WALK`, `WAIT`, `KEEP`, `REVEAL`, `SCROLL_BACK`,
  `SCROLL_FORWARD`, `FOLLOW`, `UNFOLLOW`, `AWAIT_BLOCKS`, `QUEUE`, `SPEAK_SELECTION`, `DONE`.
* `ReadingFrom`: `SCREEN`, `TOP`, `SELECTION`.
* `ReadingMessage`: `VOICE_FAILED`, `NOTHING`, `UNAVAILABLE`.
* `ReadingFact`, a screen's facts row: `VISIBLE_TOP`, `VISIBLE_BOTTOM`, `NODES`, `WEB`, `SCROLLER`,
  and `SIZE` 5.
* `VoiceHoldStep`: `USE`, `NONE`, `CHECK`, `LOAD`.
* `ReadRoute`: `READ`, `ASK`, `WAIT`.

`ReadingNative` has `create`, `free`, `start`, `done`, `answer`, `looked`, `lookedNothing`,
`window`, `visibleBottom`, `holdCreate`, `holdFree`, `holdTake`, `holdChecked`, `holdLoaded`,
`holdFailedJustNow`, `holdIdle`, `holdMayFree`, `route`, `stillAsked` and `lifetimes`. The
wrappers are `ReadingPlan`, `VoiceHold` and `ReadingRules`.

### What Kotlin still does, and why

* `Reading.kt` (new) runs the plan's effects on a `ReadingHost` and answers them, keeping the
  screens' nodes and the voice. The end of a reading (the voice shut down, the button hidden) also
  stays here, because it has to run on a coroutine that has already been cancelled.
* `ReadAloudService.kt` is the AccessibilityService. It keeps Android's node cache while a reading
  is on, and the stop button's window.
* `ScreenWalk.kt` walks the AccessibilityNodeInfo tree and makes the list's scroll actions.
* `LoadedVoice.kt` (new) keeps the neural voice, its lock and its idle timer.
* `VoiceEngine.kt` is the voice itself: sherpa-onnx Pocket TTS, through AudioTrack.
* `ReadAloud.kt` keeps the service, the Settings intent and the toast.
* Audio focus (`Voice.kt`) and the phone's own TTS (`Speaker.kt`) are unchanged.

### Proof

* **Cargo:** 16 new tests in `crates/reader`.
* **`ReadingTraceTest`** runs against `ReferenceReading`.
  * Both sides run on a fake Android with the same seed, which supplies the screens, scrolls,
    voices and refusals.
  * Every effect is compared in order: the status, the cache, the button, the voice taken and
    made, each walk, wait, scroll and reveal, each block queued, what is said, the end, and
    anything thrown.
  * A third of the sessions are cancelled at a random effect: by a stop, a new reading or the
    service being unbound. The session after one starts on a new `Reading` and a new fake, as
    after the service is recreated or the process died.
  * The window choice is also compared against main's, over the same windows.
* **`LoadedVoiceTraceTest`** runs against `ReferenceLoadedVoice`.
  * Readings take the voice, ask whether it failed and go idle, while the clock moves on, memory is
    wanted and the service is recreated. They contend for the hold's lock as they would on a
    phone.
  * Every call to Android is an effect: the files checked, each load, each voice closed, each
    wait, and what each reading was told.
  * The routing of a reading asked for is compared against main's, over all 4 routes.

| Seed | 20261122 | 7 | 4000000007 |
|---|---:|---:|---:|
| Reading sessions | 1,563 | 1,561 | 1,530 |
| Reading effects compared | 50,052 | 50,030 | 50,038 |
| Sessions cancelled | 206 | 222 | 210 |
| Scrolled on / back | 520 / 168 | 544 / 177 | 539 / 173 |
| Window lists | 20,000 | 20,000 | 20,000 |
| Voice hold events | 50,000 | 50,000 | 50,000 |
| Voice hold effects compared | 95,161 | 94,983 | 94,266 |
| Service recreated | 2,536 | 2,550 | 2,396 |
| Voice freed after idle | 241 | 258 | 265 |
| Routing asks | 20,000 | 20,000 | 20,000 |
| Differences | 0 | 0 | 0 |

**One intended difference:** three of `ReadAloudService`'s `Log.d` lines are no longer written.
They covered scrolling on: no list, the scroll refused, and where reading carried on. Nobody sees
these lines, and they are not effects. The line with the walk's counts and timing stays.

## Crash

### What moved

`crates/utilities`' `crash.rs`:

* `after(last, now)` returns `CrashAfter` bits:
  * `CHECK_LAST` (1): there was a last crash, so check whether this one repeats it.
  * `LOOP` (2): crashes are coming in a loop, within 5 seconds of each other or with the clock set
    back.
* `same(last, current)` is the repeat check.
* `report(facts, debug, names, details)` writes the report the crash screen shows and copies. The
  facts row (`CrashFact`) has `VERSION_NAME`, `VERSION_CODE`, `COMMIT_URL`, `CHANGELOG_URL`,
  `DEVICE`, `ANDROID`, `DEBUG_LOG` and `SIZE` 7. A row of the wrong length is refused.

`CrashNative` has `after`, `same` and `report`, wrapped by `CrashRules`.

### What Kotlin still does

* `CrashUtility`'s handler writes the report file first, because a crash's report has to be written
  before anything else is asked. The file names stay in Kotlin for the same reason.
* Then `afterCrash` (`AfterCrash.kt`, new) runs Rust's steps on the app's files and notifications:
  read the last crash, forget a repeat, keep the time, then notify for a loop or for a single
  crash.
* `CrashDialogActivity` reads the files, asks `CrashRules.report` for the report, then shows and
  copies it.

### The bootstrap contract

The crash path has to work when the native library fails to load:

1. **The library loads.** Everything is as above.
2. **The library fails to load.** `NativeRuntime.load()` returns false and records the failure.
   * `FlorisApplication.onCreate` returns before installing anything, as on main.
   * Settings shows `PlatformCrashScreen`: "could not load its keyboard engine", in platform Views,
     with a button that copies `CrashReport.nativeLoad`'s details. This is main's.
   * The keyboard's input view is the platform panel that offers another keyboard. This is main's.
   * `CrashDialogActivity` shows the platform screen and reads no report, so every report is
     kept. This is new: main composed the report in Kotlin, and without the library nothing can
     compose it.
   * `ClipboardMediaProvider` opens its store only if the library loaded (e5c6f4cb2). Content
     providers start before the application. Since the stores moved, the provider's eager open
     threw UnsatisfiedLinkError on a dispatcher thread and ended the process before any of the
     above could show. Forcing the failure on the emulator showed it: `FATAL EXCEPTION` on
     `DefaultDispatcher-worker-1`, from `onCreate`'s coroutine, and the instrumentation ended with
     "Process crashed".
3. **The library loads but the crash natives can't be called.** Any LinkageError counts here:
   UnsatisfiedLinkError, NoClassDefFoundError or ExceptionInInitializerError. `afterCrash` catches
   it and takes a fixed path: keep the time, notify once, forget nothing.

**Tests of the contract:**

* `CrashTraceTest.withoutTheLibraryEveryReportIsKeptAndOneCrashSaid` covers each of the three
  LinkageErrors. A failure that escapes the handler fails an assertion.
* On the emulator, the settings suite's crash mode now forces the failure before the application
  and every provider start (b8ebaf70a).
  * Arguments only reach an instrumentation after the providers have started, so the harness asks
    for the failure with a file, `files/settings-load-failure`. The instrumentation takes and
    deletes the file in `newApplication`.
  * A second `<instrumentation>` was tried instead. AGP replaces it with the configured runner.
  * The run passed:

    | Check | Result |
    |---|---|
    | `native_load_failure`, `non_compose` | true |
    | `copy_details_sanitized`, `clipboard_restored` | true |
    | `crash_theme_platform`, `ime_fallback_notice` | true |
    | `ime_process_survived`, `spell_checker_inert` | true |
    | `crash_dialog_without_library` | true |
    | `crash_reports_kept` | 1 (a synthetic report planted for the check, then removed) |
    | Fatal exceptions in the crash log | 0 |

**Both crash screens stay Android-drawn.** `CrashDialogActivity` is an Android View layout
(`crash_dialog.xml`), and the no-library screen is `PlatformCrashScreen`'s platform Views. Rust
cannot draw before its library loads, and the crash screen must also work when the Rust renderer
is the thing that crashed. Only the report's text is Rust's.

### Proof

* **Cargo:** 4 new tests in `crates/utilities`.
* **`CrashTraceTest`** runs against `ReferenceCrash`, main's handler and report.
  * The crashes come a few a second, some minutes apart, and now and then with the clock set back.
  * Between them, the crash screen reads and deletes the reports, and the app's data is sometimes
    cleared.
  * Every crash is a process that died, and every call to Android is an effect.
* **`aReportAddsNothingButItsHeadings`** shows that a report holds its inputs and fixed headings and
  nothing else, so no user text can get into it.

| Seed | 20261122 | 7 | 4000000007 |
|---|---:|---:|---:|
| Events | 50,000 | 50,000 | 50,000 |
| Effects compared | 191,502 | 192,625 | 191,339 |
| Crashes | 34,971 | 35,159 | 34,884 |
| Repeats dropped | 2,942 | 3,130 | 3,081 |
| Loops | 26,549 | 26,736 | 26,389 |
| Clock set back | 3,510 | 3,513 | 3,477 |
| Data cleared | 236 | 232 | 241 |
| Reports composed (debug) | 20,000 (10,090) | 20,000 (9,959) | 20,000 (9,994) |
| Stack traces in them | 29,995 | 29,929 | 30,059 |
| Differences | 0 | 0 | 0 |

**One intended difference:** when the library is unavailable, the user sees the platform crash
screen, every report is kept, and the notification is the single-crash one. Main never reached
this path, because it composed everything in Kotlin. It is counted over 3 failure kinds in the JVM
test and 1 forced run on the emulator.

## Mutations

`scripts/edges-mutations.py` makes 123 faults, one at a time, each in the code a decision moved
to or the Kotlin that carries it out. A fault counts as caught only when a fresh test report fails
by assertion: a Rust target needs a `panicked at` line, a JVM target an `AssertionFailedError` in
a report written after the fault. A failure to compile, or a test that errors, is not caught.
After the last fault the source is restored byte for byte, and every target it named must pass.

| Part | Faults | In Rust / Kotlin | Caught by the crate and a trace | Trace or JVM test only | Crate only | Missed |
|---|---:|---:|---:|---:|---:|---:|
| GIF | 34 | 26 / 8 | 22 | 11 | 1 | 0 |
| Dictation | 21 | 17 / 4 | 17 | 4 | 0 | 0 |
| Reading | 33 | 26 / 7 | 13 | 18 | 2 | 0 |
| Voice hold | 16 | 12 / 4 | 8 | 8 | 0 | 0 |
| Crash | 19 | 14 / 5 | 12 | 6 | 1 | 0 |
| **All** | **123** | **95 / 28** | **72** | **47** | **4** | **0** |

The restored source passed every target: `cargo:gif`, `cargo:dictation`, `cargo:reader` and
`cargo:utilities`, and the JVM targets `gif-trace`, `gif-units`, `mic-trace`, `model-trace`,
`dictation-units`, `reading-trace`, `voice-trace` and `crash-trace`, each with 0 assertion
failures.

* **Where they ran:** in separate worktrees at e5c6f4cb2. The only commit after it, 458609fed,
  changes a doc comment.
* **How the GIF faults were split:** across three runs to save time.
  * Faults 1 to 20 ran in one run, which was stopped once those were recorded.
  * Faults 21 to 27 and 28 to 34 ran in the other two, and each of those checked the restored
    source.

Four faults got past the tests on earlier runs. Each test was fixed and the fault is caught now:

* **`d0cbb0557`:** the scroll-back test counted to the constant itself, so changing the cap moved
  the test with it. Nothing held the blocks remembered to fifty either. Both are now pinned to
  main's numbers.
* **`7e24aa167`:** only a crash fact row that was too short was tested. A row that is too long is
  now refused in a test.
* **`7df6a5b14`:** a LinkageError escaping `afterCrash` errored the no-library test instead of
  failing an assertion. It now fails one.
* **`33dbed219`:** a fault that made `ModelCheck` throw failed the trace with an exception, not an
  assertion. What a check throws is now compared with main's, like its state.

## On the emulator

These checks ran on `emulator-5556`. Screenshots are in `/Volumes/2TB/aw-scratch/edges-device/` and
are not committed. Media volume was 0 throughout, the emulator runs with `-no-audio`, and the
microphone was never used.

1. **GIF**, on a `-PgifFixture=true` build using `scripts/gif-fixture.py`'s local fixtures: 209
   files, 204 GIFs, no key and no network.

   | Check | Result | Screenshot |
   |---|---|---|
   | Trending | loaded | `gif-1-trending.png` |
   | A chip | its own set | `gif-2-chip-search.png` |
   | Paging | to the end of 30 results | `gif-3-paged-to-end.png` |
   | Cancel by typing | a chip's search answers 4 s after a typed one; typing over the chip shows the typed answer and never the chip's | `gif-4*.png`, `gif-5*.png` |
   | Send into an editor that takes images | one `image/gif`, 17,435 bytes, the size of the fixture's send rendition | `gif-6-sent-to-image-editor.png` |
   | Recents | a Recents chip leads and holds the sent GIF | `gif-7-recents-chip.png`, `gif-8-recents.png` |
   | Send into a text-only editor | the page link typed instead: 30 UTF-16 units, SHA-256 matching the fixture's | `gif-9*.png` |

2. **The Compose fallback.** With Settings → Keyboard → "Draw the keyboard in Rust" off:
   * the Compose keyboard typed;
   * its GIF panel opened on the same Rust session, with Recents kept, and loaded trending
     (`compose-1..3*.png`).

   The setting was then switched back on.
3. **Dictation**, from the debug build's fixture WAV (`files/debug-mic.wav`) in place of the
   microphone.

   | Check | Result | Screenshot |
   |---|---|---|
   | Dictate | typed 75 UTF-16 units | `dictation-1-listening.png`, `dictation-2-typed-count.png` |
   | Cancel | 0 units, mic idle | `dictation-3-*.png` |
   | Model's encoder held away | "Not installed" | `dictation-4-missing-model.png` |
   | Model put back | the same 75 units, same hash | `dictation-5-after-restore-count.png` |

   The dictation proof runner compared reference and native on 7 fixtures (identical), with 20
   timings, and the production listener (identical) (`dictation-proof/comparison.json`). The
   learned-word files were unchanged by hash.
4. **Read-aloud**, muted, started from the keyboard's speaker key.

   | Check | Result | Screenshot |
   |---|---|---|
   | Reading | the renderer fixture's `reader-state` said READING, neural, playback head moving | `reading-2-follow-along.png` |
   | While reading | the stop pill over the app, the speaker key showing stop | `reading-2-follow-along.png`, `reading-4-key-shows-stop.png` |
   | The stop button | back to IDLE | `reading-3-stopped.png` |

   Follow-along while a list scrolls was not run (owed, below).
5. **A crash with the library loaded.** `am crash` raised the notification, which opened the crash
   screen with Rust's report (`crash-1-notification.png`, `crash-2-screen.png`).
   * POST_NOTIFICATIONS was granted for this check, then revoked and its flags cleared.
   * The report's shape was checked, and it held no user text.
   * The screen read and deleted 12 older reports from earlier sessions, as it always does.
6. **The native-load failure**, forced before startup, passed. See
   [the bootstrap contract](#the-bootstrap-contract).

**Restored afterwards** (`final-restored-keyboard.png`):

* The plain debug build is installed again.
* The preferences match the saved copy by hash, except `internal__version_last_use`, which every
  new build rewrites.
* Our keyboard is selected, media volume is 0 and Rust drawing is on.
* No accessibility services are enabled. `accessibility_enabled` reads 0, because the system
  derives it from whether a client is bound.
* POST_NOTIFICATIONS is revoked again, and the last crash time is as before.
* The fixture files are removed. A `settings-device-prepare-death.json` from 2026-11-15 predates
  this work and was left alone.

## Kotlin files and the ledger

Production counted lines, as `scripts/jvm-ledger.py` counts them, main 218de9e55 → branch:

| File | main | branch |
|---|---:|---:|
| `AW/gif/GifSearchSession.kt` (added) | | 162 |
| `AW/reader/Reading.kt` (added) | | 127 |
| `AW/reader/LoadedVoice.kt` (added) | | 63 |
| `FB/lib/crashutility/AfterCrash.kt` (added) | | 30 |
| `AW/gif/GifResultCache.kt` (deleted) | 26 | |
| `lib/rust/.../rust/Gif.kt` (deleted) | 3 | |
| `AW/gif/GifSearch.kt` | 150 | 25 |
| `AW/reader/ReadAloudService.kt` | 207 | 110 |
| `AW/reader/VoiceEngine.kt` | 107 | 71 |
| `FB/lib/crashutility/CrashDialogActivity.kt` | 120 | 95 |
| `AW/gif/GifRecents.kt` | 49 | 25 |
| `AW/dictation/MicAudio.kt` | 84 | 68 |
| `AW/gif/Giphy.kt` | 54 | 39 |
| `FB/lib/crashutility/CrashUtility.kt` | 275 | 271 |
| `AW/dictation/MicRecorder.kt` | 50 | 47 |
| `AW/gif/GifImages.kt` | 50 | 49 |
| `AW/reader/PageText.kt` | 52 | 52 |
| `AW/gif/GifSender.kt` | 60 | 94 |
| `AW/dictation/ModelCheck.kt` | 59 | 63 |
| `AW/reader/ScreenWalk.kt` | 67 | 71 |
| `AW/reader/ReadAloud.kt` | 87 | 90 |
| `AW/gif/GifDataSource.kt` | 6 | 8 |
| `FB/ime/clipboard/provider/ClipboardMediaProvider.kt` | 147 | 148 |
| **These 23** | **1,653** | **1,708** |

The production Kotlin grew by 55 counted lines:

* The four new files (382 lines) run Rust's plans. Before, the decisions were mixed into the code
  that carried them out; now each effect is named and run on its own, and that takes lines.
* The files the decisions left lost 327 lines between them.
* Six rows were raised, each with its reason in the ledger's note: `GifSender` +34 (it runs the send
  steps through a seam the trace drives), `ModelCheck` +4, `ScreenWalk` +4, `ReadAloud` +3,
  `GifDataSource` +2 and `ClipboardMediaProvider` +1.

The ledger has 406 rows (main 404): four added and two deleted. Twelve rows moved from M to A:
`MicAudio`, `MicRecorder`, `ModelCheck`, `GifImages`, `GifSearch`, `GifSender`, `Giphy`,
`ReadAloud`, `ReadAloudService`, `VoiceEngine`, `CrashDialogActivity` and `CrashUtility`.
`GifRecents` stays M, because it still holds the recents JSON. `scripts/jvm-ledger.py --tighten`
finds nothing left to lower: every row is at its file's count.

**Tests and harness** (lines):

* Added:
  * GIF: `GifSearchTraceTest` 424, `ReferenceGifSearch` 268, `GifHarness` 165,
    `GiphyTransportTest` 91, `ReferenceLruCache` 36, `GifSessionPlanTest` (`lib/rust`) 97.
  * Dictation: `MicTraceTest` 226, `ReferenceMic` 100, `ModelCheckTraceTest` 123,
    `ReferenceModelCheck` 50.
  * Read-aloud: `ReadingTraceTest` 237, `ReferenceReading` 180, `LoadedVoiceTraceTest` 201,
    `ReferenceLoadedVoice` 82.
  * Crash: `CrashTraceTest` 196, `ReferenceCrash` 90.
* The `gifFixture` source set: `GifFixtureSource` 57, `GifFixtureReceiver` 29, and its manifest 8.
* Scripts: `scripts/edges-mutations.py` 377 and `scripts/gif-fixture.py` 168.
* Deleted: `GifResultCacheTest` (49).
* Changed:
  * `GifComparison` 236 → 160, `GifRulesTest` 85 → 55, `GifRulesMatchTest` 62 → 54 and
    `LimitedReadTest` 34 → 21.
  * `GifSenderFilesTest` 71 → 108, `GifRecentsTest` 65 → 83, `GiphyTest` 81 → 99,
    `GifChipsTest` 32 → 50, `GiphyComparison` 396 → 406 and `ReferenceGiphy` 69 → 73.
  * `SettingsInstrumentation` 264 → 309, `ProbeActivity.java` 269 → 307 (the `images` field) and
    `settings-device-tests.py` 162 → 165.
  * `SettingsDeathActivity` (the typed edit codes main's change had broken, 44cfde61c), and one
    line each in `DictationInstrumentation`, `ProductionListenerComparison` and
    `AudioHistoryTest`.

## Owed

In [owed checks](owed-checks.md):

* **P31**: GIF on the phone with a GIPHY key: live search, paging, sends and recents. Live API
  acceptance is not claimed here.
* **P32**: dictation on the phone's real microphone: record, cancel, and a missing or damaged model.
* **P33**: read-aloud heard on the phone: the neural voice, follow-along through a long scrolling
  list, and the voice freed after idle.
* **P34**: a crash on the phone: notification, screen and report, and the crash loop stepping the
  keyboard aside.
* **E49**: follow-along while a list scrolls on the emulator. The probe has no long list, and
  other apps' screen text can't be inspected.
* **E50**: the crash loop on the emulator, with two crashes inside 5 seconds.
* **E51**: `scripts/settings-device-tests.py crash`, end to end, in light and dark. Its crash mode
  was run directly with `am instrument`, in light. The runner changed only to create the failure
  file first.
