package dev.patrickgold.florisboard.app

/**
 * The settings link an intent carries. Rust draws settings and setup and picks the first screen
 * from this (`SettingsUiOp.START`); Compose draws only the screens Rust hands to
 * [ComposeSettingsFallbackActivity].
 */
internal object SettingsEntry {
    private const val LINK = "ui://florisboard/"
    private const val VIEW = "android.intent.action.VIEW"

    /** The route a `ui://florisboard/…` link names, or null for any other intent. */
    fun linkRoute(action: String?, data: String?): String? =
        data?.takeIf { action == VIEW && it.startsWith(LINK) }?.removePrefix(LINK)
}
