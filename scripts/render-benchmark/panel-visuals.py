"""Empty-editor panel/theme screenshots and accessibility trees on both renderers."""
import argparse
from pathlib import Path
import runpy
import subprocess
import time

panels = runpy.run_path(str(Path(__file__).with_name('panels.py')))
visuals = runpy.run_path(str(Path(__file__).with_name('visuals.py')))
Device, SERVICE = panels['Device'], panels['SERVICE']


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('agent')
    parser.add_argument('output')
    args = parser.parse_args()
    root = Path(args.output)
    device = Device(root)
    original = device.original_accessibility()
    capture_preferences=device.capture_preferences()
    try:
        for theme in ('day', 'night'):
            for renderer in ('compose', 'rust'):
                device.output = root / (theme + '-' + renderer)
                device.output.mkdir(parents=True, exist_ok=True)
                device.renderer(renderer)
                visuals['theme'](device, theme)
                device.probe(True)
                device.services(SERVICE)
                device.capture('preflight')
                panels['emoji'](device, 1)
                panels['screenshot'](device, args.agent, 'smileys')
                device.tap(270, 1610)
                device.capture('people')
                panels['screenshot'](device, args.agent, 'people')
                device.hold(60, 1735, 700)
                device.capture('tone-popup', lambda ns: any((n.get('text') or n.get('description')) == '👋🏻' for n in ns))
                panels['screenshot'](device, args.agent, 'tone-popup')
                device.tap(700, 1610)
                panels['tap_name'](device, 'ABC')
                device.capture('text-after-emoji')
                panels['overflow'](device)
                panels['screenshot'](device, args.agent, 'overflow')
                for _ in range(2):
                    panels['scroll'](device)
                device.capture('editing')
                panels['screenshot'](device, args.agent, 'editing')
                data = panels['edit'](device)
                panels['screenshot'](device, args.agent, 'edit')
                header = panels['named'](data['nodes'], 'Customize action order')['bounds']
                device.tap(63, (header[1] + header[3]) // 2)
                device.capture('after-editor')
        for theme in ('day', 'night'):
            for state in ('smileys', 'people', 'tone-popup', 'overflow', 'editing', 'edit'):
                paths = [str(root / (theme + '-' + r) / (state + '.png')) for r in ('compose', 'rust')]
                output = root / (theme + '-' + state + '-pair.png')
                subprocess.check_call(['swift', 'scripts/render-benchmark/panel-pair.swift', *paths, str(output)])
    finally:
        visuals['theme'](device, 'day')
        device.restore_capture_preferences(capture_preferences)
        device.restore_accessibility(original)


if __name__ == '__main__':
    main()
