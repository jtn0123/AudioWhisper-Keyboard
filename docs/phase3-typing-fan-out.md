# Phase 3, item 1: the typing-event fan-out (A7)

Phase 3 of [the all-Rust plan](all-rust.md#3-rust-owns-the-keyboard-l) hands the keyboard to Rust
one event family at a time. Item 1 is the typing-event fan-out, grade item A7. Before this work,
one typing event fanned out from Kotlin into a string of separate calls. A space after a word
asked the session for the auto-commit candidate, then taught the tap trail, then learned the
word, then counted it, then asked for a save for each of those, then published the suggestion
row, with Kotlin deciding the order and what each answer meant. Now each typing event is **one
call that returns one plan**, and Kotlin only carries the event in and does what the plan asks
of Android.

The same item moves the smartbar's choices, from `RendererSmartbar.kt` lines 98–132 at f1fdf3f34,
into Rust: each action's icon, label and state, the arrow, the toggle's icon, and what the toggle
flips.

Branch `all-rust/typing-fan-out`, started from f1fdf3f34 and rebased onto 0d7196c8d (the theme
property editor). Its commits:

* ae452c49b: one call per typing event, one plan back.
* fa8d43c7c: the smartbar's choices made in Rust.
* 2e2ddc15e: the mutation script.
* This page and the results.

## What moved

### The typing plan

`nlp::Session::event` (`lib/rust/crates/nlp/src/typing.rs`) takes one event row and answers one
plan. In the order main's Kotlin fan-out used, the session itself:

* counts the event (`TypingCounters`' counts, now kept only in Rust);
* teaches the tap trail the word it finished, or tells it a tapped character was deleted;
* learns the finished, picked or swiped word, or takes back a learned auto-correction when it is
  undone;
* arms the undo for an auto-correction, and answers a backspace straight after it with what to
  put back;
* keeps a word the user undid a correction of, so it is not corrected again this session.

The undo-autocorrect arming moved out of `KeyboardManager` (its `autoCorrectionUndo` field and
the `AutoCorrectionUndo` class) into the session. `WordLearner` and `AutoCorrectionUndo` left
production for the test sources, where only the frozen reference uses them.

### The smartbar's choices

`render::smartbar_actions` (`lib/rust/crates/render/src/smartbar_actions.rs`) answers, for the
smartbar's packed config and a row of facts the config lacks, the following:

* whether the actions are expanded, and whether the toggle flips the extended actions or the
  shared ones;
* each action's code and flags: enabled, dictating, warming, finishing, the microphone level
  in eight bits, and reading aloud;
* each action's icon: the key's own, the toggle's, stop while reading aloud, or none;
* each action's label: a text action's first character, or "?" when there is nothing to show.

Kotlin asks through the generated `SmartbarRules.actions`.

The arrow points left exactly when *flip toggles* is on, or when the actions are expanded and
the incognito indicator is set to show behind the keys, but not both. The toggle shows an unfold
icon in the extended layout, the incognito icon when incognito replaces the toggle, and the
arrow otherwise.

## The wire shapes

All of it is declared in `crates/wire` and generated on both sides. Nothing is hand-written at
the boundary: 391 generated natives (two more than main: `NlpSession.event` and
`SmartbarRules.actions`), 0 hand-written, 0 mismatched.

`wire::typing` (`NlpSession.event(event, context, candidate, lookup)`):

* **`NlpEventKind`** has twelve kinds:
  * `KEY` 0, `TYPED` 1, `TAP_DELETED` 2, `BACKSPACE` 3, `UNDO` 4;
  * `FINISHED` 5, `PICKED` 6, `SWIPED` 7;
  * `DISARM` 8, `UNDO_CORRECTION` 9, `CORRECTION_UNDONE` 10, `FIELD` 11.

  Kinds 5, 6, 7 and 9 also take an `NlpContext` row. The session configures itself from that
  row first, as the editor stands.
* **`NlpEvent`** is one row per event with 17 fields:
  * `KIND`, and `NOW` (the uptime that saves are timed by);
  * `AIM` (whether to learn where this user taps) and `TRIGGER`;
  * the word: `WORD`, `BEFORE_COMPOSING`, `COMPOSING_START`, `COMPOSING_VALID`;
  * the selection: `TEXT_BEFORE_SELECTION`, `SELECTION_START`, `SELECTION_END`, `CURSOR_MODE`;
  * a tapped candidate: `TYPED`, `CANDIDATE`, `WORD_CANDIDATE`, `COMMIT`, `CORRECTS`.

  A kind reads only its own fields; the rest are empty.
* **`NlpEventPlan`** is the answer:
  * `SAVE` and `SAVE_AGAIN`: milliseconds until a save, or -1 for none. `SAVE_AGAIN` is a
    second save, asked for when a save that was due has not run yet.
  * `KEPT`: write the kept words now.
  * `ACCEPTED`: the tapped candidate may go in.
  * `REVERT`, `REVERT_TEXT`, `PHANTOM`: how many characters to replace to undo an
    auto-correction, what goes in their place, and whether to leave a phantom space.
  * `PUBLISH`: an `NlpSnapshot` row follows, to publish as the suggestion row.

Key presses and the screen's orientation still go straight to the tap trail. They are the hit
test's input on the touch path, and nothing in the plan depends on them.

`wire::smartbar` (`SmartbarRules.actions(config, fields)`):

* **The request** is one `SmartbarBar` row, then one `SmartbarAction` row per action.
  * `SmartbarBar`: `INCOGNITO_DISPLAY`, `INCOGNITO`, `LISTENING`, `WARMING`, `LEVEL`.
  * `SmartbarAction`: `KIND` (`NONE` 0, `KEY` 1, `TEXT` 2), `CODE`, `ENABLED`, `KEY_ICON`,
    `TEXT`.
  * Layout, flip, the shared and extended settings, dictation and reading aloud come from the
    packed config the smartbar already sends.
* **The answer** is one `SmartbarPlan` row (`EXPANDED`, `TOGGLES_EXTENDED`), then one
  `SmartbarActionPlan` row per action: `CODE`, `FLAGS`, `ICON`, `LABEL`, `KEY_LABEL`.
* **`ICON`** is a `SmartbarIcon`:
  * `NONE` 0, `KEY` 1 (the key's own icon), `ARROW_LEFT` 2, `ARROW_RIGHT` 3;
  * `UNFOLD_LESS` 4, `UNFOLD_MORE` 5, `INCOGNITO` 6, `STOP` 7.
* **`KEY_LABEL`** asks Kotlin for the key's own label.

## What Kotlin still does, and why

| File | Counted lines | What it does | Why it stays in Kotlin |
|---|---:|---|---|
| `nlp/TypingEvents.kt` (new) | 67 | Builds one `NlpEvent` row per event from `EditorContent`, sends it, starts the save timers and writes the kept words when the plan says so | The editor's facts come from Android's `InputConnection`, the timers are coroutines on the service's scope, and the kept words are a file in the app's private storage |
| `nlp/NlpManager.kt` | 343 → 289 | The worker and transport, candidate publication, and the provider's IO | The provider's dictionary and Room IO, and `SystemClock` |
| `nlp/NlpState.kt` | 118 → 117 | Builds the context row and projects the protocol | `Context` and `InputConnection` facts |
| `keyboard/KeyboardEditing.kt` | 150 → 137 | Commits, deletes, selects, and applies the revert the plan returns | `KeyEvent` and `InputConnection` |
| `keyboard/KeyboardManager.kt` | 240 → 232 | Coordinates input, editor, NLP and state | `Context` and `Toast` |
| `nlp/TypingCounters.kt` | 30 → 22 | Reads and writes the counter file | Private file IO; the counting moved to Rust |
| `suggest/TapTrail.kt` | 37 → 28 | The tap trail's handle | ART JNI ownership |
| `suggest/KeptWords.kt` | 34 → 33 | The kept-word store's file | `Context` files |
| `nlp/latin/LatinLanguageProvider.kt` | 330 → 326 | The dictionary provider | Room, `File` and `SystemClock` |
| `render/RendererSmartbar.kt` | 126 → 152 | Writes the bar's and each action's row from the key evaluator, maps each `SmartbarIcon` to its Compose vector, and binds the toggle to the settings | The evaluator's answers (enabled, icon, label) come from Compose-bound `KeyData` and `ImageVector`s, and Compose files are frozen |
| `suggest/AutoCorrectionUndo.kt` | 8 → moved to tests | | Only the frozen reference uses it |
| `suggest/WordLearner.kt` | 16 → moved to tests | | Only the frozen reference uses it |

Production Kotlin is 29 counted lines smaller in all:

* shrunk: 98 lines;
* removed (moved to tests): 24;
* added: 67 (`TypingEvents.kt`) plus 26 (`RendererSmartbar.kt`).

`RendererSmartbar.kt` grows because its decisions left, but writing the rows and mapping icon
ids to vectors arrived. Its ledger row is raised with that reason.

The ledger has 427 rows, down from main's 428:

* `AutoCorrectionUndo` and `WordLearner` left;
* `TypingEvents` arrived;
* every shrunk file's budget was tightened to its count (`scripts/jvm-ledger.py --tighten`).

Each file has its ruling in `docs/stage5-inventory.md` and in `docs/jvm-ledger.tsv`.

## Proof

### Seeded 50,000-event traces

`TypingTraceMatchTest` is the committed host recorder. It runs synthetic typing through two
sides at once, each with its own native stores, dictionary and disk:

* **Main's side:** main's Kotlin fan-out, frozen from f1fdf3f34 in `ReferenceTypingFanOut.kt`.
  Phase 6 swapped it for golden digests of its three seeded traces ([record](phase6-kotlin-references.md)).
* **The branch's side:** `TypingEvents` and the plan.

The typing is drawn from a seed. It has:

* words typed letter by letter, with typos;
* spaces and punctuation that auto-correct;
* strip picks and removals, and swipes;
* backspaces that undo corrections, and the undo key;
* cursor moves and selections, new fields, setting changes and orientation changes;
* save timers that fire;
* saves cancelled when the process dies, after which a new process starts from what was
  written;
* service recreation.

After every event, the two sides must agree on all of the following, and the run stops at the
first event that differs:

* what Android was asked to do, in order: kept-word writes and save timers;
* what went into the editor;
* the suggestion row and the smartbar's row;
* every store: learned words, kept words, tap aim and counters;
* what was written to disk.

The input is synthetic and seeded. Nothing typed by a person is in it, and the test prints only
counts and a digest of the records.

| Seed | Events | Result | Digest of every record |
|---|---:|---|---|
| 20261122 | 50,000 | identical | `3ffdf95a…d25a` |
| 7 | 50,000 | identical | `6830bdd3…c448d` |
| 424242 | 50,000 | identical | `df986071…504a` |

Each seed mixes the events in about the same proportions. Seed 20261122 had:

* 28,546 letters, 4,454 spaces, 671 Enters and 1,625 punctuation marks;
* 4,033 backspaces and 477 undo-key presses;
* 2,643 strip picks, 360 strip removals and 1,653 swipes;
* 457 cursor moves, 123 selections and 230 cleared editors;
* 325 new fields, 137 service recreations and 100 process deaths;
* 473 setting changes, 72 orientation changes and 1,206 idle gaps.

Over those events, main's side asked for the following, and the branch's side asked for the same
at every event:

* 1,421 save requests and 1,372 save timers that fired;
* 94 kept-word writes;
* 6,079 auto-corrections and 114 undone corrections.

Across the three seeds:

* the learned words changed after 1,595 to 1,805 events;
* the kept words after 93 to 113;
* the tap aim after 307 to 382;
* the counters after 3,804 to 4,028;
* the disk after 1,455 to 1,545;
* the smartbar's row after 4,281 to 4,509.

The test fails if any event kind, effect or record part was never reached. That includes picks,
swipes, undos, recreation, deaths, saves that fired, kept-word writes, and changes to every store
and the disk.

**One intended difference.** Main read the clock once for each save it asked for, and one event
could ask several times. The plan reads the event's `NOW` once. Within one event these are
microseconds apart on a phone. The trace's clock does not move within an event, so it cannot
see the difference; the save delays are equal.

### The smartbar

`SmartbarActionsMatchTest` runs 20,000 seeded smartbars through two sides:

* a frozen copy of main's choices (`ReferenceSmartbarActions.kt`, `RendererSmartbar.kt` lines
  98 and 102–126 at f1fdf3f34);
* the Rust plan through `SmartbarShown`.

The smartbars vary every layout, flip, setting, incognito mode, dictation state, reading state
and microphone level, including NaN and the infinities. Their actions include empty slots, keys
and texts. The two sides must agree on what is expanded, what the toggle flips, each action's
code and flags, each icon, and each label. They are identical except for two labels that change
on purpose, each counted and checked against main's value:

* **An empty text action** shows "?" (1,774 cases). Main showed "null", because
  `firstOrNull()` of an empty string is null and its `toString()` is "null".
* **A text starting with a surrogate pair** shows the whole character (3,593 cases). Main showed
  half of it, a lone high surrogate.

The preview, which folds the shared actions away, is covered too (2,564 cases). Main's arrow
reads the setting itself, so the Rust side reads the raw setting, not the preview's folded one.

### Cargo tests

* **`nlp` (`typing_tests.rs`):** 13 tests, by event family:
  * a key and a tapped character count, and ask for no save yet;
  * a backspace drops a waiting swipe, counts, and saves once; the undo key takes a swipe back;
  * a finished word is learned, counted and published; a word that goes on finishes nothing;
    incognito neither learns nor counts;
  * a pick goes in only when it was shown, and is learned only when committed;
  * an auto-correction arms its undo whether or not it went in; a backspace after it puts back
    what was typed; an undo needs a cursor and the correction still in place;
  * a new field forgets and publishes;
  * a save that was due and had not run asks for two;
  * a malformed event is refused.
* **`render` (`tests/smartbar_actions.rs`):** 13 tests:
  * the arrow, the extended layout, and incognito;
  * empty slots and enabled keys;
  * the microphone level, including NaN and the infinities;
  * reading aloud, key icons and labels, and text labels;
  * the packed call, the preview arrow, and Kotlin's float spellings;
  * refusals.

### Mutations

`scripts/typing-plan-mutations.py` makes 41 faults, one at a time. Each fault runs
every target it names, and the result records which targets caught it:

* the typing plan's Rust;
* `TypingEvents.kt` and `NlpState.kt`;
* the smartbar's Rust;
* `RendererSmartbar.kt`.

A case counts as caught only by a fresh failing assertion, never by a failure to compile. The
source bytes are restored afterwards, and the restored targets are run again to check that they
pass.

| Faulted | Cases | Caught by |
|---|---:|---|
| `nlp/src/typing.rs` (the typing plan) | 17 | 7 by both the cargo tests and the trace, 4 by the cargo tests alone, 6 by the trace alone |
| `nlp/TypingEvents.kt` | 5 | the trace and events tests |
| `nlp/NlpState.kt` | 2 | the trace |
| `render/src/smartbar_actions.rs` (the smartbar plan) | 13 | all 13 by both the cargo tests and the match test |
| `render/RendererSmartbar.kt` | 4 | the match test |
| **All** | **41** | **41** |

The trace missed four typing-plan faults that only the cargo tests caught:

* arming the undo only when the correction went in, which matters only when a correction is
  refused;
* undoing with a selection;
* losing the second save, which a save already due and not yet run asks for;
* not publishing after an undo.

The trace's typing never showed a difference for these. Each needs a state it reaches rarely or
not at all, or one where another effect hides the fault. The cargo tests set up each state
directly, and catch all four.

In the other direction, six faults only the trace caught. Each is an effect the cargo tests do
not observe:

* a kept word written that was never remembered;
* a pick learned for the wrong word;
* an undone correction counted twice;
* the aim setting ignored;
* a key not counted;
* a pick that never saves.

The case list, and what caught each case, is in the script and in
`build/typing-fan-out/mutations/result.json`.

### Digests

All three digests are unchanged:

* The 300-press digest `809d588b…` and the 8-popup digest `c14d19d2…` came from
  `scripts/render-benchmark/run.sh` on `aw_keyboard` (emulator-5558) with the branch build. The
  first run typed 300 units with a different digest, `8e796591…`, and is kept as
  `build/typing-fan-out/render-benchmark-1-flaked`. That is the flake already owed as E29, which
  hits main too. The run straight after it matched.
* The autocorrect-on-space digest `6e5ce6af…0051c` ("teh" and a space giving "the ") was
  reproduced in the device checks below.

### Probes

* **Reshow:** all 16 trials drew their keys. Each of the four ways of hiding (the keyboard's
  own Hide, Back, a tap outside, another app) was tried at gaps of 100, 300, 600 and 1,000 ms.
* **Reading order:** captured in five states (letters, expanded, collapsed, the row of words,
  and a popup). `compare-a11y.py --order` found no ordering issue.

  The first attempt stopped at the row-of-words capture. The probe app on the emulator was the
  2026-11-14 build, older than its redacted-capture change of 2026-11-17. It was rebuilt from
  `scripts/render-benchmark/build.sh` and installed on emulator-5558. Before that, the reshow
  probe's first attempt stopped on "Accessibility service did not bind", the case already owed
  as E30, and closing and reopening the device session freed it.

### Device checks (emulator-5558, the branch build)

All input was synthetic. The screenshots are in `build/typing-fan-out/device/` and are not
committed.

| Check | Result | Screenshot |
|---|---|---|
| "teh" and a space | "the ", digest `6e5ce6af…0051c`, next-word suggestions shown | `01-teh-space.png` |
| A backspace straight after | "teh" put back | `02-backspace-undo.png` |
| A pick from the strip | The typed word, then a completion, went in; the next letter took a phantom space | `03a-typing-hel.png`, `03b-picked.png`, `03c-picked-completion.png`, `03d-phantom-space.png` |
| Two quick spaces | ". " (digest `1ddd7c6d…`) | `04-double-space.png` |
| A glide word | Went in composing, with alternatives on the strip | `05-glide.png` |
| The smartbar, collapsed and expanded | "<" collapsed and ">" expanded with the default *flip toggles* and the indicator behind the keys; every action icon drawn | `06a-smartbar-collapsed.png`, `06b-smartbar-expanded.png` |
| Incognito on and off | The action's icon changed, the indicator showed behind the keys, and it was turned off again | `06c-incognito-expanded.png`, `06d-incognito-collapsed.png`, `06e-incognito-off-expanded.png`, `06f-restored-collapsed.png` |
| Learning across a restart | A made-up word typed twice was offered for its prefix, and was still offered after the keyboard's process was stopped and the keyboard came back | `07b-before-restart.png`, `07c-after-restart.png` |

Notes on the checks:

* **Two spaces.** The first try, with two separate presses, typed two spaces. The presses were
  further apart than the double-tap window. Two presses 120 ms apart typed ". ".
* **Learned words.** A learned word is offered as a completion only once it has been typed
  twice (`KNOWN_AFTER`).

What was restored or left:

* Incognito is back to its default.
* The editor was cleared and the IME set back to ours.
* Accessibility is off again.
* The probe app is now the current build.
* The made-up word stays among the emulator's learned words. The only way to remove it is
  "Forget learned words", which would also erase other fixtures' learning, so it was left.

### The gate

The full gate (`test`, `lintDebug`, `lib:rust:check`, `checkNativeDeclarations`,
`checkJvmLedger`, `assembleDebug`, `compileDebugAndroidTestKotlin`) passed on the branch:

| | main (0d7196c8d) | branch, rebased |
|---|---:|---:|
| JVM tests | 985 in 240 files | 993 in 243 files |
| Cargo results | 1,298 | 1,325 |
| Generated natives | 389 | 391 |
| Ledger rows | 428 | 427 |

Nothing failed or was skipped. The first run after the rebase failed
`ComposeStabilityLinkageTest`: Compose callers compiled before the rebase still referenced
stability fields that main's change had removed. It passed after `:app:clean`. Lint found only the four allowed IDs
(AndroidGradlePluginVersion ×4, NewerVersionAvailable ×4, ChromeOsAbiSupport ×1,
GradleDependency ×1).

### CPU trials

Four trials alternated main (f1fdf3f34) and the branch, two of each, into
`build/cpu-p95/typing-fan-out/`. The full entry is in
[cpu-p95-results.md](cpu-p95-results.md#the-typing-event-fan-out-2026-11-22).

| Trial | p95 | CPU time p95 | Over 2 ms | Touch to post p95 |
|---|---:|---:|---:|---:|
| main 1 | 3.465 | 1.655 | 15.2% | 261.3 |
| branch 1 | 3.557 | 1.602 | 13.6% | 272.4 |
| main 2 | 3.366 | 1.697 | 14.6% | 263.7 |
| branch 2 | 3.084 | 1.529 | 14.1% | 288.3 |

All four typed the expected text. Neither build met the 2 ms wall-clock gate, so this is not a
pass.

The branch is no slower beyond noise:

* its frame p95 falls one trial on each side of main's;
* its CPU time and its share of frames over 2 ms are lower in both trials;
* its touch-to-post p95 is higher in both, but inside the 259–302 ms that unchanged code
  measured in the other trials the same day, with level medians.

Five tries at main's first trial were thrown out: two for E29, two for E30, and one for a System
UI ANR dialog (E35). The host was loaded by other jobs throughout.

## What is owed

Device checks are numbered after those on main (P21 and E33):

* **E34.** The smartbar on a device, in the states only the match test has shown:
  * the extended layout's unfold icons;
  * the incognito icon taking the toggle's place, with the indicator set to replace the
    toggle;
  * a text action's label.

  Each needs a setting changed and then put back.
* **P22.** On the phone: the microphone action's flags while dictating (warming, finishing and
  the level), and the stop icon while reading aloud. The emulator stays muted and has no
  microphone.
* **E35.** During the CPU trials, Android's *System UI isn't responding* dialog blocked
  agent-device under host load. `agent-device alert dismiss` chose *Close app*, which restarted
  System UI. Check that agent-device can choose *Wait*, and whether the trials should stop
  above a load threshold.

Not device checks, still owed:

* **Retire the per-step NLP operations.** These are `PICK`, `LEARN`, `AIM`, `KEEP`, `TALLY`,
  `TALLY_CHANGED` and the `FIELD` op, the learner's and tap trail's per-step natives, and
  `WordLearner` and `AutoCorrectionUndo` in the test sources. They remain because the frozen
  reference still calls them. Once the trace has served its purpose, they and the references go
  together.

## What remains for items 2–4

* **2. The key and layout object graph:** `TextKeyboard`, `TextKey` and the snapshot in
  `RustKeyboardView`. The smartbar still asks Kotlin's `ComputingEvaluator` whether a key is
  enabled and for its icon and label. Once Rust owns the key graph, those three questions can
  be answered from it, and `RendererSmartbar.kt`'s row writing goes with them.
* **3. Theme, then subtype.**
* **4. A plain-View root** in place of the Compose root, proving lifecycle, insets, Snygg
  backgrounds, font changes and live inline autofill. With it, the smartbar's icon ids can map
  to drawables rather than Compose vectors.

Within item 1, Kotlin still owns these:

* the save timers, as coroutines;
* the kept-word and counter files;
* reading the editor into the event row;
* applying a revert to the editor.

Each needs Android. Phase 3's exit also wants the phone p95 again at the end (P4).
