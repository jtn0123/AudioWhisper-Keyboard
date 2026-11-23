"""The paths the settings UI mutation cases name."""
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
BASE = ROOT / 'lib/rust/crates'
CAT = BASE / 'render/src/settings_ui/catalogue'
LIC = BASE / 'render/src/settings_ui/licences'
SETTINGS = ROOT / 'app/src/main/kotlin/com/jtn0123/audiowhisper/settings'
UI = BASE / 'render/src/settings_ui'
LANG = UI / 'languages'
WORDS = UI / 'words'
PL = UI / 'project_licence'
THEMES = UI / 'themes'
SETUP = UI / 'setup'
TE = UI / 'theme_editor'
EXT = UI / 'extensions'
SNYGG = BASE / 'snygg/src'
ICONS = BASE / 'icons/src'
SETTINGS_UI = ROOT / 'app/src/main/kotlin/com/jtn0123/audiowhisper/settingsui'
APP = ROOT / 'app/src/main/kotlin/dev/patrickgold/florisboard/app'
