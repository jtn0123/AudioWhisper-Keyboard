"""Replays a bench log through two playback models: a speaker buffer capped at CAP seconds that
blocks generation (the app today), and an unbounded queue (generation never waits)."""
import re, sys
rows = [tuple(map(float, m.groups())) for m in
        (re.search(r'audio\s+([\d.]+) gen\s+([\d.]+) first\s+([\d.]+)', l) for l in open(sys.argv[1])) if m]
def run(cap):
    t = 0.0; play_end = None; gaps = []; startup = None
    for audio, gen, first in rows:
        if cap is not None and play_end is not None:
            t = max(t, play_end - cap)        # writing the last sentence blocked until this
        ready = t + first                      # audio arrives once the sentence is generated
        if play_end is None: startup = ready; play_end = ready
        gaps.append(max(0.0, ready - play_end))
        play_end = max(play_end, ready) + audio
        t += gen
    return startup, max(gaps), sum(gaps)
for name, cap in (("4 s buffer", 4.0), ("unbounded", None)):
    s, w, total = run(cap)
    print(f"  {name:11s} startup {s:.2f}  worst gap {w:.2f}  total gaps {total:.2f}")
