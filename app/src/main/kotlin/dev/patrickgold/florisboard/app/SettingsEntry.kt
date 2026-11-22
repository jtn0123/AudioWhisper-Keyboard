package dev.patrickgold.florisboard.app

import com.jtn0123.audiowhisper.rust.SettingsUiExtensionLink

/**
 * The settings link an intent carries, or the extension file it hands over. Rust draws settings,
 * setup and the importer and picks the first screen from this (`SettingsUiOp.START`); Compose
 * draws only the screens Rust hands to [ComposeSettingsFallbackActivity].
 */
internal object SettingsEntry {
    private const val LINK = "ui://florisboard/"
    private const val VIEW = "android.intent.action.VIEW"
    private const val SEND = "android.intent.action.SEND"

    /** The route a `ui://florisboard/…` link names, or null for any other intent. */
    fun linkRoute(action: String?, data: String?): String? =
        data?.takeIf { action == VIEW && it.startsWith(LINK) }?.removePrefix(LINK)

    /**
     * Where an intent opens settings: a settings link's route, or the importer reading the document
     * a VIEW intent opens ([view], its data) or a SEND intent shares ([shared], its clip's first
     * item, where Android puts `EXTRA_STREAM`), kept by [keep] for Rust to ask for. Null otherwise.
     */
    fun <D> route(action: String?, data: String?, view: D?, shared: D?, keep: (D) -> String): String? {
        val document = when (action) {
            VIEW -> view
            SEND -> shared
            else -> null
        }
        return linkRoute(action, data) ?: document?.let { SettingsUiExtensionLink.SHARED + keep(it) }
    }
}
