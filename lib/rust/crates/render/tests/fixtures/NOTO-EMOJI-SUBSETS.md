# Noto Color Emoji subsets

Two small subsets of the emulator's Noto Color Emoji 2.047 (Android 15 system fonts). The
colour-glyph cache tests use them (`src/color_glyphs_tests.rs`, `tests/color_glyph_memory.rs`).

| File | Source font | Format | Code points |
|---|---|---|---|
| `noto-color-emoji-subset.ttf` | `NotoColorEmoji.ttf` | COLRv1 | U+1F600, U+1F680, U+1F44B, U+1F44D, U+1F3FB–U+1F3FF, U+200D (with GSUB, so 👍🏽 forms) |
| `noto-color-emoji-flags-subset.ttf` | `NotoColorEmojiFlags.ttf` | CBDT | U+1F1FA, U+1F1F8 (with GSUB, so 🇺🇸 forms) |

Made with fontTools `pyftsubset`, keeping the layout tables. Both fonts are under the SIL Open
Font License 1.1 with no Reserved Font Name; the licence is in `NOTO-EMOJI-OFL.txt`.
