# Noto text subsets

Two small subsets of the emulator's Android 15 system fonts, so the key-label fit tests
(`tests/key_label_fit_scripts.rs`) draw Hebrew, which runs right to left, and CJK, which breaks
between its characters, with real glyphs rather than Roboto's missing-glyph boxes.

| File | Source font | Code points |
|---|---|---|
| `noto-sans-hebrew-subset.ttf` | `NotoSansHebrew-Regular.ttf` 1.04 | U+0020, U+0028, U+0029, U+05D0, U+05D1, U+05D9, U+05DC, U+05E2, U+05E8, U+05E9, U+05EA (עברית (ישראל)) |
| `noto-sans-cjk-subset.otf` | `NotoSansCJK-Regular.ttc` 2.004, font 0 (Noto Sans CJK JP) | U+2026, U+4E2D, U+4F53, U+56FD, U+6587, U+65E5, U+672C, U+7B80, U+8A9E, U+FF08, U+FF09, U+FF0C (中文（简体，中国）日本語…) |

Made with fontTools `pyftsubset --layout-features='*'`. Both fonts are under the SIL Open Font
License 1.1; the licence and their copyright notices are in `NOTO-TEXT-OFL.txt`. The subsets keep
the fonts' own names, neither of which is the CJK font's Reserved Font Name.
