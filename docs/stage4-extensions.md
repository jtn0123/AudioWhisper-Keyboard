# Stage 4: extensions

This page covers step 10 of [the Compose removal plan](compose-removal.md): the extension screens
move from Compose to the Rust settings renderer. The owner's rule is the one step 9 had: they are
"ported to Rust and improved, not cut, and the port names each improvement it makes"
([decision](decisions.md#everything-moves-to-rust-2026-11-21)).

## The mutation script split

Before step 10 added cases, `scripts/settings-ui-mutations.py` was 1,016 counted lines
(non-blank, non-comment), over the 500-line file limit. It is now a runner of 84 counted lines,
and the cases live in `scripts/settings_ui_mutation_cases/`, one module per screen:

| Module | Cases | Counted lines |
| --- | --- | --- |
| `general.py` (the renderer, groups 1 and 2, Jetpref writes, third-party licences) | 47 | 100 |
| `other_theme.py` | 22 | 45 |
| `languages.py` | 15 | 34 |
| `words.py` | 26 | 55 |
| `project_licence.py` | 36 | 88 |
| `themes.py` (the theme manager) | 21 | 45 |
| `setup.py` | 26 | 59 |
| `theme_editor.py` (part 2) | 38 | 88 |
| `property_editor.py` (part 3) | 46 | 101 |
| `property_values.py` (part 3: colours, swatches, icons, contrast) | 57 | 116 |
| `theme_codes.py` (part 3: the code editor, and the fallback crash fix) | 49 | 94 |

`paths.py` (17) holds the paths the cases name and `__init__.py` (7) joins the modules in their old
order. Each module keeps its own Rust targets (`RUST`) beside its cases. The runner's command line
and output are unchanged: `python3 -I scripts/settings-ui-mutations.py <output> [case …]`. `-I`
leaves the script's folder off the import path, so the runner puts it back itself, and it writes no
`__pycache__`.

**Nothing was lost.** A dump of every case's name, file, original text and mutated text in run
order, with the Rust target map and the suites, was taken from the old script and from the split
one. The two dumps are byte-identical (383 cases, 255 Rust targets; SHA-256
`71b977d8…3c55`). The dump script was a scratch file and is not committed. The split was made by
slicing the old file's source lines, so each case's text is the same characters, comments
included.
