#!/usr/bin/env python3
"""Settings-only, empty-field traces. Device must already be opened through T3.

ANDROID_SERIAL and AGENT_DEVICE must identify the owned aw_keyboard_ssd session.
python3 -I scripts/settings-performance.py OUTPUT PRIVATE_BASELINE
Build/install separately, with Gradle idle during measurement.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import subprocess
import shlex
import sys
sys.dont_write_bytecode = True
import time

SPEC = importlib.util.spec_from_file_location('settings_device', Path(__file__).parent / 'render-benchmark/settings_ui_device.py')
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('baseline', type=Path)
    args = parser.parse_args()
    d = MODULE.SettingsDevice(args.output, args.baseline)
    assert d.adb('emu', 'avd', 'name').decode().splitlines()[0] == 'aw_keyboard_ssd'
    for stream, volume in [(1, 0), (2, 0), (3, 0), (5, 0), (4, 1), (10, 1)]:
        d.adb('shell', 'cmd', 'audio', 'set-volume', str(stream), str(volume))
        assert d.adb('shell', 'cmd', 'audio', 'get-stream-volume', str(stream)).decode().strip().endswith(f'-> {volume}')
    config = '''buffers { size_kb: 32768 fill_policy: RING_BUFFER }
duration_ms: 16000
data_sources { config { name: "linux.ftrace" ftrace_config {
  ftrace_events: "sched/sched_switch" ftrace_events: "sched/sched_waking"
  atrace_categories: "gfx" atrace_categories: "view" atrace_categories: "input"
  atrace_categories: "wm" atrace_categories: "am" atrace_categories: "binder_driver"
  atrace_apps: "com.jtn0123.audiowhisper.keyboard"
} } }
data_sources { config { name: "android.surfaceflinger.frametimeline" } }
data_sources { config { name: "linux.process_stats" process_stats_config { scan_all_processes_on_start: true } } }
'''
    try:
        d.adb('shell', 'setprop', 'log.tag.aw-profile', 'DEBUG')
        for trial, renderer in enumerate(('compose', 'rust', 'rust', 'compose')):
            d.fixture()
            d.prototype(renderer)
            d.launch('settings/keyboard')
            time.sleep(1)
            label = f'{trial}-{renderer}'
            remote = f'/data/misc/perfetto-traces/aw-settings-{label}.perfetto-trace'
            (d.output / f'{label}-init.txt').write_bytes(d.adb('shell', 'logcat', '-d', '-v', 'monotonic', '-s', 'aw-render:D', 'aw-settings:D', 'aw-profile:D', '*:S'))
            d.adb('shell', 'logcat', '-c')
            d.adb('shell', 'perfetto', '--txt', '-c', '-', '-o', remote, '--background-wait', input=config.encode())
            started = time.monotonic()
            for _ in range(3):
                d.ui('gesture', 'pan', 540, 1650, 0, -1100, 600)
                d.ui('gesture', 'pan', 540, 550, 0, 1100, 600)
            (d.output / f'{label}-meminfo.txt').write_bytes(d.adb('shell', 'dumpsys', 'meminfo', MODULE.PKG))
            (d.output / f'{label}-native.txt').write_bytes(d.adb('shell', 'logcat', '-d', '-v', 'monotonic', '-s', 'aw-render:D', 'aw-settings:D', 'aw-profile:D', '*:S'))
            # Each wait is short; recording is bounded by Perfetto's own duration.
            while time.monotonic()-started < 16.5:
                time.sleep(.2)
            d.adb('pull', remote, d.output / f'{label}.perfetto-trace')
            d.adb('shell', 'rm', remote)
            print(label, flush=True)
    finally:
        try:
            for key, value in d.original['properties'].items():
                d.adb('shell', 'setprop ' + shlex.quote(key) + ' ' + shlex.quote(value))
        finally:
            d.cleanup()


if __name__ == '__main__':
    main()
