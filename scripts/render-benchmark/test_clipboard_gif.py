"""Adversarial checks for the panel semantic comparator."""
from pathlib import Path
import runpy
Tests=runpy.run_path(str(Path(__file__).with_name('compare-clipboard-gif.py')))['Tests']
