"""Input-view creation, first frame and memory with the Rust renderer on (phase 3 item 4), on the
build installed: main's Compose root or the branch's plain-View root.

Takes stage 3b package 8's samples (`host.py`): 10 cold, each a new keyboard process, and 10
warm, each a recreation by rotation in the same process. For each: the factory's return and the
first non-empty layout, from `aw-host`'s timestamps, and the first frame the renderer posts, from
logcat's monotonic time of the first `aw-render` frame line after the factory returned. For each
cold sample, the process's total PSS with the keyboard shown, 3 seconds after it shows.
python3 -I; needs ANDROID_SERIAL and AGENT_DEVICE. Records numbers only.

    python3 -I scripts/render-benchmark/view-root-timing.py build/view-root/timing/main-1
"""
import argparse
import json
import re
import runpy
import statistics
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
diff = runpy.run_path(str(HERE / 'view-root-diff.py'))
Device, PKG = diff['Device'], diff['PKG']
TAGS = ('aw-host', 'aw-render', 'aw-profile')
LINE = re.compile(r'^\s*(\d+\.\d+)\s+\d+\s+\d+\s+\w\s+(aw-host|aw-render)\s*:\s(.*)$')


def sample(device):
    """The latest creation's factory, layout and first-frame times in ms, or None until all three."""
    log = device.adb('logcat', '-d', '-v', 'monotonic', '-v', 'usec', '-s', 'aw-host:D', 'aw-render:D')
    created, layout, frame = None, None, None
    for at, tag, text in (m.groups() for m in map(LINE.match, log.splitlines()) if m):
        if tag == 'aw-host' and (m := re.match(r'renderer=1 start_ns=(\d+) created_ns=(\d+)', text)):
            start, made = int(m.group(1)), int(m.group(2))
            created, layout, frame = (float(at), (made - start) / 1e6, start), None, None
        elif tag == 'aw-host' and created and (m := re.match(rf'renderer=1 start_ns={created[2]} layout_ns=(\d+)', text)):
            layout = (int(m.group(1)) - created[2]) / 1e6
        elif tag == 'aw-render' and created and frame is None and text.startswith('frame '):
            frame = (float(at) - created[0]) * 1000 + created[1]
    if created and layout is not None and frame is not None:
        return {'created_ms': created[1], 'layout_ms': layout, 'frame_ms': frame}
    return None


def pss(device):
    """The keyboard process's total PSS in KiB."""
    found = re.search(r'TOTAL PSS:\s+(\d+)', device.adb('shell', 'dumpsys', 'meminfo', PKG))
    return int(found.group(1)) if found else None


def cold(device):
    device.adb('shell', 'am', 'force-stop', PKG)
    device.adb('logcat', '-c')
    device.probe(True)
    item = device.wait(lambda: sample(device), 'Cold timestamps absent', timeout=20)
    time.sleep(3)
    return dict(item, pss_kib=pss(device), pid=device.adb('shell', 'pidof', PKG).strip())


def warm(device):
    device.adb('shell', 'wm', 'user-rotation', 'lock', '1')
    time.sleep(1.5)
    device.adb('logcat', '-c')
    device.adb('shell', 'wm', 'user-rotation', 'lock', '0')
    item = device.wait(lambda: sample(device), 'Warm timestamps absent', timeout=20)
    time.sleep(1)
    return dict(item, pid=device.adb('shell', 'pidof', PKG).strip())


def summary(runs):
    medians = {kind: {key: statistics.median(r[key] for r in rows) for key in rows[0] if key != 'pid'}
               for kind, rows in runs.items()}
    medians['cold']['pids'] = len({r['pid'] for r in runs['cold']})
    medians['warm']['pids'] = len({r['pid'] for r in runs['warm']})
    return medians


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('out')
    parser.add_argument('--count', type=int, default=10)
    args = parser.parse_args()
    device = Device(args.out)
    out = Path(args.out)
    raw, state = diff['saved'](out, device)
    props = {tag: device.adb('shell', 'getprop', 'log.tag.' + tag).strip() for tag in TAGS}
    runs = {'cold': [], 'warm': []}
    try:
        device.mute()
        diff['pose'](device, {k: v[0] for k, v in diff['DIMENSIONS'].items()}, raw)
        for tag, level in zip(TAGS, ('DEBUG', 'DEBUG', 'INFO')):
            device.adb('shell', 'setprop', 'log.tag.' + tag, level)
        runs['cold'] = [cold(device) for _ in range(args.count)]
        runs['warm'] = [warm(device) for _ in range(args.count)]
        device.save('runs.json', runs)
        device.save('summary.json', summary(runs))
        print(json.dumps(summary(runs), sort_keys=True), flush=True)
    finally:
        for tag, value in props.items():
            device.adb('shell', 'setprop', 'log.tag.' + tag, value or '""')
        diff['restore'](device, raw, state['system'], state['access'])
        for private in ('private-state.json', 'private-prefs.bin'):
            (out / private).unlink()


if __name__ == '__main__':
    main()
