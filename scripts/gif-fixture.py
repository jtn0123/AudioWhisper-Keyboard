"""Synthetic GIPHY answers and GIFs for a fixture build (-PgifFixture=true), with no key and no network.

Writes trending.json, search.json, a search-<hash>.json for two category chips (so a chip's own
results can be told from the fallback's by colour), and media/*.gif: small two-frame GIFs drawn
here, each a flat colour with a moving stripe. `push` copies the set into the keyboard's
files/gif-fixture with run-as and switches it in (GifFixtureReceiver); `off` switches it out and
deletes it. Prints counts only. Run with python3 -I.

    python3 -I scripts/gif-fixture.py push emulator-5556 [--delay-ms N]
    python3 -I scripts/gif-fixture.py off emulator-5556
"""
import argparse
import colorsys
import hashlib
import io
import json
import os
import struct
import subprocess
import tarfile
import time
from pathlib import Path

PKG = 'com.jtn0123.audiowhisper.keyboard'
ACTION = 'com.jtn0123.audiowhisper.GIF_FIXTURE'
MEDIA = 'https://media.fixture.test/'
# Chips whose searches get their own answer; the rest fall back to search.json.
CHIP_SETS = {'Haha': 0.78, 'Yes': 0.0, 'No': 0.14}
SETS = {'trending.json': (0.55, 12), 'search.json': (0.3, 30)}


def lzw(pixels, min_size=2):
    """GIF LZW, a clear code every two literals so the code size never grows."""
    clear, end = 1 << min_size, (1 << min_size) + 1
    size = min_size + 1
    codes = [clear]
    for n, pixel in enumerate(pixels):
        if n and n % 2 == 0:
            codes.append(clear)
        codes.append(pixel)
    codes.append(end)
    out, buffer, bits = bytearray([min_size]), 0, 0
    data = bytearray()
    for code in codes:
        buffer |= code << bits
        bits += size
        while bits >= 8:
            data.append(buffer & 255)
            buffer >>= 8
            bits -= 8
    if bits:
        data.append(buffer & 255)
    for start in range(0, len(data), 255):
        chunk = data[start:start + 255]
        out += bytes([len(chunk)]) + chunk
    return bytes(out + b'\x00')


def gif(width, height, hue):
    """Two frames: a flat colour with a lighter band that moves down, looping."""
    base = [int(255 * c) for c in colorsys.hsv_to_rgb(hue, 0.65, 0.85)]
    band = [int(255 * c) for c in colorsys.hsv_to_rgb(hue, 0.25, 1.0)]
    palette = bytes(base + band + [24, 24, 24, 255, 255, 255])
    out = bytearray(b'GIF89a' + struct.pack('<HHBBB', width, height, 0xF1, 0, 0) + palette)
    out += b'\x21\xFF\x0BNETSCAPE2.0\x03\x01\x00\x00\x00'
    for frame in range(2):
        out += b'\x21\xF9\x04\x04' + struct.pack('<H', 40) + b'\x00\x00'
        out += b'\x2C' + struct.pack('<HHHHB', 0, 0, width, height, 0)
        top = height // 4 + frame * height // 3
        pixels = [1 if top <= y < top + height // 5 else (2 if x < 3 else 0)
                  for y in range(height) for x in range(width)]
        out += lzw(pixels)
    return bytes(out + b'\x3B')


