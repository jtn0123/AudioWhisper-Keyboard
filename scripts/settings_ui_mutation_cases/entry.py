"""The ways into settings (stage4-compose-host.md): each case must fail the test its comment names."""
from .paths import ROOT, UI, CAT, APP

ENTRY = APP / 'SettingsEntry.kt'
RES = ROOT / 'app/src/main'

CASES = [
    # settings_ui_not_found: an unknown link opens home, or nothing, instead of the not-found page.
    ('entry-unknown-opens-home', UI / 'mod.rs',
     '            self.navigate(catalogue::NOT_FOUND_ROUTE);\n', '            self.navigate(catalogue::HOME_ROUTE);\n'),
    ('entry-unknown-ignored', UI / 'mod.rs', '            self.navigate(catalogue::NOT_FOUND_ROUTE);\n', ''),
    # settings_ui_not_found: "@enable" falls through to the page instead of asking to enable the keyboard.
    ('entry-enable-a-page', UI / 'mod.rs', '} else if route == "@enable" {', '} else if route == "@enabled" {'),
    # settings_ui_not_found: the page's way home leads elsewhere, or its message goes.
    ('entry-page-home-elsewhere', CAT / 'not_found.rs', 'Kind::Route(HOME_ROUTE),', 'Kind::Route("settings/about"),'),
    ('entry-page-message-dropped', CAT / 'not_found.rs', 'Kind::Note,', 'Kind::Route(HOME_ROUTE),'),
    # SettingsEntryTest.everyWayIntoSettingsLandsOnItsRustScreen: a fresh activity drops its link; a later link
    # is dropped.
    ('entry-fresh-link-dropped', ENTRY,
     'if (fresh) SettingsUiOp.START to listOfNotNull(link).toTypedArray()', 'if (fresh) SettingsUiOp.START to emptyArray<String>()'),
    ('entry-later-link-dropped', ENTRY, 'link?.let { SettingsUiOp.NAVIGATE to arrayOf(it) }', 'null'),
    # SettingsEntryTest.aSettingsLinkOpensItsScreenEvenDuringSetup: the link keeps its scheme and host.
    ('entry-link-prefix-kept', ENTRY, '?.removePrefix(LINK)', ''),
    # SettingsEntryTest.anExtensionFileIsNotASettingsLink: a shared ui:// text reads as a link.
    ('entry-link-any-action', ENTRY, 'action == VIEW && it.startsWith(LINK)', 'it.startsWith(LINK)'),
    # SettingsEntryTest.anOpenedOrSharedFileStartsTheImporterOnIt: SEND imports the intent's data, not its stream.
    ('entry-send-reads-view', ENTRY, '            SEND -> shared\n', '            SEND -> view\n'),
    # SettingsEntryTest.theManifestSendsEverySettingsIntentToTheRustHost: Android's keyboard settings name the deleted
    # fallback activity; the settings link filter takes another host.
    ('entry-method-settings-elsewhere', RES / 'res/xml/method.xml',
     'android:settingsActivity="dev.patrickgold.florisboard.SettingsLauncherAlias"',
     'android:settingsActivity="dev.patrickgold.florisboard.app.ComposeSettingsFallbackActivity"'),
    ('entry-link-filter-host', RES / 'AndroidManifest.xml',
     '<data android:scheme="ui" android:host="florisboard" android:pathPrefix="/" />',
     '<data android:scheme="ui" android:host="floris" android:pathPrefix="/" />'),
]
# The Rust test binary that must panic for each Rust-only case.
RUST = dict.fromkeys(['entry-unknown-opens-home', 'entry-unknown-ignored', 'entry-enable-a-page',
                      'entry-page-home-elsewhere', 'entry-page-message-dropped'], 'settings_ui_not_found')
