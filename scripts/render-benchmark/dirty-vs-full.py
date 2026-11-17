"""On the device: the Rust keyboard after many narrowed (dirty) frames equals a fresh full redraw.

python3 -I scripts/render-benchmark/dirty-vs-full.py OUTPUT, from the repository, with
ANDROID_SERIAL (the dedicated emulator) and AGENT_DEVICE (the exact device_open command) set.
Types 36 keys at run.sh's positions (the 1080x2400 AVD), holds e for its popup and clears the
disposable editor, then screenshots. Hiding and reshowing the keyboard gives it a new surface,
drawn whole; a second screenshot follows, and the keyboard rows of the two are compared. Saves the
two PNGs and prints numbers only. OUTPUT must not exist yet.
"""
import os
import re
import runpy
import struct
import subprocess
import sys
import time
import zlib
from pathlib import Path

api = runpy.run_path(str(Path(__file__).with_name("a11y_device.py")))
POSITIONS = [(60, 1750), (165, 1750), (275, 1750), (380, 1750), (110, 1900), (215, 1900),
             (325, 1900), (435, 1900), (220, 2060), (325, 2060), (435, 2060), (540, 2060)]


def pixels(path):
    data = Path(path).read_bytes()
    assert data[:8] == b"\x89PNG\r\n\x1a\n"
    at, idat, header = 8, b"", None
    while at < len(data):
        length, kind = struct.unpack(">I4s", data[at:at + 8])
        body = data[at + 8:at + 8 + length]
        if kind == b"IHDR":
            header = struct.unpack(">IIBBBBB", body)
        elif kind == b"IDAT":
            idat += body
        at += 12 + length
    width, height, depth, colour, _, _, interlace = header
    assert depth == 8 and interlace == 0 and colour in (2, 6), header
    bpp = 3 if colour == 2 else 4
    raw = zlib.decompress(idat)
    stride = width * bpp
    rows, previous, at = [], bytearray(stride), 0
    for _ in range(height):
        kind, line = raw[at], bytearray(raw[at + 1:at + 1 + stride])
        at += 1 + stride
        for i in range(stride):
            a = line[i - bpp] if i >= bpp else 0
            b = previous[i]
            c = previous[i - bpp] if i >= bpp else 0
            if kind == 1:
                line[i] = (line[i] + a) & 255
            elif kind == 2:
                line[i] = (line[i] + b) & 255
            elif kind == 3:
                line[i] = (line[i] + (a + b) // 2) & 255
            elif kind == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                line[i] = (line[i] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        rows.append(bytes(line))
        previous = line
    return width, height, bpp, rows


def main():
    out = Path(sys.argv[1])
    if out.exists():
        raise SystemExit("refusing to overwrite " + str(out))
    agent = os.environ["AGENT_DEVICE"]
    device = api["Device"](out)
    original = device.original_accessibility()
    try:
        device.mute()
        for tag, level in (("aw-render", "DEBUG"), ("aw-profile", "DEBUG")):
            device.shell("setprop", "log.tag." + tag, level)
        device.renderer("rust")
        device.probe(no_suggestions=False)
        tree = device.capture("preflight", lambda ns: any(n.get("text") in ("e", "E") for n in ns))
        if any(n.get("text") == "E" for n in tree["nodes"]):
            device.tap(70, 2060)
        if device.prefs().get("smartbar__shared_actions_expanded") == "true":
            device.tap(1025, 1618)
        device.reset()
        time.sleep(1.5)
        device.adb("logcat", "-c")
        for _ in range(3):
            for x, y in POSITIONS:
                device.tap(x, y)
        subprocess.check_output([agent, "longpress", "275", "1750", "600"], text=True)
        time.sleep(1.0)
        device.reset()
        time.sleep(2.0)
        dirty = out.resolve() / "after-dirty-frames.png"
        subprocess.check_output([agent, "screenshot", str(dirty)], text=True)
        before = device.adb("logcat", "-d", "-s", "aw-render:D")
        subprocess.check_output([agent, "back"], text=True)
        device.wait(lambda: "mInputShown=false" in device.shell("dumpsys", "input_method"), "IME still shown")
        time.sleep(1.0)
        device.adb("logcat", "-c")
        device.tap(540, 480)
        device.wait(lambda: "mInputShown=true" in device.shell("dumpsys", "input_method"), "IME not shown")
        time.sleep(2.0)
        full = out.resolve() / "fresh-full-redraw.png"
        subprocess.check_output([agent, "screenshot", str(full)], text=True)
        after = device.adb("logcat", "-d", "-s", "aw-render:D")
        dirty_frames = sum(1 for line in before.splitlines() if " profile " in line and " full=0" in line)
        recreated = [line for line in after.splitlines() if " profile " in line]
        first = re.search(r"recreate=(\d) full=(\d)", recreated[0]) if recreated else None
        print("narrowed frames before the first capture:", dirty_frames)
        print("first frame after reshowing: recreate=%s full=%s" % (first.groups() if first else ("?", "?")))
        if "draw_error" in before + after or "post_error" in before + after:
            raise RuntimeError("native draw/post error")
        if not first or first.groups() != ("1", "1") or not dirty_frames:
            raise RuntimeError("expected narrowed frames, then a new surface drawn whole")
    finally:
        device.restore_accessibility(original)
        for tag in ("aw-render", "aw-profile"):
            device.shell("setprop", "log.tag." + tag, "INFO")
    wa, ha, bpp_a, a = pixels(dirty)
    wb, hb, bpp_b, b = pixels(full)
    assert (wa, ha, bpp_a) == (wb, hb, bpp_b)
    # On the 1080x2400 AVD the keyboard fills the bottom rows; the status bar and the editor, with
    # its blinking cursor, sit above row 1400.
    differing = [y for y in range(1400, ha) if a[y] != b[y]]
    print("size", wa, ha, "rows compared", ha - 1400, "rows differing", len(differing),
          "range", (differing[0], differing[-1]) if differing else None)
    if differing:
        worst = max(max(abs(p - q) for p, q in zip(a[y], b[y])) for y in differing)
        count = sum(sum(1 for i in range(0, len(a[y]), bpp_a) if a[y][i:i + 3] != b[y][i:i + 3])
                    for y in differing)
        print("pixels differing", count, "largest channel difference", worst)
        raise SystemExit(1)


if __name__ == "__main__":
    main()