def answer(name, hue, count, media):
    data = []
    for n in range(count):
        gif_id = f'{name}{n:02d}'
        tint = (hue + n * 0.012) % 1.0
        height = 60 + (n * 17) % 90
        media[f'{gif_id}-p.gif'] = gif(100, height, tint)
        media[f'{gif_id}-s.gif'] = gif(160, height * 160 // 100, tint)
        data.append({
            'id': gif_id, 'title': f'Fixture {n}', 'url': f'https://fixture.test/gifs/{gif_id}',
            'images': {
                'fixed_width': {'url': MEDIA + f'{gif_id}-p.gif', 'width': '100', 'height': str(height)},
                'downsized': {'url': MEDIA + f'{gif_id}-s.gif', 'width': '160', 'height': str(height * 160 // 100)},
            },
        })
    return json.dumps({'data': data, 'pagination': {'count': count}}).encode()


def fixture(delay_ms):
    files, media = {}, {}
    for file, (hue, count) in SETS.items():
        files[file] = answer(file.split('.')[0][:2], hue, count, media)
    for chip, hue in CHIP_SETS.items():
        digest = hashlib.sha256(chip.encode()).hexdigest()[:12]
        files[f'search-{digest}.json'] = answer('c' + str(len(files)), hue, 30, media)
    for name, body in media.items():
        files['media/' + name] = body
    if delay_ms:
        files['delay-ms'] = str(delay_ms).encode()
    return files


def adb(serial, *args, stdin=None):
    sdk = os.environ.get('ANDROID_HOME', os.path.expanduser('~/Library/Android/sdk'))
    return subprocess.run([sdk + '/platform-tools/adb', '-s', serial, *args], input=stdin,
                          capture_output=True, check=False)


def broadcast(serial, op):
    result = adb(serial, 'shell', 'am', 'broadcast', '-a', ACTION, '-p', PKG, '--es', 'op', op)
    return b'result=1' in result.stdout


def push(serial, delay_ms):
    files = fixture(delay_ms)
    archive = io.BytesIO()
    with tarfile.open(fileobj=archive, mode='w') as tar:
        for name, body in sorted(files.items()):
            info = tarfile.TarInfo('gif-fixture/' + name)
            info.size = len(body)
            tar.addfile(info, io.BytesIO(body))
    adb(serial, 'shell', 'run-as', PKG, 'rm', '-rf', 'files/gif-fixture')
    adb(serial, 'exec-in', 'run-as', PKG, 'sh', '-c', 'cat > files/gif-fixture.tar', stdin=archive.getvalue())
    # exec-in can return before the device has written the last of it.
    for _ in range(40):
        size = adb(serial, 'shell', 'run-as', PKG, 'stat', '-c', '%s', 'files/gif-fixture.tar').stdout.strip()
        if size == str(len(archive.getvalue())).encode():
            break
        time.sleep(0.25)
    adb(serial, 'shell', 'run-as', PKG, 'sh', '-c', "'cd files && tar xf gif-fixture.tar && rm gif-fixture.tar'")
    listed = adb(serial, 'shell', 'run-as', PKG, 'find', 'files/gif-fixture', '-type', 'f').stdout.split()
    on = broadcast(serial, 'on')
    print(f'gif fixture: {len(files)} files written, {len(listed)} on the device, '
          f'{sum(1 for n in files if n.startswith("media/"))} GIFs, delay {delay_ms} ms, switched in: {on}')
    return 0 if on and len(listed) == len(files) else 1


def off(serial):
    switched = broadcast(serial, 'off')
    adb(serial, 'shell', 'run-as', PKG, 'rm', '-rf', 'files/gif-fixture')
    left = adb(serial, 'shell', 'run-as', PKG, 'ls', 'files/gif-fixture').returncode == 0
    print(f'gif fixture: switched out: {switched}, files left: {left}')
    return 0 if switched and not left else 1


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('op', choices=['push', 'off', 'write'])
    parser.add_argument('serial_or_dir')
    parser.add_argument('--delay-ms', type=int, default=0)
    args = parser.parse_args()
    if args.op == 'write':
        out = Path(args.serial_or_dir)
        for name, body in fixture(args.delay_ms).items():
            (out / name).parent.mkdir(parents=True, exist_ok=True)
            (out / name).write_bytes(body)
        print(f'gif fixture: written to {out}')
        return 0
    return push(args.serial_or_dir, args.delay_ms) if args.op == 'push' else off(args.serial_or_dir)


if __name__ == '__main__':
    raise SystemExit(main())
