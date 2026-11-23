# Theme font subsets

Two small subsets of Dancing Script 1.002, a script face as unlike Roboto as a Latin font gets, so
the theme-font tests (`tests/theme_fonts.rs`) can tell from pixels and from the shaped runs that a
theme's `@font` drew a label, and which glyphs fell through to the default chain.

| File | Source font (SHA-256) | Code points |
|---|---|---|
| `fixture-script-regular-subset.ttf` | Android 9 SDK `data/fonts/DancingScript-Regular.ttf` (`e2aeb12315c3ef6eb16ba6e890f26483d8f0384afcc4f7b3f097b398c61f3462`), weight 400 | U+0020, U+0041–U+005A, U+0061–U+007A |
| `fixture-script-bold-subset.ttf` | Android 9 SDK `data/fonts/DancingScript-Bold.ttf` (`3c6f4588726bea7824e05c4ec4cb673ea04ad01bd2449dc826db54b6b4005ece`), weight 700 | U+0020, U+0041–U+005A, U+0061–U+007A |

No digits and no punctuation, deliberately: a `7` or a `?` in a label has to come from the default
chain, which is what the fallback test checks.

Made with fontTools 4.55.3 `subset.Subsetter` (layout features `*`, all name records, no hinting,
`VDMX`, `hdmx` and `LTSH` dropped). Dancing Script is under the SIL Open Font License 1.1 with the
Reserved Font Name "Dancing Script", and a subset is a Modified Version, so name records 1, 3, 4 and
6 were rewritten to "Fixture Script" / `FixtureScript-Regular` / `FixtureScript-Bold` and the
trademark record (7) removed; the copyright and licence records are kept. The licence and the
copyright notices are in `THEME-FONT-OFL.txt`.
