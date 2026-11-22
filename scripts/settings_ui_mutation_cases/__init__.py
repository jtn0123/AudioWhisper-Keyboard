"""The settings UI mutation cases, one module per screen, in the order scripts/settings-ui-mutations.py runs them."""
from . import general, other_theme, languages, words, project_licence, themes, setup, theme_editor, property_editor, property_values, theme_codes, extensions, ext_editor

MODULES = (general, other_theme, languages, words, project_licence, themes, setup, theme_editor, property_editor, property_values, theme_codes, extensions, ext_editor)
CASES = [case for module in MODULES for case in module.CASES]
RUST = {name: target for module in MODULES for name, target in module.RUST.items()}
assert len({case[0] for case in CASES}) == len(CASES), 'a case name is repeated'
assert set(RUST) <= {case[0] for case in CASES}, 'a Rust target names no case'
