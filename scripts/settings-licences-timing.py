#!/usr/bin/env python3
"""Device timings for the third-party licences screen, with the follow-up 3 (A5) method.

Each trial opens About on one renderer and records three Perfetto traces, with Gradle idle:

* open:   a tap on "Third-party licenses";
* dialog: a tap on the first library row, which opens its licence dialog;
* scroll: three 1100 px pans down and up the list, as settings-performance.py pans.

A tap is timed from its ACTION_UP event time. "First" is the present of the app SurfaceFrames
carrying the vsync id of the Choreographer frame the UP was delivered in (or the next one), as
A5 joins events to frames. "Settled" is the last present of the burst that follows, where a gap
of 250 ms or more between presents ends the burst (the scrollbar's 1850 ms hold is one).
Scrolling reports input -> present per event (A5's app_frames), main-thread doFrame, and for
Rust the scene slice and the worker's own numeric frame log.

ANDROID_SERIAL and AGENT_DEVICE must identify the T3-opened aw_keyboard_ssd session.
python3 -I scripts/settings-licences-timing.py OUTPUT PRIVATE_BASELINE TRACE_PROCESSOR
Receipts hold timestamps and counts only: no labels beyond the screen's fixed titles.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import re
import shlex
import sys
import time
sys.dont_write_bytecode = True


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).parent / path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


DEVICE = load('settings_device', 'render-benchmark/settings_ui_device.py')
SUMMARY = load('settings_summary', 'settings-performance-summary.py')
APP = DEVICE.PKG
TITLE = 'Third-party licenses'
FIRST_ROW = 'Activity'
GAP_NS = 250_000_000
CONFIG = '''buffers {{ size_kb: 32768 fill_policy: RING_BUFFER }}
duration_ms: {duration}
data_sources {{ config {{ name: "linux.ftrace" ftrace_config {{
  ftrace_events: "sched/sched_switch" ftrace_events: "sched/sched_waking"
  atrace_categories: "gfx" atrace_categories: "view" atrace_categories: "input"
  atrace_categories: "wm" atrace_categories: "am" atrace_categories: "binder_driver"
  atrace_apps: "com.jtn0123.audiowhisper.keyboard"
}} }} }}
data_sources {{ config {{ name: "android.surfaceflinger.frametimeline" }} }}
data_sources {{ config {{ name: "linux.process_stats" process_stats_config {{ scan_all_processes_on_start: true }} }} }}
'''


def centre(node):
    box = node['rect']
    return box['x'] + box['width'] // 2, box['y'] + box['height'] // 2


def record(d, label, duration_ms, action):
    """One trace around `action`, with the renderer's numeric log for the same span."""
    remote = f'/data/misc/perfetto-traces/aw-licences-{label}.perfetto-trace'
    d.adb('shell', 'logcat', '-c')
    d.adb('shell', 'perfetto', '--txt', '-c', '-', '-o', remote, '--background-wait',
          input=CONFIG.format(duration=duration_ms).encode())
    started = time.monotonic()
    time.sleep(1.0)
    action()
    while time.monotonic() - started < duration_ms / 1000 + 0.5:
        time.sleep(.2)
    d.wait(lambda: not d.adb('shell', 'pidof perfetto || true').strip(), 'Perfetto did not finish', timeout=20)
    d.adb('pull', remote, d.output / f'{label}.perfetto-trace')
    d.adb('shell', 'rm', remote)
    (d.output / f'{label}-native.txt').write_bytes(d.adb('shell', 'logcat', '-d', '-v', 'monotonic', '-s',
                                                          'aw-render:D', 'aw-settings:D', 'aw-profile:D', '*:S'))


def trial(d, label, renderer):
    d.fixture()
    d.prototype(renderer)
    d.launch('settings/about')
    row = d.wait(lambda: d.find(TITLE), 'About absent')
    time.sleep(2.0)
    record(d, label + '-open', 7000, lambda: d.ui('press', *centre(row)))
    first = d.wait(lambda: d.find(FIRST_ROW), 'Licences absent')
    time.sleep(1.5)
    record(d, label + '-dialog', 6000, lambda: d.ui('press', *centre(first)))
    ok = d.wait(lambda: d.find('OK'), 'Licence dialog absent')
    d.ui('press', *centre(ok))
    d.wait(lambda: not d.find('OK'), 'Licence dialog did not close')
    time.sleep(3.5)

    def pans():
        for _ in range(3):
            d.ui('gesture', 'pan', 540, 1650, 0, -1100, 600)
            d.ui('gesture', 'pan', 540, 550, 0, 1100, 600)
    record(d, label + '-scroll', 16000, pans)
    (d.output / f'{label}-meminfo.txt').write_bytes(d.adb('shell', 'dumpsys', 'meminfo', APP))
    d.launch('settings/home')


def slices(processor, trace, where):
    return SUMMARY.query(processor, trace, f"""SELECT s.ts,s.dur,s.name,th.name thread FROM slice s
        JOIN thread_track t ON s.track_id=t.id JOIN thread th ON t.utid=th.utid JOIN process p ON th.upid=p.upid
        WHERE p.name='{APP}' AND {where} ORDER BY s.ts""")


