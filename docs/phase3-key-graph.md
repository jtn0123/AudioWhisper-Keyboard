# Phase 3, item 2: the key and layout object graph

Phase 3 of [the all-Rust plan](all-rust.md#3-rust-owns-the-keyboard-l) hands the keyboard to Rust
one piece at a time. Item 2 is the key and layout object graph: what FlorisBoard's
`TextKeyboard`, `TextKey`, `LayoutManager` and the popup sets held. Before this work, Kotlin
read the layout and popup-mapping files, merged them into a `TextKeyboard`, and computed each
`TextKey` for the keyboard's state; the Rust renderer then copied that graph into its snapshot.
Now **Rust builds and computes the keys**, and the Rust renderer draws them from Rust's graph.
Kotlin reads the files, reports the facts only Android knows, answers the questions only
Android's ICU can answer, and carries out what a key does.

The smartbar's three questions about an action's key (is it enabled, what icon, what label) are
answered from the same graph, on the bar and in the overflow and editor panels, so neither
`RendererSmartbar.kt` nor `RendererPanels.kt` asks Kotlin's evaluator.

The Compose keyboard stays as the fallback and keeps the frozen Kotlin graph until
[step K](compose-removal.md#k-the-compose-keyboard). The Rust renderer's path does not touch it.

Branch `all-rust/key-graph`, built on b10bee9b1 and rebased before the final gate onto main's
20b07e922 (step 10's extension screens, which touch neither the graph nor its path). The hashes
here are the rebased ones; every build measured on the emulator was the same commit before the
rebase. Its commits:

* 09fbc375b: the key graph built and computed in Rust (`keygraph`, its wire and natives).
* 9a7c2f2f4: the smartbar's actions answered from the graph.
* c66e98a04: the Rust renderer's keys drawn from the graph (`RustKeyGraphs`, `RustKeyboard`).
* 9ac3d23ea: the seeded differential traces against main's frozen graph.
* b9af1dc67: a dead field removed (the hint's data, written and never read).
* 2df9549f3: the ledger: the new adapters, the raises, and the Compose-only graph files.
* d58854a90: the mutation script.
* 99eb363f6: fixes found by the gate and review: `keyLabelResource` keeps main's
  `context()?.getString(R.string…)` lines, which a settings test reads from the source; the new
  Kotlin wrapped to `.editorconfig`'s 120 columns; and the trace made reproducible from its seed
  (below).
* b0fde3664: the synthetic layouts draw codes from every `KeyCode`, not a hand-picked few.
* 5a9cb89a3: the probe's field takes an IME action (search, send, go, done), set before the input
  type, for the device checks of the enter key.
* 178be4287: a unit test that a grouped key takes its group's popups and then its own, and the
  `group-ignored` mutant made to compile.
* 1bc6044be: the mutation script names a mutant that doesn't compile, rather than counting it as
  not caught.
* 1916153eb: the synthetic keyboards kept comparable with `KeyCode`s drawn (below).
* 7ed65c830: the `code-answer-ignored` mutant made to compile.
* 724862978: `scripts/render-benchmark/rebuild-timing.py`, the layout rebuild timings.
* c4847d79e: the overflow and editor panels' actions answered from the graph (`ACTIONS`).
* a75f5f60c: a test that an empty smartbar slot asks the graph nothing, for the
  `empty-slot-asked` mutant (below).
* 0754dee9b: `computeDisplayName` loses the evaluator it never read, so the panels read none of
  the smartbar's evaluator.
* 26a47e3df: the rebuild timings hold each tap for 100 ms, so its release keeps a touch time.
* 12e9247cf: a unit test that a key waiting on its case goes no further, for the mutant the
  final run left alive.
* fb1a9f000: clippy's `is_multiple_of` in the actions op's row check, found by the final gate.
* This page and the results.

## What moved

### The key graph (`lib/rust/crates/keygraph`)

| Module | What it does | Main's Kotlin it replaces |
|---|---|---|
| `parse.rs` | Reads layout and popup-mapping files as `DefaultJsonConfig` did: `$` names a key's class, a missing or unknown class reads as a text key, unknown fields are ignored, a quoted number or a key type in any case is accepted, and anything else the config rejected fails the whole file | the serializers of `TextKeyData`, the selectors and `PopupSet` |
| `node.rs` | A key as its file describes it, before any state picks or cases it | `AbstractKeyData` and its subclasses, `PopupSet`, `PopupMapping` |
| `assemble.rs` | Merges the main, modifier and extension layouts into rows, with the number row and the modifier a layout's index entry names; parses each distinct file text once per graph | `LayoutManager.mergeLayouts` and its layout cache |
| `graph.rs` | One keyboard: its rows, its symbols (for the hints), its popup mapping and the default one behind it; lists the keys, lays them out, answers the smartbar | `TextKeyboard`, `TextKeyboardCache` |
| `context.rs`, `compute.rs` | Computes each key for the state: the selectors (case, shift state, variation, layout direction, character width), auto-text casing, currency slots and multi-text keys; whether it shows and is enabled; its popup keys (its own set, the subtype's mapping and the default one by label, group and key, then the symbol and number hints); its label, hinted label and icon token | `TextKey.compute` and `computeLabelsAndDrawables`, `ComputingEvaluatorImpl`'s choices, `MutablePopupSet`'s merging |
| `data.rs` | A computed key and the popup keys gathered for it, ordered by the hint settings | `KeyData`, `PopupKeys` |
| `facts.rs` | What Android reports, and the questions only it can answer | the evaluator's reads of Android |
| `calls.rs` | Rows in, rows out | |

**Questions only Android can answer.** Casing depends on the locale's ICU rules, and whether a
code point is a non-spacing mark is ICU's call; Rust does not carry ICU's tables. So a
computation that needs one of those asks: lowercase or uppercase a code point or a text, in the
subtype's locale or `Subtype.DEFAULT`'s, or is this a mark. It carries on without the answer,
hands back every question it could not answer, and Kotlin answers them through Android's
`UCharacter` and `Locale` and computes again. A graph asks each question once and keeps the
answer, so after the first computation of a layout most computations ask nothing.

Where an answer is still missing, the value stands in for itself (a code for its own case, a
text for its own). A key whose computation used a stand-in stops there and is finished in the
next round, so nothing is looked up by a stand-in label. That rule is what the trace's third
caught bug (below) needed.

### The smartbar's three questions

`Graph::action` answers, for an action's key (its `KeyType`, code and label), whether it is
enabled, its icon token and its label. It answers as the smartbar's evaluator did: for a keyboard
of mode `SMARTBAR_QUICK_ACTIONS` and `Subtype.DEFAULT`'s language, with the facts of the
active keyboard's last computation. `render::smartbar_actions` asks it through the graph handle
that `SmartbarRules.actions` now takes. `RendererSmartbar.kt` sends each action's key data and
draws the icon token it gets back.

The overflow and editor panels ask the same three questions of every action they list, and the
graph answers those too, through the `ACTIONS` op: `RendererPanels.kt` sends each action's key
(`ActionKeys`) and reads back whether it is enabled, its icon token and its label. Before this,
the panels asked the smartbar's evaluator (`evaluateEnabled`, `computeImageVector` and
`computeLabel`), which kept part of the Kotlin graph's evaluator on the Rust renderer's path.
`computeDisplayName` took the evaluator too and never read it; the parameter is gone, so the
panels no longer read the smartbar's evaluator at all.

### The Rust renderer's keys

With the Rust renderer on, `KeyboardManager` gets each keyboard from `RustKeyGraphs` instead of
the Compose cache: built once per mode and subtype from the files, and computed for each state.
`RustKeyboard` is the `Keyboard` the rest of the input path sees. It reads the computed keys back
as rows, as `Key`s carrying `TextKeyData`, so pressing a key does what it did. The renderer's
snapshot (`RustKeyboardView`) draws from those rows, its layout from `KeyGraph.layout`.

## The wire shapes

All of it is declared in `crates/wire` (`key_graph.rs`) and generated on both sides. Nothing is
hand-written at the boundary: 396 generated natives (five more than main:
`KeyGraphNative.build`, `placeholder`, `free`, `call` and `layout`), 0 hand-written, 0
mismatched.

`KeyGraph.build(fields)` takes one `KeyGraphBuild` row:

* `MODE`, `NUMBER_HINTS`, and `LOADED`, the `KeyGraphFile` bits of the files that loaded;
* the files' texts: `MAIN`, `MODIFIER`, `OWN_MODIFIER` (the modifier the main layout's index
  entry names) and `EXTENSION`, the same four for the symbols keyboard the hints come from, then
  `MAPPING` and `FALLBACK` (the subtype's popup mapping and the default one).

`KeyGraph.call(op, fields)` takes a `KeyGraphOp`:

* **`COMPUTE`**: a `KeyGraphFacts` row (state, input type, IME options, the utility key and its
  action, the language switch, a rich editor, locked, pastable, incognito, no learning, the
  subtype count, dictation, the subtype's language name and the default one, the three hint
  settings), then a code and label per currency slot, then a token and label per label resource.
  The answer is a `KeyGraphQuestion` row (`ASK`, `LOCALE`, `VALUE`) per open question, or
  nothing once every key is computed.
* **`ANSWER`**: one answer per question, in order. `KeyGraphAsk` is `LOWER_CODE`,
  `UPPER_CODE`, `LOWER_TEXT`, `UPPER_TEXT` or `MARK`; `KeyGraphLocale` is `SUBTYPE` or
  `DEFAULT`.
* **`KEYS`**: a `KeyGraphKey` row per listed key: `VISIBLE`, `ENABLED`, its data (`TYPE`, `CODE`,
  `LABEL`, `GROUP`, `POPUP`, `MULTI`), `LABELED` and `SHOWN`, `HINTED` and `HINT`, `ICON`, and
  `BASE`, its layout entry's code before any state computed it.
* **`POPUP_FLAGS`** and **`POPUPS`**: for a `KeyGraphHints` row (symbol and number hint modes,
  merge), whether each key has popup keys, and one key's popup keys as `KeyGraphData` rows,
  prioritized first.
* **`CODE_POINTS`** (a multi-text key's) and **`INFO`** (`MODE`, `ROWS`, `LISTED`).
* **`ACTIONS`**: a `KeyGraphAction` row (`TYPE`, `CODE`, `LABEL`) per smartbar action's key; a
  `KeyGraphActionAnswer` row (`ENABLED`, `ICON`, `LABELED`, `LABEL`) per action, answered as
  `Graph::action` answers the smartbar.

A popup set and a multi-text key cross as ids in the graph, so Kotlin hands out one object per
id, as main's caches shared one object per file.

`KeyGraph.layout(numbers)` lays the keys out for the keyboard's width and height, row height and
margins, and returns ten numbers per key.

`wire::smartbar` changed with the smartbar: an action's row is now `KIND`, `CODE`, `TYPE`,
`LABEL` and `TEXT` (the evaluator's `ENABLED` and `KEY_ICON` flags are gone), and the plan's
`KEY_ICON` is the icon token itself and `LABEL` the label shown (the `KEY_LABEL` flag is gone).

## What Kotlin still does, and why

| File | Counted lines | What it does | Why it stays in Kotlin |
|---|---:|---|---|
| `render/KeyGraphRows.kt` (new) | 86 | Builds the build row from the files a mode needs and the facts row from the evaluator, and answers the graph's questions | The files come from the extension manager's archives, the facts from `EditorInfo`, the keyguard, the clipboard and Jetpref, and the answers from Android's ICU |
| `render/RustKeyGraphs.kt` (new) | 83 | Keeps a graph per mode and subtype, reads the files, and runs the question-and-answer loop | `Context`, the extension manager and resources |
| `render/RustKeyboard.kt` (new) | 104 | The `Keyboard` the input path sees: the graph's keys read back as `Key`s with `TextKeyData` | The input path and a key's platform effects take `KeyData` objects |
| `keyboard/KeyboardManager.kt` | 232 → 246 | Gets the Rust renderer's keyboards from `RustKeyGraphs`, clears them with the Compose cache, and recomputes when the renderer is switched | `Context` and `Toast`; the Compose branch stays until step K |
| `keyboard/ComputingEvaluator.kt` | 202 → 209 | The label resources and icon vectors, split out of the evaluator so they are looked up by token | `Resources` and `ImageVector` |
| `keyboard/Keyboard.kt` | 72 → 73 | `rowCount` on the base class, so the height is sized from a Rust keyboard too | |
| `text/keyboard/TextKeyboardLayout.kt` (Compose) | 305 → 306 | A crash fix: a Rust keyboard still in the evaluator in the frame after switching back to Compose draws the loading placeholder instead of failing a cast | Frozen Compose; a crash fix is allowed |
| `render/RendererSmartbar.kt` | 152 → 150 | Sends each action's key data and draws the icon token | `ImageVector`s |
| `render/RendererPanels.kt` | 45 → 64 | `ActionKeys`: sends each panel action's key to `ACTIONS` and reads back its enabled state, icon token and label | `ImageVector`s and string resources |
| `render/RustKeyboardView.kt` | 410 → 404 | Draws the snapshot from the graph's rows | `SurfaceView` |
| `render/RustKeyboardLayout.kt` | 92 → 91 | | |
| `keyboard/FlorisImeSizing.kt` | 145 → 144 | Sizes from any keyboard's row count | |

Two files changed without changing their counts: `smartbar/quickaction/QuickAction.kt` (134
nonblank lines; `computeDisplayName` lost its unused parameter) and its Compose caller
`QuickActionButton.kt` (a mechanical change at two call sites).

Hand-written production Kotlin grew by 305 counted lines: 273 in the three new adapters, 42 in
the five files that grew, less 10 in the four that shrank. Item 2 does not delete Kotlin,
because the Compose fallback keeps the Kotlin graph until step K; what it does is take that
graph off the Rust renderer's path, so step K can delete it outright (below).

The ledger has 424 rows, three more than main's 421 (the new adapters). Each raise has its
reason in the note column, and the shrunk files were tightened.

### Kotlin graph files that now serve only the Compose fallback

The Rust renderer's path reaches none of these. Their ledger class is now **K** (deleted at
compose-removal step K), and the inventory says so:

* `ime/keyboard/LayoutManager.kt` (168 counted lines): reads and merges the layout files into
  the Kotlin graph. Only `KeyboardManager`'s Compose branch calls it.
* `ime/text/keyboard/TextKeyboardCache.kt` (20).
* `ime/text/keyboard/TextKey.kt` (147).
* `ime/text/keyboard/TextKeyboard.kt` (123). `PlaceholderLoadingKeyboard` in `Keyboard.kt` is
  still one; step K makes it something else.

Parts of four more serve only the fallback, noted in their ledger rows:

* `ime/keyboard/KeyData.kt`: the selectors and every `compute`. `KeyData` stays, as the input
  path's key data.
* `ime/text/keyboard/TextKeyData.kt`: `compute` and the auto-text casing. The data classes carry
  the Rust graph's keys to the input path.
* `ime/popup/PopupSet.kt`: `MutablePopupSet` and its merging. `PopupSet` stays as the popup
  identity a Rust key carries, `PopupKeys` as what a press shows.
* `ime/keyboard/LayoutAssembly.kt`: `merge` and `hints`. `plan` and `modifier` name the files the
  Rust graph is built from.

## Proof

### The differential traces

`KeyGraphMatchTest` (with `KeyGraphTrace.kt`, `KeyGraphTraceInputs.kt` and
`SyntheticLayouts.kt`) runs main's Kotlin key graph and the Rust graph side by side. Main's graph is
frozen, test-only, in `ReferenceKeyGraph.kt`: b10bee9b1's `TextKeyboard`, `TextKey.compute`,
`LayoutManager.mergeLayouts`, the popup merging and the evaluator's choices, with ICU4J standing
in for Android's ICU. The same synthetic platform answers both sides' casing and mark questions,
so a difference is the graph's, never the platform's.

**What a state is.** Each of 50,000 states per seed picks a build (four times in five one
already built, as the keyboard cache reuses them; otherwise a new one) and facts for it, computes
both keyboards, and compares them:

* **Builds** come from the 46 bundled subtype presets (their layouts, popup mappings and
  currency sets), with any of the layout types swapped for another indexed layout. All ten
  keyboard modes are drawn: characters most of the time, then symbols, symbols 2, numeric,
  numeric advanced, phone, phone 2, and the three with no layouts of their own (unspecified and
  the two smartbar modes). The number row and number hints are drawn too. The bundled presets
  are all Latin-script, and no bundled layout is right to left, so the trace adds its own:
  16 locales with casing of their own or no Latin (`tr`, `az`, `lt`, `ar`, `he`, `fa`, `ur`,
  `ru`, `el`, `hy`, `ja`, `zh-CN`, `hi`, `th`, `nl`, `de-CH`), the right-to-left direction, and
  the non-Latin number rows, symbols and numeric-advanced layouts the assets do have.
* **Past what ships**: a layout cut in half (it fails to read), a modifier the index names that
  doesn't exist, a layout that isn't indexed, and in one build in four seeded synthetic layouts,
  modifiers and popup mappings: every key class, nested selectors, multi-text keys, currency
  slots, unknown classes and fields, codes past the BMP and `Int.MIN_VALUE`.
* **Facts**: a `KeyboardState` with every shift state (caps lock included), key variation, UI
  mode and the overflow, composing and half-width flags; input types of every class with their
  flags; IME options with every action (0–7 and past them) and the no-enter, no-learning,
  navigate, no-extract-UI, no-fullscreen and force-ASCII flags; the utility key and its action,
  the language switch, a rich editor, a locked device, a pastable clipboard, incognito, the subtype
  count, dictation; language names in several scripts, blank and empty; the three hint settings;
  and currency sets, some cut short.

**What is compared**, exactly, for every listed key in order: visible, enabled, the computed
data (class, type, code, label, group, a multi-text key's code points), the label, the hinted
label, the icon token and the layout entry's code; then each key's popup keys for the facts'
hint settings and for a second, random setting (whether it has any, then for one key in eight of
those with none, and every key with some: the counts of prioritized and other keys and each
one's data); then the layout, ten numbers per key, for a random
width (now and then zero or negative), height, row height and margins, bit for bit. Popup sets and
multi-text keys must also pair one to one by identity (main's caches shared one object per
file), so a graph that parsed a shared file twice fails even with equal contents.

**Cancellation and recreation.** One new build in three starts a computation for other facts
and abandons it, its questions open, or half the time after answering one round, as a newer
state replacing an older one does. One state in 2,000 drops every keyboard, as a recreated
service does, and so does any computation that crashes (main and the branch must then crash
alike: the same exception class, and a build main can't build must fail the branch's first
computation). One state in 20 computes twice.

**Results**: 3 seeds × 50,000 states, **identical, 0 differences**:

| | seed 20261122 | seed 7 | seed 424242 |
|---|---:|---:|---:|
| keyboards built (all modes) | 10,328 | 10,396 | 10,269 |
| keys compared with a label | 1,116,225 | 1,091,584 | 1,123,803 |
| hidden / disabled keys | 28,396 / 2,145 | 28,394 / 2,078 | 29,131 / 2,112 |
| keys with popup keys compared / popup hints | 1,246,766 / 476,674 | 1,217,590 / 472,088 | 1,265,414 / 491,368 |
| multi-text keys / keys with their own popup set | 2,364 / 282,526 | 2,767 / 270,447 | 2,657 / 280,984 |
| layouts compared (and of no width) | 48,259 (1,204) | 48,203 (1,231) | 48,207 (1,221) |
| states in a right-to-left locale | 2,253 | 2,432 | 2,420 |
| builds with synthetic layouts / modifiers / mappings | 10,624 / 5,590 / 5,333 | 11,007 / 6,009 / 5,535 | 10,567 / 5,588 / 5,165 |
| broken layout / misnamed modifier | 1,717 / 2,070 | 2,104 / 1,997 | 2,036 / 1,956 |
| abandoned with questions / after an answer | 2,275 / 1,105 | 2,192 / 1,061 | 2,114 / 997 |
| recreated / crashed alike building / computing | 37 / 90 / 447 | 26 / 102 / 464 | 30 / 100 / 472 |
| distinct icon tokens on the keys | 38 | 38 | 38 |

Every mode, shift state, key variation, IME action 0–7 and input class is asserted to have been
reached on every seed. A crash is main's own behaviour on a synthetic layout, which the branch
must reproduce (the same exception class), not a difference: for example an auto-text key whose
code is no code point, which `UCharacter.toString` gives null for, so the answer adapter throws
as main's `AutoLetterState` did. The whole output is in
`build/key-graph/trace-output-4.txt` (not committed); a run takes about 94 seconds on a
loaded host.

**The smartbar** (`SmartbarActionsMatchTest`): 20,000 seeded smartbars, each with 5–11 random
actions (none, insert-text with awkward texts, insert-key with random types, codes and labels),
random settings, dictation, read-aloud and level states, and a key graph computed for random
facts. Main's choices (`ReferenceSmartbarActions.kt` driving the frozen evaluator) and the Rust
plan answered from the graph match on whether the bar is expanded, what the toggle targets, every
code and flag, every icon and every label, **except two intended label fixes, both counted**:

* an insert-text action with empty text shows "?" where main showed "null" (1,770 actions);
* text starting with a surrogate pair shows the whole character where main showed half of it
  (3,628 actions).

Each smartbar's panels are compared too: every action the overflow and editor list, with the
`NOOP` placeholder, is sent to `ActionKeys` and its enabled state, icon token and label compared
with what main's evaluator answers (`evaluateEnabled`, `computeImageVector` and
`computeLabel`), with no exception allowed: 128,152 panel actions, of which 19,010 disabled,
73,144 with an icon and 57,590 with no label, **all identical**. The test fails unless it saw a
disabled action, an icon and an unlabeled action.

**Rust unit tests** (`cargo test -p keygraph`, 20 tests, and `render`'s `smartbar_actions`, 15):
every bundled layout and popup mapping read, QWERTY's letters cased by shift and hinted with
their numbers, an uncomputed graph laid out, the parser against the config's rules, the
questions' rounds (a label waiting on its case looks nothing up; a key waiting on its case goes
no further; only codes from the first mark up are asked about; an action is answered from the last facts), a grouped key's popups (its
group's, then its own), a file read twice parsed once, the calls' rows (cut-short and misread
rows refused, and `ACTIONS` answered from the last facts), and an empty smartbar slot asking the
graph nothing.

**What the trace caught while this was built**, each fixed before the run above:

1. Main's hints computed every symbols key before lining them up with the letters; the frozen
   reference computed only the ones that lined up, and so cased a code main never cased. The
   fault was the reference's (Rust already matched main), and the reference was fixed to match
   b10bee9b1's `LayoutAssembly.hints`.
2. Popup set identity differed at state 2,404: the Rust graph parsed a file both keyboards share
   (the number row) twice, where main's layout cache shared one object. `assemble::Reader` now
   parses each distinct text once per graph (and a unit test says so).
3. 37,717 states into a seed, the branch asked about a code main never asks about. An unanswered
   case stood in for itself and the computation carried on, so a popup set was looked up by the
   stand-in label. A key whose computation used a stand-in now stops until the next round
   (`a_label_waiting_on_its_case_looks_nothing_up`).

And one in the trace itself, found while wrapping it: its synthetic layouts sat in a `HashMap`
keyed by a `LayoutType` pair, whose hash is an enum's identity hash, so the same seed drew
different builds from one JVM to the next (every run still matched; it just wasn't the same run).
They're insertion-ordered now, and two runs print the same counts.

And two gaps in the trace's coverage, closed: the first found by the mutation run (below), the
second by the trace's own counts.

1. The synthetic layouts drew codes from a hand-picked list of 31, because `KeyCode`'s codes
   are now read from Rust and its fields are private, so the generator's reflection found none
   of them. They never used a currency slot, shift, delete, enter or the language switch, and
   the `auto-currency-cased` mutant lived (b0fde3664 reads them through the getters, sorted
   by name, and fails if fewer than 90 come back).
2. Drawing every key's code from the whole list then put a non-code-point on most auto-text
   keys, which main's casing fails, so most synthetic keyboards crashed alike before they were
   compared: multi-text keys compared fell from about 3,600 a seed to under 300. Text keys now
   take a `KeyCode` one time in three, auto-text keys a currency slot's code one in 48 and
   another `KeyCode` one in 48 (1916153eb). The table above is that run: multi-text keys are
   back to 2,364–2,767 a seed, and the keys show 38 icon tokens a seed, where the first trace
   showed 13.

### Mutations

`scripts/key-graph-mutations.py` puts 73 faults into the graph, its adapters and the smartbar's
answers, one at a time, in a worktree of its own (`/Volumes/2TB/aw-scratch/key-graph-mut`), and
restores the source after each. A fault counts as caught only by a fresh assertion: a
`panicked at` line from a Rust target, or a failed assertion in a fresh JVM test report. A
fault that doesn't compile is named as such and never counted as caught (1bc6044be). The
targets:

* `cargo:keygraph`, the keygraph crate's unit tests;
* `cargo:smartbar_actions`, the render crate's smartbar tests;
* `jvm:keygraph`, the seeded traces against main's frozen graph;
* `jvm:smartbar`, the seeded smartbars and panels against main's evaluator.

The final run, on a75f5f60c:

| Faulted file | Faults | Caught |
|---|---|---|
| `keygraph/src/compute.rs` | 9 | 9 |
| `keygraph/src/context.rs` | 19 | 18, then 19 (below) |
| `keygraph/src/facts.rs` | 4 | 4 |
| `keygraph/src/graph.rs` | 6 | 6 |
| `keygraph/src/assemble.rs` | 4 | 4 |
| `keygraph/src/parse.rs` | 5 | 5 |
| `keygraph/src/calls.rs` | 5 | 5 |
| `render/src/smartbar_actions.rs` | 4 | 4 |
| `awnative/src/render_smartbar.rs` | 1 | 1 |
| `KeyGraphRows.kt` | 6 | 6 |
| `RustKeyboard.kt` | 6 | 6 |
| `RendererSmartbar.kt` | 1 | 1 |
| `RendererPanels.kt` | 3 | 3 |

A fault can be caught by more than one target. The traces caught 57, the keygraph unit tests 24,
the smartbar comparison 11 and the render crate's tests 4. Caught by one target alone: 38 by the
traces, 5 by the smartbar comparison, 3 by the keygraph unit tests and 2 by the render crate's
tests.
The restored source passed all four targets.

The run before it, on 99eb363f6, had 68 faults and caught 64. Each of its four survivors was
fixed:

* `auto-currency-cased`: the synthetic layouts drew from 31 hand-picked codes, none of them a
  currency slot's (gap 1 above). They now draw from every `KeyCode` (b0fde3664), kept
  comparable by 1916153eb.
* `group-ignored` didn't compile. It now does, with a unit test that a grouped key takes its
  group's popups and then its own (178be4287).
* `code-answer-ignored` didn't compile, and now does (7ed65c830).
* `empty-slot-asked`: nothing observed whether an empty smartbar slot asked the graph about a
  key; a test now records each key asked about (a75f5f60c).

The final run added five faults for the panels' answers (`action-always-enabled`,
`action-always-labeled`, `panel-type-dropped`, `panel-icon-lost`, `panel-label-flag-misread`)
and caught 72 of its 73. The survivor, `stand-in-key-goes-on`, deletes the early return that stops
a key once it waits on a casing answer. The first trace reached it, but 1916153eb changed which
keys the trace draws, and the trace stopped reaching it. A unit test now pins it
(12e9247cf): a shifted auto-text letter waits on its upper case, so its popup lookup
lowercases only the answered "A", never the "a" standing in for it. Rerun alone with its
sibling `stand-in-label-looked-up`, both faults were caught (`build/key-graph/mut-recheck-6.log`),
and the restored source passed.

### Digests

Both digests are unchanged on the final build (0754dee9b), from
`scripts/render-benchmark/run.sh` on `aw_keyboard` (emulator-5558) with the Rust renderer:

* the 300-press digest `809d588bf48a1e90…` (300 UTF-16 units);
* the 8-popup digest `c14d19d26f290a24…` (308 units).

The first run on the final build typed 300 units with a different digest, `c4db55e517ae882a…`,
and is kept as `build/key-graph/render-benchmark-final-1-mismatch`. That is the flake already
owed as E29, which hits main too. The failed run left its text in the editor, so the next run
refused to start; once the editor was cleared, the run after it matched. An earlier build of the branch
(99eb363f6, before the panels moved) matched on its first run.

The autocorrect-on-space digest `6e5ce6af…0051c` ("teh" and a space giving "the ", 4 units) was
reproduced on the final build, and in the device checks below on main's build and the branch's.
Each ran with the device's kept-as-typed and learned-word files moved aside (E39).

### Probes

Both ran on the final build (0754dee9b), and on 99eb363f6 before it with the same result.

* **Reshow** (`reshow.py`): all 16 trials drew their keys, every key row matching. Each of the
  four ways of hiding (the keyboard's own Hide, Back, a tap outside, another app) was tried at
  gaps of 100, 300, 600 and 1,000 ms. Results in `build/key-graph/reshow-final/`.
* **Reading order** (`a11y-order.py`): captured in five states (letters 62 nodes, expanded 68,
  collapsed 62, the row of words 64, a popup 74). `compare-a11y.py --order` found no ordering
  issue and exited 0. Results in `build/key-graph/a11y-order-final/`.

### Device checks (emulator-5558, the branch build)

All input was synthetic, on the probe's field (`scripts/render-benchmark/ProbeActivity.java`)
with the Rust renderer on. Rows up to recreation ran on the build before the panels moved to the
graph (c4847d79e), whose keyboard code is otherwise the final one; the last row ran on the final
build. The screenshots are in `build/key-graph/device/` and are not committed.

| Check | Result | Screenshot |
|---|---|---|
| "teh" and a space | "the " on main's build and the branch's, each after the kept-as-typed and learned-word files were moved aside (an earlier check had kept "teh" as typed) | `01-teh-space.png` |
| Shift, then caps lock | Capital labels after one shift; after a double shift the shift key drew the caps-lock icon, highlighted, over capital labels | `02a-shift.png`, `02b-caps-lock.png` |
| Symbols, symbols 2, numeric advanced | Each layout drawn, with its mode key's label | `03a-symbols.png`, `03b-symbols2.png`, `03c-numeric-advanced.png` |
| A number field, a phone field, phone 2 | The numeric and phone layouts, and phone 2 from its key | `04a-number.png`, `04b-phone.png`, `04c-phone2.png` |
| An email field and a URL field | "@" in place of the comma for email, "/" for a URL, each with its hint | `05a-email.png`, `05b-url.png` |
| The IME action key | A search field's key drew the search icon, a send field's the send icon (`ProbeActivity` now sets the action before the input type) | `06a-action-search.png`, `06b-action-send.png` |
| A long-press popup and hints | "e"'s popup opened with its number hint "3" chosen and ē ê é è ë around it; the letters showed their number and symbol hints | `07-popup-e.png` |
| A subtype switch | With English (US) and German both enabled, the language key switched to German's QWERTZ, its space bar named the language, and the smartbar's actions drew | `08-subtype-en-language-key.png`, `09-subtype-de-qwertz-smartbar-actions.png` |
| German's popups | "a"'s popup chose German's "ä" first, with its "@" hint and the other accents; "d" hinted "€", German's currency, and the language key showed in place of the emoji key | `10-subtype-de-popup-a.png` |
| The smartbar's overflow | Each action drawn with its icon and name; *Copy* (nothing selected) and *Clear primary clipboard clip* (nothing to paste) dimmed | `11-smartbar-overflow-enabled-state.png` |
| One-handed mode | The keyboard narrowed to one side with its panel, in German | `12-one-handed-de.png` |
| After recreation | The keyboard's process stopped and the field focused again: the keys came back | `13a-recreated-first-shot-branch.png`, `13b-recreated-branch.png` |
| The overflow, after the panels moved to the graph | Each action drawn with its icon and name from the key graph's answers; *Copy* and *Clear primary clipboard clip* dimmed with nothing selected and nothing copied; after one synthetic letter was typed and selected, *Copy* lit and *Clear* stayed dimmed | `14a-overflow-from-graph.png`, `14b-overflow-copy-lit.png` |

### The gate

`build/key-graph/gate.sh` on the rebased branch (fb1a9f000): `cargo fmt`, then
`./gradlew --stacktrace test :app:lintDebug :lib:rust:check :app:checkNativeDeclarations
:app:checkJvmLedger assembleDebug :app:compileDebugAndroidTestKotlin`. It passed.

* **JVM:** 1,006 tests in 246 files, with 0 failures, 0 errors and 0 skipped. Main has 1,005 in
  245; the new file is `KeyGraphMatchTest`, whose one test runs the three traces.
* **Cargo:** 1,402 tests, 0 failed and 0 ignored (main: 1,380).
* **Natives:** 396 generated, 0 hand-written, 0 missing and 0 mismatched (main: 391).
* **Ledger:** 424 rows for 424 production files (main: 421).
* **Lint:** only the allowed IDs: AndroidGradlePluginVersion ×4, NewerVersionAvailable ×4,
  ChromeOsAbiSupport ×1 and GradleDependency ×1.

The first gate on the rebased branch failed at `:lib:rust:cargoAndroidClippy`. Clippy reads the
actions op's `fields.len() % action::SIZE != 0` as a manual `is_multiple_of`. The op (c4847d79e)
came after the last gate before the rebase, so this was clippy's first look at it. fb1a9f000
fixes it, and the mutation script still finds all 73 of its locations.

### CPU trials

Four trials of `cpu-trials.py --rust-only --trials 1`, main (b10bee9b1), the branch (0754dee9b),
main, the branch, after the device session was closed and opened again. The dated entry is
[the key and layout object graph](cpu-p95-results.md#the-key-and-layout-object-graph-2026-11-22).

| Trial | Frames | p50 | p95 | p99 | Over 2 ms | CPU time p95 |
|---|---:|---:|---:|---:|---:|---:|
| main 1 | 1,156 | 0.993 | 3.238 | 6.165 | 164 (14.2%) | 1.490 |
| branch 1 | 1,128 | 1.054 | 3.279 | 6.862 | 164 (14.5%) | 1.752 |
| main 2 | 1,172 | 1.024 | 3.433 | 5.604 | 166 (14.2%) | 1.659 |
| branch 2 | 1,161 | 0.991 | 3.322 | 6.954 | 151 (13.0%) | 1.595 |

All four typed both digests. The branch's p95 lies between main's, and its CPU time p95 is
within main's own spread between trials, so the branch is not slower beyond noise. The press
frame's median was higher on the branch in both trials (by 0.04 to 0.28 ms, on about 50 frames
each), which the phone's p95 (P4) should look at. **Neither build met the 2 ms gate** at a load
average of 2.9 to 6.2, so this is not a pass.

### Layout rebuild timings

`scripts/render-benchmark/rebuild-timing.py` switches the keyboard's mode (?123, then ABC) and
its subtype (the language key, with English (US) and German enabled) by tapping through
agent-device, and times each switch from the tap's last touch to the last frame the renderer
posted for it: the new keyboard's rebuild and its first draw. It saves only numbers. Each tap is
held for 100 ms, so its release is drawn in a frame of its own that carries its touch time; with
a quick tap, the rebuilt keyboard's frame replaced the release frame and carried no touch, so no
tap was found (26a47e3df).

Each trial made 20 switches of each kind, and every trial found all 20 taps. Trials alternated
builds, branch first. Each trial's `rebuild.json`, with the host's load before and after, is in
`build/cpu-p95/key-graph/rebuild-<build>-<n>/`.

| Trial | Mode p50 / p95 / max (ms) | Subtype p50 / p95 / max (ms) | Load averages, start → end |
|---|---|---|---|
| branch 1 | 284.8 / 403.7 / 436.1 | 298.0 / 395.4 / 496.2 | 8.22 → 8.84 |
| main 1 | 405.5 / 566.6 / 603.3 | 241.4 / 343.6 / 390.3 | 8.70 → 8.88 |
| branch 2 | 312.9 / 418.2 / 613.9 | 294.3 / 411.8 / 413.8 | 9.18 → 6.27 |
| main 2 | 380.6 / 545.8 / 593.5 | 276.6 / 414.7 / 461.3 | 6.33 → 3.52 |

* **Mode switches** are faster on the branch in both pairs: p50 285 and 313 ms against main's
  406 and 381, and p95 404 and 418 against 567 and 546.
* **Subtype switches** are slower on the branch at p50 in both pairs: 298 and 294 ms against
  main's 241 and 277. That is 57 and 17 ms, while main's own two trials differ by 35. At p95 the
  two builds are level (395 and 412 against 344 and 415). The branch is not shown slower beyond
  the noise, but it is slower in both pairs, so the switch is owed a profile (E41).

These are emulator timings on a host also running other jobs, so they compare builds, not
devices.

## What is owed

Device checks are numbered after those on main (P24 and E37):

* **P25.** The key graph on the phone, in real apps: the enter key in a search box, a
  messaging app's send field, a browser's address bar and a form's next and done; email, URL,
  number, phone and PIN fields; popups and hints on letters and symbols; shift and caps lock; a
  second subtype; one-handed mode; the overflow's actions lit and dimmed as the clipboard and
  selection change; and the keyboard after the app is swiped away and reopened.
* **E38.** The keyboard right after its process is stopped and a field is focused again: on the
  branch, the first screenshot, about a second in, showed the key area empty, and the next one
  showed the keys (`13a-recreated-first-shot-branch.png`, `13b-recreated-branch.png`). On main,
  the first screenshot showed no keyboard yet and the next showed the keys
  (`13c-recreated-first-shot-main.png`, `13d-recreated-main.png`), so main may pass through the
  same state between screenshots. Capture the frames on both builds (a screen recording, or the
  renderer's frame log) to find out whether a first frame is drawn before the keys are computed,
  and if only the branch draws it, why.
* **E39.** The "teh" check depends on the device's learned state: once "teh" has been kept as
  typed (a backspace after the correction keeps it), "teh" and a space type "teh ". The checks
  here moved `kept-as-typed.txt` and `learned-words.txt` aside first. Device checks should start
  from a known dictionary state, or say which one they ran with.
* **E40.** Under host load from other jobs, the device steps fail in ways that look like the
  keyboard's faults but are not:
  * at load averages of 30 to 78, `adb shell am force-stop` timed out at 30 s in a device-prep
    script and passed on a retry with 150 s; at 110 it timed out at 150 s too, and passed on a
    retry a minute later;
  * `device_screenshot` failed twice with "The device hub could not complete the request".
    Each time `device_list` showed the device booted and still open, and the next screenshot
    worked;
  * at about 80, a tap on the language key in the rebuild timings landed as a long press and
    opened Android's keyboard picker, which blocked agent-device's next tap.

  Check whether the scripts' timeouts should scale with load, why the hub failed, and whether
  the timing scripts should refuse to start above a load threshold.
* **E41.** The subtype switch's rebuild on the branch was 17 and 57 ms slower at p50 than
  main's in both pairs of trials (above). Both builds keep each mode and subtype's keyboard once
  built, so after the first switch each one computes a kept keyboard for the other subtype's
  state. On the branch that means gathering the facts, answering rounds of casing questions over
  JNI, and reading the snapshot back. Profile the switch on both builds, find which step costs
  the time, and check whether a phone shows the gap.

Not device checks, still owed:

* **The Kotlin graph, at step K.** `LayoutManager`, `TextKeyboardCache`, `TextKey` and
  `TextKeyboard`, and the Compose-only parts of `KeyData`, `TextKeyData`, `PopupSet` and
  `LayoutAssembly` (above), are deleted with the Compose keyboard, and the trace's frozen
  `ReferenceKeyGraph.kt` with them once it has served its purpose.
* From item 1: retiring the per-step NLP operations once the frozen reference no longer needs
  them.

## What remains

Of item 2, nothing on the Rust renderer's path: Rust builds, computes and lays out every key it
draws, and answers the smartbar's questions. Kotlin keeps what needs Android:

* reading the layout and popup-mapping files from the extension manager's archives;
* the facts only Android knows (`EditorInfo`, the keyguard, the clipboard, Jetpref settings);
* casing and marks from Android's ICU, answered when the graph asks;
* `Key` objects carrying `TextKeyData` for the input path, and a key's platform effects;
* label resources and icon vectors, looked up by the token Rust returns.

For items 3–4:

* **3. Theme, then subtype.** The subtype still comes from Kotlin's `SubtypeManager`: which
  layouts, popup mapping, currency set and language a subtype names, and the language key's
  order. The graph takes those as files and facts, so moving the subtype to Rust changes what
  `KeyGraphRows` sends, not the graph. The theme (Snygg) is Kotlin throughout.
* **4. A plain-View root.** Icon tokens still map to Compose `ImageVector`s (`keyIconVector`,
  `iconPath`), and the panels are read in a composable. With a View root, the tokens can map to
  drawables and `RendererPanels` can read its resources outside Compose.

Phase 3's exit also wants the phone p95 again at the end (P4).
