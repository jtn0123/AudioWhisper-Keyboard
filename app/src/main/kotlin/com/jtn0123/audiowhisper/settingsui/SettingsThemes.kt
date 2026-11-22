package com.jtn0123.audiowhisper.settingsui

import androidx.activity.ComponentActivity
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.extensionManager
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import dev.patrickgold.florisboard.themeManager

/** What the native theme manager needs from the keyboard: the installed theme extensions as one `THEMES` report,
 * and the keyboard's preview theme. Both live in the extension and theme managers. Rust groups, sorts and draws the
 * themes and writes the chosen one itself. */
internal class SettingsThemes(activity: ComponentActivity, private val report: (Array<String>) -> Unit) {
    private val extensionManager by activity.extensionManager()
    private val themeManager by activity.themeManager()
    private var wanted = false
    init {
        // Once asked, a theme imported, made or deleted in the Compose extension screens reaches the manager too.
        extensionManager.themes.observe(activity) { if (wanted) report(rows()) }
    }
    /** A `themes` or `themePreview` effect. */
    fun execute(p: List<String>) {
        when (p[0]) {
            SettingsUiThemeEffect.LOAD -> { wanted = true; report(rows()) }
            SettingsUiThemeEffect.PREVIEW -> themeManager.previewThemeId.value = p[1].takeIf { it.isNotEmpty() }
                ?.let { runCatching { ExtensionComponentName.from(it) }.getOrNull() }
        }
    }
    /** The settings view is gone: the keyboard shows its own theme again, as leaving ThemeManagerScreen did. */
    fun release() { themeManager.previewThemeId.value = null }
    /** Each extension in the index's order, named as ThemeManagerScreen named it, then its themes. */
    private fun rows(): Array<String> {
        val out = mutableListOf<String>()
        for (ext in extensionManager.themes.value.orEmpty()) {
            val found = extensionManager.getExtensionById(ext.meta.id) ?: continue
            out += themeRow(SettingsUiThemeKind.EXTENSION, ext.meta.id, label = found.meta.title,
                deletable = extensionManager.canDelete(found))
            for (theme in ext.themes) out += themeRow(SettingsUiThemeKind.THEME, ext.meta.id, theme.id, theme.label, theme.isNightTheme)
        }
        return out.toTypedArray()
    }
}

/** A report row: an extension, or one of its themes. */
internal fun themeRow(kind: String, extension: String, component: String = "", label: String, night: Boolean = false,
    deletable: Boolean = false) = SettingsUiTheme.row(SettingsUiTheme.KIND to kind, SettingsUiTheme.EXTENSION to extension,
    SettingsUiTheme.COMPONENT to component, SettingsUiTheme.LABEL to label, SettingsUiTheme.NIGHT to WireFlag.encode(night),
    SettingsUiTheme.DELETABLE to WireFlag.encode(deletable))