def frames(processor, trace):
    return SUMMARY.query(processor, trace, f"""SELECT a.ts,a.dur,a.surface_frame_token token,a.jank_type,a.layer_name
        FROM actual_frame_timeline_slice a JOIN process p USING(upid)
        WHERE p.name='{APP}' AND a.surface_frame_token IS NOT NULL AND a.dur >= 0 ORDER BY a.ts""")


def ms(ns):
    return round(ns / 1e6, 3)


def tap(processor, trace):
    """UP -> first present, and UP -> the last present of the burst that follows."""
    delivered = slices(processor, trace, "s.name GLOB 'deliverInputEvent src=*'")
    if not delivered:
        return {'error': 'no delivered input'}
    up = max(delivered, key=lambda r: int(re.search(r'eventTimeNano=(\d+)', r['name'])[1]))
    event = int(re.search(r'eventTimeNano=(\d+)', up['name'])[1])
    callbacks = slices(processor, trace, "th.tid=p.pid AND s.name GLOB 'Choreographer#doFrame *'")
    shown = frames(processor, trace)
    by_token = {}
    for f in shown:
        by_token.setdefault(int(f['token']), []).append(int(f['ts']) + int(f['dur']))
    ts = int(up['ts'])
    first = None
    for cb in callbacks:
        if int(cb['ts']) + int(cb['dur']) < ts:
            continue
        presents = by_token.get(int(cb['name'].split()[1]))
        if presents:
            first = max(presents)
            break
    if first is None:
        return {'error': 'no frame after the tap'}
    later = sorted(p for f in shown if (p := int(f['ts']) + int(f['dur'])) >= first)
    settled, count = first, 1
    for p in later[1:]:
        if p - settled >= GAP_NS:
            break
        settled, count = p, count + 1
    missed = sum('App Deadline Missed' in f['jank_type'] for f in shown
                 if first <= int(f['ts']) + int(f['dur']) <= settled)
    return {'up_to_first_present_ms': ms(first - event), 'up_to_settled_ms': ms(settled - event),
            'burst_surface_frames': count, 'burst_app_deadline_missed': missed}


def dist(values):
    out = SUMMARY.distribution(values)
    return {k: (round(v, 3) if isinstance(v, float) else v) for k, v in out.items()}


def scroll(processor, trace, native):
    delivered = slices(processor, trace, "s.name GLOB 'deliverInputEvent src=*'")
    result = {'app_frames': SUMMARY.app_frames(processor, trace, delivered)}
    result['doframe'] = dist([int(r['dur']) for r in slices(processor, trace,
                              "th.tid=p.pid AND s.name GLOB 'Choreographer#doFrame *'")])
    result['render_thread_drawframe'] = dist([int(r['dur']) for r in slices(processor, trace,
                                              "th.name='RenderThread' AND s.name GLOB 'DrawFrame*'")])
    result['rust_scene'] = dist([int(r['dur']) for r in slices(processor, trace, "s.name='aw.settings.frame'")])
    logged = [v for v in SUMMARY.numeric(native) if 'post_ns' in v and 'worker_ns' in v]
    result['rust_raster'] = dist([v['draw_ns'] for v in logged])
    result['rust_worker_to_post'] = dist([v['post_ns'] - v['worker_ns'] for v in logged])
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('baseline', type=Path)
    parser.add_argument('trace_processor', type=Path)
    parser.add_argument('--trials', default='compose,rust,rust,compose')
    args = parser.parse_args()
    d = DEVICE.SettingsDevice(args.output, args.baseline)
    assert d.adb('emu', 'avd', 'name').decode().splitlines()[0] == 'aw_keyboard_ssd'
    for stream, volume in [(1, 0), (2, 0), (3, 0), (5, 0), (4, 1), (10, 1)]:
        d.adb('shell', 'cmd', 'audio', 'set-volume', str(stream), str(volume))
        assert d.adb('shell', 'cmd', 'audio', 'get-stream-volume', str(stream)).decode().strip().endswith(f'-> {volume}')
    labels = [f'{i}-{r}' for i, r in enumerate(args.trials.split(','))]
    try:
        d.adb('shell', 'setprop', 'log.tag.aw-profile', 'DEBUG')
        for label in labels:
            trial(d, label, label.split('-', 1)[1])
            print(label, flush=True)
    finally:
        try:
            for key, value in d.original['properties'].items():
                d.adb('shell', 'setprop ' + shlex.quote(key) + ' ' + shlex.quote(value))
                assert d.adb('shell', 'getprop', key).decode().strip() == value
        finally:
            d.cleanup()
            d.ui('close')
    results = []
    for label in labels:
        base = args.output / label
        results.append({'trial': label,
                        'open': tap(args.trace_processor, Path(f'{base}-open.perfetto-trace')),
                        'dialog': tap(args.trace_processor, Path(f'{base}-dialog.perfetto-trace')),
                        'scroll': scroll(args.trace_processor, Path(f'{base}-scroll.perfetto-trace'),
                                         Path(f'{base}-scroll-native.txt')),
                        'pss_kib': SUMMARY.memory(Path(f'{base}-meminfo.txt'))['pss_kib']})
    (args.output / 'summary.json').write_text(json.dumps({'method': __doc__.split('\n\n')[1], 'trials': results},
                                                         indent=2) + '\n')
    print(args.output / 'summary.json')


if __name__ == '__main__':
    main()
