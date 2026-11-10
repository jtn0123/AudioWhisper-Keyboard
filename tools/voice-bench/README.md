# Voice benchmark

Times read-aloud's neural voice (Pocket TTS through sherpa-onnx) on a device, without the app:
`scripts/bench-voice.sh <unpacked-model-folder> [adb-serial] [threads] [flow-steps]`.

- `Bench.java` loads the model like `VoiceEngine` does and generates `text.txt` one sentence at a
  time, as read-aloud does, timing each sentence: its length in speech, how long it took, and
  when its first audio arrived. (sherpa-onnx hands over a sentence's audio only once the whole
  sentence is generated, so that is when a listener would hear it.)
- `gaps.py` replays those timings through the app's playback (a speaker buffer of 4 s that
  generation waits on) and through an unbounded queue, and reports the time to the first word and
  the silences a listener would hear between sentences.
- `throttle.py` slows an emulator to a share of its CPU time (`SLOW=0.42`), to stand in for a phone.

## Standing in for the S24 Ultra

The emulator runs the phone's arm64 code natively, on the Mac's cores, which are far faster:
Geekbench 6 gives about 4,300 single-core for the M5 Pro and about 2,300 for the Snapdragon 8
Gen 3 for Galaxy. With 4 threads the phone runs one Cortex-X4 and three slower A720s, so an
emulator vCPU is about 2.4 times a phone thread: `SLOW=0.42` (which measured 2.25 times slower).
That is an estimate, not a measurement; the phone itself is the check.

The S24-like emulator (`s24_like`, a copy of `aw_keyboard`'s AVD config with `hw.cpu.ncore=8`,
`hw.ramSize=12288`, `hw.lcd.width=1080`, `hw.lcd.height=2340`, `hw.lcd.density=450`) has the
phone's 8 cores, 12 GB and default screen.

## Results, 2026-11-10 (M5 Pro)

Speed is seconds of speech per second of generation; read-aloud needs more than ×1.

| Setup | Threads | Speed | First word | Longest silence |
|---|---|---|---|---|
| Emulator, full speed (4 cores) | 4 | ×6.4 | 1.0 s | none |
| S24-like, `SLOW=0.42` | 1 | ×1.5–2.1 | 1.2–1.9 s | up to 0.6 s |
| S24-like, `SLOW=0.42` | 2 | ×2.8 | 0.9 s | none |
| S24-like, `SLOW=0.42` | 4 | ×3.0 | 0.9 s | none |
| S24-like, `SLOW=0.42` | 6 | ×1.3 | 4.3 s | 5.3 s |
| S24-like, `SLOW=0.25` (a phone 1.7× slower still) | 4 | ×1.5 | 1.6 s | 0.3 s |
| S24-like, `SLOW=0.15` | 4 | ×0.6 | 4.0 s | 5.8 s |

- In the app at `SLOW=0.42` ("Read from here" in Chrome): loaded in 1.3 s, then each paragraph
  generated at ×2.3–2.7 ("Generated 8.7 s of speech in 3.7 s").
- 4 threads stays (as for dictation); 6 spills onto the phone's slow cores and is much worse.
- Flow steps (`numSteps`, default 5) cost almost nothing: 1 step was no faster.
- The 4 s speaker buffer never caused a silence: they come from a long sentence after a short
  one, which a bigger buffer can't fix.
