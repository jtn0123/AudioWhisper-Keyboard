#!/usr/bin/env python3
"""Strict paired PNG comparison: RGB threshold 3, one-pixel raster edge band, system bars.

Requires Pillow and numpy, installed outside source (e.g. build/settings-image-venv).
No alignment, blur, similarity percentage, or broad text/row masks can turn a mismatch into
acceptance. Deliberate font-2 wrapping is reviewed separately; its changed geometry remains
visible in these conservative whole-picture counts.
"""
import argparse
import json
from pathlib import Path
import numpy as np
from PIL import Image


def edges(a):
    result = np.zeros(a.shape[:2], dtype=bool)
    # Match settings-color-probe.rs exactly: exclude a pixel when ANY RGB neighbour
    # within one physical pixel differs by >3. No further dilation or alignment.
    for dy in (-1, 0, 1):
        for dx in (-1, 0, 1):
            neighbour = np.roll(a, (dy, dx), axis=(0, 1))
            result |= np.max(np.abs(a-neighbour), axis=2) > 3
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    args = parser.parse_args()
    rows = []
    for left in sorted(args.directory.glob('*/compose-*.png')):
        right = left.with_name(left.name.replace('compose-', 'rust-', 1))
        if not right.exists():
            continue
        a = np.asarray(Image.open(left).convert('RGB'), dtype=np.int16)
        b = np.asarray(Image.open(right).convert('RGB'), dtype=np.int16)
        assert a.shape == b.shape
        mask = edges(a) | edges(b)
        mask[:72] = True
        mask[-72:] = True
        difference = np.max(np.abs(a-b), axis=2)
        failed = (difference > 3) & ~mask
        rows.append({'theme': left.parent.name, 'picture': left.stem.removeprefix('compose-'),
                     'compared_pixels': int((~mask).sum()), 'different_pixels_above_3': int(failed.sum()),
                     'max_channel_error': int(difference[~mask].max()), 'strict_match': not bool(failed.any())})
        heat = np.zeros_like(a, dtype=np.uint8)
        heat[failed] = [255, 0, 0]
        Image.fromarray(heat).save(left.with_name(left.name.replace('compose-', 'diff-', 1)))
    output = args.directory / 'comparison.json'
    output.write_text(json.dumps({'threshold_per_channel': 3, 'edge_band_physical_px': 1,
        'system_bars_top_bottom_px': 72, 'alignment': 'none', 'pairs': rows}, indent=2)+'\n')
    print(output, len(rows), 'pairs;', sum(r['strict_match'] for r in rows), 'strict matches')


if __name__ == '__main__':
    main()
