package com.jtn0123.audiowhisper.settingsui

import androidx.activity.ComponentActivity
import androidx.lifecycle.lifecycleScope
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.cacheManager
import dev.patrickgold.florisboard.extensionManager
import dev.patrickgold.florisboard.ime.nlp.LanguagePackExtension
import dev.patrickgold.florisboard.ime.theme.ThemeExtension
import dev.patrickgold.florisboard.lib.ext.Extension
import dev.patrickgold.florisboard.lib.io.FlorisRef
import kotlinx.coroutines.launch
import org.florisboard.lib.android.showLongToast

/** What the native extension screens need from the keyboard: the installed extensions as one `EXTENSIONS` report, in
 * `combinedExtensionList()` order, and deleting one, both from the extension manager; and what an import workspace
 * holds, and closing one, from the cache. Rust picks what each screen lists, builds the update link and draws them. */
internal class SettingsExtensions(private val activity: ComponentActivity, private val send: (SettingsUiOp, Array<String>) -> Unit) {
    private val extensionManager by activity.extensionManager()
    private val cacheManager by activity.cacheManager()
    private var wanted = false
    init {
        // Once asked, an extension imported, made or deleted anywhere reaches the screens too.
        for (index in listOf(extensionManager.keyboardExtensions, extensionManager.themes, extensionManager.languagePacks)) {
            index.observe(activity) { if (wanted) send(SettingsUiOp.EXTENSIONS, rows()) }
        }
    }
    /** A `SettingsUiExtensionEffect`. */
    fun execute(p: List<String>) {
        when (p[0]) {
            SettingsUiExtensionEffect.LOAD -> { wanted = true; send(SettingsUiOp.EXTENSIONS, rows()) }
            SettingsUiExtensionEffect.DELETE -> delete(p[1])
            SettingsUiExtensionEffect.FILES -> send(SettingsUiOp.EXTENSION_FILES, importRows(activity, p[1]))
            SettingsUiExtensionEffect.CLOSE -> cacheManager.importer.getWorkspaceByUuid(p[1])?.close()
        }
    }
    /** As ExtensionViewScreen deleted: a failure is a long toast and the page stays. One already gone is done. */
    private fun delete(id: String) {
        val ext = extensionManager.getExtensionById(id)
        val result = runCatching { if (ext != null) extensionManager.delete(ext) }
        result.onFailure { error ->
            activity.lifecycleScope.launch {
                activity.showLongToast(R.string.error__snackbar_message, "error_message" to error.localizedMessage)
            }
        }
        send(SettingsUiOp.EXTENSION_DONE, SettingsUiExtensionDone.row(SettingsUiExtensionDone.WORK to SettingsUiExtensionWork.DELETE,
            SettingsUiExtensionDone.EXTENSION to id, SettingsUiExtensionDone.DONE to WireFlag.encode(result.isSuccess)))
    }
    private fun rows(): Array<String> {
        val all = listOf(extensionManager.keyboardExtensions, extensionManager.themes, extensionManager.languagePacks)
        return all.flatMap { it.value.orEmpty() }
            .flatMap { extensionRows(it, extensionManager.canDelete(it)) { link -> FlorisRef.fromUrl(link).authority } }
            .flatMap { it.asList() }.toTypedArray()
    }
}

/** An extension's report rows: the extension, then what its page shows of its meta and components. A null value
 * sends no row. [authority] names a link's host. */
internal fun extensionRows(ext: Extension, deletable: Boolean, authority: (String) -> String): List<Array<String>> {
    val id = ext.meta.id
    val meta = ext.meta
    val out = mutableListOf(extensionRow(SettingsUiExtensionKind.EXTENSION, id, ext.serialType(), meta.title, meta.version,
        meta.license, deletable))
    meta.description?.let { out += extensionRow(SettingsUiExtensionKind.DESCRIPTION, id, text = it) }
    for (m in meta.maintainers) {
        out += extensionRow(SettingsUiExtensionKind.MAINTAINER, id, label = m.name)
        m.email?.let { out += extensionRow(SettingsUiExtensionKind.EMAIL, id, text = it) }
        m.url?.let { out += extensionRow(SettingsUiExtensionKind.URL, id, text = it) }
    }
    for (k in meta.keywords.orEmpty()) out += extensionRow(SettingsUiExtensionKind.KEYWORD, id, label = k)
    for ((kind, link) in listOf(SettingsUiExtensionKind.HOMEPAGE to meta.homepage, SettingsUiExtensionKind.ISSUE_TRACKER to meta.issueTracker)) {
        if (link != null) out += extensionRow(kind, id, detail = if (link.isBlank()) "" else authority(link), text = link)
    }
    val components = when (ext) {
        is ThemeExtension -> ext.themes.map { Triple(it, it.stylesheetPath(), it.isNightTheme) }
        is LanguagePackExtension -> ext.items.map { Triple(it, it.locale.localeTag(), false) }
        else -> emptyList()
    }
    for ((c, detail, night) in components) {
        out += extensionRow(SettingsUiExtensionKind.COMPONENT, id, c.id, c.label, detail, flag = night)
        for (a in c.authors) out += extensionRow(SettingsUiExtensionKind.AUTHOR, id, label = a)
    }
    return out
}

private fun extensionRow(kind: String, extension: String, id: String = "", label: String = "", detail: String = "",
    text: String = "", flag: Boolean = false) = SettingsUiExtension.row(SettingsUiExtension.KIND to kind,
    SettingsUiExtension.EXTENSION to extension, SettingsUiExtension.ID to id, SettingsUiExtension.LABEL to label,
    SettingsUiExtension.DETAIL to detail, SettingsUiExtension.TEXT to text, SettingsUiExtension.FLAG to WireFlag.encode(flag))
