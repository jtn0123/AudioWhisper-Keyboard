package dev.patrickgold.florisboard.app

/**
 * Which host the settings Activity shows, from facts it already has. Rust draws settings; Compose
 * draws only Setup, and the screens Rust hands to [ComposeSettingsFallbackActivity].
 */
internal object SettingsEntry {
    private const val LINK = "ui://florisboard/"
    private const val VIEW = "android.intent.action.VIEW"

    /** The route a `ui://florisboard/…` link names, or null for any other intent. */
    fun linkRoute(action: String?, data: String?): String? =
        data?.takeIf { action == VIEW && it.startsWith(LINK) }?.removePrefix(LINK)

    /**
     * Whether the Rust host opens. Once the keyboard is set up it always does; before, Setup
     * shows unless a settings link asks for a screen. The fallback Activity never opens it.
     */
    fun rust(allowRust: Boolean, imeSetUp: Boolean, linkRoute: String?): Boolean =
        allowRust && (imeSetUp || linkRoute != null)
}
